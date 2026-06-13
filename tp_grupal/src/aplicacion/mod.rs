//! # Aplicación - Orquestador de los componentes de la aplicación
//!
//!
//!
//! La idea es que tenga como colaboradores internos a cada uno de los componentes del programa (objetos del
//! dominio de problema). Teniendo esos colaboradores internos, sus responsabilidades son:
//!
//! - Actuar de *orquestador* entre los diferentes componentes para realizar cada acción de nuestro programa.
//!
//! - Funcionar como un "objeto observable"; quien desee escuchar actualizaciones de la Aplicacion en tiempo real puede
//!   llamar al metodo [Aplicacion::suscribir()] y enviar un [Sender<EventoAplicacion>]. Cada actualización que ocurra en el estado de la aplicación
//!   (o en sus componentes) se informara a cada uno de los suscriptores enviando un [`EventoAplicacion`].
//!
//! Lo mas típico va a ser que "las vistas de la aplicacion" sean quien se suscriban a la [Aplicacion], para actualizar el contenido
//! que se muestra cada vez que cambia el estado.

use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{
    comunicacion::lobby::ErrorLobby,
    creacion_llamada::{ConexionP2P, mediador_de_conexiones::MediadorDeConexionesP2P},
    llamada::{
        DispositivosEntrada,
        administrador_de_camaras::{
            AdministradorDeCamaras, ErrorAdministradorDeCamaras, FuenteDeVideo,
        },
        camara::Camara,
        microfono_default::MicrofonoDefault,
    },
    reproductor::{
        reproductor_audio::ReproductorAudio, reproductor_audio_default::ReproductorAudioDefault,
    },
};
use crate::{comunicacion::lobby::Lobby, llamada::ReproductoresLlamada};
use crate::{
    comunicacion::{
        comunicador::{Comunicador, ErrorComunicador},
        lobby::LobbyConComunicador,
        telefono::{ErrorTelefono, Telefono, TelefonoConComunicador},
    },
    config_room_rtc::ConfigRoomRTC,
    creacion_llamada::{
        ErrorCreadorDeConexion, creador_de_conexion_encriptada::CreadorDeConexionEncriptada,
    },
    entrada::{ErrorRecepcion, Recepcion},
    logger::Logger,
    protocolos::pca::usuario::UsuarioPCA,
    reproductor::reproductor_sin_decoder::ReproductorSinDecoder,
};
use crate::{
    llamada::{
        ErrorLlamada, Llamada, LlamadaRTP, camara::CamaraGenerica,
        creador_lente_nokwha::CreadorDeLenteNokwha,
    },
    reproductor::reproductor_rtp::ReproductorDeSesionRTP,
    sesion_rtp::sesion::EstadisticasReceiver,
};
use bytes::Bytes;
use eframe::egui::ColorImage;

#[derive(Debug)]
/// Errores ejecutando operaciones de [Aplicacion]
pub enum ErrorAplicacion {
    ErrorEnRecepcionConElComunicador(String),
    ErrorInformandoASuscribers,
    ErrorConElServidor(String),
    ErrorRecibido(String),
    ErrorTelefono(String),
    ErrorEnLobby(String),
    ErrorEnLlamada(String),
    ErrorEnElAdministradorDeCamaras(String),
    ErrorEnCreadorDeConexion(String),
}

#[derive(Clone, Debug)]
/// Representa un evento **observable** en la aplicación. Cualquier usuario de la aplicación deberia
/// conocer si estos eventos sucedieron.
pub enum EventoAplicacion {
    /// El intento de registro se concreto exitosamente
    RegistroExitoso,
    /// La sesion se inicio correctamente
    SesionIniciada,
    /// Hay una nueva lista de usuarios en la aplicacion
    UsuariosNuevos(Vec<UsuarioPCA>),
    /// Hubo un error al registrarse en la aplicación
    ErrorDeRegistro(String),
    /// Hubo un error iniciando sesión en la aplicación.
    ErrorIniciandoSesion(String),
    /// Se esta recibiendo una llamada de otro usuario
    RecibiendoLlamada(String),
    /// Se esta iniciando una llamada con otro peer
    LlamadaIniciando,
    /// El otro peer rechazo la llamada
    LlamadaRechazada,
    /// Alguno de los dos peers termino la llamada en curso
    LlamadaFinalizada,
    /// Se esta llamando a otro peer
    EnviandoLlamada(String),
    /// Hubo un error al intentar realizar una operación durante la creación de una llamada
    ErrorCreandoLlamada(String),
    /// Se rechazo una llamada externa entrante
    LlamadaExternaRechazada,
    /// Se inicio la llamada con otro peer
    LlamadaIniciada,
    /// Hay un nuevo frame disponible para ser mostrado
    NuevoFrame(ColorImage),
    /// Hay un nuevo frame local disponible para ser mostrado
    NuevoFrameLocal(ColorImage),
    /// Hay un nuevo listado de camaras disponibles para la llamada
    NuevaListaDeCamarasDisponibles(Vec<String>),
    /// Cambio la camara que se usara para videollamadas
    NuevaCamaraEnUso(String),
    /// Se informa que se cerro la sesion
    SesionCerrada,
    /// Se informa que hubo un error cerrando la sesion
    ErrorCerrandoSesion,
    /// Se muteo el microfono y no se esta enviando audio
    MicrofonoMuteado,
    /// Se desmuteo el microfono y se esta enviando audio
    MicrofonoDesmuteado,
    /// Hay nuevas estadisticas sobre la llamada que se pueden mostrar
    NuevasEstadisticas(Box<EstadisticasReceiver>),
    /// El peer remoto ofrece enviarnos un archivo
    RecibendoOfertaArchivo { nombre: String, tamanio: u64 },
    /// El peer remoto acepto nuestra oferta de archivo
    ArchivoAceptadoPorPeer,
    /// El peer remoto rechazo nuestra oferta de archivo
    ArchivoRechazadoPorPeer,
    /// Aceptamos la oferta de archivo recibida
    ArchivoRecibido { nombre: String, ruta: PathBuf },
}

