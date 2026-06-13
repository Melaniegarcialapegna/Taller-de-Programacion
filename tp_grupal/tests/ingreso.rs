use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, UdpSocket},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
};

use room_rtc_2c_25::{
    aplicacion::Aplicacion,
    comunicacion::{
        comunicador::Comunicador, comunicador_tcp::ComunicadorTCP, lobby::LobbyConComunicador,
        telefono::TelefonoConComunicador,
    },
    config_room_rtc::{ConfigRoomRTC, Direcciones},
    creacion_llamada::{
        creador_de_conexion_encriptada::CreadorDeConexionEncriptada,
        mediador_de_conexiones::MediadorDeConexionesP2P,
    },
    entrada::Recepcion,
    llamada::{
        DispositivosEntrada, LlamadaRTP, ReproductoresLlamada, camara_mock::CamaraMock,
        microfono::MicrofonoMock,
    },
    logger::Logger,
    reproductor::{
        reproductor_audio::{ReproductorAudio, ReproductorAudioDummy},
        reproductor_rtp::ReproductorDeSesionRTP,
        reproductor_sin_decoder::ReproductorSinDecoder,
    },
    vista::vista_mock::VistaMock,
};

const DIRECCION_STUN: &str = "stun.cloudflare.com:3478";

struct EntornoServidor {
    puerto: u16,
    ruta_config: String,
    ruta_usuarios: String,
    ruta_log: String,
}

#[test]
fn test_integracion_se_abre_servidor_correctamente() {
    let resultado_proceso_servidor = Command::new("target/debug/servidor")
        .arg("tests/servidor_test.conf")
        .spawn();

    assert!(if let Ok(mut proceso) = resultado_proceso_servidor {
        proceso.kill().unwrap();
        true
    } else {
        false
    });
}

#[test]
fn test_integracion_se_registra_exitosamente_y_se_informa_del_evento() {
    let numero_test = encontrar_puerto_libre(3000, 3097);
    let entorno_servidor = crear_archivo_config_servidor(numero_test);
    let proceso_servidor = crear_servidor(&entorno_servidor.ruta_config);

    let (mut aplicacion, mut vista) =
        iniciar_aplicacion_y_vista(entorno_servidor.puerto, numero_test);

    aplicacion
        .registrarse("ernesto", "ernesto")
        .expect("Deberia permitir registrarse");
    vista.esperar_y_procesar_evento();

    // Cierro servidor y borro archivos creados
    borrar_archivos_usuario(numero_test);
    cerrar_servidor_y_borrar_archivos(entorno_servidor, proceso_servidor);
    assert!(vista.se_informo_registro_exitoso());
}

#[test]
fn test_integracion_se_inicia_sesion_si_antes_se_registro_el_usuario() {
    let numero_test = encontrar_puerto_libre(3100, 3197);
    let entorno_servidor = crear_archivo_config_servidor(numero_test);
    let proceso_servidor = crear_servidor(&entorno_servidor.ruta_config);

    let (mut aplicacion, mut vista) =
        iniciar_aplicacion_y_vista(entorno_servidor.puerto, numero_test);

    aplicacion
        .registrarse("ernesto", "ernesto")
        .expect("Deberia permitir registrarse");
    vista.esperar_y_procesar_evento();
    aplicacion
        .iniciar_sesion("ernesto", "ernesto")
        .expect("Deberia permitir iniciar sesion");
    vista.esperar_y_procesar_evento();
    vista.esperar_y_procesar_evento();

    // Cierro servidor y borro archivos creados
    borrar_archivos_usuario(numero_test);
    cerrar_servidor_y_borrar_archivos(entorno_servidor, proceso_servidor);
    assert!(vista.se_informo_registro_exitoso());
    assert!(vista.se_informo_sesion_iniciada());
}

