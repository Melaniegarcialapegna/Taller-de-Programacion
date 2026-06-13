use crate::protocolos::pca::estado::EstadoUsuarioPCA;

#[derive(Debug, Clone, PartialEq)]
pub struct UsuarioPCA {
    nombre: String,
    estado: EstadoUsuarioPCA,
}

impl UsuarioPCA {
    pub fn new(nombre: String, estado: EstadoUsuarioPCA) -> Self {
        UsuarioPCA { nombre, estado }
    }

    pub fn representacion_tupla(&self) -> (String, String) {
        let estado_str = String::from(&self.estado);
        (self.nombre.to_string(), estado_str)
    }

    pub fn nombre(&self) -> String {
        self.nombre.clone()
    }

    pub fn estado(&self) -> EstadoUsuarioPCA {
        self.estado.clone()
    }
}

impl From<&UsuarioPCA> for String {
    fn from(usuario: &UsuarioPCA) -> Self {
        let nombre = &usuario.nombre;
        let estado = String::from(&usuario.estado);
        format!("{nombre};{estado}").to_string()
    }
}
