//! Comunicación RTCP

use super::error::ErrorComunicacionRTCP;
use super::sesion::{EstadisticasReceiver, EstadisticasSender};
use super::socket_udp::SocketUDP;
use crate::llamada::sesion_rtp::MensajeFinalizarLlamada;
use crate::logger::Logger;
use crate::protocolos::rtcp::tipo_paquete::ContenidoReport;
use crate::protocolos::rtcp::{
    paquete::PaqueteRTCP,
    tipo_paquete::{ContenidoPaqueteRTCP, ContenidoSenderReport},
};
use crate::sesion_rtp::sesion::ByeChannels;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Tamanio del buffer donde se almacenaran los mensajes RTCP recibidos
const TAMANIO_BUFFER_MENSAJES: usize = 256;

/// Intervalo minimo entre mensajes RTCP (definido por el estandar RFC 1889)
const ESPERA_MINIMA_ENVIO_PAQUETES_RTCP: f64 = 5.0;

// Constantes que deberian usarse para hacer el calculo de la frecuencia de mensajes RTCP
// const FRACCION_ANCHO_DE_BANDA_SENDERS: f64 = 0.25;
// const FRACCION_ANCHO_DE_BANDA_RECEIVERS: f64 = 1.0-FRACCION_ANCHO_DE_BANDA_SENDERS;
// const GANANCIA_ESTIMACION_TAMANIO_PAQUETES: f64 = 1.0/16.0;
// const ANCHO_DE_BANDA_SESION: f64 = 5.0; //En mbps
// const ANCHO_DE_BANDA_RTCP: f64 = 0.05 * ANCHO_DE_BANDA_SESION; //Por convención, RTCP usa el 5% del ancho de banda

/// Inicia la comunicación por RTCP usando el socket recibido por parametro.
///
/// Crea dos threads para hacer la comunicación:
/// - Un thread donde se escucharan los mensajes y se procesaran
/// - Un thread donde se mandaran paquetes periodicamente
///
/// IMPORTANTE: Una vez creados los dos threads termina la ejecución de la función. Esta función NO toma el main thread.
pub fn iniciar_comunicacion_rtcp(
    logger: Logger,
    mut socket: Box<dyn SocketUDP>,
    direccion_receptor: &str,
    estadisticas_sender: Arc<Mutex<EstadisticasSender>>,
    estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    bye_channel: ByeChannels,
    tx_estadisticas: Sender<EstadisticasReceiver>,
) -> Result<(), ErrorComunicacionRTCP> {
    let (rx_mensaje_bye, tx_cortar_llamada) = bye_channel;

    let socket_lectura = socket
        .clonar()
        .map_err(|_| ErrorComunicacionRTCP::ErrorClonandoSocket)?;

    let socket_escritura = socket
        .clonar()
        .map_err(|_| ErrorComunicacionRTCP::ErrorClonandoSocket)?;

    let estadisticas_receiver_escucha = Arc::clone(&estadisticas_receiver);
    let estadisticas_receiver_escritura = Arc::clone(&estadisticas_receiver);
    let estadisticas_sender_escritura = Arc::clone(&estadisticas_sender);
    let estadisticas_sender_lectura = Arc::clone(&estadisticas_sender);

    let termino = Arc::new(Mutex::new(false));
    let clon_termino = Arc::clone(&termino);

    let direccion = String::from(direccion_receptor);

    //Hilo que cambia booleano cuando finaliza llamada
    thread::spawn(move || {
        let mensaje = match rx_mensaje_bye.recv() {
            Ok(msg) => msg,
            Err(_) => return,
        };

        match mensaje {
            MensajeFinalizarLlamada::CortamosNosotros => {
                let mut lock_termino = match clon_termino.lock() {
                    Ok(lock) => lock,
                    Err(_) => return,
                };
                //solo si somos nosotros los que cortamos
                //se envia paquete bye a los ssrc activos y se sale
                if cerrar_hilo(
                    estadisticas_sender,
                    socket,
                    estadisticas_receiver,
                    &direccion,
                )
                .is_err()
                {
                    eprintln!("Error al enviar mensaje bye");
                };
                *lock_termino = true;
            }
            MensajeFinalizarLlamada::CortoElOtroPeer => {
                //unicamente le avisamos al hilo que debe salir
                let mut lock_termino = match clon_termino.lock() {
                    Ok(lock) => lock,
                    Err(_) => return,
                };

                *lock_termino = true;
            }
        }
    });

    let logger_clon = logger.clone();

    thread::spawn(move || {
        logger.info("Iniciando hilo escucha paquetes RTCP", "Comunicacion RTCP");
        if iniciar_escucha(
            socket_lectura,
            estadisticas_receiver_escucha,
            tx_cortar_llamada,
            estadisticas_sender_lectura,
            tx_estadisticas,
        )
        .is_err()
        {
            let mensaje_error = String::from(ErrorComunicacionRTCP::ErrorIniciandoConexion);
            eprintln!("{mensaje_error}");
        }
        logger.info(
            "Finalizando hilo escucha paquetes RTCP",
            "Comunicacion RTCP",
        );
    });

    let direccion = String::from(direccion_receptor);

    thread::spawn(move || {
        logger_clon.info("Iniciando hilo envio paquetes RTCP", "Comunicacion RTCP");
        if iniciar_envio_mensajes_rtcp(
            socket_escritura,
            direccion.as_str(),
            estadisticas_receiver_escritura,
            estadisticas_sender_escritura,
            termino,
        )
        .is_err()
        {
            let mensaje_error = String::from(ErrorComunicacionRTCP::ErrorIniciandoConexion);
            eprintln!("{mensaje_error}");
        }
        logger_clon.info("Finalizando hilo envio paquetes RTCP", "Comunicacion RTCP");
    });

    Ok(())
}

