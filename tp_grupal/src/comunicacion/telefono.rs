//! # Telefono - Creación de llamadas
//!
//! [Telefono] representa un objeto capaz de llamar, atender llamadas y rechazar llamadas.
//!
//! Será de interes la implementación [TelefonoConComunicador], que debera ser la usada por [Aplicacion](crate::aplicacion::Aplicacion).

use crate::aplicacion::EventoAplicacion;
use crate::logger::Logger;
use crate::{
    aplicacion::{EventoInternoAplicacion, EventoLlamada},
    comunicacion::comunicador::{Comunicador, ErrorComunicador},
    protocolos::pca::mensaje::MensajePCA,
};
use std::fmt::Display;
use std::sync::{Arc, Mutex};
use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

#[derive(Debug, Clone)]
pub enum ErrorTelefono {
    ErroConComunicador(String),
    ErrorInterno,
    AccionInvalida(String),
}

impl Display for ErrorTelefono {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorTelefono::AccionInvalida(mensaje) => {
                f.write_str(&format!("Accion invalida: {mensaje}"))
            }
            ErrorTelefono::ErroConComunicador(mensaje) => {
                f.write_str(&format!("Error en telefono con comunicador: {mensaje}"))
            }
            ErrorTelefono::ErrorInterno => f.write_str("Error interno en el telefono"),
        }
    }
}

pub enum EstadoTelefono {
    SinLlamadasEntrantes,
    RecibiendoLlamada,
    CreandoLlamada,
    EnLlamada,
}

pub trait Telefono {
    /// Llama al usuario cuyo nombre de usuario es el especificado
    fn llamar(&mut self, usuario: &str) -> Result<(), ErrorTelefono>;
    /// Atiende la llamada que se estaba recibiendo.
    ///
    /// PRE: Al momento de ejecutarse el metodo, hay una llamada que no ha sido respondida
    fn atender_llamada(&mut self) -> Result<(), ErrorTelefono>;
    /// Rechaza la llamada que se estaba recibiendo.
    ///
    /// PRE: Al momento de ejecutarse el metodo, hay una llamada que no ha sido respondida
    fn rechazar_llamada(&mut self) -> Result<(), ErrorTelefono>;
}

/// [TelefonoConComunicador] representa un telefono que puede llamar y atender o rechazar llamadas, comunicandose con el servidor mediante el [Comunicador]
/// especificado.
///
/// Sus responsabilidades serán:
/// - Notificar al servidor cuando sea necesario de las operaciones que se deseen hacer (llamar, rechazar o atender).
/// - Asegurarse que no se intenten hacer estas operaciones de una forma incorrecta (llamar a muchos usuarios al mismo tiempo, atender sin haber sido llamado, etc).
/// - Notificar a la Aplicacion cuando ocurra un cambio de estado en la creación de la Llamada, que sea recibido desde el exterior. Ejemplo: la llamada finalizo,
///   y ya se puede llamar y recibir llamadas nuevamente
pub struct TelefonoConComunicador {
    comunicador: Box<dyn Comunicador>,
    mutex_estado_llamada: Arc<Mutex<EstadoTelefono>>,
}

impl Telefono for TelefonoConComunicador {
    fn atender_llamada(&mut self) -> Result<(), ErrorTelefono> {
        let estado_llamada = self
            .mutex_estado_llamada
            .lock()
            .map_err(|_| ErrorTelefono::ErrorInterno)?;

        if !matches!(*estado_llamada, EstadoTelefono::RecibiendoLlamada) {
            return Err(ErrorTelefono::AccionInvalida(String::from(
                "ERROR: No hay ninguna llamada que atender!",
            )));
        }

        self.comunicador.enviar_mensaje(&MensajePCA::Aceptar)?;

        Ok(())
    }

