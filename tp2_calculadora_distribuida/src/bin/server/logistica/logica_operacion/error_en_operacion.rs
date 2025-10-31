use std::fmt;

const OPERADOR_INVALIDO: &str = "ERROR \"parsing error : invalid operator\"";
const OPERANDO_INVALIDO: &str = "ERROR \"parsing error : invalid operand\"";

/// Representa los posibles errores que pueden surgir intentar instanciar a una operacion.
#[derive(Debug, PartialEq)]
pub enum ErrorEnOperacion {
    /// El operador no esta permitido o no es valido.
    OperadorInvalido,
    /// El operando no cumple con los requisitos para ser parseado a un `u8`.
    OperandoInvalido,
}

impl fmt::Display for ErrorEnOperacion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorEnOperacion::OperadorInvalido => write!(f, "{}", OPERADOR_INVALIDO),
            ErrorEnOperacion::OperandoInvalido => write!(f, "{}", OPERANDO_INVALIDO),
        }
    }
}
