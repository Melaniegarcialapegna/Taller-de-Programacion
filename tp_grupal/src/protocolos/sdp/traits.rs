/// Trait que define la capacidad de parsear un conjunto de líneas SDP
/// a una estructura concreta (sesión, media o mensaje completo).
///
/// Devuelve un `Result` para manejar errores de parseo de manera segura.
pub trait ParseableSdp {
    /// Parsea un slice de líneas SDP y devuelve la estructura correspondiente.
    ///
    /// Todos los tipos concretos que implementen este trait deben tener tamaño conocido en tiempo de compilación.
    fn parsear(lineas: &[&str]) -> Result<Self, String>
    where
        Self: Sized;
}

/// Trait común para cualquier componente SDP (sesión, media, mensaje completo)
/// que pueda serializarse al formato SDP (texto plano con CRLF).
pub trait SerializableSdp {
    /// Serializa la estructura al formato estándar SDP.
    fn serializar(&self) -> String;
}
