/// Este módulo se encarga de la serialización y deserialización de los mensajes de control del Data Channel, específicamente el mensaje de apertura
/// de canal (DataChannelOpen) y el mensaje de ACK. El mensaje de apertura de canal incluye información sobre el tipo de canal, prioridad, parámetros
/// de confiabilidad, etiqueta y protocolo.
/// La función `serializar` convierte una instancia de `DataChannelOpen` en bytes para su envío a través del SCTP, mientras que la función `deserializar`
/// reconstruye una instancia de `DataChannelOpen` a partir de los bytes recibidos. Además, se proporciona una función para serializar un mensaje de ACK.
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub const PPID_DCEP: u32 = 50;
pub const MSG_OPEN: u8 = 0x03;
pub const MSG_ACK: u8 = 0x02;

pub const INICIO_CANAL_RELIABLE: isize = 0x00;
pub const INICIO_CANAL_RELIABLE_UNORDERED: isize = 0x80;
pub const INICIO_CANAL_PARTIAL_RELIABLE_REXMIT: isize = 0x01;
pub const INICIO_CANAL_PARTIAL_RELIABLE_REXMIT_UNORDERED: isize = 0x81;
pub const INICIO_CANAL_PARTIAL_RELIABLE_TIMED: isize = 0x02;
pub const INICIO_CANAL_PARTIAL_RELIABLE_TIMED_UNORDERED: isize = 0x82;

pub const INICIO_CANAL_RELIABLE_U8: u8 = 0x00;
pub const INICIO_CANAL_RELIABLE_UNORDERED_U8: u8 = 0x80;
pub const INICIO_CANAL_PARTIAL_RELIABLE_REXMIT_U8: u8 = 0x01;
pub const INICIO_CANAL_PARTIAL_RELIABLE_REXMIT_UNORDERED_U8: u8 = 0x81;
pub const INICIO_CANAL_PARTIAL_RELIABLE_TIMED_U8: u8 = 0x02;
pub const INICIO_CANAL_PARTIAL_RELIABLE_TIMED_UNORDERED_U8: u8 = 0x82;

#[derive(Debug, Clone, Copy)]
/// Enum que representa los diferentes tipos de canales que se pueden abrir en un Data Channel, cada uno con sus propias características de confiabilidad
/// y orden.
pub enum TipoCanal {
    // Considero importante destacar que en la presente implementación se utilizó unicamente el tipo de canal Reliable, se tiene en consideración
    // los otros tipos en el enum para implementar cierta fidelidad al estándar, y pensar en no limitar una futura implementación a solo un tipo de canal.
    Reliable = INICIO_CANAL_RELIABLE,
    ReliableUnordered = INICIO_CANAL_RELIABLE_UNORDERED,
    PartialReliableRexmit = INICIO_CANAL_PARTIAL_RELIABLE_REXMIT,
    PartialReliableRexmitUnordered = INICIO_CANAL_PARTIAL_RELIABLE_REXMIT_UNORDERED,
    PartialReliableTimed = INICIO_CANAL_PARTIAL_RELIABLE_TIMED,
    PartialReliableTimedUnordered = INICIO_CANAL_PARTIAL_RELIABLE_TIMED_UNORDERED,
}

#[derive(Debug)]
/// Struct que representa el mensaje de apertura de canal (DataChannelOpen) que se envía a través del SCTP para establecer un nuevo Data Channel, contiene
/// toda la información necesaria para configurar el canal, como el tipo de canal, prioridad, parámetros de confiabilidad, etiqueta y protocolo.
pub struct DataChannelOpen {
    pub tipo_canal: TipoCanal,
    pub prioridad: u16,
    pub reliability_param: u32,
    pub label: String,
    pub protocolo: String,
}

impl DataChannelOpen {
    /// Serializa la instancia de `DataChannelOpen` en bytes para su envío a través del SCTP, siguiendo el formato especificado en el estándar.
    pub fn serializar(&self) -> Bytes {
        //convierto strings a bytes
        let label_bytes = self.label.as_bytes();
        let proto_bytes = self.protocolo.as_bytes();
        let mut buf = BytesMut::with_capacity(12 + label_bytes.len() + proto_bytes.len());
        //escribe datachannel open
        buf.put_u8(MSG_OPEN);
        //completa campos requisito de canal
        buf.put_u8(self.tipo_canal as u8);
        buf.put_u16(self.prioridad);
        buf.put_u32(self.reliability_param);
        buf.put_u16(label_bytes.len() as u16);
        buf.put_u16(proto_bytes.len() as u16);
        buf.put_slice(label_bytes);
        buf.put_slice(proto_bytes);
        buf.freeze()
    }

    /// Deserializa los bytes recibidos a partir del SCTP para reconstruir una instancia de `DataChannelOpen`, verificando que el formato sea correcto y
    /// extrayendo la información necesaria para configurar el canal. Si los bytes no corresponden a un mensaje de apertura de canal válido, devuelve `None`.
    pub fn deserializar(mut data: Bytes) -> Option<Self> {
        if data.remaining() < 12 {
            return None;
        }
        if data.get_u8() != MSG_OPEN {
            return None;
        }
        //leo header fijo
        let tipo_canal_raw = data.get_u8();
        let prioridad = data.get_u16();
        let reliability_param = data.get_u32();
        let label_len = data.get_u16() as usize;
        let proto_len = data.get_u16() as usize;
        if data.remaining() < label_len + proto_len {
            return None;
        }
        // leo label, protocolo y defino el tipo de canal
        let label = String::from_utf8(data.copy_to_bytes(label_len).to_vec()).ok()?;
        let protocolo = String::from_utf8(data.copy_to_bytes(proto_len).to_vec()).ok()?;
        let tipo_canal = determinar_tipo_canal(tipo_canal_raw)?;
        // devuelvo data_channel_open completo con sus parámetros necesarios
        Some(Self {
            tipo_canal,
            prioridad,
            reliability_param,
            label,
            protocolo,
        })
    }
}

