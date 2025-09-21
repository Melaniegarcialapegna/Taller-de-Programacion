//! Este modulo contiene las funciones que se encargan de procesar una linea.

use super::errores::Error;
use super::validaciones::*;

/// Procesa una linea pasada por parametro, la cual contiene dos elementos separados por un espacio.
///
/// # Parametros
/// - `linea` : la linea que se procesara.
/// - `rango_min_elemento_1` : el valor minimo que puede tomar el 'elemento_1'
/// - `rango_min_elemento_1` : el valor maximo que puede tomar el 'elemento_1'
/// - `rango_min_elemento_2` : el valor minimo que puede tomar el 'elemento_2'
/// - `rango_max_elemento_2` : el valor maximo que puede tomar el 'elemento_2'
///
/// # Tipos de datos genericos:
/// - `T` : tipo de dato del elemento_1, este debe implementar a [`std::str::FromStr`] y [`PartialOrd`].
/// - `K` : tipo de dato del elemento_2, este debe implementar a [`std::str::FromStr`] y [`PartialOrd`].
///
/// # Errores
/// Se retornara un [`Error`] si alguno de los elementos no cumple con el formato indicado o si la linea no contiene la cantidad de elementos esperada.
///
pub fn procesar_linea<T, K>(
    linea: &str,
    rango_min_elemento_1: T,
    rango_max_elemento_1: T,
    rango_min_elemento_2: K,
    rango_max_elemento_2: K,
) -> Result<(T, K), Error>
where
    T: (std::str::FromStr) + PartialOrd,
    K: (std::str::FromStr) + PartialOrd,
{
    let elementos: Vec<&str> = linea.split_whitespace().collect();

    validar_cantidad_elementos_entrada(&elementos)?;

    let elemento_1: T = parsear(elementos[0])?;
    let elemento_2: K = parsear(elementos[1])?;

    validar_rango(&elemento_1, rango_min_elemento_1, rango_max_elemento_1)?;
    validar_rango(&elemento_2, rango_min_elemento_2, rango_max_elemento_2)?;

    Ok((elemento_1, elemento_2))
}
/// Cambia el tipo de dato del elemento
///
/// # Parametros
/// - `elemento` : al cual se pretende cambiarle el tipo de dato.
///
/// # Tipos de datos genericos:
/// - `T` : al dato al que se quiera mutar debe implementar [`std::str::FromStr`], para poder utilizar la funcion 'parse()'
///
/// # Errores
/// Se retornara un [`Error`] si no es posible cambiar el tipo de dato del elemento.
///
pub fn parsear<T: std::str::FromStr>(elemento: &str) -> Result<T, Error> {
    elemento.parse::<T>().map_err(|_| Error::ParseoNumero)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    //Test que valida el correcto funcionamiento de la funcion 'procesar_linea'.
    #[test]
    fn test01_procesar_linea_valida() {
        let mut linea = String::from("1 5");
        let resultado_1 = procesar_linea(&linea, 0, 5, 0, 5);
        assert_eq!(Ok((1, 5)), resultado_1);

        linea = String::from("  2.2  4.4 ");
        let resultado_2 = procesar_linea(&linea, 1.2, 5.4, 4.33, 5.1);
        assert_eq!(Ok((2.2, 4.4)), resultado_2);
    }

    //Test que valida que la funcion 'procesar_linea' devuelva un [`Error::ValoresFaltantes`] o [`Error::ValoresFaltantes`] en caso de que la linea a procesar posea mas elementos de los esperados.
    #[test]
    fn test02_procesar_linea_cantidad_invalida() {
        test_procesar_linea_invalida_general("", Error::ValoresFaltantes, 0, 5, 0, 5);

        test_procesar_linea_invalida_general("2", Error::ValoresFaltantes, 0, 5, 0, 5);

        test_procesar_linea_invalida_general("1 5 7", Error::ValoresSobrantes, 0, 5, 0, 5);
    }

    //Test que valida que la funcion 'procesar_linea' devuelva un [`Error::ParseoNumero`] en caso de que la linea a procesar contenga elementos que no sean del tipo esperado para luego parsear a un numero.
    #[test]
    fn test03_procesar_linea_parseo_invalido() {
        test_procesar_linea_invalida_general("hola mundo", Error::ParseoNumero, 0, 5, 0, 5);

        test_procesar_linea_invalida_general("4 #", Error::ParseoNumero, 0, 5, 0, 5);
    }

    //Test que valida que la funcion 'procesar_linea' devuelva un [`Error::ValoresFueraDeRango`] en caso de que la linea a procesar contenga elementos fuera del rango esperado.
    #[test]
    fn test04_procesar_linea_fuera_rango() {
        test_procesar_linea_invalida_general("1 5", Error::ValoresFueraDeRango, 2, 5, 0, 5);

        test_procesar_linea_invalida_general(
            "2.5 84.2",
            Error::ValoresFueraDeRango,
            2.4,
            5.0,
            5.0,
            84.1,
        );
    }

    //Test que valida el correcto funcionamiento de la funcion 'parsear'.
    #[test]
    fn test05_parsear_valido() {
        let mut linea = "4";
        let resultado_1: Result<u32, Error> = parsear(linea);
        assert_eq!(Ok(4), resultado_1);

        linea = "16.4";
        let resultado_2: Result<f32, Error> = parsear(linea);
        assert_eq!(Ok(16.4), resultado_2);
    }

    //Test que valida que la funcion 'parsear' devuelva un [`Error::ParseoNumero`] en caso de que el elemento pasado no sea convertible a un tipo de numero.
    #[test]
    fn test06_parsear_invalido() {
        let mut elemento = " ";
        let resultado_1: Result<u32, Error> = parsear(elemento);
        assert_eq!(Err(Error::ParseoNumero), resultado_1);

        elemento = "##";
        let resultado_2: Result<u32, Error> = parsear(elemento);
        assert_eq!(Err(Error::ParseoNumero), resultado_2);
    }

    // Test generico para la funcion 'procesar_linea'.
    // A partir de una linea y un error_esperado, se valida que el retorno de la funcion coincida con el error_esperado.
    //
    // #Parametros
    // - `entrada` : la linea que se procesara.
    // - `error_esperado` : el error que la funcion debe retornar.
    // - `rango_min_elemento_1` : el valor minimo que puede tomar el 'elemento_1'.
    // - `rango_min_elemento_1` : el valor maximo que puede tomar el 'elemento_1'.
    // - `rango_min_elemento_2` : el valor minimo que puede tomar el 'elemento_2'.
    // - `rango_max_elemento_2` : el valor maximo que puede tomar el 'elemento_2'.
    //
    // # Tipos de datos genericos:
    // - `T` : tipo de dato del elemento_1, este debe implementar a [`std::str::FromStr`] y [`PartialOrd`].
    // - `K` : tipo de dato del elemento_2, este debe implementar a [`std::str::FromStr`] y [`PartialOrd`].
    fn test_procesar_linea_invalida_general<K, T>(
        entrada: &str,
        error_esperado: Error,
        rango_min_elemento_1: T,
        rango_max_elemento_1: T,
        rango_min_elemento_2: K,
        rango_max_elemento_2: K,
    ) where
        T: (std::str::FromStr) + PartialOrd + Debug,
        K: (std::str::FromStr) + PartialOrd + Debug,
    {
        let linea = String::from(entrada);
        let resultado = procesar_linea(
            &linea,
            rango_min_elemento_1,
            rango_max_elemento_1,
            rango_min_elemento_2,
            rango_max_elemento_2,
        );

        assert_eq!(Err(error_esperado), resultado);
    }
}
