//! Módulo 'srtp_clave_salt.rs'
//! Este módulo define la estructura necesaria para manejar las claves y salts SRTP (Secure Real-time Transport Protocol)
//! en el contexto de DTLS (Datagram Transport Layer Security).

/// Struct que contiene la clave y el salt SRTP.
#[derive(Debug, Clone)]
pub struct SRTPClaveSalt {
    pub clave: Vec<u8>,
    pub salt: Vec<u8>,
}

/// Implementación de métodos para SRTPClaveSalt.
impl SRTPClaveSalt {
    pub fn new(clave: Vec<u8>, salt: Vec<u8>) -> Self {
        SRTPClaveSalt { clave, salt }
    }
}
