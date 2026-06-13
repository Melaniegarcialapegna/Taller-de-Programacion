use crate::protocolos::pca::error::ErrorMensajePCA;

/// Estado de los usuarios que se van a devolver con el mensaje USUARIOS ...
#[derive(Clone, Debug, PartialEq)]
pub enum EstadoUsuarioPCA {
    Disponible,
    Ocupado,
    Desconectado,
}

impl TryFrom<&str> for EstadoUsuarioPCA {
    type Error = ErrorMensajePCA;

    fn try_from(cadena: &str) -> Result<Self, Self::Error> {
        match cadena {
            "DISP" => Ok(Self::Disponible),
            "OCUP" => Ok(Self::Ocupado),
            "DESC" => Ok(Self::Desconectado),
            _ => Err(ErrorMensajePCA::ErrorMensajeIncompleto),
        }
    }
}

impl From<&EstadoUsuarioPCA> for String {
    fn from(estado: &EstadoUsuarioPCA) -> Self {
        match estado {
            EstadoUsuarioPCA::Disponible => "DISP".to_string(),
            EstadoUsuarioPCA::Ocupado => "OCUP".to_string(),
            EstadoUsuarioPCA::Desconectado => "DESC".to_string(),
        }
    }
}
