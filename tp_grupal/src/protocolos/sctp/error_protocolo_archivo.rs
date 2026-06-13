/// Errores específicos del protocolo de transferencia de archivos sobre SCTP
#[derive(Debug)]
pub enum ErrorProtocoloArchivo {
    MensajeVacio,
    TipoDesconocido(u8),
    PayloadInsuficiente,
    NombreInvalido,
}

impl std::fmt::Display for ErrorProtocoloArchivo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorProtocoloArchivo::MensajeVacio => write!(f, "Mensaje vacío"),
            ErrorProtocoloArchivo::TipoDesconocido(t) => write!(f, "Tipo desconocido: 0x{:02X}", t),
            ErrorProtocoloArchivo::PayloadInsuficiente => write!(f, "Payload insuficiente"),
            ErrorProtocoloArchivo::NombreInvalido => {
                write!(f, "Nombre de archivo inválido (UTF-8)")
            }
        }
    }
}
