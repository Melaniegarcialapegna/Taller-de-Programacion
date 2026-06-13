/// Eventos relacionados con SCTP, como ofertas de archivos, aceptación/rechazo de ofertas, recepción de datos y finalización de transferencias.
use crate::protocolos::sctp::protocolo_archivo::MensajeArchivo;
use bytes::Bytes;

#[derive(Debug)]
pub enum EventoSctp {
    /// El peer remoto ofrece un archivo para transferir.
    OfertaArchivo { nombre: String, tamanio: u64 },
    /// El peer remoto aceptó nuestra oferta de archivo — podemos empezar a enviar los bytes.
    ArchivoAceptado,
    /// El peer remoto rechazó nuestra oferta de archivo.
    ArchivoRechazado,
    /// Fragmento (o totalidad) de bytes de un archivo recibido.
    DatosArchivo(Bytes),
    /// El emisor terminó de enviar todos los chunks del archivo.
    FinArchivo,
}

impl From<MensajeArchivo> for EventoSctp {
    fn from(mensaje: MensajeArchivo) -> Self {
        match mensaje {
            MensajeArchivo::OfertaArchivo { nombre, tamanio } => {
                EventoSctp::OfertaArchivo { nombre, tamanio }
            }
            MensajeArchivo::AceptarArchivo => EventoSctp::ArchivoAceptado,
            MensajeArchivo::RechazarArchivo => EventoSctp::ArchivoRechazado,
            MensajeArchivo::DatosArchivo(datos) => EventoSctp::DatosArchivo(datos),
            MensajeArchivo::FinArchivo => EventoSctp::FinArchivo,
        }
    }
}
