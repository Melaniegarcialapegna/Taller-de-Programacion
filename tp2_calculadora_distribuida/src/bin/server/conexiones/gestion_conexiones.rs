//! En este modulo se establece la conexion por la cual el servidor escuchara a los distintos clientes.
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use crate::errores::error_en_servidor::ErrorEnServidor;
use crate::logistica::logica_calculadora::calculadora::Calculadora;
use crate::{conexiones::handler_cliente::HandlerCliente, errores::error_en_thread::ErrorEnThread};

/// Establece la conexion y crea un thread nuevo por cada nueva conexion entrante, es decir, por cada cliente.
///
/// Si alguno de los los clientes cierra al conexion de manera inesprada se notificara por medio de `stderr`.
///
/// En caso de que no se pueda conectar a la direccion indicada se retorna un [`ErrorEnServidor`].
pub fn gestionar_conexiones(direccion: String) -> Result<(), ErrorEnServidor> {
    //Se hace la conexion para poder escuchar conexiones TCP
    let listener = TcpListener::bind(&direccion).map_err(|_| ErrorEnServidor::DireccionInvalida)?;

    //Se crea la calculadora que tendran en comun los distintos clientes.
    let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

    for stream in listener.incoming() {
        //cada stream es un nuevo cliente
        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("{}", error);
                continue;
            }
        };

        //Creo la referencia de la calculadora que se le otorgara al cliente.
        let counter_calculadora = Arc::clone(&counter_calculadora);

        thread::spawn(move || {
            if let Err(error) = manejo_cliente(stream, counter_calculadora) {
                eprintln!("{}", error); //se muestra error sin la finalizacion del programa del servidor.
            }
        });
    }

    Ok(())
}

/// Por cada cliente se creara un [`HandlerCliente`] que se encargara de manejar las peticiones de este.
fn manejo_cliente(
    stream: TcpStream,
    counter_calculadora: Arc<Mutex<Calculadora>>,
) -> Result<(), ErrorEnThread> {
    // Se crea una duplica del stream que estara apuntando al mismo socket.
    let reader_cliente = stream
        .try_clone()
        .map_err(|_| ErrorEnThread::DuplicarStream)?;

    let writer_stream = stream;

    let reader_stream = BufReader::new(reader_cliente);

    let buffer = String::new();

    let mut handler_cliente =
        HandlerCliente::new(reader_stream, writer_stream, counter_calculadora, buffer);
    handler_cliente.gestionar()
}
