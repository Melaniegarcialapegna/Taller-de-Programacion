//! Panel de estadisticas - Obtener estadisticas RTCP de la llamada
//!
//! El panel de estadisticas funciona recibiendo estadisticas sobre la llamada desde un channel
//! creado en la SesionRTP. Puede reutilizarse para muchas sesiones, permitiendo cambiar la fuente
//! de las estadisticas que se almacenan.

use std::{fmt::Display, sync::mpsc::Receiver};

use crate::sesion_rtp::sesion::EstadisticasReceiver;

#[derive(Default)]
pub struct PanelEstadisticas {
    option_receiver_estadisticas: Option<Receiver<EstadisticasReceiver>>,
    ultimas_estadisticas: EstadisticasReceiver,
}

#[derive(Debug)]
pub enum ErrorPanelEstadisticas {
    /// No hay una fuente de datos a la que consultarle por estadisticas
    ErrorNoHayFuente,
}

impl Display for ErrorPanelEstadisticas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("No hay una fuente de datos. No se pueden mostrar estadisticas")
    }
}

impl PanelEstadisticas {
    /// Devuelve las ultimas estadisticas disponibles sobre la llamada.
    ///
    /// Va a fallar si se ejecuta sin antes establecer la fuente de las estadisticas
    /// con [PanelEstadisticas::cambiar_fuente]
    pub fn estadisticas(&mut self) -> Result<EstadisticasReceiver, ErrorPanelEstadisticas> {
        let ref_receiver = self
            .option_receiver_estadisticas
            .as_ref()
            .ok_or(ErrorPanelEstadisticas::ErrorNoHayFuente)?;

        self.ultimas_estadisticas = self.obtener_ultimas_estadisticas(ref_receiver);

        Ok(self.ultimas_estadisticas.clone())
    }

    /// Cambia la fuente de los datos que se van a recibir
    pub fn cambiar_fuente(
        &mut self,
        nueva_fuente: Receiver<EstadisticasReceiver>,
    ) -> Result<(), ErrorPanelEstadisticas> {
        self.option_receiver_estadisticas = Some(nueva_fuente);
        self.ultimas_estadisticas = EstadisticasReceiver::default();
        Ok(())
    }

    /// Borra la fuente de datos actual.
    pub fn borrar_fuente(&mut self) -> Result<(), ErrorPanelEstadisticas> {
        self.option_receiver_estadisticas = None;
        Ok(())
    }

    fn obtener_ultimas_estadisticas(
        &self,
        receiver: &Receiver<EstadisticasReceiver>,
    ) -> EstadisticasReceiver {
        let mut ultimas_estadisticas = self.ultimas_estadisticas.clone();
        while let Ok(estadisticas) = receiver.try_recv() {
            ultimas_estadisticas = estadisticas;
        }
        ultimas_estadisticas
    }
}

#[cfg(test)]
use crate::protocolos::rtcp::tipo_paquete::ContenidoReport;
#[cfg(test)]
use std::sync::mpsc;
#[cfg(test)]
const CANTIDAD_PAQUETES_RECIBIDOS: u32 = 1;
#[cfg(test)]
const CANTIDAD_PAQUETES_RECIBIDOS_ANT: u32 = 2;
#[cfg(test)]
const CANTIDAD_PAQUETES_ESPERADOS_ANT: u32 = 3;

#[test]
fn test_01_si_me_piden_estadisticas_y_no_hay_fuente_falla() {
    let mut panel = PanelEstadisticas::default();

    let resultado_estadisticas = panel.estadisticas();

    assert!(resultado_estadisticas.is_err());
}

#[test]
fn test_02_si_me_piden_estadisticas_y_la_fuente_no_envio_datos_devuelvo_estadisticas_default() {
    let (_, receiver_estadisticas) = mpsc::channel();
    let mut panel = PanelEstadisticas::default();

    panel
        .cambiar_fuente(receiver_estadisticas)
        .expect("Se deberia poder cambiar la fuente");
    let estadisticas = panel
        .estadisticas()
        .expect("Se deberian poder obtener estadisticas");

    assert!(estadisticas == EstadisticasReceiver::default());
}

#[test]
fn test_03_si_me_piden_estadisticas_y_la_fuente_me_mando_datos_devuelvo_esos_datos() {
    let (sender_estadisticas, receiver_estadisticas) = mpsc::channel();
    let mut panel = PanelEstadisticas::default();

    panel
        .cambiar_fuente(receiver_estadisticas)
        .expect("Se deberia poder cambiar la fuente");
    sender_estadisticas
        .send(get_estadisticas_receiver_prueba())
        .expect("Se deberian poder enviar estadisticas");
    let estadisticas = panel
        .estadisticas()
        .expect("Se deberian poder obtener estadisticas");

    assert!(estadisticas.cantidad_paquetes_recibidos == CANTIDAD_PAQUETES_RECIBIDOS);
    assert!(estadisticas.cantidad_paquetes_recibidos_anterior == CANTIDAD_PAQUETES_RECIBIDOS_ANT);
    assert!(estadisticas.cantidad_paquetes_esperados_anterior == CANTIDAD_PAQUETES_ESPERADOS_ANT);
}

#[test]
fn test_04_si_me_borran_la_fuente_y_me_piden_estadisticas_antes_de_que_me_mande_falla() {
    let (_, receiver_estadisticas) = mpsc::channel();
    let mut panel = PanelEstadisticas::default();

    panel
        .cambiar_fuente(receiver_estadisticas)
        .expect("Se deberia poder cambiar la fuente");
    panel
        .borrar_fuente()
        .expect("Se deberia poder borrar la fuente");
    let resultado_estadisticas = panel.estadisticas();

    assert!(resultado_estadisticas.is_err())
}

#[test]
fn test_05_si_me_cambian_la_fuente_y_no_recibi_datos_vuelvo_a_devolver_estadisticas_default() {
    let (sender_estadisticas, receiver_estadisticas) = mpsc::channel();
    let (_, receiver_estadisticas_nuevo) = mpsc::channel();
    let mut panel = PanelEstadisticas::default();

    // Cambio la fuente y hago que envie estadisticas y que el panel las recibda
    panel
        .cambiar_fuente(receiver_estadisticas)
        .expect("Se deberia poder cambiar la fuente");
    sender_estadisticas
        .send(get_estadisticas_receiver_prueba())
        .expect("Se deberian poder enviar estadisticas");
    let _ = panel
        .estadisticas()
        .expect("Se deberian poder consultar estadisticas");
    // Vuelvo a cambiar la fuente pero no mando nada
    panel
        .cambiar_fuente(receiver_estadisticas_nuevo)
        .expect("Se deberia poder cambiar la fuente");
    // Pido estadisticas
    let estadisticas = panel
        .estadisticas()
        .expect("Se deberian poder pedir estadisticas porque hay fuente");

    assert!(estadisticas == EstadisticasReceiver::default());
}

#[cfg(test)]
fn get_estadisticas_receiver_prueba() -> EstadisticasReceiver {
    EstadisticasReceiver {
        cantidad_paquetes_recibidos: CANTIDAD_PAQUETES_RECIBIDOS,
        cantidad_paquetes_recibidos_anterior: CANTIDAD_PAQUETES_RECIBIDOS_ANT,
        cantidad_paquetes_esperados_anterior: CANTIDAD_PAQUETES_ESPERADOS_ANT,
        contenido_report: ContenidoReport::default(),
    }
}
