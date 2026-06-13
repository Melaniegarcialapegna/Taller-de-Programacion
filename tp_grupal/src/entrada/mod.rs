//! Recepcion - Ingreso al lobby de la aplicación
//!
//! [Recepcion] representa un objeto mediante el cual una [Aplicación](crate::aplicacion::Aplicacion) puede ingresar al lobby, para ver otros usuarios
//! y poder llamarlos. Para lograrlo debe comunicarse mediante un Comunicador con el servidor de signaling.
//!
//! Lo común va a ser que [Aplicación](crate::aplicacion::Aplicacion) cree una [Recepcion] enviandole el comunicador, y la tenga como colaborador interno.
//! Luego, cuando se desee iniciar sesion o registrarse, simplemente se debe redirigir el mensaje

use std::fmt::Display;

use crate::{
    comunicacion::comunicador::{Comunicador, ErrorComunicador},
    protocolos::pca::{mensaje::MensajePCA, usuario::UsuarioPCA},
};

#[derive(Debug)]
pub enum ErrorRecepcion {
    ErrorConElComunicador(String),
    ErrorConElServidor(String),
    ErrorRecibido(String),
    ErrorSesionNoIniciada,
}

impl Display for ErrorRecepcion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ErrorRecepcion::ErrorConElComunicador(error) => {
                f.write_str(&format!("Error con el comunicador: {}", error))
            }
            ErrorRecepcion::ErrorConElServidor(error) => {
                f.write_str(&format!("Error con el servidor: {}", error))
            }
            ErrorRecepcion::ErrorRecibido(error) => {
                f.write_str(&format!("Error recibido del servidor: {}", error))
            }
            ErrorRecepcion::ErrorSesionNoIniciada => f.write_str("Error: Sesion no iniciada"),
        }
    }
}

pub struct Recepcion {
    comunicador: Box<dyn Comunicador>,
    sesion_iniciada: bool,
}

impl Recepcion {
    /// Crea una [Recepcion] que se comunicara con el servidor mediante el comunicador recibido
    pub fn new(comunicador: Box<dyn Comunicador>) -> Recepcion {
        Recepcion {
            comunicador,
            sesion_iniciada: false,
        }
    }

    /// Inicia sesion en el servidor con el usuario y contraseña especificados.
    ///
    /// En caso de iniciar sesión correctamente, devuelve un vector con los usuarios del servidor.
    ///
    /// En caso de error (ya sea en [Recepcion] o con el servidor), devuelve el error correspondiente.
    pub fn iniciar_sesion(
        &mut self,
        usuario: &str,
        contrasenia: &str,
    ) -> Result<Vec<UsuarioPCA>, ErrorRecepcion> {
        let mensaje_a_enviar = MensajePCA::Entrar(usuario.to_string(), contrasenia.to_string());
        let mensaje_recibido = self.enviar_y_esperar_respuesta(mensaje_a_enviar)?;

        if let MensajePCA::Usuarios(lista_usuarios) = mensaje_recibido {
            self.sesion_iniciada = true;
            return Ok(lista_usuarios);
        }

        if let MensajePCA::ErrorPCA(error) = mensaje_recibido {
            return Err(ErrorRecepcion::ErrorRecibido(error));
        }

        Err(ErrorRecepcion::ErrorConElServidor(
            "Se recibio un mensaje invalido del servidor".to_string(),
        ))
    }

    pub fn cerrar_sesion(&mut self) -> Result<(), ErrorRecepcion> {
        if !self.sesion_iniciada {
            return Err(ErrorRecepcion::ErrorSesionNoIniciada);
        }

        let respuesta = self
            .comunicador
            .enviar_y_escuchar_respuesta(&MensajePCA::Salir)?;

        if respuesta == MensajePCA::Salio {
            self.sesion_iniciada = false;
            Ok(())
        } else if let MensajePCA::ErrorPCA(error) = respuesta {
            Err(ErrorRecepcion::ErrorRecibido(error))
        } else {
            Err(ErrorRecepcion::ErrorConElServidor(
                "Se recibio un mensaje invalido del servidor".to_string(),
            ))
        }
    }

