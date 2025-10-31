//! Este modulo define el tipo de dato `HandlerArchivo`.
//!
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use crate::errores::error_en_cliente::ErrorEnCliente;
use crate::logistica::respuesta_servidor::RespuestaServidor;

//constantes
const MENSAJE_OP: &str = "OP";
const MENSAJE_GET: &str = "GET";

/// Maneja la conexion con un servidor enviandole las operacion que contiene un `archivo`.
pub struct HandlerArchivo<R, W>
where
    R: BufRead,
    W: Write,
{
    /// El nombre del archivo del cual se leeran las operaciones.
    ruta_archivo: String,
    /// Canal por el cual se leeran las respuestas del servidor.
    reader_stream: R,
    /// Canal por el cual se le enviaran las operaciones al servidor.
    writer_stream: W,
    /// En donde se almacenaran los mensajes enviados por el servidor.
    buffer: String,
}

impl<R, W> HandlerArchivo<R, W>
where
    R: BufRead,
    W: Write,
{
    ///Crea y devuelve una instancia de [`HandlerArchivo`].
    pub fn new(ruta_archivo: String, reader_stream: R, writer_stream: W, buffer: String) -> Self {
        HandlerArchivo {
            ruta_archivo,
            reader_stream,
            writer_stream,
            buffer,
        }
    }

    /// Envia cada linea del archivo (que representa una operacion) al servidor y maneja su respuesta.
    ///
    /// Una vez que termina de procesar el archivo le pide un resultado final.
    ///
    /// En caso de algun error retorna [`ErrorEnCliente`].
    pub fn gestionar(&mut self) -> Result<(), ErrorEnCliente> {
        let reader_file = self.abrir_archivo()?;

        for operacion in reader_file.lines() {
            let operacion = operacion.map_err(|_| ErrorEnCliente::LeerLineaArchivo)?;

            self.enviar_mensaje_servidor(MENSAJE_OP, Some(operacion))?;

            self.leer_y_manejar_respuesta_servidor()?;
        }
        self.enviar_mensaje_servidor(MENSAJE_GET, None)?;

        self.leer_y_manejar_respuesta_servidor()?;

        Ok(())
    }

    /// Abre el archivo y lo devuelve dentro de un [`BufReader`] para poder manejarlo mas comodamente.
    ///
    /// Si ocurre algun error retorna un [`ErrorEnCliente`].
    fn abrir_archivo(&self) -> Result<BufReader<File>, ErrorEnCliente> {
        let file = File::open(&self.ruta_archivo).map_err(|_| ErrorEnCliente::AbrirArchivo)?;

        Ok(BufReader::new(file))
    }

    ///Envia un mensaje al servidor a traves del `writer_stream`.
    ///
    /// Si ocurre algun error retorna un [`ErrorEnCliente`].
    fn enviar_mensaje_servidor(
        &mut self,
        mensaje: &str,
        operacion: Option<String>,
    ) -> Result<(), ErrorEnCliente> {
        let mensaje = match operacion {
            Some(operacion) => format!("{} {}\n", mensaje, operacion),
            None => format!("{}\n", mensaje),
        };

        self.writer_stream
            .write(mensaje.as_bytes())
            .map_err(|_| ErrorEnCliente::EscrituraStream)?;
        self.writer_stream
            .flush()
            .map_err(|_| ErrorEnCliente::EsperarEscritura)?;
        Ok(())
    }

    /// Lee la respuesta del servidor a traves del `reader_stream` y se encarga de manejarla.
    ///
    /// Si ocurre algun error retorna un [`ErrorEnCliente`].
    fn leer_y_manejar_respuesta_servidor(&mut self) -> Result<(), ErrorEnCliente> {
        self.buffer.clear();

        self.reader_stream
            .read_line(&mut self.buffer)
            .map_err(|_| ErrorEnCliente::LecturaStream)?;

        let respuesta = self.buffer.trim();

        self.manejar_output_respuesta(respuesta)?;

        Ok(())
    }

    /// Le muestra al cliente en caso de ser necesario lo que el servidor le envio como respuesta.
    ///
    /// Si el servidor le indica al cliente que hubo un error con su peticion se lo indica por `stderr`.
    ///
    /// Si ocurre algun error retorna un [`ErrorEnCliente`].
    fn manejar_output_respuesta(&self, respuesta: &str) -> Result<(), ErrorEnCliente> {
        match RespuestaServidor::generar_respuesta(respuesta)? {
            RespuestaServidor::OK => Ok(()),
            RespuestaServidor::Valor(valor) => {
                println!("{}", valor);
                Ok(())
            }
            RespuestaServidor::Error(error) => {
                eprintln!("{}", error);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_enviar_mensaje_servidor() {
        test_enviar_mensaje_servidor_op_valida_generica("+ 4".to_string(), "OP + 4\n".as_bytes());

        test_enviar_mensaje_servidor_op_valida_generica("- 1".to_string(), "OP - 1\n".as_bytes());

        test_enviar_mensaje_servidor_op_valida_generica("* 14".to_string(), "OP * 14\n".as_bytes());

        test_enviar_mensaje_servidor_op_valida_generica("/ 2".to_string(), "OP / 2\n".as_bytes());
    }

    fn test_enviar_mensaje_servidor_op_valida_generica(
        operacion: String,
        escritura_esperada: &[u8],
    ) {
        let nombre_archivo = String::from("random");

        let msg = String::from("random");
        let reader = BufReader::new(msg.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut archivo = HandlerArchivo::new(nombre_archivo, reader, writer, buffer);

        archivo
            .enviar_mensaje_servidor("OP", Some(operacion))
            .unwrap();

        assert_eq!(archivo.writer_stream, escritura_esperada);
    }

    #[test]
    fn test02_enviar_mensaje_servidor_get() {
        let nombre_archivo = String::from("random");

        let msg = String::from("random");
        let reader = BufReader::new(msg.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut archivo = HandlerArchivo::new(nombre_archivo, reader, writer, buffer);

        archivo.enviar_mensaje_servidor("GET", None).unwrap();

        let esperado = b"GET\n";

        assert_eq!(archivo.writer_stream, esperado);
    }

    #[test]
    fn test03_leer_respuesta_servidor() {
        test_leer_respuesta_servidor_generica("OK\n".to_string(), "OK\n".as_bytes());

        test_leer_respuesta_servidor_generica("VALUE 24\n".to_string(), "VALUE 24\n".as_bytes());
    }

    fn test_leer_respuesta_servidor_generica(respuesta_simulada: String, lectura_esperada: &[u8]) {
        let nombre_archivo = String::from("random");

        let reader = BufReader::new(respuesta_simulada.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut archivo = HandlerArchivo::new(nombre_archivo, reader, writer, buffer);

        archivo.leer_y_manejar_respuesta_servidor().unwrap();

        assert_eq!(archivo.buffer.as_bytes(), lectura_esperada);
    }
}
