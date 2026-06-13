//! # Paquete RTCP

use super::error::ErrorPaqueteRTCP;
use super::tipo_paquete::ContenidoPaqueteRTCP;

const VERSION_RTCP_VALIDA: u8 = 2; //(1000 0000) Los primeros dos bits son la version
const TAMANIO_HEADER_PAQUETE_RTCP: u16 = 8; // En bytes
const TAMANIO_BLOQUES_PAQUETE_RTCP: u16 = 4; //En bytes
pub const CONFIGURACIONES_DEFAULT: u8 = VERSION_RTCP_VALIDA << 6; //(1000 0000) Los primeros dos bits son la version

#[derive(Debug, PartialEq)]
/// Representación de un paquete del protocolo RTCP (RFC 1889)
pub struct PaqueteRTCP {
    pub configuraciones: u8,
    pub longitud_paquete: u16,
    pub ssrc: u32,
    pub payload: ContenidoPaqueteRTCP,
}

impl PaqueteRTCP {
    /// Crea un
    pub fn crear(ssrc: u32, payload: ContenidoPaqueteRTCP) -> PaqueteRTCP {
        // El tamanio va en bloques de 32 bits menos uno
        // El header mide dos bloques => Tamanio_total = Tamanio_payload + 1
        let longitud_paquete = Self::calcular_tamanio_paquete(payload.tamanio());

        PaqueteRTCP {
            configuraciones: CONFIGURACIONES_DEFAULT,
            longitud_paquete,
            ssrc,
            payload,
        }
    }

    /// Calcula el tamanio total en bytes del paquete,
    /// El tamaño de los payloads esta en bloques de 32 bits.
    /// El header mide 8 bytes
    fn calcular_tamanio_paquete(tamanio_payload_bloques: u16) -> u16 {
        tamanio_payload_bloques * TAMANIO_BLOQUES_PAQUETE_RTCP + TAMANIO_HEADER_PAQUETE_RTCP
    }
}

impl TryFrom<&[u8]> for PaqueteRTCP {
    type Error = ErrorPaqueteRTCP;

    fn try_from(bytes_paquete: &[u8]) -> Result<Self, Self::Error> {
        if bytes_paquete.len() < 3 {
            return Err(ErrorPaqueteRTCP::HeaderIncompleto);
        }

        // Compruebo que las configuraciones son correctas
        let configuraciones = bytes_paquete[0];

        // Compruebo la versión
        // La versión son los primeros dos bits del primer byte del paquete
        if configuraciones >> 6 != 2 {
            return Err(ErrorPaqueteRTCP::VersionInvalida);
        }

        // Compruebo el tipo de paquete
        let num_tipo_paquete = bytes_paquete[1];

        // Obtengo el ssrc de quien envia el paquete
        let ssrc = u32::from_be_bytes([
            bytes_paquete[4],
            bytes_paquete[5],
            bytes_paquete[6],
            bytes_paquete[7],
        ]);

        let payload = ContenidoPaqueteRTCP::crear_con_tipo(num_tipo_paquete, &bytes_paquete[8..])?;

        Ok(PaqueteRTCP::crear(ssrc, payload))
    }
}

impl From<&PaqueteRTCP> for Vec<u8> {
    fn from(paquete: &PaqueteRTCP) -> Self {
        let mut bytes_paquete: Vec<u8> = vec![];

        bytes_paquete.push(paquete.configuraciones);

        let numero_tipo = paquete.payload.numero_tipo();
        bytes_paquete.push(numero_tipo);

        let tamanio = paquete.payload.tamanio() + 1;
        let tamanio_bytes_be = tamanio.to_be_bytes();
        bytes_paquete.push(tamanio_bytes_be[0]);
        bytes_paquete.push(tamanio_bytes_be[1]);

        let ssrc_bytes = paquete.ssrc.to_be_bytes();
        bytes_paquete.push(ssrc_bytes[0]);
        bytes_paquete.push(ssrc_bytes[1]);
        bytes_paquete.push(ssrc_bytes[2]);
        bytes_paquete.push(ssrc_bytes[3]);

        let bytes_contenido: Vec<u8> = Vec::from(&paquete.payload);

        for byte in bytes_contenido {
            bytes_paquete.push(byte);
        }

        bytes_paquete
    }
}
