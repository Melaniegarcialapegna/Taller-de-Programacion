use crate::logger::Logger;
use crate::protocolos::ice::generacion_candidatos::error::ErrorGeneracionDeCandidatosICE;

pub mod error;
pub mod host;
pub mod srflx;

use self::host::generar_candidatos_host;
use self::srflx::generar_candidato_srflx;

const CONTEXTO_LOG: &str = "Generación de Candidatos ICE";

/// Genera la lista completa de candidatos ICE (host y srflx).
/// Esta función centraliza la llamada a los métodos específicos de cada tipo.
///
/// La función es el principal punto de entrada para obtener candidatos, usada en rtc_peer_connection.rs.
pub fn generar_candidatos(
    logger: &Logger,
    puerto_rtp_local: u16,
    stun_server: Option<&str>,
) -> Result<Vec<String>, ErrorGeneracionDeCandidatosICE> {
    logger.info(
        "Iniciando generación de candidatos ICE (Host y Srflx)",
        CONTEXTO_LOG,
    );
    let mut candidatos_sdp = Vec::new();

    match generar_candidatos_host(logger, puerto_rtp_local) {
        Ok(host_cands) => {
            candidatos_sdp.extend(host_cands);
            logger.info(
                &format!(
                    "{} candidato(s) Host generado(s) con éxito.",
                    candidatos_sdp.len()
                ),
                CONTEXTO_LOG,
            );
        }
        Err(e) => logger.warn(
            &format!("Fallo al generar candidatos Host: {}", e),
            CONTEXTO_LOG,
        ),
    }

    if let Some(server) = stun_server {
        match generar_candidato_srflx(logger, puerto_rtp_local, server) {
            Ok(srflx_cand) => {
                candidatos_sdp.push(srflx_cand);
                logger.info("Candidato Srflx generado con éxito.", CONTEXTO_LOG);
            }
            Err(e) => logger.warn(
                &format!("Fallo al generar candidato Srflx: {}", e),
                CONTEXTO_LOG,
            ),
        }
    } else {
        logger.info(
            "Servidor STUN no especificado, saltando generación de Srflx.",
            CONTEXTO_LOG,
        );
    }

    if candidatos_sdp.is_empty() {
        return Err(ErrorGeneracionDeCandidatosICE::SinInterfacesValidas);
    }

    Ok(candidatos_sdp)
}
