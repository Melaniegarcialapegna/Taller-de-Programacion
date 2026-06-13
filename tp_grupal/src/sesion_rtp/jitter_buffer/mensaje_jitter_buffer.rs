use crate::sesion_rtp::jitter_buffer::gestion_jitter_buffer::InformacionPaqueteRTP;

pub enum MensajeJitterBuffer {
    MensajeFinalizacionLlamada,
    InformacionPaqueteRTP(InformacionPaqueteRTP),
}
