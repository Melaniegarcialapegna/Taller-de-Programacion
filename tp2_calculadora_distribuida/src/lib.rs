//! Este proyecto simula una CALCULADORA DISTRIBUIDA
//!
//! ## Introduccion
//! El objetivo del trabajo práctico es crear una calculadora distribuida utilizando una arquitectura cliente-servidor. Contaremos con un único servidor central (la calculadora) y múltiples clientes (los operadores) que se comunican concurrentemente.
//!
//! ``` bash
//!
//!                                     -----------------
//!                    - - - - - - - >  -  calculadora  -   < - - - - - -
//!                    -                -----------------               -
//!                    -                        ^                       -
//!                    -                        -                       -
//!         -----------------           -----------------         -----------------
//!         -    operador   -           -    operador   -         -    operador   -
//!         -----------------           -----------------         -----------------
//! ```
//!
//! La comunicación entre los nodos se realiza a través de sockets. Desde el lado del servidor, cada conexión se procesará en un thread distinto.
//! Los operadores enviarán operaciones al servidor, el cual aplicará estas operaciones sobre un valor central. Las operaciones se aplicarán en orden de llegada.
//!
//! ## Protocolo de comunicacion
//!
//! Los nodos intercambiarán mensajes de texto delimitados por un salto de línea.
//!
//! El servidor aceptará dos tipos de mensajes: `OP <operacion>`; `GET`.
//!
//! Al recibir el mensaje `OP`, el servidor aplicará la operación, y responderá `OK` en caso de éxito, y `ERROR "<motivo>"` en caso de error.
//!
//! La operación tiene dos componentes, separados por whitespace: `<operador> <operando>`. El operador puede ser `+`, `-`, `*`, `/`, y el operando es un `u8`.
//!
//! Al recibir el mensaje `GET`, el servidor responderá con el valor actual de la calculadora `VALUE <valor>`.
//!
//! La especificación formal de los mensajes del protocolo está dada en notación `Backus-Naur (BNF)`.
//!

pub mod constantes;
