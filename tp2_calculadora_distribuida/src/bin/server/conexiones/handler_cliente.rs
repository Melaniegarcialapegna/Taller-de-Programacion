//! Este modulo define el tipo de dato `HandlerCliente`.
use std::{
    io::prelude::*,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::errores::error_en_thread::ErrorEnThread;
use crate::logistica::{
    logica_calculadora::calculadora::Calculadora, mensaje_cliente::MensajeCliente,
    respuesta::Respuesta,
};

const CERO_BYTES_LEIDOS: usize = 0;

/// Maneja la conexion de un cliente en particular.
pub struct HandlerCliente<R, W>
where
    R: BufRead,
    W: Write,
{
    ///Canal por el cual se leeran las peticiones del cliente.
    reader_stream: R,
    ///Canal por el cual se le enviaran repuestas al cliente.
    writer_stream: W,
    /// Calculadora sobre la que se aplicaran las peticiones del cliente.
    counter_calculadora: Arc<Mutex<Calculadora>>,
    /// Indica si el cliente termino o no de enviar peticiones
    ///
    /// Un cliente termina de enviar peticiones cuando envia el mensaje `Get`.
    se_recibio_mensaje_get: bool,
    /// En donde se almacenaran los mensajes enviados por el cliente.
    buffer: String,
}

impl<R, W> HandlerCliente<R, W>
where
    R: BufRead,
    W: Write,
{
    ///Crea y devuelve una nueva instancia de [`HandlerCliente`].
    pub fn new(
        reader_stream: R,
        writer_stream: W,
        counter_calculadora: Arc<Mutex<Calculadora>>,
        buffer: String,
    ) -> Self {
        HandlerCliente {
            reader_stream,
            writer_stream,
            counter_calculadora,
            se_recibio_mensaje_get: false,
            buffer,
        }
    }

    /// Se encarga del manejo general del cliente.
    ///
    /// Permanece en un loop hasta que el cliente cierre la conexion de forma abrupta o hasta que ya no tenga que leer.
    ///
    /// Lee la peticion del cliente, la procesa y genera una respuesta adecuada para enviarsela.
    pub fn gestionar(&mut self) -> Result<(), ErrorEnThread> {
        loop {
            self.buffer.clear();

            let bytes_leidos = self.leer_mensaje()?;
            if bytes_leidos == CERO_BYTES_LEIDOS {
                return Ok(()); //conexion se cerro de forma correcta
            }

            let mensaje = match self.procesar_mensaje()? {
                Some(mensaje) => mensaje,
                None => continue, //Si el servidor no soporto el mensaje enviado por el cliente se lo notifica
            };

            let mensaje_respuesta = self.ejecutar_mensaje(mensaje)?;

            self.enviar_respuesta(mensaje_respuesta)?;
        }
    }

    /// Lee el mensaje que el cliente le envio por medio del `reader_stream`.
    /// En caso de que la conexion aun no deberia haber terminado pero igualmente no haya que leer, o si hay algun error durante la lectura se retorna un [`ErrorEnThread`]
    fn leer_mensaje(&mut self) -> Result<usize, ErrorEnThread> {
        let bytes_leidos = self
            .reader_stream
            .read_line(&mut self.buffer)
            .map_err(|_| ErrorEnThread::LecturaStream)?;
        if bytes_leidos == CERO_BYTES_LEIDOS && !self.se_recibio_mensaje_get {
            //Se cerro la conexion antes de tiempo
            return Err(ErrorEnThread::CierreAbrupto);
        }
        Ok(bytes_leidos)
    }

    /// Procesa el mensaje enviado por el cliente, en caso de que este sea valido se retorna un [`MensajeCliente`] segun la finalidad de este (dentro de un Option). Si el mensaje es invalido se lo notifica al cliente y devuelve un `None`.
    ///
    /// Si hay algun error durante este procesamiento, por ejemplo, que el mensaje enviado por el cliente no sea un mensaje que el servidor sepa responder se le envia un mensaje al cliente especificandole esto.
    /// Y si durante el envio de este mensaje hay algun inconveniente se retorna un [`ErrorEnThread`]
    fn procesar_mensaje(&mut self) -> Result<Option<MensajeCliente>, ErrorEnThread> {
        match MensajeCliente::from_str(self.buffer.trim()) {
            Ok(mensaje) => Ok(Some(mensaje)),
            Err(error) => {
                self.enviar_respuesta(Respuesta::from(error))?; //Se le avisa al cliente que el mensaje que envio es invalido
                Ok(None)
            }
        }
    }

    /// Se encarga de ejecutar lo que el cliente pidio y devolver una [`Respuesta`] adecuada.
    ///
    /// Para esto pide el `lock` de la calculadora para asi poder ejecutar la peticion del cliente sobre esta.
    ///
    /// En caso de algun error durante estas operaciones se retorna un [`ErrorEnThread`]
    fn ejecutar_mensaje(&mut self, mensaje: MensajeCliente) -> Result<Respuesta, ErrorEnThread> {
        let mut calculadora = self
            .counter_calculadora
            .lock()
            .map_err(|_| ErrorEnThread::ObtenerLock)?;

        let respuesta = match mensaje {
            MensajeCliente::Get => {
                self.se_recibio_mensaje_get = true;
                Respuesta::Valor(calculadora.valor())
            }
            MensajeCliente::Operacion(operacion) => match calculadora.aplicar(operacion) {
                Ok(_) => Respuesta::OK,
                Err(error) => Respuesta::from(error),
            },
        };
        Ok(respuesta)
    } //se libera el lock

    /// Envia al cliente una [`Respuesta`] por medio del `writer_stream`.
    ///
    /// Si ocurre algun tipo de error se retorna un [`ErrorEnThread`]
    fn enviar_respuesta(&mut self, respuesta: Respuesta) -> Result<(), ErrorEnThread> {
        let respuesta_string = format!("{}\n", respuesta);
        self.writer_stream
            .write(respuesta_string.as_bytes())
            .map_err(|_| ErrorEnThread::EscrituraStream)?;
        self.writer_stream
            .flush()
            .map_err(|_| ErrorEnThread::EsperarEscritura)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logistica::{
        logica_calculadora::error_en_calculadora::ErrorEnCalculadora,
        logica_operacion::operacion::Operacion,
    };
    use std::io::BufReader;

    #[test]
    fn test01_leer_mensaje_cliente_valida() {
        test_leer_mensaje_cliente_valido_generico("OP + 1\n");

        test_leer_mensaje_cliente_valido_generico("OP - 8\n");

        test_leer_mensaje_cliente_valido_generico("OP * 4\n");

        test_leer_mensaje_cliente_valido_generico("OP / 2\n");
    }

    fn test_leer_mensaje_cliente_valido_generico(mensaje: &str) {
        //Creo la calculadora del servidor
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let reader = BufReader::new(mensaje.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        let bytes_leidos = cliente.leer_mensaje().unwrap();

        assert_eq!(cliente.buffer, mensaje);

        assert_eq!(bytes_leidos, mensaje.len());

        assert!(!cliente.se_recibio_mensaje_get);
    }

    #[test]
    fn test02_leer_mensaje_cliente_valida() {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let mensaje = "";
        let reader = BufReader::new(mensaje.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        cliente.se_recibio_mensaje_get = true;

        let bytes_leidos = cliente.leer_mensaje().unwrap();

        assert_eq!(cliente.buffer, mensaje);
        assert_eq!(bytes_leidos, mensaje.len());
        assert!(cliente.se_recibio_mensaje_get);
    }

    #[test]
    fn test03_leer_mensaje_cliente_invalida() {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let mensaje = "";
        let reader = BufReader::new(mensaje.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        let bytes_leidos = cliente.leer_mensaje();

        assert_eq!(bytes_leidos.unwrap_err(), ErrorEnThread::CierreAbrupto);
        assert!(!cliente.se_recibio_mensaje_get);
    }

    #[test]
    fn test04_procesar_mensaje_valido() {
        test_procesar_mensaje_valido_generico(
            "OP + 4".to_string(),
            MensajeCliente::Operacion(Operacion::Add(4)),
        );

        test_procesar_mensaje_valido_generico(
            "OP - 2".to_string(),
            MensajeCliente::Operacion(Operacion::Sub(2)),
        );

        test_procesar_mensaje_valido_generico(
            "OP * 2".to_string(),
            MensajeCliente::Operacion(Operacion::Mul(2)),
        );

        test_procesar_mensaje_valido_generico(
            "OP / 1".to_string(),
            MensajeCliente::Operacion(Operacion::Div(1)),
        );
    }

    fn test_procesar_mensaje_valido_generico(buffer: String, mensaje_esperado: MensajeCliente) {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let mensaje = " ";
        let reader = BufReader::new(mensaje.as_bytes());

        let writer = Vec::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        let mensaje = cliente
            .procesar_mensaje()
            .unwrap()
            .expect("Se esperaba un Some(mensaje)");

        assert_eq!(mensaje, mensaje_esperado);
        assert!(cliente.writer_stream.is_empty())
    }

    #[test]
    fn test05_procesar_mensaje_invalido() {
        test_procesar_mensaje_invalido_generico(
            "AA + 4".to_string(),
            "ERROR \"unexpected message\"\n".as_bytes(),
        );

        test_procesar_mensaje_invalido_generico(
            "+ 4".to_string(),
            "ERROR \"unexpected message\"\n".as_bytes(),
        );

        test_procesar_mensaje_invalido_generico(
            "OP + +".to_string(),
            "ERROR \"parsing error : invalid operand\"\n".as_bytes(),
        );

        test_procesar_mensaje_invalido_generico(
            "OP 4 4".to_string(),
            "ERROR \"parsing error : invalid operator\"\n".as_bytes(),
        );
    }

    fn test_procesar_mensaje_invalido_generico(buffer: String, mensaje_error_esperado: &[u8]) {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let mensaje = " ";
        let reader = BufReader::new(mensaje.as_bytes());

        let writer = Vec::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        let mensaje = cliente.procesar_mensaje().unwrap();

        assert_eq!(mensaje, None);
        assert_eq!(cliente.writer_stream, mensaje_error_esperado);
    }

    #[test]
    fn test06_ejecutar_mensaje_valido() {
        test_ejecutar_mensaje_valido_generico(
            MensajeCliente::from_str("OP + 4").unwrap(),
            Respuesta::OK,
        );

        test_ejecutar_mensaje_valido_generico(
            MensajeCliente::from_str("OP * 8").unwrap(),
            Respuesta::OK,
        );

        test_ejecutar_mensaje_valido_generico(
            MensajeCliente::from_str("OP / 2").unwrap(),
            Respuesta::OK,
        )
    }

    #[test]
    fn test07_ejecutar_mensaje_invalido() {
        test_ejecutar_mensaje_valido_generico(
            MensajeCliente::from_str("OP / 0").unwrap(),
            Respuesta::from(ErrorEnCalculadora::DivisionCero),
        );
    }

    fn test_ejecutar_mensaje_valido_generico(
        mensaje: MensajeCliente,
        respuesta_esperada: Respuesta,
    ) {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let msg = " ";
        let reader = BufReader::new(msg.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        let respuesta = cliente.ejecutar_mensaje(mensaje).unwrap();

        assert_eq!(respuesta, respuesta_esperada);
        assert!(!cliente.se_recibio_mensaje_get);
    }

    #[test]
    fn test08_ejecutar_mensaje_valido() {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let msg = " ";
        let reader = BufReader::new(msg.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        let mensaje = MensajeCliente::from_str("GET").unwrap();

        let respuesta = cliente.ejecutar_mensaje(mensaje).unwrap();

        assert_eq!(respuesta, Respuesta::Valor(0));
        assert!(cliente.se_recibio_mensaje_get);
    }

    #[test]
    fn test09_enviar_respuesta_cliente_valida() {
        test_enviar_respuesta_cliente_generica(Respuesta::OK, "OK\n".as_bytes());

        test_enviar_respuesta_cliente_generica(Respuesta::Valor(4), "VALUE 4\n".as_bytes());

        test_enviar_respuesta_cliente_generica(
            Respuesta::from(ErrorEnThread::FaltaCampo),
            "ERROR \"missing parameters\"\n".as_bytes(),
        );
    }

    fn test_enviar_respuesta_cliente_generica(respuesta: Respuesta, respuesta_esperada: &[u8]) {
        let counter_calculadora = Arc::new(Mutex::new(Calculadora::default()));

        let msg = " ";
        let reader = BufReader::new(msg.as_bytes());

        let writer = Vec::new();

        let buffer = String::new();

        let mut cliente = HandlerCliente::new(reader, writer, counter_calculadora, buffer);

        cliente.enviar_respuesta(respuesta).unwrap();

        assert_eq!(cliente.writer_stream, respuesta_esperada);
    }
}