/// Inicia la escucha de mensajes RTCP en el main thread actual. Debera crearse un thread nuevo
/// antes de invocar esta función, ya que es bloqueante.
fn iniciar_escucha(
    mut socket: Box<dyn SocketUDP>,
    dicc_estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    tx_cortar_llamada: Sender<MensajeFinalizarLlamada>,
    _estadisticas_sender_lock: Arc<Mutex<EstadisticasSender>>,
    tx_estadisticas: Sender<EstadisticasReceiver>,
) -> Result<(), ErrorComunicacionRTCP> {
    let mut buffer: [u8; TAMANIO_BUFFER_MENSAJES] = [0; TAMANIO_BUFFER_MENSAJES];

    let mensaje = mensaje_finalizacion_llamada().to_vec();

    loop {
        let (tamanio_mensaje, _) = socket
            .recibir(&mut buffer[..])
            .map_err(|_| ErrorComunicacionRTCP::ErrorRecibiendoMensaje)?;

        let bytes_recibidos = &buffer[..tamanio_mensaje];

        if bytes_recibidos == mensaje {
            return Ok(());
        }

        let mut prox_indice_por_leer = 0;

        // Un mismo datagrama puede incluir muchos paquetes RTCP. Leo de a un paquete hasta haber leido todos los bytes
        while prox_indice_por_leer < tamanio_mensaje {
            let paquete_rtcp =
                PaqueteRTCP::try_from(&buffer[prox_indice_por_leer..tamanio_mensaje])
                    .map_err(|_| ErrorComunicacionRTCP::PaqueteInvalidoRecibido)?;

            let ssrc_paquete = paquete_rtcp.ssrc;

            let mut dicc_estadisticas_receiver_lock = dicc_estadisticas_receiver
                .lock()
                .map_err(|_| ErrorComunicacionRTCP::ErrorRecibiendoMensaje)?;

            if let ContenidoPaqueteRTCP::Bye() = paquete_rtcp.payload {
                //SE QUEDA ESPERANDO EL LOCK

                //Se comenta pq al ser solo entre dos peers se sale cuando el otro nos corta
                // let mut estadisticas_receiver = dicc_estadisticas_receiver
                // .lock()
                // .map_err(|_| ErrorComunicacionRTCP::ErrorFinalizandoConexionConPeer)?;

                // eprintln!("-----ENTRA A PAQUETE BYE----3");
                // estadisticas_receiver.remove_entry(&ssrc_paquete);

                tx_cortar_llamada
                    .send(MensajeFinalizarLlamada::CortoElOtroPeer)
                    .map_err(|_| {
                        eprintln!("\nerror aca\n");
                        ErrorComunicacionRTCP::ErrorFinalizandoConexionConPeer
                    })?;

                return Ok(());
            }

            if let ContenidoPaqueteRTCP::ReceiverReport(_) = paquete_rtcp.payload {
                let estadisticas = EstadisticasReceiver::new(ssrc_paquete);
                dicc_estadisticas_receiver_lock
                    .entry(ssrc_paquete)
                    .or_insert(estadisticas);
                // hacer lo que sea necesario con ese reporte sobre lo que estoy mandando
            }

            if let ContenidoPaqueteRTCP::SenderReport(contenido_sender_externo) =
                paquete_rtcp.payload
            {
                dicc_estadisticas_receiver_lock
                    .entry(ssrc_paquete)
                    .or_insert(EstadisticasReceiver::new(ssrc_paquete));

                let estadisticas_receiver = dicc_estadisticas_receiver_lock
                    .get_mut(&ssrc_paquete)
                    .ok_or(ErrorComunicacionRTCP::ErrorRecibiendoMensaje)?;

                let fraccion_perdidas = calcular_fraccion_paquetes_perdidos(
                    &contenido_sender_externo,
                    estadisticas_receiver,
                );
                let contenido_report = &mut estadisticas_receiver.contenido_report;

                contenido_report.tiempo_desde_ultimo_paquete =
                    contenido_sender_externo.rtp_timestamp;
                contenido_report.frac_paquetes_perdidos = fraccion_perdidas;
                contenido_report.cant_paquetes_perdidos = calcular_cant_paquetes_perdidos(
                    contenido_sender_externo.sender_packet_count,
                    estadisticas_receiver.cantidad_paquetes_recibidos,
                );

                tx_estadisticas
                    .send(estadisticas_receiver.clone())
                    .map_err(|_| ErrorComunicacionRTCP::ErrorRecibiendoMensaje)?;
            }

            prox_indice_por_leer += usize::from(paquete_rtcp.longitud_paquete);
        }
    }
}

