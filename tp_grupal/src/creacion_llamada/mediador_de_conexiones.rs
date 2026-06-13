use std::sync::mpsc::Sender;
#[cfg(test)]
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
#[cfg(test)]
use std::time::Duration;

use crate::aplicacion::{EventoAplicacion, EventoInternoAplicacion, EventoLlamada};
use crate::comunicacion::comunicador::Comunicador;
#[cfg(test)]
use crate::comunicacion::comunicador_fake::ComunicadorFake;
#[cfg(test)]
use crate::comunicacion::comunicador_fake_y_mock::ComunicadorFakeYMock;
#[cfg(test)]
use crate::creacion_llamada::creador_de_conexion_mock::CreadorDeConexionMock;
use crate::creacion_llamada::{CreadorDeConexionP2P, ErrorCreadorDeConexion};

use crate::protocolos::pca::mensaje::MensajePCA;

/// El struct [MediadorDeConexionesP2P] provee la funcion asociada [MediadorDeConexionesP2P::iniciar]. En esa funcion se inicializa
/// el thread que va a encargarse del intercambio de offers y answers. En la documentación de esa función hay mas detalles sobre su funcionamiento.
pub struct MediadorDeConexionesP2P {}

impl MediadorDeConexionesP2P {
    /// Inicia el servicio de intercambio de offers y answers con un servidor de signaling. Para ello, recibe:
    ///
    /// - Un comunicador, que sera usado para recibir mensajes del servidor y responderlos.
    /// - Un struct que cumpla el trait [CreadorDeConexionP2P], que sera quien se encargue de generar offers y answers.
    /// - Un [Sender<EventoInternoAplicacion>], para comunicar los eventos que ocurran; en particular, se enviara un evento
    ///   cuando termine el checkeo de candidatos y se concrete la conexion entre peers
    /// - Un [Sender<EventoLlamada>], mediante el cual se le informara a Llamada que la conexión ya inicio. El evento enviado es [EventoLlamada::LlamadaIniciada],
    ///   por el cual ademas se envia una [crate::creacion_llamada::ConexionP2P] para que Llamada sepa los sockets por los cuales enviar y recibir contenido.
    pub fn iniciar(
        comunicador: Box<dyn Comunicador>,
        creador_de_conexion: Box<dyn CreadorDeConexionP2P>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        sender_eventos_llamada: Sender<EventoLlamada>,
    ) {
        thread::spawn(|| {
            Self::iniciar_comunicacion_con_signaling(
                comunicador,
                creador_de_conexion,
                sender_eventos,
                sender_eventos_llamada,
            );
        });
    }

    fn iniciar_comunicacion_con_signaling(
        comunicador: Box<dyn Comunicador>,
        creador_de_conexion: Box<dyn CreadorDeConexionP2P>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        sender_eventos_llamada: Sender<EventoLlamada>,
    ) {
        if let Err(error) = Self::_iniciar_comunicacion_con_signaling(
            comunicador,
            creador_de_conexion,
            sender_eventos,
            sender_eventos_llamada,
        ) {
            eprintln!("{error}");
        }
    }

    fn _iniciar_comunicacion_con_signaling(
        mut comunicador: Box<dyn Comunicador>,
        mut creador_de_conexion: Box<dyn CreadorDeConexionP2P>,
        sender_eventos: Sender<EventoInternoAplicacion>,
        sender_eventos_llamada: Sender<EventoLlamada>,
    ) -> Result<(), ErrorCreadorDeConexion> {
        loop {
            let mensaje = comunicador
                .escuchar_mensaje()
                .map_err(|_| ErrorCreadorDeConexion::ErrorConComunicador)?;

            let mut mensaje_a_devolver = None;
            let mut se_puede_iniciar_conexion = false;

            Self::procesar_mensaje(
                &mut creador_de_conexion,
                mensaje,
                &mut mensaje_a_devolver,
                &mut se_puede_iniciar_conexion,
            )?;

            if let Some(mensaje) = mensaje_a_devolver {
                comunicador
                    .enviar_mensaje(&mensaje)
                    .map_err(|_| ErrorCreadorDeConexion::ErrorConComunicador)?;
            }

            if se_puede_iniciar_conexion {
                creador_de_conexion.conectar()?;
                Self::notificar_llamada_iniciada(
                    &sender_eventos,
                    &sender_eventos_llamada,
                    &mut creador_de_conexion,
                )?;
            }
        }
    }

    /// De acuerdo al mensaje recibido del servidor de signaling, ejecuta la acción necesaria en el creador de conexiones.
    ///
    /// POST:
    /// - `mensaje_a_devolver` contiene un Option, que contendra un [Some] con el mensaje a responder al servidor de signaling, o [None] si no hay nada que devolver.
    /// - `se_puede_iniciar_conexion` contiene un booleano indicando si ya se realizo el intercambio necesario para iniciar la negociacion de candidatos.
    fn procesar_mensaje(
        creador_de_conexion: &mut Box<dyn CreadorDeConexionP2P>,
        mensaje: MensajePCA,
        mensaje_a_devolver: &mut Option<MensajePCA>,
        se_puede_iniciar_conexion: &mut bool,
    ) -> Result<(), ErrorCreadorDeConexion> {
        match mensaje {
            MensajePCA::PedirOffer => {
                let offer = creador_de_conexion.generar_offer()?;
                *mensaje_a_devolver = Some(MensajePCA::Offer(offer));
            }
            MensajePCA::Offer(offer) => {
                let answer = creador_de_conexion.generar_answer(&offer)?;
                *mensaje_a_devolver = Some(MensajePCA::Answer(answer));
                *se_puede_iniciar_conexion = true;
            }
            MensajePCA::Answer(answer) => {
                creador_de_conexion.recibir_answer(&answer)?;
                *se_puede_iniciar_conexion = true;
            }
            _ => {}
        };
        Ok(())
    }

