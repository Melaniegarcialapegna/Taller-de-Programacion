use crate::protocolos::pca::error::ErrorMensajePCA;
use crate::protocolos::pca::estado::EstadoUsuarioPCA;
use crate::protocolos::pca::usuario::UsuarioPCA;
use crate::protocolos::pca::visitor::{ConversorAStringPCA, VisitorMensajePCA};

#[derive(Debug, Clone, PartialEq)]
pub enum MensajePCA {
    Ok,
    /// Rechazar llamada entrante
    Rechazo,
    /// Cortar llamada en curso
    Cortar,
    /// Salir de la sesión
    Salir,
    /// Llamar al usuario con este nombre
    Llamar(String),
    // Aceptar llamada
    Aceptar,
    /// El usuario con este nombre me esta llamando
    Llamando(String),
    /// Mensaje de error del protocolo
    ErrorPCA(String),
    /// Registrar usuario con nombre y contrasenia
    Registrar(String, String),
    /// Entrar a la sesion con nombre y contrasenia
    Entrar(String, String),
    /// Usuarios del servidor junto a sus estados
    Usuarios(Vec<UsuarioPCA>),
    //PedirOffer
    PedirOffer,
    /// Offer SDP
    Offer(String),
    /// Answer SDP
    Answer(String),
    /// Actualizacion de estado de un usuario en particular
    UsuarioEstado(UsuarioPCA),
    /// El usuario se registro correctamente
    Registrado,
    /// Se cerro la sesión del usuario
    Salio,
}

impl MensajePCA {
    fn parsear_mensaje_llamar(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        Self::comprobar_cantidad_campos(&campos, 2)?;
        Ok(MensajePCA::Llamar(String::from(campos[1])))
    }

    fn parsear_mensaje_llamando(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        Self::comprobar_cantidad_campos(&campos, 2)?;
        Ok(MensajePCA::Llamando(String::from(campos[1])))
    }

    fn parsear_mensaje_error(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        if campos.len() < 3 {
            return Err(ErrorMensajePCA::ErrorMensajeIncompleto);
        }
        let mut mensaje = campos;
        mensaje.remove(0);

        Ok(MensajePCA::ErrorPCA(mensaje.join(" ")))
    }

    fn parsear_mensaje_offer(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        let msj_sin_primera_palabra = campos[1..].join(" ");
        let lineas_sdp = msj_sin_primera_palabra.replace(";", "\n");
        Ok(MensajePCA::Offer(lineas_sdp))
    }

    fn parsear_mensaje_usuario(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        Self::comprobar_cantidad_campos(&campos, 2)?;
        let info_usuario: Vec<&str> = campos[1].split(";").collect();
        let estado = EstadoUsuarioPCA::try_from(info_usuario[1])?;
        let usuario = UsuarioPCA::new(info_usuario[0].to_string(), estado);
        Ok(MensajePCA::UsuarioEstado(usuario))
    }

    fn parsear_mensaje_answer(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        let msj_sin_primera_palabra = campos[1..].join(" ");
        let lineas_sdp = msj_sin_primera_palabra.replace(";", "\n");
        Ok(MensajePCA::Answer(lineas_sdp))
    }

    fn parsear_mensaje_registrar(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        Self::comprobar_cantidad_campos(&campos, 3)?;
        Ok(MensajePCA::Registrar(
            String::from(campos[1]),
            String::from(campos[2]),
        ))
    }

    fn parsear_mensaje_entrar(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        Self::comprobar_cantidad_campos(&campos, 3)?;
        Ok(MensajePCA::Entrar(
            String::from(campos[1]),
            String::from(campos[2]),
        ))
    }

    fn parsear_mensaje_usuarios(campos: Vec<&str>) -> Result<MensajePCA, ErrorMensajePCA> {
        let mut vector_usuarios = vec![];

        for campo in campos.iter().skip(1) {
            let usuario: Vec<&str> = campo.split(";").collect();

            if usuario.len() != 2 {
                return Err(ErrorMensajePCA::ErrorMensajeIncompleto);
            }

            let estado = EstadoUsuarioPCA::try_from(usuario[1])?;

            vector_usuarios.push(UsuarioPCA::new(usuario[0].to_string(), estado));
        }

        Ok(MensajePCA::Usuarios(vector_usuarios))
    }

