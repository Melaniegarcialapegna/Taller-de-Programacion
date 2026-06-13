use crate::logger::Logger;
/// Este módulo contiene la implementación del loop principal de SCTP, que se encarga de:
/// - Leer datos desde la red (a través de DTLS)
/// - Procesar eventos de SCTP (como mensajes recibidos, cambios de estado, etc.)
/// - Enviar datos pendientes por SCTP a través de DTLS
/// - Manejar el estado del handshake de SCTP y del canal DCEP
///
/// El loop se ejecuta en un hilo separado y se comunica con el resto de la aplicación a través de canales (Sender/Receiver) para enviar eventos hacia
/// afuera y recibir datos salientes que deben ser enviados por SCTP.
use crate::protocolos::sctp::dcep::dcep_handshake::{
    EstadoDcep, iniciar_canal, procesar_mensaje_dcep,
};
use crate::protocolos::sctp::estado_sctp::EstadoSctp;
use crate::protocolos::sctp::evento_sctp::EventoSctp;
use crate::protocolos::sctp::handshake_sctp::{EstadoHandshakeSctp, avanzar_handshake_sctp};
use crate::protocolos::sctp::io_sctp::{
    drenar_eventos_app, drenar_transmisiones, recibir_desde_red, sincronizar_endpoint_y_asociacion,
};
use crate::protocolos::sctp::protocolo_archivo::MensajeArchivo;
use crate::rtc::transporte::demux::canal_demux::DemuxDtlsChannel;
use bytes::{Bytes, BytesMut};
use sctp_proto::{Association, Event, Payload, PayloadProtocolIdentifier, StreamEvent, StreamId};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;
use udp_dtls::DtlsStream;

fn leer_mensajes(
    mut mensajes: Vec<(StreamId, PayloadProtocolIdentifier, Bytes)>,
    stream_ids: Vec<StreamId>,
    asoc: &mut Association,
) -> Vec<(StreamId, PayloadProtocolIdentifier, Bytes)> {
    for stream_id in stream_ids {
        let Ok(mut stream) = asoc.stream(stream_id) else {
            continue;
        };
        while let Ok(Some(mut chunks)) = stream.read_sctp() {
            let ppi = chunks.ppi;
            let mut data = BytesMut::new();
            while let Some(chunk) = chunks.next(65535) {
                data.extend_from_slice(&chunk.bytes);
            }
            mensajes.push((stream_id, ppi, data.freeze()));
        }
    }
    mensajes
}

fn procesar_mensajes(
    mensajes: Vec<(StreamId, PayloadProtocolIdentifier, Bytes)>,
    estado_dcep: &mut EstadoDcep,
    asoc: &mut Association,
    tx_eventos: &Sender<EventoSctp>,
    logger: &Logger,
) {
    logger.info(
        &format!("procesar_mensajes: {} msgs", mensajes.len()),
        "SCTP loop",
    );
    for (sid, ppi, data) in mensajes {
        logger.info(
            &format!("msg sid={sid} ppi={ppi:?} len={}", data.len()),
            "SCTP loop",
        );
        match ppi {
            PayloadProtocolIdentifier::Dcep => {
                procesar_mensaje_dcep(sid, data, estado_dcep, asoc);
            }
            PayloadProtocolIdentifier::Binary => match MensajeArchivo::deserializar(&data) {
                Ok(mensaje) => {
                    let evento = EventoSctp::from(mensaje);
                    logger.info(
                        &format!("[SCTP loop] -> EventoSctp: {:?}", evento),
                        "SCTP loop",
                    );
                    if let Err(e) = tx_eventos.send(evento) {
                        logger.error(
                            &format!("[SCTP loop] Error enviando evento hacia afuera: {e}"),
                            "SCTP loop",
                        );
                    } else {
                        logger.info("Evento enviado al receiver externo", "SCTP loop");
                    }
                }
                Err(e) => {
                    logger.error(
                        &format!("[SCTP loop] Error deserializando mensaje binario: {e}"),
                        "SCTP loop",
                    );
                }
            },
            PayloadProtocolIdentifier::String => {
                logger.info(
                    &format!("[SCTP] Datos de tipo String en stream {sid} (no soportado)"),
                    "SCTP loop",
                );
            }
            _ => {}
        }
    }
}

