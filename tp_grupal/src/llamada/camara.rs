//! # Camara - Transmision de video capturado por un lente.
//!
//! La idea es que las camaras tengan capturadores, que seran quienes reciban la informacion obtenida por la camara.
//! Al mismo tiempo, las camaras deben poder encenderse y apagarse a gusto.

use crate::{
    llamada::{creador_lente::CreadorDeLente, lente::ErrorLente},
    sesion_rtp::comunicacion_rtp::Frame,
};
use std::{
    fmt::{Debug, Display},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

/// Representa una camara, que puede tener un "capturador". Para ser capturador se debera enviar a la camara un sender de Vec<u8>. En adelante,
/// cada frame tomado por la camara se enviara al capturador por ese channel.
pub trait Camara: Send + Sync {
    fn agregar_capturador(&mut self, capturador: Sender<Frame>) -> Result<(), ErrorCamara>;
    fn reiniciar_capturadores(&mut self) -> Result<(), ErrorCamara>;
    fn encender(&mut self) -> Result<(), ErrorCamara>;
    fn apagar(&mut self) -> Result<(), ErrorCamara>;
    fn esta_encendida(&mut self) -> bool;
}

impl Debug for dyn Camara {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Camara")
    }
}

#[derive(Debug)]
pub enum ErrorCamara {
    ErrorEncendiendoCamara,
    ErrorEnLente(String),
    ErrorEnviandoFrame,
    ErrorInterno(String),
    ErrorElLenteEstaInactivo,
}

impl Display for ErrorCamara {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self{
            ErrorCamara::ErrorEnLente(error) => f.write_str(&format!("Error en lente: {error}")),
            ErrorCamara::ErrorEncendiendoCamara => f.write_str("Error encendiendo camara"),
            ErrorCamara::ErrorEnviandoFrame => f.write_str("Error enviando frame"),
            ErrorCamara::ErrorInterno(error) => f.write_str(&format!("Error interno: {error}")),
            ErrorCamara::ErrorElLenteEstaInactivo => f.write_str("Error: La operacion no puede realizarse porque el lente de la camara esta inactivo")
        }
    }
}

pub enum InstruccionALente {
    AgregarCapturador(Sender<Frame>),
    ReiniciarCapturadores,
    Apagar,
}

/// Representa una camara que obtiene video a partir de un [Lente](crate::llamada::lente::Lente) cualquiera.
/// La idea de esto es que, independientemente del lente que se use, se pueda trabajar con la misma camara.
///
/// Los capturadores recibiran el contenido captado por el lente.
pub struct CamaraGenerica {
    senders_a_capturadores: Vec<Sender<Frame>>,
    sender_instrucciones: Option<Sender<InstruccionALente>>,
    creador_de_lente: Box<dyn CreadorDeLente>,
    esta_encendida: bool,
}

impl Camara for CamaraGenerica {
    fn agregar_capturador(&mut self, capturador: Sender<Frame>) -> Result<(), ErrorCamara> {
        // Cambio el sender a capturador de la camara
        self.senders_a_capturadores.push(capturador.clone());

        // Si hay un lente, le notifico que tiene que cambiar a quien reenvia los frames
        if let Some(sender) = &self.sender_instrucciones {
            sender
                .send(InstruccionALente::AgregarCapturador(capturador))
                .map_err(|e| ErrorCamara::ErrorInterno(format!("{e}")))?;
        };

        Ok(())
    }

    fn reiniciar_capturadores(&mut self) -> Result<(), ErrorCamara> {
        self.senders_a_capturadores = vec![];

        // Si hay un lente, le notifico que tiene que cambiar a quien reenvia los frames
        if let Some(sender) = &self.sender_instrucciones {
            sender
                .send(InstruccionALente::ReiniciarCapturadores)
                .map_err(|e| ErrorCamara::ErrorInterno(format!("{e}")))?;
        };

        Ok(())
    }

    fn encender(&mut self) -> Result<(), ErrorCamara> {
        self.iniciar_hilo_lente()?;
        self.esta_encendida = true;

        Ok(())
    }

