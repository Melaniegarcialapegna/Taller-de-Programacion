use crate::errores::error_en_cliente::ErrorEnCliente;

//constantes
const MENSAJE_OK: &str = "OK";
const MENSAJE_VALUE: &str = "VALUE";

/// Procesa la respuesta que el servidor le envia.
#[derive(PartialEq, Eq, Debug)]
pub enum RespuestaServidor {
    ///Respuesta de aplicar operacion exitosa.
    OK,
    /// Respuesta del Get
    Valor(String),
    /// Respuesta para algun fallo.
    Error(String),
}

impl RespuestaServidor {
    pub fn generar_respuesta(respuesta: &str) -> Result<Self, ErrorEnCliente> {
        if respuesta == MENSAJE_OK {
            Ok(RespuestaServidor::OK)
        } else if respuesta.starts_with(MENSAJE_VALUE) {
            let mut campos = respuesta.split_whitespace();
            let _ = campos.next().ok_or(ErrorEnCliente::RespuestaServidor)?;
            let valor: &str = campos.next().ok_or(ErrorEnCliente::RespuestaServidor)?;
            Ok(RespuestaServidor::Valor(valor.to_string()))
        } else {
            Ok(RespuestaServidor::Error(respuesta.to_string()))
        }
    }
}
