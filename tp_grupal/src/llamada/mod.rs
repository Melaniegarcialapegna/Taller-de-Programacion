//! Llamada creada mediante WebRTC
//!
//! Este trait representa una llamada en curso entre dos peers mediante WebRTC. La responsabilidad de [Llamada] sera administrar la inicialización
//! y finalización de las llamadas.
//!
//! Una vez creada la conexión entre ambos peers mediante el [MediadorDeConexiones](crate::creacion_llamada::mediador_de_conexiones::MediadorDeConexionesP2P),
//! se debera usar [Llamada] para solicitar un proximo frame a mostrar, o bien para cortar la llamada.

pub mod administrador_de_camaras;
pub mod camara;
pub mod camara_mock;
pub mod creador_lente;
pub mod creador_lente_nokwha;
pub mod lente;
pub mod lente_nokwha;
#[cfg(test)]
pub mod llamada_mock;
pub mod microfono;
pub mod microfono_default;
pub mod panel_estadisticas;
pub mod sesion_rtp;

use bytes::Bytes;
use std::{
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use crate::protocolos::sctp::evento_sctp::EventoSctp;
use crate::protocolos::sctp::protocolo_archivo::MensajeArchivo;
#[cfg(test)]
use crate::reproductor::reproductor_audio::ReproductorAudioDummy;
use crate::{
    aplicacion::{EventoAplicacion, EventoInternoAplicacion, EventoLlamada},
    llamada::{
        camara::ErrorCamara,
        panel_estadisticas::{ErrorPanelEstadisticas, PanelEstadisticas},
    },
    logger::Logger,
    sesion_rtp::{comunicacion_rtp::ComunicadoresConIO, sesion::EstadisticasReceiver},
};
use crate::{
    creacion_llamada::ConexionP2P,
    reproductor::reproductor_audio::{ErrorReproductorAudio, ReproductorAudio},
};
use crate::{
    llamada::microfono::{ErrorMicrofono, Microfono},
    reproductor::{ErrorReproductor, Reproductor},
};
use crate::{
    llamada::{
        camara::Camara,
        sesion_rtp::{ErrorSesionRTP, SesionRTP},
    },
    sesion_rtp::comunicacion_rtp::Frame,
};

#[derive(Debug)]
pub enum ErrorLlamada {
    /// Error con un [Reproductor] de video
    ErrorConReproductor(String),
    /// Error en la [SesionRTP] creada
    ErrorEnSesionRTP(String),
    /// Error en la [Camara] creada
    ErrorEnCamara(String),
    /// Error en el channel al enviar un nuevo frame a la Aplicacion
    ErrorEnviandoNuevoFrame,
    /// Error interno
    ErrorInterno(String),
    /// Error en el microfono que se usa en la llamada
    ErrorEnMicrofono(String),
    /// Error en el reproductor de audio
    ErrorConReproductorDeAudio(String),
    /// Error en el panel de estadisticas
    ErrorEnPanelDeEstadisticas(String),
}

/// Representa los reproductores de una Llamada entre dos peers: el reproductor de la camara
/// local, y el reproductor de la camara del otro peer.
#[allow(dead_code)]
pub struct ReproductoresLlamada {
    reproductor_local: Box<dyn Reproductor>,
    reproductor_frames_externos: Box<dyn Reproductor>,
    reproductor_audio: Box<dyn ReproductorAudio>,
    senders_a_reproductores: SendersAReproductores,
}

/// Representa canales mediante los cuales se pueden enviar nuevos frames a reproducir a los
/// reproductores de la llamada
#[allow(dead_code)]
pub struct SendersAReproductores {
    sender_a_reproductor_local: Sender<Frame>,
    sender_a_reproductor_frames_externos: Sender<Vec<u8>>,
    sender_audio: Sender<Vec<i16>>,
}

/// Representa los dispositivos de entrada que se usaran para hacer la videollamada
pub struct DispositivosEntrada {
    pub camara: Box<dyn Camara>,
    pub microfono: Box<dyn Microfono<i16>>,
}

/// Representa un canal de comunicacion entre hilos de [Llamada].
/// Este canal permite que los distintos hilos de [Llamada] se comuniquen entre si, y
/// ademas permite un funcionamiento similar a enviar mensajes desde el exterior a un thread determinado de Llamada.
/// El hilo receptor de este canal sera el que se encargue de iniciar y finalizar llamadas.
pub struct CanalDeComunicacionLlamada {
    sender_eventos_llamada: Sender<EventoLlamada>,
    receiver_eventos_llamada: Receiver<EventoLlamada>,
}

/// Representa medios por los cuales se pueden informar eventos ocurridos.
pub struct ComunicadoresDeEventos {
    sender_eventos_internos: Sender<EventoInternoAplicacion>,
    logger: Logger,
}

impl ReproductoresLlamada {
    pub fn new(
        reproductor_local: Box<dyn Reproductor>,
        sender_a_reproductor_local: Sender<Frame>,
        reproductor_frames_externos: Box<dyn Reproductor>,
        reproductor_audio: Box<dyn ReproductorAudio>,
        sender_a_reproductor_frames_externos: Sender<Vec<u8>>,
        sender_audio: Sender<Vec<i16>>,
    ) -> ReproductoresLlamada {
        ReproductoresLlamada {
            reproductor_local,
            reproductor_frames_externos,
            reproductor_audio,
            senders_a_reproductores: SendersAReproductores {
                sender_a_reproductor_local,
                sender_a_reproductor_frames_externos,
                sender_audio,
            },
        }
    }
}

impl Display for ErrorLlamada {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLlamada::ErrorInterno(error) => {
                f.write_str(&format!("Error interno en llamada: {error}"))
            }
            ErrorLlamada::ErrorConReproductor(error) => {
                f.write_str(&format!("Error en reproductor: {error}"))
            }
            ErrorLlamada::ErrorEnCamara(error) => f.write_str(&format!("Error en camara: {error}")),
            ErrorLlamada::ErrorEnSesionRTP(error) => {
                f.write_str(&format!("Error en Sesion RTP: {error}"))
            }
            ErrorLlamada::ErrorEnviandoNuevoFrame => f.write_str("Error enviando nuevo frame"),
            ErrorLlamada::ErrorEnMicrofono(error) => {
                f.write_str(&format!("Error en microfono: {error}"))
            }
            ErrorLlamada::ErrorConReproductorDeAudio(error) => {
                f.write_str(&format!("Error con reproductor de audio: {error}"))
            }
            ErrorLlamada::ErrorEnPanelDeEstadisticas(error) => {
                f.write_str(&format!("Error en panel de estadisticas: {error}"))
            }
        }
    }
}

