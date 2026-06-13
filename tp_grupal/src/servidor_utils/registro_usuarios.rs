use crate::servidor_utils::persistencia::{cargar_desde_archivo, guardar_en_archivo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Struct para manejar el registro y autenticación login de usuarios.
#[derive(Clone, Debug)]
pub struct RegistroUsuarios {
    usuarios: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for RegistroUsuarios {
    fn default() -> Self {
        Self::crear_registro_usuarios()
    }
}

impl RegistroUsuarios {
    /// Crea un nuevo registro de usuarios vacío.
    pub fn crear_registro_usuarios() -> Self {
        let lista = cargar_desde_archivo("usuarios.txt");

        let mut mapa = HashMap::new();
        for (user, pass) in lista {
            mapa.insert(user, pass);
        }

        Self {
            usuarios: Arc::new(Mutex::new(mapa)),
        }
    }

    /// Registra un nuevo usuario con nombre y contraseña.
    /// Si el usuario ya existe, devuelve un error.
    /// # Arguments
    /// * `nombre` - Nombre del usuario a registrar.
    /// * `contrasena` - Contraseña del usuario a registrar.
    /// # Returns
    /// * `Ok(())` si el registro fue exitoso.
    /// * `Err(String)` si el usuario ya existe o hay un error de acceso.
    pub fn registrar_usuario(&self, nombre: &str, contrasena: &str) -> Result<(), String> {
        let mut usuarios = self
            .usuarios
            .lock()
            .map_err(|_| "Error al acceder al registro de usuarios".to_string())?;

        if usuarios.contains_key(nombre) {
            return Err("El nombre de usuario ya existe".to_string());
        }

        usuarios.insert(nombre.to_string(), contrasena.to_string());

        let lista: Vec<(String, String)> = usuarios
            .iter()
            .map(|(u, p)| (u.clone(), p.clone()))
            .collect();

        // lo guardo cuando registro para atajar casos en los que se cierre de forma inesperada
        guardar_en_archivo("usuarios.txt", &lista);

        Ok(())
    }

    /// Autentica un usuario con nombre y contraseña.
    /// # Arguments
    /// * `nombre` - Nombre del usuario a autenticar.
    /// * `contrasena` - Contraseña del usuario a autenticar.
    /// # Returns
    /// * `Ok(())` si la autenticación fue exitosa.
    /// * `Err(String)` si las credenciales son incorrectas o hay un error de acceso.
    pub fn autenticar_usuario(&self, nombre: &str, contrasena: &str) -> Result<(), String> {
        let usuarios = self
            .usuarios
            .lock()
            .map_err(|_| "Error al acceder al registro de usuarios".to_string())?;

        match usuarios.get(nombre) {
            Some(contrasena_almacenada) if contrasena_almacenada == contrasena => Ok(()),
            _ => Err("Credenciales incorrectas".to_string()),
        }
    }

    // este va a servir para mostrar users en gui xq solo muestra el nombre
    /// Obtiene una lista de todos los nombres de usuario registrados.
    /// # Returns
    /// * `Ok(Vec<String>)` con los nombres de usuario.
    /// * `Err(String)` si hay un error de acceso.
    pub fn obtener_usuarios(&self) -> Result<Vec<String>, String> {
        let usuarios = self
            .usuarios
            .lock()
            .map_err(|_| "Error al acceder al registro de usuarios".to_string())?;

        Ok(usuarios.keys().cloned().collect())
    }

    /// Obtiene un HashMap completo de usuarios y sus contraseñas.
    /// # Returns
    /// * `Ok(HashMap<String, String>)` con los nombres de usuario y contraseñas.
    /// * `Err(String)` si hay un error de acceso.
    pub fn obtener_info_completa_usuarios_tuplas(&self) -> Result<Vec<(String, String)>, String> {
        let usuarios = self
            .usuarios
            .lock()
            .map_err(|_| "Error al acceder al registro de usuarios".to_string())?;

        Ok(usuarios
            .iter()
            .map(|(nombre, contrasena)| (nombre.clone(), contrasena.clone()))
            .collect())
    }

    /// Carga múltiples usuarios desde un vector de tuplas (nombre, contraseña) idealmente obtenido de un archivo.
    /// Se busca persistir usuarios previamente registrados.
    /// # Arguments
    /// * `datos` - Vector de tuplas con (nombre, contraseña).
    /// # Returns
    /// * `Ok(())` si la carga fue exitosa.
    /// * `Err(String)` si hay un error de acceso.
    pub fn cargar_usuarios(&self, datos: Vec<(String, String)>) -> Result<(), String> {
        let mut usuarios = self
            .usuarios
            .lock()
            .map_err(|_| "Error al acceder al registro de usuarios".to_string())?;

        for (nombre, contrasena) in datos {
            usuarios.insert(nombre, contrasena);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn test_registrar_y_autenticar() {
        let _ = std::fs::remove_file("usuarios.txt");
        let registro_usuarios = RegistroUsuarios::crear_registro_usuarios();
        assert!(
            registro_usuarios
                .registrar_usuario("alice", "pwd123")
                .is_ok()
        );
        assert!(
            registro_usuarios
                .autenticar_usuario("alice", "pwd123")
                .is_ok()
        );
        assert!(
            registro_usuarios
                .autenticar_usuario("alice", "wrong")
                .is_err()
        );
        assert!(
            registro_usuarios
                .autenticar_usuario("bob", "pwd123")
                .is_err()
        );
    }

    #[test]
    fn test_registrar_duplicado() {
        let _ = std::fs::remove_file("usuarios.txt");
        let registro_usuarios = RegistroUsuarios::crear_registro_usuarios();
        assert!(registro_usuarios.registrar_usuario("eve", "secret").is_ok());
        let err = registro_usuarios
            .registrar_usuario("eve", "other")
            .unwrap_err();
        assert_eq!(err, "El nombre de usuario ya existe");
    }

    #[test]
    fn test_obtener_usuarios_y_info() {
        let _ = std::fs::remove_file("usuarios.txt");
        let registro_usuarios = RegistroUsuarios::crear_registro_usuarios();
        registro_usuarios.registrar_usuario("u1", "p1").unwrap();
        registro_usuarios.registrar_usuario("u2", "p2").unwrap();

        let usuarios = registro_usuarios.obtener_usuarios().unwrap();
        let set: HashSet<_> = usuarios.into_iter().collect();
        let expected: HashSet<_> = ["u1".to_string(), "u2".to_string()].into_iter().collect();
        assert_eq!(set, expected);

        let info = registro_usuarios
            .obtener_info_completa_usuarios_tuplas()
            .unwrap();
        let info_set: HashSet<_> = info.into_iter().collect();
        let expected_info: HashSet<_> = vec![
            ("u1".to_string(), "p1".to_string()),
            ("u2".to_string(), "p2".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(info_set, expected_info);
    }

    #[test]
    fn test_cargar_usuarios() {
        let _ = std::fs::remove_file("usuarios.txt");
        let registro_usuarios = RegistroUsuarios::crear_registro_usuarios();
        let datos = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ];
        assert!(registro_usuarios.cargar_usuarios(datos).is_ok());

        let info = registro_usuarios
            .obtener_info_completa_usuarios_tuplas()
            .unwrap();
        let info_set: HashSet<_> = info.into_iter().collect();
        let expected: HashSet<_> = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(info_set, expected);
    }

    #[test]
    fn test_mutex_envenenado_devuelve_error() {
        let _ = std::fs::remove_file("usuarios.txt");
        let registro_usuarios = RegistroUsuarios::crear_registro_usuarios();
        // lo envenenamos paniqueando un hilo que tiene el lock
        let usuarios_arc = registro_usuarios.usuarios.clone();
        let handle = thread::spawn(move || {
            let _guard = usuarios_arc.lock().unwrap();
            panic!("poisoning mutex");
        });
        let _ = handle.join();

        let res = registro_usuarios.registrar_usuario("x", "y");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Error al acceder al registro de usuarios");
    }
}
