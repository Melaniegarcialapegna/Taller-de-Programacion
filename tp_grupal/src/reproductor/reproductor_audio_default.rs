use std::sync::mpsc::Receiver;

use cpal::{
    OutputCallbackInfo, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::reproductor::reproductor_audio::{ErrorReproductorAudio, ReproductorAudio};

#[allow(dead_code)]
pub struct ReproductorAudioDefault {
    stream_audio: Stream,
}

impl ReproductorAudio for ReproductorAudioDefault {
    fn iniciar_reproduccion(
        receiver_audio: Receiver<Vec<i16>>,
    ) -> Result<Box<dyn ReproductorAudio>, ErrorReproductorAudio> {
        let host = cpal::default_host();
        let dispositivo =
            host.default_output_device()
                .ok_or(ErrorReproductorAudio::ErrorCreandoReproductor(
                    "No hay dispositivo de salida".to_string(),
                ))?;
        let configuraciones = dispositivo.default_output_config().map_err(|_| {
            ErrorReproductorAudio::ErrorCreandoReproductor(
                "No hay configuraciones de salida".to_string(),
            )
        })?;
        let mut stream_configs = configuraciones.config();
        stream_configs.buffer_size = cpal::BufferSize::Default;

        let stream = dispositivo
            .build_output_stream(
                &stream_configs,
                move |data: &mut [i16], _: &OutputCallbackInfo| {
                    let audio_recibido = receiver_audio.try_recv();

                    if let Ok(audio) = audio_recibido {
                        data.copy_from_slice(&audio);
                    }
                },
                |_| {},
                None,
            )
            .map_err(|_| {
                ErrorReproductorAudio::ErrorCreandoReproductor(
                    "Fallo al crear stream de salida".to_string(),
                )
            })?;

        stream.pause().map_err(|_| {
            ErrorReproductorAudio::ErrorCreandoReproductor(
                "Fallo al pausar inicialmente el reproductor".to_string(),
            )
        })?;

        Ok(Box::new(ReproductorAudioDefault {
            stream_audio: stream,
        }))
    }

    fn pausar(&mut self) -> Result<(), ErrorReproductorAudio> {
        self.stream_audio
            .pause()
            .map_err(|e| ErrorReproductorAudio::ErrorInterno(format!("Fallo al pausar audio: {e}")))
    }

    fn despausar(&mut self) -> Result<(), ErrorReproductorAudio> {
        self.stream_audio.play().map_err(|e| {
            ErrorReproductorAudio::ErrorInterno(format!("Fallo al despausar audio: {e}"))
        })
    }
}
