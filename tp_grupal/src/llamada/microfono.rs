//! Microfono - Captura de audio

use std::{
    fmt::Display,
    sync::{Arc, Mutex, mpsc::Sender},
};

use cpal::SizedSample;

#[derive(Debug)]
pub enum ErrorMicrofono {
    ErrorApagandoMicrofono,
    ErrorInterno(String),
}

impl Display for ErrorMicrofono {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorMicrofono::ErrorApagandoMicrofono => f.write_str("Error apagando microfono"),
            ErrorMicrofono::ErrorInterno(error) => f.write_str(&format!("Error interno: {error}")),
        }
    }
}

/// Representa un microfono, que enviara el audio capturado al receptor que
/// se haya suscripto usando [Microfono::cambiar_receptor].
pub trait Microfono<T: SizedSample + Send + 'static>: Send {
    /// Deja de capturar audio del microfono
    fn mutear(&mut self) -> Result<(), ErrorMicrofono>;
    /// Empieza a capturar audio del microfono
    fn desmutear(&mut self) -> Result<(), ErrorMicrofono>;
    /// Cambia el receptor de audio
    fn cambiar_receptor(&mut self, sender_a_receptor: Sender<Vec<T>>)
    -> Result<(), ErrorMicrofono>;
    /// Elimina el receptor de audio actual. Para volver a capturar audio
    /// sera necesario llamar a [Microfono::cambiar_receptor]
    fn borrar_receptor(&mut self) -> Result<(), ErrorMicrofono>;
}

#[derive(Default)]
pub struct MicrofonoMock {
    esta_muteado: bool,
    se_cambio_receptor: bool,
    se_borro_receptor: bool,
}

impl MicrofonoMock {
    pub fn esta_muteado(&self) -> bool {
        self.esta_muteado
    }

    pub fn se_cambio_receptor(&self) -> bool {
        self.se_cambio_receptor
    }
}

impl Microfono<i16> for Arc<Mutex<MicrofonoMock>> {
    fn mutear(&mut self) -> Result<(), ErrorMicrofono> {
        let mut microfono = self.lock().unwrap();
        microfono.esta_muteado = true;
        Ok(())
    }

    fn desmutear(&mut self) -> Result<(), ErrorMicrofono> {
        let mut microfono = self.lock().unwrap();
        microfono.esta_muteado = false;
        Ok(())
    }

    fn cambiar_receptor(
        &mut self,
        _sender_a_receptor: Sender<Vec<i16>>,
    ) -> Result<(), ErrorMicrofono> {
        let mut microfono = self.lock().unwrap();
        microfono.se_cambio_receptor = true;
        Ok(())
    }

    fn borrar_receptor(&mut self) -> Result<(), ErrorMicrofono> {
        let mut microfono = self.lock().unwrap();
        microfono.se_borro_receptor = true;
        Ok(())
    }
}
