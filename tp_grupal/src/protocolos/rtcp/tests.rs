#[cfg(test)]
use super::tipo_paquete::{
    ContenidoSenderReport, TIPO_PAQUETE_BYE, TIPO_PAQUETE_RR, TIPO_PAQUETE_SR,
};

#[cfg(test)]
use super::tipo_paquete::ContenidoReceiverReport;

#[cfg(test)]
use super::tipo_paquete::{
    ContenidoPaqueteRTCP, ContenidoReport, TIPO_PAQUETE_FEEDBACK, TIPO_PAQUETE_FIR,
    TIPO_PAQUETE_NACK, TIPO_PAQUETE_PL_FEEDBACK,
};

#[cfg(test)]
use super::error::ErrorPaqueteRTCP;

#[cfg(test)]
use super::paquete::{CONFIGURACIONES_DEFAULT, PaqueteRTCP};

#[cfg(test)]
const SSRC_PARA_TESTS: u32 = 24;

#[test]
fn test_01_falla_con_bytes_faltantes() {
    let bytes_contenido: [u8; 4] = [0; 4];

    let resultado_paquete = PaqueteRTCP::try_from(&bytes_contenido[..]);

    assert!(
        resultado_paquete.is_err(),
        "ERROR: Deberia rechazar un paquete incompleto (no tiene el header completo)"
    )
}

#[test]
fn test_02_rechaza_tipo_paquete_invalido() {
    let mut bytes_contenido: [u8; 20] = [0; 20];
    bytes_contenido[0] = CONFIGURACIONES_DEFAULT; // Seteo version 2 (unica version valida)
    bytes_contenido[1] = 27; // Tipo de paquete invalido

    let resultado_paquete = PaqueteRTCP::try_from(&bytes_contenido[..]);

    assert!(
        resultado_paquete.is_err(),
        "ERROR: Deberia rechazar un paquete con tipo valido"
    )
}

#[test]
fn test_03_rechaza_tipo_paquete_invalido() {
    let mut bytes_contenido: [u8; 20] = [0; 20];
    bytes_contenido[0] = CONFIGURACIONES_DEFAULT; // Seteo version 2 (unica version valida).
    bytes_contenido[1] = 27; // Tipo de paquete invalido.
    bytes_contenido[2] = 0; // Largo en paquetes de 32 bits menos 1 incluyendo header.
    bytes_contenido[3] = 0; // Source identifier, no es ninguno en nuestro caso.

    let resultado_paquete = PaqueteRTCP::try_from(&bytes_contenido[..]);

    assert!(resultado_paquete.is_err());
    assert!(matches!(
        resultado_paquete,
        Err(ErrorPaqueteRTCP::TipoDePaqueteInvalido)
    ));
}

#[test]
fn test_04_acepta_tipo_paquete_valido() {
    let bytes_header = crear_bytes_header(TIPO_PAQUETE_RR, 7);
    let bloque_paquetes_perdidos = (128 << 24) | 325;
    let bytes_payload: Vec<u32> = vec![0, bloque_paquetes_perdidos, 1, 2, 3, 4];

    let bytes_contenido = crear_bytes_paquete_entero(bytes_header, bytes_payload);

    let resultado_paquete = PaqueteRTCP::try_from(&bytes_contenido[..]);

    assert!(resultado_paquete.is_ok());
}

#[test]
fn test_05_rechaza_version_invalida() {
    let mut bytes_contenido: [u8; 20] = [0; 20];
    bytes_contenido[0] = 0x4 << 6; // Seteo version 4 (es invalida para WebRTC).
    bytes_contenido[1] = TIPO_PAQUETE_RR; // Tipo de paquete invalido.
    bytes_contenido[2] = 0; // Largo en paquetes de 32 bits menos 1 incluyendo header.
    bytes_contenido[3] = 0; // Source identifier, no es ninguno en nuestro caso.

    let resultado_paquete = PaqueteRTCP::try_from(&bytes_contenido[..]);

    assert!(resultado_paquete.is_err());
    assert!(matches!(
        resultado_paquete,
        Err(ErrorPaqueteRTCP::VersionInvalida)
    ))
}

