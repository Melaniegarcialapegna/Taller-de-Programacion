use openh264::OpenH264API;
use openh264::encoder::{BitRate, Encoder, EncoderConfig};
use openh264::formats::{RgbSliceU8, YUVBuffer};

use super::error::ErrorComunicacionRTP;
use super::sesion::{EstadisticasReceiver, EstadisticasSender};
use super::socket_udp::SocketUDP;
use crate::logger::Logger;
use crate::protocolos::rtp::paquete::PaqueteRTP;
use crate::seguridad::srtp::errores::ErrorSRTP;
use crate::seguridad::srtp::srtp_contexto::SRTPContexto;
use crate::sesion_rtp::jitter_buffer::gestion_jitter_buffer::{
    GestionJitterBuffer, Informacion, InformacionPaqueteRTP, JitterBufferAudio, JitterBufferVideo,
};
use crate::sesion_rtp::jitter_buffer::mensaje_jitter_buffer::MensajeJitterBuffer;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Frame {
    pub bytes: Vec<u8>,
    pub anchura: usize,
    pub altura: usize,
    pub es_frame_finalizacion: bool,
}

pub struct ComunicadoresConIO {
    pub receiver_frames_camara: Receiver<Frame>,
    pub sender_frames_a_reproductor: Sender<Vec<u8>>,
    pub receiver_audio: Receiver<Vec<i16>>,
    pub sender_a_reproductor_audio: Sender<Vec<i16>>,
}

impl ComunicadoresConIO {
    pub fn new(
        receiver_frames_camara: Receiver<Frame>,
        sender_frames_a_reproductor: Sender<Vec<u8>>,
        receiver_audio: Receiver<Vec<i16>>,
        sender_a_reproductor_audio: Sender<Vec<i16>>,
    ) -> ComunicadoresConIO {
        ComunicadoresConIO {
            receiver_frames_camara,
            sender_frames_a_reproductor,
            receiver_audio,
            sender_a_reproductor_audio,
        }
    }
}

impl Frame {
    pub fn new(bytes: Vec<u8>, anchura: usize, altura: usize) -> Frame {
        Frame {
            bytes,
            anchura,
            altura,
            es_frame_finalizacion: false,
        }
    }

    pub fn frame_finalizacion() -> Frame {
        Frame {
            bytes: vec![0, 0, 0],
            anchura: 1,
            altura: 1,
            es_frame_finalizacion: true,
        }
    }
}

pub struct RtpIo {
    pub socket: Box<dyn SocketUDP>,
    pub srtp_rx: Receiver<Vec<u8>>, // este parametro nuevo viene del demux, es x donde se reciben los paquetes srtp y reemplaza al socket q teniamos antes
}

pub type UltimoTimestampAudio = Arc<Mutex<u32>>;

//Constantes
///Version valida de un paquete RTP
const VERSION_PAQUETE_RTP: u8 = 2;
/// Tipo de payload dinamico para video
const TIPO_PAYLOAD_VIDEO: u8 = 96;
/// Tipo de payload dinamico para audio
const TIPO_PAYLOAD_AUDIO: u8 = 11;

type ContextosSRTP = (SRTPContexto, SRTPContexto);

pub type Estadisticas = (
    Arc<Mutex<EstadisticasSender>>, //Estadisticas como sender de video
    Arc<Mutex<EstadisticasSender>>, // Estadisticas como sender de audio
    Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
);
// En este modulo se establece la comunicacion RTP.
// Establece una comunicacion entre dos peers por medio de un socketUdp

//  -- Campos
//   -socket : implementa [`SocketUDP`], es el medio por el cual los peers se transmiten los [`PaquetesRTP`] en formato de bytes.
//   -receiver_camara : extremo de un channel que recibe los frames decodificados de parte de la camara, este payload es el que formara
//   parte del [`PaqueteRTP`] que luego se transmitira al otro peer.
//   -sender_reproductor : extremo de un channel que envia los bytes de payload que se escucharon de un peer a un decodificador que los estara
//   escuchando para luego transmitirlos por pantalla.
//   -estadisticas_sender : estructura compartida tambien por la ComunicacionRTCP, se mantienen actualizada la informacion sobre el estado
//   la comunicacion RTP.
//   -estadisticas_receiver : estructura compartida tambien por la ComunicacionRTCP, por cada medio con el que se interactua, el cual es identificado
//   por su ssrc se tiene un diccionario en el cual se mantiene su estado.
//   -srtp_rx : extremo de un channel que recibe los paquetes SRTP del demux.