#[test]
fn test_integracion_llamada_se_informa_al_otro_cliente() {
    let puerto_rtp_app_a = encontrar_puerto_libre(3200, 3297);
    let entorno_servidor = crear_archivo_config_servidor(puerto_rtp_app_a);
    let proceso_servidor = crear_servidor(&entorno_servidor.ruta_config);
    let puerto_rtp_app_b = encontrar_puerto_libre(3300, 3397);
    let (mut aplicacion_a, mut vista_a) =
        iniciar_aplicacion_y_vista(entorno_servidor.puerto, puerto_rtp_app_a);
    let (mut aplicacion_b, mut vista_b) =
        iniciar_aplicacion_y_vista(entorno_servidor.puerto, puerto_rtp_app_b);

    // Inicio sesion en Aplicacion A
    aplicacion_a.registrarse("a", "a").unwrap();
    vista_a.esperar_y_procesar_evento(); // RegistroExitoso
    aplicacion_a.iniciar_sesion("a", "a").unwrap();
    vista_a.esperar_y_procesar_evento(); // SesionIniciada
    vista_a.esperar_y_procesar_evento(); // Usuarios

    // Inicio sesion en Aplicacion B
    aplicacion_b.registrarse("b", "b").unwrap();
    vista_b.esperar_y_procesar_evento(); // RegistroExitoso
    aplicacion_b.iniciar_sesion("b", "b").unwrap();
    vista_b.esperar_y_procesar_evento(); // SesionIniciada
    vista_b.esperar_y_procesar_evento(); // Usuarios
    vista_a.esperar_y_procesar_evento(); // UsuariosNuevos("b", desc)
    vista_a.esperar_y_procesar_evento(); // UsuariosNuevos("b", conectado)

    // Llamo a Aplicacion B desde Aplicacion A
    aplicacion_a.llamar("b").unwrap();
    vista_a.esperar_y_procesar_evento(); // EnviandoLlamada("b")

    // Recibo llamada en Aplicacion B
    vista_b.esperar_y_procesar_evento(); // RecibiendoLlamada("a")

    cerrar_servidor_y_borrar_archivos(entorno_servidor, proceso_servidor);
    borrar_archivos_usuario(puerto_rtp_app_a);
    borrar_archivos_usuario(puerto_rtp_app_b);
    assert!(vista_a.se_informo_enviando_llamada("b"));
    assert!(vista_b.se_informo_recibiendo_llamada("a"));
}

// #[test]
// fn test_integracion_se_entra_a_llamada_en_ambos_peers_si_peer_b_atiende() {
//     let puerto_rtp_app_a = encontrar_puerto_libre(3400, 3497);
//     let entorno_servidor = crear_archivo_config_servidor(puerto_rtp_app_a);
//     let proceso_servidor = crear_servidor(&entorno_servidor.ruta_config);
//     let puerto_rtp_app_b = encontrar_puerto_libre(3500, 3597);
//     let (mut aplicacion_a, mut vista_a) =
//         iniciar_aplicacion_y_vista(entorno_servidor.puerto, puerto_rtp_app_a);
//     let (mut aplicacion_b, mut vista_b) =
//         iniciar_aplicacion_y_vista(entorno_servidor.puerto, puerto_rtp_app_b);

//     // Inicio sesion en Aplicacion A
//     aplicacion_a.registrarse("a", "a").unwrap();
//     vista_a.esperar_y_procesar_evento(); // RegistroExitoso
//     aplicacion_a.iniciar_sesion("a", "a").unwrap();
//     vista_a.esperar_y_procesar_evento(); // SesionIniciada
//     vista_a.esperar_y_procesar_evento(); // Usuarios

//     // Inicio sesion en Aplicacion B
//     aplicacion_b.registrarse("b", "b").unwrap();
//     vista_b.esperar_y_procesar_evento(); // RegistroExitoso
//     aplicacion_b.iniciar_sesion("b", "b").unwrap();
//     vista_b.esperar_y_procesar_evento(); // SesionIniciada
//     vista_b.esperar_y_procesar_evento(); // Usuarios
//     vista_a.esperar_y_procesar_evento(); // UsuariosNuevos("b", desc)
//     vista_a.esperar_y_procesar_evento(); // UsuariosNuevos("b", conectado)

//     // Llamo a Aplicacion B desde Aplicacion A
//     aplicacion_a.llamar("b").unwrap();
//     vista_a.esperar_y_procesar_evento(); // EnviandoLlamada("b")

//     // Recibo llamada en Aplicacion B y la atiendo
//     vista_b.esperar_y_procesar_evento(); // RecibiendoLlamada("a")
//     aplicacion_b.atender_llamada().unwrap();

//     // Se informa que la llamada esta iniciando a ambos peers
//     vista_a.esperar_y_procesar_evento(); // LlamadaIniciando
//     vista_b.esperar_y_procesar_evento(); // LlamadaIniciando