/// Devuelve la fracción de paquetes perdidos según las `EstadisticasReceiver` de un
/// peer especifico. Las devuelve en representación de punto fijo, con la primera coma antes del primer bit (es decir, 128 representa 1/2).
fn calcular_fraccion_paquetes_perdidos(
    contenido_sender: &ContenidoSenderReport,
    estadisticas_receiver: &mut EstadisticasReceiver,
) -> u8 {
    // Calculo la cantidad de paquetes esperados desde el anterior paquete SenderReport a este
    let esperados_en_intervalo = u32::wrapping_sub(
        contenido_sender.sender_packet_count,
        estadisticas_receiver.cantidad_paquetes_esperados_anterior,
    );
    estadisticas_receiver.cantidad_paquetes_esperados_anterior =
        contenido_sender.sender_packet_count;

    // Calculo la cantidad de paquetes recibidos desde el anterior paquete SenderReport a este
    let recibidos_en_intervalo = u32::wrapping_sub(
        estadisticas_receiver.cantidad_paquetes_recibidos,
        estadisticas_receiver.cantidad_paquetes_recibidos_anterior,
    );
    estadisticas_receiver.cantidad_paquetes_recibidos_anterior =
        estadisticas_receiver.cantidad_paquetes_recibidos;

    // Calculo la fracción de paquetes perdidos (esperados/recibidos)
    let perdida_en_intervalo = u32::wrapping_sub(esperados_en_intervalo, recibidos_en_intervalo);

    if esperados_en_intervalo > 0 {
        ((perdida_en_intervalo << 8) / esperados_en_intervalo) as u8
    } else {
        0
    }
}

/// Devuelve la cantidad de paquetes perdidos en 24 bits.
fn calcular_cant_paquetes_perdidos(cant_paquetes_mandados: u32, cant_recibidos: u32) -> u32 {
    u32::wrapping_sub(cant_paquetes_mandados, cant_recibidos).min(0xffffff)
}

