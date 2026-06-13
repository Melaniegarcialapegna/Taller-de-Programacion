// nota: la de finalización no ws estándar
// la dejo por compatibilidad y para terminar bien

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct FlagsICE {
    ice_local_finalizado: Arc<AtomicBool>,
    ice_remoto_finalizado: Arc<AtomicBool>,
    shutdown_solicitado: Arc<AtomicBool>,
}

impl Default for FlagsICE {
    fn default() -> Self {
        Self::new()
    }
}

impl FlagsICE {
    pub fn new() -> Self {
        Self {
            ice_local_finalizado: Arc::new(AtomicBool::new(false)),
            ice_remoto_finalizado: Arc::new(AtomicBool::new(false)),
            shutdown_solicitado: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn solicitar_shutdown(&self) {
        self.shutdown_solicitado.store(true, Ordering::SeqCst);
    }

    pub fn shutdown_solicitado(&self) -> bool {
        self.shutdown_solicitado.load(Ordering::SeqCst)
    }

    pub fn set_ice_local_finalizado(&self) {
        self.ice_local_finalizado.store(true, Ordering::SeqCst);
    }

    pub fn set_ice_remoto_finalizado(&self) {
        self.ice_remoto_finalizado.store(true, Ordering::SeqCst);
    }

    pub fn get_ice_local_finalizado(&self) -> bool {
        self.ice_local_finalizado.load(Ordering::SeqCst)
    }

    pub fn get_ice_remoto_finalizado(&self) -> bool {
        self.ice_remoto_finalizado.load(Ordering::SeqCst)
    }

    /// Retorna verdadero si ambos peers terminaron su etapa ICE.
    pub fn ice_finalizado(&self) -> bool {
        self.get_ice_local_finalizado() && self.get_ice_remoto_finalizado()
    }
}
