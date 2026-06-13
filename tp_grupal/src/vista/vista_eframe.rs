//! Implementación del trait Vista realizada con [eframe]

use std::path::PathBuf;

use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use eframe::{
    App,
    egui::{CentralPanel, ColorImage},
};

use crate::{
    aplicacion::{Aplicacion, ErrorAplicacion, EventoAplicacion},
    protocolos::pca::usuario::UsuarioPCA,
    vista::{
        ErrorVista, Vista,
        pantallas::{
            pantalla::{AccionPantalla, Pantalla},
            pantalla_iniciando_llamada::PantallaIniciandoLlamada,
            pantalla_inicio::PantallaInicio,
            pantalla_llamada::PantallaLlamada,
            pantalla_lobby::PantallaLobby,
            pantalla_login::PantallaLogin,
            pantalla_registro::PantallaRegistro,
        },
    },
};

/// Implementación de [Vista] realizada con [eframe].
///
/// Esta vista se suscribe al instanciarse a una [Aplicacion], de la cual ademas debe guardar una referencia.
/// Antes de mostrar cada actualización, se escuchan todos los eventos y se actualiza la vista según eso.
///
/// Esta implementación tendra como colaborador interno una instancia de [Aplicacion]. Luego de renderizar cada [Pantalla] usando
/// el metodo [Pantalla::renderizar()], ejecutara el handler correspondiente a la [AccionPantalla] obtenida.
///
/// PD: Documento como esta implementado para no perdernos en caso de necesitar ampliarlo. La documentación del funcionamiento de cada parte se encuentra
/// ya sea en la [documentación del modulo](crate::vista) o la [documentacion del modulo Pantalla](crate::vista::pantallas).
#[allow(dead_code)]
pub struct VistaEframe {
    aplicacion: Aplicacion,
    rx_eventos: Receiver<EventoAplicacion>,
    pantalla_actual: Box<dyn Pantalla>,
    rx_path: Option<Receiver<Option<std::path::PathBuf>>>,
}

impl Vista for VistaEframe {
    fn actualizacion_registro_exitoso(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::RegistroExitoso);
        Ok(())
    }

    fn actualizacion_usuarios(&mut self, usuarios: Vec<UsuarioPCA>) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::UsuariosNuevos(usuarios));
        Ok(())
    }

    fn actualizar_sesion_iniciada(&mut self, _iniciada: bool) -> Result<(), ErrorVista> {
        self.pantalla_actual = Box::new(PantallaLobby::default());
        Ok(())
    }

    fn actualizacion_error_registro(&mut self, error_str: &str) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::ErrorDeRegistro(error_str.to_string()));
        Ok(())
    }

    fn actualizacion_error_iniciando_sesion(&mut self, error_str: &str) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::ErrorIniciandoSesion(
                error_str.to_string(),
            ));
        Ok(())
    }

    fn actualizacion_recibiendo_llamada(&mut self, usuario: &str) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::RecibiendoLlamada(usuario.to_string()));
        Ok(())
    }

    fn actualizacion_llamada_iniciando(&mut self) -> Result<(), ErrorVista> {
        // Actualmente esto solo va a pasar del lado de quien llama. Eso es porque
        // el evento LlamadaIniciada se envia cuando:
        // 1. El servidor pide un offer
        // 2. El servidor me manda un offer
        // Por lo pronto, no se esta mandando el offer cuando el servidor lo pide, por lo tanto
        // la situacion 1 se va a dar pero la situación 2 no.
        // Cuando se implemente el envio del offer ya va a funcionar de una!
        self.pantalla_actual = Box::new(PantallaIniciandoLlamada::default());
        Ok(())
    }

    fn actualizacion_llamada_rechazada(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::LlamadaRechazada);
        Ok(())
    }

    fn actualizacion_llamada_finalizada(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual = Box::new(PantallaLobby::default());
        Ok(())
    }

    fn actualizacion_enviando_llamada(&mut self, usuario: &str) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::EnviandoLlamada(usuario.to_string()));
        Ok(())
    }

    fn actualizacion_error_llamando(&mut self, _mensaje: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_externa_rechazada(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::LlamadaExternaRechazada);
        Ok(())
    }

    fn actualizacion_llamada_iniciada(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual = Box::new(PantallaLlamada::default());
        Ok(())
    }

    fn actualizacion_nuevo_frame(&mut self, frame: ColorImage) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::NuevoFrame(frame));
        Ok(())
    }

    fn actualizacion_nuevo_frame_local(&mut self, frame: ColorImage) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::NuevoFrameLocal(frame));
        Ok(())
    }

    fn actualizacion_nueva_lista_camaras_disponibles(
        &mut self,
        camaras_disponibles: Vec<String>,
    ) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::NuevaListaDeCamarasDisponibles(
                camaras_disponibles,
            ));
        Ok(())
    }

    fn actualizacion_nueva_camara_en_uso(&mut self, camara: String) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::NuevaCamaraEnUso(camara));
        Ok(())
    }

    fn actualizacion_sesion_cerrada(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual = Box::new(PantallaInicio::default());
        Ok(())
    }

    fn actualizacion_error_cerrando_sesion(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::ErrorCerrandoSesion);
        Ok(())
    }

    fn actualizacion_microfono_muteado(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::MicrofonoMuteado);
        Ok(())
    }

    fn actualizacion_microfono_desmuteado(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::MicrofonoDesmuteado);
        Ok(())
    }

    fn actualizacion_nuevas_estadisticas(
        &mut self,
        estadisticas: Box<crate::sesion_rtp::sesion::EstadisticasReceiver>,
    ) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::NuevasEstadisticas(estadisticas));
        Ok(())
    }

    fn actualizacion_recibiendo_oferta_archivo(
        &mut self,
        nombre: String,
        tamanio: u64,
    ) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::RecibendoOfertaArchivo { nombre, tamanio });
        Ok(())
    }

    fn actualizacion_archivo_aceptado_por_peer(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::ArchivoAceptadoPorPeer);
        Ok(())
    }

    fn actualizacion_archivo_rechazado_por_peer(&mut self) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::ArchivoRechazadoPorPeer);
        Ok(())
    }

    fn actualizacion_archivo_recibido(
        &mut self,
        nombre: String,
        ruta: PathBuf,
    ) -> Result<(), ErrorVista> {
        self.pantalla_actual
            .escuchar_evento(EventoAplicacion::ArchivoRecibido { nombre, ruta });
        Ok(())
    }
}

