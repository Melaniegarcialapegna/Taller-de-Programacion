//! ComunicadorTCP - Comunicador encriptado para hablar con el servidor
//!
//! [ComunicadorTCP] representa un [Comunicador] que habla con el servidor mediante un socket TCP encriptado. Este es el [Comunicador] que
//! debera ser usado por Aplicacion si se desea usar un socket TCP.
//!
//! **Importante**: Lo mas probable es que se requiera que muchos objetos se comuniquen con el servidor. Para lograrlo, todos los objetos que cumplan el trait [Comunicador]
//! tienen el metodo [Comunicador::crear_companiero]. Todos los mensajes recibidos del servidor llegaran a todos los comunicadores.

use std::{
    io::Write,
    net::{TcpStream, ToSocketAddrs},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{
    comunicacion::comunicador::{Comunicador, ErrorComunicador},
    encriptacion::SistemaDeEncriptacion,
    protocolos::pca::mensaje::MensajePCA,
};

pub struct ComunicadorTCP {
    sender_a_hilo_escritura: Sender<MensajePCA>,
    sender_a_hilo_lectura: Sender<MensajeComunicadorTCP>,
    receiver_de_hilo_lectura: Receiver<MensajeComunicadorTCP>,
}

enum MensajeComunicadorTCP {
    Recibido(MensajePCA),
    AgregarSuscriptor(Sender<MensajeComunicadorTCP>),
}

impl ComunicadorTCP {
    pub fn crear_comunicador(direccion_str: &str) -> Result<ComunicadorTCP, ErrorComunicador> {
        let (stream_escritura, encriptador_escritura) = Self::crear_conexion(direccion_str)?;

        // Creo una copia del stream y el encriptador para el hilo de lectura
        let stream_lectura = stream_escritura
            .try_clone()
            .map_err(|e| ErrorComunicador::ErrorDeConexion(format!("{e}")))?;

        let encriptador_lectura = encriptador_escritura.clone();

        let (sender_a_hilo_lectura, receiver_de_hilo_lectura) =
            Self::crear_thread_lectura(stream_lectura, encriptador_lectura)?;
        let sender_a_hilo_escritura =
            Self::crear_thread_escritura(stream_escritura, encriptador_escritura);

        Ok(ComunicadorTCP {
            sender_a_hilo_lectura,
            sender_a_hilo_escritura,
            receiver_de_hilo_lectura,
        })
    }

    fn enviar_mensajes(
        stream: TcpStream,
        encriptador: SistemaDeEncriptacion,
        receiver_desde_comunicador: Receiver<MensajePCA>,
    ) {
        if let Err(error) = Self::_enviar_mensajes(stream, encriptador, receiver_desde_comunicador)
        {
            dbg!(error);
        }
    }

    fn escuchar_mensajes(
        stream: TcpStream,
        encriptador: SistemaDeEncriptacion,
        receiver_comunicador: Receiver<MensajeComunicadorTCP>,
    ) {
        if let Err(error) = Self::_escuchar_mensajes(stream, encriptador, receiver_comunicador) {
            dbg!(error);
        }
    }

    fn _enviar_mensajes(
        mut stream: TcpStream,
        mut encriptador: SistemaDeEncriptacion,
        receiver_desde_comunicador: Receiver<MensajePCA>,
    ) -> Result<(), ErrorComunicador> {
        for mensaje in receiver_desde_comunicador {
            let bytes_a_enviar = encriptador
                .encriptar_mensaje(&String::from(mensaje))
                .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

            stream
                .write(&bytes_a_enviar)
                .map_err(|e| ErrorComunicador::ErrorDeConexion(format!("{e}")))?;
        }

        Ok(())
    }

    fn _escuchar_mensajes(
        mut stream: TcpStream,
        mut encriptador: SistemaDeEncriptacion,
        receiver_comunicador: Receiver<MensajeComunicadorTCP>,
    ) -> Result<(), ErrorComunicador> {
        let mut suscriptores = vec![];
        loop {
            let mensaje = encriptador
                .leer_desencriptando_mensaje(&mut stream)
                .map_err(|_| ErrorComunicador::ErrorDeConexion("Error encriptando".to_string()))?;

            let mensaje_pca = MensajePCA::try_from(&mensaje[..])
                .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

            Self::actualizar_suscriptores(&receiver_comunicador, &mut suscriptores)?;

            for sender in &suscriptores {
                sender
                    .send(MensajeComunicadorTCP::Recibido(mensaje_pca.clone()))
                    .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
            }
        }
    }

    fn crear_conexion(
        direccion_str: &str,
    ) -> Result<(TcpStream, SistemaDeEncriptacion), ErrorComunicador> {
        let mut direcciones_socket = direccion_str
            .to_socket_addrs()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
        let direccion_socket =
            direcciones_socket
                .next()
                .ok_or(ErrorComunicador::ErrorDeConexion(
                    "Fallo obteniendo direccion del socket".to_string(),
                ))?;
        let mut stream = TcpStream::connect(direccion_socket)
            .map_err(|e| ErrorComunicador::ErrorDeConexion(format!("{e}")))?;
        let encriptador = SistemaDeEncriptacion::encriptar_conexion(&mut stream).map_err(|_| {
            ErrorComunicador::ErrorDeConexion("Fallo encriptando conexion".to_string())
        })?;
        Ok((stream, encriptador))
    }

    fn crear_thread_lectura(
        stream_lectura: TcpStream,
        encriptador_lectura: SistemaDeEncriptacion,
    ) -> Result<
        (
            Sender<MensajeComunicadorTCP>,
            Receiver<MensajeComunicadorTCP>,
        ),
        ErrorComunicador,
    > {
        let (sender_a_hilo_lectura, receiver_hilo_lectura) = mpsc::channel();
        thread::spawn(move || {
            Self::escuchar_mensajes(stream_lectura, encriptador_lectura, receiver_hilo_lectura);
        });
        let (sender_a_comunicador, receiver_de_hilo_lectura) = mpsc::channel();
        sender_a_hilo_lectura
            .send(MensajeComunicadorTCP::AgregarSuscriptor(
                sender_a_comunicador,
            ))
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;
        Ok((sender_a_hilo_lectura, receiver_de_hilo_lectura))
    }

    fn actualizar_suscriptores(
        receiver_comunicador: &Receiver<MensajeComunicadorTCP>,
        suscriptores: &mut Vec<Sender<MensajeComunicadorTCP>>,
    ) -> Result<(), ErrorComunicador> {
        let mut resultado_mensaje_comunicador = receiver_comunicador.try_recv();
        while resultado_mensaje_comunicador.is_ok() {
            let mensaje_comunicador = resultado_mensaje_comunicador
                .map_err(|e| ErrorComunicador::ErrorDeConexion(format!("{e}")))?;

            if let MensajeComunicadorTCP::AgregarSuscriptor(sender) = mensaje_comunicador {
                suscriptores.push(sender);
            }

            resultado_mensaje_comunicador = receiver_comunicador.try_recv();
        }
        Ok(())
    }

    fn crear_thread_escritura(
        stream_escritura: TcpStream,
        encriptador_escritura: SistemaDeEncriptacion,
    ) -> Sender<MensajePCA> {
        let (sender_a_hilo_escritura, receiver_desde_comunicador) = mpsc::channel();
        thread::spawn(move || {
            Self::enviar_mensajes(
                stream_escritura,
                encriptador_escritura,
                receiver_desde_comunicador,
            );
        });
        sender_a_hilo_escritura
    }

    fn limpiar_channel_lectura(&mut self) {
        let mut mensaje_leido = self.receiver_de_hilo_lectura.try_recv();
        while mensaje_leido.is_ok() {
            mensaje_leido = self.receiver_de_hilo_lectura.try_recv();
        }
    }
}

impl Comunicador for ComunicadorTCP {
    fn enviar_mensaje(&mut self, mensaje: &MensajePCA) -> Result<(), ErrorComunicador> {
        self.sender_a_hilo_escritura
            .send(mensaje.clone())
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

        Ok(())
    }

    fn escuchar_mensaje(&mut self) -> Result<MensajePCA, ErrorComunicador> {
        self.limpiar_channel_lectura();

        let mensaje_leido = self
            .receiver_de_hilo_lectura
            .recv()
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

        let resultado;
        if let MensajeComunicadorTCP::Recibido(mensaje) = mensaje_leido {
            resultado = Ok(mensaje);
        } else {
            resultado = Err(ErrorComunicador::ErrorEnElComunicador);
        }

        resultado
    }
    fn crear_companiero(&mut self) -> Result<Box<dyn Comunicador>, ErrorComunicador> {
        let (sender_a_comunicador, receiver_de_hilo_lectura) = mpsc::channel();
        self.sender_a_hilo_lectura
            .send(MensajeComunicadorTCP::AgregarSuscriptor(
                sender_a_comunicador,
            ))
            .map_err(|_| ErrorComunicador::ErrorEnElComunicador)?;

        Ok(Box::new(ComunicadorTCP {
            sender_a_hilo_escritura: self.sender_a_hilo_escritura.clone(),
            sender_a_hilo_lectura: self.sender_a_hilo_lectura.clone(),
            receiver_de_hilo_lectura,
        }))
    }
}