fn procesar_eventos(
    eventos: Vec<Event>,
    estado_sctp: &mut EstadoSctp,
    estado_dcep: &mut EstadoDcep,
    tx_eventos: &Sender<EventoSctp>,
    logger: &Logger,
) {
    let mut stream_ids_a_leer: Vec<u16> = eventos
        .iter()
        .filter_map(|e| match e {
            Event::Stream(StreamEvent::Readable { id }) => Some(*id),
            _ => None,
        })
        .collect();

    if stream_ids_a_leer.is_empty() {
        let fallback: u16 = match estado_dcep {
            EstadoDcep::Abierto { stream_id } => *stream_id,
            EstadoDcep::EsperandoAck { stream_id } => *stream_id,
            _ => 0,
        };
        stream_ids_a_leer.push(fallback);
    }

    if let Some(asoc) = estado_sctp.asociacion.as_mut() {
        let mensajes_vec: Vec<(StreamId, PayloadProtocolIdentifier, Bytes)> = Vec::new();
        let mensajes = leer_mensajes(mensajes_vec, stream_ids_a_leer, asoc);
        procesar_mensajes(mensajes, estado_dcep, asoc, tx_eventos, logger);
    }
}

fn iniciar_canal_si_corresponde(
    estado_handshake: &EstadoHandshakeSctp,
    estado_dcep: &mut EstadoDcep,
    es_dtls_client: bool,
    estado_sctp: &mut EstadoSctp,
    label_canal: &str,
    logger: &Logger,
) {
    if matches!(estado_handshake, EstadoHandshakeSctp::Establecido)
        && matches!(estado_dcep, EstadoDcep::Inactivo)
        && es_dtls_client
        && let Some(asoc) = estado_sctp.asociacion.as_mut()
    {
        match iniciar_canal(asoc, es_dtls_client, label_canal) {
            Ok(sid) => {
                logger.info(
                    &format!("[DCEP] Canal DCEP iniciado en stream {sid}, esperando ACK..."),
                    "SCTP loop",
                );
                *estado_dcep = EstadoDcep::EsperandoAck { stream_id: sid };
            }
            Err(e) => logger.error(&format!("[DCEP] Error iniciando canal: {e}"), "SCTP loop"),
        }
    }
}

fn drenar_y_enviar_sctp_por_dtls(
    stream_dtls: &mut DtlsStream<DemuxDtlsChannel>,
    estado_sctp: &mut EstadoSctp,
    now: Instant,
    contexto: &str,
    label_canal: &str,
    logger: &Logger,
) -> std::io::Result<()> {
    let transmisiones = drenar_transmisiones(estado_sctp, now);

    for transmit in transmisiones {
        let Payload::RawEncode(chunks) = transmit.payload else {
            continue;
        };

        stream_dtls.write_all(&chunks.concat()).map_err(|e| {
            logger.error(
                &format!("[SCTP loop][{label_canal}] Error enviando {contexto} por DTLS: {e:?}"),
                "SCTP loop",
            );
            e
        })?;
    }

    Ok(())
}

fn manejar_eventos(
    estado_sctp: &mut EstadoSctp,
    estado_handshake: &mut EstadoHandshakeSctp,
    estado_dcep: &mut EstadoDcep,
    es_dtls_client: bool,
    label_canal: &str,
    tx_eventos: &Sender<EventoSctp>,
    logger: &Logger,
) {
    let eventos = drenar_eventos_app(estado_sctp);

    if matches!(estado_handshake, EstadoHandshakeSctp::Conectando) {
        avanzar_handshake_sctp(estado_handshake, &eventos, logger);
    }

    iniciar_canal_si_corresponde(
        estado_handshake,
        estado_dcep,
        es_dtls_client,
        estado_sctp,
        label_canal,
        logger,
    );

    logger.info(
        &format!("[SCTP loop] Eventos a procesar: {eventos:?}"),
        "SCTP loop",
    );
    procesar_eventos(eventos, estado_sctp, estado_dcep, tx_eventos, logger);
}

