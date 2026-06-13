use std::io::{Read, Write};
use std::net::TcpStream;

use crate::encriptacion::SistemaDeEncriptacion;
use crate::protocolos::pca::mensaje::MensajePCA;
use crate::servidor_utils::stream_tcp_utils::error_stream_tcp::ErrorStreamTCP;

pub trait StreamTCP: Write + Send + Sync + Read {
    fn leer_mensaje(
        &mut self,
        buffer: &mut String,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<usize, ErrorStreamTCP>;
    fn enviar_mensaje(
        &mut self,
        mensaje: MensajePCA,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<usize, ErrorStreamTCP>;
    fn clonar(&mut self) -> Result<Box<dyn StreamTCP>, ErrorStreamTCP>;
}

impl StreamTCP for TcpStream {
    fn leer_mensaje(
        &mut self,
        buffer: &mut String,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<usize, ErrorStreamTCP> {
        let mensaje_desencriptado = encriptador
            .leer_desencriptando_mensaje(self)
            .map_err(|_| ErrorStreamTCP::ErrorRecibiendoMensaje)?;

        buffer.push_str(&mensaje_desencriptado);

        Ok(mensaje_desencriptado.len())
    }

    fn enviar_mensaje(
        &mut self,
        mensaje: MensajePCA,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<usize, ErrorStreamTCP> {
        let mensaje_string = String::from(mensaje);

        let mensaje_encriptado = encriptador
            .encriptar_mensaje(&mensaje_string)
            .map_err(|_| ErrorStreamTCP::ErrorEnviandoMensaje)?;

        self.write(&mensaje_encriptado[..])
            .map_err(|_| ErrorStreamTCP::ErrorEnviandoMensaje)
    }

    fn clonar(&mut self) -> Result<Box<dyn StreamTCP>, ErrorStreamTCP> {
        let stream_clonado = self
            .try_clone()
            .map_err(|_| ErrorStreamTCP::ErrorClonando)?;

        Ok(Box::new(stream_clonado))
    }
}
