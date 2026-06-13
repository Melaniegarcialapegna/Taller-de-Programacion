use std::sync::{Arc, Mutex};

use crate::{
    creacion_llamada::{ConexionP2P, CreadorDeConexionP2P, ErrorCreadorDeConexion},
    sesion_rtp::socket_udp::MockSocketUdp,
};

#[derive(Default)]
pub struct CreadorDeConexionMock {
    se_genero_offer: bool,
    se_genero_answer: bool,
    se_recibio_answer: bool,
    se_inicio_conexion: bool,
    se_pidieron_sockets: bool,
}

impl CreadorDeConexionMock {
    pub fn se_genero_offer(&self) -> bool {
        self.se_genero_offer
    }

    pub fn se_genero_answer(&self) -> bool {
        self.se_genero_answer
    }

    pub fn se_recibio_answer(&self) -> bool {
        self.se_recibio_answer
    }

    pub fn se_inicio_conexion(&self) -> bool {
        self.se_inicio_conexion
    }
}

impl CreadorDeConexionP2P for Arc<Mutex<CreadorDeConexionMock>> {
    fn conectar(&mut self) -> Result<(), ErrorCreadorDeConexion> {
        let mut creador = self
            .lock()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        creador.se_inicio_conexion = true;
        Ok(())
    }

    fn generar_answer(&mut self, _offer: &str) -> Result<String, ErrorCreadorDeConexion> {
        let mut creador = self
            .lock()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        creador.se_genero_answer = true;
        Ok("".to_string())
    }

    fn generar_offer(&mut self) -> Result<String, ErrorCreadorDeConexion> {
        let mut creador = self
            .lock()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        creador.se_genero_offer = true;
        Ok("".to_string())
    }

    fn recibir_answer(&mut self, _answer: &str) -> Result<(), ErrorCreadorDeConexion> {
        let mut creador = self
            .lock()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        creador.se_recibio_answer = true;
        Ok(())
    }

    fn obtener_sockets(&mut self) -> Result<ConexionP2P, ErrorCreadorDeConexion> {
        let mut creador = self
            .lock()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;
        creador.se_pidieron_sockets = true;

        let socket_rtp = Box::new(MockSocketUdp::new(vec![], vec![]));
        let socket_rtcp = Box::new(MockSocketUdp::new(vec![], vec![]));

        let conexion = ConexionP2P::new(
            socket_rtp,
            socket_rtcp,
            "".to_string(),
            "".to_string(),
            None,
            None,
            None,
        );

        Ok(conexion)
    }
}
