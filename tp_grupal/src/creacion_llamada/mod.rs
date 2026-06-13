//! # CreadorDeConexion - Crear conexiones entre peers.
//!
//! En este modulo se encontraran los objetos capaces de crear una conexion P2P con otro peer. Para lograrlo, se debera usar un [CreadorDeConexionP2P].
//! Un objeto que cumpla con el trait CreadorDeConexionP2P debe ser capaz de comunicarse mediante offers y answers con otro peer. El medio por el cual se intercambian
//! los offers y answers es independiente de este objeto. En nuestra implementación, los offers y answers se compartiran usando al servidor de signaling como mediador,
//! y el objeto que tiene la responsabilidad de intercambiar los offers y answers entre el servidor y la aplicación es [MediadorDeConexionesP2P](crate::creacion_llamada::mediador_de_conexiones::MediadorDeConexionesP2P)

use bytes::Bytes;
use std::{
    fmt::Display,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    protocolos::sctp::evento_sctp::EventoSctp, seguridad::srtp::srtp_contexto::SRTPContexto,
    sesion_rtp::socket_udp::SocketUDP,
};

pub mod creador_de_conexion_encriptada;
#[cfg(test)]
pub mod creador_de_conexion_mock;
pub mod mediador_de_conexiones;

#[derive(Debug)]
/// Errores que pueden ocurrir durante la creación de una conexión P2P usando un [CreadorDeConexionP2P].
pub enum ErrorCreadorDeConexion {
    ErrorInterno(String),
    ErrorConComunicador,
    ErrorComunicandoAAPlicacion,
    ErrorComunicandoALlamada,
}

impl Display for ErrorCreadorDeConexion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCreadorDeConexion::ErrorConComunicador => f.write_str("Error con comunicador"),
            ErrorCreadorDeConexion::ErrorComunicandoALlamada => {
                f.write_str("Error comunicando a llamada")
            }
            ErrorCreadorDeConexion::ErrorComunicandoAAPlicacion => {
                f.write_str("Error comunicando a aplicacion")
            }
            ErrorCreadorDeConexion::ErrorInterno(error) => {
                f.write_str(&format!("Error interno: {error}"))
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)] //todo sacar cuando ya se este usando
/// Representa una **conexión RTP establecida** entre dos *peers*, definida por
/// los dos sockets propios (para RTP y RTCP), y la direccion de los dos sockets externos.
pub struct ConexionP2P {
    pub socket_rtp_propio: Box<dyn SocketUDP>,
    pub socket_rtcp_propio: Box<dyn SocketUDP>,
    pub direccion_rtp_externa: String,
    pub direccion_rtcp_externa: String,
    contexto_srtp_tx: Option<SRTPContexto>,
    contexto_srtp_rx: Option<SRTPContexto>,
    pub srtp_rx: Option<Receiver<Vec<u8>>>,
    pub tx_datos_sctp: Option<Sender<Bytes>>,
    pub rx_eventos_sctp: Option<Receiver<EventoSctp>>,
}

pub enum ErrorConexionP2P {
    ErrorContextosSRTPInvalidos,
}

impl ConexionP2P {
    /// Crea una nueva conexión P2P con los sockets y direcciones externas dadas, y opcionalmente con los contextos SRTP para transmisión y recepción.
    pub fn new(
        socket_rtp_propio: Box<dyn SocketUDP>,
        socket_rtcp_propio: Box<dyn SocketUDP>,
        direccion_rtp_externa: String,
        direccion_rtcp_externa: String,
        contexto_srtp_tx: Option<SRTPContexto>,
        contexto_srtp_rx: Option<SRTPContexto>,
        srtp_rx: Option<Receiver<Vec<u8>>>,
    ) -> ConexionP2P {
        ConexionP2P {
            socket_rtp_propio,
            socket_rtcp_propio,
            direccion_rtp_externa,
            direccion_rtcp_externa,
            contexto_srtp_tx,
            contexto_srtp_rx,
            srtp_rx,
            tx_datos_sctp: None,
            rx_eventos_sctp: None,
        }
    }

    /// Devuelve el contexto SRTP para transmisión, o un error si no es válido.
    pub fn contexto_srtp_tx(&mut self) -> Result<SRTPContexto, ErrorConexionP2P> {
        let contexto = self
            .contexto_srtp_tx
            .take()
            .ok_or(ErrorConexionP2P::ErrorContextosSRTPInvalidos)?;

        Ok(contexto)
    }

    /// Establece el contexto SRTP para transmisión.
    pub fn set_tx_datos_sctp(&mut self, tx: Sender<Bytes>) {
        self.tx_datos_sctp = Some(tx);
    }

    /// Establece el receptor de eventos SCTP.
    pub fn set_rx_eventos_sctp(&mut self, rx: Receiver<EventoSctp>) {
        self.rx_eventos_sctp = Some(rx);
    }

    /// Devuelve el contexto SRTP para recepción, o un error si no es válido.
    pub fn contexto_srtp_rx(&mut self) -> Result<SRTPContexto, ErrorConexionP2P> {
        let contexto = self
            .contexto_srtp_rx
            .take()
            .ok_or(ErrorConexionP2P::ErrorContextosSRTPInvalidos)?;

        Ok(contexto)
    }
}

pub trait CreadorDeConexionP2P: Send {
    /// Genera un offer para ser enviado al otro peer
    fn generar_offer(&mut self) -> Result<String, ErrorCreadorDeConexion>;
    /// Genera un answer a partir de un offer recibido
    fn generar_answer(&mut self, offer: &str) -> Result<String, ErrorCreadorDeConexion>;
    /// Recibe y registra un answer recibido de otro peer
    fn recibir_answer(&mut self, answer: &str) -> Result<(), ErrorCreadorDeConexion>;
    /// Realiza la negociacion y potencial conexion con el otro peer usando los offers y answers compartidos.
    ///
    /// PRE: Ya se realizo el intercambio de offers y answers.
    fn conectar(&mut self) -> Result<(), ErrorCreadorDeConexion>;
    /// Devuelve los dos sockets propios de la conexion establecida, junto a las dos direcciones de los sockets
    /// del peer externo.
    ///
    /// PRE: Ya se creo la conexión entre peers.
    fn obtener_sockets(&mut self) -> Result<ConexionP2P, ErrorCreadorDeConexion>;
}
