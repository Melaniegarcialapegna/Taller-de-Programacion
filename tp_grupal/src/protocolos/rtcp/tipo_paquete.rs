//! # Tipos de paquete RTCP

use super::error::ErrorPaqueteRTCP;

/// Numero asociado a un paquete de tipo FIR
pub const TIPO_PAQUETE_FIR: u8 = 192;

/// Numero asociado a un paquete de tipo NACK
pub const TIPO_PAQUETE_NACK: u8 = 193;

/// Numero asociado a un paquete de tipo SR
pub const TIPO_PAQUETE_SR: u8 = 200;

/// Numero asociado a un paquete de tipo ReceiverReport
pub const TIPO_PAQUETE_RR: u8 = 201;

/// Numero asociado a un paquete de tipo Feedback
pub const TIPO_PAQUETE_FEEDBACK: u8 = 205;

/// Numero asociado a un paquete de tipo Payload Feedback
pub const TIPO_PAQUETE_PL_FEEDBACK: u8 = 206;

/// Numero asociado a un paquete de tipo Bye
pub const TIPO_PAQUETE_BYE: u8 = 203;

/// Numeros de tipos de paquete validos para WebRTC
pub const TIPO_PAQUETE_VALIDOS: [u8; 7] = [
    TIPO_PAQUETE_FIR,
    TIPO_PAQUETE_NACK,
    TIPO_PAQUETE_SR,
    TIPO_PAQUETE_RR,
    TIPO_PAQUETE_FEEDBACK,
    TIPO_PAQUETE_PL_FEEDBACK,
    TIPO_PAQUETE_BYE,
];

/// Tamanio del contenido de un paquete ServerReport
pub const TAMANIO_CONTENIDO_PAQUETE_SR: u16 = 11;

/// Tamanio del contenido de un paquete ReceiverReport
pub const TAMANIO_CONTENIDO_PAQUETE_RR: u16 = 6;

/// Tamanio del contenido de un paquete Bye
pub const TAMANIO_CONTENIDO_PAQUETE_BYE: u16 = 0;

/// Representación del contenido de reporte de un paquete RTCP
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ContenidoReport {
    pub ssrc_report: u32,
    pub frac_paquetes_perdidos: u8,
    pub cant_paquetes_perdidos: u32,
    pub numero_mas_grande_de_paquete_recibido: u32,
    pub tiempo_est_entre_paquetes: u32,
    pub tiempo_desde_ultimo_paquete: u32,
    pub delay_desde_ultimo_paquete: u32,
}

impl ContenidoReport {
    pub fn crear_vacio_con_ssrc(ssrc: u32) -> ContenidoReport {
        ContenidoReport {
            ssrc_report: ssrc,
            frac_paquetes_perdidos: 0,
            cant_paquetes_perdidos: 0,
            numero_mas_grande_de_paquete_recibido: 0,
            tiempo_est_entre_paquetes: 0,
            tiempo_desde_ultimo_paquete: 0,
            delay_desde_ultimo_paquete: 0,
        }
    }

    /// Crea un ContenidoReport a partir de una tira de bytes. Los bytes deben estar en formato big-endian.
    pub fn crear_con_bytes(bytes: &[u8]) -> ContenidoReport {
        let ssrc_report: u32 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let frac_paquetes_perdidos: u8 = u8::from_be_bytes([bytes[4]]);

        let mascara_cant_paquetes_perdidos: u32 = 0x00FFFFFF; //Dejo los ultimos 24 bytes
        let cant_paquetes_perdidos: u32 =
            u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
                & mascara_cant_paquetes_perdidos;

        let numero_mas_grande_de_paquete_recibido: u32 =
            u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let tiempo_est_entre_paquetes: u32 =
            u32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let tiempo_desde_ultimo_paquete: u32 =
            u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let delay_desde_ultimo_paquete: u32 =
            u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);

        ContenidoReport {
            ssrc_report,
            frac_paquetes_perdidos,
            cant_paquetes_perdidos,
            numero_mas_grande_de_paquete_recibido,
            tiempo_est_entre_paquetes,
            tiempo_desde_ultimo_paquete,
            delay_desde_ultimo_paquete,
        }
    }
}

