//! # Modulo principal Cliente.

mod conexion;
mod entrada;
mod errores;
mod logistica;

use conexion::gestionar_conexion::establecer_conexion;
use entrada::gestionar_entrada::parsear_argumentos;

/// Este modulo contiene la funcion `main` del cliente.
///
/// ## Esta encargado de:
/// - Llamar al modulo [`entrada`] que se encargara del parseo y validacion de la invocacion al programa.
/// - Enviarle al modulo [`conexion`] el retorno del modulo [`entrada`] que contendra la `direccion` y el `archivo` que utilizara para establer la conexion con el servidor.
///
///
/// ## Forma de invocacion
///
/// ```bash
/// cargo run --bin client -- <direccion> <ruta_archivo>
/// ```
///
/// La `direccion` esta conformada por : `<ip:puerto>`.
///
/// ## Ejemplo de invocacion
///
/// ```bash
/// cargo run --bin client -- 127.0.0.1:12345 data/a.txt
/// ```
///
/// ## Modulos relacionados
///
/// - [`entrada`] : Se encarga de parsear y validar los argumentos de entrada.
///
/// - [`conexion`]: Se encarga de establecer la conexion con el servidor, manejarla y tambien del manejo del archivo.
///
/// - [`errores`] : Contiene a los posibles errores que pueden ocurrir durante el programa.
///
/// ## Errores
///
/// En caso de algun error irrecuperable, como que no se ingresen los parametros esperados, el archivo no exista, no se pueda establer la conexion con la direccion indicada, etc...
/// En este caso se mostrara el error por stderr junto con la finalizacion del programa.
///
fn main() {
    let mut inputs = std::env::args();

    let entrada = match parsear_argumentos(&mut inputs) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    if let Err(error) = establecer_conexion(entrada.direccion, entrada.archivo) {
        eprintln!("{}", error);
    }
}
