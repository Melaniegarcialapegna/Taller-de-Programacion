//! Módulo encargado de parsear los candidatos ICE (Interactive Connectivity Establishment)
//! presentes en líneas de SDP (Session Description Protocol) o vectores de strings.
//!
//! Cada línea de tipo `a=candidate:...` se convierte en una instancia del struct [`Candidato`],
//! realizando validaciones y registrando los resultados del parseo mediante el [`Logger`].
//!
//! # Descripción general
//! Este módulo forma parte del proceso de negociación de conectividad en WebRTC. Su responsabilidad es:
//! - Identificar líneas de candidatos ICE.
//! - Extraer los campos relevantes: **dirección IP**, **puerto** y **tipo** (`host`, `srflx`, `relay`).
//! - Validar los datos y construir los objetos [`Candidato`] correspondientes.
//! - Registrar en el log los resultados del parseo (éxitos y errores).
//!
//! # Formato esperado de una línea ICE
//! De acuerdo a la [RFC 5245 (ICE)], las líneas de candidatos suelen tener la forma:
//!
//! ```text
//! a=candidate:<foundation> <component-id> <transport> <priority> <ip> <port> typ <type> ...
//! ```
//!
//! # Funciones principales
//! - `parsear_candidatos()`: procesa un vector de strings de candidatos ICE y devuelve un vector de structs de tipo Candidato (o error).
//! - `parsear_linea()`: procesa una línea individual y crea un [`Candidato`].
//! - `tokenizar_linea()`: divide la línea en tokens individuales (por espacios).
//! - `extraer_campos()`: obtiene los valores de IP, puerto y tipo desde los tokens.
//! - `loguear_resultado_parseo()`: registra el resultado del parseo en el [`Logger`].

use super::candidato::Candidato;
use crate::logger::Logger;

const CANTIDAD_ESPERADA_TOKENS: usize = 8;
const TOKEN_IP: usize = 4;
const TOKEN_PUERTO: usize = 5;
const TOKEN_TIPO_KEYWORD: usize = 6;
const TOKEN_TIPO_VALOR: usize = 7;
pub const IDENTIFICADOR_CANDIDATO: &str = "a=candidate:";
const CONTEXTO_LOG: &str = "ICE parser";

/// Procesa un vector de líneas de candidatos ICE y devuelve un vector de resultados (`Result<Candidato, String>`).
///
/// Cada elemento del vector es un `Result<Candidato, String>`, donde:
/// - `Ok(Candidato)` representa un candidato válido.
/// - `Err(String)` contiene un mensaje de error en caso de fallo.
///
/// # Arguments
/// * `lineas` - Vector de strings con las líneas de candidatos ICE.
/// * `logger` - Referencia a un logger para registrar los resultados del parseo.
///
/// # Returns
/// Un vector con los resultados (`Vec<Result<Candidato, String>>`).
/// Procesa un vector de líneas de candidatos ICE y devuelve un vector de Result<Candidato, String>
pub fn parsear_candidatos(lineas: &Vec<String>, logger: &Logger) -> Vec<Result<Candidato, String>> {
    let mut candidatos = Vec::new();

    for linea in lineas {
        let linea = linea.trim();

        if linea.is_empty() || !linea.starts_with(IDENTIFICADOR_CANDIDATO) {
            continue;
        }

        let resultado = parsear_linea(linea);
        loguear_resultado_parseo(&resultado, logger);
        candidatos.push(resultado);
    }

    candidatos
}

/// Procesa una línea individual del texto SDP y devuelve un `Candidato` válido,
/// o un mensaje de error si la línea no cumple con el formato esperado.
///
/// # Arguments
/// * `linea` - Línea que comienza con `a=candidate:`.
///
/// # Returns
/// Un `Result<Candidato, String>` con el candidato o el error.
///
/// # Errores posibles
/// - La línea no comienza con `a=candidate:`.
/// - La cantidad de tokens es insuficiente.
/// - El puerto no es un número válido.
/// - El campo de tipo no es el esperado (`typ`).
pub fn parsear_linea(linea: &str) -> Result<Candidato, String> {
    // Solo tomo las que arranquen con a=candidate
    if !linea.starts_with(IDENTIFICADOR_CANDIDATO) {
        return Err(format!(
            "La línea seleccionada como candidato a parsear no respeta el formato esperado: {}",
            linea
        ));
    }

    let tokens = tokenizar_linea(linea)?;

    // La validación está hecha de acuerdo a un estándar específico RFC 5245 (ICE), después si es necesario lo ajustamos a nuestra implementación.
    if tokens.len() < CANTIDAD_ESPERADA_TOKENS {
        return Err(format!(
            "La cantidad de datos del candidato se encuentra incompleta: {}",
            linea
        ));
    }

    let (ip, puerto, tipo) = extraer_campos(&tokens)?;

    Candidato::crear_candidato(tipo, ip, puerto)
}

/// Divide una línea SDP en tokens individuales (separados por espacios),
/// eliminando el prefijo `a=candidate:`.
///
/// # Arguments
/// * `linea` - Línea que comienza con `a=candidate:`.
///
/// # Returns
/// Un vector de tokens (`Vec<&str>`).
///
/// # Errores posibles
/// - No se encontraron tokens después de eliminar el prefijo.
fn tokenizar_linea(linea: &str) -> Result<Vec<&str>, String> {
    // Saco el prefijo de candidato y tokenizo
    let contenido = linea.trim_start_matches(IDENTIFICADOR_CANDIDATO).trim();
    let tokens: Vec<&str> = contenido.split_whitespace().collect();

    if tokens.is_empty() {
        return Err(format!(
            "No se encontraron tokens a procesar en la línea {}",
            linea
        ));
    }

    Ok(tokens)
}

