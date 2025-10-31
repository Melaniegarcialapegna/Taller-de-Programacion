use std::fmt;

const DIVISION_CERO: &str = "ERROR \"division by zero\"";

/// Representa los posibles errores que pueden surgir al aplicar operaciones.
#[derive(Debug, PartialEq)]
pub enum ErrorEnCalculadora {
    /// Se intento realizar una division por cero, lo cual es matematicamente erroneo.
    DivisionCero,
}

impl fmt::Display for ErrorEnCalculadora {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorEnCalculadora::DivisionCero => write!(f, "{}", DIVISION_CERO),
        }
    }
}
