///Errores que pueden generarse en [`servidor_central`]

#[derive(Debug)]
pub enum ErrorServidorCentral {
    ErrorEnviandoMensajeUsuario,
    LeyendoArchivoUsuarios,
    ErrorPersistiendoUsuario,
    CreandoArchivoUsuarios,
}