fn enviar_datos_pendientes(
    rx_datos_salientes: &Receiver<Bytes>,
    estado_dcep: &EstadoDcep,
    estado_sctp: &mut EstadoSctp,
    logger: &Logger,
) {
    while let Ok(datos) = rx_datos_salientes.try_recv() {
        match estado_dcep {
            EstadoDcep::Abierto { stream_id } => {
                let Some(asoc) = estado_sctp.asociacion.as_mut() else {
                    logger.error(
                        "[SCTP loop] Quise enviar datos pero no hay asociación activa",
                        "SCTP loop",
                    );
                    break;
                };
                match asoc.stream(*stream_id) {
                    Ok(mut stream) => {
                        if let Err(e) = stream.write_sctp(&datos, PayloadProtocolIdentifier::Binary)
                        {
                            logger.error(&format!("[SCTP loop] Error escribiendo datos en stream {stream_id}: {e:?}"), "SCTP loop");
                        } else {
                            // Datos escritos exitosamente
                        }
                    }
                    Err(e) => {
                        logger.error(
                            &format!("[SCTP loop] No se pudo obtener stream {stream_id}: {e:?}"),
                            "SCTP loop",
                        );
                    }
                }
            }
            _ => {
                logger.info(&format!(
                    "[SCTP loop] Se intentó enviar datos pero el canal DCEP no está abierto (estado: {:?}). Mensaje descartado.",
                    estado_dcep
                ), "SCTP loop");
            }
        }
    }
}

fn procesar_pre_loop(
    stream_dtls: &mut DtlsStream<DemuxDtlsChannel>,
    label_canal: &str,
    estado_sctp: &mut EstadoSctp,
    logger: &Logger,
) -> bool {
    let now = Instant::now();
    drenar_y_enviar_sctp_por_dtls(
        stream_dtls,
        estado_sctp,
        now,
        "transmisiones iniciales",
        label_canal,
        logger,
    )
    .is_ok()
}