impl App for VistaEframe {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.procesar_eventos_ocurridos_en_tick().is_err() {
            eprintln!("Fallo al actualizar GUI")
        };
        // Mostrar la GUI
        CentralPanel::default().show(ctx, |_ui| {
            let accion_pantalla = self.pantalla_actual.renderizar(ctx);

            if let Err(error) = self.ejecutar_metodo_accion(accion_pantalla) {
                dbg!(error);
            };
        });

        ctx.request_repaint_after(Duration::from_millis(33));
    }
}

impl VistaEframe {
    /// Crea una [VistaEframe] que se comunique con la [Aplicacion] recibida y la devuelve.
    pub fn crear_vista(mut aplicacion: Aplicacion) -> Result<VistaEframe, ErrorVista> {
        // Creo un observer y le envio el sender a la Aplicacion para que
        // me envie sus actualizaciones
        let (tx_eventos, rx_eventos) = mpsc::channel();

        aplicacion
            .suscribir(tx_eventos)
            .map_err(|_| ErrorVista::ErrorInterno)?;

        Ok(VistaEframe {
            aplicacion,
            rx_eventos,
            pantalla_actual: Box::new(PantallaInicio::default()),
            rx_path: None,
        })
    }

    // Metodos provistos para usar si se escuchan los eventos por un channel
    fn procesar_eventos_ocurridos_en_tick(&mut self) -> Result<(), ErrorVista> {
        if let Some(rx) = &self.rx_path
            && let Ok(result) = rx.try_recv()
        {
            self.rx_path = None;
            if let Some(path) = result
                && let Err(e) = self.aplicacion.enviar_archivo(path)
            {
                eprintln!("[VistaEframe] Error enviando archivo: {e:?}");
            }
        }

        let mut evento_recibido = self.rx_eventos.try_recv();
        while evento_recibido.is_ok() {
            if let Ok(evento) = evento_recibido {
                self.procesar_evento(evento)?;
            }
            evento_recibido = self.rx_eventos.try_recv();
        }

        Ok(())
    }