fn determinar_tipo_canal(tipo_canal_raw: u8) -> Option<TipoCanal> {
    match tipo_canal_raw {
        INICIO_CANAL_RELIABLE_U8 => Some(TipoCanal::Reliable),
        INICIO_CANAL_RELIABLE_UNORDERED_U8 => Some(TipoCanal::ReliableUnordered),
        INICIO_CANAL_PARTIAL_RELIABLE_REXMIT_U8 => Some(TipoCanal::PartialReliableRexmit),
        INICIO_CANAL_PARTIAL_RELIABLE_REXMIT_UNORDERED_U8 => {
            Some(TipoCanal::PartialReliableRexmitUnordered)
        }
        INICIO_CANAL_PARTIAL_RELIABLE_TIMED_U8 => Some(TipoCanal::PartialReliableTimed),
        INICIO_CANAL_PARTIAL_RELIABLE_TIMED_UNORDERED_U8 => {
            Some(TipoCanal::PartialReliableTimedUnordered)
        }
        _ => None,
    }
}

/// Serializa un mensaje de ACK para su envío a través del SCTP, siguiendo el formato especificado en el estándar.
/// El mensaje de ACK se utiliza para confirmar la recepción
pub fn serializar_ack() -> Bytes {
    Bytes::from_static(&[MSG_ACK])
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializar_datachannel_open() {
        let dc = DataChannelOpen {
            tipo_canal: TipoCanal::Reliable,
            prioridad: 100,
            reliability_param: 0,
            label: "chat".to_string(),
            protocolo: "".to_string(),
        };

        let bytes = dc.serializar();
        assert_eq!(bytes[0], MSG_OPEN);
        assert_eq!(bytes[1], INICIO_CANAL_RELIABLE_U8);
        assert_eq!(u16::from_be_bytes([bytes[2], bytes[3]]), 100);
    }

    #[test]
    fn test_deserializar_datachannel_open_valido() {
        let dc_original = DataChannelOpen {
            tipo_canal: TipoCanal::Reliable,
            prioridad: 50,
            reliability_param: 5000,
            label: "video".to_string(),
            protocolo: "webrtc-datachannel".to_string(),
        };

        let bytes = dc_original.serializar();
        let dc_deserialized = DataChannelOpen::deserializar(bytes).unwrap();

        assert_eq!(dc_deserialized.prioridad, 50);
        assert_eq!(dc_deserialized.reliability_param, 5000);
        assert_eq!(dc_deserialized.label, "video");
        assert_eq!(dc_deserialized.protocolo, "webrtc-datachannel");
    }

    #[test]
    fn test_deserializar_mensaje_invalido() {
        let mut buf = BytesMut::new();
        buf.put_u8(0xFF);
        buf.put_u8(INICIO_CANAL_RELIABLE_U8);
        buf.put_u16(100);
        buf.put_u32(0);
        buf.put_u16(0);
        buf.put_u16(0);

        assert!(DataChannelOpen::deserializar(buf.freeze()).is_none());
    }

    #[test]
    fn test_deserializar_data_insuf() {
        let buf = BytesMut::from(&[MSG_OPEN, 0x00][..]);
        assert!(DataChannelOpen::deserializar(buf.freeze()).is_none());
    }

    #[test]
    fn test_serializar_ack() {
        let ack_bytes = serializar_ack();
        assert_eq!(ack_bytes.len(), 1);
        assert_eq!(ack_bytes[0], MSG_ACK);
    }

    #[test]
    fn test_tipos_channel() {
        let types = vec![
            TipoCanal::Reliable,
            TipoCanal::ReliableUnordered,
            TipoCanal::PartialReliableRexmit,
            TipoCanal::PartialReliableRexmitUnordered,
            TipoCanal::PartialReliableTimed,
            TipoCanal::PartialReliableTimedUnordered,
        ];

        for tipo in types {
            let dc = DataChannelOpen {
                tipo_canal: tipo,
                prioridad: 0,
                reliability_param: 0,
                label: String::new(),
                protocolo: String::new(),
            };
            let bytes = dc.serializar();
            let dc_des = DataChannelOpen::deserializar(bytes).unwrap();
            assert_eq!(dc_des.tipo_canal as u8, tipo as u8);
        }
    }

    #[test]
    fn test_label_protocolo_vacios() {
        let dc = DataChannelOpen {
            tipo_canal: TipoCanal::Reliable,
            prioridad: 0,
            reliability_param: 0,
            label: String::new(),
            protocolo: String::new(),
        };

        let bytes = dc.serializar();
        let dc_des = DataChannelOpen::deserializar(bytes).unwrap();
        assert_eq!(dc_des.label, "");
        assert_eq!(dc_des.protocolo, "");
    }

    #[test]
    fn test_protocolo_y_label_largos() {
        let long_label = "a".repeat(1000);
        let long_proto = "b".repeat(500);

        let dc = DataChannelOpen {
            tipo_canal: TipoCanal::Reliable,
            prioridad: 65535,
            reliability_param: u32::MAX,
            label: long_label.clone(),
            protocolo: long_proto.clone(),
        };

        let bytes = dc.serializar();
        let dc_des = DataChannelOpen::deserializar(bytes).unwrap();
        assert_eq!(dc_des.label, long_label);
        assert_eq!(dc_des.protocolo, long_proto);
    }
}
