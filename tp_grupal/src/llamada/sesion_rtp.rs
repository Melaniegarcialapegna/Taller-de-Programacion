//! # SesionRTP - Sesion de envio y recepcion de mensajes mediante RTP y RTCP.
//!
//! Este struct tendra la responsabilidad de administrar cuatro hilos que funcionaran durante la llamada:
//! - El hilo de envio de mensajes RTP
//! - El hilo de recepcion de mensajes RTP
//! - El hilo de envio de mensajes RTCP
//! - El hilo de recepcion de mensajes RTCP
//!
//! Al crearse, iniciara estos cuatro hilos. Cuando se reciba el mensaje [SesionRTP::cortar_llamada], se encargara
//! de que esos cuatro hilos se cierren y que los sockets queden liberados para una proxima SesionRTP.

use bytes::Bytes;
use std::{
    fmt::Display,
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use crate::{
    aplicacion::EventoLlamada,
    creacion_llamada::ConexionP2P,
    logger::Logger,
    protocolos::sctp::evento_sctp::EventoSctp,
    sesion_rtp::{
        comunicacion_rtp::ComunicadoresConIO,
        error::ErrorSesion,
        sesion::{EstadisticasReceiver, crear_sesion_rtp},
        socket_udp::SocketUDP,
    },
};

const MATAR_DEMUX: &[u8] = b"__MATAR_DEMUX__";

#[allow(dead_code)]
pub struct SesionRTP {
    sender_terminar_llamada: Sender<MensajeFinalizarLlamada>,
    socket_rtp: Box<dyn SocketUDP>,
    socket_rtcp: Box<dyn SocketUDP>,
    direccion_rtp_externa: String,
    direccion_rtcp_externa: String,
    receiver_estadisticas: Option<Receiver<EstadisticasReceiver>>,
    // Los dos proximos receivers deberian usarse para funcionalidades que todavia no estan implementadas
    // Se dejan aca para que al implementarlos sea simplemente borrarlos de aca y usarlos.
    puerto_rtp_local: u16,
    tx_datos_sctp: Option<Sender<Bytes>>,
    rx_eventos_sctp: Option<Receiver<EventoSctp>>,
}

#[derive(Debug)]
pub enum ErrorSesionRTP {
    ErrorInterno(String),
    ErrorEnThreadsSesion(String),
    ErrorCortandoLlamada,
}

pub struct SocketsRTP {
    socket_rtp: Box<dyn SocketUDP>,
    socket_rtcp: Box<dyn SocketUDP>,
}

pub enum MensajeFinalizarLlamada {
    CortoElOtroPeer,
    CortamosNosotros,
}

impl Display for ErrorSesionRTP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSesionRTP::ErrorInterno(e) => {
                f.write_str(&format!("Error interno en SesionRTP: {}", e))
            }
            ErrorSesionRTP::ErrorEnThreadsSesion(error) => {
                f.write_str(&format!("Error en threads de sesion RTP: {error}"))
            }
            ErrorSesionRTP::ErrorCortandoLlamada => {
                f.write_str("Error cortando llamada desde SesionRTP")
            }
        }
    }
}

impl From<ErrorSesion> for ErrorSesionRTP {
    fn from(error: ErrorSesion) -> Self {
        ErrorSesionRTP::ErrorEnThreadsSesion(format!("{error}"))
    }
}

impl SesionRTP {
    pub fn new(
        mut conexion: Box<ConexionP2P>,
        comunicadores_con_io: ComunicadoresConIO,
        mutex_procesando_frame: Arc<Mutex<bool>>,
        sender_eventos_llamada: Sender<EventoLlamada>,
        logger: Logger,
    ) -> Result<SesionRTP, ErrorSesionRTP> {
        // Creo una copia de los sockets para cuando tenga que enviar un mensaje cortando llamada
        // Soy consciente de que es desprolijo, pero haciendo esto puedo reutilizar la funcion crear_sesion_rtp entera
        let sockets = obtener_clones_sockets(&mut conexion)?;
        let socket_rtp = sockets.socket_rtp;
        let socket_rtcp = sockets.socket_rtcp;

        let direccion_rtp_externa = conexion.direccion_rtp_externa.clone();
        let direccion_rtcp_externa = conexion.direccion_rtcp_externa.clone();

        // Creo channel para estadisticas de la llamada
        let (sender_estadisticas, receiver_estadisticas) = mpsc::channel();

        // Creo ByeChannels
        let (sender_termino_llamada, receiver_termino_llamada) = mpsc::channel();
        let (sender_terminar_llamada, receiver_terminar_llamada) = mpsc::channel();

        // Obtengo puerto rtp local
        let puerto_rtp_local = conexion
            .socket_rtp_propio
            .local_addr()
            .map_err(|e| {
                ErrorSesionRTP::ErrorInterno(format!(
                    "Error obteniendo puerto local del socket RTP: {:?}",
                    e
                ))
            })?
            .port();

        // Obtengo el extremo del channel para recibir los paquetes SRTP del demux, en lugar del socket UDP.
        let srtp_rx = conexion.srtp_rx.take().ok_or(ErrorSesionRTP::ErrorInterno(
            "srtp_rx no disponible".to_string(),
        ))?;

        let tx_datos_sctp = conexion.tx_datos_sctp.take();
        let rx_eventos_sctp = conexion.rx_eventos_sctp.take();

        thread::spawn(move || {
            // Inicio la sesion RTP
            if let Err(e) = crear_sesion_rtp(
                conexion,
                sender_estadisticas,
                comunicadores_con_io,
                logger,
                mutex_procesando_frame,
                (receiver_terminar_llamada, sender_termino_llamada),
                srtp_rx,
            ) {
                eprintln!("{e}");
            };
        });

        thread::spawn(move || {
            Self::escuchar_mensaje_terminar_llamada(
                receiver_termino_llamada,
                sender_eventos_llamada,
            );
        });

        Ok(SesionRTP {
            sender_terminar_llamada,
            receiver_estadisticas: Some(receiver_estadisticas),
            socket_rtp,
            socket_rtcp,
            direccion_rtp_externa,
            direccion_rtcp_externa,
            puerto_rtp_local,
            tx_datos_sctp,
            rx_eventos_sctp,
        })
    }

