/// Protocolo de transferencia de archivos sobre SCTP
///
/// Este módulo define el protocolo de transferencia de archivos que se implementará sobre SCTP.
/// El protocolo incluye mensajes para ofrecer un archivo, aceptar o rechazar la oferta, enviar los datos del archivo en chunks, y señalar el fin de la
/// transferencia.
use bytes::Bytes;

use crate::protocolos::sctp::error_protocolo_archivo::ErrorProtocoloArchivo;

const TIPO_OFERTA_ARCHIVO: u8 = 0x01;
const TIPO_ACEPTAR_ARCHIVO: u8 = 0x02;
const TIPO_RECHAZAR_ARCHIVO: u8 = 0x03;
const TIPO_DATOS_ARCHIVO: u8 = 0x04;
const TIPO_FIN_ARCHIVO: u8 = 0x05;

/// Mensajes del protocolo de transferencia de archivos.
#[derive(Debug, Clone, PartialEq)]
pub enum MensajeArchivo {
    /// El emisor ofrece un archivo. Contiene nombre y tamaño en bytes.
    OfertaArchivo { nombre: String, tamanio: u64 },
    /// El receptor acepta la oferta.
    AceptarArchivo,
    /// El receptor rechaza la oferta.
    RechazarArchivo,
    /// Fragmento (o totalidad) de los bytes del archivo.
    DatosArchivo(Bytes),
    /// Señal de fin de transferencia -> el emisor terminó de enviar todos los chunks.
    FinArchivo,
}

impl MensajeArchivo {
    /// Serializa el mensaje a bytes listos para enviar por SCTP.
    pub fn serializar(&self) -> Bytes {
        match self {
            MensajeArchivo::OfertaArchivo { nombre, tamanio } => {
                let nombre_bytes = nombre.as_bytes();
                let mut buf = Vec::with_capacity(1 + 8 + nombre_bytes.len());
                buf.push(TIPO_OFERTA_ARCHIVO);
                buf.extend_from_slice(&tamanio.to_be_bytes());
                buf.extend_from_slice(nombre_bytes);
                Bytes::from(buf)
            }

            MensajeArchivo::AceptarArchivo => Bytes::from_static(&[TIPO_ACEPTAR_ARCHIVO]),

            MensajeArchivo::RechazarArchivo => Bytes::from_static(&[TIPO_RECHAZAR_ARCHIVO]),

            MensajeArchivo::DatosArchivo(datos) => {
                let mut buf = Vec::with_capacity(1 + datos.len());
                buf.push(TIPO_DATOS_ARCHIVO);
                buf.extend_from_slice(datos);
                Bytes::from(buf)
            }

            MensajeArchivo::FinArchivo => Bytes::from_static(&[TIPO_FIN_ARCHIVO]),
        }
    }

    /// Intenta deserializar un mensaje desde bytes recibidos por SCTP.
    pub fn deserializar(data: &Bytes) -> Result<MensajeArchivo, ErrorProtocoloArchivo> {
        if data.is_empty() {
            return Err(ErrorProtocoloArchivo::MensajeVacio);
        }
        let tipo = data[0];
        let payload = data.slice(1..);
        match tipo {
            TIPO_OFERTA_ARCHIVO => {
                if payload.len() < 8 {
                    return Err(ErrorProtocoloArchivo::PayloadInsuficiente);
                }
                let tamanio = u64::from_be_bytes(
                    payload[..8]
                        .try_into()
                        .map_err(|_| ErrorProtocoloArchivo::PayloadInsuficiente)?,
                );
                let nombre = String::from_utf8(payload[8..].to_vec())
                    .map_err(|_| ErrorProtocoloArchivo::NombreInvalido)?;

                Ok(MensajeArchivo::OfertaArchivo { nombre, tamanio })
            }

            TIPO_ACEPTAR_ARCHIVO => Ok(MensajeArchivo::AceptarArchivo),
            TIPO_RECHAZAR_ARCHIVO => Ok(MensajeArchivo::RechazarArchivo),
            TIPO_DATOS_ARCHIVO => Ok(MensajeArchivo::DatosArchivo(payload)),
            TIPO_FIN_ARCHIVO => Ok(MensajeArchivo::FinArchivo),
            otro => Err(ErrorProtocoloArchivo::TipoDesconocido(otro)),
        }
    }
}
