//! El archivo de configuración debe tener el formato:
//! ```text
//! port: <número de puerto>
//! log_file: <ruta del archivo de log>
//! users_file: <ruta>
//!```
//!
//! Cada línea debe contener una clave y un valor separados por dos puntos y un espacio.
//!
//! Las líneas que inician por CARACTER_COMENTARIO son ignoradas.
use crate::utils_config::*;

pub struct ConfigServidor {
    host: String,
    port: u16,
    limite_usuarios: usize,
    log_file: String,
    users_file: String,
}

impl ConfigServidor {
    pub fn new(
        host: &str,
        port: u16,
        limite_usuarios: usize,
        log_file: &str,
        users_file: &str,
    ) -> ConfigServidor {
        ConfigServidor {
            host: host.to_string(),
            port,
            limite_usuarios,
            log_file: log_file.to_string(),
            users_file: users_file.to_string(),
        }
    }

    /// Getters del host
    pub fn getter_host(&self) -> &str {
        &self.host
    }

    /// Getters del port
    pub fn getter_port(&self) -> u16 {
        self.port
    }

    //getter limite usuarios
    pub fn getter_limite_usuarios(&self) -> usize {
        self.limite_usuarios
    }
    /// Getter del log_file
    pub fn getter_log_file(&self) -> &str {
        &self.log_file
    }
    /// Getter del archivo de info sobre usuarios
    pub fn getter_users_file(&self) -> &str {
        &self.users_file
    }

    pub fn get_direccion(&self) -> String {
        format!("{}:{}", &self.host, self.port)
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
    pub fn almacenar_config(ruta: &str) -> Result<ConfigServidor, String> {
        validar_archivo(ruta)?;
        let lector = abrir_lector(ruta)?;
        let configs = procesar_lineas(lector)?;

        let host = verificar_existencia_clave(&configs, "host")?;
        let port = verificar_existencia_clave(&configs, "port")?
            .parse::<u16>()
            .map_err(|error| format!("El valor de 'port' no es un número válido: '{}'", error))?;
        let limite_usuarios = verificar_existencia_clave(&configs, "limite_usuarios")?
            .parse::<usize>()
            .map_err(|error| {
                format!(
                    "El valor de 'limite_usuarios' no es un número válido: '{}'",
                    error
                )
            })?;
        let log_file = verificar_existencia_clave(&configs, "log_file")?;
        let users_file = verificar_existencia_clave(&configs, "users_file")?;

        Ok(ConfigServidor {
            host,
            port,
            limite_usuarios,
            log_file,
            users_file,
        })
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
        let ruta = "test_config_servidor_valido.conf";
        crear_archivo(
            ruta,
            "host: 127.0.0.1\nport: 9090\nlimite_usuarios: 4\nlog_file: /tmp/test.log\nusers_file: /tmp/users_file.info",
        );

        let config = ConfigServidor::almacenar_config(ruta).expect("Debe parsear correctamente");
        assert_eq!(config.getter_host(), "127.0.0.1");
        assert_eq!(config.getter_port(), 9090);
        assert_eq!(config.getter_limite_usuarios(), 4_usize);
        assert_eq!(config.getter_log_file(), "/tmp/test.log");
        assert_eq!(config.getter_users_file(), "/tmp/users_file.info");

        fs::remove_file(ruta).unwrap();
    }
}
