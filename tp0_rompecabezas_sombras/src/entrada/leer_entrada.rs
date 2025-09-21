//! Este modulo contiene las funciones que se encargan de leer la entrada.

use super::errores::Error;
use super::procesar_linea::*;
use super::validaciones::*;
use crate::entidades::flatlander::Flatlander;
use crate::entidades::plano_bidimensional::PlanoBidimensional;
use crate::utils::constantes::*;


/// Procesa la entrada estandar y devuelve un [`PlanoBidimensional`] con la informacion recolectada.
///
///
/// # Parametros
/// - `reader` de tipo [`std::io::BufRead`] el cual se utiliza para simular la entrada estandar.
///
/// # Formato esperado para la entrada
///
/// ``` console
/// <angulo_grados> <cantidad_flatlanders = n>
/// <posicion_flatlander_1> <altura_flatlander_1>
/// ..
/// ..
/// <posicion_flatlander_n> <altura_flatlander_n>
/// ```
/// # Restricciones
/// - 10 <=`angulo_grados`<= 80
/// - 1 <=`n`<= 10^5
/// - 0 <=`posicion_flatlander`<= 3.10^5
/// - 1 <=`angulo_grados`<= 1000
///
/// # Errores
///
/// Si la entrada no respeta el formato debido se retornara un [`Error`] .
///
pub fn leer_entrada<R: std::io::BufRead>(reader: &mut R) -> Result<PlanoBidimensional, Error> {
    let mut linea = String::new();

    leer_linea(reader, &mut linea)?;

    let (mut angulo, cantidad_flatlanders) = procesar_linea(
        &linea,
        ANGULO_MIN,
        ANGULO_MAX,
        CANTIDAD_FLATLANDERS_MIN,
        CANTIDAD_FLATLANDERS_MAX,
    )?;

    angulo = pasar_a_radianes(angulo);

    //Se crea un vector que contendra a los flatlanders que se vayan procesando en cada linea
    let mut flatlanders: Vec<Flatlander> = Vec::with_capacity(cantidad_flatlanders);

    for _ in 0..cantidad_flatlanders {
        linea.clear();

        leer_linea(reader, &mut linea)?;

        let (posicion, altura) =
            procesar_linea(&linea, POSICION_MIN, POSICION_MAX, ALTURA_MIN, ALTURA_MAX)?;

        let flatlander: Flatlander = Flatlander::new(posicion, altura);

        flatlanders.push(flatlander);
    }

    Ok(PlanoBidimensional::new(angulo, flatlanders))
}

/// Lee una linea (hasta el primer \n) de la entrada y su contenido lo almacena en el String 'linea'.
///
/// # Errores
/// Se retorna un [`Error`] en caso de que se ocasione algun error de IO o si la linea se encuentra vacia.
fn leer_linea<R: std::io::BufRead>(reader: &mut R, linea: &mut String) -> Result<(), Error> {
    let lectura = reader.read_line(linea);

    validar_entrada(linea, lectura)?;

    Ok(())
}

/// Transforma un angulo en grados a un angulo en radianes.
///
/// # Parametros
/// - `angulo` : angulo en grados.
///
fn pasar_a_radianes(angulo: f64) -> f64 {
    (angulo * std::f64::consts::PI) / CIENTO_OCHENTA
}

#[cfg(test)]
mod tests {
    use super::*;

    //Test que valida el correcto funcionamiento de la funcion 'procesar_entrada' simulando distintas entradas.
    #[test]
    fn test01_procesar_entrada_valida() {
        let vector_flatlanders = vec![Flatlander::new(40, 1)];

        test_procesar_entrada_valida_general(
            b"45 1\n 40 1\n",
            std::f64::consts::FRAC_PI_4, //lo cambie por esta cte ya que clippy me lo recomendo
            vector_flatlanders,
        );

        let vector_flatlanders_2 = vec![Flatlander::new(40, 1)];

        test_procesar_entrada_valida_general(
            b"30 1\n 40 1\n",
            0.5235987755982988,
            vector_flatlanders_2,
        );

        let vector_flatlanders_3 = vec![Flatlander::new(0, 10), Flatlander::new(5, 10)];

        test_procesar_entrada_valida_general(
            b"45 2\n 0 10\n5 10\n ",
            std::f64::consts::FRAC_PI_4,
            vector_flatlanders_3,
        );

        let vector_flatlanders_4 = vec![
            Flatlander::new(50, 150),
            Flatlander::new(0, 100),
            Flatlander::new(100, 200),
        ];

        test_procesar_entrada_valida_general(
            b"30 3\n 50 150\n0 100\n100 200\n",
            0.5235987755982988,
            vector_flatlanders_4,
        );
    }

