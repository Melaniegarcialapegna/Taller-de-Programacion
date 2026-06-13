use crate::servidor_utils::servidor_central_utils::estado_usuario::EstadoUsuario;
use std::collections::HashMap;

///Mensajes que recibe el handler_usuario de parte del servidor
#[derive(Debug, PartialEq)]
pub enum MensajeUsuario {
    Ok,
    //Error("Nombre invalido")
    //Error("Constraseña invalida")
    Error(String),
    //LlamadaEntrante(quienMeLlamada)
    LlamadaEntrante(String),
    LlamadaRechazada,
    //Me aceptaron la llamada => me piden offer
    PedirOffer,
    //PedirAnswer(offerDelOtro)
    PedirAnswer(String),
    //EnviarAnswer(answerDelOtro)
    EnviarAnswer(String),
    //ActualizarEstadoUsuario(usuario,estado)
    ActualizarEstadoUsuario(String, EstadoUsuario),
    //EstadoUsuarios (vector con usuarios y estado)
    EstadoUsuarios(HashMap<String, EstadoUsuario>),
}
