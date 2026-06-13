//! Módulo `sesion`
//!
//! Contiene la estructura `SesionSdp`, que representa la sección de sesión
//! de un SDP (`v=`, `o=`, `s=`, `c=`, `t=`).
//!
//! Proporciona:
//! - Generación de sesión a partir de la configuración local.
//! - Serialización a formato SDP.
//! - Parseo seguro desde líneas de texto.
use super::traits::{ParseableSdp, SerializableSdp};
use if_addrs::get_if_addrs;
use rand::Rng;

/// Representa la parte superior del SDP (líneas v=, o=, s=, c=, t=)
#[derive(Debug, Clone)]
pub struct SesionSdp {
    version: u8,
    origen: String,
    nombre: String,
    conexion: String,
    tiempo: String,
    linea_group_bundle: Option<String>,
}

impl Default for SesionSdp {
    fn default() -> Self {
        Self::new()
    }
}

impl SesionSdp {
    /// Crea una nueva sección de "sesión"
    pub fn new() -> Self {
        let session_id: u64 = rand::thread_rng().r#gen(); // el uuid de la sesión, número aleatorio

        // intentamos obtener IP local
        let ip_local = match get_if_addrs() {
            Ok(addrs) => {
                let mut ip = "127.0.0.1".to_string(); // fallback
                for iface in addrs {
                    if !iface.is_loopback() && iface.ip().is_ipv4() {
                        ip = iface.ip().to_string();
                        break;
                    }
                }
                ip
            }
            Err(_) => {
                // si no se puede obtener interfaces, fallback
                "127.0.0.1".to_string()
            }
        };
        SesionSdp {
            version: 0,
            origen: format!("- {} 1 IN IP4 {}", session_id, ip_local),
            nombre: "-".to_string(),
            conexion: format!("IN IP4 {}", ip_local),
            tiempo: "0 0".to_string(),
            linea_group_bundle: None,
        }
    }

    // getters
    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_origen(&self) -> &str {
        &self.origen
    }

    pub fn get_nombre(&self) -> &str {
        &self.nombre
    }

    pub fn get_conexion(&self) -> &str {
        &self.conexion
    }

    pub fn get_tiempo(&self) -> &str {
        &self.tiempo
    }

    pub fn get_linea_group_bundle(&self) -> Option<String> {
        self.linea_group_bundle.clone()
    }
}

impl SerializableSdp for SesionSdp {
    fn serializar(&self) -> String {
        let mut sdp = format!(
            "v={}\r\no={}\r\ns={}\r\nc={}\r\nt={}\r\n",
            self.version, self.origen, self.nombre, self.conexion, self.tiempo
        );

        if let Some(ref bundle) = self.linea_group_bundle {
            sdp.push_str(&format!("a=group:BUNDLE {}\r\n", bundle));
        }

        sdp
    }
}

impl ParseableSdp for SesionSdp {
    /// Parsea un bloque de líneas SDP de sesión.
    /// Devuelve error si el campo "v" no se puede parsear como u8.
    fn parsear(lineas: &[&str]) -> Result<Self, String> {
        let mut sesion = SesionSdp {
            version: 0,
            origen: String::new(),
            nombre: String::new(),
            conexion: String::new(),
            tiempo: String::new(),
            linea_group_bundle: Some(String::new()),
        };
        for &linea in lineas {
            if let Some((clave, valor)) = linea.trim().split_once('=') {
                match clave {
                    "v" => {
                        sesion.version = valor
                            .parse::<u8>()
                            .map_err(|e| format!("Error parseando version: {}", e))?;
                    }
                    "o" => sesion.origen = valor.to_string(),
                    "s" => sesion.nombre = valor.to_string(),
                    "c" => sesion.conexion = valor.to_string(),
                    "t" => sesion.tiempo = valor.to_string(),
                    "a" => {
                        if valor.starts_with("group:BUNDLE") {
                            sesion.linea_group_bundle = Some(valor.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(sesion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_crea_sesion_valida() {
        let sesion = SesionSdp::new();

        assert_eq!(sesion.get_version(), 0);
        assert!(!sesion.get_origen().is_empty());
        assert_eq!(sesion.get_nombre(), "-");
        assert_eq!(sesion.get_tiempo(), "0 0");
    }

    #[test]
    fn serializar_sdp_contiene_todas_las_lineas() {
        let sesion = SesionSdp::new();
        let sdp = sesion.serializar();

        assert!(sdp.contains("v="));
        assert!(sdp.contains("o="));
        assert!(sdp.contains("s="));
        assert!(sdp.contains("c="));
        assert!(sdp.contains("t="));
    }

    #[test]
    fn parsear_reconstituye_sesion() {
        let sesion = SesionSdp::new();
        let sdp = sesion.serializar();
        let lineas: Vec<&str> = sdp.lines().collect();
        let sdp_parseado = SesionSdp::parsear(&lineas).unwrap();

        assert_eq!(sdp_parseado.get_version(), sesion.get_version());
        assert_eq!(sdp_parseado.get_origen(), sesion.get_origen());
        assert_eq!(sdp_parseado.get_conexion(), sesion.get_conexion());
    }
}
