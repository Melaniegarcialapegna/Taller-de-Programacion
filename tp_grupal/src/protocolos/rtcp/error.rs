//! # Errores RTCP

#[derive(Debug)]
/// Errores relacionados a operaciones con paquetes RTCP
pub enum ErrorPaqueteRTCP {
    /// El header del paquete estaba incompleto
    HeaderIncompleto,
    /// El tipo de paquete especificado es invalido o no es usable para WebRTC
    TipoDePaqueteInvalido,
    /// El paquete que se intento crear es invalido
    PaqueteInvalido,
    /// La versión leida es invalida (para WebRTC debe ser siempre 2)
    VersionInvalida,
}
