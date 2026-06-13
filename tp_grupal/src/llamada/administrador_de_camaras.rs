use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    sync::{Arc, Mutex},
};

use nokhwa::utils::ApiBackend;

use crate::llamada::camara_mock::CamaraMock;
use crate::llamada::{
    camara::{Camara, CamaraGenerica},
    creador_lente_nokwha::CreadorDeLenteNokwha,
};

#[derive(PartialEq, Debug, Hash)]
pub enum FuenteDeVideo {
    Nokwha,
    FuenteTest,
}

impl Display for FuenteDeVideo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FuenteDeVideo::Nokwha => f.write_str("Nokwha"),
            FuenteDeVideo::FuenteTest => f.write_str("FuenteTest"),
        }
    }
}

impl Eq for FuenteDeVideo {}

const NOMBRE_CAMARA_TEST: &str = "Camara test";

#[derive(Debug)]
pub enum ErrorAdministradorDeCamaras {
    ErrorListandoCamarasDeFuente(FuenteDeVideo),
    ErrorInterno,
    ErrorCamaraInvalida,
}

impl Display for ErrorAdministradorDeCamaras {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorAdministradorDeCamaras::ErrorInterno => f.write_str("Error interno"),
            ErrorAdministradorDeCamaras::ErrorListandoCamarasDeFuente(fuente) => {
                f.write_str(&format!("Error listando camaras de: {fuente}"))
            }
            ErrorAdministradorDeCamaras::ErrorCamaraInvalida => {
                f.write_str("Error: Se pidio una camara invalida")
            }
        }
    }
}

pub struct AdministradorDeCamaras {
    fuentes_a_usar: Vec<FuenteDeVideo>,
    camaras: HashMap<FuenteDeVideo, HashSet<String>>,
}

impl AdministradorDeCamaras {
    pub fn new(fuentes_a_usar: Vec<FuenteDeVideo>) -> AdministradorDeCamaras {
        AdministradorDeCamaras {
            fuentes_a_usar,
            camaras: HashMap::new(),
        }
    }

    pub fn camaras_disponibles(&mut self) -> Result<Vec<String>, ErrorAdministradorDeCamaras> {
        let mut camaras = vec![];

        if self.fuentes_a_usar.contains(&FuenteDeVideo::FuenteTest) {
            self.agregar_camaras_test(&mut camaras)?;
        }

        if self.fuentes_a_usar.contains(&FuenteDeVideo::Nokwha) {
            self.agregar_camaras_nokwa(&mut camaras)?;
        }

        Ok(camaras)
    }

    fn agregar_camaras_nokwa(
        &mut self,
        camaras: &mut Vec<String>,
    ) -> Result<(), ErrorAdministradorDeCamaras> {
        let camera_infos = nokhwa::query(ApiBackend::Auto).map_err(|_| {
            ErrorAdministradorDeCamaras::ErrorListandoCamarasDeFuente(FuenteDeVideo::Nokwha)
        })?;

        // Obtengo set de camaras nokwha
        let info_camaras = camera_infos;
        self.camaras.insert(FuenteDeVideo::Nokwha, HashSet::new());
        let camaras_nokwha = self
            .camaras
            .get_mut(&FuenteDeVideo::Nokwha)
            .ok_or(ErrorAdministradorDeCamaras::ErrorInterno)?;

        for info_camara in &info_camaras {
            let nombre = info_camara.human_name();
            let indice = info_camara.index().to_string();
            let nombre_camara = format!("{indice} - {nombre}");

            camaras.push(nombre_camara.clone());
            camaras_nokwha.insert(nombre_camara);
        }

        Ok(())
    }

    fn agregar_camaras_test(
        &mut self,
        camaras: &mut Vec<String>,
    ) -> Result<(), ErrorAdministradorDeCamaras> {
        let nombre_camara = NOMBRE_CAMARA_TEST.to_string();

        // Obtengo set de camaras test
        self.camaras
            .insert(FuenteDeVideo::FuenteTest, HashSet::new());
        let camaras_test = self
            .camaras
            .get_mut(&FuenteDeVideo::FuenteTest)
            .ok_or(ErrorAdministradorDeCamaras::ErrorInterno)?;

        camaras_test.insert(nombre_camara.clone());
        camaras.push(nombre_camara);

        Ok(())
    }

    pub fn crear_camara(
        &mut self,
        nombre_camara: &str,
    ) -> Result<Box<dyn Camara>, ErrorAdministradorDeCamaras> {
        let camara: Box<dyn Camara>;

        if let Some(set_camaras) = self.camaras.get(&FuenteDeVideo::FuenteTest)
            && set_camaras.contains(nombre_camara)
        {
            camara = Box::new(Arc::new(Mutex::new(CamaraMock::default())));
            Ok(camara)
        } else if let Some(set_camaras) = self.camaras.get(&FuenteDeVideo::Nokwha)
            && set_camaras.contains(nombre_camara)
        {
            let indice_str = nombre_camara
                .split("-")
                .next()
                .ok_or(ErrorAdministradorDeCamaras::ErrorInterno)?;

            let indice_camara: u32 = indice_str
                .trim()
                .parse()
                .map_err(|_| ErrorAdministradorDeCamaras::ErrorInterno)?;

            let creador_de_lente = CreadorDeLenteNokwha::new(indice_camara);
            camara = Box::new(CamaraGenerica::new(Box::new(creador_de_lente)));
            Ok(camara)
        } else {
            Err(ErrorAdministradorDeCamaras::ErrorCamaraInvalida)
        }
    }
}

#[test]
fn test_01_no_se_lista_ninguna_camara_si_no_se_agrega_ninguna_fuente() {
    let mut administrador = AdministradorDeCamaras::new(vec![]);

    let camaras_disponibles = administrador.camaras_disponibles().unwrap();

    assert!(camaras_disponibles.is_empty())
}

#[test]
fn test_02_se_listan_camaras_de_las_fuentes_agregadas() {
    let mut administrador = AdministradorDeCamaras::new(vec![FuenteDeVideo::FuenteTest]);

    let camaras_disponibles = administrador.camaras_disponibles().unwrap();

    assert!(camaras_disponibles.contains(&NOMBRE_CAMARA_TEST.to_string()));
}

#[test]
fn test_03_devuelve_camara_al_pedir_que_cree_una_camara() {
    let mut administrador = AdministradorDeCamaras::new(vec![FuenteDeVideo::FuenteTest]);

    let camaras_disponibles = administrador.camaras_disponibles().unwrap();
    let camara = administrador.crear_camara(&camaras_disponibles[0]);

    assert!(camara.is_ok())
}

#[test]
fn test_04_falla_si_se_pide_crear_una_camara_que_no_pertenece_a_ninguna_fuente() {
    let mut administrador = AdministradorDeCamaras::new(vec![FuenteDeVideo::FuenteTest]);

    let camara = administrador.crear_camara("Juan");

    assert!(camara.is_err())
}
