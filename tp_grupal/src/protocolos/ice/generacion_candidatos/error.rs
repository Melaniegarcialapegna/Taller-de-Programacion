use std::fmt;

#[derive(Debug)]
pub enum ErrorGeneracionDeCandidatosICE {
    ObtenerInterfaces(String),
    SinInterfacesValidas,
    CrearCandidato(String),
}

impl fmt::Display for ErrorGeneracionDeCandidatosICE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorGeneracionDeCandidatosICE::ObtenerInterfaces(e) => {
                write!(f, "Error obteniendo interfaces del sistema: {}", e)
            }
            ErrorGeneracionDeCandidatosICE::SinInterfacesValidas => {
                write!(f, "No se encontraron interfaces IPv4 válidas")
            }
            ErrorGeneracionDeCandidatosICE::CrearCandidato(e) => {
                write!(f, "Error creando candidato ICE: {}", e)
            }
        }
    }
}

impl std::error::Error for ErrorGeneracionDeCandidatosICE {}
