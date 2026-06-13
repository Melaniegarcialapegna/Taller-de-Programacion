use crate::logger::Logger;
use crate::protocolos::ice::candidato::Candidato;
use crate::protocolos::ice::flags_ice::FlagsICE;
use crate::protocolos::ice::parser::parsear_linea;
use crate::protocolos::ice::protocolo_stun::MensajeStun;
use crate::sesion_rtp::socket_udp::SocketUDP;
use std::collections::HashMap; // para los TIDs
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration; // para acceder al mapa

const CONTEXTO_LOG: &str = "ICE";
const INTERVALO_RETRANSMISION: Duration = Duration::from_millis(3000); // rfc
const CANAL_TIMEOUT: Duration = Duration::from_secs(5); // para el canal

#[derive(Debug, Clone)]
pub struct ParDeCandidatos {
    pub local: Candidato,
    pub remoto: Candidato,
}

// Mapa para almacenar los TIDs pendientes y sus pares de candidatos asociados.
pub type TidMap = Arc<Mutex<HashMap<[u8; 12], ParDeCandidatos>>>;

// Sockets (RTP, RTCP)
type Sockets<'a> = (&'a mut dyn SocketUDP, &'a mut dyn SocketUDP);

// Pares de candidatos SDP
type CandidatosSDP<'a> = (&'a [String], &'a [String]);

// Canal Receiver con el tipo que usás
type CanalIce = Receiver<([u8; 12], String, u16)>;

// Flags + controlling juntos
type ControlInfo<'a> = (&'a FlagsICE, bool);

type EsperaInfo<'a> = (
    TidMap,          // tid_a_par
    &'a FlagsICE,    // flags_ice
    Arc<AtomicBool>, // detener_chequeos
    bool,            // es_controlling
    &'a [u8],        // stun_request_use_candidate
);

type SocketsMut<'a> = (&'a mut dyn SocketUDP, &'a mut dyn SocketUDP);

type FinalizacionInfo<'a> = (
    String, // ip_remota
    u16,    // puerto_rtp
    &'a FlagsICE,
    &'a [u8], // mensaje finalización
);

type SocketsFinal<'a> = (&'a mut dyn SocketUDP, &'a mut dyn SocketUDP);

