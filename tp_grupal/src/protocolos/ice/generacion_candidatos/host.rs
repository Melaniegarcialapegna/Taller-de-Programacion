use crate::logger::Logger;
use crate::protocolos::ice::candidato::Candidato;
use crate::protocolos::ice::generacion_candidatos::error::ErrorGeneracionDeCandidatosICE;
use crate::protocolos::ice::parser::candidato_a_sdp;

use if_addrs::{IfAddr, get_if_addrs};

const CONTEXTO_LOG: &str = "Generación de Candidatos ICE - Host";

/// Genera candidatos ICE de tipo "host" basados en las interfaces de red locales.
pub fn generar_candidatos_host(
    logger: &Logger,
    puerto_rtp_local: u16,
) -> Result<Vec<String>, ErrorGeneracionDeCandidatosICE> {
    logger.info("Generando candidatos ICE locales (host)", CONTEXTO_LOG);

    let ips = obtener_ips_validas().map_err(|e| {
        logger.error(&format!("{}", e), CONTEXTO_LOG);
        e
    })?;

    let candidatos = construir_candidatos(&ips, puerto_rtp_local).map_err(|e| {
        logger.error(&format!("{}", e), CONTEXTO_LOG);
        e
    })?;

    Ok(candidatos)
}

/// Obtiene las direcciones IP válidas de las interfaces de red locales.
fn obtener_ips_validas() -> Result<Vec<String>, ErrorGeneracionDeCandidatosICE> {
    let ifaces = get_if_addrs()
        .map_err(|e| ErrorGeneracionDeCandidatosICE::ObtenerInterfaces(e.to_string()))?;

    let mut ips = Vec::new();

    for iface in ifaces {
        if iface.is_loopback() {
            continue;
        }

        if let IfAddr::V4(v4) = iface.addr {
            ips.push(v4.ip.to_string())
        }
    }

    if ips.is_empty() {
        return Err(ErrorGeneracionDeCandidatosICE::SinInterfacesValidas);
    }

    Ok(ips)
}

/// Auxiliar que construye los candidatos ICE en formato SDP a partir de las direcciones IP y el puerto dado.
fn construir_candidatos(
    ips: &[String],
    puerto: u16,
) -> Result<Vec<String>, ErrorGeneracionDeCandidatosICE> {
    let mut res = Vec::new();

    for (id, ip) in ips.iter().enumerate() {
        let c = Candidato::crear_candidato("host".into(), ip.into(), puerto)
            .map_err(|e| ErrorGeneracionDeCandidatosICE::CrearCandidato(e.to_string()))?;

        let cand_sdp = candidato_a_sdp(id + 1, &c);
        res.push(cand_sdp);
    }

    Ok(res)
}