/// Extrae los campos de IP, puerto y tipo desde el vector de tokens.
/// Valida la presencia del token `typ` antes del tipo de candidato.
///
/// # Arguments
/// * `tokens` - Vector de tokens obtenidos de la línea SDP.
///
/// # Returns
/// Tupla `(ip, puerto, tipo)` en caso de éxito.
///
/// # Errores posibles
/// - El token en la posición esperada no es `typ`.
/// - El puerto no es un número válido.
fn extraer_campos(tokens: &[&str]) -> Result<(String, u16, String), String> {
    let ip = tokens[TOKEN_IP].to_string();
    let puerto_str = tokens[TOKEN_PUERTO];
    // Palabra fija que denota que a continuación se especificará el tipo de candidato
    let tipo_keyword = tokens[TOKEN_TIPO_KEYWORD];
    // Tipo de candidato
    let tipo_valor = tokens[TOKEN_TIPO_VALOR];

    if tipo_keyword != "typ" {
        return Err(format!(
            "Se esperaba 'typ' en la posición 7 pero se encontró {} en su lugar",
            tipo_keyword
        ));
    }

    let puerto = puerto_str
        .parse::<u16>()
        .map_err(|_| format!("El puerto {} no es válido", puerto_str))?;

    Ok((ip, puerto, tipo_valor.to_string()))
}

/// Registra en el log el resultado del parseo de una línea SDP.
/// - Si el parseo fue exitoso, se registra como `INFO`.
/// - Si ocurrió un error, se registra como `ERROR`.
fn loguear_resultado_parseo(resultado: &Result<Candidato, String>, logger: &Logger) {
    match resultado {
        Ok(candidato) => {
            logger.info(
                &format!(
                    "Candidato válido: tipo={}, ip={}, puerto={}",
                    candidato.getter_tipo(),
                    candidato.getter_ip(),
                    candidato.getter_puerto()
                ),
                CONTEXTO_LOG,
            );
        }
        Err(error) => {
            logger.error(
                &format!("Error al parsear candidato: {}", error),
                CONTEXTO_LOG,
            );
        }
    }
}

/// Convierte un candidato ICE en su representación de línea SDP.
pub fn candidato_a_sdp(foundation_id: usize, candidato: &Candidato) -> String {
    // usamos '1' para el component-id (RTP) y 'udp' como transporte.
    format!(
        "a=candidate:{} 1 udp {} {} {} typ {}",
        foundation_id,
        candidato.getter_prioridad(),
        candidato.getter_ip(),
        candidato.getter_puerto(),
        candidato.getter_tipo()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsear_candidatos_linea_valida() {
        let logger = Logger::dummy_logger();
        let candidatos =
            vec!["a=candidate:1 1 UDP 2130706431 192.168.1.2 54400 typ host".to_string()];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert_eq!(resultados.len(), 1);
        assert!(resultados[0].is_ok());
        let candidato = resultados[0].as_ref().unwrap();
        assert_eq!(candidato.getter_ip(), "192.168.1.2");
        assert_eq!(candidato.getter_puerto(), 54400);
        assert_eq!(candidato.getter_tipo(), "host");
    }

    #[test]
    fn test_parsear_candidatos_linea_invalida() {
        let logger = Logger::dummy_logger();
        let candidatos = vec!["a=candidate:1 1 UDP 2130706431 192.168.1.2 typ host".to_string()];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert_eq!(resultados.len(), 1);
        assert!(resultados[0].is_err());
    }

    #[test]
    fn test_parsear_candidatos_linea_vacia() {
        let logger = Logger::dummy_logger();
        let candidatos = vec!["".to_string()];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert!(resultados.is_empty());
    }

    #[test]
    fn test_parsear_candidatos_linea_sin_prefijo() {
        let logger = Logger::dummy_logger();
        let candidatos = vec!["c=IN IP4 192.168.1.2".to_string()];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert!(resultados.is_empty());
    }

    #[test]
    fn test_parsear_candidatos_varias_lineas() {
        let logger = Logger::dummy_logger();
        let candidatos = vec![
            "a=candidate:1 1 UDP 2130706431 10.0.0.1 5000 typ host".to_string(),
            "a=candidate:2 1 UDP 2130706431 10.0.0.2 5001 typ srflx".to_string(),
            "a=candidate:3 1 UDP 2130706431 10.0.0.3 5002 typ relay".to_string(),
        ];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert_eq!(resultados.len(), 3);
        assert!(resultados.iter().all(|r| r.is_ok()));
        let tipos: Vec<_> = resultados
            .into_iter()
            .map(|r| r.unwrap().getter_tipo().to_string())
            .collect();
        assert_eq!(tipos, vec!["host", "srflx", "relay"]);
    }

    #[test]
    fn test_parsear_candidatos_tipo_keyword_incorrecto() {
        let logger = Logger::dummy_logger();
        let candidatos =
            vec!["a=candidate:1 1 UDP 2130706431 192.168.1.2 54400 type host".to_string()];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert_eq!(resultados.len(), 1);
        assert!(resultados[0].is_err());
        assert!(
            resultados[0]
                .as_ref()
                .err()
                .unwrap()
                .contains("Se esperaba 'typ'")
        );
    }

    #[test]
    fn test_parsear_candidatos_puerto_invalido() {
        let logger = Logger::dummy_logger();
        let candidatos =
            vec!["a=candidate:1 1 UDP 2130706431 192.168.1.2 abc typ host".to_string()];
        let resultados = parsear_candidatos(&candidatos, &logger);
        assert_eq!(resultados.len(), 1);
        assert!(resultados[0].is_err());
        assert!(
            resultados[0]
                .as_ref()
                .err()
                .unwrap()
                .contains("no es válido")
        );
    }
}
