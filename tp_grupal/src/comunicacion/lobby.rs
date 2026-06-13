//! Lobby - Lista de usuarios de la Aplicacion
//!
//! [Lobby] representa un objeto capaz de mantener el listado de usuarios actualizado, con su estado y sus nombres

use crate::{
    aplicacion::{EventoAplicacion, EventoInternoAplicacion},
    comunicacion::comunicador::Comunicador,
    logger::Logger,
    protocolos::pca::{mensaje::MensajePCA, usuario::UsuarioPCA},
};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex, MutexGuard, mpsc::Sender},
    thread,
};

#[derive(Debug)]
pub enum ErrorLobby {
    ErrorConElComunicador(String),
    ErrorObteniendoLockUsuarios,
    ErrorComunicandoAAplicacion,
}

impl Display for ErrorLobby {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLobby::ErrorConElComunicador(e) => {
                f.write_str(&format!("Error con el comunicador: {e}"))
            }
            ErrorLobby::ErrorComunicandoAAplicacion => {
                f.write_str("Error comunicando a Aplicacion")
            }
            ErrorLobby::ErrorObteniendoLockUsuarios => {
                f.write_str("Error obteniendo lock de usuarios")
            }
        }
    }
}

pub trait Lobby {
    fn usuarios(&self) -> Result<Vec<UsuarioPCA>, ErrorLobby>;
}

#[derive(Debug)]
/// [LobbyConComunicador] representa un [Lobby] que se mantiene actualizado según las modificaciones escuchadas
/// del servidor mediante un [Comunicador].
///
/// En la implementación actual, ademas, [LobbyConComunicador] tiene un channel hacia el notificador de [Aplicacion](crate::aplicacion::Aplicacion),
/// que le permite informar la nueva lista de usuarios cada vez que cambia.
///
/// **Importante**: La razón de que [Aplicacion](crate::aplicacion::Aplicacion) no mantenga como colaborador interno a [LobbyConComunicador] es que justamente los eventos se informan en tiempo real (es decir,
/// en cuanto se modifica la lista de usuarios se informa al notificador, y este ultimo le informa a los subscribers de la [Aplicacion](crate::aplicacion::Aplicacion)). Si se quisiera
/// consultar los usuarios sincronicamente, entonces si se deberia mantener al [LobbyConComunicador] como colaborador interno.
pub struct LobbyConComunicador {
    mutex_usuarios: Arc<Mutex<HashMap<String, UsuarioPCA>>>,
}

impl LobbyConComunicador {
    /// Crea un LobbyConComunicador que escuchara las actualizaciones mediante el [Comunicador] especificado, y las informara como un [EventoInternoAplicacion]
    /// por el [Sender] especificado.
    pub fn new(
        comunicador: Box<dyn Comunicador>,
        sender_evento_interno: Sender<EventoInternoAplicacion>,
        logger: Logger,
    ) -> LobbyConComunicador {
        let usuarios = HashMap::new();
        let mutex_usuarios = Arc::new(Mutex::new(usuarios));
        let clon_mutex_usuarios = Arc::clone(&mutex_usuarios);

        thread::spawn(move || {
            Self::escuchar_actualizaciones_usuarios(
                comunicador,
                clon_mutex_usuarios,
                sender_evento_interno,
                logger,
            )
        });

        LobbyConComunicador { mutex_usuarios }
    }

    fn escuchar_actualizaciones_usuarios(
        comunicador: Box<dyn Comunicador>,
        mutex_usuarios: Arc<Mutex<HashMap<String, UsuarioPCA>>>,
        sender_evento_interno: Sender<EventoInternoAplicacion>,
        logger: Logger,
    ) {
        if let Err(error) = Self::_escuchar_actualizaciones_usuarios(
            comunicador,
            mutex_usuarios,
            sender_evento_interno,
            logger.clone(),
        ) {
            logger.error(&format!("{error}"), "Lobby");
        };
    }