/// Representa un evento **interno** en la Aplicacion, es decir, un evento que ocurre
/// en alguno de los componentes
#[derive(Debug)]
pub enum EventoInternoAplicacion {
    /// Hay una nueva lista de usuarios en la aplicacion
    NuevoSuscriptor(Sender<EventoAplicacion>),
    EventoObservable(EventoAplicacion),
}

#[derive(Debug)]
pub enum EventoLlamada {
    LlamadaIniciada(Box<ConexionP2P>),
    LlamadaFinalizada,
    NuevaCamara(Box<dyn Camara>),
    MutearMicrofono,
    DesmutearMicrofono,
    EnviarArchivo(PathBuf),
    OfertaArchivoRecibida { nombre: String, tamanio: u64 },
    ArchivoAceptadoPorPeer,
    ArchivoRechazadoPorPeer,
    AceptarArchivo,
    RechazarArchivo,
    ArchivoRecibido { nombre: String, datos: Bytes },
}

/// Representa una estructura capaz de realizar todas las operaciones de nuestra aplicación.
///
/// En la [documentación del modulo](self) se explica el funcionamiento esperado.
pub struct Aplicacion {
    recepcion: Recepcion,
    sender_eventos_internos: Sender<EventoInternoAplicacion>,
    telefono: Box<dyn Telefono>,
    llamada: Box<dyn Llamada>,
    lobby: Box<dyn Lobby>,
    administrador_de_camaras: AdministradorDeCamaras,
}

impl Aplicacion {
    /// Crea una aplicación que se comunicara con el servidor mediante el comunicador recibido.
    pub fn new(
        mut comunicador: Box<dyn Comunicador>,
        config: ConfigRoomRTC,
        logger: Logger,
    ) -> Result<Aplicacion, ErrorAplicacion> {
        // Creo un comunicador para cada componente que lo necesite
        let comunicador_recepcion = comunicador.crear_companiero()?;
        let comunicador_telefono = comunicador.crear_companiero()?;
        let comunicador_creador_conexiones = comunicador.crear_companiero()?;

        // Creo la recepcion
        let recepcion = Recepcion::new(comunicador_recepcion);

        // Creo el lobby
        let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
        let lobby =
            LobbyConComunicador::new(comunicador, sender_eventos_internos.clone(), logger.clone());

        // Creo el telefono
        let (sender_eventos_llamada_a_telefono, receiver_eventos_desde_llamada) = mpsc::channel();
        let telefono = TelefonoConComunicador::new(
            comunicador_telefono,
            sender_eventos_internos.clone(),
            receiver_eventos_desde_llamada,
            logger.clone(),
        )?;

        // Creo mediador de conexiones
        let creador_conexion =
            CreadorDeConexionEncriptada::iniciar_creador(config, logger.clone())?;
        let (sender_eventos_a_llamada, receiver_eventos_llamada) = mpsc::channel();
        MediadorDeConexionesP2P::iniciar(
            comunicador_creador_conexiones,
            Box::new(creador_conexion),
            sender_eventos_internos.clone(),
            sender_eventos_a_llamada.clone(),
        );

        // Creo camara
        let creador_lente_nokwha = CreadorDeLenteNokwha::new(0); //todo poder actualizar indice camara
        let camara = CamaraGenerica::new(Box::new(creador_lente_nokwha));

        // Creo reproductores
        let mut reproductor = ReproductorDeSesionRTP::new().map_err(ErrorLlamada::from)?;
        let sender_a_reproductor = reproductor.obtener_sender_frames();
        let (sender_frames_locales, receiver_frames_locales) = mpsc::channel();
        let reproductor_local = ReproductorSinDecoder::new(receiver_frames_locales);
        let (sender_audio, receiver_audio) = mpsc::channel();
        let reproductor_audio = ReproductorAudioDefault::iniciar_reproduccion(receiver_audio)
            .map_err(ErrorLlamada::from)?;
        let reproductores = ReproductoresLlamada::new(
            Box::new(reproductor_local),
            sender_frames_locales,
            Box::new(reproductor),
            reproductor_audio,
            sender_a_reproductor,
            sender_audio,
        );

        // Creo microfono
        let microfono = MicrofonoDefault::<i16>::new().map_err(ErrorLlamada::from)?;

        // Creo dispositivos
        let dispositivos = DispositivosEntrada {
            camara: Box::new(camara),
            microfono: Box::new(microfono),
        };

        // Creo llamada
        let llamada = LlamadaRTP::new(
            dispositivos,
            reproductores,
            sender_eventos_internos.clone(),
            sender_eventos_a_llamada,
            receiver_eventos_llamada,
            sender_eventos_llamada_a_telefono,
            logger,
        )?;

        // Creo administrador de camaras
        let administrador_de_camaras = AdministradorDeCamaras::new(vec![FuenteDeVideo::Nokwha]);

        thread::spawn(move || {
            Self::escuchar_eventos_internos(receiver_eventos_internos);
        });

        Ok(Aplicacion {
            recepcion,
            sender_eventos_internos,
            lobby: Box::new(lobby),
            telefono: Box::new(telefono),
            llamada: Box::new(llamada),
            administrador_de_camaras,
        })
    }