// Observacion : hay test unitarios que se encuentran comentados, para descomentarlos se deben descomentar las lineas indicadas!
// Es la unica manera de testear de manera unitaria.

//  Se establece una nueva comunicacionRTP
//  Se crea un nuevo hilo el cual estara escuchando los datagramas del otro peer por medio de un SocketUDP.
//  Al mismo tiempo se le estaran enviando por el mismo medio los datagramas de nuestro video.

//  En caso de Error se retorna un [`ErrorComunicacionRTP`].
pub fn iniciar_comunicacion_rtp(
    logger: Logger,
    rtp_io: RtpIo,
    direccion_receptor: &str,
    puntas_channels: ComunicadoresConIO,
    estadisticas: Estadisticas,
    reproductor_procesando_frame: Arc<Mutex<bool>>,
    contextos_srtp: ContextosSRTP,
) -> Result<(), ErrorComunicacionRTP> {
    let (contexto_srtp_tx, contexto_srtp_rx) = contextos_srtp;
    let (mut socket, srtp_rx) = (rtp_io.socket, rtp_io.srtp_rx);

    let (estadisticas_sender_video, estadisticas_sender_audio, estadisticas_receiver) =
        estadisticas;
    let referencia_estadisticas_receiver = Arc::clone(&estadisticas_receiver);
    let clon_referencia_estadisticas_receiver = Arc::clone(&referencia_estadisticas_receiver);

    let clon_logger = logger.clone();

    //Se crea thread que estara encargado de recibir paquetesRTP en formato de bytes, actualizar
    // estadisticas sobre el emisor de este paquete y enviar la informacion que este contiene al decodificador por
    // medio de un channel.
    let clon_contexto_srtp_rx = contexto_srtp_rx.clone();
    let sender_frames = puntas_channels.sender_frames_a_reproductor.clone();
    let sender_audio = puntas_channels.sender_a_reproductor_audio.clone();
    let handle = thread::spawn(move || {
        logger.info("Iniciando hilo escucha paquetes RTP", "Comunicacion RTP");
        if manejo_recepcion_datagramas(
            srtp_rx,
            sender_frames,
            sender_audio,
            clon_referencia_estadisticas_receiver,
            reproductor_procesando_frame,
            clon_contexto_srtp_rx,
        )
        .is_err()
        {
            logger.info("Cerrando hilo escucha paquetes RTP", "Comunicacion RTP");
        }
        logger.info("Finalizando hilo escucha paquetes RTP", "Comunicacion RTP");
    });

    // Se crean hilos de envio y recepcion de audio
    let direccion_receptor_owned = direccion_receptor.to_owned();

    let clon_socket_para_audio = socket
        .clonar()
        .map_err(|_| ErrorComunicacionRTP::ErrorIniciandoConexion)?;
    let _ = manejo_envio_datagramas_audio(
        //todo agarrar handle y esperarlo
        clon_socket_para_audio,
        contexto_srtp_tx.clone(),
        puntas_channels.receiver_audio,
        estadisticas_sender_audio,
        direccion_receptor_owned,
    );

    clon_logger.info("Iniciando hilo envio paquetes RTP", "Comunicacion RTP");
    //El hilo principal se encargara de escuchar por medio de un channel la informacion que la camara le envie
    //y enviara esta informacion a un destinatario en forma de paquete
    let resultado_hilo_envio = manejo_envio_datagramas_video(
        socket,
        direccion_receptor,
        puntas_channels.receiver_frames_camara,
        Arc::clone(&estadisticas_sender_video),
        estadisticas_receiver,
        contexto_srtp_tx,
    );

    if let Err(error) = resultado_hilo_envio {
        // Si el error es al escuchar de un channel, es porque
        if !matches!(error, ErrorComunicacionRTP::ErrorEscucharChannel) {
            return Err(error);
        }
    }

    //Se espera a que el hilo que esta escuchando al otro peer tmb finalice
    let _ = handle.join();
    // let _ = handle_envio_audio.join();
    clon_logger.info("Finalizando hilo envio paquetes RTP", "Comunicacion RTP");
    Ok(())
}

