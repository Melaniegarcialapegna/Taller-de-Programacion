use crate::protocolos::pca::mensaje::MensajePCA;

/// Interfaz que debera cumplir cualquier visitor que quiera aplicar operaciones
/// sobre las variantes de mensajes del protocolo PCA
pub trait VisitorMensajePCA {
    fn visitar_mensaje_ok(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_rechazo(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_cortar(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_salir(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_llamar(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_llamando(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_error(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_registrar(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_entrar(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_usuarios(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_offer(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_answer(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_aceptar(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_pedir_offer(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_actualizacion_estado_usuario(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_registrado(&self, mensaje: &MensajePCA) -> String;
    fn visitar_mensaje_salio(&self, mensaje: &MensajePCA) -> String;
}

/// Visitor del protocolo PCA que convierte cada variante a su representación en String.
#[derive(Default)]
pub struct ConversorAStringPCA;

impl ConversorAStringPCA {
    fn representacion_mensaje_un_operando(mensaje: &str, operando: &str) -> String {
        format!("{mensaje} {operando}\n")
    }

    fn representacion_mensaje_dos_operandos(
        mensaje: &str,
        operando_uno: &str,
        operando_dos: &str,
    ) -> String {
        format!("{mensaje} {operando_uno} {operando_dos}\n")
    }
}

impl VisitorMensajePCA for ConversorAStringPCA {
    fn visitar_mensaje_ok(&self, _mensaje: &MensajePCA) -> String {
        String::from("OK\n")
    }

    fn visitar_mensaje_cortar(&self, _mensaje: &MensajePCA) -> String {
        String::from("CORTAR\n")
    }

    fn visitar_mensaje_rechazo(&self, _mensaje: &MensajePCA) -> String {
        String::from("RECHAZO\n")
    }

    fn visitar_mensaje_salir(&self, _mensaje: &MensajePCA) -> String {
        String::from("SALIR\n")
    }

    fn visitar_mensaje_aceptar(&self, _mensaje: &MensajePCA) -> String {
        String::from("ACEPTAR\n")
    }

    fn visitar_mensaje_pedir_offer(&self, _mensaje: &MensajePCA) -> String {
        String::from("PEDIR_OFFER\n")
    }

    fn visitar_mensaje_entrar(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Entrar(nombre, contrasenia) = mensaje {
            ConversorAStringPCA::representacion_mensaje_dos_operandos("ENTRAR", nombre, contrasenia)
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_answer(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Answer(answer) = mensaje {
            let lineas_como_csv = answer.replace("\n", ";");
            ConversorAStringPCA::representacion_mensaje_un_operando("ANSWER", &lineas_como_csv)
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_error(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::ErrorPCA(contenido) = mensaje {
            ConversorAStringPCA::representacion_mensaje_un_operando("ERROR", contenido)
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_llamando(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Llamando(nombre) = mensaje {
            ConversorAStringPCA::representacion_mensaje_un_operando("LLAMANDO", nombre)
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_llamar(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Llamar(nombre) = mensaje {
            ConversorAStringPCA::representacion_mensaje_un_operando("LLAMAR", nombre)
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_offer(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Offer(offer) = mensaje {
            let lineas_como_csv = offer.replace("\n", ";");
            ConversorAStringPCA::representacion_mensaje_un_operando("OFFER", &lineas_como_csv)
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_registrar(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Registrar(nombre, contrasenia) = mensaje {
            ConversorAStringPCA::representacion_mensaje_dos_operandos(
                "REGISTRAR",
                nombre,
                contrasenia,
            )
        } else {
            "".to_string() // no deberia pasar
        }
    }

    fn visitar_mensaje_usuarios(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::Usuarios(vector_usuarios) = mensaje {
            let usuarios: Vec<String> = vector_usuarios.iter().map(String::from).collect();
            let usuarios_str = usuarios.join(" ");
            format!("USUARIOS {usuarios_str}\n")
        } else {
            "".to_string() // No deberia pasar
        }
    }

    fn visitar_mensaje_actualizacion_estado_usuario(&self, mensaje: &MensajePCA) -> String {
        if let MensajePCA::UsuarioEstado(usuario) = mensaje {
            let usuario_str = String::from(usuario);
            format!("USUARIO {usuario_str}\n")
        } else {
            "".to_string() // No deberia pasar
        }
    }

    fn visitar_mensaje_registrado(&self, _mensaje: &MensajePCA) -> String {
        "REGISTRADO\n".to_string()
    }

    fn visitar_mensaje_salio(&self, _mensaje: &MensajePCA) -> String {
        "SALIO\n".to_string()
    }
}
