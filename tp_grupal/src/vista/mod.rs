//! # Vista -  Representación de una vista generica para la aplicación.
//!
//! En este modulo se provee la interfaz [Vista], que deberá ser implementada por cualquier
//! vista que desee mostrar el estado de [Aplicacion](crate::aplicacion::Aplicacion).
//!
//! La idea es que la vista posea un metodo para responder a cada [EventoAplicacion].
//!
//! El metodo [Vista::procesar_evento()] se implementa automaticamente para todos los tipos que implementen
//! el trait Vista, y su función es invocar el metodo correspondiente al [EventoAplicacion] recibido. Cada implementación de Vista podra recibir
//! los eventos por el medio que sea, y luego invocar [Vista::procesar_evento()] para que se haga la actualización correspondiente.
//!
//! En el modulo vista se puede encontrar:
//!
//! - [VistaEframe](self::vista_eframe::VistaEframe), que es una implementación del trait Vista realizada con Eframe.
//! - El modulo [pantallas](self::pantallas). Este contiene el trait [Pantalla](crate::vista::pantallas::pantalla::Pantalla), que define la interfaz minima necesaria
//!   para que una pantalla pueda ser mostrada por [VistaEframe](self::vista_eframe::VistaEframe). Ademas, contiene todas las implementaciónes del
//!   trait [Pantalla](crate::vista::pantallas::pantalla::Pantalla), es decir, el codigo de cada pantalla de la aplicación.

use std::path::PathBuf;

use eframe::egui::ColorImage;

use crate::{
    aplicacion::EventoAplicacion, protocolos::pca::usuario::UsuarioPCA,
    sesion_rtp::sesion::EstadisticasReceiver,
};

pub mod customizacion;
pub mod pantallas;
pub mod vista_eframe;
pub mod vista_mock;

#[derive(Debug)]
pub enum ErrorVista {
    ErrorInterno,
    ErrorEnElObservador,
    ErrorAplicacion(String),
}