    //Test que valida que la funcion 'procesar_entrada' devuelva un [`Error::ValoresFaltantes`] o [`Error::ValoresFaltantes`] en caso de que alguna de las lineas procesadas contenga menos o mas elementos de los esperados.
    #[test]
    fn test02_procesar_linea_cantidad_invalida() {
        test_procesar_entrada_invalida_general(b"45 2\n 0 10\n5\n", Error::ValoresFaltantes);

        test_procesar_entrada_invalida_general(b"45 2\n 0 10\n5 10 15\n", Error::ValoresSobrantes);
    }

    //Test que valida que la funcion 'procesar_entrada' devuelva un [`Error::ParseoNumero`] en caso de que alguna de las lineas procesadas contenga algun elemento el cual no es posible parsear a un tipo numerico.
    #[test]
    fn test03_procesar_entrada_parseo_invalido() {
        test_procesar_entrada_invalida_general(b"45 2\n 0 10\nhola mundo\n", Error::ParseoNumero);

        test_procesar_entrada_invalida_general(b"45 2\n4 #\n5 10\n", Error::ParseoNumero);
    }

    //Test que valida que la funcion 'procesar_entrada' devuelva un [`Error::ValoresFueraDeRango`] en caso de que alguna de las lineas procesadas contenga algun elemento fuera de los rangos predefinidos.
    #[test]
    fn test04_procesar_entrada_fuera_rango() {
        test_procesar_entrada_invalida_general(b"81 2\n 0 10\n5 10\n", Error::ValoresFueraDeRango);

        test_procesar_entrada_invalida_general(
            b"45 2\n 0 10\n5 20000\n",
            Error::ValoresFueraDeRango,
        );
    }

    //Test que valida que la funcion 'procesar_entrada' devuelva un [`Error::LineasFaltantes`] en caso de que hayan menos lineas que las esperadas.
    #[test]
    fn test05_procesar_entrada_linea_faltante() {
        test_procesar_entrada_invalida_general(b"\n", Error::LineasFaltantes);

        test_procesar_entrada_invalida_general(b"45 2\n 0 10\n", Error::LineasFaltantes);
    }

    // Test generico para la funcion 'procesar_entrada'.
    // A partir de un angulo y un vector de flatlanders que conforman al [`PlanoBidimensional`] que se espera que sea devuelto por la funcion, se valida que estos coincidan.
    //
    // #Parametros
    // - `input` : la entrada que se procesara.
    // - `angulo` : el angulo en radianes que tendra el plano.
    // - `flatlanders` : vector con los flatlanders que se situaran en el plano.
    //
    fn test_procesar_entrada_valida_general(
        input: &[u8],
        angulo: f64,
        flatlanders: Vec<Flatlander>,
    ) {
        let mut reader = input;
        let resultado = leer_entrada(&mut reader).unwrap();

        let plano = PlanoBidimensional::new(angulo, flatlanders);

        assert_eq!(plano.to_string(), resultado.to_string());
    }

    // Test generico para la funcion 'procesar_entrada'.
    // A partir de una entrada y un error_esperado, se valida que el retorno de la funcion coincida con el error_esperado.
    //
    // #Parametros
    // - `input` : la entrada que se procesara.
    // - `error_esperado` : el error que la funcion debe retornar.
    //
    fn test_procesar_entrada_invalida_general(input: &[u8], error_esperado: Error) {
        let mut reader = input;
        let resultado = leer_entrada(&mut reader);

        assert_eq!(resultado.unwrap_err(), error_esperado);
    }
}
