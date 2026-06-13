use std::sync::{Arc, Mutex};

use crate::{
    comunicacion::comunicador::{Comunicador, ErrorComunicador},
    protocolos::pca::mensaje::MensajePCA,
};

pub struct ComunicadorMockYStud {
    mensajes_enviados: Vec<MensajePCA>,
    mensaje_a_devolver: MensajePCA,
}

impl ComunicadorMockYStud {
    pub fn new(mensaje_a_devolver: MensajePCA) -> ComunicadorMockYStud {
        ComunicadorMockYStud {
            mensajes_enviados: vec![],
            mensaje_a_devolver,
        }
    }

    pub fn se_envio_mensaje(&self, mensaje: MensajePCA) -> bool {
        self.mensajes_enviados.contains(&mensaje)
    }

    pub fn cantidad_mensajes_enviados(&self) -> usize {
        self.mensajes_enviados.len()
    }
}

impl Comunicador for Arc<Mutex<ComunicadorMockYStud>> {
    fn crear_companiero(&mut self) -> Result<Box<dyn Comunicador>, ErrorComunicador> {
        Ok(Box::new(Arc::clone(self)))
    }

    fn enviar_mensaje(&mut self, mensaje: &MensajePCA) -> Result<(), ErrorComunicador> {
        let mut comunicador = self
            .lock()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
        comunicador.mensajes_enviados.push(mensaje.clone());

        Ok(())
    }

    fn enviar_y_escuchar_respuesta(
        &mut self,
        mensaje_a_enviar: &MensajePCA,
    ) -> Result<MensajePCA, ErrorComunicador> {
        let mut comunicador = self
            .lock()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
        comunicador.mensajes_enviados.push(mensaje_a_enviar.clone());

        Ok(comunicador.mensaje_a_devolver.clone())
    }

    fn escuchar_mensaje(&mut self) -> Result<MensajePCA, ErrorComunicador> {
        let comunicador = self
            .lock()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
        Ok(comunicador.mensaje_a_devolver.clone())
    }
}