// función que usa rtcpeerconnection
/// Inicia la negociación de todos los pares de candidatos en paralelo.
///
/// El flujo general es el siguiente:
/// 1. Prepara todos los pares de candidatos locales/remotos.
/// 2. Inicializa estructuras de control (TidMap, AtomicBool).
/// 3. Lanza hilos de chequeo STUN, uno por cada par, para enviar requests periódicas.
/// 4. El hilo principal espera la primera respuesta exitosa en el canal de comunicación.
/// 5. Si es Controlling, envía la nominación (USE_CANDIDATE).
/// 6. Detiene y une todos los hilos de chequeo.
/// 7. Marca la finalización local de ICE y notifica al peer.
pub fn negociar_par_candidatos<'a>(
    logger: &'a Logger,
    sockets: Sockets<'a>,
    sdp: CandidatosSDP<'a>,
    canal_y_control: (CanalIce, ControlInfo<'a>),
) -> Result<ParDeCandidatos, String> {
    let (socket_rtp_clonable, socket_rtcp_clonable) = sockets;
    let (candidatos_locales_sdp, candidatos_remotos_sdp) = sdp;
    let (rx_ice, (flags_ice, es_controlling)) = canal_y_control;

    // 1. Prepara Pares de Candidatos
    let todos_los_pares =
        preparar_pares_de_candidatos(candidatos_locales_sdp, candidatos_remotos_sdp)?;

    // 2. Inicializa estructuras de control y mensajes STUN
    let detener_chequeos = Arc::new(AtomicBool::new(false));
    let tid_a_par: TidMap = Arc::new(Mutex::new(HashMap::new()));
    let stun_request_use_candidate = MensajeStun::binding_request_use_candidate().serialize();
    let stun_request_ice_finalizado = MensajeStun::binding_request_ice_finalizado().serialize();

    // 3. Lanza Hilos de Chequeo STUN
    let handles = lanzar_hilos_de_chequeo(
        logger,
        todos_los_pares,
        socket_rtp_clonable,
        socket_rtcp_clonable,
        tid_a_par.clone(),
        detener_chequeos.clone(),
    )?;

    // 4. Espera el primer resultado exitoso (bucle principal)
    let resultado_negociacion = esperar_resultado_exitoso(
        logger,
        rx_ice,
        (
            tid_a_par,
            flags_ice,
            detener_chequeos.clone(),
            es_controlling,
            &stun_request_use_candidate[..],
        ),
        (socket_rtp_clonable, socket_rtcp_clonable),
    );

    // 6. Detiene y Une Hilos
    detener_y_unir_hilos(logger, detener_chequeos, handles);

    // 7. Marca la finalización y retorna el resultado
    match resultado_negociacion {
        Ok(par) => {
            // Se encontró el par exitoso, se notifica y se retorna.
            let ip_remota = par.remoto.getter_ip().to_string();
            let puerto_remoto_rtp = par.remoto.getter_puerto();

            finalizar_y_retornar_exito(
                logger,
                par,
                (
                    ip_remota,
                    puerto_remoto_rtp,
                    flags_ice,
                    &stun_request_ice_finalizado[..],
                ),
                (socket_rtp_clonable, socket_rtcp_clonable),
            )
        }
        Err(e) => {
            // no se encontró ningún par funcional o falló la sincronización
            Err(e)
        }
    }
}

/// Prepara la lista de pares de candidatos a chequear a partir de las strings SDP.
fn preparar_pares_de_candidatos(
    candidatos_locales_sdp: &[String],
    candidatos_remotos_sdp: &[String],
) -> Result<Vec<ParDeCandidatos>, String> {
    let mut candidatos_locales = Vec::new();
    let mut candidatos_remotos = Vec::new();

    for candidato in candidatos_locales_sdp {
        match parsear_linea(candidato) {
            Ok(candidato) => candidatos_locales.push(candidato),
            Err(_) => {
                return Err(format!(
                    "Error al parsear candidato local SDP '{}'.",
                    candidato
                ));
            }
        }
    }

    for candidato in candidatos_remotos_sdp {
        match parsear_linea(candidato) {
            Ok(candidato) => candidatos_remotos.push(candidato),
            Err(_) => {
                return Err(format!(
                    "Error al parsear candidato remoto SDP '{}'.",
                    candidato
                ));
            }
        }
    }

    if candidatos_locales.is_empty() || candidatos_remotos.is_empty() {
        return Err("No se encontraron candidatos locales o remotos válidos.".to_string());
    }

    let mut todos_los_pares: Vec<ParDeCandidatos> = Vec::new();

    // emparejar todos los candidatos: cada local con cada remoto
    for local in candidatos_locales {
        for remoto in &candidatos_remotos {
            todos_los_pares.push(ParDeCandidatos {
                local: local.clone(),
                remoto: remoto.clone(),
            });
        }
    }
    Ok(todos_los_pares)
}

