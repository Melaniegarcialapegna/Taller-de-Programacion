//! Módulo encargado de la creación y gestión de sockets UDP utilizados por RTP y RTCP.
//!
//! Este módulo abstrae la inicialización y duplicación de sockets mediante la estructura Sockets.
//!
//! Cada instancia de Sockets mantiene dos sockets UDP:
//! - RTP: Para transmisión principal de medios.
//! - RTCP: Para control y sincronización.
//!
//! Ambos pueden clonarse para usarse en distintos hilos o componentes a lo largo del ciclo de vida de la instancia de RTCPeerConnection a la que pertenecen.

use crate::logger::Logger;
use crate::sesion_rtp::socket_udp::SocketUDP;
use std::net::UdpSocket;

/// Contiene los sockets UDP que usamos tanto para RTP como para RTCP.
///
/// Permite crear, acceder, y clonar los sockets necesarios.
pub struct Sockets {
    rtp: Box<dyn SocketUDP>,
    rtcp: Box<dyn SocketUDP>,
    // para DTLS los necesito sin wrappers
    rtp_raw: UdpSocket,
    rtcp_raw: UdpSocket,
}

impl Sockets {
    /// Crea una nueva instancia de Sockets inicializando los sockets RTP y RTCP.
    ///
    /// El socket RTCP se crea automáticamente en el puerto siguiente al RTP.
    ///
    /// # Parámetros
    /// - `logger`: Referencia al logger para registrar eventos.
    /// - `addr`: Dirección IP o hostname donde enlazar los sockets.
    /// - `puerto_rtp`: Puerto base para RTP.
    ///
    /// # Errores
    /// Retorna `Err` si ocurre un error al crear alguno de los sockets UDP.
    pub fn new(logger: &Logger, addr: &str, puerto_rtp: u16) -> Result<Self, String> {
        let rtp_raw = UdpSocket::bind(format!("{}:{}", addr, puerto_rtp))
            .map_err(|e| format!("Error creando socket UDP RTP: {}", e))?;

        let rtcp_raw = UdpSocket::bind(format!("{}:{}", addr, puerto_rtp + 1))
            .map_err(|e| format!("Error creando socket UDP RTCP: {}", e))?;

        logger.info(
            &format!("Creando socket UDP en {}:{}", addr, puerto_rtp),
            "Sockets",
        );
        logger.info(
            &format!("Creando socket UDP en {}:{}", addr, puerto_rtp + 1),
            "Sockets",
        );

        let rtp = Self::crear_socket_udp(&rtp_raw, logger)?;
        let rtcp = Self::crear_socket_udp(&rtcp_raw, logger)?;

        Ok(Self {
            rtp,
            rtcp,
            rtp_raw,
            rtcp_raw,
        })
    }

    fn crear_socket_udp(
        raw_socket: &UdpSocket,
        logger: &Logger,
    ) -> Result<Box<dyn SocketUDP>, String> {
        logger.info("Clonando socket UDP (wrapper)", "Sockets");

        let cloned = raw_socket
            .try_clone()
            .map_err(|e| format!("No se pudo clonar socket UDP: {}", e))?;

        Ok(Box::new(cloned))
    }

    // getters
    pub fn clonar_socket_rtp(&mut self) -> Result<Box<dyn SocketUDP>, String> {
        self.get_rtp()
            .clonar()
            .map_err(|_| "Error clonando RTP".into())
    }

    pub fn clonar_socket_rtcp(&mut self) -> Result<Box<dyn SocketUDP>, String> {
        self.get_rtcp()
            .clonar()
            .map_err(|_| "Error clonando RTCP".into())
    }

    pub fn obtener_raw_rtp(&self) -> Result<UdpSocket, String> {
        self.rtp_raw
            .try_clone()
            .map_err(|_| "Error clonando raw RTP".into())
    }

    pub fn obtener_raw_rtcp(&self) -> Result<UdpSocket, String> {
        self.rtcp_raw
            .try_clone()
            .map_err(|_| "Error clonando raw RTCP".into())
    }

    // getters privadors que usamos sólo para los getters de arriba que sí son públicos y para quien los usa son básicamente un getter/handler del socket
    /// Devuelve una referencia mutable al socket RTP.
    fn get_rtp(&mut self) -> &mut dyn SocketUDP {
        &mut *self.rtp
    }

    /// Devuelve una referencia mutable al socket RTCP.
    fn get_rtcp(&mut self) -> &mut dyn SocketUDP {
        &mut *self.rtcp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sesion_rtp::socket_udp::{MockSocketUdp, SocketUDP};
    use std::sync::{Arc, Mutex};

    fn crear_mock_socket() -> Box<dyn SocketUDP> {
        let mut vector: Vec<Vec<u8>> = vec![];
        let v1 = vec![1_u8];
        vector.push(v1);

        let v2 = vec![2_u8];
        vector.push(v2);

        let v3 = vec![3_u8];
        vector.push(v3);

        let v4 = vec![4_u8];
        vector.push(v4);

        Box::new(MockSocketUdp {
            bytes_enviados: Arc::new(Mutex::new(Vec::new())),
            bytes_que_se_leeran: vector,
            posicion_lectura: 0,
        })
    }

    #[test]
    fn sockets_se_crean_correctamente() {
        let logger = Logger::dummy_logger();

        let sockets = Sockets::new(&logger, "127.0.0.1", 8000);
        assert!(sockets.is_ok(), "No se pudieron crear los sockets UDP");
    }

    #[test]
    fn crear_socket_udp_devuelve_error_si_no_puede_bindear() {
        let logger = Logger::dummy_logger();
        // debería fallar por intentar bindear a una dirección inválida
        // intenté testear con puertos ocupados pero si por equis razón el puerto que usamos acá está libre el test va a fallar
        let sockets = Sockets::new(&logger, "123.123.123.123", 1);
        assert!(
            sockets.is_err(),
            "Debería fallar al intentar crear socket en puerto reservado"
        );
    }

    #[test]
    fn clonar_sockets_funciona_con_mock() {
        // creamos manualmente los mocks en lugar de usar new() que bindea puertos reales
        let mut sockets = Sockets {
            rtp: crear_mock_socket(),
            rtcp: crear_mock_socket(),
            rtp_raw: UdpSocket::bind("127.0.0.1:8000").unwrap(),
            rtcp_raw: UdpSocket::bind("127.0.0.1:8001").unwrap(),
        };

        let clon_rtp = sockets.clonar_socket_rtp();
        let clon_rtcp = sockets.clonar_socket_rtcp();

        assert!(clon_rtp.is_ok());
        assert!(clon_rtcp.is_ok());
    }
}
