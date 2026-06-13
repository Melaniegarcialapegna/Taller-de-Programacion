//! Módulo `logs`
//!
//! Este módulo define las estructuras y funciones auxiliares para el registro
//! de eventos y estados relevantes durante el flujo de intercambio de offer-answer.
//!
//! Está pensado para mantener el módulo principal `rtc_peer_connection` limpio
//! delegando en éste módulo todo el manejo de logs.

use crate::protocolos::sdp::descripcion_de_sesion::DescripcionDeSesion;
use crate::rtc::rtc_peer_connection::RTCPeerConnection;

const CONTEXTO_LOG: &str = "RTCPeerConnection";

/// Representa un evento relevante dentro del flujo de `RTCPeerConnection`
/// que se registrará en los logs.
///
/// Cada variante del enum corresponde a un punto clave del ciclo
/// de comunicación (Offer, Answer, errores, etc.).
pub(crate) enum EventoRTC<'a> {
    OfferGenerado,
    OfferGuardado(&'a str),
    OfferRecibido,
    AnswerGuardado(&'a str),
    AnswerRecibido,
    AnswerProcesado,
    Error(&'a str),
}

/// Trait que agrupa las funciones auxiliares de logging utilizadas por `RTCPeerConnection`.
///
/// Permite centralizar toda la lógica de logs en un único módulo.
/// Estas funciones son usadas internamente por `RTCPeerConnection` para
/// informar sobre generación de SDP, parsing de candidatos ICE, y estados intermedios.
pub(crate) trait LogRTC {
    fn log_evento(&self, evento: EventoRTC);
    fn log_detalles_offer(&self, offer: &DescripcionDeSesion);
    fn log_estado_medias(&self, sdp: &DescripcionDeSesion, peer: &str);
    fn log_estado_medias_despues_de_answer(&self);
    // fn loguear_candidatos_ice_de_sdp(&self, sdp: &DescripcionDeSesion);
}

/// Implementación del trait [`LogRTC`] para [`RTCPeerConnection`].
///
/// Todas las llamadas a `self.logger.info(...)` o `self.logger.error(...)`
/// se centralizan acá, manteniendo la responsabilidad de logging
/// separada de la lógica de negocio de la conexión.
impl LogRTC for RTCPeerConnection {
    /// Loguea un evento general (inicio, generación, error, etc.)
    fn log_evento(&self, evento: EventoRTC) {
        use EventoRTC::*;
        match evento {
            OfferGenerado => self.logger.info(
                "Estructura SDP Offer generada correctamente (DescripcionDeSesion)",
                CONTEXTO_LOG,
            ),
            OfferGuardado(ruta) => self.logger.info(
                &format!("Archivo Offer SDP guardado en {}", ruta),
                CONTEXTO_LOG,
            ),
            OfferRecibido => self.logger.info("SDP Offer recibido", CONTEXTO_LOG),
            AnswerGuardado(ruta) => self
                .logger
                .info(&format!("Answer guardado en {}", ruta), CONTEXTO_LOG),
            AnswerRecibido => self.logger.info("Answer remoto recibido", CONTEXTO_LOG),
            AnswerProcesado => self
                .logger
                .info("Answer remoto procesado correctamente", CONTEXTO_LOG),
            Error(e) => self.logger.error(&format!("Error: {}", e), CONTEXTO_LOG),
        }
    }

    /// Loguea los detalles comparativos entre Offer recibido y Answer generado.
    fn log_detalles_offer(&self, offer: &DescripcionDeSesion) {
        for (i, media) in offer.get_medias().iter().enumerate() {
            self.logger.info(
                &format!(
                    "Media[{}] '{}': {} candidatos locales",
                    i,
                    media.get_tipo(),
                    media.get_candidatos_ice_locales().len()
                ),
                "SDP",
            );

            for candidato in media.get_candidatos_ice_locales() {
                self.logger.info(
                    &format!("Candidato ICE local generado: {}", candidato),
                    "SDP",
                );
            }
        }
    }

    /// Loguea el estado actual de las descripciones de media (cantidad de candidatos locales/remotos).
    fn log_estado_medias(&self, sdp: &DescripcionDeSesion, peer: &str) {
        for (i, media) in sdp.get_medias().iter().enumerate() {
            self.logger.info(
                &format!(
                    "{} - DescripcionDeMedia[{}]: {} locales, {} remotos",
                    peer,
                    i,
                    media.get_candidatos_ice_locales().len(),
                    media.get_candidatos_ice_remotos().len()
                ),
                CONTEXTO_LOG,
            );
        }
    }

    /// Loguea el estado de las descripciones de media del peer A después de recibir el Answer.
    fn log_estado_medias_despues_de_answer(&self) {
        if let Some(local) = &self.get_sdp_local() {
            for (i, media) in local.get_medias().iter().enumerate() {
                self.logger.info(
                    &format!(
                        "Peer A - DescripcionDeMedia[{}]: {} locales, {} remotos",
                        i,
                        media.get_candidatos_ice_locales().len(),
                        media.get_candidatos_ice_remotos().len()
                    ),
                    CONTEXTO_LOG,
                );
            }
        }
    }
}
