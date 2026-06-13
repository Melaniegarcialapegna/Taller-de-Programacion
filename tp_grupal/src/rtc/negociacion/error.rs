pub enum ErrorDeNegociacion {
    SdpFaltante(String),
    MediasFaltantes,
    ErrorDeICE(String),
}

impl std::fmt::Display for ErrorDeNegociacion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorDeNegociacion::SdpFaltante(mensaje) => write!(f, "No se encontró SDP {mensaje}"),
            ErrorDeNegociacion::MediasFaltantes => {
                write!(f, "No hay medias disponibles para negociar")
            }
            ErrorDeNegociacion::ErrorDeICE(mensaje) => write!(f, "Fallo del ICE: {mensaje}"),
        }
    }
}
