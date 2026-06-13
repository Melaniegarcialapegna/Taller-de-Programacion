#[cfg(test)]
use crate::servidor_utils::servidor_central_utils::estado_usuario::EstadoUsuario;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::sync::mpsc::Receiver;
#[cfg(test)]
use std::{collections::HashMap, sync::mpsc::Sender};
#[cfg(test)]
use std::{
    fs::File,
    sync::mpsc::{self},
    thread,
};

#[cfg(test)]
use crate::servidor_utils::{
    mensajes::{mensaje_servidor::MensajeServidor, mensaje_usuario::MensajeUsuario},
    servidor_central_utils::servidor_central::ServidorCentral,
};

#[cfg(test)]
//TEST de funcionalidad de [`servidor_central`]
fn generar_estructura_servidor_default_tests(
    ruta_archivo: String,
) -> (
    ServidorCentral,
    Sender<MensajeServidor>,
    Receiver<MensajeServidor>,
) {
    let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

    let servidor = ServidorCentral::crear(ruta_archivo, 15).expect("Error al crear el servidor");

    (servidor, tx_servidor, rx_servidor)
}

#[test]
fn test_comprobar_funcionamiento_persistir_usuario() {
    let nombre_archivo = "archivos_test/test_prueba.txt";

    File::create(nombre_archivo).expect("Fallo creando archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Registrar(
            "meujeje".to_string(),
            "manuelitaViviaEnPehuajo".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");

    assert_eq!(respuesta_servidor, MensajeUsuario::Ok)
}

#[test]
fn test01_se_crea_servidor_central_archivo_usuarios_vacio() {
    let nombre_archivo = "archivos_test/test01.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");
    let (servidor, _tx_servidor, _rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());
    assert!(servidor.usuarios.is_empty());
    assert!(servidor.estado_usuarios.is_empty());
    assert!(servidor.usuarios_conectados.is_empty());
    assert!(servidor.usuarios_disponibles.is_empty());
    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test02_se_crea_servidor_central_archivo_con_un_usuario() {
    let nombre_archivo = "archivos_test/test02.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");
    let (servidor, _tx_servidor, _rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());
    assert!(!servidor.usuarios.is_empty());
    assert!(!servidor.estado_usuarios.is_empty());
    assert!(servidor.usuarios_conectados.is_empty());
    assert!(servidor.usuarios_disponibles.is_empty());

    assert_eq!(
        servidor.usuarios.get_key_value("melaniegl"),
        Some((&"melaniegl".to_string(), &"radiohead".to_string()))
    );
    assert_eq!(
        servidor.estado_usuarios.get_key_value("melaniegl"),
        Some((&"melaniegl".to_string(), &EstadoUsuario::Desconectado))
    );
    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test03_se_crea_servidor_central_archivo_con_varios_usuarios() {
    let nombre_archivo = "archivos_test/test03.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [
        ("melaniegl", "radiohead"),
        ("larita", "mafalda424"),
        ("tomiTo", "futbol2004"),
        ("superCreativa", "maniConChocolate"),
    ];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, _tx_servidor, _rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());
    assert!(!servidor.usuarios.is_empty());
    assert!(!servidor.estado_usuarios.is_empty());
    assert!(servidor.usuarios_conectados.is_empty());
    assert!(servidor.usuarios_disponibles.is_empty());

    assert_eq!(
        servidor.usuarios.get_key_value("melaniegl"),
        Some((&"melaniegl".to_string(), &"radiohead".to_string()))
    );
    assert_eq!(
        servidor.estado_usuarios.get_key_value("melaniegl"),
        Some((&"melaniegl".to_string(), &EstadoUsuario::Desconectado))
    );

    assert_eq!(
        servidor.usuarios.get_key_value("larita"),
        Some((&"larita".to_string(), &"mafalda424".to_string()))
    );
    assert_eq!(
        servidor.estado_usuarios.get_key_value("larita"),
        Some((&"larita".to_string(), &EstadoUsuario::Desconectado))
    );

    assert_eq!(
        servidor.usuarios.get_key_value("tomiTo"),
        Some((&"tomiTo".to_string(), &"futbol2004".to_string()))
    );
    assert_eq!(
        servidor.estado_usuarios.get_key_value("tomiTo"),
        Some((&"tomiTo".to_string(), &EstadoUsuario::Desconectado))
    );

    assert_eq!(
        servidor.usuarios.get_key_value("superCreativa"),
        Some((
            &"superCreativa".to_string(),
            &"maniConChocolate".to_string()
        ))
    );
    assert_eq!(
        servidor.estado_usuarios.get_key_value("superCreativa"),
        Some((&"superCreativa".to_string(), &EstadoUsuario::Desconectado))
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test04_fallo_se_quiere_registrar_usuario_existente() {
    let nombre_archivo = "archivos_test/test04.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    assert_eq!(
        servidor.usuarios.get_key_value("melaniegl"),
        Some((&"melaniegl".to_string(), &"radiohead".to_string()))
    );

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Registrar(
            "melaniegl".to_string(),
            "meuDeuManito".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta_servidor,
        MensajeUsuario::Error("Nombre usuario no disponible".to_string())
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test04b_fallo_se_quiere_registrar_usuario_invalido() {
    let nombre_archivo = "archivos_test/test04b.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    assert_eq!(
        servidor.usuarios.get_key_value("melaniegl"),
        Some((&"melaniegl".to_string(), &"radiohead".to_string()))
    );

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Registrar(
            "    ".to_string(),
            "meuDeuManito".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta_servidor,
        MensajeUsuario::Error("Usuario invalido".to_string())
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test05_fallo_contrasenia_invalida() {
    let nombre_archivo = "archivos_test/test05.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Registrar(
            "mirkito".to_string(),
            " ".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta_servidor,
        MensajeUsuario::Error("Contraseña invalida".to_string())
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test06_registro_usuario_valido() {
    let nombre_archivo = "archivos_test/test06.txt";

    File::create(nombre_archivo).expect("Fallo creando archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Registrar(
            "dynamo".to_string(),
            "manuelitaViviaEnPehuajo".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");

    assert_eq!(respuesta_servidor, MensajeUsuario::Ok);
}

#[test]
fn test07_fallo_logueo_usuario_inexistente() {
    let nombre_archivo = "archivos_test/test07.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [
        ("melaniegl", "radiohead"),
        ("larita", "mafalda424"),
        ("tomiTo", "futbol2004"),
        ("superCreativa", "maniConChocolate"),
    ];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "dynamo".to_string(),
            "soda1990".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta_servidor,
        MensajeUsuario::Error(String::from("Nombre usuario inexistente"))
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test08_fallo_logueo_usuario_contrasenia_invalida() {
    let nombre_archivo = "archivos_test/test08.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [
        ("melaniegl", "radiohead"),
        ("larita", "mafalda424"),
        ("tomiTo", "futbol2004"),
        ("superCreativa", "maniConChocolate"),
    ];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "theSmiths".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta_servidor,
        MensajeUsuario::Error(String::from("Contraseña incorrecta"))
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test09_se_loguea_usuario() {
    let nombre_archivo = "archivos_test/test09.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta2_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le  envia estado del resto de los usuarios
    let mut usuarios = HashMap::new();
    usuarios.insert(String::from("larita"), EstadoUsuario::Desconectado);

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios)
    );
    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test09b_se_loguea_usuario_otro_no_puede_loguearse_con_ese_usuario() {
    let nombre_archivo = "archivos_test/test09b.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta2_servidor = rx_usuario
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le  envia estado del resto de los usuarios
    let mut usuarios = HashMap::new();
    usuarios.insert(String::from("larita"), EstadoUsuario::Desconectado);

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios)
    );

    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta_servidor2 = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta_servidor2,
        MensajeUsuario::Error("Usuario ya logueado".to_string())
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test10_se_notifica_usuarios_disponibles_sobre_nuevo_usuario() {
    let nombre_archivo = "archivos_test/test10.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios = HashMap::new();
    usuarios.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios)
    );

    tx_servidor
        .send(MensajeServidor::Registrar(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta4_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta4_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("dynamo"),
            EstadoUsuario::Desconectado
        )
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test11_se_notifica_usuarios_disponibles_sobre_nuevo_usuario() {
    let nombre_archivo = "archivos_test/test11.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios = HashMap::new();
    usuarios.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios)
    );

    tx_servidor
        .send(MensajeServidor::Registrar(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta4_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta4_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("dynamo"),
            EstadoUsuario::Desconectado
        )
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test12_se_rechaza_llamada_si_usuario_no_disponible() {
    let nombre_archivo = "archivos_test/test12.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios = HashMap::new();
    usuarios.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios)
    );

    tx_servidor
        .send(MensajeServidor::Registrar(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta4_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta4_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("dynamo"),
            EstadoUsuario::Desconectado
        )
    );

    tx_servidor
        .send(MensajeServidor::Llamar(
            "melaniegl".to_string(),
            "dynamo".to_string(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta5_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta5_servidor, MensajeUsuario::LlamadaRechazada);

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test13_se_llama_a_otro_usuario_flujo_completo() {
    let nombre_archivo = "archivos_test/test13.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    //LOGUEO USUARIO 2
    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios1 = HashMap::new();
    usuarios1.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios1)
    );

    // REGISTRO USUARIO 1
    tx_servidor
        .send(MensajeServidor::Registrar(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1.clone(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta4_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta4_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("dynamo"),
            EstadoUsuario::Desconectado
        )
    );

    //LOGUEO USUARIO 1
    tx_servidor
        .send(MensajeServidor::Loguear(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta5_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta5_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios2 = HashMap::new();
    usuarios2.insert(String::from("larita"), EstadoUsuario::Desconectado);
    usuarios2.insert(String::from("melaniegl"), EstadoUsuario::Disponible);

    let respuesta6_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta6_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios2)
    );

    //se le avisa al resto de usuarios disponibles sobre este cambio en el estado
    let respuesta6b_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta6b_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(String::from("dynamo"), EstadoUsuario::Disponible)
    );

    //USUARIO 2 LLAMA A USUARIO 1
    tx_servidor
        .send(MensajeServidor::Llamar(
            "melaniegl".to_string(),
            "dynamo".to_string(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta7_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta7_servidor,
        MensajeUsuario::LlamadaEntrante(String::from("melaniegl"))
    );

    //USUARIO1 ACEPTA LLAMADA A USUARIO2
    tx_servidor
        .send(MensajeServidor::AceptarLlamada("melaniegl".to_string()))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta8_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le pide offer a USUARIO2
    assert_eq!(respuesta8_servidor, MensajeUsuario::PedirOffer);

    //USUARIO 2 ENVIA OFFER A USUARIO1
    tx_servidor
        .send(MensajeServidor::EnviarOffer(
            "dynamo".to_string(),
            String::from("datita offer"),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta9_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le pide annswer a USUARIO2
    assert_eq!(
        respuesta9_servidor,
        MensajeUsuario::PedirAnswer(String::from("datita offer"))
    );

    //USUARIO1 ENVIA ANSWER A USUARIO2
    tx_servidor
        .send(MensajeServidor::EnviarAnswer(
            "melaniegl".to_string(),
            "dynamo".to_string(),
            String::from("datita answer"),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta10_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le pide annswer a USUARIO2
    assert_eq!(
        respuesta10_servidor,
        MensajeUsuario::EnviarAnswer(String::from("datita answer"))
    );

    //AMBOS PASAN A ESTAR EN ESTADO OCUPADO

    //CORTAN LA LLAMADA Y PASAN A ESTAR EN ESTADO DISPONIBLE

    //CORTA USUARIO1
    tx_servidor
        .send(MensajeServidor::EstadoDisponible("dynamo".to_string()))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta11_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le  envia estado del resto de los usuarios
    let mut usuarios3 = HashMap::new();
    usuarios3.insert(String::from("larita"), EstadoUsuario::Desconectado);
    usuarios3.insert(String::from("melaniegl"), EstadoUsuario::Ocupado);

    assert_eq!(
        respuesta11_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios3)
    );

    //CORTA USUARIO2
    tx_servidor
        .send(MensajeServidor::EstadoDisponible("melaniegl".to_string()))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta12_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le  envia estado del resto de los usuarios
    let mut usuarios4 = HashMap::new();
    usuarios4.insert(String::from("larita"), EstadoUsuario::Desconectado);
    usuarios4.insert(String::from("dynamo"), EstadoUsuario::Disponible);

    assert_eq!(
        respuesta12_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios4)
    );
    // a usuario 1 le llego notificacion de usuario 2 disponible
    let respuesta13_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");
    assert_eq!(
        respuesta13_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("melaniegl"),
            EstadoUsuario::Disponible
        )
    );

    //SE DESCONECTAN

    //se desconecta USUARIO1 y se le notifica USUARIO 2
    tx_servidor
        .send(MensajeServidor::Desconectarse("dynamo".to_string()))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta14_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le avisa a usuario 2 que el usuario1 se desconecto
    assert_eq!(
        respuesta14_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("dynamo"),
            EstadoUsuario::Desconectado
        )
    );

    let respuesta15_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta15_servidor, MensajeUsuario::Ok);

    //Se desconecta usuario2
    //CORTA USUARIO1
    tx_servidor
        .send(MensajeServidor::Desconectarse("melaniegl".to_string()))
        .expect("Fallo al enviar mensaje al servidor");
    let respuesta16_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta16_servidor, MensajeUsuario::Ok);

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test14_usuario_rechaza_llamada_entrante() {
    let nombre_archivo = "archivos_test/test14.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (servidor, tx_servidor, rx_servidor) =
        generar_estructura_servidor_default_tests(nombre_archivo.to_string());

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    //LOGUEO USUARIO 2
    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios1 = HashMap::new();
    usuarios1.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios1)
    );

    // REGISTRO USUARIO 1
    tx_servidor
        .send(MensajeServidor::Registrar(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1.clone(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    //El usuario disponible recibe la notificacion de que hay un nuevo usuario desconectado
    let respuesta4_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta4_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(
            String::from("dynamo"),
            EstadoUsuario::Desconectado
        )
    );

    //LOGUEO USUARIO 1
    tx_servidor
        .send(MensajeServidor::Loguear(
            "dynamo".to_string(),
            "amoeba".to_string(),
            tx_usuario1,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta5_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta5_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios2 = HashMap::new();
    usuarios2.insert(String::from("larita"), EstadoUsuario::Desconectado);
    usuarios2.insert(String::from("melaniegl"), EstadoUsuario::Disponible);

    let respuesta6_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta6_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios2)
    );

    //se le avisa al resto de usuarios disponibles sobre este cambio en el estado
    let respuesta6b_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta6b_servidor,
        MensajeUsuario::ActualizarEstadoUsuario(String::from("dynamo"), EstadoUsuario::Disponible)
    );

    //USUARIO 2 LLAMA A USUARIO 1
    tx_servidor
        .send(MensajeServidor::Llamar(
            "melaniegl".to_string(),
            "dynamo".to_string(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta7_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta7_servidor,
        MensajeUsuario::LlamadaEntrante(String::from("melaniegl"))
    );

    //USUARIO1 RECHAZA LLAMADA A USUARIO2
    tx_servidor
        .send(MensajeServidor::RechazarLlamada("melaniegl".to_string()))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta8_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    //se le pide offer a USUARIO2
    assert_eq!(respuesta8_servidor, MensajeUsuario::LlamadaRechazada);

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test15_usuario_no_puede_conectarse_si_se_alcanza_maximo_conectados() {
    let nombre_archivo = "archivos_test/test15.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

    //se limita a un usuario conectado en simultaneo
    let servidor =
        ServidorCentral::crear(nombre_archivo.to_string(), 1).expect("Error al crear el servidor");

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    //LOGUEO USUARIO 2
    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios1 = HashMap::new();
    usuarios1.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios1)
    );

    // LOGUEO AL USUARIO1
    tx_servidor
        .send(MensajeServidor::Loguear(
            "larita".to_string(),
            "mafalda424".to_string(),
            tx_usuario1.clone(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta3_servidor,
        MensajeUsuario::Error(String::from("No hay lugar para una nueva conexion"))
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}

#[test]
fn test16_usuario_puede_conectarse_si_se_desconecta_otro() {
    let nombre_archivo = "archivos_test/test16.txt";
    File::create(nombre_archivo).expect("Fallo creando archivo");

    let usuarios = [("melaniegl", "radiohead"), ("larita", "mafalda424")];

    let mut buffer = String::new();
    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let mut f = File::options()
        .append(true)
        .open(nombre_archivo)
        .expect("Fallo al abrir el archivo");
    writeln!(&mut f, "{}", buffer).expect("Fallo al escribir el archivo");

    let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

    //se limita a un usuario conectado en simultaneo
    let servidor =
        ServidorCentral::crear(nombre_archivo.to_string(), 1).expect("Error al crear el servidor");

    thread::spawn(move || {
        servidor
            .iniciar_escucha(rx_servidor)
            .expect("Fallo en servidor central al escuchar mensajes");
    });

    let (tx_usuario1, rx_usuario1) = mpsc::channel::<MensajeUsuario>();
    let (tx_usuario2, rx_usuario2) = mpsc::channel::<MensajeUsuario>();

    //LOGUEO USUARIO 2
    tx_servidor
        .send(MensajeServidor::Loguear(
            "melaniegl".to_string(),
            "radiohead".to_string(),
            tx_usuario2,
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta1_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta1_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios1 = HashMap::new();
    usuarios1.insert(String::from("larita"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios1)
    );

    // LOGUEO AL USUARIO1
    tx_servidor
        .send(MensajeServidor::Loguear(
            "larita".to_string(),
            "mafalda424".to_string(),
            tx_usuario1.clone(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta3_servidor,
        MensajeUsuario::Error(String::from("No hay lugar para una nueva conexion"))
    );

    // DESCONECTO AL USUARIO2
    tx_servidor
        .send(MensajeServidor::Desconectarse("melaniegl".to_string()))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario2
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    // LOGUEO AL USUARIO1
    tx_servidor
        .send(MensajeServidor::Loguear(
            "larita".to_string(),
            "mafalda424".to_string(),
            tx_usuario1.clone(),
        ))
        .expect("Fallo al enviar mensaje al servidor");

    let respuesta3_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(respuesta3_servidor, MensajeUsuario::Ok);

    //se le  envia estado del resto de los usuarios
    let mut usuarios1 = HashMap::new();
    usuarios1.insert(String::from("melaniegl"), EstadoUsuario::Desconectado);

    let respuesta2_servidor = rx_usuario1
        .recv()
        .expect("Fallo al recibir respuesta de servidor central");

    assert_eq!(
        respuesta2_servidor,
        MensajeUsuario::EstadoUsuarios(usuarios1)
    );

    fs::remove_file(nombre_archivo).expect("Fallo eliminando el archivo");
}