    fn apagar(&mut self) -> Result<(), ErrorCamara> {
        if let Some(sender) = &self.sender_instrucciones {
            sender
                .send(InstruccionALente::Apagar)
                .map_err(|e| ErrorCamara::ErrorInterno(format!("{e}")))?;
        } else {
            return Err(ErrorCamara::ErrorElLenteEstaInactivo);
        }

        for sender in &mut self.senders_a_capturadores {
            sender.send(Frame::frame_finalizacion()).map_err(|_| {
                ErrorCamara::ErrorInterno("Fallo al enviar frame de finalizacion".to_string())
            })?;
        }

        self.sender_instrucciones = None;
        self.esta_encendida = false;

        Ok(())
    }

    fn esta_encendida(&mut self) -> bool {
        self.esta_encendida
    }
}

impl CamaraGenerica {
    pub fn new(lente: Box<dyn CreadorDeLente>) -> CamaraGenerica {
        CamaraGenerica {
            senders_a_capturadores: vec![],
            sender_instrucciones: None,
            creador_de_lente: lente,
            esta_encendida: false,
        }
    }

    fn enviar_frames_lente(
        creador_de_lente: Box<dyn CreadorDeLente>,
        senders_a_capturadores: Vec<Sender<Frame>>,
        receiver_instrucciones: Receiver<InstruccionALente>,
    ) {
        if let Err(error) = Self::_enviar_frames_lente(
            creador_de_lente,
            senders_a_capturadores,
            receiver_instrucciones,
        ) {
            eprintln!("{error}");
        }
    }

    fn _enviar_frames_lente(
        mut creador_de_lente: Box<dyn CreadorDeLente>,
        mut senders_a_capturadores: Vec<Sender<Frame>>,
        receiver_instrucciones: Receiver<InstruccionALente>,
    ) -> Result<(), ErrorCamara> {
        let mut lente = creador_de_lente
            .crear_lente()
            .map_err(|_| ErrorCamara::ErrorEnLente("Fallo al crear lente".to_string()))?;
        loop {
            let frame = lente.obtener_frame()?;

            let se_debe_apagar_lente = Self::procesar_instrucciones_recibidas(
                &mut senders_a_capturadores,
                &receiver_instrucciones,
            );
            if se_debe_apagar_lente {
                return Ok(());
            }

            for sender in &mut senders_a_capturadores {
                sender
                    .send(frame.clone())
                    .map_err(|_| ErrorCamara::ErrorEnviandoFrame)?;
            }
        }
    }

    fn procesar_instrucciones_recibidas(
        senders_a_capturadores: &mut Vec<Sender<Frame>>,
        receiver_instrucciones: &Receiver<InstruccionALente>,
    ) -> bool {
        let resultado_instruccion = receiver_instrucciones.try_recv();

        if let Ok(instruccion) = resultado_instruccion {
            match instruccion {
                InstruccionALente::AgregarCapturador(nuevo_capturador) => {
                    senders_a_capturadores.push(nuevo_capturador);
                }
                InstruccionALente::Apagar => return true,
                InstruccionALente::ReiniciarCapturadores => *senders_a_capturadores = vec![],
            }
        };

        false
    }

    fn iniciar_hilo_lente(&mut self) -> Result<(), ErrorCamara> {
        // Creo un canal de comunicacion con el lente
        let (sender_instrucciones, receiver_instrucciones) = mpsc::channel();
        self.sender_instrucciones = Some(sender_instrucciones);

        let creador_clon = self
            .creador_de_lente
            .clonar()
            .map_err(|e| ErrorCamara::ErrorInterno(format!("{e}")))?;

        let senders_a_capturadores = self.senders_a_capturadores.clone();

        thread::spawn(move || {
            Self::enviar_frames_lente(creador_clon, senders_a_capturadores, receiver_instrucciones);
        });
        Ok(())
    }
}

impl From<ErrorLente> for ErrorCamara {
    fn from(error: ErrorLente) -> Self {
        ErrorCamara::ErrorEnLente(format!("{error}"))
    }
}

#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
use crate::llamada::creador_lente::CreadorDeLenteStud;

#[test]
fn test_01_camara_no_envia_frames_si_no_esta_encendida() {
    let (sender_frames, receiver_frames) = mpsc::channel();
    let creador_lente = CreadorDeLenteStud::new(vec![1]);
    let mut camara = CamaraGenerica::new(Box::new(creador_lente));
    camara
        .agregar_capturador(sender_frames)
        .expect("Se deberia poder agregar un capturador");

    let frame_recibido = receiver_frames.recv_timeout(Duration::from_millis(500));

    assert!(frame_recibido.is_err());
}

