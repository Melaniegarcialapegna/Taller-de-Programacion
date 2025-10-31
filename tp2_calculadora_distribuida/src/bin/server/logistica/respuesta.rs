//! Este modulo define el tipo de dato `Respuesta`.
use crate::errores::error_en_thread::ErrorEnThread;
use crate::logistica::logica_calculadora::error_en_calculadora::ErrorEnCalculadora;
use std::fmt::Display;

//constantes
const MENSAJE_OK: &str = "OK";
const MENSAJE_VALUE: &str = "VALUE";

/// Respuesta que se le enviara al cliente.
#[derive(PartialEq, Eq, Debug)]
pub enum Respuesta {
    ///Respuesta de aplicar operacion exitosa.
    OK,
    /// Respuesta del Get
    Valor(u8),
    /// Respuesta para algun fallo.
    Error(String),
}

impl Display for Respuesta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Respuesta::OK => write!(f, "{}", MENSAJE_OK),
            Respuesta::Valor(valor) => {
                write!(f, "{} {}", MENSAJE_VALUE, valor)
            }
            Respuesta::Error(razon) => write!(f, "{}", razon),
        }
    }
}

impl From<ErrorEnThread> for Respuesta {
    fn from(error: ErrorEnThread) -> Self {
        Respuesta::Error(error.to_string())
    }
}

impl From<ErrorEnCalculadora> for Respuesta {
    fn from(error: ErrorEnCalculadora) -> Self {
        Respuesta::Error(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_respuesta() {
        test_respuesta_generico(Respuesta::OK, "OK");

        test_respuesta_generico(Respuesta::Valor(4), "VALUE 4");

        test_respuesta_generico(
            Respuesta::from(ErrorEnThread::FaltaCampo),
            "ERROR \"missing parameters\"",
        );

        let error = String::from("Error de lectura");
        test_respuesta_generico(Respuesta::Error(error), "Error de lectura");
    }

    fn test_respuesta_generico(respuesta: Respuesta, respuesta_esperada: &str) {
        assert_eq!(respuesta.to_string(), respuesta_esperada);
    }
}