    fn _escuchar_actualizaciones_usuarios(
        mut comunicador: Box<dyn Comunicador>,
        mutex_usuarios: Arc<Mutex<HashMap<String, UsuarioPCA>>>,
        sender_evento_interno: Sender<EventoInternoAplicacion>,
        mut logger: Logger,
    ) -> Result<(), ErrorLobby> {
        loop {
            let mensaje = comunicador.escuchar_mensaje().map_err(|_| {
                ErrorLobby::ErrorConElComunicador("Fallo al leer un mensaje".to_string())
            })?;

            match mensaje {
                MensajePCA::Usuarios(lista_usuarios) => Self::reemplazar_usuarios(
                    &mutex_usuarios,
                    lista_usuarios,
                    sender_evento_interno.clone(),
                    &mut logger,
                )?,
                MensajePCA::UsuarioEstado(usuario_modificado) => Self::actualizar_usuario(
                    &mutex_usuarios,
                    usuario_modificado,
                    sender_evento_interno.clone(),
                    &mut logger,
                )?,
                _ => {}
            };
        }
    }

    fn reemplazar_usuarios(
        mutex_usuarios: &Arc<Mutex<HashMap<String, UsuarioPCA>>>,
        lista_usuarios: Vec<UsuarioPCA>,
        sender_evento_interno: Sender<EventoInternoAplicacion>,
        logger: &mut Logger,
    ) -> Result<(), ErrorLobby> {
        let cantidad_usuarios = lista_usuarios.len();
        logger.info(
            &format!(
                "Se recibio una nueva lista con {} usuarios. Se reemplaza por la lista actual",
                cantidad_usuarios
            ),
            "Lobby",
        );

        let mut usuarios = mutex_usuarios
            .lock()
            .map_err(|_| ErrorLobby::ErrorObteniendoLockUsuarios)?;
        *usuarios = HashMap::new();

        for usuario in lista_usuarios {
            let nombre = usuario.nombre();
            usuarios.insert(nombre, usuario);
        }

        let usuarios = Self::obtener_usuarios(usuarios);
        let evento_ocurrido = EventoAplicacion::UsuariosNuevos(usuarios);
        sender_evento_interno
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorLobby::ErrorComunicandoAAplicacion)?;

        Ok(())
    }

    fn actualizar_usuario(
        mutex_usuarios: &Arc<Mutex<HashMap<String, UsuarioPCA>>>,
        usuario_modificado: UsuarioPCA,
        sender_evento_interno: Sender<EventoInternoAplicacion>,
        logger: &mut Logger,
    ) -> Result<(), ErrorLobby> {
        logger.info(
            &format!(
                "Se recibio una actualizacion en el estado del usuario {}",
                usuario_modificado.nombre()
            ),
            "Lobby",
        );

        let mut usuarios = mutex_usuarios
            .lock()
            .map_err(|_| ErrorLobby::ErrorObteniendoLockUsuarios)?;

        usuarios.insert(usuario_modificado.nombre(), usuario_modificado);

        let usuarios = Self::obtener_usuarios(usuarios);
        let evento_ocurrido = EventoAplicacion::UsuariosNuevos(usuarios);
        sender_evento_interno
            .send(EventoInternoAplicacion::EventoObservable(evento_ocurrido))
            .map_err(|_| ErrorLobby::ErrorComunicandoAAplicacion)?;

        Ok(())
    }

    fn obtener_usuarios(usuarios: MutexGuard<'_, HashMap<String, UsuarioPCA>>) -> Vec<UsuarioPCA> {
        let clon_usuarios = usuarios.clone();
        clon_usuarios.into_values().collect()
    }
}

impl Lobby for LobbyConComunicador {
    fn usuarios(&self) -> Result<Vec<UsuarioPCA>, ErrorLobby> {
        let usuarios = self
            .mutex_usuarios
            .lock()
            .map_err(|_| ErrorLobby::ErrorObteniendoLockUsuarios)?;

        Ok(Self::obtener_usuarios(usuarios))
    }
}

#[derive(Default)]
pub struct LobbyDummy {}

impl Lobby for LobbyDummy {
    fn usuarios(&self) -> Result<Vec<UsuarioPCA>, ErrorLobby> {
        Ok(vec![])
    }
}

