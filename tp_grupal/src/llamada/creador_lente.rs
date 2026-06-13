use crate::llamada::lente::{ErrorLente, Lente, LenteStud};

/// Representa un objeto capaz de crear instancias de un Lente especifico, y que ademas tiene
/// la informacion necesaria para crear otros creadores de lentes identicos mediante [CreadorDeLente::clonar]
pub trait CreadorDeLente: Send + Sync {
    fn crear_lente(&mut self) -> Result<Box<dyn Lente>, ErrorLente>;
    fn clonar(&mut self) -> Result<Box<dyn CreadorDeLente>, ErrorLente>;
}

pub struct CreadorDeLenteStud {
    frame_a_devolver: Vec<u8>,
}

impl CreadorDeLenteStud {
    pub fn new(frame_a_devolver: Vec<u8>) -> CreadorDeLenteStud {
        CreadorDeLenteStud { frame_a_devolver }
    }
}

impl CreadorDeLente for CreadorDeLenteStud {
    fn crear_lente(&mut self) -> Result<Box<dyn Lente>, ErrorLente> {
        let lente = LenteStud::new(self.frame_a_devolver.clone());
        Ok(Box::new(lente))
    }

    fn clonar(&mut self) -> Result<Box<dyn CreadorDeLente>, ErrorLente> {
        let creador_clonado = CreadorDeLenteStud::new(self.frame_a_devolver.clone());

        Ok(Box::new(creador_clonado))
    }
}
