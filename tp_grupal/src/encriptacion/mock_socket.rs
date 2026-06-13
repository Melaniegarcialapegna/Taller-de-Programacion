#[cfg(test)]
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

#[cfg(test)]
pub struct MockStreamTcp {
    bytes_que_se_leeran: Vec<Vec<u8>>,
    pub bytes_escritos: Arc<Mutex<Vec<Vec<u8>>>>,
}

#[cfg(test)]
impl MockStreamTcp {
    pub fn new(bytes_que_se_leeran: Vec<Vec<u8>>) -> MockStreamTcp {
        MockStreamTcp {
            bytes_que_se_leeran,
            bytes_escritos: Arc::new(Mutex::new(vec![])),
        }
    }
}

#[cfg(test)]
impl Write for MockStreamTcp {
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let mut mutex_bytes_escritos = self.bytes_escritos.lock().expect("Error en el mock");
        mutex_bytes_escritos.push(data.to_vec());

        Ok(data.len())
    }
}

#[cfg(test)]
impl Read for MockStreamTcp {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let mensaje_a_leer = self.bytes_que_se_leeran.remove(0);
        let bytes_a_leer = &mensaje_a_leer[..];

        let cantidad_a_leer = if bytes_a_leer.len() < buffer.len() {
            bytes_a_leer.len()
        } else {
            buffer.len()
        };

        buffer[..cantidad_a_leer].copy_from_slice(&bytes_a_leer[..cantidad_a_leer]);

        Ok(cantidad_a_leer)
    }
}
