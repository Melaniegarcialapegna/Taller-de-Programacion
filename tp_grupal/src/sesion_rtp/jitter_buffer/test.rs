#[cfg(test)]
use crate::sesion_rtp::jitter_buffer::gestion_jitter_buffer::GestionJitterBuffer;
#[cfg(test)]
use crate::sesion_rtp::jitter_buffer::gestion_jitter_buffer::InformacionPaqueteRTP;
#[cfg(test)]
use crate::sesion_rtp::jitter_buffer::gestion_jitter_buffer::JitterBuffer;
#[cfg(test)]
use crate::sesion_rtp::jitter_buffer::gestion_jitter_buffer::JitterBufferVideo;
#[cfg(test)]
use crate::sesion_rtp::jitter_buffer::mensaje_jitter_buffer::MensajeJitterBuffer;
#[cfg(test)]
use std::sync::mpsc;
#[cfg(test)]
use std::thread;
#[cfg(test)]
use std::time::Duration;
#[cfg(test)]
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[test]
fn test01_envian_mensaje_finalizacion_llamada() {
    let jitter_buffer: JitterBuffer = Arc::new(Mutex::new(BTreeMap::new()));

    let ultimo_timestamp = Arc::new(Mutex::new(0));
    let llamada_finalizada = Arc::new(Mutex::new(false));
    let ref_llamada_finalizada = Arc::clone(&llamada_finalizada);
    let (tx_rtp, rx_rtp) = mpsc::channel::<MensajeJitterBuffer>();

    thread::spawn(move || {
        JitterBufferVideo::recibir_paquetes_rtp(
            jitter_buffer,
            30,
            rx_rtp,
            ultimo_timestamp,
            ref_llamada_finalizada,
        )
        .expect("Fallo al recibir_paquetes_rtp");
    });

    tx_rtp
        .send(MensajeJitterBuffer::MensajeFinalizacionLlamada)
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

    assert!(*lock)
}

#[test]
fn test02_envian_mensaje_timestamp_menor() {
    let jitter_buffer: JitterBuffer = Arc::new(Mutex::new(BTreeMap::new()));
    let ref_jitter_buffer = Arc::clone(&jitter_buffer);
    let ultimo_timestamp = Arc::new(Mutex::new(10));
    let llamada_finalizada = Arc::new(Mutex::new(false));
    let ref_llamada_finalizada = Arc::clone(&llamada_finalizada);
    let (tx_rtp, rx_rtp) = mpsc::channel::<MensajeJitterBuffer>();

    thread::spawn(move || {
        JitterBufferVideo::recibir_paquetes_rtp(
            ref_jitter_buffer,
            30,
            rx_rtp,
            ultimo_timestamp,
            ref_llamada_finalizada,
        )
        .expect("Fallo al recibir_paquetes_rtp");
    });

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 2,
                payload: Vec::new(),
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

    assert!(!(*lock));

    let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

    assert!((*lock_jitter_buffer).is_empty())
}

#[test]
fn test03_envian_mensaje_timestamp_se_agrega_a_arbol() {
    let jitter_buffer: JitterBuffer = Arc::new(Mutex::new(BTreeMap::new()));
    let ref_jitter_buffer = Arc::clone(&jitter_buffer);
    let ultimo_timestamp = Arc::new(Mutex::new(0));
    let llamada_finalizada = Arc::new(Mutex::new(false));
    let ref_llamada_finalizada = Arc::clone(&llamada_finalizada);
    let (tx_rtp, rx_rtp) = mpsc::channel::<MensajeJitterBuffer>();

    thread::spawn(move || {
        JitterBufferVideo::recibir_paquetes_rtp(
            ref_jitter_buffer,
            30,
            rx_rtp,
            ultimo_timestamp,
            ref_llamada_finalizada,
        )
        .expect("Fallo al recibir_paquetes_rtp");
    });

    let v1: Vec<u8> = vec![4_u8];
    let v2: Vec<u8> = vec![4_u8];

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 10,
                payload: v1,
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

    assert!(!(*lock));

    let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

    assert!((*lock_jitter_buffer).contains_key(&(10_u32)));

    let contenido = (*lock_jitter_buffer).get(&(10_u32));

    assert_eq!(contenido, Some(&v2))
}