/// Representa una Llamada mediante WebRTC.
pub trait Llamada {
    /// Solicita al reproductor local y al reproductor de frames externos que se envie un proximo frame.
    /// Llamada enviara los proximos frames enviando dos [EventoAplicacion].
    ///
    /// En caso de fallar, devuelve un [ErrorLlamada].
    fn enviar_proximo_frame(&mut self) -> Result<(), ErrorLlamada>;
    /// Corta la llamada actual mediante WebRTC volviendo al estado inicial de [Llamada].
    ///
    /// En caso de fallar, devuelve un [ErrorLlamada].
    fn cortar_llamada(&mut self) -> Result<(), ErrorLlamada>;
    /// Cambia la camara que se usara para la llamada
    ///
    /// En caso de fallar, devuelve un [ErrorLlamada].
    fn cambiar_camara(&mut self, nueva_camara: Box<dyn Camara>) -> Result<(), ErrorLlamada>;
    /// Mutea el microfono que se usa durante la llamada
    ///
    /// En caso de fallar, devuelve un [ErrorLlamada].
    fn mutear_microfono(&mut self) -> Result<(), ErrorLlamada>;
    /// Desmutea el microfono que se usa durante la llamada
    ///
    /// En caso de fallar, devuelve un [ErrorLlamada].
    fn desmutear_microfono(&mut self) -> Result<(), ErrorLlamada>;
    ///
    /// En caso de fallar, devuelve un [ErrorLlamada]
    fn enviar_archivo(&mut self, path: PathBuf) -> Result<(), ErrorLlamada>;
    /// Acepta la oferta de archivo recibida del peer remoto
    fn aceptar_archivo(&mut self) -> Result<(), ErrorLlamada>;
    /// Rechaza la oferta de archivo recibida del peer remoto
    fn rechazar_archivo(&mut self) -> Result<(), ErrorLlamada>;
}

#[allow(dead_code)]
pub struct LlamadaRTP {
    pub reproductor: Box<dyn Reproductor>,
    pub reproductor_local: Box<dyn Reproductor>,
    pub reproductor_audio: Box<dyn ReproductorAudio>,
    pub sender_eventos_aplicacion: Sender<EventoInternoAplicacion>,
    pub sender_eventos_llamada: Sender<EventoLlamada>,
    mutex_panel_estadisticas: Arc<Mutex<PanelEstadisticas>>,
}

impl Llamada for LlamadaRTP {
    fn enviar_proximo_frame(&mut self) -> Result<(), ErrorLlamada> {
        let frame_otro_peer = self.reproductor.proximo_frame()?;
        let frame_local = self.reproductor_local.proximo_frame()?;

        self.sender_eventos_aplicacion
            .send(EventoInternoAplicacion::EventoObservable(
                EventoAplicacion::NuevoFrame(frame_otro_peer),
            ))
            .map_err(|_| ErrorLlamada::ErrorEnviandoNuevoFrame)?;

        self.sender_eventos_aplicacion
            .send(EventoInternoAplicacion::EventoObservable(
                EventoAplicacion::NuevoFrameLocal(frame_local),
            ))
            .map_err(|_| ErrorLlamada::ErrorEnviandoNuevoFrame)?;

        self.reproductor_audio.despausar()?;

        // Se puede tardar unos frames en registrar la fuente. Mientras tanto,
        // se pueden simplemente no enviar estadisticas hasta que esten disponibles
        if let Ok(estadisticas) = self.obtener_estadisticas_panel() {
            self.sender_eventos_aplicacion
                .send(EventoInternoAplicacion::EventoObservable(
                    EventoAplicacion::NuevasEstadisticas(Box::new(estadisticas)),
                ))
                .map_err(|_| ErrorLlamada::ErrorEnviandoNuevoFrame)?;
        };

        Ok(())
    }

    fn cortar_llamada(&mut self) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::LlamadaFinalizada)
            .map_err(|_| {
                ErrorLlamada::ErrorInterno(
                    "Fallo enviando el mensaje LlamadaFinalizada dentro de Llamada".to_string(),
                )
            })?;

        self.reproductor_audio.pausar()?;

        Ok(())
    }

    fn cambiar_camara(&mut self, nueva_camara: Box<dyn Camara>) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::NuevaCamara(nueva_camara))
            .map_err(|_| {
                ErrorLlamada::ErrorInterno("Fallo al enviar nueva camara a la llamada".to_string())
            })?;

        Ok(())
    }

    fn mutear_microfono(&mut self) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::MutearMicrofono)
            .map_err(|_| ErrorLlamada::ErrorInterno("Fallo al mutear microfono".to_string()))?;
        Ok(())
    }

    fn desmutear_microfono(&mut self) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::DesmutearMicrofono)
            .map_err(|_| ErrorLlamada::ErrorInterno("Fallo al desmutear microfono".to_string()))?;
        Ok(())
    }

    fn enviar_archivo(&mut self, path: PathBuf) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::EnviarArchivo(path))
            .map_err(|_| ErrorLlamada::ErrorInterno("Fallo al enviar archivo".to_string()))?;
        Ok(())
    }

    fn aceptar_archivo(&mut self) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::AceptarArchivo)
            .map_err(|_| ErrorLlamada::ErrorInterno("Fallo al aceptar archivo".to_string()))?;
        Ok(())
    }

    fn rechazar_archivo(&mut self) -> Result<(), ErrorLlamada> {
        self.sender_eventos_llamada
            .send(EventoLlamada::RechazarArchivo)
            .map_err(|_| ErrorLlamada::ErrorInterno("Fallo al rechazar archivo".to_string()))?;
        Ok(())
    }
}

