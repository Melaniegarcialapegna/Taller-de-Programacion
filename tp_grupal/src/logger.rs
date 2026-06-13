//! Módulo que define el logger del servidor, permitiendo registrar mensajes de diferentes niveles (INFO, WARN, ERROR) en un archivo de log.
//! El logger utiliza un channel para enviar mensajes a un thread dedicado que escribe en el archivo de log.
//! Esto evita bloqueos en el thread principal del servidor al realizar operaciones de escritura en disco.
//!
//! # Funciones principales:
//! - `new(archivo_log: &str) -> Logger`: Crea un nuevo logger que escribe en el archivo especificado.
//! - `info(&self, mensaje: &str, contexto: &str)`: Registra un mensaje de nivel INFO.
//! - `warn(&self, mensaje: &str, contexto: &str)`: Registra un mensaje de nivel WARN.
//! - `error(&self, mensaje: &str, contexto: &str)`: Registra un mensaje de nivel ERROR.
//! - `dummy_logger() -> Logger`: Crea un logger de prueba que no escribe en ningún archivo (utilizado en tests).
//!
//! # Formato de los mensajes de log:
//! Los mensajes de log tienen el formato: `hora - tipo_mensaje - contexto [thread_id] : mensaje`
//! donde `hora` es el timestamp en segundos desde epoch (punto de referencia UNIX), `tipo_mensaje` puede ser INFO, WARN o ERROR, `contexto` es un string que
//! indica donde se generó el mensaje, `thread_id` es el identificador del thread que generó el mensaje, y `mensaje` es el contenido del mensaje.

use std::{
    fs::OpenOptions,
    io::Write,
    sync::mpsc::{Sender, channel},
    thread::spawn,
    time::{SystemTime, UNIX_EPOCH},
};

/// Obtiene el timestamp actual en segundos desde epoch.
/// Si ocurre un error, retorna "0".
///
/// # Returns
/// Un `String` con el timestamp en segundos.
fn timestamp_actual() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duracion) => format!("{}", duracion.as_secs()),
        Err(_) => "0".to_string(),
    }
}

#[derive(Clone)]
pub struct Logger {
    pub logger: Sender<String>,
}

impl Logger {
    /// Crea un nuevo logger que escribe en el archivo especificado.
    /// Si no puede abrir el archivo, imprime un error en stderr y el logger no funciona.
    /// El logger utiliza un thread dedicado a escribir en el archivo, evitando bloqueos en el thread principal.
    ///
    /// # Arguments
    /// * `archivo_log` - La ruta del archivo donde se escribirán los logs.
    ///
    /// # Returns
    /// Una instancia de `Logger`.
    pub fn new(archivo_log: &str) -> Logger {
        let (tx, rx) = channel();
        let path = archivo_log.to_string();

        spawn(move || {
            let archivo = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path);

            let mut archivo = match archivo {
                Ok(archivo) => archivo,
                Err(error) => {
                    eprintln!("Failed to open log file: {}", error);
                    return;
                }
            };

            for mensaje in rx {
                if let Err(error) = writeln!(archivo, "{}", mensaje) {
                    eprintln!("Failed to write on log file: {}", error);
                }
            }
        });

        Logger { logger: tx }
    }

    /// Función interna para registrar un mensaje de log con el formato adecuado.
    ///
    /// # Arguments
    /// * `nivel` - El nivel del mensaje (INFO, WARN, ERROR).
    /// * `mensaje` - El contenido del mensaje.
    /// * `contexto` - El contexto donde se generó el mensaje.
    ///
    /// # Ejemplo de uso
    /// ``` ignore
    /// logger.log("INFO", "Servidor iniciado", "main");
    /// ```
    fn log(&self, nivel: &str, mensaje: &str, contexto: &str) {
        let hora = timestamp_actual();
        let thread_id = format!("{:?}", std::thread::current().id());
        let log_mensaje = format!(
            "{} - {} - {} [{}] : {}",
            hora, nivel, contexto, thread_id, mensaje
        );

        match self.logger.send(log_mensaje) {
            Ok(_) => (),
            Err(error) => eprintln!("Failed to send log message: {}", error),
        }
    }

    /// Registra un mensaje de tipo ERROR.
    ///
    /// # Arguments
    /// * `mensaje` - El contenido del mensaje.
    /// * `contexto` - El contexto donde se generó el mensaje.
    /// # Ejemplo de uso
    /// ``` ignore
    /// logger.error("failed to parse request", "server");
    /// ```
    pub fn error(&self, mensaje: &str, contexto: &str) {
        self.log("ERROR", mensaje, contexto);
    }

    /// Registra un mensaje de tipo INFO.
    ///
    /// # Arguments
    /// * `mensaje` - El contenido del mensaje.
    /// * `contexto` - El contexto donde se generó el mensaje.
    /// # Ejemplo de uso
    /// ``` ignore
    /// logger.info("GET request received", "server");
    /// ```
    pub fn info(&self, mensaje: &str, contexto: &str) {
        self.log("INFO", mensaje, contexto);
    }

    /// Registra un mensaje de tipo WARN.
    ///
    /// # Arguments
    /// * `mensaje` - El contenido del mensaje.
    /// * `contexto` - El contexto donde se generó el mensaje.
    ///
    /// # Ejemplo de uso
    /// ``` ignore
    /// logger.warn("unknown problem occurred", "server");
    /// ```
    pub fn warn(&self, mensaje: &str, contexto: &str) {
        self.log("WARN", mensaje, contexto);
    }

    /// Crea un logger de prueba que no escribe en ningún archivo.
    /// Utilizado en tests para evitar crear archivos de log reales.
    ///
    /// # Returns
    /// Una instancia de `Logger` que no escribe en ningún archivo.
    /// Los mensajes enviados a este logger se descartan.
    #[cfg(test)]
    pub fn dummy_logger() -> Logger {
        let (tx, _rx) = channel();

        spawn(move || {
            loop {
                std::thread::park();
            }
        });

        Logger { logger: tx }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_logger_no_hace_panic() {
        let logger = Logger::dummy_logger();
        logger.info("mensaje info", "test");
        logger.error("mensaje error", "test");
        logger.warn("mensaje warn", "test");
        //si no hay panic, el test pasa
    }

    #[test]
    fn test_timestamp_actual_no_vacio() {
        let ts = timestamp_actual();
        assert!(!ts.is_empty());
    }
}
