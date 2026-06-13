use std::{cell::RefCell, rc::Rc};

use crate::llamada::{ErrorLlamada, Llamada};

#[derive(Default)]
pub struct LlamadaMock {
    se_pidio_enviar_frame: bool,
}

impl LlamadaMock {
    pub fn se_pidio_enviar_frame(&self) -> bool {
        self.se_pidio_enviar_frame
    }
}

impl Llamada for Rc<RefCell<LlamadaMock>> {
    fn enviar_proximo_frame(&mut self) -> Result<(), ErrorLlamada> {
        let mut llamada_mock = self.borrow_mut();
        llamada_mock.se_pidio_enviar_frame = true;

        Ok(())
    }

    fn cortar_llamada(&mut self) -> Result<(), ErrorLlamada> {
        Ok(())
    }

    fn cambiar_camara(
        &mut self,
        _nueva_camara: Box<dyn super::camara::Camara>,
    ) -> Result<(), ErrorLlamada> {
        Ok(())
    }

    fn desmutear_microfono(&mut self) -> Result<(), ErrorLlamada> {
        Ok(())
    }

    fn mutear_microfono(&mut self) -> Result<(), ErrorLlamada> {
        Ok(())
    }

    fn enviar_archivo(&mut self, _path: std::path::PathBuf) -> Result<(), ErrorLlamada> {
        Ok(())
    }

    fn aceptar_archivo(&mut self) -> Result<(), ErrorLlamada> {
        Ok(())
    }

    fn rechazar_archivo(&mut self) -> Result<(), ErrorLlamada> {
        Ok(())
    }
}
