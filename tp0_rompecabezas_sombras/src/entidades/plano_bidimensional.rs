//! Este modulo define el tipo de dato `PlanoBidimensional`.

use crate::entidades::flatlander::Flatlander;
use crate::entidades::sombra::Sombra;
use std::fmt;

///Representa un mundo plano, el cual puede ser representado como una carretera infinita que se extiende a lo largo del eje X.
///
/// # Campos
/// - `flatlanders`: cada uno de estos es una entidad de tipo  [`Flatlander`], la cual tiene una `posicion` y una `altura`. Estos entes se situan ordenados por posicion a lo largo del mundo plano.
/// - `angulo_respecto_suelo` : es el angulo `theta` que se genera cuando una luz al oeste proyecta a un [`Flatlander`].
///
#[derive(Debug)]
pub struct PlanoBidimensional {
    //Angulo en radianes
    pub angulo_respecto_suelo: f64,

    //Vector con [`Flatlander`] ordenados de manera ascendente por su posicion.
    pub flatlanders: Vec<Flatlander>,
}

impl fmt::Display for PlanoBidimensional {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {:?}", self.angulo_respecto_suelo, self.flatlanders)
    }
}

impl PlanoBidimensional {
    ///Crea y devuelve una instancia de `PlanoBidimensional`
    ///
    /// Recibe un angulo respecto al suelo y un vector de [`Flatlander`] el cual ordena por posicion en el eje.
    pub fn new(angulo_respecto_suelo: f64, mut flatlanders: Vec<Flatlander>) -> PlanoBidimensional {
        flatlanders.sort_by_key(|i| i.posicion);

        PlanoBidimensional {
            angulo_respecto_suelo,
            flatlanders,
        }
    }

    /// Calcula la longitud total de la sombra generada por los multiples flatlanders.
    pub fn calcular_longitud_sombra(&self) -> f64 {
        let sombras: Vec<Sombra> = self.generador_de_sombras();

        let mut longitud_total: f64 = 0.0;
        //Posicion del punto mas lejano cubierto por las sombras procesadas.
        let mut distancia_cubierta: f64 = 0.0;

        for sombra in &sombras {
            if sombra.posicion_inicial >= distancia_cubierta {
                //No hay superposicion entre la sombra procesada y las previamente procesadas
                longitud_total += sombra.calcular_longitud();
                distancia_cubierta = sombra.posicion_final
            } else if sombra.posicion_final > distancia_cubierta {
                //Hay superposicion parcial
                longitud_total += sombra.posicion_final - distancia_cubierta; //Se suma el fragmento que no se superpone 
                distancia_cubierta = sombra.posicion_final
            }
        }
        longitud_total
    }

    /// Genera y devuelve un Vector de tipo [`Sombra`], en el cual se almacenan las sombras de los distintos flatlanders.
    fn generador_de_sombras(&self) -> Vec<Sombra> {
        let mut sombras: Vec<Sombra> = Vec::with_capacity(self.flatlanders.len());

        for i in 0..self.flatlanders.len() {
            sombras.push(self.generar_sombra_flatlander(i));
        }
        sombras
    }

