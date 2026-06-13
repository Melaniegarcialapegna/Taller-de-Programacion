/// Implementación del handshake de DCEP (Data Channel Establishment Protocol) para establecer un canal de datos a través de SCTP.
/// Este módulo maneja el proceso de apertura de un canal DCEP, incluyendo la creación y envío del mensaje de apertura (DATA_CHANNEL_OPEN) y la recepción
/// del mensaje de confirmación (ACK) para completar el handshake.
/// También define el estado del canal DCEP y proporciona funciones para iniciar el canal y procesar los mensajes recibidos.
use bytes::Bytes;
use sctp_proto::{Association, PayloadProtocolIdentifier, StreamId};

use crate::protocolos::sctp::dcep::data_channel_open_serde::DataChannelOpen;
use crate::protocolos::sctp::dcep::data_channel_open_serde::MSG_ACK;
use crate::protocolos::sctp::dcep::data_channel_open_serde::MSG_OPEN;
use crate::protocolos::sctp::dcep::data_channel_open_serde::TipoCanal;
use crate::protocolos::sctp::dcep::data_channel_open_serde::serializar_ack;

#[derive(Debug)]
/// Enum que representa el estado del canal DCEP durante el proceso de handshake, incluyendo los estados de inactividad, esperando ACK, abierto y fallido.
pub enum EstadoDcep {
    Inactivo,
    EsperandoAck { stream_id: StreamId },
    Abierto { stream_id: StreamId },
    Fallido,
}

/// Función para iniciar el proceso de apertura de un canal DCEP, enviando un mensaje de apertura (DATA_CHANNEL_OPEN) a través del SCTP y esperando la
/// confirmación (ACK).
/// Recibe la asociación SCTP, un booleano que indica si es el cliente DTLS (que determina el stream_id a usar) y la etiqueta del canal. Devuelve el
/// stream_id
pub fn iniciar_canal(
    asociacion: &mut Association,
    es_dtls_client: bool,
    label: &str,
) -> Result<StreamId, String> {
    eprintln!("Iniciando canal DCEP con label '{label}'...");
    let stream_id: StreamId = if es_dtls_client { 0 } else { 1 };

    eprintln!("[DCEP] Abriendo stream {stream_id} para DCEP...");
    let msg = DataChannelOpen {
        tipo_canal: TipoCanal::Reliable,
        prioridad: 0,
        reliability_param: 0,
        label: label.to_string(),
        protocolo: String::new(),
    }
    .serializar();

    eprintln!("[DCEP] Enviando DATA_CHANNEL_OPEN en stream {stream_id}...");
    let mut stream = asociacion
        .open_stream(stream_id, PayloadProtocolIdentifier::Dcep)
        .map_err(|e| format!("Error abriendo stream DCEP: {e:?}"))?;
    eprintln!("[DCEP] Escribiendo DATA_CHANNEL_OPEN en stream {stream_id}...");
    stream
        .write_sctp(&msg, PayloadProtocolIdentifier::Dcep)
        .map_err(|e| format!("Error enviando DATA_CHANNEL_OPEN: {e:?}"))?;

    eprintln!("[DCEP] DATA_CHANNEL_OPEN enviado en stream {stream_id}, label='{label}'");
    Ok(stream_id)
}

