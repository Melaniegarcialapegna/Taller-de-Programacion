pub mod entidades;
pub mod entrada;
pub mod utils;

use entrada::errores::Error;
use std::io::{BufRead, Write};

/// Ejecuta el programa principal.
///
/// Lee la entrada estandar, y valida que esta cumpla con el formato esperado. Luego, si la entrada estandar es valida, se genera el [`PlanoBidimensional`](entidades::plano_bidimensional::PlanoBidimensional) a partir de ella.Finalmente, con este se calcula la longitud de la sombra, la cual se escribe en el `writer`.
///
/// Si durante el procesamiento de la entrada se produce algun tipo de error, se escribira un mensaje describiendo cual fue en el `writer_error`.
///
/// # Parametros:
/// - `reader` : es la entrada de datos, se utiliza para poder simular `stdin` para los test.
/// - `writer` : se utiliza para poder acceder a lo que el programa escribe por `stdout`.
/// - `writer_error`: se utiliza para poder acceder a los mensajes que el programa envia por `stderr`.
///
/// # Tipos de datos genericos:
/// - `R` : implementa [`std::io::BufRead`].
/// - `W` : implementa [`std::io::Write`].
/// - `WE` : implementa [`std::io::Write`].
///
///
/// # Ejemplo
///
/// ```
///    let input = b"30 3\n50 150\n0 100\n 100 200\n"; //simula entrada estandar.
///    let mut reader = &input[..];
///    let mut output: Vec<u8> = Vec::new();
///    let mut output_err: Vec<u8> = Vec::new();
///
///    let _ = rompecabezas_de_las_sombras::ejecutar_programa(&mut reader, &mut output, &mut output_err);
///    
///    let longitud_sombra = String::from_utf8(output).unwrap();
///    println!("{}",longitud_sombra); //imprime la longitud de la sombra ya que la entrada es valida.
///
/// ```
///
///
/// # Errores
/// Si al escribir en el `writer` o en el `writer_error` se produce un error, se lo retorna.
pub fn ejecutar_programa<R: BufRead, W: Write, WE: Write>(
    reader: &mut R,
    writer: &mut W,
    writer_error: &mut WE,
) -> Result<(), Error> {
    let plano_bidimensional = match entrada::leer_entrada::leer_entrada(reader) {
        Ok(plano) => plano,
        Err(error) => {
            writeln!(writer_error, "{}", error).map_err(|_| Error::LecturaIO)?;
            return Ok(());
        }
    };

    let longitud_sombra = plano_bidimensional.calcular_longitud_sombra();
    writeln!(writer, "{}", longitud_sombra).map_err(|_| Error::LecturaIO)?;
    Ok(())
}
