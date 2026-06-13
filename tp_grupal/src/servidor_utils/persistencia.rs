use std::{fs, path::Path};

/// Carga usuarios desde un archivo .txt en formato: nombre;contrasena
///
/// # Arguments
/// * `ruta_archivo` - Ruta al archivo de usuarios.
/// # Returns
/// * `Vec<(String, String)>` - Vector de tuplas con (nombre, contrasena).
pub fn cargar_desde_archivo(ruta_archivo: &str) -> Vec<(String, String)> {
    if !Path::new(ruta_archivo).exists() {
        return vec![];
    }

    let contenido = match fs::read_to_string(ruta_archivo) {
        Ok(c) => c,
        Err(_) => return vec![], // Si hay error leyendo, devolvemos un vector vacío
    };

    contenido
        .lines()
        .filter_map(|linea| {
            let partes: Vec<&str> = linea.trim().split(';').collect();

            if partes.len() != 2 {
                // LOGUEAR: línea mal formateada
                return None; // por ahora ignoramos lineas mal formateadas (no debería pasar porque las escribimos nosotros, las logueamos)
            }

            Some((partes[0].to_string(), partes[1].to_string()))
        })
        .collect()
}

/// Guarda usuarios al archivo .txt en formato: nombre;contrasena
///
/// # Arguments
/// * `ruta_archivo` - Ruta al archivo de usuarios.
/// * `usuarios` - Slice de tuplas con (nombre, contrasena).
pub fn guardar_en_archivo(ruta_archivo: &str, usuarios: &[(String, String)]) {
    let mut buffer = String::new();

    for (nombre, pass) in usuarios {
        buffer.push_str(&format!("{};{}\n", nombre, pass));
    }

    let _ = fs::write(ruta_archivo, buffer);
    // LOGUEAR: si hay error o si escribe bien
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, time::SystemTime};

    // auxiliares para tests
    fn path_unico_temporal(suffix: &str) -> PathBuf {
        let ns = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("persistencia_test_{}_{}.txt", ns, suffix))
    }

    fn eliminar_si_existe(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cargar_archivo_inexistente_devuelve_vacio() {
        let path = path_unico_temporal("noexiste");
        eliminar_si_existe(&path);
        let resultado = cargar_desde_archivo(path.to_str().unwrap());
        assert!(resultado.is_empty());
    }

    #[test]
    fn guardar_sobrescribe_archivo_existente() {
        let path = path_unico_temporal("overwrite");
        // contenido inicial
        fs::write(&path, "tmp;tmp\nother;val\n").unwrap();

        let usuarios_nuevos = vec![("nuevo".to_string(), "clave".to_string())];
        guardar_en_archivo(path.to_str().unwrap(), &usuarios_nuevos);

        let contenido = fs::read_to_string(&path).unwrap();
        eliminar_si_existe(&path);

        assert_eq!(contenido, "nuevo;clave\n");
    }

    #[test]
    fn guardar_y_cargar_usuarios() {
        let path = path_unico_temporal("guardar_cargar");
        let usuarios_originales = vec![
            ("user1".to_string(), "pass1".to_string()),
            ("user2".to_string(), "pass2".to_string()),
        ];
        guardar_en_archivo(path.to_str().unwrap(), &usuarios_originales);

        let usuarios_cargados = cargar_desde_archivo(path.to_str().unwrap());
        eliminar_si_existe(&path);

        assert_eq!(usuarios_originales, usuarios_cargados);
    }
}
