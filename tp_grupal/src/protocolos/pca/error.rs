/// Errores del protocolo
#[derive(Debug)]
pub enum ErrorMensajePCA {
    /// La primera palabra del mensaje es correcta, pero alguno de sus atributos esta incompleto
    ErrorMensajeIncompleto,
    /// El mensaje no pertenece al protocolo (ni esta cerca)
    ErrorMensajeInvalido,
}
