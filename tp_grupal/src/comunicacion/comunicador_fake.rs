use std::sync::mpsc::{self, Receiver, Sender};

use crate::{
    comunicacion::comunicador::{Comunicador, ErrorComunicador},
    protocolos::pca::mensaje::MensajePCA,
};

pub struct ComunicadorFake {
    sender_mensajes: Sender<MensajePCA>,
    receiver_mensajes: Receiver<MensajePCA>,
    senders_a_companieros: Vec<Sender<MensajePCA>>,
}

impl ComunicadorFake {
    pub fn new(
        sender_mensajes: Sender<MensajePCA>,
        receiver_mensajes: Receiver<MensajePCA>,
    ) -> ComunicadorFake {
        ComunicadorFake {
            sender_mensajes,
            receiver_mensajes,
            senders_a_companieros: vec![],
        }
    }

    fn agregar_sender_a_companiero(&mut self, sender: Sender<MensajePCA>) {
        self.senders_a_companieros.push(sender);
    }
}

impl Comunicador for ComunicadorFake {
    fn escuchar_mensaje(&mut self) -> Result<MensajePCA, ErrorComunicador> {
        let mensaje = self
            .receiver_mensajes
            .recv()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

        for sender_companiero in &mut self.senders_a_companieros {
            sender_companiero
                .send(mensaje.clone())
                .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
        }

        Ok(mensaje)
    }

    fn crear_companiero(&mut self) -> Result<Box<dyn Comunicador>, ErrorComunicador> {
        // Agrego un sender desde el comunicador actual al companiero
        let (sender_a_companiero, receiver_companiero) = mpsc::channel();
        self.senders_a_companieros.push(sender_a_companiero.clone());

        // Agrego un sender desde el companiero al comunicador actual
        let mut companiero = ComunicadorFake::new(sender_a_companiero, receiver_companiero);
        companiero.agregar_sender_a_companiero(self.sender_mensajes.clone());

        Ok(Box::new(companiero))
    }

    fn enviar_mensaje(&mut self, _mensaje: &MensajePCA) -> Result<(), ErrorComunicador> {
        // Esta funcionalidad no esta implementada para este comunicador
        Ok(())
    }
}
