use crate::{
    comunicacion::comunicador::{Comunicador, ErrorComunicador},
    protocolos::pca::mensaje::MensajePCA,
};

pub struct ComunicadorStub {
    mensaje_a_devolver: MensajePCA,
}

impl ComunicadorStub {
    pub fn new(mensaje_a_devolver: MensajePCA) -> ComunicadorStub {
        ComunicadorStub { mensaje_a_devolver }
    }
}

impl Comunicador for ComunicadorStub {
    fn enviar_mensaje(&mut self, _mensaje: &MensajePCA) -> Result<(), ErrorComunicador> {
        Ok(())
    }

    fn escuchar_mensaje(&mut self) -> Result<MensajePCA, ErrorComunicador> {
        if matches!(self.mensaje_a_devolver, MensajePCA::ErrorPCA(_)) {
            return Err(ErrorComunicador::ErrorRecibido("".to_string()));
        }

        Ok(self.mensaje_a_devolver.clone())
    }

    fn crear_companiero(&mut self) -> Result<Box<dyn Comunicador>, ErrorComunicador> {
        Ok(Box::new(ComunicadorStub {
            mensaje_a_devolver: self.mensaje_a_devolver.clone(),
        }))
    }
}
