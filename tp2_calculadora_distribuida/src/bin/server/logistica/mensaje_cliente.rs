//! Este modulo define el tipo de dato `MensajeCliente`.
use crate::errores::error_en_thread::ErrorEnThread;
use crate::logistica::logica_operacion::operacion::Operacion;
use std::str::FromStr;

//constantes
const MENSAJE_OP: &str = "OP";
const MENSAJE_GET: &str = "GET";

/// Representa las variantes de tipos de mensajes que puede responder el servidor de parte de un cliente.
#[derive(PartialEq, Eq, Debug)]
pub enum MensajeCliente {
    /// El cliente esta pidiendo aplicar una [`Operacion`] a la calculadora.
    Operacion(Operacion),
    /// El cliente esta solicitando saber el valor actual de la calculadora.
    Get,
}

impl FromStr for MensajeCliente {
    type Err = ErrorEnThread;

    /// Crea una nueva instancia de [`MensajeCliente`] a partir de un string.
    /// Recibe un string enviado por el cliente y lo parsea segun el requerimiento de este.
    /// En caso de no ser un mensaje que el servidor sepa responder, se retorta un [`ErrorEnThread`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();
        let token = tokens.next().ok_or(ErrorEnThread::FaltaCampo)?;

        match token {
            MENSAJE_OP => {
                let operador = tokens.next().ok_or(ErrorEnThread::FaltaCampo)?;
                let numero = tokens.next().ok_or(ErrorEnThread::FaltaCampo)?;
                let operacion = Operacion::nueva_operacion(operador, numero)?;
                Ok(MensajeCliente::Operacion(operacion))
            }
            MENSAJE_GET => Ok(MensajeCliente::Get),
            _ => Err(ErrorEnThread::MensajeRecibidoInvalido),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_mensaje_cliente() {
        mensaje_cliente_basica_valida_generica(
            "OP + 4",
            MensajeCliente::Operacion(Operacion::Add(4)),
        );

        mensaje_cliente_basica_valida_generica(
            "OP - 2",
            MensajeCliente::Operacion(Operacion::Sub(2)),
        );

        mensaje_cliente_basica_valida_generica(
            "OP * 20",
            MensajeCliente::Operacion(Operacion::Mul(20)),
        );

        mensaje_cliente_basica_valida_generica(
            "OP / 14",
            MensajeCliente::Operacion(Operacion::Div(14)),
        );
    }

    fn mensaje_cliente_basica_valida_generica(mensaje_str: &str, mensaje_esperado: MensajeCliente) {
        let nuevo_mensaje = MensajeCliente::from_str(mensaje_str).unwrap();

        assert_eq!(nuevo_mensaje, mensaje_esperado);
    }

    #[test]
    fn test02_mensaje_cliente() {
        mensaje_cliente_basica_valida_generica("GET", MensajeCliente::Get);
    }

    #[test]
    fn test03_mensaje_cliente_invalido() {
        mensaje_cliente_basica_invalida_generica("", ErrorEnThread::FaltaCampo);

        mensaje_cliente_basica_invalida_generica("OP 4", ErrorEnThread::FaltaCampo);

        mensaje_cliente_basica_invalida_generica("# + 4", ErrorEnThread::MensajeRecibidoInvalido);
    }

    fn mensaje_cliente_basica_invalida_generica(mensaje_str: &str, error_esperado: ErrorEnThread) {
        let nuevo_mensaje = MensajeCliente::from_str(mensaje_str);

        assert_eq!(nuevo_mensaje.unwrap_err(), error_esperado);
    }
}
