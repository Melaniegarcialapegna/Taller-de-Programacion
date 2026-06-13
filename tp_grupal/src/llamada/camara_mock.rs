use crate::{
    llamada::camara::{Camara, ErrorCamara},
    sesion_rtp::comunicacion_rtp::Frame,
};
use std::sync::{Arc, Mutex, mpsc::Sender};

#[derive(Default)]
pub struct CamaraMock {
    se_agrego_capturador: bool,
}

impl CamaraMock {
    pub fn se_agrego_capturador(&self) -> bool {
        self.se_agrego_capturador
    }
}

impl Camara for Arc<Mutex<CamaraMock>> {
    fn agregar_capturador(&mut self, _capturador: Sender<Frame>) -> Result<(), ErrorCamara> {
        let mut camara = self.lock().unwrap();
        camara.se_agrego_capturador = true;

        Ok(())
    }

    fn reiniciar_capturadores(&mut self) -> Result<(), ErrorCamara> {
        Ok(())
    }

    fn encender(&mut self) -> Result<(), ErrorCamara> {
        Ok(())
    }

    fn apagar(&mut self) -> Result<(), ErrorCamara> {
        Ok(())
    }

    fn esta_encendida(&mut self) -> bool {
        false
    }
}
