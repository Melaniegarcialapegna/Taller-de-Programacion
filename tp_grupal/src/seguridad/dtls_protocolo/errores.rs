//! Módulo que define los errores específicos del protocolo DTLS
#[derive(Debug, Clone)]
pub enum ErrorDTLSProtocolo {
    ErrorGenerandoCertificado,
    ErrorFingerprintNoCoincide { esperado: String, recibido: String },
    ErrorFingerprintInvalida,
    ErrorFingerprintAlgoritmoNoSoportado,
    ErrorFingerprintRemotoInexistente,
    ErrorRolNoEstablecido,
    ErrorEnviandoMensaje,
    ErrorRecibiendoMensaje,
    ErrorCertificadoLocalNoDisponible,
    ErrorCertificadoRemotoNoDisponible,
    ErrorCertificadoRemotoInexistente,
    ErrorSetupInvalido,
    ErrorSetupInexistente,
    ErrorAnswererConActpass,
    ErrorCertificadoInvalido,
    ErrorHandshakeDTLS,
    ErrorSocketDTLS,
    ErrorRolIncorrecto,
    ErrorObteniendoSocketRtpParaDtls,
    ErrorObteniendoSocketRtcpParaDtls,
    ErrorStreamDTLS,
    ClavesSRTPNoDisponibles,
    ErrorIniciarPostDTLS,
}

impl From<ErrorDTLSProtocolo> for String {
    /// Convierte un `ErrorDTLSProtocolo` en una cadena de texto descriptiva.
    ///
    /// # Arguments
    /// * `error` - El error de tipo `ErrorDTLSProtocolo` a convertir.
    ///
    /// # Returns
    /// * `String` - Una cadena de texto que describe el error.
    fn from(error: ErrorDTLSProtocolo) -> Self {
        match error {
            ErrorDTLSProtocolo::ErrorGenerandoCertificado => {
                String::from("ERROR: No se pudo generar el certificado DTLS")
            }
            ErrorDTLSProtocolo::ErrorFingerprintNoCoincide { esperado, recibido } => {
                format!(
                    "ERROR: La fingerprint no coincide. Esperado: {}, Recibido: {}",
                    esperado, recibido
                )
            }
            ErrorDTLSProtocolo::ErrorFingerprintInvalida => {
                String::from("ERROR: La fingerprint es inválida")
            }
            ErrorDTLSProtocolo::ErrorFingerprintAlgoritmoNoSoportado => {
                String::from("ERROR: El algoritmo de la fingerprint no es soportado")
            }
            ErrorDTLSProtocolo::ErrorFingerprintRemotoInexistente => String::from(
                "ERROR: No se encontró la fingerprint remota en la descripción de sesión",
            ),
            ErrorDTLSProtocolo::ErrorRolNoEstablecido => {
                String::from("ERROR: El rol DTLS no ha sido establecido")
            }
            ErrorDTLSProtocolo::ErrorEnviandoMensaje => {
                String::from("ERROR: Fallo al enviar mensaje DTLS")
            }
            ErrorDTLSProtocolo::ErrorRecibiendoMensaje => {
                String::from("ERROR: Fallo al recibir mensaje DTLS")
            }
            ErrorDTLSProtocolo::ErrorCertificadoLocalNoDisponible => {
                String::from("ERROR: El certificado local DTLS no está disponible")
            }
            ErrorDTLSProtocolo::ErrorCertificadoRemotoNoDisponible => {
                String::from("ERROR: El certificado remoto DTLS no está disponible")
            }
            ErrorDTLSProtocolo::ErrorSetupInvalido => {
                String::from("ERROR: El valor de setup DTLS es inválido")
            }
            ErrorDTLSProtocolo::ErrorSetupInexistente => String::from(
                "ERROR: No se encontró el valor de setup DTLS en la descripción de sesión",
            ),
            ErrorDTLSProtocolo::ErrorAnswererConActpass => {
                String::from("ERROR: El answerer no puede tener el valor 'actpass' en setup DTLS")
            }
            ErrorDTLSProtocolo::ErrorCertificadoInvalido => {
                String::from("ERROR: El certificado DTLS es inválido")
            }
            ErrorDTLSProtocolo::ErrorHandshakeDTLS => {
                String::from("ERROR: Fallo durante el handshake DTLS")
            }
            ErrorDTLSProtocolo::ErrorSocketDTLS => String::from("ERROR: Fallo en el socket DTLS"),
            ErrorDTLSProtocolo::ErrorCertificadoRemotoInexistente => {
                String::from("ERROR: No se encontró el certificado remoto DTLS")
            }
            ErrorDTLSProtocolo::ErrorRolIncorrecto => {
                String::from("ERROR: El rol DTLS establecido es incorrecto para la operación")
            }
            ErrorDTLSProtocolo::ErrorObteniendoSocketRtpParaDtls => {
                String::from("ERROR: No se pudo obtener el socket RTP para DTLS")
            }
            ErrorDTLSProtocolo::ErrorObteniendoSocketRtcpParaDtls => {
                String::from("ERROR: No se pudo obtener el socket RTCP para DTLS")
            }
            ErrorDTLSProtocolo::ErrorStreamDTLS => {
                String::from("ERROR: Fallo al crear el stream DTLS")
            }
            ErrorDTLSProtocolo::ClavesSRTPNoDisponibles => {
                String::from("ERROR: Las claves SRTP no están disponibles")
            }
            ErrorDTLSProtocolo::ErrorIniciarPostDTLS => {
                String::from("ERROR: Fallo al iniciar la fase post-DTLS")
            }
        }
    }
}

impl std::fmt::Display for ErrorDTLSProtocolo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}
