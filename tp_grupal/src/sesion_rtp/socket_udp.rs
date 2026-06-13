use super::error::ErrorSocketUDP;
use std::fmt::Debug;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::{Arc, Mutex};

//Trait para poder Mockear SocketsUDP
pub trait SocketUDP: Send {
    fn local_addr(&self) -> Result<SocketAddr, ErrorSocketUDP>;
    //poner &SocketAddr en vez de &str
    fn enviar(&mut self, data: &[u8], receptor: &str) -> Result<usize, ErrorSocketUDP>;
    fn recibir(&mut self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), ErrorSocketUDP>;
    fn clonar(&mut self) -> Result<Box<dyn SocketUDP>, ErrorSocketUDP>;
    fn mutar_no_bloqueante(&mut self) -> Result<(), ErrorSocketUDP>;
    fn mutar_bloqueante(&mut self) -> Result<(), ErrorSocketUDP>;
}

impl Debug for dyn SocketUDP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SocketUDP")
    }
}

impl SocketUDP for UdpSocket {
    fn enviar(&mut self, data: &[u8], receptor: &str) -> Result<usize, ErrorSocketUDP> {
        self.send_to(data, receptor)
            .map_err(|_| ErrorSocketUDP::ErrorEnviandoDatos)
    }
    fn recibir(&mut self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), ErrorSocketUDP> {
        self.recv_from(buffer)
            .map_err(|_| ErrorSocketUDP::ErrorRecibiendoDatos)
    }

    fn clonar(&mut self) -> Result<Box<dyn SocketUDP>, ErrorSocketUDP> {
        self.try_clone()
            .map(|socket| Box::new(socket) as Box<dyn SocketUDP>) //dyn pq se determina en tiempo de ejecucion
            .map_err(|_| ErrorSocketUDP::ErrorClonarSocketUDP)
    }

    fn mutar_no_bloqueante(&mut self) -> Result<(), ErrorSocketUDP> {
        self.set_nonblocking(true)
            .map_err(|_| ErrorSocketUDP::ErrorEnviandoDatos)?;
        Ok(())
    }

    fn mutar_bloqueante(&mut self) -> Result<(), ErrorSocketUDP> {
        self.set_nonblocking(false)
            .map_err(|_| ErrorSocketUDP::ErrorEnviandoDatos)?;
        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, ErrorSocketUDP> {
        self.local_addr()
            .map_err(|_| ErrorSocketUDP::ErrorEnviandoDatos)
    }
}
pub struct MockSocketUdp {
    pub bytes_enviados: Arc<Mutex<Vec<u8>>>,
    pub bytes_que_se_leeran: Vec<Vec<u8>>,
    pub posicion_lectura: usize,
}

impl MockSocketUdp {
    pub fn new(bytes_enviar: Vec<u8>, bytes_leer: Vec<Vec<u8>>) -> MockSocketUdp {
        MockSocketUdp {
            bytes_enviados: Arc::new(Mutex::new(bytes_enviar)),
            bytes_que_se_leeran: bytes_leer,
            posicion_lectura: 0,
        }
    }
}

impl SocketUDP for MockSocketUdp {
    fn enviar(&mut self, data: &[u8], _receptor: &str) -> Result<usize, ErrorSocketUDP> {
        self.bytes_enviados
            .lock()
            .map_err(|_| ErrorSocketUDP::ErrorEnviandoDatos)?
            .extend_from_slice(data);

        Ok(data.len())
    }

    fn recibir(&mut self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), ErrorSocketUDP> {
        let datagrama = &self.bytes_que_se_leeran[self.posicion_lectura];
        self.posicion_lectura += 1;

        let cantidad_a_leer = datagrama.len();
        buffer[..cantidad_a_leer].copy_from_slice(datagrama);

        // Creo un socket_addr valido para devolver, es unicamente para cumplir
        // con el trait
        let socket_addr_str = String::from("127.0.0.1:3000");
        let mut socket_addr_iter = socket_addr_str
            .to_socket_addrs()
            .map_err(|_| ErrorSocketUDP::ErrorRecibiendoDatos)?;

        let socket_addr = socket_addr_iter
            .next()
            .ok_or(ErrorSocketUDP::ErrorRecibiendoDatos)?;

        Ok((cantidad_a_leer, socket_addr))
    }

    fn clonar(&mut self) -> Result<Box<dyn SocketUDP>, ErrorSocketUDP> {
        let mock_clonado = MockSocketUdp {
            bytes_enviados: self.bytes_enviados.clone(),
            bytes_que_se_leeran: self.bytes_que_se_leeran.clone(),
            posicion_lectura: self.posicion_lectura,
        };

        Ok(Box::new(mock_clonado))
    }

    //TODO
    fn mutar_no_bloqueante(&mut self) -> Result<(), ErrorSocketUDP> {
        Ok(())
    }

    fn mutar_bloqueante(&mut self) -> Result<(), ErrorSocketUDP> {
        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, ErrorSocketUDP> {
        let socket_addr_str = String::from("127.0.0.1:9090");
        let mut socket_addr_iter = socket_addr_str
            .to_socket_addrs()
            .map_err(|_| ErrorSocketUDP::ErrorRecibiendoDatos)?;

        socket_addr_iter
            .next()
            .ok_or(ErrorSocketUDP::ErrorRecibiendoDatos)
    }
}
