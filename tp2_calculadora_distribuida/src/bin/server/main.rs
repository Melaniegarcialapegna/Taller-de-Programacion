//! # Modulo principal Servidor.

mod conexiones;
mod entrada;
mod errores;
mod logistica;

use crate::conexiones::gestion_conexiones::gestionar_conexiones;
use crate::entrada::gestionar_entrada::parsear_argumentos;
use std::env;

/// Este modulo contiene la funcion `main` del servidor.
///
/// ## Esta encargado de:
/// - Llamar al modulo [`entrada`] que se encargara del parseo y validacion de la invocacion al programa.
/// - Enviarle al modulo [`conexiones`] el retorno del modulo [`entrada`] que contendra la `direccion` que se utilizara para establecer la conexion.
///
///
/// ## Forma de invocacion
///
/// ```bash
/// cargo run --bin client -- <direccion>
/// ```
///
/// La `direccion` esta conformada por : `<ip:puerto>`.
///
/// ## Ejemplo de invocacion
///
/// ```bash
/// cargo run --bin client -- 127.0.0.1:12345
/// ```
///
/// ## Modulos relacionados
///
/// - [`entrada`] : Se encarga de parsear y validar los argumentos de entrada.
///
/// - [`conexiones`] : Se encarga de establecer la conexion por la que escuchara a los clientes, y tambien del manejo de cada uno de estos.
///
/// - [`logistica`] : Tiene la logica que el servidor utiliza para manejar las peticiones de los clientes y tambien sus respuestas.
///
/// - [`errores`] : Contiene a los posibles errores que pueden ocurrir durante el programa.
///
/// ## Errores
///
/// En caso de algun error irrecuperable, como que no se ingresen los parametros esperados o que no se pueda establer la conexion con la direccion indicada.
/// En este caso se mostrara el error por stderr junto con la finalizacion del programa.
///
fn main() {
    let mut inputs = env::args();

    let entrada = match parsear_argumentos(&mut inputs) {
        Ok(valor) => valor,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    if let Err(error) = gestionar_conexiones(entrada.direccion) {
        eprintln!("{}", error);
    }
}