    /// PRE: Los componentes que funcionan en threads aparte son creados previo a llamar a este metodo,
    /// y notifican sus eventos al receiver_eventos_internos recibido como parametro.
    pub fn con_componentes(
        recepcion: Recepcion,
        lobby: Box<dyn Lobby>,
        telefono: Box<dyn Telefono>,
        sender_eventos_internos: Sender<EventoInternoAplicacion>,
        receiver_eventos_internos: Receiver<EventoInternoAplicacion>,
        llamada: Box<dyn Llamada>,
    ) -> Result<Aplicacion, ErrorAplicacion> {
        let administrador_de_camaras = AdministradorDeCamaras::new(vec![FuenteDeVideo::FuenteTest]);

        thread::spawn(move || {
            Self::escuchar_eventos_internos(receiver_eventos_internos);
        });

        Ok(Aplicacion {
            recepcion,
            llamada,
            sender_eventos_internos,
            telefono,
            lobby,
            administrador_de_camaras,
        })
    }

    /// Inicia sesión en la aplicación con el usuario y contrasenia especificados.
    ///
    /// En caso de que la sesión se inicie correctamente, envia notifica a los suscriptores de la Aplicación de los eventos
    /// [EventoAplicacion::SesionIniciada] y [EventoAplicacion::UsuariosNuevos].
    ///
    /// En caso de que falle al iniciar sesion, se envia el evento [EventoAplicacion::ErrorIniciandoSesion] a los suscriptores,
    /// indicando que error ocurrio.
    pub fn iniciar_sesion(
        &mut self,
        usuario: &str,
        contrasenia: &str,
    ) -> Result<(), ErrorAplicacion> {
        let resultado_inicio_sesion = self.recepcion.iniciar_sesion(usuario, contrasenia);

        let evento_ocurrido = match resultado_inicio_sesion {
            Ok(_) => EventoAplicacion::SesionIniciada,
            Err(error) => EventoAplicacion::ErrorIniciandoSesion(format!("{error}")),
        };

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Se registra en la aplicación en la aplicación con el usuario y contrasenia especificados.
    ///
    /// En caso de que la sesión se registre correctamente, envia notifica a los suscriptores de la Aplicación el evento
    /// [EventoAplicacion::RegistroExitoso].
    ///
    /// En caso de que falle al registrarse, se envia el evento [EventoAplicacion::ErrorDeRegistro] a los suscriptores,
    /// indicando que error ocurrio
    pub fn registrarse(&mut self, usuario: &str, contrasenia: &str) -> Result<(), ErrorAplicacion> {
        let resultado_registro = self.recepcion.registrarse(usuario, contrasenia);

        let evento_ocurrido = match resultado_registro {
            Ok(_) => EventoAplicacion::RegistroExitoso,
            Err(error) => EventoAplicacion::ErrorIniciandoSesion(format!("{error}")),
        };

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Cierra la sesion actual.
    ///
    /// En caso de que la sesion se cierre correctamente, notifica a los suscriptores de Aplicacion el evento
    /// [EventoAplicacion::SesionCerrada]
    ///
    /// En caso de fallo, se envia el evento [EventoAplicacion::ErrorCerrandoSesion] a los suscriptores, indicando
    /// el error ocurrido
    pub fn cerrar_sesion(&mut self) -> Result<(), ErrorAplicacion> {
        let resultado_cerrar_sesion = self.recepcion.cerrar_sesion();

        let evento_ocurrido = if resultado_cerrar_sesion.is_ok() {
            EventoAplicacion::SesionCerrada
        } else {
            EventoAplicacion::ErrorCerrandoSesion
        };

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Solicita la lista de usuarios actualizados.
    ///
    /// En caso de que se pueda responder con una lista de usuarios valida, esta se notificara mediante el evento
    /// [EventoAplicacion::UsuariosNuevos].
    ///
    /// En caso de no haber una lista, simplemente no envia ningun evento.
    pub fn usuarios(&mut self) -> Result<(), ErrorAplicacion> {
        let usuarios = self.lobby.usuarios()?;

        let evento_ocurrido = EventoAplicacion::UsuariosNuevos(usuarios);

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Llama al usuario con el nombre especificado. Devuelve el resultado de la operación, que puede ser fallido.
    pub fn llamar(&mut self, usuario: &str) -> Result<(), ErrorAplicacion> {
        let resultado_llamar = self.telefono.llamar(usuario);

        let evento_ocurrido = match resultado_llamar {
            Ok(_) => EventoAplicacion::EnviandoLlamada(usuario.to_string()),
            Err(error) => EventoAplicacion::ErrorCreandoLlamada(format!("{error}")),
        };

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Rechaza la llamada que se estaba recibiendo. Devuelve el resultado de la operación, que puede ser fallido.
    ///
    /// Si no se estaba recibiendo una llamada, el resultado sera un error.
    pub fn rechazar_llamada(&mut self) -> Result<(), ErrorAplicacion> {
        let resultado_rechazar = self.telefono.rechazar_llamada();

        let evento_ocurrido = match resultado_rechazar {
            Ok(_) => EventoAplicacion::LlamadaExternaRechazada,
            Err(error) => {
                EventoAplicacion::ErrorCreandoLlamada(format!("Error rechazando llamada: {error}"))
            }
        };

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Atiende la llamada que se estaba recibiendo. Devuelve el resultado de la operación, que puede ser fallido.
    ///
    /// Si no se estaba recibiendo una llamada, el resultado sera un error.
    pub fn atender_llamada(&mut self) -> Result<(), ErrorAplicacion> {
        let resultado_atender = self.telefono.atender_llamada();

        let evento_ocurrido = match resultado_atender {
            Ok(_) => None, //El evento se va a enviar cuando el servidor me pida el offer, indicando que se atendio correctamente la llamada
            Err(error) => Some(EventoAplicacion::ErrorCreandoLlamada(format!(
                "Error rechazando llamada: {error}"
            ))),
        };

        if let Some(evento) = evento_ocurrido {
            self.sender_eventos_internos
                .send(EventoInternoAplicacion::EventoObservable(evento))
                .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;
        };

        Ok(())
    }

    /// Solicita que se envie un nuevo frame de la camara
    pub fn enviar_nuevo_frame(&mut self) -> Result<(), ErrorAplicacion> {
        self.llamada.enviar_proximo_frame()?;
        Ok(())
    }

    /// Corta la llamada en curso. Si no hay llamada en curso, devuelve un error
    pub fn cortar_llamada(&mut self) -> Result<(), ErrorAplicacion> {
        self.llamada.cortar_llamada()?;

        Ok(())
    }

    /// Lista las camaras disponibles para la llamada
    pub fn camaras_disponibles(&mut self) -> Result<(), ErrorAplicacion> {
        let camaras_disponibles = self.administrador_de_camaras.camaras_disponibles()?;

        let evento_ocurrido = EventoAplicacion::NuevaListaDeCamarasDisponibles(camaras_disponibles);

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Cambia la camara que se usara en la llamada por una especifica
    pub fn cambiar_camara(&mut self, nombre_nueva_camara: &str) -> Result<(), ErrorAplicacion> {
        let nueva_camara = self
            .administrador_de_camaras
            .crear_camara(nombre_nueva_camara)?;

        self.llamada.cambiar_camara(nueva_camara)?;

        let evento_ocurrido = EventoAplicacion::NuevaCamaraEnUso(nombre_nueva_camara.to_string());

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Mutea el microfono de la llamada
    pub fn mutear_microfono(&mut self) -> Result<(), ErrorAplicacion> {
        self.llamada.mutear_microfono()?;

        let evento_ocurrido = EventoAplicacion::MicrofonoMuteado;

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Mutea el microfono de la llamada
    pub fn desmutear_microfono(&mut self) -> Result<(), ErrorAplicacion> {
        self.llamada.desmutear_microfono()?;

        let evento_ocurrido = EventoAplicacion::MicrofonoDesmuteado;

        self.sender_eventos_internos
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Agrega el [Sender<EventoAplicacion>] recibido a la lista de suscribers de la aplicacion. En adelante,
    /// cada vez que ocurra algún cambio en la aplicacion se enviara por el channel el [EventoAplicacion] respectivo.
    pub fn suscribir(
        &mut self,
        sender_suscriber: Sender<EventoAplicacion>,
    ) -> Result<(), ErrorAplicacion> {
        self.sender_eventos_internos
            .send(EventoInternoAplicacion::NuevoSuscriptor(sender_suscriber))
            .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

        Ok(())
    }

    /// Envía un archivo al peer remoto. El resultado de la operación puede ser fallido si ocurre algo en la llamada,
    /// o si ocurre un error durante el proceso de envío.
    pub fn enviar_archivo(&mut self, path: PathBuf) -> Result<(), ErrorAplicacion> {
        self.llamada
            .enviar_archivo(path)
            .map_err(|e| ErrorAplicacion::ErrorEnLlamada(format!("{e}")))
    }

    /// Acepta la oferta de archivo recibida del peer remoto. El resultado de la operación puede ser fallido si pasa algo inesperado en la llamada
    pub fn aceptar_archivo(&mut self) -> Result<(), ErrorAplicacion> {
        self.llamada
            .aceptar_archivo()
            .map_err(|e| ErrorAplicacion::ErrorEnLlamada(format!("{e}")))
    }

    /// Rechaza la oferta de archivo recibida del peer remoto. El resultado de la operación puede ser fallido si pasa algo inesperado en la llamada
    pub fn rechazar_archivo(&mut self) -> Result<(), ErrorAplicacion> {
        self.llamada
            .rechazar_archivo()
            .map_err(|e| ErrorAplicacion::ErrorEnLlamada(format!("{e}")))
    }

    /// Notifica a los suscriptores del [EventoAplicacion] indicado.
    fn notificar_a_suscriptores(
        suscriptores: &mut Vec<Sender<EventoAplicacion>>,
        evento_a_enviar: EventoAplicacion,
    ) -> Result<(), ErrorAplicacion> {
        for sender_suscriber in suscriptores {
            sender_suscriber
                .send(evento_a_enviar.clone())
                .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;
        }

        Ok(())
    }

    fn escuchar_eventos_internos(receiver_eventos_internos: Receiver<EventoInternoAplicacion>) {
        if let Err(error) = Self::_escuchar_eventos_internos(receiver_eventos_internos) {
            dbg!(error);
        }
    }

    fn _escuchar_eventos_internos(
        receiver_eventos_internos: Receiver<EventoInternoAplicacion>,
    ) -> Result<(), ErrorAplicacion> {
        let mut suscriptores = vec![];

        loop {
            let evento_interno = receiver_eventos_internos
                .recv()
                .map_err(|_| ErrorAplicacion::ErrorInformandoASuscribers)?;

            match evento_interno {
                EventoInternoAplicacion::NuevoSuscriptor(sender) => suscriptores.push(sender),
                EventoInternoAplicacion::EventoObservable(evento) => {
                    Self::notificar_a_suscriptores(&mut suscriptores, evento)?
                }
            }
        }
    }
}

impl From<ErrorRecepcion> for ErrorAplicacion {
    fn from(error: ErrorRecepcion) -> Self {
        match error {
            ErrorRecepcion::ErrorConElComunicador(error) => {
                ErrorAplicacion::ErrorEnRecepcionConElComunicador(error)
            }
            ErrorRecepcion::ErrorConElServidor(error) => ErrorAplicacion::ErrorConElServidor(error),
            ErrorRecepcion::ErrorRecibido(error) => ErrorAplicacion::ErrorRecibido(error),
            ErrorRecepcion::ErrorSesionNoIniciada => {
                ErrorAplicacion::ErrorEnRecepcionConElComunicador(
                    "No se puede cerrar sesion porque no se inicio".to_string(),
                )
            }
        }
    }
}

impl From<ErrorComunicador> for ErrorAplicacion {
    fn from(error: ErrorComunicador) -> Self {
        match error {
            ErrorComunicador::ErrorDeConexion(e) => ErrorAplicacion::ErrorConElServidor(format!(
                "Ocurrio un error de conexion con el servidor: {e}"
            )),
            ErrorComunicador::ErrorEnElComunicador => ErrorAplicacion::ErrorConElServidor(
                "Ocurrio un error con el comunicador".to_string(),
            ),
            ErrorComunicador::ErrorRecibido(error) => ErrorAplicacion::ErrorRecibido(error),
        }
    }
}

impl From<ErrorTelefono> for ErrorAplicacion {
    fn from(error: ErrorTelefono) -> Self {
        match error {
            ErrorTelefono::AccionInvalida(mensaje) => {
                ErrorAplicacion::ErrorTelefono(format!("Accion invalida: {mensaje}"))
            }
            ErrorTelefono::ErroConComunicador(error_str) => {
                ErrorAplicacion::ErrorTelefono(error_str)
            }
            ErrorTelefono::ErrorInterno => {
                ErrorAplicacion::ErrorTelefono("Error inesperado en el telefono".to_string())
            }
        }
    }
}

impl From<ErrorCreadorDeConexion> for ErrorAplicacion {
    fn from(error: ErrorCreadorDeConexion) -> Self {
        ErrorAplicacion::ErrorEnCreadorDeConexion(format!("{error}"))
    }
}

impl From<ErrorLlamada> for ErrorAplicacion {
    fn from(error: ErrorLlamada) -> Self {
        ErrorAplicacion::ErrorEnLlamada(format!("Error en llamada: {error}"))
    }
}

impl From<ErrorLobby> for ErrorAplicacion {
    fn from(error: ErrorLobby) -> Self {
        match error {
            ErrorLobby::ErrorComunicandoAAplicacion => ErrorAplicacion::ErrorEnLobby(
                "Error en lobby: Error comunicando a aplicacion".to_string(),
            ),
            ErrorLobby::ErrorObteniendoLockUsuarios => ErrorAplicacion::ErrorEnLobby(
                "Error en lobby: Error obteniendo lock de usuarios".to_string(),
            ),
            ErrorLobby::ErrorConElComunicador(error) => {
                ErrorAplicacion::ErrorEnLobby(format!("Error en lobby: {error}"))
            }
        }
    }
}

impl From<ErrorAdministradorDeCamaras> for ErrorAplicacion {
    fn from(error: ErrorAdministradorDeCamaras) -> Self {
        ErrorAplicacion::ErrorEnElAdministradorDeCamaras(format!(
            "Error en el administrador de camaras: {error}"
        ))
    }
}

#[cfg(test)]
use crate::comunicacion::{
    comunicador_fake::ComunicadorFake, comunicador_mock::ComunicadorMockYStud,
    comunicador_stub::ComunicadorStub, lobby::LobbyDummy, telefono_dummy::TelefonoDummy,
    telefono_mock::TelefonoMock, telefono_stub::TelefonoStub,
};

#[cfg(test)]
use crate::protocolos::pca::mensaje::MensajePCA;

#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
use crate::protocolos::pca::estado::EstadoUsuarioPCA;

#[cfg(test)]
use std::sync::{Arc, Mutex};

#[cfg(test)]
use std::{cell::RefCell, rc::Rc};

#[cfg(test)]
use crate::creacion_llamada::creador_de_conexion_mock::CreadorDeConexionMock;

#[cfg(test)]
use crate::llamada::llamada_mock::LlamadaMock;

#[test]
fn test_01_inicia_sesion_al_recibir_mensaje_usuarios() {
    let comunicador = ComunicadorStub::new(MensajePCA::Usuarios(vec![]));
    let recepcion = Recepcion::new(Box::new(comunicador));
    let lobby = LobbyDummy::default();
    let telefono = TelefonoDummy::default();
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    let respuesta = aplicacion.iniciar_sesion("USUARIO", "CONTRASENIA");

    assert!(respuesta.is_ok());
}

#[test]
fn test_02_se_registra_al_recibir_registrado() {
    let comunicador = ComunicadorStub::new(MensajePCA::Registrado);
    let recepcion = Recepcion::new(Box::new(comunicador));
    let lobby = LobbyDummy::default();
    let telefono = TelefonoDummy::default();
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    let respuesta = aplicacion.registrarse("USUARIO", "CONTRASENIA");

    assert!(respuesta.is_ok());
}

#[test]
fn test_03_se_avisa_a_suscribers_al_registrarse() {
    let comunicador = ComunicadorStub::new(MensajePCA::Registrado);
    let recepcion = Recepcion::new(Box::new(comunicador));
    let lobby = LobbyDummy::default();
    let telefono = TelefonoDummy::default();
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();
    let (tx_evento, rx_evento) = mpsc::channel();

    aplicacion.suscribir(tx_evento).unwrap();
    aplicacion
        .registrarse("USUARIO", "CONTRASENIA")
        .expect("Deberia registrarse correctamente");

    // El primer evento que recibo debería ser RegistroExitoso
    let evento_recibido = rx_evento
        .recv_timeout(Duration::from_millis(1500))
        .expect("Se deberia recibir un EventoAplicacion::RegistroExitoso");
    assert!(matches!(evento_recibido, EventoAplicacion::RegistroExitoso));

    // No deberia recibir otro evento aparte de RegistroExitoso
    let sig_evento_recibido = rx_evento.try_recv();
    assert!(sig_evento_recibido.is_err());
}

#[test]
fn test_04_se_avisa_a_suscribers_al_iniciar_sesion() {
    // Creo comunicador
    let comunicador = ComunicadorStub::new(MensajePCA::Usuarios(vec![]));

    // Creo channels entre componentes
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();

    // Creo componentes
    let recepcion = Recepcion::new(Box::new(comunicador));
    let lobby = LobbyDummy::default();
    let telefono = TelefonoDummy::default();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));

    // Creo aplicacion
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    // Me suscribo a la aplicacion
    let (tx_evento, rx_evento) = mpsc::channel();
    aplicacion.suscribir(tx_evento).unwrap();

    aplicacion
        .iniciar_sesion("USUARIO", "CONTRASENIA")
        .expect("Deberia registrarse correctamente");

    // El primer evento que recibo debería ser SesionIniciada
    let evento_recibido = rx_evento
        .recv_timeout(Duration::from_millis(1500))
        .expect("Se deberia recibir un EventoAplicacion::SesionIniciada");
    assert!(matches!(evento_recibido, EventoAplicacion::SesionIniciada));

    // No deberia recibir otro evento aparte de SesionIniciada
    let sig_evento_recibido = rx_evento.try_recv();
    assert!(sig_evento_recibido.is_err());
}

#[test]
fn test_05_se_avisa_a_suscribers_al_recibir_usuarios_en_el_lobby() {
    // Creo comunicadores
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();

    // Creo channels entre componentes
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();

    // Creo componentes
    let lobby = LobbyConComunicador::new(
        Box::new(comunicador),
        sender_eventos_internos.clone(),
        Logger::dummy_logger(),
    );
    let telefono = Rc::new(RefCell::new(TelefonoMock::default()));
    let recepcion = Recepcion::new(comunicador_recepcion);
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));

    // Creo aplicacion
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(Rc::clone(&telefono)),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    // Suscribo un observer a la aplicacion
    let (tx_evento, rx_evento) = mpsc::channel();
    aplicacion.suscribir(tx_evento).unwrap();

    let usuario = UsuarioPCA::new("Juan".to_string(), EstadoUsuarioPCA::Disponible);
    sender_mensajes
        .send(MensajePCA::Usuarios(vec![usuario.clone()]))
        .unwrap();

    let evento_aplicacion_recibido = rx_evento.recv_timeout(Duration::from_millis(1500)).unwrap();

    if let EventoAplicacion::UsuariosNuevos(usuarios) = evento_aplicacion_recibido {
        assert!(usuarios == vec![usuario]);
    } else {
        panic!("Se deberia informar que hay usuarios nuevos")
    }
}

#[test]
fn test_06_se_avisa_al_telefono_al_realizar_llamada_y_telefono_avisa_a_comunicador() {
    // Creo comunicadores
    let mut mutex_comunicador = Arc::new(Mutex::new(ComunicadorMockYStud::new(MensajePCA::Ok)));
    let comunicador_recepcion = mutex_comunicador.crear_companiero().unwrap();

    // Creo channels entre componentes
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let (sender_evento_llamada, receiver_eventos_llamada) = mpsc::channel();

    // Creo componentes
    let lobby = LobbyConComunicador::new(
        Box::new(Arc::clone(&mutex_comunicador)),
        sender_eventos_internos.clone(),
        Logger::dummy_logger(),
    );
    let telefono = TelefonoConComunicador::new(
        Box::new(Arc::clone(&mutex_comunicador)),
        sender_eventos_internos.clone(),
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();
    let recepcion = Recepcion::new(comunicador_recepcion);
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));

    // Creo aplicacion
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    aplicacion.llamar("Juan").unwrap();

    let comunicador = mutex_comunicador.lock().unwrap();
    assert!(comunicador.cantidad_mensajes_enviados() == 1);
    assert!(comunicador.se_envio_mensaje(MensajePCA::Llamar("Juan".to_string())));

    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_07_se_avisa_a_suscribers_al_enviar_llamada() {
    // Creo comunicadores
    let mut mutex_comunicador = Arc::new(Mutex::new(ComunicadorMockYStud::new(MensajePCA::Ok)));
    let comunicador_recepcion = mutex_comunicador.crear_companiero().unwrap();

    // Creo channels entre componentes
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let (sender_evento_llamada, receiver_eventos_llamada) = mpsc::channel();

    // Creo componentes
    let lobby = LobbyConComunicador::new(
        Box::new(Arc::clone(&mutex_comunicador)),
        sender_eventos_internos.clone(),
        Logger::dummy_logger(),
    );
    let telefono = TelefonoConComunicador::new(
        Box::new(Arc::clone(&mutex_comunicador)),
        sender_eventos_internos.clone(),
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();
    let recepcion = Recepcion::new(comunicador_recepcion);
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));

    // Creo aplicacion
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    let (sender_eventos_aplicacion, receiver_eventos_aplicacion) = mpsc::channel();
    aplicacion.suscribir(sender_eventos_aplicacion).unwrap();
    aplicacion.llamar("Juan").unwrap();

    let evento_recibido = receiver_eventos_aplicacion
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();
    if let EventoAplicacion::EnviandoLlamada(usuario) = evento_recibido {
        assert!(usuario.eq("Juan"));
    } else {
        panic!("Deberia recibirse el evento EnviandoLlamada");
    }

    sender_evento_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_08_se_avisa_a_telefono_que_se_desea_rechazar_llamada() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();
    let lobby = LobbyDummy::default();
    let telefono = Rc::new(RefCell::new(TelefonoMock::default()));
    let recepcion = Recepcion::new(comunicador_recepcion);
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(Rc::clone(&telefono)),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    aplicacion.rechazar_llamada().unwrap();

    assert!(telefono.borrow().se_recibio_rechazar());
}

#[test]
fn test_09_se_avisa_a_telefono_que_se_desea_atender_la_llamada() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();
    let lobby = LobbyDummy::default();
    let telefono = Rc::new(RefCell::new(TelefonoMock::default()));
    let recepcion = Recepcion::new(comunicador_recepcion);
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(Rc::clone(&telefono)),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    aplicacion.atender_llamada().unwrap();

    assert!(telefono.borrow().se_recibio_atender());
}

#[test]
fn test_10_se_avisa_a_suscribers_al_rechazar_llamada() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();
    let lobby = LobbyDummy::default();
    let telefono = Rc::new(RefCell::new(TelefonoMock::default()));
    let recepcion = Recepcion::new(comunicador_recepcion);
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(Rc::clone(&telefono)),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    let (sender_eventos, receiver_eventos) = mpsc::channel();
    aplicacion.suscribir(sender_eventos).unwrap();

    aplicacion.rechazar_llamada().unwrap();
    let evento_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    assert!(matches!(
        evento_recibido,
        EventoAplicacion::LlamadaExternaRechazada
    ));
}

#[test]
fn test_11_se_avisa_a_suscribers_si_hubo_error_al_rechazar_llamada() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();
    let lobby = LobbyDummy::default();
    let error_de_telefono = ErrorTelefono::ErrorInterno;
    let telefono = TelefonoStub::new(Err(error_de_telefono));
    let recepcion = Recepcion::new(comunicador_recepcion);
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    let (sender_eventos, receiver_eventos) = mpsc::channel();
    aplicacion.suscribir(sender_eventos).unwrap();

    aplicacion.rechazar_llamada().unwrap();
    let evento_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    assert!(matches!(
        evento_recibido,
        EventoAplicacion::ErrorCreandoLlamada(_)
    ));
}

#[test]
fn test_12_se_avisa_a_suscribers_si_hubo_error_al_atender_llamada() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();
    let lobby = LobbyDummy::default();
    let error_de_telefono = ErrorTelefono::ErrorInterno;
    let telefono = TelefonoStub::new(Err(error_de_telefono));
    let recepcion = Recepcion::new(comunicador_recepcion);
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    let (sender_eventos, receiver_eventos) = mpsc::channel();
    aplicacion.suscribir(sender_eventos).unwrap();

    aplicacion.atender_llamada().unwrap();
    let evento_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    assert!(matches!(
        evento_recibido,
        EventoAplicacion::ErrorCreandoLlamada(_)
    ));
}

#[test]
fn test_13_se_avisa_a_suscribers_que_la_llamada_inicio() {
    // Creo comunicadores para cada componente
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mut comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let comunicador_recepcion = comunicador.crear_companiero().unwrap();

    // Creo channels entre componentes
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();

    // Creo lobby
    let lobby = LobbyDummy::default();

    // Creo telefono
    let telefono = TelefonoStub::new(Err(ErrorTelefono::AccionInvalida("".to_string())));

    // Creo recepcion
    let recepcion = Recepcion::new(comunicador_recepcion);

    // Creo llamada
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));

    // Creo creador de conexiones
    let creador_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(creador_conexion),
        sender_eventos_internos.clone(),
        sender_eventos_llamada,
    );

    // Inicializo aplicacion con cada componente
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(llamada_mock),
    )
    .unwrap();

    // El test se suscribe como observer de la aplicacion.
    let (sender_eventos, receiver_eventos_aplicacion) = mpsc::channel();
    aplicacion.suscribir(sender_eventos).unwrap();
    thread::sleep(Duration::from_millis(500));

    // Envio el mensaje Offer, que lo va a recibir el comunicador
    sender_mensajes
        .send(MensajePCA::Offer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(1000));

    let evento_llamada = receiver_eventos_llamada
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();
    let evento_aplicacion = receiver_eventos_aplicacion
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    assert!(matches!(evento_llamada, EventoLlamada::LlamadaIniciada(..)));
    assert!(matches!(
        evento_aplicacion,
        EventoAplicacion::LlamadaIniciada
    ));
}

#[test]
fn test_14_se_le_pide_a_llamada_que_envie_nuevo_frame_cuando_nos_lo_piden() {
    let comunicador = ComunicadorStub::new(MensajePCA::Registrado);
    let recepcion = Recepcion::new(Box::new(comunicador));
    let lobby = LobbyDummy::default();
    let telefono = TelefonoDummy::default();
    let (sender_eventos_internos, receiver_eventos_internos) = mpsc::channel();
    let llamada_mock = Rc::new(RefCell::new(LlamadaMock::default()));
    let mut aplicacion = Aplicacion::con_componentes(
        recepcion,
        Box::new(lobby),
        Box::new(telefono),
        sender_eventos_internos,
        receiver_eventos_internos,
        Box::new(Rc::clone(&llamada_mock)),
    )
    .unwrap();

    let respuesta_enviar_nuevo_frame = aplicacion.enviar_nuevo_frame();
    let llamada = llamada_mock.borrow();

    assert!(respuesta_enviar_nuevo_frame.is_ok());
    assert!(llamada.se_pidio_enviar_frame());
}