/// Inicia el envio de mensajes RTCP en el main thread actual. Debera crearse un thread nuevo
/// antes de invocar esta función, ya que es bloqueante.
fn iniciar_envio_mensajes_rtcp(
    mut socket: Box<dyn SocketUDP>,
    direccion_rtcp: &str,
    dicc_estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    estadisticas_sender: Arc<Mutex<EstadisticasSender>>,
    termino: Arc<Mutex<bool>>,
) -> Result<(), ErrorComunicacionRTCP> {
    let socket_clonado = socket
        .clonar()
        .map_err(|_| ErrorComunicacionRTCP::ErrorClonandoSocket)?;

    enviar_primer_mensaje_rtcp(
        Arc::clone(&estadisticas_sender),
        socket_clonado,
        direccion_rtcp,
    )?;

    loop {
        //se chequea si termino sesion
        {
            let lock_termino = match termino.lock() {
                Ok(lock) => lock,
                Err(_) => return Err(ErrorComunicacionRTCP::ErrorEnviandoMensaje),
            };
            if *lock_termino {
                return Ok(());
            }
        }

        let socket_clonado = socket
            .clonar()
            .map_err(|_| ErrorComunicacionRTCP::ErrorClonandoSocket)?;

        enviar_mensajes_rtcp(
            socket_clonado,
            direccion_rtcp,
            Arc::clone(&dicc_estadisticas_receiver),
            Arc::clone(&estadisticas_sender),
        )?;

        let espera_segs = calcular_espera_para_enviar();
        let espera = Duration::from_secs(espera_segs as u64);
        thread::sleep(espera);
    }
}

fn cerrar_hilo(
    estadisticas_sender: Arc<Mutex<EstadisticasSender>>,
    mut socket: Box<dyn SocketUDP>,
    dicc_estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    direccion_rtcp: &str,
) -> Result<(), ErrorComunicacionRTCP> {
    let ssrc;
    {
        let lock_estadisticas = estadisticas_sender
            .lock()
            .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;
        ssrc = lock_estadisticas.ssrc;
    }

    let bytes = generar_bytes_paquete_bye(ssrc);

    {
        let estadisticas = dicc_estadisticas_receiver
            .lock()
            .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;

        if estadisticas.is_empty() {
            Ok(())
        } else {
            socket
                .enviar(&bytes[..], direccion_rtcp)
                .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;
            Ok(())
        }
    }
}

fn generar_bytes_paquete_bye(ssrc: u32) -> Vec<u8> {
    let contenido_paquete = ContenidoPaqueteRTCP::Bye();
    let paquete = PaqueteRTCP::crear(ssrc, contenido_paquete);
    Vec::from(&paquete)
}

fn calcular_espera_para_enviar() -> f64 {
    ESPERA_MINIMA_ENVIO_PAQUETES_RTCP
}