    fn rechazar_llamada(&mut self) -> Result<(), ErrorTelefono> {
        let mut estado_llamada = self
            .mutex_estado_llamada
            .lock()
            .map_err(|_| ErrorTelefono::ErrorInterno)?;

        if !matches!(*estado_llamada, EstadoTelefono::RecibiendoLlamada) {
            return Err(ErrorTelefono::AccionInvalida(String::from(
                "ERROR: No hay ninguna llamada que rechazar!",
            )));
        }

        self.comunicador.enviar_mensaje(&MensajePCA::Rechazo)?;

        *estado_llamada = EstadoTelefono::SinLlamadasEntrantes;

        Ok(())
    }

    fn llamar(&mut self, usuario: &str) -> Result<(), ErrorTelefono> {
        let mut estado_llamada = self
            .mutex_estado_llamada
            .lock()
            .map_err(|_| ErrorTelefono::ErrorInterno)?;

        if !matches!(*estado_llamada, EstadoTelefono::SinLlamadasEntrantes) {
            return Err(ErrorTelefono::AccionInvalida(String::from(
                "ERROR: No se pueden llamar dos usuarios al mismo tiempo",
            )));
        }

        self.comunicador
            .enviar_mensaje(&MensajePCA::Llamar(usuario.to_string()))?;

        *estado_llamada = EstadoTelefono::CreandoLlamada;

        Ok(())
    }
}

impl TelefonoConComunicador {
    /// Crea un [TelefonoConComunicador] que se comunicara mediante el [Comunicador] especificado. Recibe un [Sender<EventoAplicacion>] para
    /// avisar al notificador de [Aplicacion](crate::aplicacion::Aplicacion) de los eventos externos recibidos. Tambien recibe un [Receiver<EventoLlamada>] por
    /// el cual la llamada podra informarle de eventos ocurridos en la llamada que deban actualizar el estado del [TelefonoConComunicador] (ejemplo: se informa que
    /// la llamada finalizo, entonces el telefono debe poder llamar y recibir llamadas nuevamente).
    pub fn new(
        mut comunicador: Box<dyn Comunicador>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        receiver_eventos_llamada: Receiver<EventoLlamada>,
        logger: Logger,
    ) -> Result<TelefonoConComunicador, ErrorTelefono> {
        let logger_para_hilo_comunicador = logger.clone();
        let logger_para_hilo_llamada_finalizada = logger.clone();

        let comunicador_para_usar_sincronicamente = comunicador.crear_companiero()?;
        let comunicador_para_eventos_llamada = comunicador.crear_companiero()?;
        let estado_llamada = EstadoTelefono::SinLlamadasEntrantes;
        let mutex_estado_llamada = Arc::new(Mutex::new(estado_llamada));

        let clon_mutex_estado_llamada = Arc::clone(&mutex_estado_llamada);
        let sender_eventos_para_mensajes_comunicador = sender_eventos.clone();
        thread::spawn(move || {
            Self::escuchar_mensajes_de_comunicador(
                comunicador,
                sender_eventos_para_mensajes_comunicador,
                clon_mutex_estado_llamada,
                logger_para_hilo_comunicador,
            );
        });

        let clon_mutex_estado_llamada_para_eventos = Arc::clone(&mutex_estado_llamada);
        let sender_eventos_para_mensajes_llamada = sender_eventos.clone();
        thread::spawn(move || {
            Self::escuchar_eventos_llamada(
                clon_mutex_estado_llamada_para_eventos,
                receiver_eventos_llamada,
                comunicador_para_eventos_llamada,
                sender_eventos_para_mensajes_llamada,
                logger_para_hilo_llamada_finalizada,
            );
        });

        Ok(TelefonoConComunicador {
            comunicador: comunicador_para_usar_sincronicamente,
            mutex_estado_llamada,
        })
    }

    fn escuchar_eventos_llamada(
        mutex_estado_llamada: Arc<Mutex<EstadoTelefono>>,
        receiver_eventos_llamada: Receiver<EventoLlamada>,
        comunicador: Box<dyn Comunicador>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        logger: Logger,
    ) {
        if let Err(error) = Self::_escuchar_eventos_llamada(
            mutex_estado_llamada,
            receiver_eventos_llamada,
            comunicador,
            sender_eventos,
            logger.clone(),
        ) {
            logger.error(&format!("{error}"), "Telefono");
        }
    }

