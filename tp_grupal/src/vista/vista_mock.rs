use std::{
    cell::RefCell,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use crate::{
    aplicacion::EventoAplicacion,
    protocolos::pca::usuario::UsuarioPCA,
    vista::{ErrorVista, Vista},
};

/// Tiempo maximo que se esperara por un evento determinado.
/// Aproximadamente 15 segundos dejaria siempre como minimo porque el proceso de iniciar una llamada de los dos
/// lados puede tardar por el intercambio de offers y answers y los checkeos
const TIEMPO_ESPERA_MAXIMO: u64 = 30000;

/// Objeto simulador tipo mock usado para testear que se ejecuten
/// determinados metodos de la vista.
pub struct VistaMock {
    sesion_iniciada: bool,
    usuarios_actualizados: bool,
    ultimos_usuarios: Vec<UsuarioPCA>,
    registro_exitoso: bool,
    enviando_llamada: bool,
    usuario_siendo_llamado: String,
    recibiendo_llamada: bool,
    llamada_iniciada: bool,
    usuario_llamandome: String,
    receiver_eventos: Receiver<EventoAplicacion>,
    sender_eventos: Sender<EventoAplicacion>,
}

impl Default for VistaMock {
    fn default() -> Self {
        let (sender_eventos, receiver_eventos) = mpsc::channel();
        VistaMock {
            sesion_iniciada: false,
            usuarios_actualizados: false,
            ultimos_usuarios: vec![],
            registro_exitoso: false,
            receiver_eventos,
            sender_eventos,
            enviando_llamada: false,
            usuario_siendo_llamado: String::new(),
            recibiendo_llamada: false,
            usuario_llamandome: String::new(),
            llamada_iniciada: false,
        }
    }
}

impl VistaMock {
    pub fn sesion_iniciada(&self) -> bool {
        self.sesion_iniciada
    }

    pub fn usuarios_actualizados(&self) -> bool {
        self.usuarios_actualizados
    }

    pub fn ultimos_usuarios(&self) -> Vec<UsuarioPCA> {
        self.ultimos_usuarios.clone()
    }

    pub fn se_informo_registro_exitoso(&self) -> bool {
        self.registro_exitoso
    }

    pub fn se_informo_sesion_iniciada(&self) -> bool {
        self.sesion_iniciada
    }

    pub fn se_informo_enviando_llamada(&self, usuario: &str) -> bool {
        self.enviando_llamada && self.usuario_siendo_llamado == usuario
    }

    pub fn se_informo_recibiendo_llamada(&self, usuario: &str) -> bool {
        self.recibiendo_llamada && self.usuario_llamandome == usuario
    }

    pub fn se_informo_llamada_iniciada(&self) -> bool {
        self.llamada_iniciada
    }

    pub fn obtener_sender_eventos(&self) -> Sender<EventoAplicacion> {
        self.sender_eventos.clone()
    }

    pub fn procesar_eventos(&mut self) {
        while let Ok(evento) = self.receiver_eventos.try_recv() {
            self.procesar_evento(evento)
                .expect("Fallo al procesar un evento");
        }
    }

    pub fn esperar_y_procesar_evento(&mut self) {
        let evento = self
            .receiver_eventos
            .recv_timeout(Duration::from_millis(TIEMPO_ESPERA_MAXIMO))
            .expect("Fallo al recibir un evento");
        dbg!(&evento);
        self.procesar_evento(evento)
            .expect("Fallo procesando un evento");
    }
}

impl Vista for VistaMock {
    fn actualizacion_usuarios(&mut self, usuarios: Vec<UsuarioPCA>) -> Result<(), ErrorVista> {
        self.usuarios_actualizados = true;
        self.ultimos_usuarios = usuarios;

        Ok(())
    }

    fn actualizar_sesion_iniciada(&mut self, iniciada: bool) -> Result<(), ErrorVista> {
        self.sesion_iniciada = iniciada;

        Ok(())
    }

    fn actualizacion_registro_exitoso(&mut self) -> Result<(), ErrorVista> {
        self.registro_exitoso = true;

        Ok(())
    }

    fn actualizacion_error_registro(&mut self, _error_str: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_error_iniciando_sesion(&mut self, _error_str: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_recibiendo_llamada(&mut self, usuario: &str) -> Result<(), ErrorVista> {
        self.recibiendo_llamada = true;
        self.usuario_llamandome = usuario.to_string();
        Ok(())
    }

    fn actualizacion_llamada_iniciando(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_rechazada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_finalizada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_enviando_llamada(&mut self, usuario: &str) -> Result<(), ErrorVista> {
        self.enviando_llamada = true;
        self.usuario_siendo_llamado = usuario.to_string();
        Ok(())
    }

    fn actualizacion_error_llamando(&mut self, _mensaje: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_externa_rechazada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_iniciada(&mut self) -> Result<(), ErrorVista> {
        self.llamada_iniciada = true;
        Ok(())
    }

    fn actualizacion_nuevo_frame(
        &mut self,
        _frame: eframe::egui::ColorImage,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nuevo_frame_local(
        &mut self,
        _frame: eframe::egui::ColorImage,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nueva_lista_camaras_disponibles(
        &mut self,
        _camaras_disponibles: Vec<String>,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nueva_camara_en_uso(&mut self, _camara: String) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_error_cerrando_sesion(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_sesion_cerrada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_microfono_desmuteado(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_microfono_muteado(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nuevas_estadisticas(
        &mut self,
        _estadisticas: Box<crate::sesion_rtp::sesion::EstadisticasReceiver>,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_recibiendo_oferta_archivo(
        &mut self,
        _nombre: String,
        _tamanio: u64,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_archivo_aceptado_por_peer(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_archivo_rechazado_por_peer(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_archivo_recibido(
        &mut self,
        _nombre: String,
        _ruta: std::path::PathBuf,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }
}

impl Vista for &RefCell<VistaMock> {
    fn actualizacion_usuarios(&mut self, usuarios: Vec<UsuarioPCA>) -> Result<(), ErrorVista> {
        self.borrow_mut().usuarios_actualizados = true;
        self.borrow_mut().ultimos_usuarios = usuarios;

        Ok(())
    }

    fn actualizar_sesion_iniciada(&mut self, iniciada: bool) -> Result<(), ErrorVista> {
        self.borrow_mut().sesion_iniciada = iniciada;

        Ok(())
    }

    fn actualizacion_registro_exitoso(&mut self) -> Result<(), ErrorVista> {
        self.borrow_mut().registro_exitoso = true;

        Ok(())
    }

    fn actualizacion_error_registro(&mut self, _error_str: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_error_iniciando_sesion(&mut self, _error_str: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_recibiendo_llamada(&mut self, _usuario: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_iniciando(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_rechazada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_finalizada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_enviando_llamada(&mut self, _usuario: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_error_llamando(&mut self, _mensaje: &str) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_externa_rechazada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_llamada_iniciada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nuevo_frame(
        &mut self,
        _frame: eframe::egui::ColorImage,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nuevo_frame_local(
        &mut self,
        _frame: eframe::egui::ColorImage,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nueva_lista_camaras_disponibles(
        &mut self,
        _camaras_disponibles: Vec<String>,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nueva_camara_en_uso(&mut self, _camara: String) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_error_cerrando_sesion(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_sesion_cerrada(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_microfono_desmuteado(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_microfono_muteado(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_nuevas_estadisticas(
        &mut self,
        _estadisticas: Box<crate::sesion_rtp::sesion::EstadisticasReceiver>,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_recibiendo_oferta_archivo(
        &mut self,
        _nombre: String,
        _tamanio: u64,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_archivo_aceptado_por_peer(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_archivo_rechazado_por_peer(&mut self) -> Result<(), ErrorVista> {
        Ok(())
    }

    fn actualizacion_archivo_recibido(
        &mut self,
        _nombre: String,
        _ruta: std::path::PathBuf,
    ) -> Result<(), ErrorVista> {
        Ok(())
    }
}