#[cfg(test)]
use crate::{
    comunicacion::comunicador_fake::ComunicadorFake, protocolos::pca::estado::EstadoUsuarioPCA,
};

#[cfg(test)]
use std::{sync::mpsc, time::Duration};

#[test]
fn test_01_lista_de_usuarios_empieza_vacia() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let (sender_evento_interno, _) = mpsc::channel();
    let comunicador = ComunicadorFake::new(sender_mensajes, receiver_mensajes);
    let lobby = LobbyConComunicador::new(
        Box::new(comunicador),
        sender_evento_interno,
        Logger::dummy_logger(),
    );

    let usuarios = lobby.usuarios().unwrap();

    assert!(usuarios.is_empty());
}

#[test]
fn test_02_lista_de_usuarios_se_actualiza_al_recibir_mensaje_usuarios() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let (sender_evento_interno, receiver_evento_interno) = mpsc::channel();
    let comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let lobby = LobbyConComunicador::new(
        Box::new(comunicador),
        sender_evento_interno,
        Logger::dummy_logger(),
    );

    let usuario = UsuarioPCA::new("Juan".to_string(), EstadoUsuarioPCA::Disponible);

    let usuarios_pre_mensaje = lobby.usuarios().unwrap();
    sender_mensajes
        .send(MensajePCA::Usuarios(vec![usuario.clone()]))
        .unwrap();
    thread::sleep(Duration::from_millis(1000));
    let usuarios_post_mensaje = lobby.usuarios().unwrap();

    assert!(usuarios_pre_mensaje.is_empty());
    assert!(usuarios_post_mensaje == vec![usuario]);

    // Uso las puntas de los channels no testeadas para que no se termine su lifetime y panickee el test
    sender_mensajes.send(MensajePCA::Ok).unwrap();
    let _ = receiver_evento_interno.recv().unwrap();
}

#[test]
fn test_03_se_actualiza_usuario_al_recibir_mensaje_actualizacion_usuarios() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let (sender_evento_interno, receiver_evento_interno) = mpsc::channel();
    let comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let lobby = LobbyConComunicador::new(
        Box::new(comunicador),
        sender_evento_interno,
        Logger::dummy_logger(),
    );

    let usuario = UsuarioPCA::new("Juan".to_string(), EstadoUsuarioPCA::Disponible);
    let usuario_modificado = UsuarioPCA::new("Juan".to_string(), EstadoUsuarioPCA::Ocupado);

    sender_mensajes
        .send(MensajePCA::Usuarios(vec![usuario.clone()]))
        .unwrap();
    sender_mensajes
        .send(MensajePCA::UsuarioEstado(usuario_modificado.clone()))
        .unwrap();
    thread::sleep(Duration::from_millis(1000));
    let usuarios_post_mensaje = lobby.usuarios().unwrap();

    assert!(usuarios_post_mensaje == vec![usuario_modificado]);

    // Uso las puntas de los channels no testeadas para que no se termine su lifetime y panickee el test
    sender_mensajes.send(MensajePCA::Ok).unwrap();
    let _ = receiver_evento_interno.recv().unwrap();
}

#[test]
fn test_04_se_comunica_el_evento_interno_al_actualizar_usuarios() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let (sender_evento_interno, receiver_evento_interno) = mpsc::channel();
    let comunicador = ComunicadorFake::new(sender_mensajes.clone(), receiver_mensajes);
    let _ = LobbyConComunicador::new(
        Box::new(comunicador),
        sender_evento_interno,
        Logger::dummy_logger(),
    );
    let usuario = UsuarioPCA::new("Juan".to_string(), EstadoUsuarioPCA::Disponible);

    sender_mensajes
        .send(MensajePCA::Usuarios(vec![usuario]))
        .unwrap();
    let evento_interno = receiver_evento_interno.recv().unwrap();

    assert!(matches!(
        evento_interno,
        EventoInternoAplicacion::EventoObservable(EventoAplicacion::UsuariosNuevos(_))
    ));
}
