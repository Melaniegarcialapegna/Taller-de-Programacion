use std::fmt::{self, Display};

use crate::protocolos::pca::estado::EstadoUsuarioPCA;

///Estructura que define el estado de un usuario dentro del servidor central
#[derive(Clone, PartialEq, Debug)]
pub enum EstadoUsuario {
    Disponible,
    Ocupado,
    Desconectado,
}

impl Display for EstadoUsuario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EstadoUsuario::Disponible => write!(f, "DISP"),
            EstadoUsuario::Ocupado => write!(f, "OCUP"),
            EstadoUsuario::Desconectado => write!(f, "DESC"),
        }
    }
}

impl From<EstadoUsuario> for EstadoUsuarioPCA {
    fn from(estado: EstadoUsuario) -> EstadoUsuarioPCA {
        match estado {
            EstadoUsuario::Disponible => EstadoUsuarioPCA::Disponible,
            EstadoUsuario::Ocupado => EstadoUsuarioPCA::Ocupado,
            EstadoUsuario::Desconectado => EstadoUsuarioPCA::Desconectado,
        }
    }
}
