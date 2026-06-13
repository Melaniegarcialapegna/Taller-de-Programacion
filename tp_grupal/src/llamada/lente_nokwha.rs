//! Lente Nokwha - Capturar camara local

use nokhwa::{
    Camera,
    utils::{
        CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, mjpeg_to_rgb,
        yuyv422_to_rgb,
    },
};

use crate::{
    llamada::lente::{ErrorLente, Lente},
    sesion_rtp::comunicacion_rtp::Frame,
};

/// Representa un Lente que captura una camara local mediante Nokwha.
/// Puede ser usado dentro de una [CamaraGenerica](crate::llamada::camara::CamaraGenerica) para transmitir video
/// de la camara local.
pub struct LenteNokwha {
    camara_nokwha: Camera, // No confundir con el struct Camara. Esto es algo especifico de este lente
    transmision_encendida: bool,
}

impl Lente for LenteNokwha {
    fn obtener_frame(&mut self) -> Result<Frame, ErrorLente> {
        if !self.transmision_encendida {
            self.camara_nokwha
                .open_stream()
                .map_err(|_| ErrorLente::ErrorObteniendoFrame)?;

            self.transmision_encendida = true;
        }

        let frame = self
            .camara_nokwha
            .frame()
            .map_err(|_| ErrorLente::ErrorInterno)?;

        let bytes_rgb = self.obtener_bytes_rgb(&frame)?;

        let ancho_frame = frame.resolution().width() as usize;
        let alto_frame = frame.resolution().height() as usize;

        Ok(Frame::new(bytes_rgb, ancho_frame, alto_frame))
    }
}

impl LenteNokwha {
    /// Crea un lente que capture la camara local que corresponde al indice especificado
    pub fn new(indice_camara: u32) -> Result<LenteNokwha, ErrorLente> {
        let camara_nokwha = Camera::with_backend(
            CameraIndex::Index(indice_camara),
            RequestedFormat::with_formats(
                RequestedFormatType::AbsoluteHighestFrameRate,
                &[FrameFormat::YUYV],
            ),
            nokhwa::utils::ApiBackend::Video4Linux,
        )
        .map_err(|_| ErrorLente::ErrorInterno)?;

        Ok(LenteNokwha {
            camara_nokwha,
            transmision_encendida: false,
        })
    }

    fn obtener_bytes_rgb(&mut self, frame: &nokhwa::Buffer) -> Result<Vec<u8>, ErrorLente> {
        let frame_format = self.camara_nokwha.frame_format();
        let bytes_buffer = frame.buffer();
        let bytes_rgb = match frame_format {
            FrameFormat::MJPEG => {
                mjpeg_to_rgb(bytes_buffer, false).map_err(|_| ErrorLente::ErrorObteniendoFrame)
            }
            FrameFormat::YUYV => {
                yuyv422_to_rgb(bytes_buffer, false).map_err(|_| ErrorLente::ErrorObteniendoFrame)
            }
            _ => Err(ErrorLente::ErrorObteniendoFrame),
        }?;
        Ok(bytes_rgb)
    }
}
