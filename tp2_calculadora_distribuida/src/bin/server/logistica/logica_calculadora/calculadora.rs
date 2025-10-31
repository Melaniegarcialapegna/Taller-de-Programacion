//! Este modulo define el tipo de dato `Calculadora`.
use crate::logistica::logica_calculadora::error_en_calculadora::ErrorEnCalculadora;
use crate::logistica::logica_operacion::operacion::Operacion;

const DIV_CERO: u8 = 0;

///Calculadora basica de u8 con overflow circular.
#[derive(Default)]
pub struct Calculadora {
    ///Almacena el resultado parcial tras aplicar operaciones, puede tomar valores dentro del rango `[0;256)`.
    valor: u8,
}

impl Calculadora {
    /// Retorna el `valor` actual de la calculadora.
    pub fn valor(&self) -> u8 {
        self.valor
    }

    /// Aplica una [`Operacion`] al `valor` actual de la calculadora.
    pub fn aplicar(&mut self, op: Operacion) -> Result<(), ErrorEnCalculadora> {
        match op {
            Operacion::Add(operand) => {
                self.valor = self.valor.wrapping_add(operand);
                Ok(())
            }
            Operacion::Sub(operand) => {
                self.valor = self.valor.wrapping_sub(operand);
                Ok(())
            }
            Operacion::Mul(operand) => {
                self.valor = self.valor.wrapping_mul(operand);
                Ok(())
            }
            Operacion::Div(operand) => {
                if operand == DIV_CERO {
                    return Err(ErrorEnCalculadora::DivisionCero);
                }
                self.valor = self.valor.wrapping_div(operand);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test01_calculadora_nueva() {
        let calculadora = Calculadora::default();
        let valor = calculadora.valor();

        assert_eq!(valor, 0);
    }

    #[test]
    fn test02_calculadora_falla_dividir_cero() {
        let mut calculadora = Calculadora::default();
        let operacion = Operacion::Div(0);

        let resultado_operacion = calculadora.aplicar(operacion);
        assert_eq!(
            resultado_operacion.unwrap_err(),
            ErrorEnCalculadora::DivisionCero
        );
    }

    #[test]
    fn test03_calculadora_varias_operaciones() {
        let mut calculadora = Calculadora::default();
        let operaciones = vec![Operacion::Add(4), Operacion::Sub(2), Operacion::Mul(2)];

        for operacion in operaciones {
            let _ = calculadora.aplicar(operacion);
        }
        let valor = calculadora.valor();

        assert_eq!(valor, 4);
    }

    #[test]
    fn test04_calculadora_varias_operaciones() {
        let mut calculadora = Calculadora::default();
        let operaciones = vec![
            Operacion::Add(24),
            Operacion::Sub(2),
            Operacion::Div(2),
            Operacion::Mul(4),
            Operacion::Sub(20),
            Operacion::Mul(2),
            Operacion::Add(4),
            Operacion::Sub(20),
            Operacion::Div(4),
        ];

        for operacion in operaciones {
            let _ = calculadora.aplicar(operacion);
        }
        let valor = calculadora.valor();

        assert_eq!(valor, 8);
    }
}