fn manejo_envio_datagramas_audio(
    socket: Box<dyn SocketUDP + 'static>,
    contexto_srtp_tx: SRTPContexto,
    receiver_audio: Receiver<Vec<i16>>,
    estadisticas_sender_audio: Arc<Mutex<EstadisticasSender>>,
    direccion_receptor: String,
) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Err(error) = _manejo_envio_datagramas_audio(
            socket,
            contexto_srtp_tx,
            receiver_audio,
            estadisticas_sender_audio,
            direccion_receptor,
        ) {
            let error_str = String::from(error);
            eprintln!("Error enviando audio: {error_str}")
        };
    })
}

fn _manejo_envio_datagramas_audio(
    mut socket: Box<dyn SocketUDP + 'static>,
    mut contexto_srtp_tx: SRTPContexto,
    receiver_audio: Receiver<Vec<i16>>,
    estadisticas_sender_audio: Arc<Mutex<EstadisticasSender>>,
    direccion_receptor: String,
) -> Result<(), ErrorComunicacionRTP> {
    for audio in receiver_audio {
        let audio_a_enviar = obtener_bytes_audio_a_enviar(audio);

        let paquete = obtener_paquete_a_enviar(&estadisticas_sender_audio, audio_a_enviar)?;

        let estadisticas_audio = estadisticas_sender_audio
            .lock()
            .map_err(|_| ErrorComunicacionRTP::ErrorObtenerLock)?;
        let ssrc = estadisticas_audio.ssrc;

        let mut bytes_paquete = Vec::from(&paquete);
        contexto_srtp_tx
            .proteger_y_firmar_rtp(ssrc, &mut bytes_paquete)
            .map_err(|_| ErrorComunicacionRTP::ErrorSRTPProtegiendo)?;

        socket
            .enviar(&bytes_paquete, &direccion_receptor)
            .map_err(|_| ErrorComunicacionRTP::ErrorEnviandoAudio)?;
    }
    Ok(())
}

fn obtener_paquete_a_enviar(
    estadisticas_sender_audio: &Arc<Mutex<EstadisticasSender>>,
    audio_a_enviar: Vec<u8>,
) -> Result<PaqueteRTP, ErrorComunicacionRTP> {
    let (
        ssrc,
        mut cantidad_paquetes_enviados,
        mut cantidad_bytes_enviados,
        mut ultimo_numero_secuencia,
    ) = obtener_datos_nuevo_paquete(estadisticas_sender_audio)
        .map_err(|_| ErrorComunicacionRTP::ErrorEnviandoAudio)?;

    //Se establece el timestamp para el paqueteRTP
    let timestamp = obtener_timestamp_actual();

    let paquete = PaqueteRTP {
        version: VERSION_PAQUETE_RTP,
        padding: 0,
        extension: 0,
        conteo_csrc: 0,
        marcador: 0,
        tipo_payload: TIPO_PAYLOAD_AUDIO,
        numero_de_secuencia: ultimo_numero_secuencia,
        padding_bytes: 0,
        ssrc,
        timestamp,
        lista_csrc: Vec::new(),
        payload: audio_a_enviar,
    };

    actualizar_estadisticas_sender(
        &mut cantidad_paquetes_enviados,
        &mut cantidad_bytes_enviados,
        &mut ultimo_numero_secuencia,
        Vec::from(&paquete).len() as u32,
        timestamp,
        estadisticas_sender_audio,
    )?;

    Ok(paquete)
}

fn obtener_timestamp_actual() -> u32 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(tiempo) => tiempo.as_millis() as u32,
        Err(_) => 0,
    }
}

fn obtener_bytes_audio_a_enviar(audio: Vec<i16>) -> Vec<u8> {
    let mut audio_a_enviar = Vec::new();

    for pieza_audio in audio {
        let bytes_pieza: [u8; 2] = pieza_audio.to_be_bytes();
        audio_a_enviar.push(bytes_pieza[0]);
        audio_a_enviar.push(bytes_pieza[1]);
    }
    audio_a_enviar
}