pub trait Vista {
    /// Actualiza la vista para indicar que se inicio sesion correctamente
    fn actualizar_sesion_iniciada(&mut self, iniciada: bool) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hay una nueva lista de usuarios
    fn actualizacion_usuarios(&mut self, usuarios: Vec<UsuarioPCA>) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que se registro correctamente
    fn actualizacion_registro_exitoso(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hubo un error registrando el usuario
    fn actualizacion_error_registro(&mut self, error_str: &str) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hubo un error iniciando sesion
    fn actualizacion_error_iniciando_sesion(&mut self, error_str: &str) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que se esta recibiendo una llamada de un usuario determinado
    fn actualizacion_recibiendo_llamada(&mut self, usuario: &str) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que se inicio una llamada con otro peer
    fn actualizacion_llamada_iniciando(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que el otro peer rechazo nuestra llamada
    fn actualizacion_llamada_rechazada(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que el otro peer rechazo nuestra llamada
    fn actualizacion_llamada_finalizada(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que el otro peer rechazo nuestra llamada
    fn actualizacion_enviando_llamada(&mut self, usuario: &str) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hubo un error al llamar un usuario
    fn actualizacion_error_llamando(&mut self, mensaje: &str) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hubo un error al llamar un usuario
    fn actualizacion_llamada_externa_rechazada(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hubo un error al llamar un usuario
    fn actualizacion_llamada_iniciada(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para mostrar un nuevo frame
    fn actualizacion_nuevo_frame(&mut self, frame: ColorImage) -> Result<(), ErrorVista>;

    /// Actualiza la vista para mostrar un nuevo frame local
    fn actualizacion_nuevo_frame_local(&mut self, frame: ColorImage) -> Result<(), ErrorVista>;

    /// Actualiza la vista para mostrar que hay una nueva lista de camaras disponibles
    fn actualizacion_nueva_lista_camaras_disponibles(
        &mut self,
        camaras_disponibles: Vec<String>,
    ) -> Result<(), ErrorVista>;

    /// Actualiza para indicar la camara que ahora se usara para videollamadas
    fn actualizacion_nueva_camara_en_uso(&mut self, camara: String) -> Result<(), ErrorVista>;

    /// Actualiza para indicar que se cerro la sesion
    fn actualizacion_sesion_cerrada(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza para indicar que hubo un error cerrando la sesion
    fn actualizacion_error_cerrando_sesion(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que se muteo el microfono
    fn actualizacion_microfono_muteado(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que se desmuteo el microfono
    fn actualizacion_microfono_desmuteado(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que hay nuevas estadisticas que se pueden mostrar
    fn actualizacion_nuevas_estadisticas(
        &mut self,
        estadisticas: Box<EstadisticasReceiver>,
    ) -> Result<(), ErrorVista>;

    /// Actualiza la vista para mostrar el popup de oferta de archivo entrante
    fn actualizacion_recibiendo_oferta_archivo(
        &mut self,
        nombre: String,
        tamanio: u64,
    ) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que el peer acepto nuestra oferta de archivo
    fn actualizacion_archivo_aceptado_por_peer(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que el peer rechazo nuestra oferta de archivo
    fn actualizacion_archivo_rechazado_por_peer(&mut self) -> Result<(), ErrorVista>;

    /// Actualiza la vista para indicar que se recibió y guardó un archivo
    fn actualizacion_archivo_recibido(
        &mut self,
        nombre: String,
        ruta: PathBuf,
    ) -> Result<(), ErrorVista>;

    /// Recibe un [EventoAplicacion] y ejecuta el metodo de [Vista] correspondiente a ese evento
    fn procesar_evento(&mut self, evento: EventoAplicacion) -> Result<(), ErrorVista> {
        match evento {
            EventoAplicacion::RegistroExitoso => self.actualizacion_registro_exitoso()?,
            EventoAplicacion::SesionIniciada => self.actualizar_sesion_iniciada(true)?,
            EventoAplicacion::UsuariosNuevos(usuarios) => self.actualizacion_usuarios(usuarios)?,
            EventoAplicacion::ErrorDeRegistro(error_str) => {
                self.actualizacion_error_registro(&error_str)?
            }
            EventoAplicacion::ErrorIniciandoSesion(error_str) => {
                self.actualizacion_error_iniciando_sesion(&error_str)?
            }
            EventoAplicacion::RecibiendoLlamada(usuario) => {
                self.actualizacion_recibiendo_llamada(&usuario)?
            }
            EventoAplicacion::LlamadaIniciando => self.actualizacion_llamada_iniciando()?,
            EventoAplicacion::LlamadaRechazada => self.actualizacion_llamada_rechazada()?,
            EventoAplicacion::LlamadaFinalizada => self.actualizacion_llamada_finalizada()?,
            EventoAplicacion::EnviandoLlamada(usuario) => {
                self.actualizacion_enviando_llamada(&usuario)?
            }
            EventoAplicacion::ErrorCreandoLlamada(mensaje) => {
                self.actualizacion_error_llamando(&mensaje)?
            }
            EventoAplicacion::LlamadaExternaRechazada => {
                self.actualizacion_llamada_externa_rechazada()?
            }
            EventoAplicacion::LlamadaIniciada => self.actualizacion_llamada_iniciada()?,
            EventoAplicacion::NuevoFrame(frame) => self.actualizacion_nuevo_frame(frame)?,
            EventoAplicacion::NuevoFrameLocal(frame) => {
                self.actualizacion_nuevo_frame_local(frame)?
            }
            EventoAplicacion::NuevaListaDeCamarasDisponibles(camaras_disponibles) => {
                self.actualizacion_nueva_lista_camaras_disponibles(camaras_disponibles)?
            }
            EventoAplicacion::NuevaCamaraEnUso(camara) => {
                self.actualizacion_nueva_camara_en_uso(camara)?
            }
            EventoAplicacion::ErrorCerrandoSesion => self.actualizacion_error_cerrando_sesion()?,
            EventoAplicacion::SesionCerrada => self.actualizacion_sesion_cerrada()?,
            EventoAplicacion::MicrofonoMuteado => self.actualizacion_microfono_muteado()?,
            EventoAplicacion::MicrofonoDesmuteado => self.actualizacion_microfono_desmuteado()?,
            EventoAplicacion::NuevasEstadisticas(estadisticas) => {
                self.actualizacion_nuevas_estadisticas(estadisticas)?
            }
            EventoAplicacion::RecibendoOfertaArchivo { nombre, tamanio } => {
                self.actualizacion_recibiendo_oferta_archivo(nombre, tamanio)?
            }
            EventoAplicacion::ArchivoAceptadoPorPeer => {
                self.actualizacion_archivo_aceptado_por_peer()?
            }
            EventoAplicacion::ArchivoRechazadoPorPeer => {
                self.actualizacion_archivo_rechazado_por_peer()?
            }
            EventoAplicacion::ArchivoRecibido { nombre, ruta } => {
                self.actualizacion_archivo_recibido(nombre, ruta)?
            }
        };

        Ok(())
    }
}