    fn _escuchar_eventos_llamada(
        mutex_estado_llamada: Arc<Mutex<EstadoTelefono>>,
        receiver_eventos_llamada: Receiver<EventoLlamada>,
        mut comunicador: Box<dyn Comunicador>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        logger: Logger,
    ) -> Result<(), ErrorTelefono> {
        loop {
            let _ = receiver_eventos_llamada
                .recv()
                .map_err(|_| ErrorTelefono::ErrorInterno)?;

            logger.info("Llamada nos informa que la llamada termino. Cambiando estado del telefono a EstadoTelefono::SinLlamadasEntrantes", "Telefono");

            comunicador.enviar_mensaje(&MensajePCA::Cortar)?;

            Self::enviar_evento_aplicacion(&sender_eventos, EventoAplicacion::LlamadaFinalizada)?;
            Self::actualizar_estado_telefono(
                &mutex_estado_llamada,
                EstadoTelefono::SinLlamadasEntrantes,
            )?;
        }
    }

    fn escuchar_mensajes_de_comunicador(
        comunicador: Box<dyn Comunicador>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        mutex_estado_llamada: Arc<Mutex<EstadoTelefono>>,
        logger: Logger,
    ) {
        if let Err(error) = Self::_escuchar_mensajes_de_comunicador(
            comunicador,
            sender_eventos,
            mutex_estado_llamada,
            logger.clone(),
        ) {
            logger.error(&format!("{error}"), "Telefono");
        }
    }

    fn _escuchar_mensajes_de_comunicador(
        mut comunicador: Box<dyn Comunicador>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        mutex_estado_llamada: Arc<Mutex<EstadoTelefono>>,
        logger: Logger,
    ) -> Result<(), ErrorTelefono> {
        loop {
            let mensaje = comunicador.escuchar_mensaje()?;

            let mut evento_a_enviar = None;
            let mut estado_nuevo = None;

            match mensaje {
                MensajePCA::Llamando(usuario) => {
                    logger.info(
                        &format!("Servidor informa que {} nos esta llamando", usuario),
                        "Telefono",
                    );
                    evento_a_enviar = Some(EventoAplicacion::RecibiendoLlamada(usuario));
                    estado_nuevo = Some(EstadoTelefono::RecibiendoLlamada)
                }
                MensajePCA::PedirOffer => {
                    logger.info("Servidor nos solicita offer. Cambiando estado de Telefono a EstadoTelefono::EnLlamada", "Telefono");
                    evento_a_enviar = Some(EventoAplicacion::LlamadaIniciando);
                    estado_nuevo = Some(EstadoTelefono::EnLlamada);
                }
                MensajePCA::Offer(_) => {
                    logger.info("Servidor manda un offer. Cambiando estado de Telefono a EstadoTelefono::EnLlamada", "Telefono");
                    evento_a_enviar = Some(EventoAplicacion::LlamadaIniciando);
                    estado_nuevo = Some(EstadoTelefono::EnLlamada);
                }
                MensajePCA::Rechazo => {
                    logger.info("Servidor manda un offer. Cambiando estado de Telefono a EstadoTelefono::SinLlamadasEntrantes", "Telefono");
                    evento_a_enviar = Some(EventoAplicacion::LlamadaRechazada);
                    estado_nuevo = Some(EstadoTelefono::SinLlamadasEntrantes);
                }
                _ => {}
            };

            if let Some(evento) = evento_a_enviar {
                Self::enviar_evento_aplicacion(&sender_eventos, evento)?;
            };

            if let Some(estado) = estado_nuevo {
                Self::actualizar_estado_telefono(&mutex_estado_llamada, estado)?;
            }
        }
    }

