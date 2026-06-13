///Errores que pueden generarse en [`handler_usuario`]

#[derive(Debug)]
pub enum ErrorUsuario {
    LecturaStream,
    EnviandoMensajeServidorCentral,
    RecibiendoMensajeServidorCentral,
    EnviandoMensajeUsuarioPorStream,
    ObteniendoLock,
    MensajeFueraDeContexto,
    ErrorInterno,
}