/// Envia el mensaje SenderReport a cada uno de los peers que lo estan escuchando.
fn enviar_mensajes_rtcp(
    mut socket: Box<dyn SocketUDP>,
    direccion_rtcp: &str,
    dicc_estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    estadisticas_sender: Arc<Mutex<EstadisticasSender>>,
) -> Result<usize, ErrorComunicacionRTCP> {
    //En este tiempo quizas llego el mensaje bye del otro pero como esta bloqueado el dicc no se pudo actualizar
    let dicc_estadisticas_lock = dicc_estadisticas_receiver
        .lock()
        .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;

    let mut suma_tamanios_mensajes = 0;
    let mut cantidad_mensajes: usize = 0;

    for clave in dicc_estadisticas_lock.keys() {
        let estadisticas_receiver = dicc_estadisticas_lock
            .get(clave)
            .ok_or(ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;

        let estadisticas_sender = estadisticas_sender
            .lock()
            .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;

        let mut contenido_paquete = estadisticas_receiver.contenido_report.clone();

        let timestamp_actual = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("ERROR: Error inesperado al obtener timestamp actual")
            .as_secs();

        let timestamp_actual_bits_medio = (timestamp_actual & 0x0000FFFFFFFF0000) >> 16;
        let timestamp_actual_rtp: u32 = timestamp_actual_bits_medio as u32;

        contenido_paquete.delay_desde_ultimo_paquete = u32::wrapping_sub(
            timestamp_actual_rtp,
            contenido_paquete.tiempo_desde_ultimo_paquete,
        );
        let contenido_sender_report = ContenidoSenderReport {
            contenido_report: contenido_paquete.clone(),
            ntp_timestamp: timestamp_actual,
            rtp_timestamp: timestamp_actual_rtp,
            sender_packet_count: estadisticas_sender.cantidad_paquetes_enviados,
            sender_octet_count: estadisticas_sender.cantidad_bytes_enviados,
        };
        let contenido_paquete = ContenidoPaqueteRTCP::SenderReport(contenido_sender_report);
        let paquete = PaqueteRTCP::crear(estadisticas_sender.ssrc, contenido_paquete);
        let bytes_paquete = Vec::from(&paquete);

        suma_tamanios_mensajes += bytes_paquete.len();
        cantidad_mensajes += 1;

        socket
            .enviar(&bytes_paquete[..], direccion_rtcp)
            .map_err(|_| {
                eprintln!("rompe");
                ErrorComunicacionRTCP::ErrorEnviandoMensaje
            })?;
    }

    if cantidad_mensajes > 0 {
        Ok(suma_tamanios_mensajes / cantidad_mensajes)
    } else {
        Ok(0)
    }
}

fn mensaje_finalizacion_llamada() -> Vec<u8> {
    let mut mensaje_bytes: Vec<u8> = Vec::new();

    let mensaje = "END".as_bytes();

    for letra in mensaje {
        mensaje_bytes.push(*letra);
    }

    mensaje_bytes
}

fn enviar_primer_mensaje_rtcp(
    lock_estadisticas_sender: Arc<Mutex<EstadisticasSender>>,
    mut socket: Box<dyn SocketUDP>,
    direccion_rtcp: &str,
) -> Result<(), ErrorComunicacionRTCP> {
    let mut contenido_paquete = ContenidoReport::crear_vacio_con_ssrc(5);
    let estadisticas_sender = lock_estadisticas_sender
        .lock()
        .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;

    let timestamp_actual = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("ERROR: Error inesperado al obtener timestamp actual")
        .as_secs();

    let timestamp_actual_bits_medio = (timestamp_actual & 0x0000FFFFFFFF0000) >> 16;
    let timestamp_actual_rtp: u32 = timestamp_actual_bits_medio as u32;

    contenido_paquete.delay_desde_ultimo_paquete = u32::wrapping_sub(
        timestamp_actual_rtp,
        contenido_paquete.tiempo_desde_ultimo_paquete,
    );
    let contenido_sender_report = ContenidoSenderReport {
        contenido_report: contenido_paquete.clone(),
        ntp_timestamp: timestamp_actual,
        rtp_timestamp: timestamp_actual_rtp,
        sender_packet_count: estadisticas_sender.cantidad_paquetes_enviados,
        sender_octet_count: estadisticas_sender.cantidad_bytes_enviados,
    };
    let contenido_paquete = ContenidoPaqueteRTCP::SenderReport(contenido_sender_report);
    let paquete = PaqueteRTCP::crear(estadisticas_sender.ssrc, contenido_paquete);
    let bytes_paquete = Vec::from(&paquete);

    socket
        .enviar(&bytes_paquete[..], direccion_rtcp)
        .map_err(|_| ErrorComunicacionRTCP::ErrorEnviandoMensaje)?;

    Ok(())
}

// #[cfg(test)]
// use super::socket_udp::MockSocketUdp;

// #[cfg(test)]
// use crate::protocolos::rtcp::tipo_paquete::{ContenidoReceiverReport, ContenidoReport};

// #[cfg(test)]
// const SSRC_EXTERNO_TESTS: u32 = 1234;

// #[cfg(test)]
// const SSRC_PROPIO_TESTS: u32 = 4321;

// #[test]
// /// Este test va a loopear infinitamente si no pasa correctamente, porque falla cuando la función queda en loop infinito
// /// Me encantaria hacerlo mas prolijo pero el Halting Problem me lo impide
// fn test_01_se_deja_de_escuchar_al_enviar_paquete_bye() {
//     let bytes_paquete_bye = bytes_paquete_bye();

//     let mut mock_socket_recepcion = MockSocketUdp {
//         bytes_que_se_leeran: bytes_paquete_bye,
//         bytes_enviados: Arc::new(Mutex::new(vec![])),
//         posicion_lectura: 0,
//     };
//     let estadisticas_receiver = Arc::new(Mutex::new(HashMap::new()));
//     let mock_socket_clonado: Box<dyn SocketUDP> = mock_socket_recepcion.clonar().unwrap();

//     let estadisticas_sender = Arc::new(Mutex::new(EstadisticasSender {
//         cantidad_bytes_enviados: 0,
//         ssrc: SSRC_PROPIO_TESTS,
//         cantidad_paquetes_enviados: 0,
//         ultimo_numero_secuencia: 0,
//         ultimo_timestamp_enviado: 0,
//         sesion_finalizada: false,
//     }));

//     let resultado = iniciar_escucha(
//         mock_socket_clonado,
//         Arc::clone(&estadisticas_receiver),
//         Arc::clone(&estadisticas_sender),
//     );

//     assert!(resultado.is_ok())
// }

// #[test]
// fn test_02_se_agregan_estadisticas_de_paquete_rtcp_con_origen_nuevo() {
//     let contenido_report = ContenidoReport::crear_vacio_con_ssrc(SSRC_EXTERNO_TESTS);
//     let bytes_paquetes = bytes_paquete_receiver_report_con_bye(contenido_report);

//     let mut mock_socket_recepcion = MockSocketUdp {
//         bytes_que_se_leeran: bytes_paquetes,
//         bytes_enviados: Arc::new(Mutex::new(vec![])),
//         posicion_lectura: 0,
//     };
//     let estadisticas_receiver = Arc::new(Mutex::new(HashMap::new()));
//     let mock_socket_clonado: Box<dyn SocketUDP> = mock_socket_recepcion.clonar().unwrap();

//     let estadisticas_sender = Arc::new(Mutex::new(EstadisticasSender {
//         cantidad_bytes_enviados: 0,
//         ssrc: SSRC_PROPIO_TESTS,
//         cantidad_paquetes_enviados: 0,
//         ultimo_numero_secuencia: 0,
//         ultimo_timestamp_enviado: 0,
//         sesion_finalizada: false,
//     }));

//     let resultado = iniciar_escucha(
//         mock_socket_clonado,
//         Arc::clone(&estadisticas_receiver),
//         Arc::clone(&estadisticas_sender),
//     );
//     let estadisticas_receiver_lock = estadisticas_receiver
//         .lock()
//         .expect("ERROR: Error obteniendo el lock de las estadisticas");

//     assert!(resultado.is_ok());
//     assert!(estadisticas_receiver_lock.len() == 1);
//     assert!(estadisticas_receiver_lock.contains_key(&SSRC_EXTERNO_TESTS));
// }

// #[test]
// fn test_03_se_registran_estadisticas_al_recibir_sender_report() {
//     let timestamp_actual = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .expect("ERROR: Error inesperado al obtener timestamp actual")
//         .as_secs();

//     let timestamp_actual_bits_medio = (timestamp_actual & 0x0000FFFFFFFF0000) >> 16;
//     let timestamp_actual_rtp: u32 = timestamp_actual_bits_medio as u32;

//     let contenido_report = ContenidoReport::crear_vacio_con_ssrc(SSRC_EXTERNO_TESTS);
//     let bytes_paquete = bytes_paquete_sender_report_con_bye(contenido_report, timestamp_actual);

//     let mut mock_socket_recepcion = MockSocketUdp {
//         bytes_que_se_leeran: bytes_paquete,
//         bytes_enviados: Arc::new(Mutex::new(vec![])),
//         posicion_lectura: 0,
//     };
//     let mock_socket_clonado: Box<dyn SocketUDP> = mock_socket_recepcion.clonar().unwrap();
//     let estadisticas_receiver = Arc::new(Mutex::new(HashMap::new()));

//     let estadisticas_sender = Arc::new(Mutex::new(EstadisticasSender {
//         cantidad_bytes_enviados: 0,
//         ssrc: SSRC_PROPIO_TESTS,
//         cantidad_paquetes_enviados: 0,
//         ultimo_numero_secuencia: 0,
//         ultimo_timestamp_enviado: 0,
//         sesion_finalizada: false,
//     }));

//     let resultado = iniciar_escucha(
//         mock_socket_clonado,
//         Arc::clone(&estadisticas_receiver),
//         Arc::clone(&estadisticas_sender),
//     );
//     let estadisticas_receiver_lock = estadisticas_receiver
//         .lock()
//         .expect("ERROR: Error obteniendo el lock de las estadisticas");

//     let report_de_media_externa = estadisticas_receiver_lock
//         .get(&SSRC_EXTERNO_TESTS)
//         .expect("ERROR: El SSRC ya deberia estar registrado en las estadisticas");

//     let contenido_report = &report_de_media_externa.contenido_report;

//     assert!(resultado.is_ok());
//     assert!(contenido_report.tiempo_desde_ultimo_paquete == timestamp_actual_rtp);
// }

// #[test]
// fn test_04_no_se_envia_sender_report_si_no_hay_destino() {
//     let bytes_enviados_socket = Arc::new(Mutex::new(vec![]));
//     let mut mock_sender = MockSocketUdp {
//         bytes_enviados: Arc::clone(&bytes_enviados_socket),
//         bytes_que_se_leeran: vec![],
//         posicion_lectura: 0,
//     };
//     let mock_socket_clonado: Box<dyn SocketUDP> = mock_sender.clonar().unwrap();

//     let estadisticas_sender = EstadisticasSender {
//         ssrc: SSRC_PROPIO_TESTS,
//         cantidad_bytes_enviados: 64,
//         cantidad_paquetes_enviados: 4,
//         ultimo_numero_secuencia: 4,
//         ultimo_timestamp_enviado: 755000,
//         sesion_finalizada: false,
//     };
//     let arc_estadisticas_sender = Arc::new(Mutex::new(estadisticas_sender));

//     let arc_estadisticas_receiver = Arc::new(Mutex::new(HashMap::new()));

//     let resultado = enviar_mensajes_rtcp(
//         mock_socket_clonado,
//         "127.0.0.1:8080",
//         Arc::clone(&arc_estadisticas_receiver),
//         Arc::clone(&arc_estadisticas_sender),
//     );
//     let bytes_enviados_lock = bytes_enviados_socket
//         .lock()
//         .expect("ERROR: No se pudo obtener un lock");

//     assert!(resultado.is_ok());
//     assert!(bytes_enviados_lock.is_empty())
// }

// #[test]
// fn test_05_se_actualizan_timestamps_al_enviar_sender_report() {
//     let bytes_enviados_socket = Arc::new(Mutex::new(vec![]));
//     let mut mock_sender = MockSocketUdp {
//         bytes_enviados: Arc::clone(&bytes_enviados_socket),
//         bytes_que_se_leeran: vec![],
//         posicion_lectura: 0,
//     };
//     let mock_socket_clonado: Box<dyn SocketUDP> = mock_sender.clonar().unwrap();

//     let estadisticas_sender = EstadisticasSender {
//         ssrc: SSRC_PROPIO_TESTS,
//         cantidad_bytes_enviados: 64,
//         cantidad_paquetes_enviados: 4,
//         ultimo_numero_secuencia: 4,
//         ultimo_timestamp_enviado: 755000,
//         sesion_finalizada: false,
//     };
//     let contenido_report_receiver = ContenidoReport {
//         ssrc_report: SSRC_EXTERNO_TESTS,
//         cant_paquetes_perdidos: 2,
//         tiempo_desde_ultimo_paquete: 12345,
//         frac_paquetes_perdidos: 128,
//         numero_mas_grande_de_paquete_recibido: 4,
//         tiempo_est_entre_paquetes: 123,
//         delay_desde_ultimo_paquete: 0,
//     };
//     let estadisticas_receiver = EstadisticasReceiver {
//         cantidad_paquetes_esperados_anterior: 2,
//         cantidad_paquetes_recibidos: 4,
//         cantidad_paquetes_recibidos_anterior: 2,
//         contenido_report: contenido_report_receiver,
//     };
//     let arc_estadisticas_sender = Arc::new(Mutex::new(estadisticas_sender));

//     let arc_estadisticas_receiver = Arc::new(Mutex::new(HashMap::new()));
//     {
//         let mut estadisticas_rec_lock = arc_estadisticas_receiver
//             .lock()
//             .expect("ERROR: No se pudo obtener un lock");

//         estadisticas_rec_lock
//             .entry(SSRC_EXTERNO_TESTS)
//             .or_insert(estadisticas_receiver);
//     }

//     let resultado = enviar_mensajes_rtcp(
//         mock_socket_clonado,
//         "127.0.0.1:8080",
//         Arc::clone(&arc_estadisticas_receiver),
//         Arc::clone(&arc_estadisticas_sender),
//     );
//     let bytes_enviados_lock = bytes_enviados_socket
//         .lock()
//         .expect("ERROR: No se pudo obtener un lock");

//     let paquete_enviado = PaqueteRTCP::try_from(&bytes_enviados_lock[..])
//         .expect("ERROR: Los bytes enviados son invalidos");

//     assert!(resultado.is_ok());
//     assert!(bytes_enviados_lock.len() == 52);
//     assert!(matches!(
//         paquete_enviado.payload,
//         ContenidoPaqueteRTCP::SenderReport(_)
//     ));
//     assert!(
//         if let ContenidoPaqueteRTCP::SenderReport(contenido_sender) = paquete_enviado.payload {
//             assert!(contenido_sender.ntp_timestamp > 0);
//             assert!(contenido_sender.rtp_timestamp > 0);
//             assert!(contenido_sender.contenido_report.delay_desde_ultimo_paquete > 0);
//             true
//         } else {
//             false
//         }
//     )
// }

// #[cfg(test)]
// fn bytes_paquete_bye() -> Vec<u8> {
//     let contenido_paquete = ContenidoPaqueteRTCP::Bye();
//     let paquete = PaqueteRTCP::crear(SSRC_EXTERNO_TESTS, contenido_paquete);
//     Vec::from(&paquete)
// }

// #[cfg(test)]
// fn bytes_paquete_receiver_report(contenido_report: ContenidoReport) -> Vec<u8> {
//     let contenido_receiver_report = ContenidoReceiverReport { contenido_report };
//     let contenido_paquete = ContenidoPaqueteRTCP::ReceiverReport(contenido_receiver_report);
//     let paquete = PaqueteRTCP::crear(SSRC_EXTERNO_TESTS, contenido_paquete);
//     Vec::from(&paquete)
// }

// #[cfg(test)]
// fn bytes_paquete_sender_report(contenido_report: ContenidoReport, timestamp: u64) -> Vec<u8> {
//     let timestamp_actual_bits_medio = (timestamp & 0x0000FFFFFFFF0000) >> 16;
//     let timestamp_actual_rtp: u32 = timestamp_actual_bits_medio as u32;

//     let contenido_sender_report = ContenidoSenderReport {
//         ntp_timestamp: timestamp,
//         rtp_timestamp: timestamp_actual_rtp,
//         sender_packet_count: 1,
//         sender_octet_count: 8,
//         contenido_report,
//     };

//     let contenido_paquete = ContenidoPaqueteRTCP::SenderReport(contenido_sender_report);
//     let paquete = PaqueteRTCP::crear(SSRC_EXTERNO_TESTS, contenido_paquete);
//     Vec::from(&paquete)
// }

// #[cfg(test)]
// fn bytes_paquete_sender_report_con_bye(
//     contenido_report: ContenidoReport,
//     timestamp: u64,
// ) -> Vec<u8> {
//     let mut bytes_paquete_rr = bytes_paquete_sender_report(contenido_report, timestamp);
//     let bytes_paquete_bye = bytes_paquete_bye();

//     for byte in bytes_paquete_bye {
//         bytes_paquete_rr.push(byte);
//     }

//     bytes_paquete_rr
// }

// #[cfg(test)]
// fn bytes_paquete_receiver_report_con_bye(contenido_report: ContenidoReport) -> Vec<u8> {
//     let mut bytes_paquete_rr = bytes_paquete_receiver_report(contenido_report);
//     let bytes_paquete_bye = bytes_paquete_bye();

//     for byte in bytes_paquete_bye {
//         bytes_paquete_rr.push(byte);
//     }

//     bytes_paquete_rr
// }