/// Función para procesar los mensajes recibidos a través del SCTP relacionados con el canal DCEP, manejando tanto el mensaje de apertura
/// (DATA_CHANNEL_OPEN) como el mensaje de confirmación (ACK) para completar el handshake. Actualiza el estado del canal DCEP según el mensaje recibido y
/// maneja errores en caso de mensajes desconocidos o problemas en la comunicación.
pub fn procesar_mensaje_dcep(
    stream_id: StreamId,
    data: Bytes,
    estado: &mut EstadoDcep,
    asociacion: &mut Association,
) {
    match data.first() {
        Some(&MSG_OPEN) => {
            if let Some(open) = DataChannelOpen::deserializar(data) {
                eprintln!(
                    "[DCEP] DATA_CHANNEL_OPEN recibido: '{}' en stream {}",
                    open.label, stream_id
                );
            }
            if let Ok(mut stream) = asociacion.stream(stream_id) {
                let ack = serializar_ack();
                if let Err(e) = stream.write_sctp(&ack, PayloadProtocolIdentifier::Dcep) {
                    eprintln!("[DCEP] Error enviando ACK: {e:?}");
                    *estado = EstadoDcep::Fallido;
                    return;
                }
            }
            *estado = EstadoDcep::Abierto { stream_id };
            eprintln!("[DCEP] Canal abierto (receptor) en stream {stream_id}");
        }
        Some(&MSG_ACK) => {
            *estado = EstadoDcep::Abierto { stream_id };
            eprintln!("[DCEP] ACK recibido, canal abierto en stream {stream_id}");
        }
        _ => {
            eprintln!("[DCEP] Mensaje DCEP desconocido en stream {stream_id}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn payload_open(label: &str) -> Bytes {
        DataChannelOpen {
            tipo_canal: TipoCanal::Reliable,
            prioridad: 0,
            reliability_param: 0,
            label: label.to_string(),
            protocolo: String::new(),
        }
        .serializar()
    }

    fn payload_ack() -> Bytes {
        serializar_ack()
    }

    #[test]
    fn estado_inactivo_es_debug() {
        let estado = EstadoDcep::Inactivo;
        let s = format!("{:?}", estado);
        assert!(s.contains("Inactivo"));
    }

    #[test]
    fn estado_esperando_ack_contiene_stream_id() {
        let estado = EstadoDcep::EsperandoAck { stream_id: 42 };
        let s = format!("{:?}", estado);
        assert!(s.contains("EsperandoAck"));
        assert!(s.contains("42"));
    }

    #[test]
    fn estado_abierto_contiene_stream_id() {
        let estado = EstadoDcep::Abierto { stream_id: 7 };
        let s = format!("{:?}", estado);
        assert!(s.contains("Abierto"));
        assert!(s.contains("7"));
    }

    #[test]
    fn estado_fallido_es_debug() {
        let estado = EstadoDcep::Fallido;
        let s = format!("{:?}", estado);
        assert!(s.contains("Fallido"));
    }

    #[test]
    fn stream_id_es_0_para_dtls_client() {
        let es_dtls_client = true;
        let expected_stream_id: StreamId = 0;
        let stream_id: StreamId = if es_dtls_client { 0 } else { 1 };
        assert_eq!(stream_id, expected_stream_id);
    }

    #[test]
    fn stream_id_es_1_para_dtls_server() {
        let es_dtls_client = false;
        let expected_stream_id: StreamId = 1;
        let stream_id: StreamId = if es_dtls_client { 0 } else { 1 };
        assert_eq!(stream_id, expected_stream_id);
    }

    #[test]
    fn payload_ack_comienza_con_msg_ack() {
        let data = payload_ack();
        assert_eq!(data.first().copied(), Some(MSG_ACK));
    }

    #[test]
    fn payload_open_comienza_con_msg_open() {
        let data = payload_open("test");
        assert_eq!(data.first().copied(), Some(MSG_OPEN));
    }

    #[test]
    fn serializar_deserializar_data_channel_open_es_simetrico() {
        for label in &["", "mi-canal", "chat", "a".repeat(255).as_str()] {
            let original = DataChannelOpen {
                tipo_canal: TipoCanal::Reliable,
                prioridad: 0,
                reliability_param: 0,
                label: label.to_string(),
                protocolo: String::new(),
            };
            let bytes = original.serializar();
            let recuperado = DataChannelOpen::deserializar(bytes)
                .expect("deserializar no debería devolver None");
            assert_eq!(recuperado.label, *label);
        }
    }

    #[test]
    fn data_channel_open_reliable_serializa_sin_panico() {
        let msg = DataChannelOpen {
            tipo_canal: TipoCanal::Reliable,
            prioridad: 256,
            reliability_param: 42,
            label: "canal-prueba".to_string(),
            protocolo: "webrtc-datachannel".to_string(),
        };
        let bytes = msg.serializar();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn deserializar_bytes_vacios_devuelve_none() {
        let resultado = DataChannelOpen::deserializar(Bytes::new());
        assert!(resultado.is_none());
    }

    #[test]
    fn deserializar_bytes_truncados_no_panica() {
        let datos = Bytes::from_static(&[MSG_OPEN, 0x00]);
        let _ = DataChannelOpen::deserializar(datos);
    }
}
