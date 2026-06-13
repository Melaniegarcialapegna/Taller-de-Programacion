use std::{cell::RefCell, rc::Rc};

use crate::comunicacion::telefono::{ErrorTelefono, Telefono};

#[derive(Default)]
pub struct TelefonoMock {
    se_recibio_llamar: bool,
    se_recibio_rechazar: bool,
    se_recibio_atender: bool,
}

impl TelefonoMock {
    pub fn se_recibio_llamar(&self) -> bool {
        self.se_recibio_llamar
    }

    pub fn se_recibio_rechazar(&self) -> bool {
        self.se_recibio_rechazar
    }

    pub fn se_recibio_atender(&self) -> bool {
        self.se_recibio_atender
    }
}

impl Telefono for Rc<RefCell<TelefonoMock>> {
    fn atender_llamada(&mut self) -> Result<(), ErrorTelefono> {
        self.borrow_mut().se_recibio_atender = true;

        Ok(())
    }

    fn llamar(&mut self, _usuario: &str) -> Result<(), ErrorTelefono> {
        self.borrow_mut().se_recibio_llamar = true;

        Ok(())
    }

    fn rechazar_llamada(&mut self) -> Result<(), ErrorTelefono> {
        self.borrow_mut().se_recibio_rechazar = true;

        Ok(())
    }
}