/// Lanza los hilos de chequeo STUN para cada par de candidatos.
fn lanzar_hilos_de_chequeo(
    logger: &Logger,
    todos_los_pares: Vec<ParDeCandidatos>,
    socket_rtp_clonable: &mut dyn SocketUDP,
    socket_rtcp_clonable: &mut dyn SocketUDP,
    tid_a_par: TidMap,
    detener_chequeos: Arc<AtomicBool>,
) -> Result<Vec<JoinHandle<()>>, String> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    // lanzar Hilos de Chequeo
    for par in todos_los_pares.into_iter() {
        // cada hilo/par crea su propia Request con un TID único
        let stun_request_base = MensajeStun::binding_request().serialize();

        // obtenemos el TID del Request que acabamos de crear
        let mensaje_stun = match MensajeStun::deserialize(&stun_request_base) {
            Ok(m) => m,
            Err(e) => {
                logger.error(
                    &format!(
                        "Error al deserializar STUN Request base para obtener TID: {}",
                        e
                    ),
                    CONTEXTO_LOG,
                );
                // retornamos temprano del hilo de chequeo si falla la deserialización interna
                return Err("Error interno al preparar STUN Request base.".to_string());
            }
        };
        let tid_request = mensaje_stun.transaction_id;

        {
            let mut mapa = match tid_a_par.lock() {
                Ok(m) => m,
                Err(_) => {
                    logger.error(
                        "Fallo al adquirir el lock de tid_a_par (Inserción). Terminando hilo.",
                        CONTEXTO_LOG,
                    );
                    return Err("Error interno de sincronización.".to_string());
                }
            };
            mapa.insert(tid_request, par.clone());
        }

        // clonamos los sockets para que el hilo tenga su propia copia
        let socket_rtp_clone = match socket_rtp_clonable.clonar() {
            Ok(s) => s,
            Err(_) => {
                logger.error("Fallo al clonar socket RTP para el hilo", CONTEXTO_LOG);
                detener_chequeos.store(true, Ordering::SeqCst);
                return Err("Error interno al clonar socket RTP.".to_string());
            }
        };
        let socket_rtcp_clone = match socket_rtcp_clonable.clonar() {
            Ok(s) => s,
            Err(_) => {
                logger.error("Fallo al clonar socket RTP para el hilo", CONTEXTO_LOG);
                detener_chequeos.store(true, Ordering::SeqCst);
                return Err("Error interno al clonar socket RTCP.".to_string());
            }
        };

        let detener_chequeos_clone = detener_chequeos.clone();

        let handle = thread::spawn(move || {
            // retransmisión del mensaje único (mismo TID por retransmisión)
            let stun_request_base_clone = stun_request_base;

            // la lógica de chequeo y retransmisión
            hilo_chequeo_stun(
                detener_chequeos_clone,
                stun_request_base_clone,
                par,
                socket_rtp_clone,
                socket_rtcp_clone,
            );
        });

        handles.push(handle);
    }

    Ok(handles)
}

/// Lógica del hilo de chequeo: envía STUN Requests periódicamente (con retransmisiones)
fn hilo_chequeo_stun(
    detener_chequeos: Arc<AtomicBool>,
    stun_request_base: Vec<u8>,
    par: ParDeCandidatos,
    mut socket_rtp_clone: Box<dyn SocketUDP>,
    mut socket_rtcp_clone: Box<dyn SocketUDP>,
) {
    while !detener_chequeos.load(Ordering::SeqCst) {
        // enviar STUN Request por RTP
        let remote_rtp_addr = format!("{}:{}", par.remoto.getter_ip(), par.remoto.getter_puerto());
        let _ = socket_rtp_clone.enviar(&stun_request_base, &remote_rtp_addr);

        // enviar STUN Request por RTCP
        let remote_rtcp_addr = format!(
            "{}:{}",
            par.remoto.getter_ip(),
            par.remoto.getter_puerto() + 1
        );
        let _ = socket_rtcp_clone.enviar(&stun_request_base, &remote_rtcp_addr);

        // como se pueden perder paquetes por udp, rfc dice que hay q mandar constantemente los checks con un intervalo
        // por si alguno no llega
        //
        // entiendo que ninguno va a necesitar una retransmision porque encontramos uno valido al toque y
        // nunca me pasó q se pierda alguno,
        // así que no es común que se logueen retransmisiones, pero debería estar x las dudas
        thread::sleep(INTERVALO_RETRANSMISION);
    }
}

