/// Errores específicos del protocolo SCTP
#[derive(Debug)]
pub enum ErrorSctp {
    YaExisteAsociacionActiva,
    ErrorAlIniciarAsociacion(String),
    ErrorAlIniciarDesdeElEstadoActual(String),
    ErrorHandleAsociacionNoCoincide,
    ErrorIniciarPostDTLS,
    ErrorCanalNoDisponible,
    ErrorEnvioCanal,
}

impl From<ErrorSctp> for String {
    fn from(error: ErrorSctp) -> Self {
        match error {
            ErrorSctp::YaExisteAsociacionActiva => {
                "Ya existe una asociación SCTP activa".to_string()
            }
            ErrorSctp::ErrorAlIniciarAsociacion(msg) => {
                format!("Error al iniciar asociación SCTP: {}", msg)
            }
            ErrorSctp::ErrorAlIniciarDesdeElEstadoActual(msg) => {
                format!(
                    "No se puede iniciar handshake SCTP saliente desde el estado actual: {}",
                    msg
                )
            }
            ErrorSctp::ErrorHandleAsociacionNoCoincide => {
                "El handle de la asociación no coincide con el esperado".to_string()
            }
            ErrorSctp::ErrorIniciarPostDTLS => {
                "No se puede iniciar asociación SCTP después de que DTLS esté activo".to_string()
            }
            ErrorSctp::ErrorCanalNoDisponible => {
                "El canal DCEP no está disponible para iniciar la asociación SCTP".to_string()
            }
            ErrorSctp::ErrorEnvioCanal => "Error al enviar datos por el canal DCEP".to_string(),
        }
    }
}
