#[cfg(test)]
use crate::protocolos::pca::error::ErrorMensajePCA;
#[cfg(test)]
use crate::protocolos::pca::estado::EstadoUsuarioPCA;
#[cfg(test)]
use crate::protocolos::pca::mensaje::MensajePCA;
#[cfg(test)]
use crate::protocolos::pca::usuario::UsuarioPCA;

#[test]
fn test_01_se_parsea_mensaje_ok_correctamente() {
    let mensaje_str = "OK\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(matches!(mensaje, MensajePCA::Ok))
}

#[test]
fn test_02_se_rechaza_mensaje_ok_invalido() {
    let mensaje_str = "OK messi\n";
    let resultado_mensaje = MensajePCA::try_from(mensaje_str);

    assert!(resultado_mensaje.is_err())
}

#[test]
fn test_03_se_parsea_mensaje_rechazo_correctamente() {
    let mensaje_str = "RECHAZO\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(matches!(mensaje, MensajePCA::Rechazo))
}

#[test]
fn test_04_se_rechaza_mensaje_rechazo_invalido() {
    let mensaje_str = "RECHAZO messi\n";
    let respuesta_mensaje = MensajePCA::try_from(mensaje_str);

    assert!(respuesta_mensaje.is_err())
}

#[test]
fn test_05_se_parsea_mensaje_cortar_correctamente() {
    let mensaje_str = "CORTAR\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(matches!(mensaje, MensajePCA::Cortar))
}

#[test]
fn test_06_se_rechaza_mensaje_cortar_invalido() {
    let mensaje_str = "CORTAR messi\n";
    let respuesta_mensaje = MensajePCA::try_from(mensaje_str);

    assert!(respuesta_mensaje.is_err())
}

#[test]
fn test_07_se_parsea_mensaje_salir_correctamente() {
    let mensaje_str = "SALIR\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(matches!(mensaje, MensajePCA::Salir))
}

#[test]
fn test_08_se_parsea_mensaje_llamar_correctamente() {
    let mensaje_str = "LLAMAR messi\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::Llamar(nombre) = mensaje {
        assert!(nombre == "messi");
        true
    } else {
        false
    })
}

#[test]
fn test_09_se_rechaza_mensaje_llamando_invalido() {
    let mensaje_str = "LLAMAR messi ernesto\n";
    let resultado_mensaje = MensajePCA::try_from(mensaje_str);

    assert!(resultado_mensaje.is_err())
}

#[test]
fn test_10_se_parsea_mensaje_llamando_correctamente() {
    let mensaje_str = "LLAMANDO messi\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::Llamando(nombre) = mensaje {
        assert!(nombre == "messi");
        true
    } else {
        false
    })
}

#[test]
fn test_11_se_rechaza_mensaje_llamando_invalido() {
    let mensaje_str = "LLAMANDO messi ernesto\n";
    let resultado_mensaje = MensajePCA::try_from(mensaje_str);

    assert!(resultado_mensaje.is_err())
}

#[test]
fn test_12_se_parsea_mensaje_error_correctamente() {
    let mensaje_str = "ERROR messi ernesto\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::ErrorPCA(contenido) = mensaje {
        dbg!(&contenido);
        assert!(contenido == "messi ernesto");
        true
    } else {
        false
    })
}

#[test]
fn test_13_se_parsea_mensaje_registrar_correctamente() {
    let mensaje_str = "REGISTRAR messi messi123\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(
        if let MensajePCA::Registrar(usuario, contrasenia) = mensaje {
            assert!(usuario == "messi");
            assert!(contrasenia == "messi123");
            true
        } else {
            false
        }
    )
}

#[test]
fn test_13_se_parsea_mensaje_entrar_correctamente() {
    let mensaje_str = "ENTRAR messi messi123\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::Entrar(usuario, contrasenia) = mensaje {
        assert!(usuario == "messi");
        assert!(contrasenia == "messi123");
        true
    } else {
        false
    })
}

#[test]
fn test_14_se_parsea_mensaje_usuarios_correctamente() {
    let mensaje_str = "USUARIOS messi;DISP ernesto;OCUP\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::Usuarios(vector_usuarios) = mensaje {
        assert!(vector_usuarios.len() == 2);
        let primer_usuario = vector_usuarios
            .first()
            .expect("Deberia parsear dos usuarios");
        let segundo_usuario = vector_usuarios
            .get(1)
            .expect("Deberia parsear dos usuarios");

        assert!(primer_usuario.nombre() == "messi");
        assert!(matches!(
            primer_usuario.estado(),
            EstadoUsuarioPCA::Disponible
        ));

        assert!(segundo_usuario.nombre() == "ernesto");
        assert!(matches!(
            segundo_usuario.estado(),
            EstadoUsuarioPCA::Ocupado
        ));

        true
    } else {
        false
    })
}