/// Bucle principal para esperar la primera respuesta STUN exitosa.
fn esperar_resultado_exitoso<'a>(
    logger: &'a Logger,
    canal: CanalIce,
    info: EsperaInfo<'a>,
    sockets: SocketsMut<'a>,
) -> Result<ParDeCandidatos, String> {
    let rx_ice = canal;

    let (tid_a_par, flags_ice, detener_chequeos, es_controlling, stun_request_use_candidate) = info;

    let (socket_rtp_clonable, socket_rtcp_clonable) = sockets;

    logger.info(
        "Esperando primer resultado exitoso de AgenteICE en el canal...",
        CONTEXTO_LOG,
    );

    let mut par_exitoso: Option<ParDeCandidatos> = None;
    loop {
        // recibimos el TID, IP y Puerto
        match rx_ice.recv_timeout(CANAL_TIMEOUT) {
            Ok((tid_recibido, ip_remota, puerto_remoto)) => {
                // validar que el TID recibido es uno de los TIDs pendientes
                let par_candidato_asociado = match extraer_par_de_mapa(
                    tid_a_par.clone(),
                    tid_recibido,
                    detener_chequeos.clone(),
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        logger.error(&e, CONTEXTO_LOG);
                        break;
                    }
                };

                // si se encontró el TID y el par asociado fue extraído:
                if let Some(par) = par_candidato_asociado {
                    let es_rtp = par.remoto.getter_puerto() == puerto_remoto
                        && par.remoto.getter_ip() == ip_remota;
                    let es_rtcp = par.remoto.getter_puerto() + 1 == puerto_remoto
                        && par.remoto.getter_ip() == ip_remota;

                    if es_rtp || es_rtcp {
                        logger.info(
                            &format!("Conexión exitosa detectada y validada por TID - Remoto: {}:{} con Local: {}:{}", 
                                ip_remota, puerto_remoto,
                                par.local.getter_ip(), par.local.getter_puerto()
                            ),
                            CONTEXTO_LOG
                        );
                        par_exitoso = Some(par.clone());

                        // si soy controlling, mando un STUN adicional con USE_CANDIDATE
                        manejar_nominacion(
                            logger,
                            es_controlling,
                            &par,
                            socket_rtp_clonable,
                            socket_rtcp_clonable,
                            stun_request_use_candidate,
                        );

                        // se encontró el par, salimos del loop de espera
                        break;
                    } else {
                        // si el TID era válido pero la dirección no, lo ignoramos, pero el check ya se consumió
                        logger.warn(
                            "Recibido TID válido pero la dirección no coincide con el par esperado. Ignorando.", 
                            CONTEXTO_LOG
                        );
                    }
                } else {
                    // si el TID no estaba en el mapa, es una respuesta tardía o no solicitada, se ignora
                    logger.info(
                        "Recibido STUN Response con TID no solicitado. Ignorando.",
                        CONTEXTO_LOG,
                    );
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if flags_ice.get_ice_remoto_finalizado() {
                    logger.info("ICE remoto finalizado antes de encontrar candidato local. Terminando chequeo.", CONTEXTO_LOG);
                    break;
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err("Canal de AgenteICE desconectado inesperadamente.".to_string());
            }
        }
    }

    // devolvemos el resultado encontrado (o error si falló la sincronización)
    match par_exitoso {
        Some(par) => Ok(par),
        None => Err("No se encontró ningún par de candidatos funcional.".to_string()),
    }
}

/// Intenta extraer y remover el par de candidatos asociado al TID del mapa.
fn extraer_par_de_mapa(
    tid_a_par: TidMap,
    tid_recibido: [u8; 12],
    detener_chequeos: Arc<AtomicBool>,
) -> Result<Option<ParDeCandidatos>, String> {
    let mut mapa = match tid_a_par.lock() {
        Ok(m) => m,
        Err(_) => {
            detener_chequeos.store(true, Ordering::SeqCst);
            return Err(
                "Fallo al adquirir el lock de tid_a_par (Remoción). Terminando negociación."
                    .to_string(),
            );
        }
    };
    // intentamos remover el TID, si existe, la respuesta es válida y correlacionada
    Ok(mapa.remove(&tid_recibido))
}

/// Envía el mensaje USE_CANDIDATE si el agente es Controlling.
fn manejar_nominacion(
    logger: &Logger,
    es_controlling: bool,
    par: &ParDeCandidatos,
    socket_rtp_clonable: &mut dyn SocketUDP,
    socket_rtcp_clonable: &mut dyn SocketUDP,
    stun_request_use_candidate: &[u8],
) {
    if es_controlling {
        let remote_addr_rtp = format!("{}:{}", par.remoto.getter_ip(), par.remoto.getter_puerto());
        let remote_addr_rtcp = format!(
            "{}:{}",
            par.remoto.getter_ip(),
            par.remoto.getter_puerto() + 1
        );

        // se usa el Request con TID nuevo por ser una nueva transacción (nominación)
        let _ = socket_rtp_clonable.enviar(stun_request_use_candidate, &remote_addr_rtp);
        let _ = socket_rtcp_clonable.enviar(stun_request_use_candidate, &remote_addr_rtcp);

        logger.info(
            &format!(
                "Controlling: Se envió USE_CANDIDATE binario a {} (RTP y RTCP)",
                par.remoto.getter_ip()
            ),
            CONTEXTO_LOG,
        );
    } else {
        // soy controlled
        logger.info(
            "Controlled: Primer check exitoso. Esperando nominación del Controlling...",
            CONTEXTO_LOG,
        );
    }
}

/// Detiene los hilos y espera su finalización.
fn detener_y_unir_hilos(
    logger: &Logger,
    detener_chequeos: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
) {
    // detener los hilos de chequeo y esperar su finalización
    logger.info("Iniciando limpieza de hilos de chequeo...", CONTEXTO_LOG);
    detener_chequeos.store(true, Ordering::SeqCst);

    // esto creo que habría q dejarlo por si los threads siguen en el sleep de la retransmisión
    // no sé si basta el join
    thread::sleep(INTERVALO_RETRANSMISION * 2);

    for handle in handles {
        let _ = handle.join(); // esperar a que el hilo termine
    }
    logger.info("Limpieza de hilos de chequeo finalizada.", CONTEXTO_LOG);
}

/// Notifica al peer la finalización de ICE y retorna el par exitoso.
fn finalizar_y_retornar_exito<'a>(
    logger: &'a Logger,
    par_exitoso: ParDeCandidatos,
    info: FinalizacionInfo<'a>,
    sockets: SocketsFinal<'a>,
) -> Result<ParDeCandidatos, String> {
    let (ip_remota, puerto_remoto_rtp, flags_ice, stun_request_ice_finalizado) = info;
    let (socket_rtp_clonable, socket_rtcp_clonable) = sockets;

    flags_ice.set_ice_local_finalizado();

    // enviar STUN_BINDING_REQUEST_ICE_FINALIZADO por RTP y RTCP
    let destino_rtp = format!("{}:{}", ip_remota, puerto_remoto_rtp);
    let destino_rtcp = format!("{}:{}", ip_remota, puerto_remoto_rtp + 1);

    // se usa un TID nuevo para el mensaje de finalización, ya que es una nueva Request
    let _ = socket_rtp_clonable.enviar(stun_request_ice_finalizado, &destino_rtp);
    let _ = socket_rtcp_clonable.enviar(stun_request_ice_finalizado, &destino_rtcp);

    logger.info(
        &format!(
            "Se notificó la finalización de ICE al peer en {} (RTP y RTCP) con mensaje binario.",
            ip_remota
        ),
        CONTEXTO_LOG,
    );

    Ok(par_exitoso)
}
