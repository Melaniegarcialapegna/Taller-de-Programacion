use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::thread::JoinHandle;

use crate::logger::Logger;
use crate::protocolos::ice::flags_ice::FlagsICE;
use crate::protocolos::ice::protocolo_stun::MensajeStun;
use crate::sesion_rtp::socket_udp::SocketUDP;

const CONTEXTO_LOG: &str = "Agente ICE";

// el canal incluye el TID ([u8; 12])
type CanalICE = (
    FlagsICE,
    Sender<([u8; 12], String, u16)>,
    Receiver<([u8; 12], String, u16)>,
    (JoinHandle<()>, JoinHandle<()>),
);
const BUF_SIZE: usize = 1500;

/// Representa el agente ICE responsable de manejar la negociación de conectividad entre peers.
pub struct AgenteICE {
    /// Flags de estado del proceso ICE.
    pub flags: FlagsICE,
    /// Canal de envío para comunicar pares detectados.
    pub tx: Sender<([u8; 12], String, u16)>,
    /// Canal de recepción de direcciones detectadas por los hilos ICE.
    pub rx: Option<Receiver<([u8; 12], String, u16)>>,
    /// Si true, este agente es 'Controlling', sino 'Controlled'.
    es_controlling: Arc<AtomicBool>,
    thread_handles: Option<(JoinHandle<()>, JoinHandle<()>)>,
    tx_termino_ice: Sender<String>,
}

impl AgenteICE {
    pub fn new(
        logger: &Logger,
        socket_rtp: Box<dyn SocketUDP>,
        socket_rtcp: Box<dyn SocketUDP>,
        tx_termino_ice: Sender<String>,
    ) -> Self {
        let (flags, tx, rx, thread_handles) =
            Self::crear_y_lanzar_hilos(logger, socket_rtp, socket_rtcp, tx_termino_ice.clone());
        Self {
            flags,
            tx,
            rx: Some(rx),
            es_controlling: Arc::new(AtomicBool::new(false)),
            thread_handles: Some(thread_handles),
            tx_termino_ice,
        }
    }

    /// Reinicia el agente ICE, recreando canales y flags, y relanzando los hilos.
    pub fn reiniciar_agente_ice(
        &mut self,
        logger: &Logger,
        socket_rtp: Box<dyn SocketUDP>,
        socket_rtcp: Box<dyn SocketUDP>,
    ) {
        let (flags, tx, rx, thread_handles) = Self::crear_y_lanzar_hilos(
            logger,
            socket_rtp,
            socket_rtcp,
            self.tx_termino_ice.clone(),
        );
        self.flags = flags;
        self.tx = tx;
        self.rx = Some(rx);
        self.es_controlling.store(false, Ordering::SeqCst);
        self.thread_handles = Some(thread_handles);
    }

    // setter
    pub fn set_controlling(&self, controlling: bool) {
        self.es_controlling.store(controlling, Ordering::SeqCst);
    }

    pub fn es_controlling(&self) -> bool {
        self.es_controlling.load(Ordering::SeqCst)
    }

    /// Auxiliar que inicializa flags, crea canales y lanza los hilos ICE.
    fn crear_y_lanzar_hilos(
        logger: &Logger,
        socket_rtp: Box<dyn SocketUDP>,
        socket_rtcp: Box<dyn SocketUDP>,
        tx_termino_ice: Sender<String>,
    ) -> CanalICE {
        let flags = FlagsICE::new();
        let (tx, rx) = channel();
        let handle_rtp = Self::iniciar_hilo_ice(
            socket_rtp,
            "RTP",
            logger.clone(),
            tx.clone(),
            flags.clone(),
            tx_termino_ice.clone(),
        );
        let handle_rtcp = Self::iniciar_hilo_ice(
            socket_rtcp,
            "RTCP",
            logger.clone(),
            tx.clone(),
            flags.clone(),
            tx_termino_ice,
        );

        (flags, tx, rx, (handle_rtp, handle_rtcp))
    }

    /// Lanza un hilo que se encarga de escuchar y procesar mensajes STUN en un socket.
    fn iniciar_hilo_ice(
        mut socket: Box<dyn SocketUDP>,
        nombre: &str,
        logger: Logger,
        tx_ice: Sender<([u8; 12], String, u16)>,
        flags: FlagsICE,
        tx_termino_ice: Sender<String>,
    ) -> JoinHandle<()> {
        let nombre = nombre.to_string();
        thread::spawn(move || {
            eprintln!("COMENZO HILO ICE");
            Self::recibir_mensajes_stun(socket.as_mut(), &nombre, logger, tx_ice, flags);
            eprintln!("TERMINO HILO ICE");
            let _ = tx_termino_ice.send(nombre.to_string());
        })
    }

    pub fn detener_hilos_ice(&mut self) {
        eprintln!("Solicitando detención de hilos ICE...");
        self.flags.solicitar_shutdown();

        if let Some((handle_rtp, handle_rtcp)) = self.thread_handles.take() {
            let _ = handle_rtp.join();
            let _ = handle_rtcp.join();
            eprintln!("Hilos ICE detenidos completamente");
        }
    }

