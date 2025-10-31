//! Este modulo define el tipo de dato `ErrorEnServidor`.
use crate::errores::constantes_error_en_servidor::*;
use std::fmt::{Display, Formatter, Result};

/// Representa los posibles errores que pueden surgir en el servidor.
#[derive(Debug, PartialEq)]
pub enum ErrorEnServidor {
    /// Falta un parametro al invocar al programa.
    FaltaParametro,
    /// No se pudo establer la conexion.
    DireccionInvalida,
}

impl Display for ErrorEnServidor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mensaje = match self {
            ErrorEnServidor::FaltaParametro => FALTA_PARAMETRO,
            ErrorEnServidor::DireccionInvalida => DIRECCION_INVALIDA,
        };
        write!(f, "{}", mensaje)
    }
}