#[test]
fn test04_envian_mensaje_tam_buffer_lleno() {
    let jitter_buffer: JitterBuffer = Arc::new(Mutex::new(BTreeMap::new()));
    let ref_jitter_buffer = Arc::clone(&jitter_buffer);
    let ultimo_timestamp = Arc::new(Mutex::new(0));
    let llamada_finalizada = Arc::new(Mutex::new(false));
    let ref_llamada_finalizada = Arc::clone(&llamada_finalizada);
    let (tx_rtp, rx_rtp) = mpsc::channel::<MensajeJitterBuffer>();

    thread::spawn(move || {
        JitterBufferVideo::recibir_paquetes_rtp(
            ref_jitter_buffer,
            1,
            rx_rtp,
            ultimo_timestamp,
            ref_llamada_finalizada,
        )
        .expect("Fallo al recibir_paquetes_rtp");
    });

    let v1: Vec<u8> = vec![4_u8];
    let v2: Vec<u8> = vec![4_u8];

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 10,
                payload: v1,
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    {
        let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

        assert!(!(*lock));

        let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

        assert!((*lock_jitter_buffer).contains_key(&(10_u32)));

        let contenido = (*lock_jitter_buffer).get(&(10_u32));

        assert_eq!(contenido, Some(&v2));
    } //libero lock

    //CAMBIO VALOR Y SE PISA PQ SE OCUPO TAMAÑO JITTER_BUFFER

    let v3: Vec<u8> = vec![4_u8];
    let v4: Vec<u8> = vec![4_u8];

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 4,
                payload: v3,
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

    assert!(!(*lock));

    let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

    assert!((*lock_jitter_buffer).contains_key(&(4_u32)));

    let contenido = (*lock_jitter_buffer).get(&(4_u32));

    assert_eq!(contenido, Some(&v4));

    //no contiene el anterior
    assert!(!(*lock_jitter_buffer).contains_key(&(10_u32)));
}

#[test]
fn test05_envian_mensaje_tam_buffer_lleno_se_elimina_timestamp_mas_peq() {
    let jitter_buffer: JitterBuffer = Arc::new(Mutex::new(BTreeMap::new()));
    let ref_jitter_buffer = Arc::clone(&jitter_buffer);
    let ultimo_timestamp = Arc::new(Mutex::new(0));
    let llamada_finalizada = Arc::new(Mutex::new(false));
    let ref_llamada_finalizada = Arc::clone(&llamada_finalizada);
    let (tx_rtp, rx_rtp) = mpsc::channel::<MensajeJitterBuffer>();

    thread::spawn(move || {
        JitterBufferVideo::recibir_paquetes_rtp(
            ref_jitter_buffer,
            2,
            rx_rtp,
            ultimo_timestamp,
            ref_llamada_finalizada,
        )
        .expect("Fallo al recibir_paquetes_rtp");
    });

    let v1: Vec<u8> = vec![4_u8];
    let v2: Vec<u8> = vec![4_u8];

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 10,
                payload: v1,
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    {
        let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

        assert!(!(*lock));

        let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

        assert!((*lock_jitter_buffer).contains_key(&(10_u32)));

        let contenido = (*lock_jitter_buffer).get(&(10_u32));

        assert_eq!(contenido, Some(&v2));
    } //libero lock

    //CAMBIO VALOR Y SE PISA PQ SE OCUPO TAMAÑO JITTER_BUFFER

    let v3: Vec<u8> = vec![4_u8];
    let v4: Vec<u8> = vec![4_u8];

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 4,
                payload: v3,
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    {
        let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

        assert!(!(*lock));

        let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

        assert!((*lock_jitter_buffer).contains_key(&(4_u32)));

        let contenido = (*lock_jitter_buffer).get(&(4_u32));

        assert_eq!(contenido, Some(&v4));
    }

    //SE AGREGA OTRO

    let v5: Vec<u8> = vec![4_u8];
    let v6: Vec<u8> = vec![4_u8];

    tx_rtp
        .send(MensajeJitterBuffer::InformacionPaqueteRTP(
            InformacionPaqueteRTP {
                timestamp: 12,
                payload: v5,
            },
        ))
        .expect("Fallo al enviar mensaje por channel");

    thread::sleep(Duration::from_secs(5));

    let lock = llamada_finalizada.lock().expect("Fallo obteniendo el lock");

    assert!(!(*lock));

    let lock_jitter_buffer = jitter_buffer.lock().expect("Fallo obteniendo el lock");

    assert!((*lock_jitter_buffer).contains_key(&(12_u32)));

    let contenido = (*lock_jitter_buffer).get(&(12_u32));

    assert_eq!(contenido, Some(&v6));

    //contiene la enterior que no era la mas pequeña
    assert!((*lock_jitter_buffer).contains_key(&(10_u32)));

    //no contiene el anterior mas pequeña
    assert!(!(*lock_jitter_buffer).contains_key(&(4_u32)));
}
