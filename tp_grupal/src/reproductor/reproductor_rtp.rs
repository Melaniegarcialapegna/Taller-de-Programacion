use eframe::egui::{Color32, ColorImage};
use openh264::decoder::Decoder;
#[cfg(test)]
use openh264::encoder::Encoder;
use openh264::formats::YUVSource;
use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use std::thread;

/// Representación de un reproductor de video para una SesionRTP
pub struct ReproductorDeSesionRTP {
    ultimo_frame: Arc<Mutex<ColorImage>>,
    sender_frames: Sender<Vec<u8>>,
    mutex_procesando_frame: Arc<Mutex<bool>>,
}

impl Debug for ReproductorDeSesionRTP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Reproductor")
    }
}

impl ReproductorDeSesionRTP {
    pub fn new() -> Result<ReproductorDeSesionRTP, ErrorReproductor> {
        let (sender_frames, receiver_frames) = mpsc::channel();
        let imagen_de_fondo = Self::obtener_imagen_default();
        let ultimo_frame = Arc::new(Mutex::new(imagen_de_fondo));

        let mutex_procesando_frame =
            Self::iniciar_reproduccion(receiver_frames, Arc::clone(&ultimo_frame))?;

        Ok(ReproductorDeSesionRTP {
            ultimo_frame,
            sender_frames,
            mutex_procesando_frame,
        })
    }
}

impl Reproductor for ReproductorDeSesionRTP {
    fn proximo_frame(&mut self) -> Result<ColorImage, ErrorReproductor> {
        let lock_ultimo_frame = self
            .ultimo_frame
            .lock()
            .map_err(|_| ErrorReproductor::ErrorObteniendoLock)?;

        let pixeles_imagen = lock_ultimo_frame.pixels.clone();
        let tamanio_imagen = lock_ultimo_frame.size;

        Ok(ColorImage::new(tamanio_imagen, pixeles_imagen))
    }

    fn esta_procesando_frame(&mut self) -> Result<Arc<Mutex<bool>>, ErrorReproductor> {
        Ok(Arc::clone(&self.mutex_procesando_frame))
    }
}

impl ReproductorDeSesionRTP {
    pub fn obtener_sender_frames(&mut self) -> Sender<Vec<u8>> {
        self.sender_frames.clone()
    }

    fn obtener_imagen_default() -> ColorImage {
        ColorImage::filled([120, 120], Color32::from_rgb(6, 1, 20))
    }

    fn iniciar_reproduccion(
        receiver: Receiver<Vec<u8>>,
        ultimo_frame: Arc<Mutex<ColorImage>>,
    ) -> Result<Arc<Mutex<bool>>, ErrorReproductor> {
        // Si este booleano esta en true, no me van a mandar mas frames (sino hay un delay tremendo)
        let procesando_frame = Arc::new(Mutex::new(false));
        let ref_procesando_frame = Arc::clone(&procesando_frame);
        thread::spawn(move || {
            let mut decoder = match Decoder::new() {
                Ok(decoder) => decoder,
                Err(_) => {
                    eprintln!("Error creando decoder!");
                    return;
                }
            };
            // Aca habria que agarrar el ultimo frame de rtp y procesar, y despues sacar, y todo adentro de un loop
            let clon_procesando_frame = Arc::clone(&ref_procesando_frame);
            for mensaje in receiver {
                {
                    let mut mutex_procesando_frame = match clon_procesando_frame.lock() {
                        Ok(mutex) => mutex,
                        Err(_) => {
                            eprintln!("Error obteniendo el lock de procesando_frame");
                            return;
                        }
                    };
                    *mutex_procesando_frame = true
                }
                let ref_ultimo_frame = Arc::clone(&ultimo_frame);

                if Self::procesar_frame(&mut decoder, mensaje, ref_ultimo_frame).is_err() {
                    // todo loggear!
                    eprintln!("Ocurrio un error procesando un frame");
                };

                {
                    let mut mutex_procesando_frame = match clon_procesando_frame.lock() {
                        Ok(mutex) => mutex,
                        Err(_) => {
                            eprintln!("Error obteniendo el lock de procesando_frame");
                            return;
                        }
                    };
                    *mutex_procesando_frame = false
                }
            }
        });
        Ok(Arc::clone(&procesando_frame))
    }

    /// Inserta un frame RGB directamente en la cola del reproductor para mostrarlo.
    /// Se usa exclusivamente para la vista local (preview) cuando la cámara se muestra sin compresión H264.
    pub fn mostrar_frame_local(&self, imagen: ColorImage) -> Result<(), ErrorReproductor> {
        let mut mutex_imagen = self
            .ultimo_frame
            .lock()
            .map_err(|_| ErrorReproductor::ErrorObteniendoLock)?;

        *mutex_imagen = imagen;
        Ok(())
    }