/// Se encarga de manejar la recepcion de datagramas que recibe de la camara y enviar esta informacion al otro peer.
///
/// Flujo:
/// Se decodificara la imagen de la camara y por medio de un channel se nos envia el payload con la informacion de esta.
/// Esta informacion la estaremos escuchando por medio del receiver_camara.
/// Con este payload se generara el [`PaqueteRTP`], se actualizaran estadisticas y luego de parsearlo a bytes se le envia
/// al otro peer por medio de un SocketUDP
///
/// Esto se hace en loop hasta que se detecte que la conexion llego a su fin.
/// Una conexion llego a su fin cuando el ultimo peer conectado envia un PaqueteRTCP_BYE.
///
/// En caso de Error se retorna un [`ErrorComunicacionRTP`].
pub fn manejo_envio_datagramas_video(
    mut socket: Box<dyn SocketUDP>,
    direccion_receptor: &str,
    receiver_camara: Receiver<Frame>,
    estadisticas_sender: Arc<Mutex<EstadisticasSender>>,
    estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    mut contexto_srtp_tx: SRTPContexto,
) -> Result<(), ErrorComunicacionRTP> {
    let (
        ssrc,
        mut cantidad_paquetes_enviados,
        mut cantidad_bytes_enviados,
        mut ultimo_numero_secuencia,
    ) = obtener_datos_nuevo_paquete(&estadisticas_sender)?;

    //Mientras la conexion no se termine.
    loop {
        //Se escucha informacion datagrama de camara de manera bloqueante.
        let frame_recibido = receiver_camara.recv().map_err(|_| {
            eprintln!(" ERROR : Se dejo de escuchar de la camara");
            ErrorComunicacionRTP::ErrorEscucharChannel
        })?;

        if frame_recibido.es_frame_finalizacion {
            return Ok(());
        }

        if frame_recibido.bytes.eq(&mensaje_finalizacion_llamada()) {
            break;
        }

        let payload = encodear_frame(frame_recibido)?;

        {
            let estadisticas = estadisticas_receiver
                .lock()
                .map_err(|_| ErrorComunicacionRTP::ErrorObtenerLock)?;

            if estadisticas.is_empty() {
                //Si no hay a quien enviarle nuestros frames se esperan
                // 3 segundos para evitar busy waits
                let espera = Duration::from_secs(1);
                thread::sleep(espera);
                continue;
            }
        }

        //Se establece el timestamp para el paqueteRTP
        let tiempo = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| ErrorComunicacionRTP::ErrorEstablecerTimestamp)?;

        let timestamp = tiempo.as_millis() as u32;

        //Se crea paqueteRTP
        let paquete_rtp = PaqueteRTP {
            version: VERSION_PAQUETE_RTP,
            padding: 0,
            extension: 0,
            conteo_csrc: 0,
            marcador: 0,
            tipo_payload: TIPO_PAYLOAD_VIDEO,
            numero_de_secuencia: ultimo_numero_secuencia,
            timestamp,
            ssrc,
            lista_csrc: Vec::new(),
            payload,
            padding_bytes: 0,
        };

        //Se parsea a bytes al paqueteRTP
        let mut paquete_bytes = Vec::from(&paquete_rtp);
        contexto_srtp_tx
            .proteger_y_firmar_rtp(ssrc, &mut paquete_bytes)
            .map_err(|_e: ErrorSRTP| ErrorComunicacionRTP::ErrorSRTPProtegiendo)?;

        //Se realiza el envio de paquete en bytes por medio del socketRTP
        socket
            .enviar(&paquete_bytes, direccion_receptor)
            .map_err(|_| ErrorComunicacionRTP::ErrorEnvioSocketUDP)?;

        //Se actualizan las estadisticas del Sender
        actualizar_estadisticas_sender(
            &mut cantidad_paquetes_enviados,
            &mut cantidad_bytes_enviados,
            &mut ultimo_numero_secuencia,
            paquete_bytes.len() as u32,
            timestamp,
            &estadisticas_sender,
        )?;
    }
    Ok(())
}

fn obtener_datos_nuevo_paquete(
    estadisticas_sender: &Arc<Mutex<EstadisticasSender>>,
) -> Result<(u32, u32, u32, u16), ErrorComunicacionRTP> {
    let ssrc: u32;
    let cantidad_paquetes_enviados: u32;
    let cantidad_bytes_enviados: u32;
    let ultimo_numero_secuencia: u16;
    {
        let estadisticas = estadisticas_sender
            .lock()
            .map_err(|_| ErrorComunicacionRTP::ErrorObtenerLock)?;
        ssrc = estadisticas.ssrc;
        cantidad_paquetes_enviados = estadisticas.cantidad_paquetes_enviados;
        cantidad_bytes_enviados = estadisticas.cantidad_bytes_enviados;
        ultimo_numero_secuencia = estadisticas.ultimo_numero_secuencia;
    }
    Ok((
        ssrc,
        cantidad_paquetes_enviados,
        cantidad_bytes_enviados,
        ultimo_numero_secuencia,
    ))
}

