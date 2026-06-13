//! Lente - Capturar video de una fuente
//!
//! La idea de que haya un Lente es que para obtener video de una nueva fuente solo haga falta implementar el metodo [Lente::obtener_frame()].
//! Luego, cualquier lente podra ser usado con una [CamaraGenerica](crate::llamada::camara::CamaraGenerica)
//! para transmitir el video captado. En ese caso, tambien se debera crear un CreadorDeLente, que es simplemente un objeto que tiene la información
//! para crear instancias del Lente deseado.

use std::{fmt::Display, thread, time::Duration};

use crate::sesion_rtp::comunicacion_rtp::Frame;

pub enum ErrorLente {
    /// Error en el Lente
    ErrorInterno,
    /// Error al obtener un frame
    ErrorObteniendoFrame,
}

impl Display for ErrorLente {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Error interno")
    }
}

/// Representa el lente de una [CamaraGenerica](crate::llamada::camara::CamaraGenerica)
pub trait Lente {
    /// Obtiene un frame de la fuente de video captada por el lente.
    fn obtener_frame(&mut self) -> Result<Frame, ErrorLente>;
}

pub struct LenteStud {
    frame_a_devolver: Vec<u8>,
}

impl LenteStud {
    pub fn new(frame_a_devolver: Vec<u8>) -> LenteStud {
        LenteStud { frame_a_devolver }
    }
}

impl Lente for LenteStud {
    fn obtener_frame(&mut self) -> Result<Frame, ErrorLente> {
        thread::sleep(Duration::from_millis(33)); // Simular 30 frames por segundo aprox.

        let bytes = self.frame_a_devolver.clone();
        let frame = Frame::new(bytes, 20, 20);

        Ok(frame)
    }
}
