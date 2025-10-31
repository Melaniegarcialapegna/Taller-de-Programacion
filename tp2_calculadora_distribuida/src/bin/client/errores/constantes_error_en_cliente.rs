//Mensajes error
pub const FALTA_PARAMETRO: &str =
    "ERROR \"missing parameters. Usage: cargo run --bin server -- <address> <file>\"";
pub const ARCHIVO_INEXISTENTE: &str = "ERROR \"file does not exist\"";
pub const DIRECCION_INVALIDA: &str =
    "ERROR \"could not connect to the address passed as parameter\"";
pub const ABRIR_ARCHIVO: &str = "ERROR \"file could not be opened\"";
pub const LEER_LINEA_ARCHIVO: &str = "ERROR \"could not read a line from file\"";
pub const RESPUESTA_SERVIDOR: &str =
    "ERROR \"server did not send a response in the expected format\"";
