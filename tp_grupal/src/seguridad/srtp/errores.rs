//! Módulo que define los errores específicos del protocolo SRTP
#[derive(Debug, Clone)]
pub enum ErrorSRTP {
    ErrorClaveSaltInvalida(String),
    ErrorPaqueteDemasiadoCorto(String),
    ErrorCifradoAES(String),
    ErrorHMAC(String),
    ErrorReplay(String),
    ErrorInterno(String),
}

impl From<ErrorSRTP> for String {
    /// Convierte un `ErrorSRTP` en una cadena de texto descriptiva.
    ///
    /// # Arguments
    /// * `error` - El error de tipo `ErrorSRTP` a convertir.
    ///
    /// # Returns
    /// * `String` - Una cadena de texto que describe el error.
    fn from(error: ErrorSRTP) -> Self {
        match error {
            ErrorSRTP::ErrorClaveSaltInvalida(msg) => {
                format!("ERROR SRTP: Clave/Salt inválido - {}", msg)
            }
            ErrorSRTP::ErrorPaqueteDemasiadoCorto(msg) => {
                format!(
                    "ERROR SRTP: El paquete es demasiado corto para ser procesado - {}",
                    msg
                )
            }
            ErrorSRTP::ErrorCifradoAES(msg) => {
                format!("ERROR SRTP: Error durante el cifrado AES - {}", msg)
            }
            ErrorSRTP::ErrorHMAC(msg) => {
                format!(
                    "ERROR SRTP: Error durante el cálculo/verificación HMAC - {}",
                    msg
                )
            }
            ErrorSRTP::ErrorReplay(msg) => {
                format!("ERROR SRTP: Paquete repetido detectado - {}", msg)
            }
            ErrorSRTP::ErrorInterno(msg) => {
                format!("ERROR SRTP: Error interno - {msg}")
            }
        }
    }
}
