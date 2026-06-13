use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread,
    time::Duration,
};

use crate::sesion_rtp::jitter_buffer::{
    error::ErrorJitterBuffer, mensaje_jitter_buffer::MensajeJitterBuffer,
};

//Constantes
pub const TOLERANCIA: u32 = 3000;
pub const LIMITE_AUDIO_SIN_VARIAR: u32 = 8;

//El tipo [`JitterBuffer`] implementa un B tree el cual contiene los timestamps y el payload de las medias.
//El criterio de ordenamiento es segun timestamp
pub type JitterBuffer = Arc<Mutex<BTreeMap<u32, Vec<u8>>>>;

pub struct InformacionPaqueteRTP {
    pub timestamp: u32,
    pub payload: Vec<u8>,
}

pub enum Informacion {
    //Contiene el sender al decodificador de video y el arc mutex de reproductor_procesando_frame
    InformacionVideo(Sender<Vec<u8>>, Arc<Mutex<bool>>),
    //Contiene el sender al reproductor de audio
    InformacionAudio(Sender<Vec<i16>>),
}

//El trait [GestionJitterBuffer] es quien se encarga de crear los hilos correspondientes para el funcionamiento del jitter buffer.
//La funcion enviar payload se debe implementar, mientras que el resto tienen una implementacion general.
pub trait GestionJitterBuffer: Send + 'static {
    //Se encarga de crear los hilos que en conjunto gestionaran al jitter bufffer.
    fn iniciar_gestion_jitter_buffer(
        rx_rtp: Receiver<MensajeJitterBuffer>,
        tam_buffer: usize,
        informacion: Informacion,
        ultimo_timestamp_audio: Arc<Mutex<u32>>,
    ) {
        let jitter_buffer: JitterBuffer = Arc::new(Mutex::new(BTreeMap::new()));
        let ref_jitter_buffer = Arc::clone(&jitter_buffer);

        let ultimo_timestamp = Arc::new(Mutex::new(0));
        let ref_ultimo_timestamp = Arc::clone(&ultimo_timestamp);

        let llamada_finalizada = Arc::new(Mutex::new(false));
        let ref_llamada_finalizada = Arc::clone(&llamada_finalizada);
        //hilo que escuchara los paquetesRTP
        thread::spawn(move || {
            if Self::recibir_paquetes_rtp(
                ref_jitter_buffer,
                tam_buffer,
                rx_rtp,
                ref_ultimo_timestamp,
                ref_llamada_finalizada,
            )
            .is_err()
            {
                eprintln!("Error en hilo escucha jitter buffer");
            }
        });

        //hilo que enviara el payload al decodificador
        thread::spawn(move || {
            if Self::enviar_payload(
                jitter_buffer,
                ultimo_timestamp,
                llamada_finalizada,
                informacion,
                ultimo_timestamp_audio,
            )
            .is_err()
            {
                eprintln!("Error en hilo escucha jitter buffer");
            }
        });
    }

    //Recibe el payload y los timestamp de la media por medio de un [MensajeJitterBuffer].
    //Por este tipo de mensaje tambien se le notifica cuando debe morir el thread cuando la llamada finaliza.
    //Cuando llega [MensajeJitterBuffer::InformaconPaqueteRTP] se lo inserta en el jitter buffer solo si el timestamp no
    //es menor al del ultimo enviado. En caso de que lo sea, se lo descarta ya que es de un paquete viejo.
    fn recibir_paquetes_rtp(
        jitter_buffer: JitterBuffer,
        tam_buffer: usize,
        rx_rtp: Receiver<MensajeJitterBuffer>,
        ultimo_timestamp: Arc<Mutex<u32>>,
        llamada_finalizada: Arc<Mutex<bool>>,
    ) -> Result<(), ErrorJitterBuffer> {
        for mensaje in rx_rtp {
            match mensaje {
                MensajeJitterBuffer::MensajeFinalizacionLlamada => {
                    //le aviso al hilo que esta enviando paquetes que la llamada finalizo y salgo
                    {
                        let mut lock_llamada_finalizada = llamada_finalizada
                            .lock()
                            .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                        *lock_llamada_finalizada = true;
                    } //libero lock
                    return Ok(());
                }
                MensajeJitterBuffer::InformacionPaqueteRTP(info_paquete) => {
                    //si timestamp es menor a ultimo mostrado se descarta
                    {
                        let lock_ultimo_timestamp = ultimo_timestamp
                            .lock()
                            .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                        if *lock_ultimo_timestamp > info_paquete.timestamp {
                            continue;
                        }
                    }
                    //en caso contrario saco el de timestamp mas pequeño y agrego este
                    {
                        let mut lock_jitter_buffer = jitter_buffer
                            .lock()
                            .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;

                        //si jitterBuffer tiene lugar lo inserto
                        if lock_jitter_buffer.len() < tam_buffer {
                            lock_jitter_buffer.insert(info_paquete.timestamp, info_paquete.payload)
                        } else {
                            //en caso contrario saco el de timestamp mas pequeño y agrego este
                            lock_jitter_buffer.pop_first();
                            lock_jitter_buffer.insert(info_paquete.timestamp, info_paquete.payload)
                        }
                    }
                }
            };
        }
        Ok(())
    }

    //Se encarga de gestionar y enviar el payload por un channel a quien corresponda.
    fn enviar_payload(
        jitter_buffer: JitterBuffer,
        ultimo_timestamp: Arc<Mutex<u32>>,
        llamada_finalizada: Arc<Mutex<bool>>,
        informacion: Informacion,
        ultimo_timestamp_audio: Arc<Mutex<u32>>,
    ) -> Result<(), ErrorJitterBuffer>;
}