/// Spawnea el loop principal de SCTP en un thread separado. Este loop se encarga de manejar toda la lógica de SCTP.
///
/// Parámetros:
/// - `stream_dtls`: El stream DTLS a través del cual se enviarán y recibirán los datos SCTP.
/// - `estado_sctp`: El estado inicial de SCTP, que será modificado por el loop a medida que se procesen eventos y mensajes.
/// - `estado_handshake`: El estado del handshake de SCTP, que se actualizará a medida que avance el proceso de establecimiento de la asociación.
/// - `estado_dcep`: El estado del canal DCEP, que se actualizará a medida que se inicie el canal y se reciban mensajes DCEP.
/// - `es_dtls_client`: Indica si este extremo es el cliente DTLS, lo cual afecta la lógica de inicio del canal DCEP.
/// - `label_canal`: Una etiqueta descriptiva para el canal, utilizada en los logs para identificar a qué canal se refieren los mensajes.
/// - `remote`: La dirección remota del peer, utilizada para identificar la fuente de los mensajes recibidos.
///
/// Retorna:
/// - Un `JoinHandle` del thread donde se ejecuta el loop, que puede ser utilizado para esperar a que el thread termine o para otras operaciones de manejo de threads.
/// - Un `Sender<Bytes>` que puede ser utilizado por otras partes de la aplicación para enviar datos que deben ser transmitidos por SCTP.
/// - Un `Receiver<EventoSctp>` a través del cual se recibirán eventos de SCTP que ocurran dentro del loop, como mensajes recibidos o cambios de estado.
pub fn spawnear_loop_sctp(
    mut stream_dtls: DtlsStream<DemuxDtlsChannel>,
    estados: (EstadoSctp, EstadoHandshakeSctp, EstadoDcep),
    es_dtls_client: bool,
    label_canal: String,
    remote: SocketAddr,
    logger: Logger,
) -> (thread::JoinHandle<()>, Sender<Bytes>, Receiver<EventoSctp>) {
    let (tx_datos_salientes, rx_datos_salientes) = mpsc::channel::<Bytes>();
    let (tx_eventos, rx_eventos) = mpsc::channel::<EventoSctp>();
    let (mut estado_sctp, mut estado_handshake, mut estado_dcep) = estados;

    let handle = thread::spawn(move || {
        if !procesar_pre_loop(&mut stream_dtls, &label_canal, &mut estado_sctp, &logger) {
            return;
        }

        let mut buf = [0u8; 65535];

        loop {
            let now = Instant::now();

            match stream_dtls.read(&mut buf) {
                Ok(0) => {
                    logger.info(
                        "Conexión cerrada por el peer, finalizando loop",
                        "SCTP loop",
                    );
                    EstadoSctp::finalizar_llamada_sctp(&mut estado_sctp);
                    break;
                }
                Ok(n) => {
                    let data = Bytes::copy_from_slice(&buf[..n]);
                    if let Err(e) = recibir_desde_red(&mut estado_sctp, now, remote, None, data) {
                        logger.error(&format!("Error procesando datos recibidos por DTLS: {e:?}, descartando paquete"), "SCTP loop");
                        continue;
                    }
                    manejar_eventos(
                        &mut estado_sctp,
                        &mut estado_handshake,
                        &mut estado_dcep,
                        es_dtls_client,
                        &label_canal,
                        &tx_eventos,
                        &logger,
                    );
                    enviar_datos_pendientes(
                        &rx_datos_salientes,
                        &estado_dcep,
                        &mut estado_sctp,
                        &logger,
                    );
                    sincronizar_endpoint_y_asociacion(&mut estado_sctp);
                    if drenar_y_enviar_sctp_por_dtls(
                        &mut stream_dtls,
                        &mut estado_sctp,
                        now,
                        "transmisión",
                        &label_canal,
                        &logger,
                    )
                    .is_err()
                    {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Es el llamado "tick" de SCTP, es necesario porque SCTP tiene timeouts internos para retransmisiones, etc. que deben ser
                    // manejados aunque no haya datos entrantes. -> Se habla de esto cuando se menciona que SCTP tiene estados internos, habla de
                    // hearbeats, ticks, retransmisiones, etc que necesitan avanzar incluso si no hay datos entrantes.

                    // Lo accionamos con WouldBlock porque el stream DTLS está en modo no bloqueante, entonces cuando no hay datos para leer,
                    // en vez de bloquear el thread, devuelve este error, lo cual es una señal perfecta para ejecutar la lógica de "tick" de SCTP.
                    let now = Instant::now();
                    enviar_datos_pendientes(
                        &rx_datos_salientes,
                        &estado_dcep,
                        &mut estado_sctp,
                        &logger,
                    );
                    if let Some(asoc) = estado_sctp.asociacion.as_mut() {
                        asoc.handle_timeout(now);
                    }
                    sincronizar_endpoint_y_asociacion(&mut estado_sctp);
                    if drenar_y_enviar_sctp_por_dtls(
                        &mut stream_dtls,
                        &mut estado_sctp,
                        now,
                        "timeout flush",
                        &label_canal,
                        &logger,
                    )
                    .is_err()
                    {
                        break;
                    }
                }
                Err(_) => {
                    logger.error("Error leyendo desde DTLS, finalizando loop", "SCTP loop");
                    EstadoSctp::finalizar_llamada_sctp(&mut estado_sctp);
                    break;
                }
            }
        }
    });

    (handle, tx_datos_salientes, rx_eventos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocolos::sctp::dcep::dcep_handshake::EstadoDcep;
    use crate::protocolos::sctp::estado_sctp::EstadoSctp;
    use crate::protocolos::sctp::handshake_sctp::EstadoHandshakeSctp;
    use crate::protocolos::sctp::rol_sctp::RolConexion;
    use bytes::Bytes;
    use sctp_proto::EndpointConfig;
    use std::sync::{Arc, mpsc};

    fn estado_sin_asociacion() -> EstadoSctp {
        EstadoSctp::inicializar_sctp(
            RolConexion::Inicia,
            Arc::new(EndpointConfig::default()),
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_enviar_datos_pendientes_descarta_si_dcep_no_abierto() {
        // Si el canal DCEP no está abierto, los datos deben descartarse sin panic
        let (tx, rx) = mpsc::channel::<Bytes>();
        let mut estado = estado_sin_asociacion();
        let logger = Logger::dummy_logger();

        tx.send(Bytes::from_static(b"hola")).unwrap();

        enviar_datos_pendientes(&rx, &EstadoDcep::Inactivo, &mut estado, &logger);
    }

    #[test]
    fn test_enviar_datos_pendientes_canal_vacio_no_hace_nada() {
        let (_tx, rx) = mpsc::channel::<Bytes>();
        let mut estado = estado_sin_asociacion();
        let logger = Logger::dummy_logger();
        enviar_datos_pendientes(&rx, &EstadoDcep::Inactivo, &mut estado, &logger);
    }

    #[test]
    fn test_iniciar_canal_si_corresponde_no_hace_nada_si_no_establecido() {
        let mut estado = estado_sin_asociacion();
        let mut estado_dcep = EstadoDcep::Inactivo;
        let logger = Logger::dummy_logger();

        iniciar_canal_si_corresponde(
            &EstadoHandshakeSctp::Conectando,
            &mut estado_dcep,
            true,
            &mut estado,
            "test-canal",
            &logger,
        );

        assert!(matches!(estado_dcep, EstadoDcep::Inactivo));
    }
}
