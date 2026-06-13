use super::constantes::*;
use super::error::ErrorPaqueteRTP;
use super::validaciones;

//Ver tema de que carga util tiene un tipo de longitud definida!

/// Representa un paquete RTP segun protocolo (RFC 3550)
///
/// Esta estructura contiene los campos del header de RTP estandar,lista de CSRCs,payload,
/// y bytes de padding los cuales se utilizan para poder replicar la estructura cuando se descodifica.
///
/// Para poder crear un PaqueteRTP a partir de Bytes [`PaqueteRTP::generar_paquete`]
/// Para serializar un PaqueteRTP [`PaqueteRTP::decodificar_paquete`].
#[derive(Debug, PartialEq)]
pub struct PaqueteRTP {
    /// Version del protocolo RTP, debe ser 2.
    pub version: u8,
    /// Indica si hay padding.
    pub padding: u8,
    /// Indica si hay una extension en el header.
    pub extension: u8, //De momento siempre 0
    /// Cantidad de CSRCs.
    pub conteo_csrc: u8,
    ///Bit de marcador.
    pub marcador: u8,
    /// Tipo de carga util (payload type)
    pub tipo_payload: u8,
    /// Numero de secuencia.
    pub numero_de_secuencia: u16,
    /// Timestamp del paquete.
    pub timestamp: u32,
    /// Identificador SSRC.
    pub ssrc: u32,
    /// Lista de CSRCs.
    pub lista_csrc: Vec<u32>,
    /// Datos.
    pub payload: Vec<u8>,
    /// Cantidad de bytes de padding luego del payload.
    pub padding_bytes: usize,
}

//si extension 1 -> ver si necesario
// pub struct ExtensionPaqueteRTP{
//  perfil: u16,//16 bits
//  longitud: u16,//16 bits
//  datos: Vec<u8>,
// }

//RECORDATORIO : estandar no exige que el payload o el padding esten alineados, salvo en casos especiales como extensiones.. si hacemos extension tener esto en cuenta!

impl TryFrom<&[u8]> for PaqueteRTP {
    type Error = ErrorPaqueteRTP;

    /// Genera un [`PaqueteRTP`] a partir de un vector de bytes.
    ///
    /// Si ocurre algun error durante la creacion del paquete se retorna un [`ErrorPaqueteRTP`]
    fn try_from(paquete_en_bytes: &[u8]) -> Result<Self, Self::Error> {
        validaciones::validar_bytes_minimos_header(paquete_en_bytes)?;

        //TOMO PRIMER BYTE
        let byte_0 = paquete_en_bytes[0]; //version,padding,extension,conteo_csrc

        let version = (byte_0 >> 6) & 0x03;
        validaciones::validar_version(version)?;
        let padding = (byte_0 >> 5) & 0x01;
        let padding_bytes = validaciones::obtener_bytes_padding(padding, paquete_en_bytes)?;
        let extension = (byte_0 >> 4) & 0x01;
        validaciones::validar_extension(extension)?;
        let conteo_csrc = byte_0 & 0x0F;

        //TOMO SEGUNDO BYTE
        let byte_1: u8 = paquete_en_bytes[1]; //marcador,tipo_carga_util

        let marcador = byte_1 >> 7;
        let tipo_payload = byte_1 & 0x7F; //No es necesario verificar que no sea RR o SR ya que al estar representado por 7 bits nunca podra valer 200 o 201
        let numero_de_secuencia = u16::from_be_bytes([paquete_en_bytes[2], paquete_en_bytes[3]]);

        let timestamp = u32::from_be_bytes([
            paquete_en_bytes[4],
            paquete_en_bytes[5],
            paquete_en_bytes[6],
            paquete_en_bytes[7],
        ]);

        let ssrc = u32::from_be_bytes([
            paquete_en_bytes[8],
            paquete_en_bytes[9],
            paquete_en_bytes[10],
            paquete_en_bytes[11],
        ]);

        //SI CONTEO_CSRC > 0
        let mut offset = OFFSET_BYTES_HEADER;
        let mut lista_csrc: Vec<u32> = Vec::new();
        for _ in 0..conteo_csrc {
            validaciones::validar_cantidad_bytes(offset, paquete_en_bytes)?;
            let csrc = u32::from_be_bytes([
                paquete_en_bytes[offset],
                paquete_en_bytes[offset + 1],
                paquete_en_bytes[offset + 2],
                paquete_en_bytes[offset + 3],
            ]);
            lista_csrc.push(csrc);
            offset += CUATRO_BYTES;
        }

        //EL RESTO ES PAYLOAD
        //Se quita el padding
        let payload = paquete_en_bytes[offset..paquete_en_bytes.len() - padding_bytes].to_vec();

        Ok(PaqueteRTP {
            version,
            padding,
            extension,
            conteo_csrc,
            marcador,
            tipo_payload,
            numero_de_secuencia,
            timestamp,
            ssrc,
            lista_csrc,
            payload,
            padding_bytes,
        })
    }
}