    /// Se registra en el servidor con el usuario y contraseña especificados.
    ///
    /// En caso de registrarse correctamente, devuelve un [Ok()] vacio.
    ///
    /// En caso de error (ya sea en [Recepcion] o con el servidor), devuelve el error correspondiente.
    pub fn registrarse(&mut self, usuario: &str, contrasenia: &str) -> Result<(), ErrorRecepcion> {
        let mensaje_a_enviar = MensajePCA::Registrar(usuario.to_string(), contrasenia.to_string());
        let mensaje_recibido = self.enviar_y_esperar_respuesta(mensaje_a_enviar)?;

        if let MensajePCA::ErrorPCA(error) = mensaje_recibido {
            return Err(ErrorRecepcion::ErrorRecibido(error));
        }

        if !matches!(mensaje_recibido, MensajePCA::Registrado) {
            return Err(ErrorRecepcion::ErrorConElServidor(
                "Se recibio un mensaje invalido del servidor".to_string(),
            ));
        }

        Ok(())
    }

    fn enviar_y_esperar_respuesta(
        &mut self,
        mensaje_a_enviar: MensajePCA,
    ) -> Result<MensajePCA, ErrorRecepcion> {
        self.comunicador.enviar_mensaje(&mensaje_a_enviar)?;

        let mensaje_recibido = self.comunicador.escuchar_mensaje()?;

        Ok(mensaje_recibido)
    }
}

impl From<ErrorComunicador> for ErrorRecepcion {
    fn from(error: ErrorComunicador) -> Self {
        ErrorRecepcion::ErrorConElComunicador(format!("{error}"))
    }
}

#[cfg(test)]
use crate::comunicacion::comunicador_fake_y_mock::ComunicadorFakeYMock;
#[cfg(test)]
use crate::comunicacion::comunicador_stub::ComunicadorStub;
#[cfg(test)]
use std::sync::mpsc;
#[cfg(test)]
use std::sync::{Arc, Mutex};

#[test]
fn test_01_se_inicia_sesion_con_usuario_correcto() {
    let comunicador = ComunicadorStub::new(MensajePCA::Usuarios(vec![]));
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.iniciar_sesion("USUARIO", "CONTRASENIA");

    assert!(resultado.is_ok());
}

#[test]
fn test_02_falla_al_iniciar_con_credenciales_invalidas() {
    let comunicador = ComunicadorStub::new(MensajePCA::ErrorPCA("".to_string()));
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.iniciar_sesion("USUARIO", "CONTRASENIA");

    assert!(resultado.is_err());
}

#[test]
fn test_03_falla_al_recibir_mensaje_invalido() {
    let comunicador = ComunicadorStub::new(MensajePCA::Registrado);
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.iniciar_sesion("USUARIO", "CONTRASENIA");

    assert!(resultado.is_err());
    assert!(matches!(
        resultado,
        Err(ErrorRecepcion::ErrorConElServidor(_))
    ));
}

#[test]
fn test_04_se_registra_correctamente_si_recibe_registrado() {
    let comunicador = ComunicadorStub::new(MensajePCA::Registrado);
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.registrarse("USUARIO", "CONTRASENIA");

    assert!(resultado.is_ok());
}

#[test]
fn test_05_falla_si_recibe_errorpca() {
    let comunicador = ComunicadorStub::new(MensajePCA::ErrorPCA("".to_string()));
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.registrarse("USUARIO", "CONTRASENIA");

    assert!(resultado.is_err());
    assert!(matches!(
        resultado,
        Err(ErrorRecepcion::ErrorConElComunicador(_))
    ));
}

#[test]
fn test_06_falla_si_recibe_mensaje_equivocado() {
    let comunicador = ComunicadorStub::new(MensajePCA::Ok);
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.registrarse("USUARIO", "CONTRASENIA");

    assert!(resultado.is_err());
    assert!(matches!(
        resultado,
        Err(ErrorRecepcion::ErrorConElServidor(_))
    ));
}

