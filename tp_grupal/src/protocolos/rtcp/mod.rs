//! # Implementación del protocolo RTCP
//! El estandar para este protocolo esta disponible en [RFC 1889](https://datatracker.ietf.org/doc/html/rfc1889).
//!
//! La implementación permite crear paquetes RTCP a partir de bytes, y tambien inicializar paquetes y convertirlos en bytes.

pub mod error;
pub mod paquete;
pub mod tests;
pub mod tipo_paquete;
