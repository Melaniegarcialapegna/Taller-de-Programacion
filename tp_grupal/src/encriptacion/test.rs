#[cfg(test)]
use crate::encriptacion::SistemaDeEncriptacion;
#[cfg(test)]
use crate::encriptacion::TAMANIO_CLAVE_HASH;
#[cfg(test)]
use crate::encriptacion::mock_socket::MockStreamTcp;
#[cfg(test)]
use rsa::RsaPrivateKey;
#[cfg(test)]
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey};
#[cfg(test)]
use std::sync::{Arc, Mutex};

#[test]
fn test_01_se_intercambian_clave_publica_y_clave_hash() {
    // Creo par clave privada-clave publica
    let clave_privada_test = crear_clave_privada();

    let (_clave_publica_test, clave_publica_bytes) = obtener_clave_publica(&clave_privada_test);
    let clave_hash = obtener_clave_hash();

    // Creo mock para el socket.
    let mensajes_a_enviar = vec![clave_publica_bytes, clave_hash.as_bytes()[..].to_vec()];
    let mock_socket = MockStreamTcp::new(mensajes_a_enviar);
    let bytes_escritos_clon = Arc::clone(&mock_socket.bytes_escritos);

    // Creo mensaje encriptado y lo desencripto
    SistemaDeEncriptacion::encriptar_conexion(mock_socket).expect("Error encriptando conexion");

    let (primer_mensaje, _) = obtener_mensajes_escritos(bytes_escritos_clon);

    let resultado_clave_publica = RsaPublicKey::from_pkcs1_pem(&primer_mensaje);
    assert!(resultado_clave_publica.is_ok())
}

#[cfg(test)]
fn obtener_mensajes_escritos(bytes_escritos_clon: Arc<Mutex<Vec<Vec<u8>>>>) -> (String, String) {
    let mutex_bytes_escritos = bytes_escritos_clon.lock().unwrap();
    let bytes_primer_mensaje_escrito = mutex_bytes_escritos
        .first()
        .expect("No se recibio la clave publica")
        .clone();

    let bytes_segundo_mensaje_escrito = mutex_bytes_escritos
        .get(1)
        .expect("No se recibio la clave hash")
        .clone();

    let primer_mensaje = String::from_utf8(bytes_primer_mensaje_escrito).unwrap();
    let segundo_mensaje = String::from_utf8(bytes_segundo_mensaje_escrito).unwrap();
    (primer_mensaje, segundo_mensaje)
}

#[cfg(test)]
fn obtener_clave_hash() -> String {
    let mut bytes_random: Vec<u8> = vec![];

    for _ in 0..TAMANIO_CLAVE_HASH {
        let byte_random: u8 = rand::random();
        bytes_random.push((byte_random % 50) + 50);
    }

    String::from_utf8(bytes_random).unwrap()
}

#[cfg(test)]
fn crear_clave_privada() -> RsaPrivateKey {
    let thread_rng = rand::thread_rng();
    let mut rng = thread_rng;
    let bits = 1024;

    // Creo par clave privada-clave publica
    RsaPrivateKey::new(&mut rng, bits).unwrap()
}

#[cfg(test)]
fn obtener_clave_publica(clave_privada: &RsaPrivateKey) -> (RsaPublicKey, Vec<u8>) {
    use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::LineEnding};

    let clave_publica = RsaPublicKey::from(clave_privada);
    let clave_publica_str = clave_publica.to_pkcs1_pem(LineEnding::LF).unwrap();
    let clave_publica_bits = clave_publica_str.as_bytes().to_vec();
    (clave_publica, clave_publica_bits)
}
