use crate::comunicacion::telefono::{ErrorTelefono, Telefono};

pub struct TelefonoStub {
    resultado_a_devolver: Result<(), ErrorTelefono>,
}

impl TelefonoStub {
    pub fn new(resultado_a_devolver: Result<(), ErrorTelefono>) -> TelefonoStub {
        TelefonoStub {
            resultado_a_devolver,
        }
    }
}

impl Telefono for TelefonoStub {
    fn atender_llamada(&mut self) -> Result<(), ErrorTelefono> {
        self.resultado_a_devolver.clone()
    }

    fn llamar(&mut self, _usuario: &str) -> Result<(), ErrorTelefono> {
        self.resultado_a_devolver.clone()
    }

    fn rechazar_llamada(&mut self) -> Result<(), ErrorTelefono> {
        self.resultado_a_devolver.clone()
    }
}
