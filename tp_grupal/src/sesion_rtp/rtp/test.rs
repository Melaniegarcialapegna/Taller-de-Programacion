#[cfg(test)]
use crate::logger::Logger;
//use crate::protocolos::rtcp::tipo_paquete::ContenidoReport;
//use crate::protocolos::rtp::paquete::PaqueteRTP;
#[cfg(test)]
use crate::seguridad::srtp::srtp_contexto::SRTPContexto;
#[cfg(test)]
use crate::sesion_rtp::comunicacion_rtp::*;
#[cfg(test)]
use crate::sesion_rtp::sesion::ContextosSRTP;
//use crate::sesion_rtp::sesion::EstadisticasReceiver;
#[cfg(test)]
use crate::sesion_rtp::sesion::EstadisticasSender;
#[cfg(test)]
use crate::sesion_rtp::socket_udp::MockSocketUdp;
#[cfg(test)]
use crate::sesion_rtp::socket_udp::SocketUDP;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex, mpsc};
//use std::thread;
#[cfg(test)]
fn mensaje_finalizacion_llamada() -> Vec<u8> {
    let mut mensaje_bytes: Vec<u8> = Vec::new();

    let mensaje = "END".as_bytes();

    for letra in mensaje {
        mensaje_bytes.push(*letra);
    }

    mensaje_bytes
}

#[test]
fn test01_iniciar_comunicacion_rtp() {
    //Se le envia al hilo que esta escuchando del socket que debe terminar
    let mensaje1 = vec![mensaje_finalizacion_llamada()];

    let mut socket_rtp = MockSocketUdp {
        bytes_enviados: Arc::new(Mutex::new(Vec::new())),
        bytes_que_se_leeran: mensaje1,
        posicion_lectura: 0,
    };

    let socket_mockeado = socket_rtp.clonar().unwrap();

    //Channel para simular camara
    let (sender_camara, receiver_camara) = mpsc::channel();
    //Channel para simular decodificador video
    let (sender_reproductor, _receiver_reproductor) = mpsc::channel::<Vec<u8>>();
    // Channel para simular audio
    let (sender_audio, receiver_audio) = mpsc::channel();

    let estadisticas_sender = Arc::new(Mutex::new(EstadisticasSender {
        ssrc: 242424,
        cantidad_paquetes_enviados: 0,
        cantidad_bytes_enviados: 0,
        ultimo_numero_secuencia: 0,
        ultimo_timestamp_enviado: 0,
    }));

    let estadisticas_receiver = Arc::new(Mutex::new(HashMap::new()));

    let mensaje2 = mensaje_finalizacion_llamada();

    //Se le envia al hilo que esta escuchando de la camara que debe terminar
    sender_camara
        .send(Frame::new(mensaje2, 20, 20))
        .expect("Fallo al enviar por channel");

    let logger = Logger::dummy_logger();

    let contexto_srtp_rx_1: SRTPContexto = SRTPContexto::new(Vec::new(), Vec::new());
    let contexto_srtp_rx_2: SRTPContexto = SRTPContexto::new(Vec::new(), Vec::new());
    let estadisticas: Estadisticas = (
        Arc::clone(&estadisticas_sender),
        estadisticas_sender,
        estadisticas_receiver,
    );
    let contextos: ContextosSRTP = (contexto_srtp_rx_1, contexto_srtp_rx_2);
    let comunicadores_con_io = ComunicadoresConIO::new(
        receiver_camara,
        sender_reproductor,
        receiver_audio,
        sender_audio,
    );

    let (_, receiver_srtp_rx) = mpsc::channel::<Vec<u8>>();

    let rtp_io = RtpIo {
        socket: socket_mockeado,
        srtp_rx: receiver_srtp_rx, // canal propio, no conectado a sender_reproductor
    };

    let resultado_iniciar_comunicacion = iniciar_comunicacion_rtp(
        logger,
        rtp_io,
        "127.0.0.1:8080",
        comunicadores_con_io,
        estadisticas,
        Arc::new(Mutex::new(false)),
        contextos,
    );

    // Se verifica que todo salga correcto sin fallas
    assert!(resultado_iniciar_comunicacion.is_ok());
}

// //Se corrobora que se lea bien del channel y se envie los datos de manera correcta hasta que se termine la conexion
// #[test]
// fn test02_manejo_envio_datagramas() {
//     let socket_rtp = MockSocketUdp {
//         bytes_enviados: Arc::new(Mutex::new(Vec::new())),
//         bytes_que_se_leeran: Vec::new(),
//         posicion_lectura: 0,
//     };

//     let bytes_enviados = Arc::clone(&socket_rtp.bytes_enviados);

//     //Channel para simular camara
//     let (sender_camara, receiver_camara) = mpsc::channel::<Vec<u8>>();

//     //Simulo que camara envie informacion
//     //Envio un byte de payload por paquete
//     sender_camara.send(vec![0x01]).unwrap();
//     sender_camara.send(vec![0x02]).unwrap();
//     sender_camara.send(vec![0x03]).unwrap();

//     let mensaje_finalizar = mensaje_finalizacion_llamada();

//     sender_camara.send(mensaje_finalizar).unwrap();

//     let estadisticas_sender = Arc::new(Mutex::new(EstadisticasSender {
//         ssrc: 242424,
//         cantidad_paquetes_enviados: 0,
//         cantidad_bytes_enviados: 0,
//         ultimo_numero_secuencia: 0,
//         ultimo_timestamp_enviado: 0,
//     }));