    /// Parsear mensaje cuyo contenido es una sola palabra, sin argumentos.
    /// - `mensaje_str` es el mensaje entero recibido
    /// - `mensaje_a_devolver` es el tipo de mensaje que se devolvera si la cantidad de argumentos es correcta (1)
    fn parsear_mensaje_simple(
        mensaje_str: &str,
        mensaje_a_devolver: MensajePCA,
    ) -> Result<MensajePCA, ErrorMensajePCA> {
        let campos: Vec<&str> = mensaje_str.split_ascii_whitespace().collect();

        Self::comprobar_cantidad_campos(&campos, 1)?;

        Ok(mensaje_a_devolver)
    }

    /// Comprueba que la cantidad de campos de `campos_mensaje` coincide con `cantidad_esperada`
    fn comprobar_cantidad_campos(
        campos_mensaje: &[&str],
        cantidad_esperada: usize,
    ) -> Result<(), ErrorMensajePCA> {
        if campos_mensaje.len() != cantidad_esperada {
            return Err(ErrorMensajePCA::ErrorMensajeIncompleto);
        }

        Ok(())
    }

    /// Aceptar cualquier visitor para los mensajes que devuelva un String.
    fn aceptar_visitor_str(&self, visitor: &dyn VisitorMensajePCA) -> String {
        match self {
            Self::Ok => visitor.visitar_mensaje_ok(self),
            Self::Cortar => visitor.visitar_mensaje_cortar(self),
            Self::Rechazo => visitor.visitar_mensaje_rechazo(self),
            Self::Salir => visitor.visitar_mensaje_salir(self),
            Self::Answer(_) => visitor.visitar_mensaje_answer(self),
            Self::Entrar(_, _) => visitor.visitar_mensaje_entrar(self),
            Self::ErrorPCA(_) => visitor.visitar_mensaje_error(self),
            Self::Registrar(_, _) => visitor.visitar_mensaje_registrar(self),
            Self::Usuarios(_) => visitor.visitar_mensaje_usuarios(self),
            Self::Llamar(_) => visitor.visitar_mensaje_llamar(self),
            Self::Llamando(_) => visitor.visitar_mensaje_llamando(self),
            Self::Offer(_) => visitor.visitar_mensaje_offer(self),
            Self::PedirOffer => visitor.visitar_mensaje_pedir_offer(self),
            Self::Aceptar => visitor.visitar_mensaje_aceptar(self),
            Self::UsuarioEstado(_) => visitor.visitar_mensaje_actualizacion_estado_usuario(self),
            Self::Registrado => visitor.visitar_mensaje_registrado(self),
            Self::Salio => visitor.visitar_mensaje_salio(self),
        }
    }
}

impl TryFrom<&str> for MensajePCA {
    type Error = ErrorMensajePCA;

    fn try_from(cadena: &str) -> Result<Self, Self::Error> {
        let campos: Vec<&str> = cadena.split_ascii_whitespace().collect();
        let primera_palabra = campos[0];

        match primera_palabra {
            "OK" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Ok),
            "RECHAZO" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Rechazo),
            "CORTAR" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Cortar),
            "SALIR" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Salir),
            "LLAMAR" => MensajePCA::parsear_mensaje_llamar(campos),
            "LLAMANDO" => MensajePCA::parsear_mensaje_llamando(campos),
            "ERROR" => MensajePCA::parsear_mensaje_error(campos),
            "REGISTRAR" => MensajePCA::parsear_mensaje_registrar(campos),
            "ENTRAR" => MensajePCA::parsear_mensaje_entrar(campos),
            "USUARIOS" => MensajePCA::parsear_mensaje_usuarios(campos),
            "OFFER" => MensajePCA::parsear_mensaje_offer(campos),
            "ANSWER" => MensajePCA::parsear_mensaje_answer(campos),
            "ACEPTAR" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Aceptar),
            "PEDIR_OFFER" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::PedirOffer),
            "USUARIO" => MensajePCA::parsear_mensaje_usuario(campos),
            "REGISTRADO" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Registrado),
            "SALIO" => MensajePCA::parsear_mensaje_simple(cadena, MensajePCA::Salio),
            _ => Err(ErrorMensajePCA::ErrorMensajeInvalido),
        }
    }
}

impl From<MensajePCA> for String {
    fn from(mensaje: MensajePCA) -> Self {
        mensaje.aceptar_visitor_str(&ConversorAStringPCA)
    }
}

impl From<&MensajePCA> for String {
    fn from(mensaje: &MensajePCA) -> Self {
        mensaje.aceptar_visitor_str(&ConversorAStringPCA)
    }
}
