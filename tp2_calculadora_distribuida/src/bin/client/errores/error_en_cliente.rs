//! Este modulo define el tipo de dato `ErrorEnCliente`.

use crate::errores::constantes_error_en_cliente::*;
use calculadora_distribuida::constantes::*;
use std::fmt::{Display, Formatter, Result};

/// Representa los posibles errores que pueden surgir en el cliente.
#[derive(Debug, PartialEq)]
pub enum ErrorEnCliente {
    /// Falta un parametro al invocar al programa.
    FaltaParametro,
    /// El archivo enviado por parametro no existe.
    ArchivoInexistente,
    /// No se pudo establer la conexion.
    DireccionInvalida,
    /// Error al intentar abrir el archivo pasado por parametro.
    AbrirArchivo,
    /// Fallo al intentar leer una linea en el archivo.
    LeerLineaArchivo,
    /// No fue posible duplicar un stream.
    DuplicarStream,
    /// Error al escribir de un stream.
    EscrituraStream,
    /// Fallo mientras se esperaba que se termine de escribir en el stream.
    EsperarEscritura,
    /// Error al leer del stream.
    LecturaStream,
    /// Se produjo algun tipo de fallo al intentar procesar la respuesta del servidor.
    RespuestaServidor,
}

//Pasar a ctes
impl Display for ErrorEnCliente {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mensaje: &'static str = match self {
            ErrorEnCliente::FaltaParametro => FALTA_PARAMETRO,
            ErrorEnCliente::ArchivoInexistente => ARCHIVO_INEXISTENTE,
            ErrorEnCliente::DireccionInvalida => DIRECCION_INVALIDA,
            ErrorEnCliente::AbrirArchivo => ABRIR_ARCHIVO,
            ErrorEnCliente::LeerLineaArchivo => LEER_LINEA_ARCHIVO,
            ErrorEnCliente::DuplicarStream => DUPLICAR_STREAM,
            ErrorEnCliente::EscrituraStream => ESCRITURA_STREAM,
            ErrorEnCliente::EsperarEscritura => ESPERAR_ESCRITURA,
            ErrorEnCliente::LecturaStream => LECTURA_STREAM,
            ErrorEnCliente::RespuestaServidor => RESPUESTA_SERVIDOR,
        };
        write!(f, "{}", mensaje)
    }
}
