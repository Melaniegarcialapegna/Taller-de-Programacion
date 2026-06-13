//! Reproductor de video sencillo para [eframe], para el codec H264.

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use eframe::egui::ColorImage;

pub mod reproductor_audio;
pub mod reproductor_audio_default;
pub mod reproductor_mock;
pub mod reproductor_rtp;
pub mod reproductor_sin_decoder;

/// Errores de un [Reproductor]
#[derive(Debug)]
pub enum ErrorReproductor {
    /// Fallo al crear un reproductor
    ErrorCreandoReproductor,
    /// Hubo un error de IO al leer bytes
    ErrorLeyendoBytes,
    /// Fallo al procesar una imagen recibida por el channel
    ErrorRecibiendoImagenes,
    /// Fallo al codificar una imagen en el formato H264
    ErrorEncodeandoImagen,
    /// Fallo al obtener algun lock,
    ErrorObteniendoLock,
}

impl Display for ErrorReproductor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ErrorReproductor::ErrorCreandoReproductor => f.write_str("Error creando reproductor"),
            ErrorReproductor::ErrorEncodeandoImagen => f.write_str("Error encodeando imagen"),
            ErrorReproductor::ErrorLeyendoBytes => f.write_str("Error leyendo bytes"),
            ErrorReproductor::ErrorObteniendoLock => f.write_str("Error obteniendo lock"),
            ErrorReproductor::ErrorRecibiendoImagenes => f.write_str("Error recibiendo imagenes"),
        }
    }
}

/// Representa un reproductor de video, al que se le puede consultar por el proximo frame
/// a mostrar.
pub trait Reproductor {
    /// Devuelve el proximo frame a renderizar y lo desencola de la cola de frames del reproductor.
    fn proximo_frame(&mut self) -> Result<ColorImage, ErrorReproductor>;
    /// Devuelve un Arc<Mutex<bool>>, cuyo valor indicara si el reproductor esta desencodeando un frame
    /// Util si se pretende decidir si enviar o no un frame según el estado del Reproductor.
    fn esta_procesando_frame(&mut self) -> Result<Arc<Mutex<bool>>, ErrorReproductor>;
}