/// Se encarga de manejar el envio de datagramas que recibe de otro medio y enviar esta informacion al decodificador.
///
/// Flujo:
/// Se recibe un PaqueteRTP en bytes proveniente de otro peer por medio de un SocketUDP.
/// Se lo tranforma en un [`PaqueteRTP`] para poder calcular mejor las estadisticas y se las actualiza.
/// Luego por medio de un channel se le envia al decodificador los datagramas que contenia el payload de este.
///
/// Esto se hace en loop hasta que se detecte que la conexion llego a su fin.
/// Una conexion llego a su fin cuando el ultimo peer conectado envia un PaqueteRTCP_BYE.
///
/// Por cada ssrc(media) se mantiene una clave en un diccionario en donde se mantiene un registro de su estado.
/// El socket para esta funcion se encuentra dentro de un Box por cuestiones de implementacion.
///
/// En caso de Error se retorna un [`ErrorComunicacionRTP`].
pub fn manejo_recepcion_datagramas(
    srtp_rx: Receiver<Vec<u8>>, // reemplaza el socket - ahora recibe del demux
    sender_reproductor: Sender<Vec<u8>>,
    sender_audio: Sender<Vec<i16>>,
    estadisticas_receiver: Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
    reproductor_procesando_frame: Arc<Mutex<bool>>,
    mut contexto_srtp_rx: SRTPContexto,
) -> Result<(), ErrorComunicacionRTP> {
    let timestamp_audio = Arc::new(Mutex::new(0));
    let referencia_timestam_audio = Arc::clone(&timestamp_audio);

    //se inicia gestion del jitterBuffer
    let (tx_rtp_video, rx_rtp_video) = mpsc::channel::<MensajeJitterBuffer>();
    let (tx_rtp_audio, rx_rtp_audio) = mpsc::channel::<MensajeJitterBuffer>();
    //iniciar_gestion_jitter_buffer(rx_rtp, sender_reproductor, 25, reproductor_procesando_frame);

    JitterBufferVideo::iniciar_gestion_jitter_buffer(
        rx_rtp_video,
        20,
        Informacion::InformacionVideo(sender_reproductor, reproductor_procesando_frame),
        timestamp_audio,
    );

    JitterBufferAudio::iniciar_gestion_jitter_buffer(
        rx_rtp_audio,
        15,
        Informacion::InformacionAudio(sender_audio),
        referencia_timestam_audio,
    );

    let mensaje = mensaje_finalizacion_llamada().to_vec();
    loop {
        //Se lee un paquete del canal SRTP (antes era del SocketUDP directamente,
        // ahora el demux es el unico lector del socket y nos reenvía lo que corresponde)
        match srtp_rx.recv() {
            Ok(bytes_recibidos) => {
                if bytes_recibidos == mensaje {
                    //Se le avisa a los hilos hitterBuffer que termino la llamada
                    tx_rtp_video
                        .send(MensajeJitterBuffer::MensajeFinalizacionLlamada)
                        .map_err(|_| ErrorComunicacionRTP::ErrorEnviarChannel)?;
                    tx_rtp_audio
                        .send(MensajeJitterBuffer::MensajeFinalizacionLlamada)
                        .map_err(|_| ErrorComunicacionRTP::ErrorEnviarChannel)?;
                    break;
                }

                let mut paquete_srtp = bytes_recibidos;

                if paquete_srtp.len() < 12 {
                    // RTP header mínimo
                    return Err(ErrorComunicacionRTP::ErrorSerializarAPaquete);
                }

                // El SSRC está en los bytes 8..12 del header RTP (sin cifrar)
                let ssrc_recibido = u32::from_be_bytes([
                    paquete_srtp[8],
                    paquete_srtp[9],
                    paquete_srtp[10],
                    paquete_srtp[11],
                ]);

                // Verificación HMAC + anti-replay + descifrado in-place
                contexto_srtp_rx
                    .verificar_y_desproteger_rtp(ssrc_recibido, &mut paquete_srtp)
                    .map_err(|e: ErrorSRTP| {
                        println!("[SRTP-RX] FALLÓ verificar_y_desproteger_rtp: {:?}", e);
                        ErrorComunicacionRTP::ErrorSRTPVerificando
                    })?;

                let paquete_rtp = PaqueteRTP::try_from(paquete_srtp.as_slice())
                    .map_err(|_| ErrorComunicacionRTP::ErrorSerializarAPaquete)?;

                // Ignoro los paquetes de video
                if paquete_rtp.tipo_payload == TIPO_PAYLOAD_AUDIO {
                    tx_rtp_audio
                        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
                            InformacionPaqueteRTP {
                                timestamp: paquete_rtp.timestamp,
                                payload: paquete_rtp.payload,
                            },
                        ))
                        .map_err(|_| ErrorComunicacionRTP::ErrorEnviarChannel)?;
                } else {
                    //Se le envia al jitterBuffer
                    tx_rtp_video
                        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
                            InformacionPaqueteRTP {
                                timestamp: paquete_rtp.timestamp,
                                payload: paquete_rtp.payload,
                            },
                        ))
                        .map_err(|_| ErrorComunicacionRTP::ErrorEnviarChannel)?;
                }

                actualizar_estadisticas_receiver(
                    paquete_rtp.ssrc,
                    paquete_rtp.numero_de_secuencia as u32,
                    &estadisticas_receiver,
                )?;
            }
            Err(_) => {
                return Err(ErrorComunicacionRTP::ErrorRecibirSocketUDP);
            }
        }
    }
    Ok(())
}

