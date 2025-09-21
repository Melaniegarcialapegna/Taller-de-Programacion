//! Este modulo define el tipo de dato `Sombra`.

///Es un intervalo que representa una sombra.
///
/// # Campos
/// - `posicion_inicial`: extremo izquierdo de la sombra.
/// - `posicion_final` : extremo derecho de la sombra.
///
#[derive(Debug, PartialEq)]
pub struct Sombra {
    pub posicion_inicial: f64,
    pub posicion_final: f64,
}

impl Sombra {
    ///Crea y devuelve una instancia de `Sombra`
    ///
    ///Recibe el punto de inicio y fin del intervalo que representara.
    pub fn new(posicion_inicial: f64, posicion_final: f64) -> Sombra {
        Sombra {
            posicion_inicial,
            posicion_final,
        }
    }
    ///Devuelve la longitud que cubre la sombra
    pub fn calcular_longitud(&self) -> f64 {
        self.posicion_final - self.posicion_inicial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //Test que valida el correcto funcionamiento de la funcion 'calcular_longitud'.
    #[test]
    fn test01_calcular_longitud() {
        let posicion_inicial = 2.0;
        let posicion_final = 4.0;

        assert_eq!(
            2.0,
            Sombra::new(posicion_inicial, posicion_final).calcular_longitud()
        );
    }

    //Test que valida el correcto funcionamiento de la funcion 'calcular_longitud'.
    #[test]
    fn test02_calcular_longitud() {
        let posicion_inicial = 2.0;
        let posicion_final = 4.2;

        assert_eq!(
            2.2,
            Sombra::new(posicion_inicial, posicion_final).calcular_longitud()
        );
    }

    //Test que valida el correcto funcionamiento de la funcion 'calcular_longitud'.
    #[test]
    fn test03_calcular_longitud() {
        let posicion_inicial = 42.5;
        let posicion_final = 54.2;

        let mut resultado = Sombra::new(posicion_inicial, posicion_final).calcular_longitud();

        resultado = (resultado * 10.0).floor() / 10.0;

        assert_eq!(11.7, resultado);
    }
}
