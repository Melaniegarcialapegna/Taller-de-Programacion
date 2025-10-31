//! Este modulo define el tipo de dato `Operacion`.
use crate::logistica::logica_operacion::error_en_operacion::ErrorEnOperacion;

//Constantes para operadores
const OP_SUMA: &str = "+";
const OP_RESTA: &str = "-";
const OP_MULTIPLICACION: &str = "*";
const OP_DIVISION: &str = "/";

/// Representa una operacion que es el conjunto de un operador con su operando.
///
/// Se admiten cuatro tipo de operaciones : suma, resta, multiplicacion y division.
#[derive(PartialEq, Eq, Debug)]
pub enum Operacion {
    /// Representa la suma y el [`u8`] que se sumara.
    Add(u8),
    /// Representa la resta y el [`u8`] que se restara.
    Sub(u8),
    /// Representa la multiplicacion y el [`u8`] por el que se multiplicara.
    Mul(u8),
    /// Representa la division y el [`u8`] por el que se dividira.
    Div(u8),
}

impl Operacion {
    /// Crea una nueva instancia de operacion a partir de dos string que representan un `operador` y un `operando`.
    /// En caso de que el `operador` no sea de uno de los tipos admitidos, o si el `operando` no es parseable a un `u8`, se retorna un [`ErrorEnOperacion`].
    pub fn nueva_operacion(operador: &str, operando: &str) -> Result<Self, ErrorEnOperacion> {
        let operando: u8 = operando
            .parse()
            .map_err(|_| ErrorEnOperacion::OperandoInvalido)?;

        match operador {
            OP_SUMA => Ok(Operacion::Add(operando)),
            OP_RESTA => Ok(Operacion::Sub(operando)),
            OP_MULTIPLICACION => Ok(Operacion::Mul(operando)),
            OP_DIVISION => Ok(Operacion::Div(operando)),
            _ => Err(ErrorEnOperacion::OperadorInvalido),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_operacion() {
        test_operacion_valida_generica("+", "4", Operacion::Add(4));

        test_operacion_valida_generica("-", "2", Operacion::Sub(2));

        test_operacion_valida_generica("*", "10", Operacion::Mul(10));

        test_operacion_valida_generica("/", "8", Operacion::Div(8));
    }

    fn test_operacion_valida_generica(
        operador: &str,
        operando: &str,
        operacion_esperada: Operacion,
    ) {
        let nueva_operacion = Operacion::nueva_operacion(operador, operando).unwrap();

        assert_eq!(nueva_operacion, operacion_esperada);
    }
    #[test]
    fn test02_operacion_invalida() {
        test_operacion_invalida_generica("*", "#", ErrorEnOperacion::OperandoInvalido);

        test_operacion_invalida_generica("*", " ", ErrorEnOperacion::OperandoInvalido);

        test_operacion_invalida_generica(" ", "4", ErrorEnOperacion::OperadorInvalido);

        test_operacion_invalida_generica("hola", "4", ErrorEnOperacion::OperadorInvalido);
    }

    fn test_operacion_invalida_generica(
        operador: &str,
        operando: &str,
        error_esperado: ErrorEnOperacion,
    ) {
        let nueva_operacion = Operacion::nueva_operacion(operador, operando);

        assert_eq!(nueva_operacion.unwrap_err(), error_esperado);
    }
}