impl LlamadaRTP {
    /// Crea una LlamadaRTP. Recibe:
    /// - reproductores: Seran los reproductores de video local y externo de la llamada
    /// - sender_eventos_aplicacion: Sera el medio por el cual se informara de eventos ocurridos dentro de [Llamada]
    /// - sender_eventos_llamada: Sera el medio para enviar mensajes al thread que iniciara y finalizara llamadas
    /// - receiver_eventos_llamada: Sera quien reciba los mensajes relacionados a iniciar llamadas y finalizarlas.
    /// - camara: Sera la camara a utilizar como camara local.
    /// - sender_eventos_llamada_a_telefono: Sera el medio por el cual se informe al Telefono cuando una llamada finalice,
    ///   para que este permita que se vuelvan a iniciar llamadas.
    pub fn new(
        dispositivos: DispositivosEntrada,
        mut reproductores: ReproductoresLlamada,
        sender_eventos_aplicacion: Sender<EventoInternoAplicacion>,
        sender_eventos_llamada: Sender<EventoLlamada>,
        receiver_eventos_llamada: Receiver<EventoLlamada>,
        sender_eventos_llamada_a_telefono: Sender<EventoLlamada>,
        logger: Logger,
    ) -> Result<LlamadaRTP, ErrorLlamada> {
        let mutex_procesando_frame = reproductores
            .reproductor_frames_externos
            .esta_procesando_frame()?;

        // Creo un sender para que el hilo pueda comunicar a Aplicacion eventos sucedidos
        let sender_eventos_aplicacion_clon = sender_eventos_aplicacion.clone();

        // Creo comunicacion interna de Llamada
        let sender_eventos_llamada_clon = sender_eventos_llamada.clone();
        let comunicacion_llamada = CanalDeComunicacionLlamada {
            sender_eventos_llamada,
            receiver_eventos_llamada,
        };

        // Creo un panel de estadisticas al que se le consultaran reportes sobre la llamada
        let panel_estadisticas = PanelEstadisticas::default();
        let mutex_panel_estadisticas = Arc::new(Mutex::new(panel_estadisticas));
        let clon_mutex_panel_estadisticas = Arc::clone(&mutex_panel_estadisticas);

        // Creo comunicadores de eventos
        let comunicadores_eventos = ComunicadoresDeEventos {
            logger,
            sender_eventos_internos: sender_eventos_aplicacion_clon,
        };

        thread::spawn(move || {
            Self::escuchar_eventos_llamada(
                comunicacion_llamada,
                dispositivos,
                reproductores.senders_a_reproductores,
                mutex_procesando_frame,
                comunicadores_eventos,
                sender_eventos_llamada_a_telefono,
                clon_mutex_panel_estadisticas,
            );
        });

        Ok(LlamadaRTP {
            reproductor: reproductores.reproductor_frames_externos,
            reproductor_local: reproductores.reproductor_local,
            reproductor_audio: reproductores.reproductor_audio,
            sender_eventos_aplicacion,
            sender_eventos_llamada: sender_eventos_llamada_clon,
            mutex_panel_estadisticas,
        })
    }

    fn escuchar_eventos_llamada(
        comunicacion_llamada: CanalDeComunicacionLlamada,
        dispositivos: DispositivosEntrada,
        senders_a_reproductores: SendersAReproductores,
        mutex_procesando_frame: Arc<Mutex<bool>>,
        comunicadores_eventos: ComunicadoresDeEventos,
        sender_eventos_llamada_a_telefono: Sender<EventoLlamada>,
        mutex_panel_estadisticas: Arc<Mutex<PanelEstadisticas>>,
    ) {
        let logger = comunicadores_eventos.logger.clone();
        if let Err(e) = Self::_escuchar_eventos_llamada(
            comunicacion_llamada,
            dispositivos,
            senders_a_reproductores,
            mutex_procesando_frame,
            comunicadores_eventos,
            sender_eventos_llamada_a_telefono,
            mutex_panel_estadisticas,
        ) {
            logger.error(&format!("{e}"), "LlamadaRTP");
        }
    }