impl From<&PaqueteRTP> for Vec<u8> {
    /// A partir de un [`PaqueteRTP`] genera un vector de bytes.
    ///
    /// Si ocurre algun error durante la creacion del paquete se retorna un [`ErrorPaqueteRTP`]
    fn from(paquete_rtp: &PaqueteRTP) -> Vec<u8> {
        let mut informacion_bytes = Vec::new();

        let byte_0 = ((paquete_rtp.version & 0x03) << 6)
            | ((paquete_rtp.padding & 0x01) << 5)
            | ((paquete_rtp.extension & 0x01) << 4)
            | (paquete_rtp.conteo_csrc & 0x0F);
        informacion_bytes.push(byte_0);

        let byte_1 = (paquete_rtp.marcador & 0x01) << 7 | (paquete_rtp.tipo_payload & 0x7F);
        informacion_bytes.push(byte_1);

        for byte in paquete_rtp.numero_de_secuencia.to_be_bytes() {
            informacion_bytes.push(byte);
        }

        for byte in paquete_rtp.timestamp.to_be_bytes() {
            informacion_bytes.push(byte);
        }

        for byte in paquete_rtp.ssrc.to_be_bytes() {
            informacion_bytes.push(byte);
        }

        for csrc in &paquete_rtp.lista_csrc {
            for byte in csrc.to_be_bytes() {
                informacion_bytes.push(byte);
            }
        }

        for byte in &paquete_rtp.payload {
            informacion_bytes.push(*byte);
        }

        if paquete_rtp.padding == PADDING_ACTIVADO && paquete_rtp.padding_bytes > VALOR_PADDING_NULO
        {
            informacion_bytes.extend(vec![0x00; paquete_rtp.padding_bytes - 1]);
            informacion_bytes.push(paquete_rtp.padding_bytes as u8);
        }

        informacion_bytes
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test01_cantidad_bytes_invalidos() {
        let informacion_bytes: [u8; 2] = [0; 2];
        let paquete_rtp = PaqueteRTP::try_from(&informacion_bytes[..]);

        assert!(paquete_rtp.is_err());
        assert_eq!(
            paquete_rtp.unwrap_err(),
            ErrorPaqueteRTP::BytesInsuficientes
        )
    }

    #[test]
    fn test02_version_invalida() {
        let mut informacion_bytes: [u8; 12] = [0; 12];
        informacion_bytes[0] = 0x01 << 6;
        let paquete_rtp = PaqueteRTP::try_from(&informacion_bytes[..]);

        assert!(paquete_rtp.is_err());
        assert_eq!(paquete_rtp.unwrap_err(), ErrorPaqueteRTP::VersionInvalida)
    }

    #[test]
    fn test03_valor_padding_invalido() {
        let mut informacion_bytes: [u8; 12] = [0; 12];
        let version = 0x01 << 7; //version en 2
        let padding = 0x01 << 5; //padding en 1
        let byte_0 = version | padding;
        informacion_bytes[0] = byte_0;
        let paquete_rtp = PaqueteRTP::try_from(&informacion_bytes[..]);

        assert!(paquete_rtp.is_err());
        assert_eq!(
            paquete_rtp.unwrap_err(),
            ErrorPaqueteRTP::ValorPaddingInvalido
        )
    }

    //El paquete ademas del header deberia tener una cantidad de bytes que permita contener dos csrc
    #[test]
    fn test04_cantidad_bytes_invilido() {
        let mut informacion_bytes: [u8; 12] = [0; 12];
        let version = 0x01 << 7; //version en 2
        let conteo_csrc = 0x02; //2 csrc en lista
        let byte_0 = version | conteo_csrc;
        informacion_bytes[0] = byte_0;
        let paquete_rtp = PaqueteRTP::try_from(&informacion_bytes[..]);

        assert!(paquete_rtp.is_err());
        assert_eq!(
            paquete_rtp.unwrap_err(),
            ErrorPaqueteRTP::BytesInsuficientes
        )
    }

    //La extension del header no esta habilidata
    #[test]
    fn test05_extension_uno_invalida() {
        let mut informacion_bytes: [u8; 12] = [0; 12];
        let version = 0x01 << 7; //version en 2
        let extension = 0x01 << 4; //extension en 1
        informacion_bytes[0] = version | extension;
        let paquete_rtp = PaqueteRTP::try_from(&informacion_bytes[..]);

        assert!(paquete_rtp.is_err());
        assert_eq!(
            paquete_rtp.unwrap_err(),
            ErrorPaqueteRTP::ExtensionInabilitada
        )
    }

    /// Estructura para testear ya que se supera el limite de cantidad de parametros en la llamada de una funcion.
    struct DataTestear {
        version: u8,
        padding: u8,
        extension: u8,
        conteo_csrc: u8,
        marcador: u8,
        tipo_payload: u8,
        numero_de_secuencia: u16,
        timestamp: u32,
        ssrc: u32,
        lista_csrc: Vec<u32>,
        payload: Vec<u8>,
        padding_bytes: usize,
    }

    //Al decodificar un PaqueteRTP y luego volver a codificarlo se vuelve al mismo PaqueteRTP original
    //Se testea con distintos valores
    #[test]
    fn test06_ciclo_completo() {
        test_ciclo_completo_generico(DataTestear {
            version: 2,
            padding: 0,
            extension: 0,
            conteo_csrc: 2,
            marcador: 1,
            tipo_payload: 96,
            numero_de_secuencia: 2404,
            timestamp: 2222222,
            ssrc: 24112004,
            lista_csrc: vec![2222, 4444],
            payload: vec![10, 20, 30, 40],
            padding_bytes: 0,
        });

        test_ciclo_completo_generico(DataTestear {
            version: 2,
            padding: 1,
            extension: 0,
            conteo_csrc: 4,
            marcador: 1,
            tipo_payload: 96,
            numero_de_secuencia: 2404,
            timestamp: 2222222,
            ssrc: 24112004,
            lista_csrc: vec![2222, 4444, 2222, 4444],
            payload: vec![10, 20, 30, 40, 50, 60, 70],
            padding_bytes: 14,
        });
    }

    fn test_ciclo_completo_generico(data_paquete: DataTestear) {
        let paquete_original = PaqueteRTP {
            version: data_paquete.version,
            padding: data_paquete.padding,
            extension: data_paquete.extension,
            conteo_csrc: data_paquete.conteo_csrc,
            marcador: data_paquete.marcador,
            tipo_payload: data_paquete.tipo_payload,
            numero_de_secuencia: data_paquete.numero_de_secuencia,
            timestamp: data_paquete.timestamp,
            ssrc: data_paquete.ssrc,
            lista_csrc: data_paquete.lista_csrc,
            payload: data_paquete.payload,
            padding_bytes: data_paquete.padding_bytes,
        };

        let paquete_en_bytes = Vec::from(&paquete_original);

        let paquete_decodificado = PaqueteRTP::try_from(&paquete_en_bytes[..])
            .expect("Error al generar nuevamente el PaqueteRTP");

        assert_eq!(paquete_original, paquete_decodificado);
    }
}