#[test]
fn test_15_se_parsea_mensaje_offer_correctamente() {
    // No checkeo que el offer sea correcto porque eso no es responsabilidad de este protocolo
    let mensaje_str = "OFFER offer_de_ejemplo\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::Offer(contenido) = mensaje {
        assert!(contenido == "offer_de_ejemplo");
        true
    } else {
        false
    })
}

#[test]
fn test_16_se_parsea_mensaje_offer_correctamente() {
    // No checkeo que el answer sea correcto porque eso no es responsabilidad de este protocolo
    let mensaje_str = "ANSWER answer_de_ejemplo\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(if let MensajePCA::Answer(contenido) = mensaje {
        assert!(contenido == "answer_de_ejemplo");
        true
    } else {
        false
    })
}

#[test]
fn test_17_se_rechaza_mensaje_totalmente_invalido() {
    // No checkeo que el answer sea correcto porque eso no es responsabilidad de este protocolo
    let mensaje_str = "messiiii\n";
    let resultado_mensaje = MensajePCA::try_from(mensaje_str);

    assert!(resultado_mensaje.is_err());
    assert!(matches!(
        resultado_mensaje,
        Err(ErrorMensajePCA::ErrorMensajeInvalido)
    ));
}

#[test]
fn test_18_se_convierte_mensaje_simple_a_string_correctamente() {
    // No checkeo que el answer sea correcto porque eso no es responsabilidad de este protocolo
    let mensaje_inicial_str = "OK\n";
    let mensaje_error = mensaje_error_para(mensaje_inicial_str);
    let mensaje = MensajePCA::try_from(mensaje_inicial_str).expect(&mensaje_error);
    let mensaje_parseado_str = String::from(mensaje);

    assert!(mensaje_inicial_str == mensaje_parseado_str)
}

#[test]
fn test_20_se_convierte_mensaje_un_operando_a_string_correctamente() {
    // No checkeo que el answer sea correcto porque eso no es responsabilidad de este protocolo
    let mensaje_inicial_str = "LLAMANDO messi\n";
    let mensaje_error = mensaje_error_para(mensaje_inicial_str);
    let mensaje = MensajePCA::try_from(mensaje_inicial_str).expect(&mensaje_error);
    let mensaje_parseado_str = String::from(mensaje);

    assert!(mensaje_inicial_str == mensaje_parseado_str)
}

#[test]
fn test_21_se_convierte_mensaje_usuarios_a_string_correctamente() {
    // No checkeo que el answer sea correcto porque eso no es responsabilidad de este protocolo
    let mensaje_inicial_str = "USUARIOS messi;DISP ernesto;OCUP juan;DESC\n";
    let mensaje_error = mensaje_error_para(mensaje_inicial_str);
    let mensaje = MensajePCA::try_from(mensaje_inicial_str).expect(&mensaje_error);
    let mensaje_parseado_str = String::from(mensaje);

    dbg!(&mensaje_parseado_str);

    assert!(mensaje_inicial_str == mensaje_parseado_str)
}

#[cfg(test)]
fn mensaje_error_para(mensaje_str: &str) -> String {
    format!("Se deberia aceptar el mensaje {mensaje_str}")
}

#[test]
fn test_22_se_parsea_mensaje_aceptar_correctamente() {
    let mensaje_str = "ACEPTAR\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(matches!(mensaje, MensajePCA::Aceptar))
}

#[test]
fn test_23_se_parsea_mensaje_pedir_offer_correctamente() {
    let mensaje_str = "PEDIR_OFFER\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);

    assert!(matches!(mensaje, MensajePCA::PedirOffer))
}

#[test]
fn test_24_se_parsea_usuario_correctamente() {
    let mensaje_str = "USUARIO melanie;DISP\n";
    let mensaje_error = mensaje_error_para(mensaje_str);
    let mensaje = MensajePCA::try_from(mensaje_str).expect(&mensaje_error);
    let _usuario = UsuarioPCA::new("melanie".to_string(), EstadoUsuarioPCA::Disponible);

    assert!(matches!(mensaje, MensajePCA::UsuarioEstado(_usuario)))
}
