//! En este modulo se parsean los argumentos de la llamada al servidor.
use crate::errores::error_en_servidor::ErrorEnServidor;

/// Almacena una direccion.
#[derive(Debug, PartialEq)]
pub struct Entrada {
    pub direccion: String,
}

/// Devuelve una [`Entrada`] que contiene la direccion que posteriormente se utilizara para realizar la conexion.
///
/// Si se reciben menos parametros de los esperados se retornara un [`ErrorEnServidor`].
pub fn parsear_argumentos<T>(mut inputs: T) -> Result<Entrada, ErrorEnServidor>
where
    T: Iterator<Item = String>,
{
    inputs.next(); //se ignora el nombre del binario
    let direccion = inputs.next().ok_or(ErrorEnServidor::FaltaParametro)?;

    Ok(Entrada { direccion })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_parsear_argumentos_valida() {
        let inputs = vec!["skipear".to_string(), "127.0.0.1:8080".to_string()].into_iter();

        let entrada = parsear_argumentos(inputs).unwrap();
        assert_eq!(entrada.direccion, "127.0.0.1:8080");
    }

    #[test]
    fn test01_parsear_argumentos_invalida() {
        let inputs = vec!["skipear".to_string()].into_iter();

        let entrada = parsear_argumentos(inputs);
        assert_eq!(entrada.unwrap_err(), ErrorEnServidor::FaltaParametro);
    }
}
