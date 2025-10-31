//! En este modulo se establece la conexion con el servidor.

use crate::{conexion::handler_archivo::HandlerArchivo, errores::error_en_cliente::ErrorEnCliente};
use std::{io::BufReader, net::TcpStream};

///Establece la conexion con el servidor mediante la `direccion` indicada.
///
/// En caso de algun tipo de error, como que no se pueda establer la conexion, se retorna un [`ErrorEnCliente`].  
pub fn establecer_conexion(direccion: String, input_file: String) -> Result<(), ErrorEnCliente> {
    //Se hace la conexion con el servidor.
    let stream = TcpStream::connect(&direccion).map_err(|_| ErrorEnCliente::DireccionInvalida)?;

    // Se crea una duplica del stream que estara apuntando al mismo socket.
    let reader_servidor = stream
        .try_clone()
        .map_err(|_| ErrorEnCliente::DuplicarStream)?;

    let writer_stream = stream;

    let reader_stream = BufReader::new(reader_servidor);

    let buffer = String::new();

    // Se maneja la conexion con el servidor mediante un [`HandlerArchivo`]
    let mut handler_archivo = HandlerArchivo::new(input_file, reader_stream, writer_stream, buffer);
    handler_archivo.gestionar()?;

    Ok(())
}
