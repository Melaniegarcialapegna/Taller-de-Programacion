use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use crate::{
    comunicacion::{
        comunicador::{Comunicador, ErrorComunicador},
        comunicador_fake::ComunicadorFake,
    },
    protocolos::pca::mensaje::MensajePCA,
};

pub struct ComunicadorFakeYMock {
    comunicador_fake: Box<dyn Comunicador>,
    mutex_mensajes_enviados: Arc<Mutex<Vec<MensajePCA>>>,
}

impl ComunicadorFakeYMock {
    pub fn new(
        sender_mensajes: Sender<MensajePCA>,
        receiver_mensajes: Receiver<MensajePCA>,
        mutex_mensajes_enviados: Arc<Mutex<Vec<MensajePCA>>>,
    ) -> ComunicadorFakeYMock {
        let comunicador_fake = ComunicadorFake::new(sender_mensajes, receiver_mensajes);

        ComunicadorFakeYMock {
            comunicador_fake: Box::new(comunicador_fake),
            mutex_mensajes_enviados,
        }
    }

    pub fn con_componentes(
        comunicador_fake: Box<dyn Comunicador>,
        mutex_mensajes_enviados: Arc<Mutex<Vec<MensajePCA>>>,
    ) -> ComunicadorFakeYMock {
        ComunicadorFakeYMock {
            comunicador_fake,
            mutex_mensajes_enviados,
        }
    }
}

impl Comunicador for ComunicadorFakeYMock {
    fn crear_companiero(&mut self) -> Result<Box<dyn Comunicador>, ErrorComunicador> {
        let comunicador_fake = self.comunicador_fake.crear_companiero()?;

        let comunicador_fake_y_mock = Box::new(ComunicadorFakeYMock::con_componentes(
            comunicador_fake,
            Arc::clone(&self.mutex_mensajes_enviados),
        ));

        Ok(comunicador_fake_y_mock)
    }

    fn enviar_mensaje(&mut self, mensaje: &MensajePCA) -> Result<(), ErrorComunicador> {
        let mut mensajes_enviados = self
            .mutex_mensajes_enviados
            .lock()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

        mensajes_enviados.push(mensaje.clone());
        Ok(())
    }

    fn escuchar_mensaje(&mut self) -> Result<MensajePCA, ErrorComunicador> {
        self.comunicador_fake.escuchar_mensaje()
    }
}
