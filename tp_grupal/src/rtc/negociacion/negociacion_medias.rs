//! Módulo `negociacion`
//!
//! Este módulo se encarga de manejar la negociación de medias entre peers usando SDP y ICE.
//! Contiene funciones para seleccionar medias activas post-finalización de intercambio de SDPs offer-answer, finalizar la
//! negociación determinando endpoints válidos, y parsear candidatos ICE tipo host.

use crate::logger::Logger;
use crate::protocolos::ice::chequeo_ice;
use crate::protocolos::ice::flags_ice::FlagsICE;
use crate::protocolos::sdp::descripcion_de_sesion::DescripcionDeSesion;
use crate::rtc::negociacion::error::ErrorDeNegociacion;
use crate::rtc::negociacion::media_activa::MediaActiva;
use crate::sesion_rtp::socket_udp::SocketUDP;
use std::sync::mpsc::Receiver;

const CONTEXTO_LOG: &str = "Negociación";

type Sdps<'a> = (&'a DescripcionDeSesion, &'a DescripcionDeSesion);

type Sockets<'a> = (&'a mut dyn SocketUDP, &'a mut dyn SocketUDP);

type CanalYControl<'a> = (Receiver<([u8; 12], String, u16)>, (&'a FlagsICE, bool));

/// Busca, en cada media activa, un candidato válido, delegando al módulo de ICE la generación de pares de candidatos y sus respectivos connectivity checks.
/// Retorna el candidato cuyos conectivity checks (tanto el de RTP como el de RTCP) fueron exitosos.
pub fn negociar_y_obtener_candidato<'a>(
    logger: &'a Logger,
    sdps: Sdps<'a>,
    sockets: Sockets<'a>,
    canal_y_control: CanalYControl<'a>,
) -> Result<(String, String, u16, u16), ErrorDeNegociacion> {
    let (sdp_local, sdp_remoto) = sdps;
    let (socket_rtp, socket_rtcp) = sockets;
    let (rx_ice, (flags_ice, es_controlling)) = canal_y_control;

    let medias = seleccionar_medias_activas(sdp_local, sdp_remoto, logger)?;

    let mut todos_candidatos_locales: Vec<String> = Vec::new();
    let mut todos_candidatos_remotos: Vec<String> = Vec::new();

    for media in medias {
        todos_candidatos_locales.extend(media.candidatos_locales);
        todos_candidatos_remotos.extend(media.candidatos_remotos);
    }

    match chequeo_ice::negociar_par_candidatos(
        logger,
        (socket_rtp, socket_rtcp),
        (&todos_candidatos_locales, &todos_candidatos_remotos),
        (rx_ice, (flags_ice, es_controlling)),
    ) {
        Ok(par) => {
            return Ok((
                par.local.getter_tipo().to_string(),
                par.remoto.getter_ip().to_string(),
                par.remoto.getter_puerto(),
                par.remoto.getter_puerto() + 1,
            ));
        }
        Err(e) => {
            logger.error(
                &format!("Fallo ICE al chequear todos los pares: {}", e),
                CONTEXTO_LOG,
            );
        }
    }

    Err(ErrorDeNegociacion::ErrorDeICE(
        "No se pudo establecer conectividad ICE".into(),
    ))
}

/// Selecciona las medias entre las que podemos buscar un candidato ICE válido
/// O sea, aquellas que tanto A como B soportan (por tipo y códecs)
pub fn seleccionar_medias_activas(
    sdp_local: &DescripcionDeSesion,
    sdp_remoto: &DescripcionDeSesion,
    _logger: &Logger,
) -> Result<Vec<MediaActiva>, ErrorDeNegociacion> {
    let medias_local = sdp_local.get_medias();
    let medias_remoto = sdp_remoto.get_medias();

    if medias_local.is_empty() {
        return Err(ErrorDeNegociacion::SdpFaltante("local".into()));
    }

    if medias_remoto.is_empty() {
        return Err(ErrorDeNegociacion::SdpFaltante("remoto".into()));
    }

    let mut activas = Vec::new();
    for media_local in medias_local {
        if let Some(media_remota) = medias_remoto
            .iter()
            .find(|m| m.get_mid() == media_local.get_mid())
        {
            let puerto_local = media_local.get_puerto();
            let puerto_remoto = media_remota.get_puerto();

            // una media con puerto 0 es una media rechazada, que no debemos considerar
            if puerto_local == 0 || puerto_remoto == 0 {
                continue;
            }

            let puerto_local_rtcp = puerto_local + 1;
            let puerto_remoto_rtcp = puerto_remoto + 1;

            activas.push(MediaActiva {
                mid: media_local.get_mid().to_string(),
                tipo: media_local.get_tipo().to_string(),
                puerto_local_rtp: puerto_local,
                puerto_local_rtcp,
                puerto_remoto_rtp: puerto_remoto,
                puerto_remoto_rtcp,
                candidatos_remotos: media_remota.get_candidatos_ice_locales().clone(),
                candidatos_locales: media_local.get_candidatos_ice_locales().clone(),
            });
        }
    }

    if activas.is_empty() {
        return Err(ErrorDeNegociacion::MediasFaltantes);
    }

    Ok(activas)
}