//     let ref_estadisticas = Arc::clone(&estadisticas_sender);

//     let mut estadistica_juguete = HashMap::new();
//     estadistica_juguete.insert(
//         14_u32,
//         EstadisticasReceiver {
//             cantidad_paquetes_recibidos: 14,
//             cantidad_paquetes_recibidos_anterior: 0,
//             cantidad_paquetes_esperados_anterior: 0,
//             contenido_report: ContenidoReport::crear_vacio_con_ssrc(14),
//         },
//     );

//     let estadisticas_receiver = Arc::new(Mutex::new(estadistica_juguete));

//     let contexto_srtp_rx: SRTPContexto = SRTPContexto::new(Vec::new(), Vec::new());

//     let handle_resultado = thread::spawn(move || {
//         manejo_envio_datagramas(
//             Box::new(socket_rtp),
//             "127.0000.0000",
//             receiver_camara,
//             ref_estadisticas,
//             estadisticas_receiver,
//             contexto_srtp_rx,
//         )
//     });

//     let resultado_iniciar_comunicacion = handle_resultado.join();

//     //Chequeo que todo salio correcto
//     assert!(resultado_iniciar_comunicacion.is_ok());

//     //Son 12 bytes de header + 1byte de payload => 13 bytes por paquete

//     let paquetes_enviados = bytes_enviados.lock().unwrap();

//     let cantidad_paquetes_enviados = paquetes_enviados.len() / 13;

//     assert_eq!(cantidad_paquetes_enviados, 3);

//     //Verifico que el payload sea el esperado
//     let paquete1 = paquetes_enviados[12];
//     assert_eq!(paquete1, 0x01);

//     let paquete2 = paquetes_enviados[25];
//     assert_eq!(paquete2, 0x02);

//     let paquete3 = paquetes_enviados[38];
//     assert_eq!(paquete3, 0x03);

//     //Verifico que numero secuencia sea la correcta
//     let num_sec_paquete_1 = u16::from_be_bytes([paquetes_enviados[2], paquetes_enviados[3]]);
//     assert_eq!(num_sec_paquete_1, 0);

//     //le voy sumando siempre 13
//     let num_sec_paquete_2 = u16::from_be_bytes([paquetes_enviados[15], paquetes_enviados[16]]);
//     assert_eq!(num_sec_paquete_2, 1);

//     let num_sec_paquete_3 = u16::from_be_bytes([paquetes_enviados[28], paquetes_enviados[29]]);
//     assert_eq!(num_sec_paquete_3, 2);

//     //Verifico que las estadisticas sean las correctas
//     let estadisticas = estadisticas_sender.lock().expect("Error : obtener el lock");
//     assert_eq!(estadisticas.cantidad_bytes_enviados, 13 * 3);
//     assert_eq!(estadisticas.cantidad_paquetes_enviados, 3);
// }

// #[test]
// fn test03_manejo_llegada_datagramas() {
//     let paquete_rtp_simulado = PaqueteRTP {
//         version: 2,
//         padding: 0,
//         extension: 0,
//         conteo_csrc: 0,
//         marcador: 0,
//         tipo_payload: 96,
//         numero_de_secuencia: 0,
//         timestamp: 0,
//         ssrc: 242424,
//         lista_csrc: Vec::new(),
//         payload: vec![0x01],
//         padding_bytes: 0,
//     };

//     let paquete_vec = Vec::from(&paquete_rtp_simulado);

//     let bytes_leera_socket = vec![paquete_vec, mensaje_finalizacion_llamada()];

//     let socket_rtp = MockSocketUdp {
//         bytes_enviados: Arc::new(Mutex::new(Vec::new())),
//         bytes_que_se_leeran: bytes_leera_socket,
//         posicion_lectura: 0,
//     };

//     //Channel para simular camara
//     let (sender_reproductor, _receiver_reproductor) = mpsc::channel::<Vec<u8>>();

//     let mut estadistica_juguete = HashMap::new();
//     estadistica_juguete.insert(
//         242424_u32,
//         EstadisticasReceiver {
//             cantidad_paquetes_recibidos: 0,
//             cantidad_paquetes_recibidos_anterior: 0,
//             cantidad_paquetes_esperados_anterior: 0,
//             contenido_report: ContenidoReport::crear_vacio_con_ssrc(242424),
//         },
//     );

//     let estadisticas_receiver = Arc::new(Mutex::new(estadistica_juguete));

//     let reproductor_procesando_frame = Arc::new(Mutex::new(true));

//     let ref_estadisticas_receiver = Arc::clone(&estadisticas_receiver);

//     let contexto_srtp_rx: SRTPContexto = SRTPContexto::new(Vec::new(), Vec::new());

//     let handle_resultado = thread::spawn(move || {
//         manejo_recepcion_datagramas(
//             Box::new(socket_rtp),
//             sender_reproductor,
//             ref_estadisticas_receiver,
//             reproductor_procesando_frame,
//             contexto_srtp_rx
//         )
//     });

//     let resultado_iniciar_comunicacion = handle_resultado.join();

//     //Chequeo que todo salio correcto
//     assert!(resultado_iniciar_comunicacion.is_ok());

//     let estadisticas = estadisticas_receiver
//         .lock()
//         .expect("Error : no se pudo obtener el lock");

//     let dicc = estadisticas
//         .get(&242424_u32)
//         .expect("Error : no existe la clave esperada");

//     assert_eq!(dicc.cantidad_paquetes_recibidos, 1_u32)
// }