    /// Genera y devuelve la [`Sombra`] de un [`Flatlander`] respecto de un angulo respecto al suelo.
    fn generar_sombra_flatlander(&self, indice_flatlander: usize) -> Sombra {
        let flatlander = &self.flatlanders[indice_flatlander];
        let longitud_sombra = (flatlander.altura as f64) / self.angulo_respecto_suelo.tan();

        Sombra::new(
            flatlander.posicion as f64,
            flatlander.posicion as f64 + longitud_sombra,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //Test que valida el correcto funcionamiento de la funcion 'generar_sombra_flatlander'.
    #[test]
    fn test01_generar_sombra_flatlander() {
        let flatlander_1 = Flatlander::new(5, 10);

        test_generar_sombra_flatlander_general(flatlander_1, 54.0, 5.0, 10.0);

        let flatlander_2 = Flatlander::new(48, 56);

        test_generar_sombra_flatlander_general(flatlander_2, 24.0, 48.0, 56.0);
    }

    //Test que valida el correcto funcionamiento de la funcion 'generador_de_sombras'.
    #[test]
    fn test02_generarador_de_sombras() {
        let flatlander_1 = Flatlander::new(5, 10);

        let angulo_sol_radianes =
            54.0 * std::f64::consts::PI / crate::utils::constantes::CIENTO_OCHENTA;

        let plano = PlanoBidimensional::new(angulo_sol_radianes, vec![flatlander_1]);

        let sombra_flatlander_1 = Sombra::new(5.0, 5.0 + (10.0 / angulo_sol_radianes.tan()));

        let vector_sombras_flatlanders = plano.generador_de_sombras();

        assert_eq!(vec![sombra_flatlander_1], vector_sombras_flatlanders);
    }

    //Test que valida el correcto funcionamiento de la funcion 'generador_de_sombras'.
    #[test]
    fn test03_generarador_de_sombras() {
        let flatlander_1 = Flatlander::new(5, 60);
        let flatlander_2 = Flatlander::new(8, 12);
        let flatlander_3 = Flatlander::new(26, 24);

        let angulo_sol_radianes =
            54.0 * std::f64::consts::PI / crate::utils::constantes::CIENTO_OCHENTA;

        let plano = PlanoBidimensional::new(
            angulo_sol_radianes,
            vec![flatlander_1, flatlander_2, flatlander_3],
        );

        let sombra_flatlander_1 = Sombra::new(5.0, 5.0 + (60.0 / angulo_sol_radianes.tan()));
        let sombra_flatlander_2 = Sombra::new(8.0, 8.0 + (12.0 / angulo_sol_radianes.tan()));
        let sombra_flatlander_3 = Sombra::new(26.0, 26.0 + (24.0 / angulo_sol_radianes.tan()));

        let vector_sombras_flatlanders = plano.generador_de_sombras();

        assert_eq!(
            vec![
                sombra_flatlander_1,
                sombra_flatlander_2,
                sombra_flatlander_3
            ],
            vector_sombras_flatlanders
        );
    }

    //Test que valida el correcto funcionamiento de la funcion 'calcular_longitud_sombra'.
    #[test]
    fn test04_calcular_longitud_sombra_plano() {
        let flatlander = Flatlander::new(5, 10);

        test_calcular_longitud_sombra_plano_general(vec![flatlander], 54.0, 7.2654);
    }

    //Test que valida el correcto funcionamiento de la funcion 'calcular_longitud_sombra'.
    #[test]
    fn test05_calcular_longitud_sombra_plano() {
        let flatlander_1 = Flatlander::new(8, 12);
        let flatlander_2 = Flatlander::new(5, 60);
        let flatlander_3 = Flatlander::new(26, 24);

        test_calcular_longitud_sombra_plano_general(
            vec![flatlander_1, flatlander_2, flatlander_3],
            24.0,
            134.7622,
        );
    }

    //Test que valida el correcto funcionamiento de la funcion 'calcular_longitud_sombra'.
    #[test]
    fn test06_calcular_longitud_sombra_plano() {
        let flatlander_1 = Flatlander::new(15, 48);
        let flatlander_2 = Flatlander::new(2, 25);
        let flatlander_3 = Flatlander::new(18, 2);

        test_calcular_longitud_sombra_plano_general(
            vec![flatlander_1, flatlander_2, flatlander_3],
            32.0,
            89.8160,
        );
    }

    // Test generico para la funcion 'generar_sombra_flatlander'.
    //
    // #Parametros
    // - `flatlander` : ente al que se le calculara la sombra que proyecta.
    // - `angulo`.
    // - `posicion_flatlander`.
    // - `altura_flatlander`.
    //
    fn test_generar_sombra_flatlander_general(
        flatlander: Flatlander,
        angulo: f64,
        posicion_flatlander: f64,
        altura_flatlander: f64,
    ) {
        let angulo_sol_radianes =
            angulo * std::f64::consts::PI / crate::utils::constantes::CIENTO_OCHENTA;

        let plano = PlanoBidimensional::new(angulo_sol_radianes, vec![flatlander]);

        // Act: generar sombras
        let sombra_flatlander = plano.generar_sombra_flatlander(0);

        assert_eq!(
            Sombra::new(
                posicion_flatlander,
                posicion_flatlander + (altura_flatlander / angulo_sol_radianes.tan())
            ),
            sombra_flatlander
        );
    }

    // Test generico para la funcion 'calcular_longitud_sombra'.
    // A partir de un angulo y un vector de flatlanders, se valida que la funcion devuelva la longitud_esperada de la sombra.
    //
    // #Parametros
    // - `vector_flatlanders`.
    // - `angulo`.
    // - `longitud_esperada`.
    //
    fn test_calcular_longitud_sombra_plano_general(
        vector_flatlanders: Vec<Flatlander>,
        angulo: f64,
        longitud_esperada: f64,
    ) {
        let angulo_sol_radianes =
            angulo * std::f64::consts::PI / crate::utils::constantes::CIENTO_OCHENTA;

        let plano = PlanoBidimensional::new(angulo_sol_radianes, vector_flatlanders);

        let longitud_sombra = plano.calcular_longitud_sombra();

        assert!(
            (longitud_esperada - longitud_sombra).abs()
                < crate::utils::constantes::COTA_ERROR_PERMITIDA_TEST
        );
    }
}