    /// Devuelve un clon del sender para enviar datos por SCTP. Si no se tiene el sender, devuelve None.
    pub fn clonar_tx_datos_sctp(&self) -> Option<Sender<Bytes>> {
        self.tx_datos_sctp.clone()
    }

    /// Envía un mensaje de cortado de llamada a la otra parte, y luego se encarga de cerrar los sockets y matar los threads de la sesion RTP.
    pub fn cortar_llamada(&mut self) -> Result<(), ErrorSesionRTP> {
        self.sender_terminar_llamada
            .send(MensajeFinalizarLlamada::CortamosNosotros)
            .map_err(|_| ErrorSesionRTP::ErrorCortandoLlamada)?;

        self.socket_rtp
            .enviar(
                &Self::mensaje_finalizacion_llamada(),
                &self.direccion_rtp_externa,
            )
            .map_err(|_| ErrorSesionRTP::ErrorCortandoLlamada)?;

        let stopper =
            UdpSocket::bind("0.0.0.0:0").map_err(|_| ErrorSesionRTP::ErrorCortandoLlamada)?;

        let addr: SocketAddr = format!("127.0.0.1:{}", self.puerto_rtp_local)
            .parse::<SocketAddr>()
            .map_err(|_| ErrorSesionRTP::ErrorCortandoLlamada)?;

        let _ = stopper.send_to(MATAR_DEMUX, addr);

        Ok(())
    }

    /// Devuelve el receiver para recibir las estadisticas de la llamada. Si ya se ha solicitado antes, devuelve un error.
    pub fn obtener_receiver_estadisticas(
        &mut self,
    ) -> Result<Receiver<EstadisticasReceiver>, ErrorSesionRTP> {
        let receiver = self
            .receiver_estadisticas
            .take()
            .ok_or(ErrorSesionRTP::ErrorInterno(
                "Receiver ya solicitado".to_string(),
            ))?;

        Ok(receiver)
    }

    /// Devuelve el receiver para recibir los eventos de SCTP. Si ya se ha solicitado antes, devuelve None.
    pub fn obtener_rx_eventos_sctp(&mut self) -> Option<Receiver<EventoSctp>> {
        self.rx_eventos_sctp.take()
    }

    fn mensaje_finalizacion_llamada() -> Vec<u8> {
        let mut mensaje_bytes: Vec<u8> = Vec::new();

        let mensaje = "END".as_bytes();

        for letra in mensaje {
            mensaje_bytes.push(*letra);
        }

        mensaje_bytes
    }

    fn escuchar_mensaje_terminar_llamada(
        receiver_cortar_llamada: Receiver<MensajeFinalizarLlamada>,
        sender_eventos_llamada: Sender<EventoLlamada>,
    ) {
        if let Err(error) = Self::_escuchar_mensaje_terminar_llamada(
            receiver_cortar_llamada,
            sender_eventos_llamada,
        ) {
            dbg!(error);
        }
    }

    fn _escuchar_mensaje_terminar_llamada(
        receiver_se_corto_llamada: Receiver<MensajeFinalizarLlamada>,
        sender_eventos_llamada: Sender<EventoLlamada>,
    ) -> Result<(), ErrorSesionRTP> {
        loop {
            let mensaje_recibido = receiver_se_corto_llamada
                .recv()
                .map_err(|_| ErrorSesionRTP::ErrorCortandoLlamada)?;

            if !matches!(mensaje_recibido, MensajeFinalizarLlamada::CortoElOtroPeer) {
                continue;
            }

            sender_eventos_llamada
                .send(EventoLlamada::LlamadaFinalizada)
                .map_err(|_| ErrorSesionRTP::ErrorCortandoLlamada)?;

            return Ok(());
        }
    }
}

fn obtener_clones_sockets(conexion: &mut Box<ConexionP2P>) -> Result<SocketsRTP, ErrorSesionRTP> {
    let socket_rtp = conexion
        .socket_rtp_propio
        .clonar()
        .map_err(|_| ErrorSesionRTP::ErrorInterno("Error clonando socket rtp".to_string()))?;
    let socket_rtcp = conexion
        .socket_rtcp_propio
        .clonar()
        .map_err(|_| ErrorSesionRTP::ErrorInterno("Error clonando socket rtcp".to_string()))?;

    Ok(SocketsRTP {
        socket_rtp,
        socket_rtcp,
    })
}
