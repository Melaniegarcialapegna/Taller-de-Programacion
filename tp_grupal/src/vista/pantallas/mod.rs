//! # Pantallas - Interfaz grafica de cada pantalla de [VistaEframe](crate::vista::vista_eframe::VistaEframe)
//!
//! En este modulo se debe implementar toda la logica relacionada a cada una de las pantallas
//! de la interfaz grafica [VistaEframe](crate::vista::vista_eframe::VistaEframe).
//!
//! Se provee el trait [Pantalla](self::pantalla::Pantalla), que sera la interfaz minima necesaria para que una pantalla pueda ser mostrada.
//! La documentación detallada del funcionamiento de esto se encuentra en la documentación del trait. El tipo [VistaEframe](crate::vista::vista_eframe::VistaEframe)
//! contiene una pantalla actual en la forma [Box<dyn Pantalla>], y en cada frame ejecutara el metodo renderizar de esa pantalla actual.

pub mod pantalla;
pub mod pantalla_iniciando_llamada;
pub mod pantalla_inicio;
pub mod pantalla_llamada;
pub mod pantalla_lobby;
pub mod pantalla_login;
pub mod pantalla_registro;
pub mod utils;
