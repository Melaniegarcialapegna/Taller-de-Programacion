use crate::logger::Logger;
use crate::protocolos::ice::candidato::Candidato;
use crate::protocolos::ice::parser::candidato_a_sdp;
use crate::protocolos::ice::protocolo_stun::MensajeStun;

use std::net::{SocketAddr, ToSocketAddrs, UdpSocket}; // cuando pase lo de sockets.rs a socket_udp para tener una funcion para bindear desde ahi lo cambio
use std::time::Duration;

const CONTEXTO_LOG: &str = "Generación de Candidatos ICE - Srflx";

const TIMEOUT_SECS: u64 = 7; // para esperar respuesta stun
const TAMANIO_MAX_RESPONSE: usize = 512;

/// Genera un candidato ICE de tipo "srflx" (Server Reflexive) contactando a un servidor STUN.
pub fn generar_candidato_srflx(
    logger: &Logger,
    puerto_rtp_local: u16,
    stun_server: &str,
) -> Result<String, String> {
    logger.info(
        "Generando candidato ICE Server Reflexive (srflx)",
        CONTEXTO_LOG,
    );

    // crear y bindear un socket temporal (0.0.0.0:0)
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| {
        format!(
            "Error al enlazar socket UDP para STUN en puerto efímero: {}",
            e
        )
    })?;

    // conectar al servidor STUN
    let remote_addr: SocketAddr = stun_server
        .to_socket_addrs()
        .map_err(|e| format!("Error al resolver la dirección STUN {}: {}", stun_server, e))?
        .find(|addr| addr.is_ipv4())
        .ok_or_else(|| {
            "Error: No se encontró una dirección IPv4 válida para el servidor STUN".to_string()
        })?;

    socket.connect(remote_addr).map_err(|e| {
        format!(
            "Error al conectar el socket al servidor STUN {}: {}",
            stun_server, e
        )
    })?;

    // construir y serializar el Binding Request
    let request = MensajeStun::binding_request();
    let request_buffer = request.serialize();

    logger.info(
        &format!(
            "Enviando Binding Request de {} bytes a {}",
            request_buffer.len(),
            stun_server
        ),
        CONTEXTO_LOG,
    );

    // envío y recepción
    socket
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Error al establecer timeout en socket: {}", e))?;

    socket
        .send(&request_buffer)
        .map_err(|e| format!("Error al enviar Binding Request a STUN: {}", e))?;

    let mut response_buffer = [0; TAMANIO_MAX_RESPONSE];
    let bytes_received = socket
        .recv(&mut response_buffer)
        .map_err(|e| format!("Timeout o error recibiendo respuesta STUN: {}", e))?;

    logger.info(
        &format!("Recibidos {} bytes del servidor STUN", bytes_received),
        CONTEXTO_LOG,
    );

    // deserializar y validar
    let response = MensajeStun::deserialize(&response_buffer[..bytes_received])
        .map_err(|e| format!("Error deserializando respuesta STUN: {}", e))?;

    if !response.es_response() {
        return Err(
            "Mensaje STUN recibido no es un Binding Response o es error (esperado 0x0101)"
                .to_string(),
        );
    }

    // extraer la dirección mapeada (candidato srflx)
    let mapped_addr = response.get_direccion_mappeada().map_err(|e| {
        format!(
            "Error al extraer dirección mapeada (Srflx) del mensaje STUN: {}",
            e
        )
    })?;

    logger.info(
        &format!(
            "Dirección Srflx obtenida: {}:{} (Candidato Server Reflexive)",
            mapped_addr.ip, mapped_addr.puerto
        ),
        CONTEXTO_LOG,
    );

    // construir candidato y formato SDP
    let c = Candidato::crear_candidato("srflx".into(), mapped_addr.ip, puerto_rtp_local)
        .map_err(|e| format!("Error al construir objeto Candidato Srflx: {}", e))?;

    Ok(candidato_a_sdp(2, &c))
}