    /// Escucha eventos del tipo [EventoLlamada], que corresponden a eventos relacionados a una llamada
    /// en curso o a una llamada proxima a iniciar. Estos eventos se escuchan dentro en un thread propio,
    /// que pertenece a [LlamadaRTP]. Los eventos escuchados pueden ser recibidos (de momento) de tres lugares:
    ///
    /// - De la propia LlamadaRTP, para ejecutar un metodo cuyas consecuencias deban reflejarse en este hilo.
    ///
    /// - Por el [MediadorDeConexiones](crate::creacion_llamada::mediador_de_conexiones::MediadorDeConexionesP2P),
    ///   que nos podra informar cuando la conexion se haya establecido y la llamada haya iniciado.
    ///
    /// - De [SesionRTP], que nos podra informar cuando se nos hayan cortado la llamada desde el otro peer para que
    ///   nosotros tambien cortemos.
    ///
    /// Tambien se tiene un [Sender] del que escuchara [Telefono](crate::comunicacion::telefono::Telefono), por el cual
    /// se le avisara cuando la llamada finalice para que pueda escuchar nuevas llamadas
    fn _escuchar_eventos_llamada(
        comunicacion_llamada: CanalDeComunicacionLlamada,
        mut dispositivos: DispositivosEntrada,
        senders_a_reproductores: SendersAReproductores,
        mutex_procesando_frame: Arc<Mutex<bool>>,
        comunicadores_eventos: ComunicadoresDeEventos,
        sender_eventos_llamada_a_telefono: Sender<EventoLlamada>,
        mut mutex_panel_estadisticas: Arc<Mutex<PanelEstadisticas>>,
    ) -> Result<(), ErrorLlamada> {
        let mutex_sesion = Arc::new(Mutex::new(None));
        let mut tx_datos_sctp: Option<Sender<Bytes>> = None;
        let mut tx_respuesta_sctp: Option<Sender<Bytes>> = None;
        let mut path_archivo_pendiente: Option<PathBuf> = None;

        loop {
            let evento_recibido = comunicacion_llamada
                .receiver_eventos_llamada
                .recv()
                .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;

            match evento_recibido {
                EventoLlamada::LlamadaIniciada(conexion) => {
                    let (sesion_nueva, rx_eventos_sctp) = Self::iniciar_nueva_llamada(
                        &mut dispositivos,
                        &senders_a_reproductores,
                        &mutex_procesando_frame,
                        conexion,
                        comunicacion_llamada.sender_eventos_llamada.clone(),
                        &mut mutex_panel_estadisticas,
                        comunicadores_eventos.logger.clone(),
                    )?;
                    {
                        let mut sesion = mutex_sesion.lock().map_err(|_| {
                            ErrorLlamada::ErrorInterno(
                                "Fallo obteniendo lock de SesionRTP".to_string(),
                            )
                        })?;
                        tx_datos_sctp = sesion_nueva.clonar_tx_datos_sctp();
                        tx_respuesta_sctp = tx_datos_sctp.clone();
                        *sesion = Some(sesion_nueva);
                    }
                    // spawneo thread que escucha eventos SCTP entrantes y los reenvía como EventoLlamada
                    if let Some(rx) = rx_eventos_sctp {
                        let sender_llamada = comunicacion_llamada.sender_eventos_llamada.clone();
                        thread::spawn(move || {
                            Self::escuchar_eventos_sctp(rx, sender_llamada);
                        });
                    }
                }
                EventoLlamada::LlamadaFinalizada => {
                    path_archivo_pendiente = None; // limpio x las dudas, asi no interfiere en futura llamada
                    tx_datos_sctp = None;
                    tx_respuesta_sctp = None;
                    Self::finalizar_llamada(
                        &mut dispositivos.camara,
                        &mut dispositivos.microfono,
                        &mutex_sesion,
                        &comunicadores_eventos.sender_eventos_internos,
                        sender_eventos_llamada_a_telefono.clone(),
                        &mut mutex_panel_estadisticas,
                    )?
                }
                EventoLlamada::NuevaCamara(nueva_camara) => {
                    dispositivos.camara = nueva_camara;
                }
                EventoLlamada::MutearMicrofono => dispositivos.microfono.mutear()?,
                EventoLlamada::DesmutearMicrofono => dispositivos.microfono.desmutear()?,
                EventoLlamada::EnviarArchivo(path) => {
                    if let Some(tx) = &tx_datos_sctp {
                        let tx_clon = tx.clone();
                        let path_clon = path.clone();
                        // abro thread para leer el archivo y enviar la oferta, para no bloquear el hilo de eventos de llamada con la lectura del archivo
                        thread::spawn(move || {
                            let nombre = match path_clon.file_name().and_then(|n| n.to_str()) {
                                Some(n) => n.to_string(),
                                None => "archivo".to_string(),
                            };
                            match fs::metadata(&path_clon) {
                                Ok(meta) => {
                                    let oferta = MensajeArchivo::OfertaArchivo {
                                        nombre,
                                        tamanio: meta.len(),
                                    }
                                    .serializar();
                                    if let Err(e) = tx_clon.send(oferta) {
                                        eprintln!(
                                            "[Llamada] Error enviando oferta de archivo: {e}"
                                        );
                                    } else {
                                        eprintln!(
                                            "[Llamada] Oferta de archivo enviada: {:?}",
                                            path_clon
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[Llamada] Error leyendo metadata de {:?}: {e}",
                                        path_clon
                                    )
                                }
                            }
                        });
                        path_archivo_pendiente = Some(path);
                    } else {
                        eprintln!(
                            "[Llamada] Se intentó enviar archivo pero el canal SCTP no está disponible"
                        );
                    }
                }
                EventoLlamada::OfertaArchivoRecibida { nombre, tamanio } => {
                    comunicadores_eventos
                        .sender_eventos_internos
                        .send(EventoInternoAplicacion::EventoObservable(
                            EventoAplicacion::RecibendoOfertaArchivo { nombre, tamanio },
                        ))
                        .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;
                }
                EventoLlamada::ArchivoAceptadoPorPeer => {
                    comunicadores_eventos
                        .sender_eventos_internos
                        .send(EventoInternoAplicacion::EventoObservable(
                            EventoAplicacion::ArchivoAceptadoPorPeer,
                        ))
                        .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;
                    // ahora q el peer aceptó enviamos
                    if let (Some(tx), Some(path)) = (&tx_datos_sctp, path_archivo_pendiente.take())
                    {
                        let tx_clon = tx.clone();
                        thread::spawn(move || match fs::read(&path) {
                            Ok(datos) => {
                                let chunk_size: usize = 32_768; //son 32kb lol
                                //let chunk_size: usize = 65_536; // son 64kb, el maximo que se puede enviar por SCTP supuestamente !!
                                for chunk in datos.chunks(chunk_size) {
                                    let msg =
                                        MensajeArchivo::DatosArchivo(Bytes::copy_from_slice(chunk))
                                            .serializar();
                                    if let Err(e) = tx_clon.send(msg) {
                                        eprintln!("[Llamada] Error enviando datos de archivo: {e}");
                                        return;
                                    }
                                }
                                let fin = MensajeArchivo::FinArchivo.serializar();
                                if let Err(e) = tx_clon.send(fin) {
                                    eprintln!("[Llamada] Error enviando FinArchivo: {e}");
                                }
                                eprintln!("[Llamada] Archivo enviado: {:?}", path);
                            }
                            Err(e) => eprintln!("[Llamada] Error leyendo archivo {:?}: {e}", path),
                        });
                    }
                }
                EventoLlamada::ArchivoRechazadoPorPeer => {
                    path_archivo_pendiente = None;
                    comunicadores_eventos
                        .sender_eventos_internos
                        .send(EventoInternoAplicacion::EventoObservable(
                            EventoAplicacion::ArchivoRechazadoPorPeer,
                        ))
                        .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;
                }
                EventoLlamada::AceptarArchivo => {
                    if let Some(tx) = &tx_respuesta_sctp {
                        let msg = MensajeArchivo::AceptarArchivo.serializar();
                        if let Err(e) = tx.send(msg) {
                            eprintln!("[Llamada] Error enviando AceptarArchivo por SCTP: {e}");
                        }
                    }
                }
                EventoLlamada::RechazarArchivo => {
                    if let Some(tx) = &tx_respuesta_sctp {
                        let msg = MensajeArchivo::RechazarArchivo.serializar();
                        if let Err(e) = tx.send(msg) {
                            eprintln!("[Llamada] Error enviando RechazarArchivo por SCTP: {e}");
                        }
                    }
                }
                EventoLlamada::ArchivoRecibido { nombre, datos } => {
                    let dir_base = match std::env::current_dir() {
                        Ok(dir) => dir,
                        Err(_) => PathBuf::from("."),
                    };
                    let destino = dir_base.join("archivos_recibidos").join(&nombre);

                    if let Some(parent) = destino.parent()
                        && let Err(e) = fs::create_dir_all(parent)
                    {
                        eprintln!("[Llamada] Error creando directorio de recepción: {e}");
                    }

                    match fs::write(&destino, &datos) {
                        Ok(_) => {
                            eprintln!("[Llamada] Archivo guardado en {:?}", destino);
                            comunicadores_eventos
                                .sender_eventos_internos
                                .send(EventoInternoAplicacion::EventoObservable(
                                    EventoAplicacion::ArchivoRecibido {
                                        nombre,
                                        ruta: destino,
                                    },
                                ))
                                .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;
                        }
                        Err(e) => eprintln!("[Llamada] Error guardando archivo {:?}: {e}", destino),
                    }
                }
            };
        }
    }

    /// Inicia una nueva llamada mediante la [ConexionP2P] recibida. Los pasos que se siguen para establecer la nueva llamada
    /// son los siguientes:
    ///
    /// 1. Se preparan los dispositivos de captura y se crea un struct de [ComunicadoresConIO], que consiste de un pares de receivers y un pares de senders de channels.
    ///    De los dos receivers se recibira el audio y video capturados para ser transmitidos al otro peer. Los dos senders serviran para enviar el audio y video recibido
    ///    del otro peer, para ser transmitido en los reproductores de audio y video del cliente.
    ///
    /// 2. Se crea la SesionRTP, que recibira la conexion, los comunicadores con los dispositivos
    ///    y un [Sender] mediante el cual nos podra avisar cuando la llamada finalice.
    ///
    /// 3. Se cambia la fuente de datos del panel de estadisticas para que ahora muestre datos sobre la sesion actual.
    ///
    /// 4. Almaceno la SesionRTP actual por si se necesita mas adelante.
    fn iniciar_nueva_llamada(
        dispositivos: &mut DispositivosEntrada,
        senders_a_reproductores: &SendersAReproductores,
        mutex_procesando_frame: &Arc<Mutex<bool>>,
        conexion: Box<ConexionP2P>,
        sender_eventos_llamada: Sender<EventoLlamada>,
        mutex_panel_estadisticas: &mut Arc<Mutex<PanelEstadisticas>>,
        logger: Logger,
    ) -> Result<(SesionRTP, Option<Receiver<EventoSctp>>), ErrorLlamada> {
        // Creo un canal para mandar frames de la camara
        let (sender_frames_camara, receiver_frames_camara) = mpsc::channel();

        // Configuro la camara y la enciendo
        if Self::iniciar_camara(
            &mut dispositivos.camara,
            &senders_a_reproductores.sender_a_reproductor_local,
            sender_frames_camara,
        )
        .is_err()
        {
            eprintln!("Fallo al abrir la camara local")
        };

        //Creo un canal para mandar audio y le digo al microfono que mande ahi
        let (sender_audio, receiver_audio) = mpsc::channel();
        dispositivos.microfono.cambiar_receptor(sender_audio)?;
        dispositivos.microfono.mutear()?;

        // Creo comunicadores con IO
        let comunicadores_con_io = ComunicadoresConIO::new(
            receiver_frames_camara,
            senders_a_reproductores
                .sender_a_reproductor_frames_externos
                .clone(),
            receiver_audio,
            senders_a_reproductores.sender_audio.clone(),
        );

        // Creo la sesion RTP
        let mut sesion_nueva = SesionRTP::new(
            conexion,
            comunicadores_con_io,
            Arc::clone(mutex_procesando_frame),
            sender_eventos_llamada,
            logger,
        )?;

        Self::actualizar_panel_estadisticas(mutex_panel_estadisticas, &mut sesion_nueva)?;

        // Extraigo el rx de eventos SCTP antes de devolver la sesion
        let rx_eventos_sctp = sesion_nueva.obtener_rx_eventos_sctp();

        Ok((sesion_nueva, rx_eventos_sctp))
    }

    /// Finaliza la llamada que estaba en curso y vuelve a dejar los componentes de la llamada preparados para
    /// el inicio de una nueva llamada.
    ///
    /// Este metodo se ejecutara cuando se reciba el evento [EventoLlamada::LlamadaFinalizada], que se puede recibir
    /// en dos casos:
    ///
    /// - Si se ejecuta directamente el metodo [Llamada::cortar_llamada]
    /// - Si el otro peer nos corta la llamada enviandonos un mensaje BYE por RTCP.
    ///
    /// En ambos casos, lo que debemos hacer es cortar la llamada. **Observación**: si se corta la llamada ejecutando el metodo
    /// [Llamada::cortar_llamada], el otro peer tambien nos cortara a nosotros (decision de diseño). Por eso, nos va a llegar el evento BYE por RTCP.
    /// En ese caso, no hay que hacer nada porque significa que la llamada ya se corto de ambos lados.
    ///
    /// Los pasos que se siguen (de momento) para finalizar la llamada son los siguientes:
    ///
    /// 1. Se le dice a la [SesionRTP] actual que corte la llamada.
    /// 2. Se mutea el microfono, y luego se elimina el receptor que se tenia del mismo (la sesion estaba escucha el audio capturado, y ahora ya no).
    /// 3. Se reinicia la camara, que tampoco enviara mas frames a la sesion para ser transmitidos.
    /// 4. Se borra la fuente de datos del panel de estadisticas.
    /// 5. Se elimina la referencia a la [SesionRTP] ahora finalizada
    /// 6. Se le informa al Telefono que la llamada termino enviando un [EventoLlamada::LlamadaFinalizada] mediante un Sender
    /// 7. Se envia a Aplicacion el evento [EventoAplicacion::LlamadaFinalizada], para que sea informado a todos sus suscribers.
    ///
    /// POST: Luego de ejecutar este metodo, el hilo que escucha eventos de llamada esta preparado para iniciar una nueva llamada.
    fn finalizar_llamada(
        camara: &mut Box<dyn Camara + 'static>,
        microfono: &mut Box<dyn Microfono<i16>>,
        mutex_sesion: &Arc<Mutex<Option<SesionRTP>>>,
        sender_eventos_aplicacion: &Sender<EventoInternoAplicacion>,
        sender_eventos_llamada_a_telefono: Sender<EventoLlamada>,
        mutex_panel_estadisticas: &mut Arc<Mutex<PanelEstadisticas>>,
    ) -> Result<(), ErrorLlamada> {
        // Le digo a la Sesion que corte la llamada
        let mut option_sesion = mutex_sesion
            .lock()
            .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;

        // Si no hay sesion, entonces el otro peer me corto porque yo corte primero
        // En ese caso, no hay que hacer nada al recibir este evento.
        if option_sesion.is_none() {
            return Ok(());
        }

        let sesion = option_sesion.as_mut().ok_or(ErrorLlamada::ErrorInterno(
            "Se quiso obtener una sesion y no hay ninguna".to_string(),
        ))?;
        sesion.cortar_llamada()?;

        // Apago el microfono
        microfono.mutear()?;
        microfono.borrar_receptor()?;

        // Reinicio la camara
        // Si falla no rompe el programa, porque podria cerrarse el programa si hay algun fallo con
        // el lente de la camara, que puede ser reemplazado por otro sin problema.
        if let Err(e) = Self::reiniciar_camara(camara) {
            eprintln!("Error reiniciando camara: {e}");
            eprintln!("La llamada se cerrara igualmente. Revise el estado de la camara");
        }

        // Le digo al panel de estadisticas que ya no hay fuente de datos
        let mut panel_estadisticas = mutex_panel_estadisticas.lock().map_err(|_| {
            ErrorLlamada::ErrorInterno(
                "Fallo obteniendo el lock del panel de estadisticas".to_string(),
            )
        })?;
        panel_estadisticas.borrar_fuente()?;

        // Establezco que no hay ninguna sesion activa
        *option_sesion = None;

        // Informo a telefono para que permita volver a llamar
        sender_eventos_llamada_a_telefono
            .send(EventoLlamada::LlamadaFinalizada)
            .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;

        // Informo a la aplicacion que se cerro la Aplicacion
        let evento_ocurrido = EventoAplicacion::LlamadaFinalizada;
        sender_eventos_aplicacion
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|e| ErrorLlamada::ErrorInterno(format!("{e}")))?;
        Ok(())
    }

