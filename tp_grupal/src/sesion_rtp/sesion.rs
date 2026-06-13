use super::comunicacion_rtcp::iniciar_comunicacion_rtcp;
use super::comunicacion_rtp::RtpIo;
use super::comunicacion_rtp::iniciar_comunicacion_rtp;
use super::error::ErrorSesion;
use crate::creacion_llamada::ConexionP2P;
use crate::llamada::sesion_rtp::MensajeFinalizarLlamada;
use crate::logger::Logger;
use crate::protocolos::rtcp::tipo_paquete::ContenidoReport;
use crate::seguridad::srtp::srtp_contexto::SRTPContexto;
use crate::sesion_rtp::comunicacion_rtp::ComunicadoresConIO;
use rand::Rng;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub type ContextosSRTP = (SRTPContexto, SRTPContexto);
pub type ByeChannels = (
    Receiver<MensajeFinalizarLlamada>,
    Sender<MensajeFinalizarLlamada>,
);

///Estructura que persiste la informacion de la media identificada por un ssrc durante una sesion.
///De momento unicamente se hace de video
pub struct EstadisticasSender {
    pub ssrc: u32,
    pub cantidad_paquetes_enviados: u32,
    pub cantidad_bytes_enviados: u32,
    pub ultimo_numero_secuencia: u16,
    pub ultimo_timestamp_enviado: u32,
}

impl EstadisticasSender {
    pub fn new(ssrc: u32) -> EstadisticasSender {
        EstadisticasSender {
            ssrc,
            cantidad_paquetes_enviados: 0,
            cantidad_bytes_enviados: 0,
            ultimo_numero_secuencia: 0,
            ultimo_timestamp_enviado: 0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
///Estructura que persiste la informacion de la informacion que recibe de otras medias durante una sesion.
pub struct EstadisticasReceiver {
    pub cantidad_paquetes_recibidos: u32,
    pub cantidad_paquetes_recibidos_anterior: u32,
    pub cantidad_paquetes_esperados_anterior: u32,
    pub contenido_report: ContenidoReport,
}

impl EstadisticasReceiver {
    pub fn new(ssrc: u32) -> EstadisticasReceiver {
        let contenido_report = ContenidoReport::crear_vacio_con_ssrc(ssrc);
        EstadisticasReceiver {
            contenido_report,
            cantidad_paquetes_esperados_anterior: 0,
            cantidad_paquetes_recibidos: 0,
            cantidad_paquetes_recibidos_anterior: 0,
        }
    }
}

///Se inicia una Sesion RTP que se encargara de crear las estructuras necesarias para la sincronizacion/gestion de esta.
pub fn crear_sesion_rtp(
    mut conexion: Box<ConexionP2P>,
    sender_estadisticas: Sender<EstadisticasReceiver>,
    puntas_channels: ComunicadoresConIO,
    logger: Logger,
    procesando_frame: Arc<Mutex<bool>>,
    bye_channel: ByeChannels,
    srtp_rx: Receiver<Vec<u8>>,
) -> Result<(), ErrorSesion> {
    logger.info("Iniciando sesion RTP", "Sesion RTP");

    // Obtengo
    let (contexto_srtp_tx, contexto_srtp_rx) = obtener_contextos_srtp(&mut conexion)?;

    let mut rng = rand::thread_rng();

    // Creo estadisticas como sender de video y audio
    let ssrc_sesion_video: u32 = rng.gen_range(0..100);
    let estadisticas_sender_video = EstadisticasSender::new(ssrc_sesion_video);
    let estadisticas_sender_audio = EstadisticasSender::new(ssrc_sesion_video + 1);

    let referencia_estadisticas_video = Arc::new(Mutex::new(estadisticas_sender_video));
    let referencia_estadisticas_audio = Arc::new(Mutex::new(estadisticas_sender_audio));
    let dicc_ssrc: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let clon_logger = logger.clone();
    let clon_logger_rtcp = logger.clone();

    iniciar_comunicacion_rtcp(
        clon_logger_rtcp,
        conexion.socket_rtcp_propio,
        &conexion.direccion_rtcp_externa,
        Arc::clone(&referencia_estadisticas_video),
        Arc::clone(&dicc_ssrc),
        bye_channel,
        sender_estadisticas,
    )
    .map_err(|_| ErrorSesion::ComunicacionRTCP)?;

    let contextos_srtp = (contexto_srtp_tx, contexto_srtp_rx);
    let estadisticas = (
        referencia_estadisticas_video,
        referencia_estadisticas_audio,
        Arc::clone(&dicc_ssrc),
    );

    let rtp_io = RtpIo {
        socket: conexion.socket_rtp_propio,
        srtp_rx, // este parametro nuevo viene del demux, es x donde se reciben los paquetes srtp y reemplaza al socket q teniamos antes
    };

    iniciar_comunicacion_rtp(
        clon_logger,
        rtp_io,
        &conexion.direccion_rtp_externa,
        puntas_channels,
        estadisticas,
        procesando_frame,
        contextos_srtp,
    )
    .map_err(|_| ErrorSesion::ComunicacionRTP)?;

    logger.info(
        "Sesion RTP iniciada correctamente con contextos SRTP incluidos",
        "sesion RTP",
    );

    Ok(())
}

fn obtener_contextos_srtp(
    conexion: &mut ConexionP2P,
) -> Result<(SRTPContexto, SRTPContexto), ErrorSesion> {
    let contexto_srtp_tx = conexion
        .contexto_srtp_tx()
        .map_err(|_| ErrorSesion::ErrorCreandoSesion)?;
    let contexto_srtp_rx = conexion
        .contexto_srtp_rx()
        .map_err(|_| ErrorSesion::ErrorCreandoSesion)?;
    Ok((contexto_srtp_tx, contexto_srtp_rx))
}
