//! Este modulo define el tipo de dato `Flatlander`.

///Representa un ente en un eje X (una recta).
///
/// # Campos
/// - `posicion`: la posicion en la que se encuentra respecto al eje X.
/// - `altura` : la ultura del ente.
#[derive(Debug)]
pub struct Flatlander {
    pub posicion: u64,
    pub altura: u64,
}

impl Flatlander {
    ///Crea y devuelve una instancia de `Flatlander`
    ///
    /// Recibe una posicion respecto al eje X y una altura.
    pub fn new(posicion: u64, altura: u64) -> Flatlander {
        Flatlander { posicion, altura }
    }
}