//     // Se informa que la llamada inicio en ambos peers
//     vista_a.esperar_y_procesar_evento(); // LlamadaIniciada
//     vista_b.esperar_y_procesar_evento(); // LlamadaIniciada

//     cerrar_servidor_y_borrar_archivos(entorno_servidor, proceso_servidor);
//     borrar_archivos_usuario(puerto_rtp_app_a);
//     borrar_archivos_usuario(puerto_rtp_app_b);
//     assert!(vista_a.se_informo_llamada_iniciada());
//     assert!(vista_b.se_informo_llamada_iniciada());
// }

fn encontrar_puerto_libre(desde: u16, hasta: u16) -> u16 {
    for numero in desde..hasta {
        if checkear_puerto_libre(numero)
            && checkear_puerto_libre(numero + 1)
            && checkear_puerto_libre(numero + 2)
        {
            return numero;
        }
    }

    panic!(
        "No hay puertos libres disponibles en el rango {}-{}",
        desde, hasta
    )
}

fn checkear_puerto_libre(puerto: u16) -> bool {
    let resultado_socket_udp = UdpSocket::bind(format!("127.0.0.1:{puerto}"));
    let resultado_socket_tcp = TcpListener::bind(format!("127.0.0.1:{puerto}"));

    let libre_udp = resultado_socket_udp.is_ok();
    let libre_tcp = resultado_socket_udp.is_ok();

    if let Ok(socket) = resultado_socket_tcp {
        drop(socket);
    }
    if let Ok(socket) = resultado_socket_udp {
        drop(socket);
    }

    libre_udp && libre_tcp
}

fn crear_archivo_config_servidor(puerto_rtp: u16) -> EntornoServidor {
    let puerto = puerto_rtp + 2;
    let ruta_usuarios = format!("tests/{}_usuarios.txt", puerto_rtp);
    let ruta_log = format!("/tests/{}_servidor.log", puerto_rtp);
    let contenido_test_servidor = format!(
        "host: 0.0.0.0\nport: {}\nlimite_usuarios: 10\nlog_file: {}\nusers_file: {}\n",
        puerto, ruta_log, ruta_usuarios
    );
    let ruta_config = format!("tests/{}_servidor.conf", puerto_rtp);
    dbg!(puerto);

    let mut archivo = File::create(&ruta_config).expect("Fallo creando config del servidor");
    archivo
        .write_all(contenido_test_servidor.as_bytes())
        .expect("Fallo escribiendo el archivo");

    EntornoServidor {
        puerto,
        ruta_config,
        ruta_usuarios,
        ruta_log,
    }
}

fn crear_servidor(ruta_config_servidor: &str) -> Child {
    // Iniciar servidor
    let mut proceso_servidor = Command::new("target/debug/servidor")
        .arg(ruta_config_servidor)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Se deberia poder iniciar el servidor");

    let salida = proceso_servidor.stdout.take().unwrap();
    let mut reader = BufReader::new(salida);
    reader.read_line(&mut String::new()).unwrap(); // "Archivo de configuración cargado"
    reader.read_line(&mut String::new()).unwrap(); // "Servidor iniciado correctamente"

    proceso_servidor
}

fn iniciar_aplicacion_y_vista(puerto_servidor: u16, numero_test: u16) -> (Aplicacion, VistaMock) {
    let mut aplicacion = crear_aplicacion_test(numero_test, puerto_servidor);
    let vista = VistaMock::default();
    let sender_eventos_a_vista = vista.obtener_sender_eventos();
    aplicacion
        .suscribir(sender_eventos_a_vista)
        .expect("Fallo al suscribirse a aplicacion");
    (aplicacion, vista)
}

