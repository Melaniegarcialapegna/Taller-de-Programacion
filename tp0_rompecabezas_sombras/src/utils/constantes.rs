//CONSTANTES DEL PROGRAMA
/// Cantidad de valores que se espera que tenga una linea.
pub const CANT_VALORES_ENTRADA_LINEA: usize = 2;
/// Se utiliza para hacer la conversion de grados a radianes.
pub const CIENTO_OCHENTA: f64 = 180.0;

//CONSTANTES RANGOS
///Valor minimo que debe tomar el angulo respecto al suelo en grados.
pub const ANGULO_MIN: f64 = 10.0;
///Valor maximo que puede tomar el angulo respecto al suelo en grados.
pub const ANGULO_MAX: f64 = 80.0;

///Valor minimo de flatlanders que deben haber en un plano.
pub const CANTIDAD_FLATLANDERS_MIN: usize = 1;
///Valor maximo de flatlanders que deben haber en un plano.
pub const CANTIDAD_FLATLANDERS_MAX: usize = 100000;

///Valor minimo que debe tomar la posicion de un flatlander.
pub const POSICION_MIN: u64 = 0;
///Valor maximo que puede tomar la posicion de un flatlander.
pub const POSICION_MAX: u64 = 300000;

///Valor minimo que debe tomar la altura de un flatlander.
pub const ALTURA_MIN: u64 = 1;
///Valor maximo que puede tomar la altura de un flatlander.
pub const ALTURA_MAX: u64 = 1000;

// CONSTANTES MENSAJES ERROR
pub const LECTURA_IO_MENSAJE_ERROR: &str = "Error: \"IO\"";
pub const VALORES_FUERA_RANGO_MENSAJE_ERROR: &str = "Error: \"Fuera de rango\"";
pub const VALOR_FALTANTE_MENSAJE_ERROR: &str = "Error: \"Valor faltante\"";
pub const VALOR_SOBRANTE_MENSAJE: &str = "Error: \"Valor sobrante\"";
pub const PARSEO_NUMERO_MENSAJE: &str = "Error: \"Numero invalido\"";
pub const LINEA_FALTANTE_MENSAJE_ERROR: &str = "Error: \"Linea faltante\"";

// CONSTANTES TEST
/// Cota de error para el valor que el programa devuelve como longitud de la sombra de un plano.
pub const COTA_ERROR_PERMITIDA_TEST: f64 = 1e-4;
pub const NO_SE_PUDO_PARSEAR_TEST: &str = "No se pudo pasar el valor a un numero";

// CONSTANTES MENSAJES ERROR TEST
pub const LECTURA_IO_MENSAJE_ERROR_TEST: &[u8] = b"Error: \"IO\"\n";
pub const VALORES_FUERA_RANGO_MENSAJE_ERROR_TEST: &[u8] = b"Error: \"Fuera de rango\"\n";
pub const VALOR_FALTANTE_MENSAJE_ERROR_TEST: &[u8] = b"Error: \"Valor faltante\"\n";
pub const VALOR_SOBRANTE_MENSAJE_TEST: &[u8] = b"Error: \"Valor sobrante\"\n";
pub const PARSEO_NUMERO_MENSAJE_TEST: &[u8] = b"Error: \"Numero invalido\"\n";
pub const LINEA_FALTANTE_MENSAJE_ERROR_TEST: &[u8] = b"Error: \"Linea faltante\"\n";
