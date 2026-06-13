//! Comunicador - Comunicarse con el servidor
//!
//! [Comunicador] representa un objeto capaz de comunicarse con el servidor, enviando y escuchando mensajes del mismo.
//!
//! Todos los comunicadores permiten "crear un par" con el metodo [Comunicador::crear_companiero]. El comunicador original y todos los
//! comunicadores "compañeros" creados con este metodo recibiran los mismos mensajes. Esta herramienta es esencial porque **permite que varios
//! objetos se comuniquen con su propio comunicador con el servidor, evitando que un solo comunicador deba compartirse entre todos**.

use std::fmt::Display;

use crate::protocolos::pca::mensaje::MensajePCA;

#[derive(Debug)]
pub enum ErrorComunicador {
    ErrorRecibido(String),
    ErrorEnElComunicador,
    ErrorDeConexion(String),
}

impl Display for ErrorComunicador {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ErrorComunicador::ErrorDeConexion(e) => f.write_str(&format!("Error de conexion: {e}")),
            ErrorComunicador::ErrorEnElComunicador => f.write_str("Error con el comunicador"),
            ErrorComunicador::ErrorRecibido(error) => {
                f.write_str(&format!("Error recibido: {}", error))
            }
        }
    }
}

pub trait Comunicador: Send + 'static {
    fn crear_companiero(&mut self) -> Result<Box<dyn Comunicador>, ErrorComunicador>;
    fn enviar_mensaje(&mut self, mensaje: &MensajePCA) -> Result<(), ErrorComunicador>;
    fn escuchar_mensaje(&mut self) -> Result<MensajePCA, ErrorComunicador>;

    // Implementado automaticamente
    fn enviar_y_escuchar_respuesta(
        &mut self,
        mensaje_a_enviar: &MensajePCA,
    ) -> Result<MensajePCA, ErrorComunicador> {
        self.enviar_mensaje(mensaje_a_enviar)?;
        self.escuchar_mensaje()
    }
}
