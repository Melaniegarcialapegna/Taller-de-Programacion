use std::sync::{Arc, Mutex};

use eframe::egui::ColorImage;

use crate::reproductor::{ErrorReproductor, Reproductor};

#[derive(Default)]
pub struct ReproductorMock {
    se_pidio_proximo_frame: bool,
}

impl ReproductorMock {
    pub fn se_pidio_proximo_frame(&self) -> bool {
        self.se_pidio_proximo_frame
    }
}

impl Reproductor for Arc<Mutex<ReproductorMock>> {
    fn proximo_frame(&mut self) -> Result<ColorImage, ErrorReproductor> {
        let mut reproductor = self.lock().unwrap();

        reproductor.se_pidio_proximo_frame = true;
        Ok(ColorImage::example())
    }

    fn esta_procesando_frame(&mut self) -> Result<Arc<Mutex<bool>>, ErrorReproductor> {
        Ok(Arc::new(Mutex::new(false)))
    }
}
