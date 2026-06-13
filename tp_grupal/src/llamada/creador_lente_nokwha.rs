use crate::llamada::{
    creador_lente::CreadorDeLente,
    lente::{ErrorLente, Lente},
    lente_nokwha::LenteNokwha,
};

/// Contiene la información necesaria para crear un [LenteNokwha] para una camara especifica.
pub struct CreadorDeLenteNokwha {
    indice_camara: u32,
}

impl CreadorDeLenteNokwha {
    pub fn new(indice_camara: u32) -> CreadorDeLenteNokwha {
        CreadorDeLenteNokwha { indice_camara }
    }
}

impl CreadorDeLente for CreadorDeLenteNokwha {
    fn crear_lente(&mut self) -> Result<Box<dyn Lente>, ErrorLente> {
        let lente = LenteNokwha::new(self.indice_camara).map_err(|_| ErrorLente::ErrorInterno)?;

        Ok(Box::new(lente))
    }

    fn clonar(&mut self) -> Result<Box<dyn CreadorDeLente>, ErrorLente> {
        let creador_nuevo = CreadorDeLenteNokwha::new(self.indice_camara);

        Ok(Box::new(creador_nuevo))
    }
}
