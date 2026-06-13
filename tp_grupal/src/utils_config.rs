//! Este modulo contiene funciones para leer y parsear el archivo de configuración.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Caracter que marca una línea como un comentario a ignorar en el archivo de configuración
const CARACTER_COMENTARIO: &str = "#";

// Función que se encarga de validar que el archivo exista
pub fn validar_archivo(ruta: &str) -> Result<(), String> {
    if !Path::new(ruta).exists() {
        return Err(format!("El archivo de configuración '{}' no existe.", ruta));
    }
    Ok(())
}

// Función que se encarga de abrir el archivo y devolver un lector
pub fn abrir_lector(ruta: &str) -> Result<io::BufReader<File>, String> {
    let archivo =
        File::open(ruta).map_err(|error| format!("Error al abrir el archivo: {}", error))?;
    Ok(io::BufReader::new(archivo))
}

// Función que se encarga de procesar las líneas del archivo y devolver un diccionario con las claves y valores -> lo implemente así porque
// tiene mejor escalabilidad en caso de que necesitemos (y seguro lo necesitemos) agregar más claves en el futuro, además queda mucho más prolijo
// que un enfoque con vectores o tuplas (que lo plantee primero y quedaba eterno y para escalar ese modelo teniamos que modificar un montón de cosas).
pub fn procesar_lineas(lector: io::BufReader<File>) -> Result<HashMap<String, String>, String> {
    let mut hash_config = HashMap::new();

    let lineas_iter = lector.lines().enumerate().peekable();

    for (indice, linea_res) in lineas_iter {
        let linea = linea_res.map_err(|e| format!("Error en línea {}: {}", indice + 1, e))?;
        let linea = linea.trim(); // limpio espacios al inicio y al final

        if linea.is_empty() || linea.starts_with(CARACTER_COMENTARIO) {
            continue;
        }

        procesar_linea(&mut hash_config, linea, indice + 1)?;
    }

    Ok(hash_config)
}

/// Procesa una línea simple de formato `clave:valor` y la inserta en el HashMap.
///
/// - Verifica duplicados de clave.
/// - Devuelve error si la línea no tiene el formato esperado.
pub fn procesar_linea(
    hash_config: &mut HashMap<String, String>,
    linea: &str,
    numero_linea: usize,
) -> Result<(), String> {
    let partes: Vec<&str> = linea.splitn(2, ':').collect();
    if partes.len() != 2 {
        return Err(format!(
            "Formato incorrecto en la línea {}: '{}'. Se esperaba 'clave: valor'.",
            numero_linea, linea
        ));
    }

    let clave = partes[0].trim();
    let valor = partes[1].trim();

    if hash_config.contains_key(clave) {
        return Err(format!(
            "Clave duplicada '{}' en línea {}",
            clave, numero_linea
        ));
    }
    if valor.is_empty() {
        return Err(format!(
            "Formato incorrecto en la línea {}: '{}'. Se esperaba 'clave: valor'.",
            numero_linea, linea
        ));
    }

    hash_config.insert(clave.to_string(), valor.to_string());
    Ok(())
}

// Función que se encarga de verificar que la clave que pedimos existe en el diccionario de configs.
pub fn verificar_existencia_clave(
    configs: &HashMap<String, String>,
    clave: &str,
) -> Result<String, String> {
    configs.get(clave).map(|v| v.to_string()).ok_or_else(|| {
        format!(
            "La clave '{}' no fue encontrada dentro del archivo de configuración.",
            clave
        )
    })
}
