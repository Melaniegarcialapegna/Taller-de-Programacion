//! Módulo de parseo para el archivo de configuración.
//! El archivo de configuración debe tener el formato:
//! ```text
//! port: <número de puerto>
//! host_remoto: <dirección IP o nombre de host remoto>
//! port_remoto: <número de puerto remoto>
//! log_file: <ruta del archivo de log>
//! sdp_offer_file: <ruta>
//! sdp_answer_file: <ruta>
//! ```
//!
//! Cada línea debe contener una clave y un valor separados por dos puntos y un espacio.
//!
//! El módulo también maneja errores relacionados con la lectura del archivo y el parseo de los valores.
//! Las líneas que inician por CARACTER_COMENTARIO son ignoradas.

use crate::utils_config::*;

// Srtuct temporal que después o sacamos o movemos a otro archivo, para que el clippy no se queje con el too many arguments en el config
#[derive(Debug, Clone)]
pub struct Direcciones {
    host: String,
    port_rtp: u16,
    port_rtcp: u16,
}

impl Direcciones {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port_rtp: port,
            port_rtcp: port + 1,
        }
    }

    pub fn getter_host(&self) -> &str {
        &self.host
    }

    pub fn getter_port_rtp(&self) -> u16 {
        self.port_rtp
    }

    pub fn getter_port_rtcp(&self) -> u16 {
        self.port_rtcp
    }
}

// Implemento con derive Debug para facilitar los asserts en los tests
#[derive(Debug, Clone)]
pub struct ConfigRoomRTC {
    locales: Direcciones, // rtp y rtcp
    remotas: Direcciones, // rtp y rtcp
    log_file: String,
    sdp_offer_file: String,
    sdp_answer_file: String,
    stun_server: String,
    direccion_signaling: String,
}

impl ConfigRoomRTC {
    /// Crea una nueva instancia de `Config`.
    pub fn crear_struct(
        locales: Direcciones,
        remotas: Direcciones,
        log_file: String,
        sdp_offer_file: String,
        sdp_answer_file: String,
        stun_server: String,
        direccion_signaling: String,
    ) -> Self {
        Self {
            locales,
            remotas,
            log_file,
            sdp_offer_file,
            sdp_answer_file,
            stun_server,
            direccion_signaling,
        }
    }
    /// Getters del host
    pub fn getter_host_local(&self) -> &str {
        &self.locales.host
    }
    pub fn getter_host_remoto(&self) -> &str {
        &self.remotas.host
    }
    /// Getters del port
    pub fn getter_port_rtp_local(&self) -> u16 {
        self.locales.port_rtp
    }
    pub fn getter_port_rtp_remoto(&self) -> u16 {
        self.remotas.port_rtp
    }
    pub fn getter_port_rtcp_local(&self) -> u16 {
        self.locales.port_rtcp
    }
    pub fn getter_port_rtcp_remoto(&self) -> u16 {
        self.remotas.port_rtcp
    }
    /// Getter del log_file
    pub fn getter_log_file(&self) -> &str {
        &self.log_file
    }
    /// Getter del archivo sdp offer
    pub fn getter_sdp_offer_file(&self) -> &str {
        &self.sdp_offer_file
    }
    /// Getter del archivo sdp answer
    pub fn getter_sdp_answer_file(&self) -> &str {
        &self.sdp_answer_file
    }
    pub fn getter_stun_server(&self) -> &str {
        &self.stun_server
    }
    pub fn getter_direccion_signaling(&self) -> &str {
        &self.direccion_signaling
    }
    /// Lee y parsea el archivo de configuración.
    ///
    /// # Arguments
    /// * `ruta` - La ruta del archivo de configuración.
    ///
    /// # Returns
    /// Un `Result` que contiene la estructura `Config` o un `String` con el mensaje de error.
    ///
    /// # Errores
    /// * Si el archivo no existe o no se puede abrir, se devuelve un `String` con el mensaje de error.
    /// * Si el archivo tiene un formato incorrecto, se devuelve un `String` con el mensaje de error.
    /// * Si la clave `port` no es un número válido, se devuelve un `String` con el mensaje de error.
    pub fn almacenar_config(ruta: &str) -> Result<ConfigRoomRTC, String> {
        validar_archivo(ruta)?;
        let lector = abrir_lector(ruta)?;
        let configs = procesar_lineas(lector)?;

        let port = verificar_existencia_clave(&configs, "port")?
            .parse::<u16>()
            .map_err(|error| format!("El valor de 'port' no es un número válido: '{}'", error))?;
        let log_file = verificar_existencia_clave(&configs, "log_file")?;
        let sdp_offer_file = verificar_existencia_clave(&configs, "sdp_offer_file")?;
        let sdp_answer_file = verificar_existencia_clave(&configs, "sdp_answer_file")?;
        let stun_server = verificar_existencia_clave(&configs, "stun_server")?;
        let direccion_signaling = verificar_existencia_clave(&configs, "direccion_signaling")?;

        Ok(ConfigRoomRTC::crear_struct(
            Direcciones::new("0".to_string(), port), // el host lo cargamos después con if_addr
            Direcciones::new("0".to_string(), 0),    // inicialmente, vacíos, se cargan post-ICE
            log_file,
            sdp_offer_file,
            sdp_answer_file,
            stun_server,
            direccion_signaling,
        ))
    }

