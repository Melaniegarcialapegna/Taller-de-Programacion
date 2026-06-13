use std::{fmt::Display, sync::mpsc::Receiver};

#[derive(Debug)]
pub enum ErrorReproductorAudio {
    ErrorCreandoReproductor(String),
    ErrorInterno(String),
}

impl Display for ErrorReproductorAudio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorReproductorAudio::ErrorCreandoReproductor(e) => {
                f.write_str(&format!("Error creando reproductor de audio: {e}"))
            }
            ErrorReproductorAudio::ErrorInterno(e) => {
                f.write_str(&format!("Error interno en el reproductor de audio{e}"))
            }
        }
    }
}

pub trait ReproductorAudio: Send {
    fn iniciar_reproduccion(
        receiver: Receiver<Vec<i16>>,
    ) -> Result<Box<dyn ReproductorAudio>, ErrorReproductorAudio>
    where
        Self: Sized;

    fn despausar(&mut self) -> Result<(), ErrorReproductorAudio>;
    fn pausar(&mut self) -> Result<(), ErrorReproductorAudio>;
}

pub struct ReproductorAudioDummy {
    #[allow(dead_code)]
    receiver: Receiver<Vec<i16>>,
}

impl ReproductorAudio for ReproductorAudioDummy {
    fn iniciar_reproduccion(
        receiver: Receiver<Vec<i16>>,
    ) -> Result<Box<dyn ReproductorAudio>, ErrorReproductorAudio> {
        Ok(Box::new(ReproductorAudioDummy { receiver }))
    }

    fn despausar(&mut self) -> Result<(), ErrorReproductorAudio> {
        Ok(())
    }

    fn pausar(&mut self) -> Result<(), ErrorReproductorAudio> {
        Ok(())
    }
}