impl From<&ContenidoReport> for Vec<u8> {
    fn from(contenido: &ContenidoReport) -> Self {
        let mut bytes: Vec<u8> = vec![];

        let ssrc_be_bytes = contenido.ssrc_report.to_be_bytes();
        for byte in ssrc_be_bytes {
            bytes.push(byte)
        }

        let frac_paquetes_perdidos: u32 = u32::from(contenido.frac_paquetes_perdidos);
        let bloque_paquetes = frac_paquetes_perdidos << 24 | contenido.cant_paquetes_perdidos;
        let bloque_paquetes_be_bytes = bloque_paquetes.to_be_bytes();
        for byte in bloque_paquetes_be_bytes {
            bytes.push(byte)
        }

        let num_mas_grande_p_perdidos_be_bytes = contenido
            .numero_mas_grande_de_paquete_recibido
            .to_be_bytes();
        for byte in num_mas_grande_p_perdidos_be_bytes {
            bytes.push(byte)
        }

        let tiempo_est_entre_paquetes_be_bytes = contenido.tiempo_est_entre_paquetes.to_be_bytes();
        for byte in tiempo_est_entre_paquetes_be_bytes {
            bytes.push(byte)
        }

        let tiempo_desde_ultimo_paquete_be_bytes =
            contenido.tiempo_desde_ultimo_paquete.to_be_bytes();
        for byte in tiempo_desde_ultimo_paquete_be_bytes {
            bytes.push(byte)
        }

        let delay_desde_ultimo_paquete_be_bytes =
            contenido.delay_desde_ultimo_paquete.to_be_bytes();
        for byte in delay_desde_ultimo_paquete_be_bytes {
            bytes.push(byte)
        }

        bytes
    }
}

/// Representación del contenido de un paquete RTCP de tipo SenderReport
#[derive(Debug, PartialEq)]
pub struct ContenidoSenderReport {
    pub ntp_timestamp: u64,
    pub rtp_timestamp: u32,
    pub sender_packet_count: u32,
    pub sender_octet_count: u32,
    // Asumo que va a haber solo un report para los fines del tp (pq hay solo video, y solo entre dos peers)
    // Si se quisiera agregar mas de uno, aca habria que poner un Vec<ContenidoReport>
    pub contenido_report: ContenidoReport,
}

impl ContenidoSenderReport {
    /// Crea un ContenidoSenderReport a partir de una tira de bytes que representa el contenido.
    /// Los bytes deben estar en formato big-endian.
    fn crear_con_payload(payload: &[u8]) -> ContenidoSenderReport {
        let ntp_timestamp: u64 = u64::from_be_bytes([
            payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
            payload[7],
        ]);

        let rtp_timestamp: u32 =
            u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);

        let sender_packet_count: u32 =
            u32::from_be_bytes([payload[12], payload[13], payload[14], payload[15]]);

        let sender_octet_count: u32 =
            u32::from_be_bytes([payload[16], payload[17], payload[18], payload[19]]);

        let contenido_report = ContenidoReport::crear_con_bytes(&payload[20..]);

        ContenidoSenderReport {
            ntp_timestamp,
            rtp_timestamp,
            sender_packet_count,
            sender_octet_count,
            contenido_report,
        }
    }
}

impl From<&ContenidoSenderReport> for Vec<u8> {
    fn from(contenido: &ContenidoSenderReport) -> Self {
        let mut bytes_contenido: Vec<u8> = vec![];

        let npt_timestamp_bytes = contenido.ntp_timestamp.to_be_bytes();
        for byte in npt_timestamp_bytes {
            bytes_contenido.push(byte);
        }

        let rtp_timestamp_bytes = contenido.rtp_timestamp.to_be_bytes();
        for byte in rtp_timestamp_bytes {
            bytes_contenido.push(byte);
        }

        let sender_packet_count = contenido.sender_packet_count.to_be_bytes();
        for byte in sender_packet_count {
            bytes_contenido.push(byte)
        }

        let sender_octet_count = contenido.sender_octet_count.to_be_bytes();
        for byte in sender_octet_count {
            bytes_contenido.push(byte)
        }

        let bytes_report: Vec<u8> = Vec::from(&contenido.contenido_report);
        for byte in bytes_report {
            bytes_contenido.push(byte);
        }

        bytes_contenido
    }
}

/// Representación del contenido de un paquete RTCP de tipo ReceiverReport
#[derive(Debug, PartialEq)]
pub struct ContenidoReceiverReport {
    // Asumo que va a haber solo un report para los fines del tp (pq hay solo video, y solo entre dos peers)
    // Si se quisiera agregar mas de uno, aca habria que poner un Vec<ContenidoReport>
    pub contenido_report: ContenidoReport,
}

impl ContenidoReceiverReport {
    /// Crea un ContenidoReceiverReport a partir de una tira de bytes que representa el contenido.
    /// Los bytes deben estar en formato big-endian.
    fn crear_con_payload(payload: &[u8]) -> ContenidoReceiverReport {
        let contenido_report = ContenidoReport::crear_con_bytes(payload);

        ContenidoReceiverReport { contenido_report }
    }
}

