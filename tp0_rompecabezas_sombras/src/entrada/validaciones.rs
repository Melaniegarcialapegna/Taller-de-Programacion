//! Este modulo contiene las funciones encargadas de las validaciones.

use super::errores::Error;
use crate::utils::constantes::CANT_VALORES_ENTRADA_LINEA;

/// Valida que la cantidad de elementos en una linea sea la esperada.
///
/// # Parametros
/// - `elementos`: vector con los elementos que contiene la linea.
///
/// # Errores
/// Si la cantidad de elementos es la linea no es la esperada se retornara un [`Error`] .
///
pub fn validar_cantidad_elementos_entrada(elementos: &[&str]) -> Result<(), Error> {
    if elementos.len() < CANT_VALORES_ENTRADA_LINEA {
        return Err(Error::ValoresFaltantes);
    } else if elementos.len() > CANT_VALORES_ENTRADA_LINEA {
        return Err(Error::ValoresSobrantes);
    }
    Ok(())
}

/// Valida que un elemento este dentro de un rango pasado por parametro.
///
/// # Parametros
/// - `elemento`: el elemento al que se evaluara.
/// - `desde`: valor minimo que puede tomar el elemento.
/// - `hasta`: valor maximo que puede tomar el elemento.
///
/// # Tipos de datos genericos:
/// - `T` : tipo de dato del elemento, el cual debe implementar [`std::cmp::PartialOrd`] para poder hacer las operaciones `<` y `>`'
///
/// # Errores
/// Si el elemento no se encuentra en el rango se retornara un [`Error`] .
pub fn validar_rango<T: PartialOrd>(elemento: &T, desde: T, hasta: T) -> Result<(), Error> {
    if elemento < &desde || elemento > &hasta {
        return Err(Error::ValoresFueraDeRango);
    }
    Ok(())
}

/// Valida que la linea que se leyo no haya generado algun error o en caso de no haberlo ocasionado, que no este vacia.
///
/// # Parametros
/// - `linea`: contiene la el contenido de la linea leida.
/// - `lectura`: contiene el largo de la linea o un error.
///
/// # Errores
/// Si la linea esta vacia o hubo algun error al leerla se retornara un [`Error`] .
///
pub fn validar_entrada(
    linea: &mut str,
    lectura: Result<usize, std::io::Error>,
) -> Result<(), Error> {
    if lectura.is_err() {
        return Err(Error::LecturaIO);
    } else if linea.trim().is_empty() {
        return Err(Error::LineasFaltantes);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    //Test que valida el correcto funcionamiento de la funcion 'validar_cantidad_elementos_entrada'.
    #[test]
    fn test01_validar_cantidad_elementos_entrada_valida() {
        let elementos = vec!["hola", "mundo"];
        let resultado: Result<(), Error> = validar_cantidad_elementos_entrada(&elementos);
        assert_eq!(Ok(()), resultado);
    }

    //Test que valida que la funcion 'validar_cantidad_elementos_entrada' devuelva un [`Error::ValoresFaltantes`] o [`Error::ValoresSobrantes`] en caso de la cantidad de elementos no sea la esperada.
    #[test]
    fn test02_validar_cantidad_elementos_entrada_invalida() {
        let mut elementos = vec!["hola"];
        let mut resultado: Result<(), Error> = validar_cantidad_elementos_entrada(&elementos);
        assert_eq!(Err(Error::ValoresFaltantes), resultado);

        elementos = vec!["hola", "mundo", "feliz"];
        resultado = validar_cantidad_elementos_entrada(&elementos);
        assert_eq!(Err(Error::ValoresSobrantes), resultado);
    }

    //Test que valida el correcto funcionamiento de la funcion 'validar_rango'.
    #[test]
    fn test03_validar_rango_valido() {
        let elemento_1 = 5;
        let mut resultado: Result<(), Error> = validar_rango(&elemento_1, 2, 6);
        assert_eq!(Ok(()), resultado);

        let elemento_2 = 10.14;
        resultado = validar_rango(&elemento_2, 10.1, 16.5);
        assert_eq!(Ok(()), resultado);
    }

    //Test que valida que la funcion 'validar_rango' devuelva un [`Error::ValoresFueraDeRango`] en caso de que un elemento no este dentro del rango indicado.
    #[test]
    fn test04_validar_rango_invalido() {
        let elemento_1 = 5;
        let mut resultado: Result<(), Error> = validar_rango(&elemento_1, 6, 10);
        assert_eq!(Err(Error::ValoresFueraDeRango), resultado);

        let elemento_2 = 16.51;
        resultado = validar_rango(&elemento_2, 10.1, 16.5);
        assert_eq!(Err(Error::ValoresFueraDeRango), resultado);
    }

    //Test que valida el correcto funcionamiento de la funcion 'validar_entrada'.
    #[test]
    fn test05_validar_entrada_valida() {
        let mut linea = String::from("hola mundo");
        let lectura: Result<usize, std::io::Error> = Ok(linea.len()); //si se lee correctamente
        let resultado = validar_entrada(&mut linea, lectura);
        assert_eq!(Ok(()), resultado)
    }

    //Test que valida que la funcion 'validar_entrada' devuelva un [`Error::LineasFaltantes`] en caso de que una linea leida este vacia.
    #[test]
    fn test06_validar_entrada_vacia_invalida() {
        let mut linea = String::from(" ");
        let lectura: Result<usize, std::io::Error> = Ok(linea.len()); //si se lee correctamente
        let resultado = validar_entrada(&mut linea, lectura);
        assert_eq!(Err(Error::LineasFaltantes), resultado)
    }

    //Test que valida que la funcion 'validar_entrada' devuelva un [`Error::LecturaIO`] en caso de que al leer una linea se haya producido un error IO.
    #[test]
    fn test07_validar_entrada_lectura_invalida() {
        let mut linea = String::from("hola mundo");
        let lectura: Result<usize, std::io::Error> = Err(std::io::Error::other("error"));
        let resultado = validar_entrada(&mut linea, lectura);
        assert_eq!(Err(Error::LecturaIO), resultado)
    }
}
