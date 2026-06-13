use std::sync::{Arc, Mutex, mpsc::Receiver};

use eframe::egui::{Color32, ColorImage};

use crate::{
    reproductor::{ErrorReproductor, Reproductor},
    sesion_rtp::comunicacion_rtp::Frame,
};

pub struct ReproductorSinDecoder {
    receiver_frames: Receiver<Frame>,
    ultimo_frame: ColorImage,
}

impl ReproductorSinDecoder {
    pub fn new(receiver_frames: Receiver<Frame>) -> ReproductorSinDecoder {
        ReproductorSinDecoder {
            receiver_frames,
            ultimo_frame: Self::imagen_por_defecto(),
        }
    }

    fn imagen_por_defecto() -> ColorImage {
        ColorImage::filled([120, 120], Color32::from_rgb(6, 1, 20))
    }
}

impl Reproductor for ReproductorSinDecoder {
    fn proximo_frame(&mut self) -> Result<ColorImage, ErrorReproductor> {
        while let Ok(frame) = self.receiver_frames.try_recv() {
            self.ultimo_frame = ColorImage::from_rgb([frame.anchura, frame.altura], &frame.bytes);
        }

        Ok(self.ultimo_frame.clone())
    }

    fn esta_procesando_frame(&mut self) -> Result<Arc<Mutex<bool>>, ErrorReproductor> {
        Ok(Arc::new(Mutex::new(false)))
    }
}

#[cfg(test)]
use std::sync::mpsc;

#[test]
fn test_01_se_muestra_imagen_por_defecto_si_no_hay_frames() {
    let (_, receiver_frames) = mpsc::channel();
    let mut reproductor = ReproductorSinDecoder::new(receiver_frames);

    let frame_recibido = reproductor.proximo_frame().unwrap();

    assert!(frame_recibido == ColorImage::filled([120, 120], Color32::from_rgb(6, 1, 20)));
}

#[test]
fn test_02_se_muestra_frame_recibido_si_solo_se_recibio_uno() {
    let (sender_frames, receiver_frames) = mpsc::channel();
    let mut reproductor = ReproductorSinDecoder::new(receiver_frames);

    sender_frames
        .send(Frame::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2, 2))
        .unwrap();
    let frame_recibido = reproductor.proximo_frame().unwrap();

    assert!(frame_recibido == ColorImage::filled([2, 2], Color32::from_rgb(0, 0, 0)));
}

#[test]
fn test_03_se_muestra_ultimo_frame_recibido_aunque_ya_se_haya_mostrado() {
    let (sender_frames, receiver_frames) = mpsc::channel();
    let mut reproductor = ReproductorSinDecoder::new(receiver_frames);

    sender_frames
        .send(Frame::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2, 2))
        .unwrap();
    let _ = reproductor.proximo_frame().unwrap();
    let frame_recibido = reproductor.proximo_frame().unwrap();

    assert!(frame_recibido == ColorImage::filled([2, 2], Color32::from_rgb(0, 0, 0)));
}