    /// Agrega dos capturadores a la camara: un sender al reproductor local, y un sender del que se recibiran los frames
    /// a ser transmitidos. Luego, enciende la camara
    fn iniciar_camara(
        camara: &mut Box<dyn Camara + 'static>,
        sender_a_reproductor_local: &Sender<Frame>,
        sender_frames_camara: Sender<Frame>,
    ) -> Result<(), ErrorLlamada> {
        camara.agregar_capturador(sender_frames_camara)?;
        camara.agregar_capturador(sender_a_reproductor_local.clone())?;
        camara.encender()?;

        Ok(())
    }

    /// Apaga la camara y reinicia su lista de capturadores.
    fn reiniciar_camara(camara: &mut Box<dyn Camara + 'static>) -> Result<(), ErrorLlamada> {
        camara.apagar()?;
        camara.reiniciar_capturadores()?;
        Ok(())
    }

    /// Escucha eventos del canal SCTP (ofertas de archivo, aceptaciones, rechazos, datos)
    /// y los reenvía como [EventoLlamada] para que sean procesados en el loop principal.
    fn escuchar_eventos_sctp(
        rx: Receiver<EventoSctp>,
        sender_eventos_llamada: Sender<EventoLlamada>,
    ) {
        let mut nombre_archivo: Option<String> = None;
        let mut tamanio_esperado: u64 = 0;
        let mut buffer: Vec<u8> = Vec::new();

        for evento in rx {
            let evento_llamada = match evento {
                EventoSctp::OfertaArchivo { nombre, tamanio } => {
                    nombre_archivo = Some(nombre.clone());
                    tamanio_esperado = tamanio;
                    buffer = Vec::with_capacity(tamanio as usize);
                    EventoLlamada::OfertaArchivoRecibida { nombre, tamanio }
                }
                EventoSctp::ArchivoAceptado => EventoLlamada::ArchivoAceptadoPorPeer,
                EventoSctp::ArchivoRechazado => {
                    nombre_archivo = None;
                    tamanio_esperado = 0;
                    buffer = Vec::new();
                    EventoLlamada::ArchivoRechazadoPorPeer
                }
                EventoSctp::DatosArchivo(datos) => {
                    buffer.extend_from_slice(&datos);
                    eprintln!(
                        "[Llamada] Acumulados {}/{} bytes de archivo",
                        buffer.len(),
                        tamanio_esperado
                    );
                    continue;
                }
                EventoSctp::FinArchivo => {
                    let nombre = match nombre_archivo.take() {
                        Some(n) => n,
                        None => "archivo".to_string(),
                    };
                    let datos = Bytes::from(std::mem::take(&mut buffer));
                    tamanio_esperado = 0;
                    eprintln!(
                        "[Llamada] Archivo recibido completo: {} ({} bytes)",
                        nombre,
                        datos.len()
                    );
                    EventoLlamada::ArchivoRecibido { nombre, datos }
                }
            };
            if sender_eventos_llamada.send(evento_llamada).is_err() {
                break;
            }
        }
    }