    fn actualizar_estado_telefono(
        mutex_estado_llamada: &Arc<Mutex<EstadoTelefono>>,
        estado: EstadoTelefono,
    ) -> Result<(), ErrorTelefono> {
        let mut estado_telefono = mutex_estado_llamada
            .lock()
            .map_err(|_| ErrorTelefono::ErrorInterno)?;
        *estado_telefono = estado;
        Ok(())
    }

    fn enviar_evento_aplicacion(
        sender_eventos: &Sender<EventoInternoAplicacion>,
        evento: EventoAplicacion,
    ) -> Result<(), ErrorTelefono> {
        sender_eventos
            .send(EventoInternoAplicacion::EventoObservable(evento))
            .map_err(|_| ErrorTelefono::ErrorInterno)?;
        Ok(())
    }
}

impl From<ErrorComunicador> for ErrorTelefono {
    fn from(error: ErrorComunicador) -> Self {
        ErrorTelefono::ErroConComunicador(error.to_string())
    }
}

#[cfg(test)]
use crate::comunicacion::comunicador_fake_y_mock::ComunicadorFakeYMock;

#[cfg(test)]
use std::{
    sync::mpsc::{self},
    time::Duration,
};

#[cfg(test)]
use crate::comunicacion::comunicador_mock::ComunicadorMockYStud;