#[cfg(test)]
fn crear_aplicacion_test(numero_test: u16, puerto_servidor: u16) -> Aplicacion {
    // Creo config
    let config = obtener_config_cliente_test(numero_test, puerto_servidor);

    // Creo comunicador
    let mut comunicador =
        ComunicadorTCP::crear_comunicador(config.getter_direccion_signaling()).unwrap();

    // Creo un comunicador para cada componente que lo necesite
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();
    let comunicador_telefono = comunicador.crear_companiero().unwrap();
    let comunicador_creador_conexiones = comunicador.crear_companiero().unwrap();

    // Creo sender y receiver de eventos internos
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();

    // Creo logger
    let logger = Logger::new(&format!("tests/{}_aplicacion.log", numero_test));

    // Creo recepcion
    let recepcion = Recepcion::new(comunicador_recepcion);

    // Creo lobby
    let lobby = LobbyConComunicador::new(
        Box::new(comunicador),
        sender_eventos_internos.clone(),
        logger.clone(),
    );

    // Creo telefono
    let (sender_eventos_llamada_a_telefono, receiver_eventos_llamada_de_telefono) = mpsc::channel();
    let telefono = TelefonoConComunicador::new(
        comunicador_telefono,
        sender_eventos_internos.clone(),
        receiver_eventos_llamada_de_telefono,
        logger.clone(),
    )
    .unwrap();

    // Creo mediador de conexiones
    let creador_conexion =
        CreadorDeConexionEncriptada::iniciar_creador(config, logger.clone()).unwrap();
    let (sender_eventos_a_llamada, receiver_eventos_llamada) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        comunicador_creador_conexiones,
        Box::new(creador_conexion),
        sender_eventos_internos.clone(),
        sender_eventos_a_llamada.clone(),
    );

    // Creo camara
    let camara = Arc::new(Mutex::new(CamaraMock::default()));

    // Creo reproductores
    let mut reproductor = ReproductorDeSesionRTP::new().unwrap();
    let sender_a_reproductor = reproductor.obtener_sender_frames();
    let (sender_frames_locales, receiver_frames_locales) = mpsc::channel();
    let reproductor_local = ReproductorSinDecoder::new(receiver_frames_locales);
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(reproductor_local),
        sender_frames_locales,
        Box::new(reproductor),
        reproductor_audio,
        sender_a_reproductor,
        sender_audio,
    );

    // Creo microfono
    let microfono = Arc::new(Mutex::new(MicrofonoMock::default()));

    // Creo dispositivos
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(microfono),
    };

    // Creo llamada
    let llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_eventos_internos.clone(),
        sender_eventos_a_llamada,
        receiver_eventos_llamada,
        sender_eventos_llamada_a_telefono,
        logger,
    )
    .unwrap();

    let resultado_aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada),
    );

    if let Err(error) = &resultado_aplicacion {
        dbg!(error);
    };

    resultado_aplicacion.unwrap()
}

fn obtener_config_cliente_test(puerto_rtp: u16, puerto_servidor: u16) -> ConfigRoomRTC {
    ConfigRoomRTC::crear_struct(
        Direcciones::new("127.0.0.1".to_string(), puerto_rtp),
        Direcciones::new("127.0.0.1".to_string(), puerto_rtp),
        format!("tests/{}.log", puerto_rtp),
        format!("tests/{}_offer.txt", puerto_rtp),
        format!("tests/{}_answer.txt", puerto_rtp),
        DIRECCION_STUN.to_string(),
        format!("127.0.0.1:{puerto_servidor}"),
    )
}

fn cerrar_servidor_y_borrar_archivos(
    entorno_servidor: EntornoServidor,
    mut proceso_servidor: Child,
) {
    borrar_archivo_si_existe(&entorno_servidor.ruta_config);
    borrar_archivo_si_existe(&entorno_servidor.ruta_log);
    borrar_archivo_si_existe(&entorno_servidor.ruta_usuarios);
    proceso_servidor.kill().unwrap(); // Para que el servidor siga en scope hasta este punto (despues de esta linea se va de scope)
}

fn borrar_archivos_usuario(puerto_rtp_app: u16) {
    let ruta_log_aplicacion = format!("tests/{}_aplicacion.log", puerto_rtp_app);
    let ruta_archivo_offer = format!("tests/{}_offer.txt", puerto_rtp_app);
    let ruta_archivo_answer = format!("tests/{}_answer.txt", puerto_rtp_app);
    let ruta_archivo_log = format!("tests/{}.log", puerto_rtp_app);

    borrar_archivo_si_existe(&ruta_log_aplicacion);
    borrar_archivo_si_existe(&ruta_archivo_offer);
    borrar_archivo_si_existe(&ruta_archivo_answer);
    borrar_archivo_si_existe(&ruta_archivo_log);
}

fn borrar_archivo_si_existe(ruta_archivo: &str) {
    if Path::new(ruta_archivo).exists() {
        fs::remove_file(ruta_archivo)
            .expect("Deberia borrarse el archivo de configuracion del servidor");
    };
}