#[test]
fn test_07_se_envia_mensaje_salir_si_se_quiere_cerrar_sesion() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let mutex_comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        Arc::clone(&mutex_mensajes_enviados),
    );
    let mut recepcion = Recepcion::new(Box::new(mutex_comunicador));

    sender_mensajes.send(MensajePCA::Usuarios(vec![])).unwrap();
    recepcion
        .iniciar_sesion("asd", "asd")
        .expect("Se deberia poder iniciar sesion");
    sender_mensajes.send(MensajePCA::Salio).unwrap();
    recepcion
        .cerrar_sesion()
        .expect("Se deberia poder cerrar sesion");

    let comunicador = mutex_mensajes_enviados.lock().unwrap();
    assert!(comunicador.contains(&MensajePCA::Salir))
}

#[test]
fn test_08_devuelve_ok_si_servidor_informa_que_se_cerro_sesion() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let mutex_comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        mutex_mensajes_enviados,
    );
    let mut recepcion = Recepcion::new(Box::new(mutex_comunicador));

    sender_mensajes.send(MensajePCA::Usuarios(vec![])).unwrap();
    recepcion
        .iniciar_sesion("asd", "asd")
        .expect("Se deberia poder iniciar sesion");
    sender_mensajes.send(MensajePCA::Salio).unwrap();
    let resultado_cerrar_sesion = recepcion.cerrar_sesion();

    assert!(resultado_cerrar_sesion.is_ok());
}

#[test]
fn test_09_devuelve_error_si_servidor_informa_error() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let mutex_comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        mutex_mensajes_enviados,
    );
    let mut recepcion = Recepcion::new(Box::new(mutex_comunicador));

    sender_mensajes.send(MensajePCA::Usuarios(vec![])).unwrap();
    recepcion
        .iniciar_sesion("asd", "asd")
        .expect("Se deberia poder iniciar sesion");
    sender_mensajes
        .send(MensajePCA::ErrorPCA("".to_string()))
        .unwrap();
    let resultado_cerrar_sesion = recepcion.cerrar_sesion();

    if let Err(error) = resultado_cerrar_sesion {
        assert!(matches!(error, ErrorRecepcion::ErrorRecibido(_)))
    } else {
        panic!("Si se recibe un error del comunicador debe devolverse el error")
    }
}

#[test]
fn test_10_se_puede_volver_a_iniciar_sesion_despues_de_cerrar_sesion() {
    let (sender_mensajes, receiver_mensajes) = mpsc::channel();
    let mutex_mensajes_enviados = Arc::new(Mutex::new(vec![]));
    let mutex_comunicador = ComunicadorFakeYMock::new(
        sender_mensajes.clone(),
        receiver_mensajes,
        mutex_mensajes_enviados,
    );
    let mut recepcion = Recepcion::new(Box::new(mutex_comunicador));

    sender_mensajes.send(MensajePCA::Usuarios(vec![])).unwrap();
    recepcion
        .iniciar_sesion("asd", "asd")
        .expect("Se deberia poder iniciar sesion");
    sender_mensajes.send(MensajePCA::Salio).unwrap();
    recepcion
        .cerrar_sesion()
        .expect("Se deberia poder cerrar sesion");
    sender_mensajes.send(MensajePCA::Usuarios(vec![])).unwrap();
    let resultado_volver_a_iniciar = recepcion.iniciar_sesion("asd", "asd");

    assert!(resultado_volver_a_iniciar.is_ok())
}

#[test]
fn test_11_no_se_puede_cerrar_sesion_si_no_se_inicio() {
    let comunicador = ComunicadorStub::new(MensajePCA::Ok);
    let mut recepcion = Recepcion::new(Box::new(comunicador));

    let resultado = recepcion.cerrar_sesion();

    assert!(resultado.is_err());
    assert!(matches!(
        resultado,
        Err(ErrorRecepcion::ErrorSesionNoIniciada)
    ));
}