pub struct JitterBufferVideo;
impl GestionJitterBuffer for JitterBufferVideo {
    fn enviar_payload(
        jitter_buffer: JitterBuffer,
        ultimo_timestamp: Arc<Mutex<u32>>,
        llamada_finalizada: Arc<Mutex<bool>>,
        informacion: Informacion,
        ultimo_timestamp_audio: Arc<Mutex<u32>>,
    ) -> Result<(), ErrorJitterBuffer> {
        let (tx_decodificador, reproductor_procesando_frame) = match informacion {
            Informacion::InformacionVideo(sender, reproductor) => (sender, reproductor),
            _ => return Err(ErrorJitterBuffer::SinInformacionVideo),
        };

        let mut enviar_frame: bool;
        let mut contador: u32 = 0;
        let mut timestamp_anterior_audio: u32 = 0;
        loop {
            //si la llamada ya termino dejamos de enviar el payload al decodificar y matamos hilo
            {
                let lock_llamada_finalizada = llamada_finalizada
                    .lock()
                    .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                if *lock_llamada_finalizada {
                    return Ok(());
                }
            }
            //si el decodificador esta procesando otro frame volvemos a esperar 30 milisegundos
            {
                let procesando_frame = reproductor_procesando_frame
                    .lock()
                    .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                enviar_frame = !(*procesando_frame);
            }

            if enviar_frame {
                let mut lock_jitter_buffer = jitter_buffer
                    .lock()
                    .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;

                //hasta hallar un frame con un timestamp lo suficientemente cercano al timestamp del audio
                loop {
                    //elimino del jitter buffer y le envio al decodificador el mas reciente
                    //estaria bien descartarlo pq significaria que esta muy atrasado respecto al audio
                    if let Some((timestamp, payload)) = lock_jitter_buffer.pop_first() {
                        //verifico la cercania con el audio
                        {
                            let lock_ultimo_timestamp_audio = ultimo_timestamp_audio
                                .lock()
                                .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                            let timestamp_audio = *lock_ultimo_timestamp_audio;

                            if timestamp_audio == timestamp_anterior_audio {
                                contador += 1;
                            } else {
                                contador = 0;
                                timestamp_anterior_audio = timestamp_audio;
                            }

                            if contador <= LIMITE_AUDIO_SIN_VARIAR {
                                //si ya comenzo transmision audio o si se pauso (para evitar busy waits)
                                if timestamp + TOLERANCIA < timestamp_audio {
                                    //video atrasado
                                    continue;
                                }

                                if timestamp > TOLERANCIA + timestamp_audio {
                                    //dudo que pase pero -> video adelantado
                                    thread::sleep(Duration::from_millis(3));
                                    continue;
                                }
                            }
                        }
                        tx_decodificador
                            .send(payload)
                            .map_err(|_| ErrorJitterBuffer::EnviandoPorChannel)?;

                        {
                            let mut lock_ultimo_timestamp = ultimo_timestamp
                                .lock()
                                .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                            *lock_ultimo_timestamp = timestamp;
                        }
                    }
                    break;
                }
            }

            thread::sleep(Duration::from_millis(20));
        }
    }
}

pub struct JitterBufferAudio;
impl GestionJitterBuffer for JitterBufferAudio {
    fn enviar_payload(
        jitter_buffer: JitterBuffer,
        ultimo_timestamp: Arc<Mutex<u32>>,
        llamada_finalizada: Arc<Mutex<bool>>,
        informacion: Informacion,
        ultimo_timestamp_audio: Arc<Mutex<u32>>,
    ) -> Result<(), ErrorJitterBuffer> {
        let tx_reproductor_audio = match informacion {
            Informacion::InformacionAudio(sender) => sender,
            _ => return Err(ErrorJitterBuffer::SinInformacionAudio),
        };

        loop {
            //si la llamada ya termino dejamos de enviar el payload al decodificar y matamos hilo
            {
                let lock_llamada_finalizada = llamada_finalizada
                    .lock()
                    .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                if *lock_llamada_finalizada {
                    return Ok(());
                }
            }

            {
                let mut lock_jitter_buffer = jitter_buffer
                    .lock()
                    .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;

                //elimino del jitter buffer y envio el mas reciente
                if let Some((timestamp, mut payload)) = lock_jitter_buffer.pop_first() {
                    let mut audio = Vec::new();
                    let mut iterador_payload = payload.iter_mut();

                    while let Some(primer_byte) = iterador_payload.next()
                        && let Some(segundo_byte) = iterador_payload.next()
                    {
                        let muestra: i16 = i16::from_be_bytes([*primer_byte, *segundo_byte]);
                        audio.push(muestra);
                    }

                    tx_reproductor_audio
                        .send(audio)
                        .map_err(|_| ErrorJitterBuffer::EnviandoPorChannel)?;

                    {
                        let mut lock_ultimo_timestamp_audio = ultimo_timestamp_audio
                            .lock()
                            .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                        *lock_ultimo_timestamp_audio = timestamp;
                    }

                    {
                        let mut lock_ultimo_timestamp = ultimo_timestamp
                            .lock()
                            .map_err(|_| ErrorJitterBuffer::ObteniendoLock)?;
                        *lock_ultimo_timestamp = timestamp;
                    }
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}
