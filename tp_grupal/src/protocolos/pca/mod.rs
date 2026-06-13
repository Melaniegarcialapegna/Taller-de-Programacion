//! # Implementación del protocolo Protocolo de Chocolate con Almendras (PCA)
//!
//! Los mensajes del protocolo son los siguientes:
//! - REGISTRAR usuario contraseña
//! - ENTRAR usuario contraseña
//! - LLAMAR usuario_destino
//! - LLAMANDO usuario_origen
//! - RECHAZO
//! - OFFER offer
//! - ANSWER answer
//! - USUARIOS usuario_1;(DISP|OCUP) …
//! - CORTAR
//! - SALIR
//! - ERROR mensaje
//! - OK
//!

/// Errores del protocolo
pub mod error;
/// Estado de los usuarios que devuelve el servidor
pub mod estado;
///Mensaje del protocolo
pub mod mensaje;
/// Tests del protocolo
pub mod tests;
/// Usuario devuelto por el servidor en el mensaje USUARIOS...
pub mod usuario;
/// Visitors para los mensajes del protocolo
pub mod visitor;
