//! En este modulo se parsean los argumentos de la llamada al cliente.
use crate::errores::error_en_cliente::ErrorEnCliente;
use std::path::Path;

/// Almacena una direccion y el nombre de un archivo.
#[derive(Debug, PartialEq)]
pub struct Entrada {
    pub direccion: String,
    pub archivo: String,
}
/// Devuelve una [`Entrada`] que contiene la direccion que posteriormente se utilizara para realizar la conexion, junto con el nombre de un archivo del cual se leeran operaciones.
///
/// Si se reciben menos parametros de los esperados se retornara un [`ErrorEnCliente`].
pub fn parsear_argumentos<T>(mut inputs: T) -> Result<Entrada, ErrorEnCliente>
where
    T: Iterator<Item = String>,
{
    inputs.next();

    let direccion = inputs.next().ok_or(ErrorEnCliente::FaltaParametro)?;
    let input_file = inputs.next().ok_or(ErrorEnCliente::FaltaParametro)?;

    //se valida que el archivo exista
    if !Path::new(&input_file).exists() {
        return Err(ErrorEnCliente::ArchivoInexistente);
    }

    Ok(Entrada {
        direccion,
        archivo: input_file,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test01_parsear_argumentos_valida() {
        let inputs = vec![
            "skipear".to_string(),
            "127.0.0.1:8080".to_string(),
            "data/a.txt".to_string(),
        ]
        .into_iter();

        let entrada = parsear_argumentos(inputs).unwrap();
        assert_eq!(entrada.direccion, "127.0.0.1:8080");
        assert_eq!(entrada.archivo, "data/a.txt");
    }

    #[test]
    fn test02_parsear_argumentos_invalida() {
        test_parsear_argumentos_invalida_generica(
            vec!["skipear".to_string()],
            ErrorEnCliente::FaltaParametro,
        );

        test_parsear_argumentos_invalida_generica(
            vec!["skipear".to_string(), "127.0.0.1:8080".to_string()],
            ErrorEnCliente::FaltaParametro,
        );

        test_parsear_argumentos_invalida_generica(
            vec![
                "skipear".to_string(),
                "127.0.0.1:8080".to_string(),
                "data/no_existe.txt".to_string(),
            ],
            ErrorEnCliente::ArchivoInexistente,
        );
    }

    fn test_parsear_argumentos_invalida_generica(
        vector_args: Vec<String>,
        error_esperado: ErrorEnCliente,
    ) {
        let inputs = vector_args.into_iter();

        let entrada = parsear_argumentos(inputs);
        assert_eq!(entrada.unwrap_err(), error_esperado);
    }
}