    fn actualizar_panel_estadisticas(
        mutex_panel_estadisticas: &mut Arc<Mutex<PanelEstadisticas>>,
        sesion_nueva: &mut SesionRTP,
    ) -> Result<(), ErrorLlamada> {
        let receiver_estadisticas = sesion_nueva.obtener_receiver_estadisticas()?;

        let mut panel_estadisticas = mutex_panel_estadisticas.lock().map_err(|_| {
            ErrorLlamada::ErrorInterno(
                "Error obteniendo lock del panel de estadisticas".to_string(),
            )
        })?;

        panel_estadisticas.cambiar_fuente(receiver_estadisticas)?;
        Ok(())
    }

    fn obtener_estadisticas_panel(&mut self) -> Result<EstadisticasReceiver, ErrorLlamada> {
        let mut panel_estadisticas = self.mutex_panel_estadisticas.lock().map_err(|_| {
            ErrorLlamada::ErrorEnPanelDeEstadisticas(
                "Fallo obteniendo el lock del panel de estadisticas".to_string(),
            )
        })?;
        let estadisticas = panel_estadisticas.estadisticas()?;
        Ok(estadisticas)
    }
}

impl From<ErrorReproductor> for ErrorLlamada {
    fn from(error: ErrorReproductor) -> Self {
        Self::ErrorConReproductor(format!("Error en el reproductor: {error}"))
    }
}

