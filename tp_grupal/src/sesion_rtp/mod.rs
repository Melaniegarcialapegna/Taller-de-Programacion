//! # Sesion RTP - Envio y recepción de mensajes RTP y RTCP
//!
//! Modulo que contiene la logica de envio y recepción de mensajes RTP y RTCP. Implementado según el estandar RFC 1889
//!
//! ## Threads
//!
//! La comunicación esta separada en 2 secciones:
//! - RTP, donde se recibiran y enviaran paquetes de datos con el video transmitido
//! - RTCP, donde se recibiran y enviaran paquetes para controlar los datos enviados por RTP
//!
//! Cada una de estas partes administra los mensajes en dos threads separados, uno para el envio de mensajes y otro para la escucha y procesamiento.
//!
//! ## Conexión con GUI y camara
//!
//! El modulo Comunicación RTP recibe paquetes de datos de video desde la camara mediante un *channel* `mpsc`. Luego, crea un paquete RTP con esos datos y los envia mediante el socket.
//!
//! Por otro lado, al recibir paquetes RTP con datos de video, estos se envian tambien mediante *channels* `mpsc` al reproductor de video, que procesara y mostrara los frames en la GUI.

pub mod comunicacion_rtcp;
pub mod comunicacion_rtp;
pub mod error;
pub mod jitter_buffer;
pub mod rtp;
pub mod sesion;
pub mod socket_udp;