#[test]
fn test_01_se_envia_mensaje_llamar_al_comunicador_al_intentar_llamar() {
    let mutex_comunicador = Arc::new(Mutex::new(ComunicadorMockYStud::new(MensajePCA::Ok)));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(Arc::clone(&mutex_comunicador)),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").expect("Fallo al llamar");

    let comunicador = mutex_comunicador.lock().unwrap();
    assert!(comunicador.cantidad_mensajes_enviados() == 1);
    assert!(comunicador.se_envio_mensaje(MensajePCA::Llamar("Juan".to_string())));

    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_02_se_informa_a_aplicacion_al_recibir_llamada() {
    // Ojo con este test!! Al Telefono le mandamos un comunicador, y ese comunicador DEBE SER EL MISMO
    // que este escuchando mensajes activamente. El telefono se tiene que quedar con el companiero si o si.
    // Si se hace al reves (es decir, el telefono se queda con el comunicador original y le da el hermano al thread que escucha)
    // no va a pasar el test por como funciona ComunicadorFake. Lo que pasa es que el comunicador original deberia retransmitirle
    // el mensaje al hermano, pero si este comunicador original solo se usa al Llamar, no va a escuchar el mensaje hasta no ejecutar ese metodo y
    // nunca se lo va a retransmitir a su compañero. En consecuencia, si se hace al reves, este test falla por Timeout recibiendo del channel.
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        mutex_mensajes_enviados,
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    sender_mensajes
        .send(MensajePCA::Llamando("Juan".to_string()))
        .unwrap();
    let mensaje_recibido_en_aplicacion = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    if let EventoInternoAplicacion::EventoObservable(evento) = mensaje_recibido_en_aplicacion {
        assert!(matches!(evento, EventoAplicacion::RecibiendoLlamada(_)))
    } else {
        panic!("Se deberia informar un evento observable.")
    }
    // Esto debe quedar aca para que telefono no se vaya de scope y eso haga que el comunicador principal se vaya de scope
    let _ = telefono.llamar("Juan");

    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_03_se_envia_mensaje_rechazo_al_comunicador_al_rechazar_llamada() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    sender_mensajes
        .send(MensajePCA::Llamando("Juan".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    telefono
        .rechazar_llamada()
        .expect("Deberia poderse cortar la llamada");

    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(mensajes_enviados.contains(&MensajePCA::Rechazo));
    let _ = receiver_eventos.try_recv();

    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_04_se_envia_mensaje_aceptar_al_comunicador_al_atender_llamada() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    sender_mensajes
        .send(MensajePCA::Llamando("Juan".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    telefono
        .atender_llamada()
        .expect("Deberia poderse atender la llamada");

    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(mensajes_enviados.contains(&MensajePCA::Aceptar));
    let _ = receiver_eventos.try_recv();

    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_05_no_se_puede_atender_una_llamada_si_no_se_recibio_ninguna() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    let resultado_atender = telefono.atender_llamada();

    assert!(resultado_atender.is_err());

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_06_se_avisa_aplicacion_al_iniciar_llamada_porque_me_atendieron() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::PedirOffer)
        .unwrap();
    let mensaje_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    if let EventoInternoAplicacion::EventoObservable(evento) = mensaje_recibido {
        assert!(matches!(evento, EventoAplicacion::LlamadaIniciando));
    } else {
        panic!("Deberia enviarse un evento observable")
    }

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_07_se_avisa_a_aplicacion_al_cerrar_llamada_porque_me_rechazaron() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Rechazo)
        .unwrap();
    let mensaje_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    if let EventoInternoAplicacion::EventoObservable(evento) = mensaje_recibido {
        assert!(matches!(evento, EventoAplicacion::LlamadaRechazada));
    } else {
        panic!("Deberia enviarse un evento observable")
    }

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_08_no_se_puede_llamar_a_otro_usuario_si_se_llamo_a_uno_y_todavia_no_respondio() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    let resultado_segundo_llamado = telefono.llamar("Pedro");

    assert!(resultado_segundo_llamado.is_err());

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_09_no_se_puede_llamar_a_otro_usuario_si_me_llamaron_y_todavia_no_respondi() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Llamando("Juan".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    let resultado_segundo_llamado = telefono.llamar("Pedro");

    assert!(resultado_segundo_llamado.is_err());

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_10_puedo_volver_a_llamar_si_rechazo_la_llamada() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Llamando("Juan".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    telefono.rechazar_llamada().unwrap();
    let resultado_segundo_llamado = telefono.llamar("Pedro");

    assert!(resultado_segundo_llamado.is_ok());

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_11_puedo_volver_a_llamar_si_me_rechazan_una_llamada() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Rechazo)
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    let resultado_segundo_llamado = telefono.llamar("Pedro");

    assert!(resultado_segundo_llamado.is_ok());

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_12_se_envia_mensaje_cortar_al_comunicador_cuando_se_corta_la_llamada() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::PedirOffer)
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(mensajes_enviados.contains(&MensajePCA::Cortar));

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_13_se_avisa_a_aplicacion_al_cortar_llamada() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::PedirOffer)
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();

    // Deberia recibir primero el evento LlamadaIniciada y luego LlamadaFinalizada
    let _ = receiver_eventos.recv().unwrap();
    let evento_interno_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    if let EventoInternoAplicacion::EventoObservable(evento) = evento_interno_recibido {
        assert!(matches!(evento, EventoAplicacion::LlamadaFinalizada));
    } else {
        panic!("Deberia recibirse un evento LlamadaFinalizada");
    }

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_14_si_se_termina_la_llamada_puedo_iniciar_una_nueva() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    telefono.llamar("Juan").unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::PedirOffer)
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let resultado_segunda_llamada = telefono.llamar("Pedro");

    assert!(resultado_segunda_llamada.is_ok());
    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}

#[test]
fn test_15_se_avisa_a_aplicacion_que_inicio_llamada_si_recibo_un_offer() {
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    let mut telefono = TelefonoConComunicador::new(
        Box::new(comunicador),
        sender_eventos,
        receiver_eventos_llamada,
        Logger::dummy_logger(),
    )
    .unwrap();

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Llamando("Juan".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    telefono.atender_llamada().unwrap();
    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Offer("asd".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let _ = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();
    let segundo_evento_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();
    if let EventoInternoAplicacion::EventoObservable(evento) = segundo_evento_recibido {
        assert!(matches!(evento, EventoAplicacion::LlamadaIniciando));
    } else {
        panic!("Deberia recibirse el evento LlamadaIniciada")
    }

    let _ = receiver_eventos.try_recv();
    sender_eventos_llamada
        .send(EventoLlamada::LlamadaFinalizada)
        .unwrap();
}