impl From<ErrorSesionRTP> for ErrorLlamada {
    fn from(error: ErrorSesionRTP) -> Self {
        ErrorLlamada::ErrorEnSesionRTP(format!("Error en SesionRTP: {error}"))
    }
}

impl From<ErrorCamara> for ErrorLlamada {
    fn from(error: ErrorCamara) -> Self {
        ErrorLlamada::ErrorEnCamara(format!("Error en camara: {error}"))
    }
}

impl From<ErrorMicrofono> for ErrorLlamada {
    fn from(error: ErrorMicrofono) -> Self {
        ErrorLlamada::ErrorEnMicrofono(format!("Error en microfono: {error}"))
    }
}

impl From<ErrorReproductorAudio> for ErrorLlamada {
    fn from(error: ErrorReproductorAudio) -> Self {
        ErrorLlamada::ErrorConReproductorDeAudio(format!("{error}"))
    }
}

impl From<ErrorPanelEstadisticas> for ErrorLlamada {
    fn from(error: ErrorPanelEstadisticas) -> Self {
        ErrorLlamada::ErrorEnPanelDeEstadisticas(format!("{error}"))
    }
}

#[cfg(test)]
use crate::llamada::camara_mock::CamaraMock;
#[cfg(test)]
use crate::llamada::microfono::MicrofonoMock;
#[cfg(test)]
use crate::reproductor::reproductor_mock::ReproductorMock;
#[cfg(test)]
use crate::sesion_rtp::socket_udp::MockSocketUdp;
#[cfg(test)]
use std::time::Duration;

#[test]
fn test_01_si_me_piden_enviar_proximo_frame_se_lo_pido_al_reproductor() {
    let (sender_a_reproductor, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();
    let mutex_reproductor = Arc::new(Mutex::new(ReproductorMock::default()));
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));
    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor)),
        reproductor_audio,
        sender_a_reproductor,
        sender_audio,
    );
    let microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(microfono),
    };
    let mut llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    llamada
        .enviar_proximo_frame()
        .expect("Deberia poderse pedir el proximo frame");

    let reproductor = mutex_reproductor.lock().unwrap();
    assert!(reproductor.se_pidio_proximo_frame());

    let _ = receiver_evento_aplicacion.try_recv();
    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    let _ = receiver_telefono.try_recv();
}

#[test]
fn test_02_si_me_piden_proximo_frame_envio_a_la_aplicacion_el_frame_recibido_del_reproductor() {
    let (sender_a_reproductor, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();
    let mutex_reproductor = Arc::new(Mutex::new(ReproductorMock::default()));
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));
    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor)),
        reproductor_audio,
        sender_a_reproductor,
        sender_audio,
    );
    let microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(microfono),
    };
    let mut llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    llamada
        .enviar_proximo_frame()
        .expect("Deberia poderse pedir el proximo");
    let reproductor = mutex_reproductor.lock().unwrap();
    let evento_recibido = receiver_evento_aplicacion
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    assert!(reproductor.se_pidio_proximo_frame());
    if let EventoInternoAplicacion::EventoObservable(evento) = evento_recibido {
        assert!(matches!(evento, EventoAplicacion::NuevoFrame(_)));
    } else {
        panic!("Deberia recibirse un evento NuevoFrame")
    }

    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    let _ = receiver_telefono.try_recv();
}

#[test]
fn test_03_se_agrega_capturador_de_sesion_al_iniciar_la_llamada() {
    let (sender_a_reproductor, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();
    let mutex_reproductor = Arc::new(Mutex::new(ReproductorMock::default()));
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));
    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let (_, receiver_srtp_rx) = mpsc::channel::<Vec<u8>>();
    let mutex_camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor)),
        reproductor_audio,
        sender_a_reproductor,
        sender_audio,
    );
    let microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(Arc::clone(&mutex_camara)),
        microfono: Box::new(microfono),
    };
    let _ = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    let socket_rtp_propio_dummy = Box::new(MockSocketUdp::new(vec![], vec![]));
    let socket_rtcp_propio_dummy = Box::new(MockSocketUdp::new(vec![], vec![]));
    let conexion = ConexionP2P::new(
        socket_rtp_propio_dummy,
        socket_rtcp_propio_dummy,
        "localhost:3000".to_string(),
        "localhost:3001".to_string(),
        None,
        None,
        Some(receiver_srtp_rx),
    );
    sender_evento_llamada
        .send(EventoLlamada::LlamadaIniciada(Box::new(conexion)))
        .expect("Se deberia enviar el evento LlamadaIniciada");
    thread::sleep(Duration::from_millis(500));

    let camara = mutex_camara.lock().unwrap();
    assert!(camara.se_agrego_capturador());

    let _ = receiver_evento_aplicacion.try_recv();
    let _ = receiver_telefono.try_recv();
}

