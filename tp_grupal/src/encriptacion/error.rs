/// Errores relacionados a la encriptación de un Stream mediante [`SistemaEncriptacion`]
#[derive(Debug)]
pub enum ErrorEncriptacion {
    ErrorCreandoClavePublica,
    ErrorEnviandoClavePublica,
    ErrorEnviandoClaveHash,
    ErrorEncriptandoMensaje,
    ErrorDesencriptandoMensaje,
    ErrorCreandoClaveHash,
}