fn encodear_frame(frame: Frame) -> Result<Vec<u8>, ErrorComunicacionRTP> {
    let api = OpenH264API::from_source();
    let configs = EncoderConfig::new().bitrate(BitRate::from_bps(1500));

    let mut encoder = Encoder::with_api_config(api, configs)
        .map_err(|_| ErrorComunicacionRTP::ErrorEncodeandoFrame)?;

    let bytes_rgb = RgbSliceU8::new(&frame.bytes[..], (frame.anchura, frame.altura));
    let bytes_imagen_yuv = YUVBuffer::from_rgb8_source(bytes_rgb);
    let bytes_encodeados = encoder
        .encode(&bytes_imagen_yuv)
        .map_err(|_| ErrorComunicacionRTP::ErrorEncodeandoFrame)?;

    let bytes_frame_encodeado = bytes_encodeados.to_vec();
    Ok(bytes_frame_encodeado)
}

fn actualizar_estadisticas_sender(
    cantidad_paquetes_enviados: &mut u32,
    cantidad_bytes_enviados: &mut u32,
    ultimo_numero_secuencia: &mut u16,
    tamanio_paquete: u32,
    timestamp: u32,
    estadisticas_sender: &Arc<Mutex<EstadisticasSender>>,
) -> Result<(), ErrorComunicacionRTP> {
    //Se actualizan las estadisticas del Sender
    *cantidad_paquetes_enviados += 1;
    *cantidad_bytes_enviados += tamanio_paquete;
    *ultimo_numero_secuencia += 1;

    let mut estadisticas = estadisticas_sender
        .lock()
        .map_err(|_| ErrorComunicacionRTP::ErrorObtenerLock)?;
    estadisticas.cantidad_paquetes_enviados = *cantidad_paquetes_enviados;
    estadisticas.cantidad_bytes_enviados = *cantidad_bytes_enviados;
    estadisticas.ultimo_numero_secuencia = *ultimo_numero_secuencia;
    estadisticas.ultimo_timestamp_enviado = timestamp;
    Ok(())
}

fn actualizar_estadisticas_receiver(
    ssrc: u32,
    numero_secuencia: u32,
    estadisticas_receiver: &Arc<Mutex<HashMap<u32, EstadisticasReceiver>>>,
) -> Result<(), ErrorComunicacionRTP> {
    //Se actualizan las estadisticas para el ssrc
    //Momentaneamente todos los ssrc son aceptados de una (en realidad se los toma como validos luego de una cant de secuencias validas)
    let mut estadisticas = estadisticas_receiver
        .lock()
        .map_err(|_| ErrorComunicacionRTP::ErrorObtenerLock)?;

    //En caso de ser la primera vez que se recibe un paquete de el ssrc se lo registra/inicializa en el diccionario
    let reporte_ssrc = estadisticas
        .entry(ssrc)
        .or_insert(EstadisticasReceiver::new(ssrc));

    reporte_ssrc.cantidad_paquetes_recibidos += 1;

    if numero_secuencia
        > reporte_ssrc
            .contenido_report
            .numero_mas_grande_de_paquete_recibido
    {
        reporte_ssrc
            .contenido_report
            .numero_mas_grande_de_paquete_recibido = numero_secuencia;
    }
    Ok(())
}

fn mensaje_finalizacion_llamada() -> Vec<u8> {
    let mut mensaje_bytes: Vec<u8> = Vec::new();

    let mensaje = "END".as_bytes();

    for letra in mensaje {
        mensaje_bytes.push(*letra);
    }

    mensaje_bytes
}