    /// Bucle principal de recepción de mensajes STUN.
    fn recibir_mensajes_stun(
        socket: &mut dyn SocketUDP,
        nombre: &str,
        logger: Logger,
        tx_ice: Sender<([u8; 12], String, u16)>,
        flags: FlagsICE,
    ) {
        let mut buffer = [0u8; BUF_SIZE];
        logger.info(
            &format!("Hilo de recepción '{}' iniciado", nombre),
            CONTEXTO_LOG,
        );
        loop {
            // chequeo de shutdown solicitado
            if flags.shutdown_solicitado() {
                logger.info(
                    &format!("Hilo '{}' detenido por shutdown explícito", nombre),
                    CONTEXTO_LOG,
                );
                break;
            }
            // chequeo de finalización de ICE
            if flags.ice_finalizado() {
                logger.info(
                    &format!("Hilo '{}' finalizado (ambos peers terminaron)", nombre),
                    CONTEXTO_LOG,
                );
                // el hilo sale del loop y termina, liberando el control del socket
                break;
            }
            match socket.recibir(&mut buffer) {
                Ok((len, addr)) => {
                    Self::procesar_mensaje_stun(
                        &buffer[..len],
                        addr,
                        socket,
                        &logger,
                        &tx_ice,
                        &flags,
                    );
                }
                Err(_) => {
                    // es bloqueante asiq no debería pasar
                }
            }
        }
    }

    /// Procesa un mensaje STUN recibido, determinando su tipo (Request o Response).
    fn procesar_mensaje_stun(
        buffer: &[u8],
        addr: SocketAddr,
        socket: &mut dyn SocketUDP,
        logger: &Logger,
        tx_ice: &Sender<([u8; 12], String, u16)>,
        flags: &FlagsICE,
    ) {
        if buffer.len() < MensajeStun::min_size() {
            logger.warn(
                &format!("Recibido mensaje muy corto desde {}. Ignorando.", addr),
                CONTEXTO_LOG,
            );
            return;
        }

        let mensaje_stun = match MensajeStun::deserialize(buffer) {
            Ok(mensaje) => mensaje,
            Err(e) => {
                logger.warn(
                    &format!("Error al deserializar mensaje STUN desde {}: {}", addr, e),
                    CONTEXTO_LOG,
                );
                return;
            }
        };

        let log_mensaje = if mensaje_stun.es_request() {
            Some(format!(
                "Recibido STUN REQUEST (Attrs len: {}) desde {}",
                mensaje_stun.payload.len(),
                addr
            ))
        } else if mensaje_stun.es_response() && mensaje_stun.contiene_ice_finalizado() {
            Some(format!(
                "Recibido STUN RESPONSE FINALIZADO (Attrs len: {}) desde {}",
                mensaje_stun.payload.len(),
                addr
            ))
        } else if mensaje_stun.es_response() {
            // saque el log este porque se me cargaba mucho el log, se podría loguear y sacar este if
            // para mi con los response de use candidate/finalizacion alcanza pero bueno
            None
        } else {
            logger.info(
                "Mensaje STUN recibido no es Request ni Response",
                CONTEXTO_LOG,
            );
            None
        };

        if let Some(mensaje) = log_mensaje {
            logger.info(&mensaje, CONTEXTO_LOG);
        }

        if mensaje_stun.es_request() {
            Self::manejar_stun_request(mensaje_stun, addr, socket, logger, flags);
        } else if mensaje_stun.es_response() {
            Self::manejar_stun_response(mensaje_stun, addr, tx_ice, logger, flags);
        }
    }

    /// Maneja un mensaje STUN Binding Request.
    fn manejar_stun_request(
        mensaje_stun: MensajeStun,
        addr: SocketAddr,
        socket: &mut dyn SocketUDP,
        logger: &Logger,
        flags: &FlagsICE,
    ) {
        if mensaje_stun.contiene_ice_finalizado() {
            flags.set_ice_remoto_finalizado();
            logger.info(
                "Peer remoto envió STUN REQUEST con ICE_FINALIZADO",
                CONTEXTO_LOG,
            );
        } else if mensaje_stun.contiene_use_candidate() {
            // el peer remoto (Controlling) está nominando un par de candidatos.
            flags.set_ice_local_finalizado();
            logger.info(
                "Se recibió STUN REQUEST con USE_CANDIDATE (Nominación). ICE local finalizado.",
                CONTEXTO_LOG,
            );
        }

        // se mantiene el TID de la request para la response
        // el Response incluye el flag ICE_FINALIZADO si el estado local ya finalizó
        let response = mensaje_stun
            .binding_response_for_request(flags.get_ice_local_finalizado())
            .serialize();
        let addr_str = addr.to_string();
        let _ = socket.enviar(&response, &addr_str);
    }

    /// Maneja un mensaje STUN Binding Response.
    fn manejar_stun_response(
        mensaje_stun: MensajeStun,
        addr: SocketAddr,
        tx_ice: &Sender<([u8; 12], String, u16)>,
        logger: &Logger,
        flags: &FlagsICE,
    ) {
        if mensaje_stun.contiene_ice_finalizado() {
            flags.set_ice_remoto_finalizado();
            logger.info("Se recibió STUN RESPONSE con ICE_FINALIZADO", CONTEXTO_LOG);
        }

        // enviamos el TID de la Response para que chequeo_ice.rs lo valide
        let _ = tx_ice.send((
            mensaje_stun.transaction_id,
            addr.ip().to_string(),
            addr.port(),
        ));
    }
}
