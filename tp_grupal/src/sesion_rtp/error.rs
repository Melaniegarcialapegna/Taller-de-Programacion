use std::fmt::Display;

#[derive(Debug)]
pub enum ErrorSocketUDP {
    ErrorRecibiendoDatos,
    ErrorEnviandoDatos,
    ErrorClonarSocketUDP,
}
#[derive(Debug)]
pub enum ErrorSesion {
    ComunicacionRTP,
    ComunicacionRTCP,
    ErrorCreandoSesion,
}

impl Display for ErrorSesion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSesion::ComunicacionRTCP => f.write_str("Error en comunicacion RTCP"),
            ErrorSesion::ComunicacionRTP => f.write_str("Error en comunicacion RTP"),
            ErrorSesion::ErrorCreandoSesion => f.write_str("Error creando sesion"),
        }
    }
}

#[derive(Debug)]
pub enum ErrorComunicacionRTP {
    ErrorIniciandoConexion,
    ErrorObtenerLock,
    ErrorEscucharChannel,
    ErrorEnviarChannel,
    ErrorEstablecerTimestamp,
    ErrorEnvioSocketUDP,
    ErrorRecibirSocketUDP,
    ErrorSerializarAPaquete,
    ErrorSRTPProtegiendo,
    ErrorSRTPVerificando,
    ErrorEncodeandoFrame,
    ErrorEnviandoAudio,
    ErrorRecibiendoAudio,
}

pub enum ErrorComunicacionRTCP {
    ErrorIniciandoConexion,
    ErrorRecibiendoMensaje,
    PaqueteInvalidoRecibido,
    ErrorEnviandoMensaje,
    ErrorClonandoSocket,
    ErrorFinalizandoConexionConPeer,
}

impl From<ErrorComunicacionRTP> for String {
    fn from(error: ErrorComunicacionRTP) -> Self {
        match error {
            ErrorComunicacionRTP::ErrorIniciandoConexion => {
                String::from("ERROR: Fallo al inicializar la conexion RTP.")
            }
            ErrorComunicacionRTP::ErrorObtenerLock => String::from("ERROR: Al obtener el lock."),
            ErrorComunicacionRTP::ErrorEscucharChannel => {
                String::from("ERROR: Fallo al recibir informacion de un channel.")
            }
            ErrorComunicacionRTP::ErrorEnviarChannel => {
                String::from("ERROR: Fallo al enviar informacion de un channel.")
            }
            ErrorComunicacionRTP::ErrorEstablecerTimestamp => String::from(
                "ERROR: No fue posible establecer un timestamp para la creacion de un paquete RTP.",
            ),
            ErrorComunicacionRTP::ErrorEnvioSocketUDP => {
                String::from("ERROR: Fallo al enviar informacion por medio de un SocketUDP.")
            }
            ErrorComunicacionRTP::ErrorRecibirSocketUDP => {
                String::from("ERROR: Fallo al recibir informacion por medio de un SocketUDP.")
            }
            ErrorComunicacionRTP::ErrorSerializarAPaquete => {
                String::from("ERROR: No fue posible serializar un paquete RTP.")
            }
            ErrorComunicacionRTP::ErrorSRTPProtegiendo => {
                String::from("ERROR: Fallo al proteger y firmar un paquete RTP con SRTP.")
            }
            ErrorComunicacionRTP::ErrorSRTPVerificando => {
                String::from("ERROR: Fallo al verificar y desproteger un paquete RTP con SRTP.")
            }
            ErrorComunicacionRTP::ErrorEncodeandoFrame => {
                String::from("ERROR: Fallo al encodear un frame")
            }
            ErrorComunicacionRTP::ErrorEnviandoAudio => {
                String::from("ERROR: Fallo al enviar audio")
            }
            ErrorComunicacionRTP::ErrorRecibiendoAudio => {
                String::from("ERROR: Fallo al recibir audio")
            }
        }
    }
}

impl From<ErrorComunicacionRTCP> for String {
    fn from(error: ErrorComunicacionRTCP) -> Self {
        match error {
            ErrorComunicacionRTCP::ErrorClonandoSocket => {
                String::from("ERROR: Fallo al clonar un socket")
            }
            ErrorComunicacionRTCP::ErrorIniciandoConexion => {
                String::from("ERROR: No se pudo iniciar el envio y recepción de mensajes RTCP")
            }
            ErrorComunicacionRTCP::PaqueteInvalidoRecibido => {
                String::from("ERROR: Se recibio un paquete invalido")
            }
            ErrorComunicacionRTCP::ErrorEnviandoMensaje => {
                String::from("ERROR: Fallo al enviar un mensaje RTCP")
            }
            ErrorComunicacionRTCP::ErrorRecibiendoMensaje => {
                String::from("ERROR: Fallo al recibir un mensaje RTCP")
            }
            ErrorComunicacionRTCP::ErrorFinalizandoConexionConPeer => {
                String::from("ERROR: Fallo al finalizar la llamada con otro peer")
            }
        }
    }
}
