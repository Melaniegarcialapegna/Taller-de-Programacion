//! Este modulo define el tipo de dato `ErrorEnThread`.
use crate::errores::constantes_error_en_thread::*;
use crate::logistica::logica_operacion::error_en_operacion::ErrorEnOperacion;
use calculadora_distribuida::constantes::*;
use std::fmt::{Display, Formatter, Result};

/// Representa los posibles errores que pueden surgir al manejar los mensajes que envia el cliente.
#[derive(Debug, PartialEq)]
pub enum ErrorEnThread {
    /// No fue posible duplicar un stream.
    DuplicarStream,
    /// Error al leer del stream.
    LecturaStream,
    /// Error al escribir de un stream.
    EscrituraStream,
    /// Fallo mientras se esperaba que se termine de escribir en el stream.
    EsperarEscritura,
    /// Un cliente cerro la conexion antes de lo esperado.
    CierreAbrupto,
    /// Fallo al intentar obtener el lock de la calculadora.
    ObtenerLock,
    /// En el mensaje enviado por el cliente hay menos parametros de los esperados.
    FaltaCampo,
    /// El mensaje enviado por el cliente es un mensaje que el servidor no sabe responder.
    MensajeRecibidoInvalido,
    /// Fallo en operacion, la operacion enviada por el cliente no es valida
    ErrorOperacion(ErrorEnOperacion),
}

//Pasar a ctes
impl Display for ErrorEnThread {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorEnThread::DuplicarStream => write!(f, "{}", DUPLICAR_STREAM),
            ErrorEnThread::LecturaStream => write!(f, "{}", LECTURA_STREAM),
            ErrorEnThread::EscrituraStream => write!(f, "{}", ESCRITURA_STREAM),
            ErrorEnThread::EsperarEscritura => write!(f, "{}", ESPERAR_ESCRITURA),
            ErrorEnThread::CierreAbrupto => write!(f, "{}", CIERRE_ABRUPTO),
            ErrorEnThread::ObtenerLock => write!(f, "{}", OBTENER_LOCK),
            ErrorEnThread::FaltaCampo => write!(f, "{}", FALTA_CAMPO),
            ErrorEnThread::MensajeRecibidoInvalido => write!(f, "{}", MENSAJE_RECIBIDO_INVALIDO),
            ErrorEnThread::ErrorOperacion(error) => write!(f, "{}", error),
        }
    }
}

impl From<ErrorEnOperacion> for ErrorEnThread {
    fn from(error: ErrorEnOperacion) -> Self {
        ErrorEnThread::ErrorOperacion(error)
    }
}