impl From<&ContenidoReceiverReport> for Vec<u8> {
    fn from(contenido: &ContenidoReceiverReport) -> Self {
        let mut bytes = vec![];

        let bytes_report: Vec<u8> = Vec::from(&contenido.contenido_report);

        for byte in bytes_report {
            bytes.push(byte);
        }

        bytes
    }
}

#[derive(Debug, PartialEq)]
pub struct ContenidoFIR {}

#[derive(Debug, PartialEq)]
pub struct ContenidoNACK {}

#[derive(Debug, PartialEq)]
pub struct ContenidoFeedback {}

#[derive(Debug, PartialEq)]
pub struct ContenidoPayloadFeedback {}

/// Representación del contenido de un paquete RTCP de cualquier tipo
#[derive(Debug, PartialEq)]
pub enum ContenidoPaqueteRTCP {
    SenderReport(ContenidoSenderReport),
    ReceiverReport(ContenidoReceiverReport),
    Bye(),
    // De aca para abajo, decidir cuales vamos a implementar (son opcionales)
    FIR(ContenidoFIR),
    NACK(ContenidoNACK),
    Feedback(ContenidoFeedback),
    PayloadFeedback(ContenidoPayloadFeedback),
}

impl From<&ContenidoPaqueteRTCP> for Vec<u8> {
    fn from(contenido_paquete: &ContenidoPaqueteRTCP) -> Self {
        match contenido_paquete {
            ContenidoPaqueteRTCP::SenderReport(contenido) => Vec::from(contenido),
            ContenidoPaqueteRTCP::ReceiverReport(contenido) => Vec::from(contenido),
            ContenidoPaqueteRTCP::Bye() => vec![],
            // Se debera implementar From<Contenido...> para cualquier otro tipo de paquete que se quiera agregar
            _ => vec![],
        }
    }
}

impl ContenidoPaqueteRTCP {
    /// Crea un ContenidoPaqueteRTCP con un tipo valido (un `u8`), y una tira de bytes representado el contenido
    pub fn crear_con_tipo(
        num_tipo: u8,
        payload: &[u8],
    ) -> Result<ContenidoPaqueteRTCP, ErrorPaqueteRTCP> {
        match num_tipo {
            TIPO_PAQUETE_SR => Ok(ContenidoPaqueteRTCP::SenderReport(
                ContenidoSenderReport::crear_con_payload(payload),
            )),
            TIPO_PAQUETE_RR => Ok(ContenidoPaqueteRTCP::ReceiverReport(
                ContenidoReceiverReport::crear_con_payload(payload),
            )),
            TIPO_PAQUETE_BYE => Ok(ContenidoPaqueteRTCP::Bye()),
            // De aca para abajo no estan implementados. Por lo pronto dejo el contenido de cada uno como structs
            // vacios. Si se quiere implementar uno hay que definir los atributos e implementar crear_con_payload()
            TIPO_PAQUETE_FIR => Ok(ContenidoPaqueteRTCP::FIR(ContenidoFIR {})),
            TIPO_PAQUETE_NACK => Ok(ContenidoPaqueteRTCP::NACK(ContenidoNACK {})),
            TIPO_PAQUETE_FEEDBACK => Ok(ContenidoPaqueteRTCP::Feedback(ContenidoFeedback {})),
            TIPO_PAQUETE_PL_FEEDBACK => Ok(ContenidoPaqueteRTCP::PayloadFeedback(
                ContenidoPayloadFeedback {},
            )),
            _ => Err(ErrorPaqueteRTCP::TipoDePaqueteInvalido),
        }
    }

    /// Devuelve el tamanio del contenido del paquete RTCP, medido en bloques de 32 bits
    pub fn tamanio(&self) -> u16 {
        match self {
            Self::SenderReport(_) => TAMANIO_CONTENIDO_PAQUETE_SR,
            Self::ReceiverReport(_) => TAMANIO_CONTENIDO_PAQUETE_RR,
            Self::Bye() => TAMANIO_CONTENIDO_PAQUETE_BYE,
            // El resto de tipos de paquete no estan implementados, por lo pronto devuelvo 0
            _ => 0,
        }
    }

    /// Devuelve el numero asociado al tipo de paquete RTCP del contenido.
    pub fn numero_tipo(&self) -> u8 {
        match self {
            Self::SenderReport(_) => TIPO_PAQUETE_SR,
            Self::ReceiverReport(_) => TIPO_PAQUETE_RR,
            Self::Bye() => TIPO_PAQUETE_BYE,
            Self::FIR(_) => TIPO_PAQUETE_FIR,
            Self::Feedback(_) => TIPO_PAQUETE_FEEDBACK,
            Self::NACK(_) => TIPO_PAQUETE_NACK,
            Self::PayloadFeedback(_) => TIPO_PAQUETE_PL_FEEDBACK,
        }
    }
}
