//! Módulo 'claves_srtp.rs'
//! Este módulo define las estructuras necesarias para manejar las claves SRTP (Secure Real-time Transport Protocol)
//! en el contexto de DTLS (Datagram Transport Layer Security).
//! Proporciona las estructuras `ClavesSRTPBrutas` y `ClavesSRTP` que encapsulan las claves y salts necesarias
//! para cifrar y descifrar los flujos de medios seguros.

/// Struct que contiene las claves SRTP en su forma bruta.
#[derive(Debug, Clone)]
pub struct ClavesSRTPBrutas {
    pub client_key: [u8; 16],
    pub server_key: [u8; 16],
    pub client_salt: [u8; 14],
    pub server_salt: [u8; 14],
}

/// Struct que contiene las claves SRTP organizadas según el rol DTLS.
#[derive(Debug, Clone)]
pub struct ClavesSRTP {
    pub clave_tx: Vec<u8>, // Transmit key
    pub clave_rx: Vec<u8>, // Receive key
    pub salt_tx: Vec<u8>,  // Transmit salt
    pub salt_rx: Vec<u8>,  // Receive salt
}