    fn procesar_frame(
        decoder: &mut Decoder,
        mensaje: Vec<u8>,
        ultimo_frame: Arc<Mutex<ColorImage>>,
    ) -> Result<(), ErrorReproductor> {
        let imagen = Self::procesar_imagen_desde_bytes(decoder, mensaje)
            .map_err(|_| ErrorReproductor::ErrorLeyendoBytes)?;

        let mut mutex_imagen = ultimo_frame
            .lock()
            .map_err(|_| ErrorReproductor::ErrorObteniendoLock)?;

        *mutex_imagen = imagen;

        Ok(())
    }

    fn procesar_imagen_desde_bytes(
        decoder: &mut Decoder,
        bytes: Vec<u8>,
    ) -> Result<ColorImage, ErrorReproductor> {
        let paquete_yuv = decoder.decode(&bytes).map_err(|e| {
            println!("Error: {}", e);
            ErrorReproductor::ErrorLeyendoBytes
        })?;

        if let Some(yuv) = paquete_yuv {
            let (anchura, altura) = yuv.dimensions();
            let largo_rgb = yuv.rgb8_len();
            let mut bytes_rgb = vec![0; largo_rgb];
            yuv.write_rgb8(&mut bytes_rgb);

            let imagen = ColorImage::from_rgb([anchura, altura], &bytes_rgb);

            Ok(imagen)
        } else {
            Err(ErrorReproductor::ErrorLeyendoBytes)
        }
    }
}

#[cfg(test)]
use openh264::formats::{RgbSliceU8, YUVBuffer};
#[cfg(test)]
use std::time::Duration;

use crate::reproductor::{ErrorReproductor, Reproductor};

#[cfg(test)]
const ANCHO_IMAGEN: usize = 150;

#[cfg(test)]
const ALTO_IMAGEN: usize = 150;

#[test]
fn test_01_se_leen_nuevas_imagenes_enviadas() {
    let mut reproductor = ReproductorDeSesionRTP::new().unwrap();
    let tx = reproductor.obtener_sender_frames();

    // RgbSliceU8 -> YUVBuffer -> EncodedBitStream -> Vec<u8>, lo que necesito
    let mut encoder = obtener_encoder_configurado();
    let bytes_imagen_en_bruto = obtener_bytes_imagen_default();
    let bytes_rgb = RgbSliceU8::new(&bytes_imagen_en_bruto[..], (ANCHO_IMAGEN, ALTO_IMAGEN));
    let bytes_imagen_yuv = YUVBuffer::from_rgb8_source(bytes_rgb);
    let bytes_encodeados = encoder.encode(&bytes_imagen_yuv).unwrap();
    let bytes_encodeados = bytes_encodeados.to_vec();

    // Mando los bytes para que los reciba el reproductor
    tx.send(bytes_encodeados).unwrap();

    // Para evitar que se lockee la cola de frames antes de procesar el nuevo frame
    // y agregarlo. Si no pasa el test, subir esto
    thread::sleep(Duration::from_secs(1));

    let imagen = reproductor.proximo_frame().unwrap();
    assert!(imagen.size == [ANCHO_IMAGEN, ALTO_IMAGEN]);
}

#[test]
fn test_02_reproductor_permite_encodear_en_su_formato() {
    let mut reproductor = ReproductorDeSesionRTP::new().unwrap();
    let tx = reproductor.obtener_sender_frames();

    let bytes_imagen = obtener_bytes_imagen_default();

    let mut encoder = obtener_encoder_configurado();

    let bytes_rgb = RgbSliceU8::new(&bytes_imagen[..], (ANCHO_IMAGEN, ALTO_IMAGEN));
    let bytes_imagen_yuv = YUVBuffer::from_rgb8_source(bytes_rgb);
    let bytes_encodeados = encoder.encode(&bytes_imagen_yuv).unwrap();
    let bytes_imagen_encodeada = bytes_encodeados.to_vec();

    // Le mando esos bytes al mismo reproductor, deberia poder procesarlos
    // y luego devolverlos como imagen nuevamente
    tx.send(bytes_imagen_encodeada).unwrap();
    thread::sleep(Duration::from_secs(1));

    let imagen = reproductor.proximo_frame().unwrap();
    assert!(imagen != ReproductorDeSesionRTP::obtener_imagen_default());
    assert!(imagen.size == [ANCHO_IMAGEN, ALTO_IMAGEN]);
}

#[cfg(test)]
fn obtener_encoder_configurado() -> Encoder {
    Encoder::new().unwrap()
}

/// Devuelve un vector de bytes que representan una imagen en formato RGB de tamaño 2x2 pixeles.
#[cfg(test)]
fn obtener_bytes_imagen_default() -> Vec<u8> {
    vec![0; ANCHO_IMAGEN * ALTO_IMAGEN * 3]
}