    /// Notifica a Aplicacion que la llamada inicio, y tambien le notifica a Llamada, enviandole ademas la [ConexionP2P](crate::creacion_llamada::ConexionP2P) creada.
    fn notificar_llamada_iniciada(
        sender_eventos: &Sender<EventoInternoAplicacion>,
        sender_eventos_llamada: &Sender<EventoLlamada>,
        creador_de_conexion: &mut Box<dyn CreadorDeConexionP2P>,
    ) -> Result<(), ErrorCreadorDeConexion> {
        sender_eventos
            .send(EventoInternoAplicacion::EventoObservable(
                EventoAplicacion::LlamadaIniciada,
            ))
            .map_err(|_| ErrorCreadorDeConexion::ErrorComunicandoAAPlicacion)?;

        let conexion = creador_de_conexion
            .obtener_sockets()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        sender_eventos_llamada
            .send(EventoLlamada::LlamadaIniciada(Box::new(conexion)))
            .map_err(|_| ErrorCreadorDeConexion::ErrorComunicandoALlamada)?;
        Ok(())
    }
}

#[test]
fn test_01_se_le_pide_el_offer_al_creador_de_conexion_al_recibir_pedir_offer() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador =
        ComunicadorFake::new(sender_mensajes_a_ser_escuchados.clone(), receiver_mensajes);
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::PedirOffer)
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    assert!(creador_de_conexion.se_genero_offer());
}

#[test]
fn test_02_se_le_envia_offer_a_comunicador_al_recibir_pedir_offer() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::PedirOffer)
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(creador_de_conexion.se_genero_offer());
    assert!(mensajes_enviados.contains(&MensajePCA::Offer("".to_string())))
}

#[test]
fn test_03_se_recibe_offer_y_se_genera_answer_al_recibir_offer_desde_el_signaling() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador =
        ComunicadorFake::new(sender_mensajes_a_ser_escuchados.clone(), receiver_mensajes);
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Offer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    assert!(creador_de_conexion.se_genero_answer());
}

#[test]
fn test_04_se_le_envia_answer_al_comunicador_despues_de_recibir_offer() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Offer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(creador_de_conexion.se_genero_answer());
    assert!(mensajes_enviados.contains(&MensajePCA::Answer("".to_string())))
}

#[test]
fn test_05_se_registra_answer_al_recibirlo_desde_el_signaling() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador =
        ComunicadorFake::new(sender_mensajes_a_ser_escuchados.clone(), receiver_mensajes);
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Answer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    assert!(creador_de_conexion.se_recibio_answer());
}

#[test]
fn test_06_se_inicia_conexion_despues_de_registrar_answer() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador =
        ComunicadorFake::new(sender_mensajes_a_ser_escuchados.clone(), receiver_mensajes);
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Answer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    assert!(creador_de_conexion.se_recibio_answer());
    assert!(creador_de_conexion.se_inicio_conexion());
}

#[test]
fn test_07_se_inicia_conexion_despues_de_enviar_answer() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let comunicador =
        ComunicadorFake::new(sender_mensajes_a_ser_escuchados.clone(), receiver_mensajes);
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, _) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Offer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    assert!(creador_de_conexion.se_genero_answer());
    assert!(creador_de_conexion.se_inicio_conexion());
}

#[test]
fn test_08_se_avisa_a_aplicacion_al_iniciar_llamada() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, _) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Offer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    let evento_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(creador_de_conexion.se_genero_answer());
    assert!(mensajes_enviados.contains(&MensajePCA::Answer("".to_string())));
    if let EventoInternoAplicacion::EventoObservable(evento) = evento_recibido {
        assert!(matches!(evento, EventoAplicacion::LlamadaIniciada));
    } else {
        panic!("Deberia recibirse un evento LlamadaIniciada");
    }
}

#[test]
fn test_09_se_avisa_a_llamada_al_iniciar_llamada() {
    let (sender_mensajes_a_ser_escuchados, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let comunicador = ComunicadorFakeYMock::new(
        sender_mensajes_a_ser_escuchados.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let mutex_creador_de_conexion = Arc::new(Mutex::new(CreadorDeConexionMock::default()));
    let (sender_eventos, receiver_eventos) = mpsc::channel();
    let (sender_eventos_llamada, receiver_eventos_llamada) = mpsc::channel();
    MediadorDeConexionesP2P::iniciar(
        Box::new(comunicador),
        Box::new(Arc::clone(&mutex_creador_de_conexion)),
        sender_eventos,
        sender_eventos_llamada,
    );

    sender_mensajes_a_ser_escuchados
        .send(MensajePCA::Offer("".to_string()))
        .unwrap();
    thread::sleep(Duration::from_millis(500));
    let evento_recibido = receiver_eventos
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();
    let evento_llamada_recibido = receiver_eventos_llamada
        .recv_timeout(Duration::from_millis(1500))
        .unwrap();

    let creador_de_conexion = mutex_creador_de_conexion.lock().unwrap();
    let mensajes_enviados = mutex_mensajes_enviados.lock().unwrap();
    assert!(creador_de_conexion.se_genero_answer());
    assert!(mensajes_enviados.contains(&MensajePCA::Answer("".to_string())));
    assert!(matches!(
        evento_llamada_recibido,
        EventoLlamada::LlamadaIniciada(..)
    ));
    if let EventoInternoAplicacion::EventoObservable(evento) = evento_recibido {
        assert!(matches!(evento, EventoAplicacion::LlamadaIniciada));
    } else {
        panic!("Deberia recibirse un evento LlamadaIniciada");
    }
}