#[test]
fn test_06_se_lee_exactamente_el_length() {
    let bytes_header = crear_bytes_header(TIPO_PAQUETE_RR, 7);
    let bloque_paquetes_perdidos = (128 << 24) | 325;
    let bytes_payload: Vec<u32> = vec![0, bloque_paquetes_perdidos, 1, 2, 3, 4];

    let bytes_contenido = crear_bytes_paquete_entero(bytes_header, bytes_payload);

    let paquete = PaqueteRTCP::try_from(&bytes_contenido[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    assert!(
        paquete.longitud_paquete == 32,
        "ERROR: El largo deberia ser 32"
    )
}

#[test]
fn test_07_se_envia_bien_el_ssrc() {
    let mut bytes_paquete_rr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_RR, vec![]);
    // Cambio el ssrc (el metodo usado arriba pone SSRC_PARA_TESTS)
    let ssrc: u32 = 1234;
    let ssrc_bytes = ssrc.to_be_bytes();
    bytes_paquete_rr[4] = ssrc_bytes[0];
    bytes_paquete_rr[5] = ssrc_bytes[1];
    bytes_paquete_rr[6] = ssrc_bytes[2];
    bytes_paquete_rr[7] = ssrc_bytes[3];

    let paquete = PaqueteRTCP::try_from(&bytes_paquete_rr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    assert!(paquete.ssrc == 1234)
}

#[test]
fn test_08_se_lee_contenido_correcto_de_tipo_correcto() {
    let bytes_paquete_sr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_SR, vec![45, 54, 32, 32, 32, 32]);

    let paquete_sr = PaqueteRTCP::try_from(&bytes_paquete_sr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    assert!(matches!(
        paquete_sr.payload,
        ContenidoPaqueteRTCP::SenderReport(_)
    ));
}

#[test]
fn test_09_se_lee_contenido_de_tipo_correcto_dos_tipos() {
    let bytes_paquete_sr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_SR, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_rr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_RR, vec![45, 54, 32, 32, 32]);

    let paquete_sr = PaqueteRTCP::try_from(&bytes_paquete_sr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_rr = PaqueteRTCP::try_from(&bytes_paquete_rr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    assert!(matches!(
        paquete_sr.payload,
        ContenidoPaqueteRTCP::SenderReport(_)
    ));
    assert!(matches!(
        paquete_rr.payload,
        ContenidoPaqueteRTCP::ReceiverReport(_)
    ));
}

#[test]
fn test_10_se_lee_contenido_de_tipo_correcto_todos_los_tipos() {
    let bytes_paquete_sr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_SR, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_rr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_RR, vec![45, 54, 32, 32, 32]);
    let bytes_paquete_fir =
        crear_bytes_paquete_rtcp(TIPO_PAQUETE_FIR, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_nack = crear_bytes_paquete_rtcp(TIPO_PAQUETE_NACK, vec![45, 54, 32, 32, 32]);
    let bytes_paquete_feedback =
        crear_bytes_paquete_rtcp(TIPO_PAQUETE_FEEDBACK, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_plfeedback =
        crear_bytes_paquete_rtcp(TIPO_PAQUETE_PL_FEEDBACK, vec![45, 54, 32, 32, 32]);

    let paquete_sr = PaqueteRTCP::try_from(&bytes_paquete_sr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_rr = PaqueteRTCP::try_from(&bytes_paquete_rr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_fir = PaqueteRTCP::try_from(&bytes_paquete_fir[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_nack = PaqueteRTCP::try_from(&bytes_paquete_nack[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_feedback = PaqueteRTCP::try_from(&bytes_paquete_feedback[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_plfeedback = PaqueteRTCP::try_from(&bytes_paquete_plfeedback[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    assert!(matches!(
        paquete_sr.payload,
        ContenidoPaqueteRTCP::SenderReport(_)
    ));
    assert!(matches!(
        paquete_rr.payload,
        ContenidoPaqueteRTCP::ReceiverReport(_)
    ));
    assert!(matches!(paquete_fir.payload, ContenidoPaqueteRTCP::FIR(_)));
    assert!(matches!(
        paquete_nack.payload,
        ContenidoPaqueteRTCP::NACK(_)
    ));
    assert!(matches!(
        paquete_feedback.payload,
        ContenidoPaqueteRTCP::Feedback(_)
    ));
    assert!(matches!(
        paquete_plfeedback.payload,
        ContenidoPaqueteRTCP::PayloadFeedback(_)
    ));
}

#[test]
fn test_11_se_lee_contenido_de_tipo_correcto_todos_los_tipos() {
    let bytes_paquete_sr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_SR, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_rr = crear_bytes_paquete_rtcp(TIPO_PAQUETE_RR, vec![45, 54, 32, 32, 32]);
    let bytes_paquete_fir =
        crear_bytes_paquete_rtcp(TIPO_PAQUETE_FIR, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_nack = crear_bytes_paquete_rtcp(TIPO_PAQUETE_NACK, vec![45, 54, 32, 32, 32]);
    let bytes_paquete_feedback =
        crear_bytes_paquete_rtcp(TIPO_PAQUETE_FEEDBACK, vec![45, 54, 32, 32, 32, 32]);
    let bytes_paquete_plfeedback =
        crear_bytes_paquete_rtcp(TIPO_PAQUETE_PL_FEEDBACK, vec![45, 54, 32, 32, 32]);

    let paquete_sr = PaqueteRTCP::try_from(&bytes_paquete_sr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_rr = PaqueteRTCP::try_from(&bytes_paquete_rr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_fir = PaqueteRTCP::try_from(&bytes_paquete_fir[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_nack = PaqueteRTCP::try_from(&bytes_paquete_nack[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_feedback = PaqueteRTCP::try_from(&bytes_paquete_feedback[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");
    let paquete_plfeedback = PaqueteRTCP::try_from(&bytes_paquete_plfeedback[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    assert!(matches!(
        paquete_sr.payload,
        ContenidoPaqueteRTCP::SenderReport(_)
    ));
    assert!(matches!(
        paquete_rr.payload,
        ContenidoPaqueteRTCP::ReceiverReport(_)
    ));
    assert!(matches!(paquete_fir.payload, ContenidoPaqueteRTCP::FIR(_)));
    assert!(matches!(
        paquete_nack.payload,
        ContenidoPaqueteRTCP::NACK(_)
    ));
    assert!(matches!(
        paquete_feedback.payload,
        ContenidoPaqueteRTCP::Feedback(_)
    ));
    assert!(matches!(
        paquete_plfeedback.payload,
        ContenidoPaqueteRTCP::PayloadFeedback(_)
    ));
}

#[test]
fn test_12_contenido_paquete_sr_es_valido() {
    let bytes_header = crear_bytes_header(TIPO_PAQUETE_SR, 12);
    let bloque_paquetes_perdidos = (128 << 24) | 325;
    let bytes_payload: Vec<u32> = vec![0, 1, 1, 2, 3, 0, bloque_paquetes_perdidos, 1, 2, 3, 4];

    let bytes_paquete_sr = crear_bytes_paquete_entero(bytes_header, bytes_payload);

    let paquete_sr = PaqueteRTCP::try_from(&bytes_paquete_sr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    let valores_esperados = vec![0, 325, 1, 2, 3, 4];

    if let ContenidoPaqueteRTCP::SenderReport(contenido_sr) = paquete_sr.payload {
        let contenido_report = &contenido_sr.contenido_report;
        let valores_obtenidos = crear_vector_valores_obtenidos_report(contenido_report);

        assert_coinciden_atributos_sender(&contenido_sr, 1, 1, 2, 3);
        assert_coinciden_atributos_report_rr(
            &contenido_sr.contenido_report,
            128,
            valores_esperados,
            valores_obtenidos,
        );
    } else {
        panic!()
    }
}

#[test]
fn test_13_contenido_paquete_rr_es_valido() {
    let bytes_header = crear_bytes_header(TIPO_PAQUETE_RR, 7);
    let bloque_paquetes_perdidos = (128 << 24) | 325;
    let bytes_payload: Vec<u32> = vec![0, bloque_paquetes_perdidos, 1, 2, 3, 4];

    let bytes_paquete_rr = crear_bytes_paquete_entero(bytes_header, bytes_payload);

    let paquete_rr = PaqueteRTCP::try_from(&bytes_paquete_rr[..])
        .expect("ERROR: Se deberia parsear una tira de bytes valida");

    let valores_esperados = vec![0, 325, 1, 2, 3, 4];

    if let ContenidoPaqueteRTCP::ReceiverReport(contenido_rr) = paquete_rr.payload {
        let contenido_report = contenido_rr.contenido_report;
        let valores_obtenidos = crear_vector_valores_obtenidos_report(&contenido_report);

        assert_coinciden_atributos_report_rr(
            &contenido_report,
            128,
            valores_esperados,
            valores_obtenidos,
        );
    } else {
        panic!()
    }
}

#[test]
fn test_14_contenido_paquete_bye_es_valido() {
    let bytes_header = crear_bytes_header(TIPO_PAQUETE_BYE, 2);
    let bytes_payload: Vec<u32> = vec![];
    let bytes_paquete = crear_bytes_paquete_entero(bytes_header, bytes_payload);

    let paquete =
        PaqueteRTCP::try_from(&bytes_paquete[..]).expect("DEBERIA PODER CREARSE EL TIPO BYE");

    assert!(matches!(paquete.payload, ContenidoPaqueteRTCP::Bye()))
}

#[test]
fn test_15_paquete_bye_se_convierte_en_bytes() {
    let paquete = PaqueteRTCP::crear(0, ContenidoPaqueteRTCP::Bye());
    let tamanio: u16 = 1;

    let bytes_paquete = Vec::from(&paquete);
    let bytes_tamanio_be = tamanio.to_be_bytes();
    let valores_esperados: Vec<u8> = vec![
        CONFIGURACIONES_DEFAULT,
        TIPO_PAQUETE_BYE,
        bytes_tamanio_be[0],
        bytes_tamanio_be[1],
        0,
        0,
        0,
        0,
    ];

    assert!(bytes_paquete.len() == 8);
    for i in 0..bytes_paquete.len() {
        assert!(bytes_paquete[i] == valores_esperados[i])
    }
}

#[test]
fn test_16_paquete_sr_se_convierte_en_bytes() {
    let contenido_report_paquete = ContenidoReport {
        ssrc_report: 1,
        frac_paquetes_perdidos: 2,
        cant_paquetes_perdidos: 3,
        numero_mas_grande_de_paquete_recibido: 4,
        tiempo_est_entre_paquetes: 5,
        tiempo_desde_ultimo_paquete: 6,
        delay_desde_ultimo_paquete: 7,
    };

    let contenido_paquete = ContenidoSenderReport {
        ntp_timestamp: 1,
        rtp_timestamp: 2,
        sender_octet_count: 3,
        sender_packet_count: 4,
        contenido_report: contenido_report_paquete,
    };

    let paquete = PaqueteRTCP::crear(0, ContenidoPaqueteRTCP::SenderReport(contenido_paquete));

    let bytes_paquete = Vec::from(&paquete);

    let paquete_rearmado = PaqueteRTCP::try_from(bytes_paquete.as_slice())
        .expect("ERROR: Se deberia poder crear un paquete a partir de los bytes resultantes");

    assert_eq!(paquete_rearmado, paquete);
}

#[test]
fn test_17_paquete_rr_se_convierte_en_bytes() {
    let contenido_report_paquete = ContenidoReport {
        ssrc_report: 1,
        frac_paquetes_perdidos: 2,
        cant_paquetes_perdidos: 3,
        numero_mas_grande_de_paquete_recibido: 4,
        tiempo_est_entre_paquetes: 5,
        tiempo_desde_ultimo_paquete: 6,
        delay_desde_ultimo_paquete: 7,
    };

    let contenido_paquete = ContenidoReceiverReport {
        contenido_report: contenido_report_paquete,
    };

    let paquete = PaqueteRTCP::crear(0, ContenidoPaqueteRTCP::ReceiverReport(contenido_paquete));

    let bytes_paquete = Vec::from(&paquete);

    let paquete_rearmado = PaqueteRTCP::try_from(bytes_paquete.as_slice())
        .expect("ERROR: Se deberia poder crear un paquete a partir de los bytes resultantes");

    assert_eq!(paquete_rearmado, paquete);
}

#[cfg(test)]
fn crear_bytes_paquete_entero(bytes_header: [u8; 8], bytes_payload_vec: Vec<u32>) -> [u8; 64] {
    let mut bytes_paquete: [u8; 64] = [0; 64];

    for (i, byte) in bytes_header.iter().enumerate() {
        bytes_paquete[i] = *byte;
    }

    for (i, bloque) in bytes_payload_vec.iter().enumerate() {
        let bytes_bloque_be = bloque.to_be_bytes();
        bytes_paquete[8 + (4 * i)] = bytes_bloque_be[0];
        bytes_paquete[8 + (4 * i) + 1] = bytes_bloque_be[1];
        bytes_paquete[8 + (4 * i) + 2] = bytes_bloque_be[2];
        bytes_paquete[8 + (4 * i) + 3] = bytes_bloque_be[3];
    }

    bytes_paquete
}

#[cfg(test)]
fn crear_vector_valores_obtenidos_report(contenido_report: &ContenidoReport) -> Vec<u32> {
    vec![
        contenido_report.ssrc_report,
        contenido_report.cant_paquetes_perdidos,
        contenido_report.numero_mas_grande_de_paquete_recibido,
        contenido_report.tiempo_est_entre_paquetes,
        contenido_report.tiempo_desde_ultimo_paquete,
        contenido_report.delay_desde_ultimo_paquete,
    ]
}

#[cfg(test)]
fn assert_coinciden_atributos_sender(
    contenido_sr: &ContenidoSenderReport,
    ntp_timestamp: u64,
    rtp_timestamp: u32,
    sender_packet_count: u32,
    sender_octet_count: u32,
) {
    assert!(contenido_sr.ntp_timestamp == ntp_timestamp);
    assert!(contenido_sr.rtp_timestamp == rtp_timestamp);
    assert!(contenido_sr.sender_packet_count == sender_packet_count);
    assert!(contenido_sr.sender_octet_count == sender_octet_count);
}

#[cfg(test)]
fn assert_coinciden_atributos_report_rr(
    contenido_rep: &ContenidoReport,
    frac_paquetes_perdidos: u8,
    valores_esperados: Vec<u32>,
    valores_obtenidos: Vec<u32>,
) {
    assert!(contenido_rep.frac_paquetes_perdidos == frac_paquetes_perdidos);

    for i in 0..valores_esperados.len() {
        assert!(valores_esperados[i] == valores_obtenidos[i])
    }
}

#[cfg(test)]
fn crear_bytes_paquete_rtcp(tipo_paquete: u8, payload: Vec<u32>) -> [u8; 200] {
    let mut bytes_contenido: [u8; 200] = [0; 200];
    let largo: u16 = (payload.len() + 1).try_into().unwrap();
    let bytes_header = crear_bytes_header(tipo_paquete, largo);

    for (i, byte) in bytes_header.iter().enumerate() {
        bytes_contenido[i] = *byte;
    }

    for (i, bloque_datos) in payload.iter().enumerate() {
        let indice = 8 + (i * 4);
        let bytes_bloque_datos = bloque_datos.to_be_bytes();
        bytes_contenido[indice] = bytes_bloque_datos[0];
        bytes_contenido[indice + 1] = bytes_bloque_datos[1];
        bytes_contenido[indice + 2] = bytes_bloque_datos[2];
        bytes_contenido[indice + 3] = bytes_bloque_datos[3];
    }

    bytes_contenido
}

#[cfg(test)]
fn crear_bytes_header(num_tipo_paquete: u8, largo: u16) -> [u8; 8] {
    let mut bytes_header: [u8; 8] = [0; 8];

    bytes_header[0] = CONFIGURACIONES_DEFAULT;
    bytes_header[1] = num_tipo_paquete;

    let largo_be_bytes = largo.to_be_bytes();
    bytes_header[2] = largo_be_bytes[0];
    bytes_header[3] = largo_be_bytes[1];

    let ssrc_be_bytes = SSRC_PARA_TESTS.to_be_bytes();
    bytes_header[4] = ssrc_be_bytes[0];
    bytes_header[5] = ssrc_be_bytes[1];
    bytes_header[6] = ssrc_be_bytes[2];
    bytes_header[7] = ssrc_be_bytes[3];

    bytes_header
}