#[test]
fn test_02_se_reciben_frames_al_encender_la_camara() {
    let (sender_frames, receiver_frames) = mpsc::channel();
    let creador_lente = CreadorDeLenteStud::new(vec![1]);
    let mut camara = CamaraGenerica::new(Box::new(creador_lente));
    camara
        .agregar_capturador(sender_frames)
        .expect("Se deberia poder agregar un capturador");

    camara
        .encender()
        .expect("Se deberia poder encender la camara");
    let frame_recibido = receiver_frames.recv_timeout(Duration::from_millis(500));

    assert!(frame_recibido.is_ok());
}

#[test]
fn test_03_se_reenvian_los_mismos_frames_que_capta_el_lente() {
    let (sender_frames, receiver_frames) = mpsc::channel();
    let creador_lente = CreadorDeLenteStud::new(vec![8]);
    let mut camara = CamaraGenerica::new(Box::new(creador_lente));
    camara
        .agregar_capturador(sender_frames)
        .expect("Se deberia poder agregar un capturador");

    camara
        .encender()
        .expect("Se deberia poder encender la camara");
    let frame_recibido = receiver_frames
        .recv_timeout(Duration::from_millis(500))
        .unwrap();

    assert!(frame_recibido.bytes == vec![8]);
}

#[test]
fn test_04_se_envian_frames_a_nuevo_capturador_si_cambia_con_la_camara_encendida() {
    let (sender_frames, receiver_frames) = mpsc::channel();
    let creador_lente = CreadorDeLenteStud::new(vec![1]);
    let mut camara = CamaraGenerica::new(Box::new(creador_lente));
    camara
        .agregar_capturador(sender_frames)
        .expect("Se deberia poder agregar un  capturador");

    //
    camara
        .encender()
        .expect("Se deberia poder encender la camara");
    let (nuevo_sender_frames, nuevo_receiver_frames) = mpsc::channel();
    camara.reiniciar_capturadores().unwrap();
    camara
        .agregar_capturador(nuevo_sender_frames)
        .expect("Se deberia poder cambiar el capturador");
    let frame_recibido_en_anterior = receiver_frames.recv_timeout(Duration::from_millis(500));
    let frame_recibido_en_nuevo = nuevo_receiver_frames.recv_timeout(Duration::from_millis(500));

    assert!(frame_recibido_en_anterior.is_err());
    assert!(frame_recibido_en_nuevo.is_ok());
}

#[test]
fn test_05_se_dejan_de_recibir_frames_al_apagar_la_camara() {
    let creador_lente = CreadorDeLenteStud::new(vec![1]);
    let mut camara = CamaraGenerica::new(Box::new(creador_lente));
    // Agrego capturador a camara
    let (sender_frames, receiver_frames) = mpsc::channel();
    camara
        .agregar_capturador(sender_frames)
        .expect("Se deberia poder agregar un  capturador");

    camara
        .encender()
        .expect("Se deberia poder encender la camara");
    camara.apagar().expect("Se deberia poder apagar la camara");

    let mut recibio_un_ultimo_frame = false;
    for _ in 0..5 {
        let frame_recibido = receiver_frames.recv_timeout(Duration::from_millis(50));
        if frame_recibido.is_err() {
            recibio_un_ultimo_frame = true;
            break;
        }
    }

    if !recibio_un_ultimo_frame {
        panic!("Deberia dejar de recibir frames al apagar la camara")
    }
}

#[test]
fn test_06_se_envian_frames_a_multiples_capturadores() {
    let creador_lente = CreadorDeLenteStud::new(vec![1]);
    let mut camara = CamaraGenerica::new(Box::new(creador_lente));

    // Agrego un capturador a camara
    let (sender_frames, receiver_frames) = mpsc::channel();
    camara
        .agregar_capturador(sender_frames)
        .expect("Se deberia poder agregar un capturador");

    // Agrego otro capturador a camara
    let (otro_sender_frames, otro_receiver_frames) = mpsc::channel();
    camara
        .agregar_capturador(otro_sender_frames)
        .expect("Se deberia poder agregar otro capturador");

    camara
        .encender()
        .expect("Se deberia poder encender la camara");

    let resultado_primer_capturador = receiver_frames.recv_timeout(Duration::from_millis(1500));
    let resultado_segundo_capturador =
        otro_receiver_frames.recv_timeout(Duration::from_millis(1500));

    assert!(resultado_primer_capturador.is_ok());
    assert!(resultado_segundo_capturador.is_ok());
}
