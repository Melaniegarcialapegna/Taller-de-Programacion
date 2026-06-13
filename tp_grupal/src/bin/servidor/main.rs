mod conexiones;
use crate::conexiones::iniciar_servidor;
use room_rtc_2c_25::servidor_utils;
use room_rtc_2c_25::{config_servidor::ConfigServidor, logger::Logger};

fn main() {
    let input = std::env::args().collect::<Vec<String>>();

    if input.len() != 2 {
        eprintln!("El programa espera únicamente el archivo de configuración por parámetro.");
        return;
    }

    let ruta = &input[1];

    let config = match ConfigServidor::almacenar_config(ruta) {
        Ok(cfg) => {
            println!("Archivo de configuración cargado exitosamente");
            cfg
        }
        Err(err) => {
            eprintln!("Error al cargar configuración: {}", err);
            return;
        }
    };

    let logger = Logger::new(config.getter_log_file());
    logger.info("Iniciando Servidor", "main");

    if let Err(error) = iniciar_servidor(config) {
        eprintln!("{:?}", error);
    }
}