    pub fn establecer_informacion_conexion(
        &mut self,
        interfaz: String,
        puerto_rtp: u16,
        puerto_rtcp: u16,
    ) {
        self.remotas.host = interfaz;
        self.remotas.port_rtp = puerto_rtp;
        self.remotas.port_rtcp = puerto_rtcp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn crear_archivo(ruta: &str, contenido: &str) {
        let mut archivo = fs::File::create(ruta).unwrap();
        write!(archivo, "{}", contenido).unwrap();
    }

    // La estructura de los tests es:
    // 1. Creo un archivo temporal con el contenido a testear.
    // 2. Llamo a la función almacenar_config con la ruta del archivo temporal.
    // 3. Verifico que el resultado sea el esperado (ya sea un Config válido o un error específico).
    // 4. Elimino el archivo temporal.

    #[test]
    fn test_config_valido() {
        let ruta = "test_config_valido.conf";
        crear_archivo(
            ruta,
            "port: 8080\nlog_file: /tmp/test.log\nsdp_offer_file: /tmp/offer.sdp\nsdp_answer_file: /tmp/answer.sdp\nstun_server: stun.cloudflare.com:3478\ndireccion_signaling:0.0.0.0:3000\n",
        );

        let config = ConfigRoomRTC::almacenar_config(ruta).expect("Debe parsear correctamente");
        assert_eq!(config.getter_port_rtp_local(), 8080);
        assert_eq!(config.getter_log_file(), "/tmp/test.log");
        assert_eq!(config.getter_sdp_offer_file(), "/tmp/offer.sdp");
        assert_eq!(config.getter_sdp_answer_file(), "/tmp/answer.sdp");
        assert_eq!(config.getter_stun_server(), "stun.cloudflare.com:3478");

        fs::remove_file(ruta).unwrap();
    }

    #[test]
    fn test_archivo_inexistente() {
        let resultado = ConfigRoomRTC::almacenar_config("no_existe.conf");
        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("no existe"));
    }

    #[test]
    fn test_linea_mal_formateada() {
        let ruta = "test_linea_mal_formateada.conf";
        crear_archivo(ruta, "host=127.0.0.1\n");

        let resultado = ConfigRoomRTC::almacenar_config(ruta);
        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("Formato incorrecto"));

        fs::remove_file(ruta).unwrap();
    }

    #[test]
    fn test_falta_clave() {
        let ruta = "test_falta_clave.conf";
        crear_archivo(ruta, "host: 127.0.0.1\nlog_file: /tmp/log.log\n");

        let resultado = ConfigRoomRTC::almacenar_config(ruta);
        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("La clave 'port'"));

        fs::remove_file(ruta).unwrap();
    }

    #[test]
    fn test_puerto_invalido() {
        let ruta = "test_puerto_invalido.conf";
        crear_archivo(ruta, "host: localhost\nport: abc\nlog_file: /tmp/log.log\n");

        let resultado = ConfigRoomRTC::almacenar_config(ruta);
        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("no es un número válido"));

        fs::remove_file(ruta).unwrap();
    }

    #[test]
    fn test_clave_duplicada() {
        let ruta = "test_clave_duplicada.conf";
        crear_archivo(
            ruta,
            "host: localhost\nport: 8080\nhost: 192.168.0.1\nlog_file: /tmp/log.log\n",
        );

        let resultado = ConfigRoomRTC::almacenar_config(ruta);
        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("Clave duplicada"));

        fs::remove_file(ruta).unwrap();
    }
}
