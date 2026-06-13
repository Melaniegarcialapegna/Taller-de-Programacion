#[derive(Debug, PartialEq)]
pub enum ErrorPaqueteRTP {
    ///Version distinta de 2
    VersionInvalida,
    ///Hay padding y el valor del byte final excede la longitud total del paquete
    ValorPaddingInvalido,
    ///Si hay extension y no esta alineada
    AlineamientoInvalido,
    ///Si no se llegan a cubrir los bytes necesarios
    BytesInsuficientes,
    ///Se pretende extender el header aunque no esta activado el mecanismo de extension
    ExtensionInabilitada,
}
