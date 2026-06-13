use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
    thread,
};

use room_rtc_2c_25::{config_servidor::ConfigServidor, encriptacion::SistemaDeEncriptacion};

use crate::servidor_utils::{
    handler_usuario_utils::handler_usuario::HandlerUsuario,
    mensajes::mensaje_servidor::MensajeServidor,
    servidor_central_utils::servidor_central::ServidorCentral,
};

#[derive(Debug)]
pub enum ErrorConexiones {
    ErrorIniciarServidor,
    ErrorIniciarServidorCentral,
    DuplicarStream,
    ErrorUsuario,
    ErrorEncriptacion,
}

pub fn iniciar_servidor(config: ConfigServidor) -> Result<(), ErrorConexiones> {
    let direccion = config.get_direccion();

    let listener =
        TcpListener::bind(direccion).map_err(|_| ErrorConexiones::ErrorIniciarServidor)?;

    //Channel de comunicacion cliente->servidor
    let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

    //Lo pongo afuera ya que si no se puede levantar por alguna razon los usuarios no se tendria que poder iniciar el servidor(¿)
    let ruta_archivo_usuarios = config.getter_users_file().to_string();
    let limite_usuarios_conectados = config.getter_limite_usuarios();
    let servidor_central =
        ServidorCentral::crear(ruta_archivo_usuarios, limite_usuarios_conectados)
            .map_err(|_| ErrorConexiones::ErrorIniciarServidorCentral)?;

    thread::spawn(move || {
        if servidor_central.iniciar_escucha(rx_servidor).is_err() {
            eprintln!("Error en servidor");
        }
    });

    println!("Servidor iniciado correctamente");
    for stream in listener.incoming() {
        //cada stream es un nuevo usuario

        match stream {
            Ok(stream) => {
                let ref_tx_servidor = tx_servidor.clone();

                thread::spawn(move || {
                    if let Err(error) = gestionar_usuario(stream, ref_tx_servidor) {
                        eprintln!("{:?}", error);
                    }
                });
            }
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        };
    }
    Ok(())
}

fn gestionar_usuario(
    stream: TcpStream,
    tx_servidor: Sender<MensajeServidor>,
) -> Result<(), ErrorConexiones> {
    // Se crea una duplica del stream que estara apuntando al mismo socket.
    let mut reader_stream = stream
        .try_clone()
        .map_err(|_| ErrorConexiones::DuplicarStream)?;

    let encriptacion = SistemaDeEncriptacion::encriptar_conexion(&mut reader_stream)
        .map_err(|_| ErrorConexiones::ErrorEncriptacion)?;

    let writer_stream = stream;

    let usuario = HandlerUsuario::new(tx_servidor, encriptacion);
    usuario
        .gestionar(Box::new(reader_stream), Box::new(writer_stream))
        .map_err(|_| ErrorConexiones::ErrorUsuario)
}
