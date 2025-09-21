use rompecabezas_de_las_sombras::utils::constantes::*;

//Test que valida que el programa no falle ante una entrada estandar valida y que ademas devuelva el resultado esperado por salida estandar.
#[test]
fn test01_entrada_valida() {
    test_entrada_valida_general(b"   45 2\n 0 10\n15 20\n", 30.0);
}
//Test que valida que el programa no falle ante una entrada estandar valida y que ademas devuelva el resultado esperado por salida estandar.
#[test]
//ver como pasrlo a numeros y asi poder comparar la diferencia
fn test02_entrada_valida() {
    test_entrada_valida_general(b"45 2\n 0 10\n5 10\n", 15.0000000000000);
}
//Test que valida que el programa no falle ante una entrada estandar valida y que ademas devuelva el resultado esperado por salida estandar.
#[test]
fn test03_entrada_valida() {
    test_entrada_valida_general(b"30 3\n50 150\n0 100\n 100 200\n", 446.4101615137755);
}
//Test que valida que el programa no falle ante una entrada estandar valida y que ademas devuelva el resultado esperado por salida estandar.
#[test]
fn test04_entrada_valida() {
    test_entrada_valida_general(b"45 3\n50 150\n0 100\n 100 200\n", 300.00000000000006);
}

//Test que valida que el programa falle y emita el mensaje correcto por stderr cuando se le pasan menos valores de los esperados en una linea.
#[test]
fn test05_linea_cantidad_elementos_invalida() {
    test_entrada_invalida_general(b"45 \n50 50\n0 100\n", VALOR_FALTANTE_MENSAJE_ERROR_TEST);

    test_entrada_invalida_general(b"45 2\n 50\n0 100\n", VALOR_FALTANTE_MENSAJE_ERROR_TEST);

    test_entrada_invalida_general(
        b"45 3\n50 50\n0 100\n0 \n",
        VALOR_FALTANTE_MENSAJE_ERROR_TEST,
    );
}

//Test que valida que el programa falle y emita el mensaje correcto por stderr cuando se le pasan mas valores de los esperados en una linea.
#[test]
fn test06_linea_cantidad_elementos_invalida() {
    test_entrada_invalida_general(b"45 2 7\n50 150\n0 100\n", VALOR_SOBRANTE_MENSAJE_TEST);

    test_entrada_invalida_general(b"45 3\n50 150\n0 100 8\n", VALOR_SOBRANTE_MENSAJE_TEST);

    test_entrada_invalida_general(b"45 7\n50 10 8\n0 100\n", VALOR_SOBRANTE_MENSAJE_TEST);
}

//Test que valida que el programa falle y emita el mensaje correcto por stderr cuando se le pasan valores que no se pueden parsear a un valor numerico.
#[test]
fn test07_parseo_invalido() {
    test_entrada_invalida_general(b"45 2\n50 150\n# 100\n", PARSEO_NUMERO_MENSAJE_TEST);

    test_entrada_invalida_general(b"45 hola\n50 150\n4 100\n", PARSEO_NUMERO_MENSAJE_TEST);

    test_entrada_invalida_general(b"45 2\n50 @\n5 100\n", PARSEO_NUMERO_MENSAJE_TEST);
}

//Test que valida que el programa falle y emita el mensaje correcto por stderr cuando el valor de una linea esta por fuera del rango esperado.
#[test]
fn test08_valor_fuera_rango() {
    test_entrada_invalida_general(
        b"45 2\n50 1500000\n20 100\n",
        VALORES_FUERA_RANGO_MENSAJE_ERROR_TEST,
    );

    test_entrada_invalida_general(
        b"0 2\n50 10\n20 100\n",
        VALORES_FUERA_RANGO_MENSAJE_ERROR_TEST,
    );

    test_entrada_invalida_general(
        b"45 2\n50 9\n20 100000\n",
        VALORES_FUERA_RANGO_MENSAJE_ERROR_TEST,
    );
}

//Test que valida que el programa falle y emita el mensaje correcto por stderr cuando se le pasan menos lineas de las esperadas.
#[test]
fn test09_linea_faltante() {
    test_entrada_invalida_general(b"45 2\n 0 10\n", LINEA_FALTANTE_MENSAJE_ERROR_TEST);

    test_entrada_invalida_general(b" ", LINEA_FALTANTE_MENSAJE_ERROR_TEST);

    test_entrada_invalida_general(b"\n", LINEA_FALTANTE_MENSAJE_ERROR_TEST);

    test_entrada_invalida_general(b"45 1\n \n \n", LINEA_FALTANTE_MENSAJE_ERROR_TEST);
}

// Test generico para validar el funcionamiento del programa.
//
// #Parametros
// - `input` : la entrada que se procesara.
// - `resultado_esperado` : lo que se espera que el programa devuelva por stdout.
//
fn test_entrada_valida_general(input: &[u8], resultado_esperado: f64) {
    let mut reader = input;
    let mut output: Vec<u8> = Vec::new();
    let mut output_err: Vec<u8> = Vec::new();

    let _ =
        rompecabezas_de_las_sombras::ejecutar_programa(&mut reader, &mut output, &mut output_err);

    let valor_sombra = parsear_output_a_numero(output);

    //Se valida que el error sea menor a 10^-4
    assert!((resultado_esperado - valor_sombra).abs() < COTA_ERROR_PERMITIDA_TEST);
    assert!(output_err.is_empty());
}

// Test generico para validar el funcionamiento del programa cuando debe fallar.
//
// #Parametros
// - `input` : la entrada que se procesara.
// - `mensaje_error_esperado` : el mensaje que se espera que el programa devuelva por stderr.
fn test_entrada_invalida_general(input: &[u8], mensaje_error_esperado: &[u8]) {
    let mut reader = input;
    let mut output: Vec<u8> = Vec::new();
    let mut output_err: Vec<u8> = Vec::new();

    let _ =
        rompecabezas_de_las_sombras::ejecutar_programa(&mut reader, &mut output, &mut output_err);

    assert_eq!(output_err, mensaje_error_esperado);
    assert!(output.is_empty());
}

fn parsear_output_a_numero(output: Vec<u8>) -> f64 {
    String::from_utf8(output)
        .unwrap()
        .trim()
        .parse()
        .expect("{NO_SE_PUDO_PARSEAR}")
}
