//! # Microfono Default - Capturar audio del dispositivo por defecto
//!
//! Contiene [MicrofonoDefault], que sera un "objeto" capaz de capturar el audio interno
//! de la computadora (de su dispositivo por defecto). Para lograr esto, el microfono tendra dos threads internos:
//!
//! - Un hilo creado por CPAL al ejecutar [Stream::play]. Ese hilo sera administrado
//!   por el microfono usando [Stream], y por medio de esa estructura podra solicitar que se mutee o desmutee
//!   el microfono.
//!
//! - Un hilo de retransmision, con el que el microfono se comunicara mediante enviando [InstruccionARetransmisor].
//!   Este hilo sera el encargado de retransmitir el audio capturado por el hilo de CPAL. Ese audio tambien se recibira
//!   por medio de instrucciones de tipo [InstruccionARetransmisor]. Ademas, permitira que se cambie el receptor del audio,
//!   que sera a quien se retransmita el audio recibido por el hilo de CPAL. La existencia de este hilo se justifica en que
//!   el audio capturado de CPAL se retransmite ejecutando un *callback*, lo que dificulta que se modifique el [Sender] mediante el cual
//!   debe retransmitir el audio.

use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use cpal::{
    SizedSample, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::llamada::microfono::{ErrorMicrofono, Microfono};

pub enum InstruccionARetransmisor<T: SizedSample + Send + 'static> {
    RetransmitirAudio(Vec<T>),
    CambiarReceptor(Sender<Vec<T>>),
    BorrarReceptor,
}

/// Representa un microfono que capturara audio del dispositivo por defecto.
pub struct MicrofonoDefault<T: SizedSample + Send + 'static> {
    stream_audio: Stream,
    sender_instrucciones: Sender<InstruccionARetransmisor<T>>,
}

impl<T: SizedSample + Send + 'static> Microfono<T> for MicrofonoDefault<T> {
    fn mutear(&mut self) -> Result<(), ErrorMicrofono> {
        self.stream_audio
            .pause()
            .map_err(|e| ErrorMicrofono::ErrorInterno(format!("{e}")))?;

        Ok(())
    }

    fn desmutear(&mut self) -> Result<(), ErrorMicrofono> {
        self.stream_audio
            .play()
            .map_err(|e| ErrorMicrofono::ErrorInterno(format!("{e}")))?;

        Ok(())
    }

    fn cambiar_receptor(
        &mut self,
        sender_a_receptor: Sender<Vec<T>>,
    ) -> Result<(), ErrorMicrofono> {
        self.sender_instrucciones
            .send(InstruccionARetransmisor::CambiarReceptor(sender_a_receptor))
            .map_err(|_| {
                ErrorMicrofono::ErrorInterno("Fallo al cambiar receptor de audio".to_string())
            })?;

        Ok(())
    }

    fn borrar_receptor(&mut self) -> Result<(), ErrorMicrofono> {
        self.sender_instrucciones
            .send(InstruccionARetransmisor::BorrarReceptor)
            .map_err(|_| {
                ErrorMicrofono::ErrorInterno("Fallo al cambiar receptor de audio".to_string())
            })?;

        Ok(())
    }
}

impl<T: SizedSample + Send + 'static> MicrofonoDefault<T> {
    pub fn new() -> Result<MicrofonoDefault<T>, ErrorMicrofono> {
        let (sender_instrucciones, receiver_instrucciones) = mpsc::channel();

        let stream = Self::obtener_stream_audio(sender_instrucciones.clone())?;

        thread::spawn(move || {
            Self::retransmitir_audio(receiver_instrucciones);
        });

        Ok(MicrofonoDefault {
            stream_audio: stream,
            sender_instrucciones,
        })
    }

    fn retransmitir_audio(receiver_audio: Receiver<InstruccionARetransmisor<T>>) {
        if let Err(e) = Self::_retransmitir_audio(receiver_audio) {
            eprintln!("Error en retransmisor: {e}");
        };
    }

    fn _retransmitir_audio(
        receiver_audio: Receiver<InstruccionARetransmisor<T>>,
    ) -> Result<(), ErrorMicrofono> {
        let mut receptor = None;

        for instruccion in receiver_audio {
            match instruccion {
                InstruccionARetransmisor::CambiarReceptor(nuevo_receptor) => {
                    receptor = Some(nuevo_receptor)
                }
                InstruccionARetransmisor::RetransmitirAudio(audio) => {
                    if let Some(sender) = &receptor {
                        sender.send(audio).map_err(|e| {
                            ErrorMicrofono::ErrorInterno(format!(
                                "Fallo al retransmitir audio: {e}"
                            ))
                        })?
                    }
                }
                InstruccionARetransmisor::BorrarReceptor => receptor = None,
            }
        }

        Ok(())
    }

    fn obtener_stream_audio(
        sender_audio: Sender<InstruccionARetransmisor<T>>,
    ) -> Result<Stream, ErrorMicrofono> {
        // Obtengo host default para el audio
        let host = cpal::default_host();

        // Obtengo dispositivo default
        let device = host
            .default_input_device()
            .ok_or(ErrorMicrofono::ErrorInterno(
                "No hay microfonos disponibles".to_string(),
            ))?;

        // Agarro config default
        let configs_posibles = device.default_input_config().map_err(|_| {
            ErrorMicrofono::ErrorInterno(
                "No hay configuraciones de microfono disponibles".to_string(),
            )
        })?;
        let mut config = configs_posibles.config();
        config.buffer_size = cpal::BufferSize::Default;

        // Creo stream y le digo que envie todos los datos por el sender de audio
        let stream = device
            .build_input_stream::<T, _, _>(
                &config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    Self::transmitir_audio(&sender_audio, data);
                },
                |_| {
                    eprintln!("Error interno en microfono");
                },
                None,
            )
            .map_err(|_| {
                ErrorMicrofono::ErrorInterno("No se pudo crear el stream de audio".to_string())
            })?;

        Ok(stream)
    }

    fn transmitir_audio(sender_audio: &Sender<InstruccionARetransmisor<T>>, data: &[T]) {
        let instruccion = InstruccionARetransmisor::RetransmitirAudio(data.to_owned());
        let _ = sender_audio.send(instruccion);
    }
}
