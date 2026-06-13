use crate::servidor_utils::mensajes::mensaje_usuario::MensajeUsuario;
use std::fmt;
use std::{fmt::Display, sync::mpsc::Sender};

//Mensajes que recibe el servidor de parte del handler usuario

#[derive(Debug)]
pub enum MensajeServidor {
    //Registrar(nombre,contraseña,tx_usuario)
    Registrar(String, String, Sender<MensajeUsuario>),
    //Loguear(nombre,contraseña,tx_usuario)
    Loguear(String, String, Sender<MensajeUsuario>),
    //Llamar(quienLlama,aQuienLlama)
    Llamar(String, String),
    //AceptarLlamada(aQuienLeAcepta)
    AceptarLlamada(String),
    //RechazarLlamada(aQuienLeRechazoLlamada)
    RechazarLlamada(String),
    //EnviarOffer(aQuienEnvioOffer,offer)
    EnviarOffer(String, String),
    //EnviarAnswer(quienLaEnvia,aQuienEnvioAnswer,Answer)
    EnviarAnswer(String, String, String), //los pone en ocupados
    //EstadoDisponible(quienSoy) => luego de cortar
    EstadoDisponible(String),
    //Desconectarse(quienSoy)
    Desconectarse(String),
}

//implemento Display para testeo
impl Display for MensajeServidor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MensajeServidor::Registrar(usuario, contrasenia, _) => {
                write!(f, "REGISTRAR {};{};sender", usuario, contrasenia)
            }
            MensajeServidor::Loguear(usuario, contrasenia, _) => {
                write!(f, "LOGUEAR {};{};sender", usuario, contrasenia)
            }
            MensajeServidor::Llamar(quien_llama, quien_es_llamado) => {
                write!(f, "LLAMAR {} siendo {}", quien_es_llamado, quien_llama)
            }
            MensajeServidor::AceptarLlamada(a_quien_se_le_acepta_llamada) => {
                write!(f, "LLAMADA ACEPTADA a {}", a_quien_se_le_acepta_llamada)
            }
            MensajeServidor::RechazarLlamada(a_quien_se_le_rechaza_llamada) => {
                write!(f, "LLAMADA RECHAZADA a {}", a_quien_se_le_rechaza_llamada)
            }
            MensajeServidor::EnviarOffer(a_quien_envio_offer, offer) => {
                write!(f, "ENVIO a {} OFFER : {}", a_quien_envio_offer, offer)
            }
            MensajeServidor::EnviarAnswer(quien_la_envia, a_quien_se_envia, answer) => write!(
                f,
                "ENVIO a {} siendo {} ANSWER : {}",
                a_quien_se_envia, quien_la_envia, answer
            ),
            MensajeServidor::EstadoDisponible(usuario) => {
                write!(f, "ACTUALIZAR A DISPONIBLE {}", usuario)
            }
            MensajeServidor::Desconectarse(usuario) => write!(f, "DESCONECTAR a {}", usuario),
        }
    }
}