    fn ejecutar_metodo_accion(&mut self, accion: AccionPantalla) -> Result<(), ErrorAplicacion> {
        match accion {
            AccionPantalla::IrALogin => self.handler_boton_ir_a_login(),
            AccionPantalla::IrARegistro => self.handler_boton_ir_a_registro(),
            AccionPantalla::Volver => self.handler_boton_volver_login(),
            AccionPantalla::IntentarLogin(usuario, contrasenia) => {
                self.handler_boton_iniciar_sesion(usuario, contrasenia)
            }
            AccionPantalla::Ninguna => Ok(()),
            AccionPantalla::IntentarRegistro(usuario, contrasenia) => {
                self.handler_boton_intentar_registro(usuario, contrasenia)
            }
            AccionPantalla::Llamar(usuario) => self.handler_boton_llamar(usuario),
            AccionPantalla::AtenderLlamada => self.handler_boton_atender_llamada(),
            AccionPantalla::RechazarLlamada => self.handler_boton_rechazar_llamada(),
            AccionPantalla::NuevoFrame => self.handler_nuevo_frame(),
            AccionPantalla::CortarLlamada => self.handler_boton_cortar_llamada(),
            AccionPantalla::PedirUsuarios => self.handler_pedir_usuarios(),
            AccionPantalla::PedirListaDeCamaras => self.handler_pedir_lista_de_camaras(),
            AccionPantalla::CambiarCamara(nombre_camara_nueva) => {
                self.handler_cambiar_camara(nombre_camara_nueva)
            }
            AccionPantalla::CerrarSesion => self.handler_boton_cerrar_sesion(),
            AccionPantalla::MutearMicrofono => self.handler_mutear_microfono(),
            AccionPantalla::DesmutearMicrofono => self.handler_desmutear_microfono(),
            AccionPantalla::AbrirDialogoArchivo => self.handler_abrir_dialogo_archivo(),
            AccionPantalla::EnviarArchivo(path) => self.handler_enviar_archivo(path),
            AccionPantalla::AceptarArchivo => self.handler_aceptar_archivo(),
            AccionPantalla::RechazarArchivo => self.handler_rechazar_archivo(),
        }
    }

    fn handler_boton_ir_a_login(&mut self) -> Result<(), ErrorAplicacion> {
        self.pantalla_actual = Box::new(PantallaLogin::default());
        Ok(())
    }

    fn handler_boton_ir_a_registro(&mut self) -> Result<(), ErrorAplicacion> {
        self.pantalla_actual = Box::new(PantallaRegistro::default());
        Ok(())
    }

    fn handler_boton_intentar_registro(
        &mut self,
        usuario: String,
        contrasenia: String,
    ) -> Result<(), ErrorAplicacion> {
        self.aplicacion.registrarse(&usuario, &contrasenia)
    }

    fn handler_boton_volver_login(&mut self) -> Result<(), ErrorAplicacion> {
        self.pantalla_actual = Box::new(PantallaInicio::default());
        Ok(())
    }

    fn handler_boton_iniciar_sesion(
        &mut self,
        usuario: String,
        contrasenia: String,
    ) -> Result<(), ErrorAplicacion> {
        self.aplicacion.iniciar_sesion(&usuario, &contrasenia)
    }

    fn handler_boton_llamar(&mut self, usuario: String) -> Result<(), ErrorAplicacion> {
        self.aplicacion.llamar(&usuario)
    }

    fn handler_boton_atender_llamada(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.atender_llamada()
    }

    fn handler_boton_rechazar_llamada(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.rechazar_llamada()
    }

    fn handler_nuevo_frame(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.enviar_nuevo_frame()
    }

    fn handler_boton_cortar_llamada(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.cortar_llamada()
    }

    fn handler_pedir_usuarios(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.usuarios()
    }

    fn handler_pedir_lista_de_camaras(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.camaras_disponibles()
    }

    fn handler_cambiar_camara(&mut self, nombre_camara: String) -> Result<(), ErrorAplicacion> {
        self.aplicacion.cambiar_camara(&nombre_camara)
    }

    fn handler_boton_cerrar_sesion(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.cerrar_sesion()
    }
    fn handler_mutear_microfono(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.mutear_microfono()
    }

    fn handler_desmutear_microfono(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.desmutear_microfono()
    }

    fn handler_abrir_dialogo_archivo(&mut self) -> Result<(), ErrorAplicacion> {
        let (tx, rx) = mpsc::channel();
        self.rx_path = Some(rx);
        std::thread::spawn(move || {
            let output = std::process::Command::new("zenity")
                .args(["--file-selection", "--title=Seleccionar archivo"])
                .output();
            let path = output
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| std::path::PathBuf::from(s.trim()));
            let _ = tx.send(path);
        });
        Ok(())
    }

    fn handler_enviar_archivo(&mut self, path: PathBuf) -> Result<(), ErrorAplicacion> {
        self.aplicacion.enviar_archivo(path)
    }

    fn handler_aceptar_archivo(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.aceptar_archivo()
    }

    fn handler_rechazar_archivo(&mut self) -> Result<(), ErrorAplicacion> {
        self.aplicacion.rechazar_archivo()
    }
}