#[test]
fn test_04_se_le_pide_a_reproductor_local_un_frame_al_pedir_nuevo_frame() {
    // Creo reproductor para sesion
    let mutex_reproductor_sesion = Arc::new(Mutex::new(ReproductorMock::default()));
    // Creo reproductor local
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));

    // Obtengo sender para reproductores
    let (sender_a_reproductor_sesion, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();

    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor_sesion)),
        reproductor_audio,
        sender_a_reproductor_sesion,
        sender_audio,
    );
    let microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(microfono),
    };
    let mut llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    llamada
        .enviar_proximo_frame()
        .expect("Deberia poderse pedir el proximo frame");

    let reproductor_local = mutex_reproductor_local.lock().unwrap();
    let reproductor_sesion = mutex_reproductor_sesion.lock().unwrap();
    assert!(reproductor_local.se_pidio_proximo_frame());
    assert!(reproductor_sesion.se_pidio_proximo_frame());

    let _ = receiver_evento_aplicacion.try_recv();
    let _ = receiver_telefono.try_recv();
    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_05_se_le_envia_a_aplicacion_el_nuevo_frame_local_al_pedir_frame_a_llamada() {
    // Creo reproductor para sesion
    let mutex_reproductor_sesion = Arc::new(Mutex::new(ReproductorMock::default()));
    // Creo reproductor local
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));

    // Obtengo sender para reproductores
    let (sender_a_reproductor_sesion, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();

    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor_sesion)),
        reproductor_audio,
        sender_a_reproductor_sesion,
        sender_audio,
    );
    let microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(microfono),
    };
    let mut llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    llamada
        .enviar_proximo_frame()
        .expect("Deberia poderse pedir el proximo frame");

    let primer_evento_recibido = receiver_evento_aplicacion
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();
    let segundo_evento_recibido = receiver_evento_aplicacion
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    if let EventoInternoAplicacion::EventoObservable(evento) = primer_evento_recibido {
        assert!(matches!(evento, EventoAplicacion::NuevoFrame(_)))
    }
    if let EventoInternoAplicacion::EventoObservable(evento) = segundo_evento_recibido {
        assert!(matches!(evento, EventoAplicacion::NuevoFrameLocal(_)))
    }

    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    let _ = receiver_telefono.try_recv();
}

#[test]
fn test_06_se_le_pide_a_microfono_que_se_mutee_al_recibir_mutear() {
    // Creo reproductor para sesion
    let mutex_reproductor_sesion = Arc::new(Mutex::new(ReproductorMock::default()));
    // Creo reproductor local
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));

    // Obtengo sender para reproductores
    let (sender_a_reproductor_sesion, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();

    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor_sesion)),
        reproductor_audio,
        sender_a_reproductor_sesion,
        sender_audio,
    );
    let mutex_microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(Arc::clone(&mutex_microfono)),
    };
    let mut llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    llamada
        .mutear_microfono()
        .expect("Se deberia poder mutear el microfono");
    thread::sleep(Duration::from_millis(500));

    let microfono = mutex_microfono.lock().unwrap();
    assert!(microfono.esta_muteado());

    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    let _ = receiver_telefono.try_recv();
    let _ = receiver_evento_aplicacion.try_recv();
}

#[test]
fn test_07_se_le_pide_a_microfono_que_se_desmutee_al_recibir_desmutear() {
    // Creo reproductor para sesion
    let mutex_reproductor_sesion = Arc::new(Mutex::new(ReproductorMock::default()));
    // Creo reproductor local
    let mutex_reproductor_local = Arc::new(Mutex::new(ReproductorMock::default()));

    // Obtengo sender para reproductores
    let (sender_a_reproductor_sesion, _) = mpsc::channel();
    let (sender_a_telefono, receiver_telefono) = mpsc::channel();
    let (sender_a_reproductor_local, _) = mpsc::channel();

    let (sender_evento_aplicacion, receiver_evento_aplicacion) = mpsc::channel();
    let (sender_evento_llamada, receiver_evento_llamada) = mpsc::channel();
    let camara = Arc::new(Mutex::new(CamaraMock::default()));
    let (sender_audio, receiver_audio) = mpsc::channel();
    let reproductor_audio = ReproductorAudioDummy::iniciar_reproduccion(receiver_audio).unwrap();
    let reproductores = ReproductoresLlamada::new(
        Box::new(Arc::clone(&mutex_reproductor_local)),
        sender_a_reproductor_local,
        Box::new(Arc::clone(&mutex_reproductor_sesion)),
        reproductor_audio,
        sender_a_reproductor_sesion,
        sender_audio,
    );
    let mutex_microfono = Arc::new(Mutex::new(MicrofonoMock::default()));
    let dispositivos = DispositivosEntrada {
        camara: Box::new(camara),
        microfono: Box::new(Arc::clone(&mutex_microfono)),
    };
    let mut llamada = LlamadaRTP::new(
        dispositivos,
        reproductores,
        sender_evento_aplicacion,
        sender_evento_llamada.clone(),
        receiver_evento_llamada,
        sender_a_telefono,
        Logger::dummy_logger(),
    )
    .unwrap();

    llamada
        .mutear_microfono()
        .expect("Se deberia poder mutear el microfono");
    llamada
        .desmutear_microfono()
        .expect("Se deberia poder desmutear el microfono");
    thread::sleep(Duration::from_millis(500));

    let microfono = mutex_microfono.lock().unwrap();
    assert!(!microfono.esta_muteado());

    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    let _ = receiver_telefono.try_recv();
    let _ = receiver_evento_aplicacion.try_recv();
}
