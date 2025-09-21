//! Este modulo define el tipo de dato `Error`.

use crate::utils::constantes::*;
use std::fmt;

/// Representa los posibles errores que pueden surgir al leer la entrada estandar.
///
/// # Errores definidos
/// - `Error::LecturaIO` : error al intentar leer una linea por entrada estandar.
/// - `Error::ValoresFueraDeRango` : un valor esta por fuera del rango esperado.
/// - `Error::ValoresFaltantes` : en una linea hay menos valores de los esperados.
/// - `Error::ValoresSobrantes` : en una linea hay mas valores de los esperados.
/// - `Error::ParseoNumero` : no se pudo convertir a un tipo numerico un elemento que deberia poder convertise.
/// - `Error::LineasFaltantes` : hay menos lineas que las esperadas.
///
#[derive(PartialEq, Debug)]
pub enum Error {
    //Fallo cuando se intento leer de la entrada estandar.
    LecturaIO,
    //Un valor esta por fuera del rango predefinido.
    ValoresFueraDeRango,
    //Faltan elementos en una linea.
    ValoresFaltantes,
    //Sobran elementos en una linea.
    ValoresSobrantes,
    //No se pudo convertir a un tipo numerico.
    ParseoNumero,
    //Se esperaban mas lineas de entrada.
    LineasFaltantes,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mensaje = match self {
            Error::LecturaIO => LECTURA_IO_MENSAJE_ERROR,
            Error::ValoresFueraDeRango => VALORES_FUERA_RANGO_MENSAJE_ERROR,
            Error::ValoresFaltantes => VALOR_FALTANTE_MENSAJE_ERROR,
            Error::ValoresSobrantes => VALOR_SOBRANTE_MENSAJE,
            Error::ParseoNumero => PARSEO_NUMERO_MENSAJE,
            Error::LineasFaltantes => LINEA_FALTANTE_MENSAJE_ERROR,
        };
        write!(f, "{}", mensaje)
    }
}
