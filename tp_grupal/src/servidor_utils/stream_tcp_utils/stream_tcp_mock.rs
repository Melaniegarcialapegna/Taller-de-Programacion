use crate::encriptacion::SistemaDeEncriptacion;
use crate::protocolos::pca::mensaje::MensajePCA;
use crate::servidor_utils::stream_tcp_utils::error_stream_tcp::ErrorStreamTCP;
use crate::servidor_utils::stream_tcp_utils::stream_tcp::StreamTCP;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

//Trait para simular un StreamTCP en test
impl StreamTCP for MockStreamTCP {
    fn leer_mensaje(
        &mut self,
        buffer: &mut String,
        _encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<usize, ErrorStreamTCP> {
        let mut buf: [u8; 24000] = [0; 24000];
        match self.read(&mut buf) {
            Ok(n) => {
                if !buf.is_empty() {
                    buffer.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Ok(n)
            }
            Err(_) => Err(ErrorStreamTCP::ErrorRecibiendoMensaje),
        }
    }

    fn enviar_mensaje(
        &mut self,
        mensaje: MensajePCA,
        _encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<usize, ErrorStreamTCP> {
        let mensaje_string = String::from(mensaje);
        self.write(mensaje_string.as_bytes())
            .map_err(|_| ErrorStreamTCP::ErrorEnviandoMensaje)
    }

    fn clonar(&mut self) -> Result<Box<dyn StreamTCP>, ErrorStreamTCP> {
        Ok(Box::new(self.clone()))
    }
}

#[derive(Clone)]
pub struct MockStreamTCP {
    bytes_que_se_leeran: Vec<u8>,
    pub bytes_escritos: Arc<Mutex<Vec<u8>>>,
    posicion_lectura: usize,
}

impl MockStreamTCP {
    pub fn new(bytes_que_se_leeran: Vec<u8>) -> MockStreamTCP {
        MockStreamTCP {
            bytes_que_se_leeran,
            bytes_escritos: Arc::new(Mutex::new(vec![])),
            posicion_lectura: 0,
        }
    }
}

impl Write for MockStreamTCP {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = self
            .bytes_escritos
            .lock()
            .map_err(|_| std::io::Error::other("Error en el lock de escritura"))?;
        lock.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

//para que sea mas real deberia leer hasta\n !!
impl Read for MockStreamTCP {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let bytes_a_leer = &self.bytes_que_se_leeran[self.posicion_lectura..];
        let cantidad_a_leer = std::cmp::min(bytes_a_leer.len(), buffer.len());

        buffer[..cantidad_a_leer].copy_from_slice(&bytes_a_leer[..cantidad_a_leer]);

        self.posicion_lectura += cantidad_a_leer;

        Ok(cantidad_a_leer)
    }
}
