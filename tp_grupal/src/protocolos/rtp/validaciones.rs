use super::constantes::*;
/// Modulo con validaciones necesarias al momento de crear un [`PaqueteRTP`]
use super::error::ErrorPaqueteRTP;

/// Valida que el header al menos cumpla con la cantidad minima de bytes segun el protocolo estandar.
pub fn validar_bytes_minimos_header(paquete_en_bytes: &[u8]) -> Result<(), ErrorPaqueteRTP> {
    if paquete_en_bytes.len() < CANT_MIN_BYTES_HEADER {
        return Err(ErrorPaqueteRTP::BytesInsuficientes);
    }
    Ok(())
}

///Valida que la version del paquete rtp sea 2
pub fn validar_version(version: u8) -> Result<(), ErrorPaqueteRTP> {
    if version != VERSION_VALIDA {
        return Err(ErrorPaqueteRTP::VersionInvalida);
    }
    Ok(())
}

///Obtiene la cantidad de bytes de padding que tiene el header luego del payload.
pub fn obtener_bytes_padding(
    padding: u8,
    paquete_en_bytes: &[u8],
) -> Result<usize, ErrorPaqueteRTP> {
    let mut padding_bytes = 0;
    if padding == PADDING_ACTIVADO {
        let longitud_padding = match paquete_en_bytes.last() {
            Some(&longitud) => longitud as usize,
            None => return Err(ErrorPaqueteRTP::ValorPaddingInvalido),
        };
        if longitud_padding == VALOR_PADDING_NULO || longitud_padding > paquete_en_bytes.len() {
            eprintln!("3");
            return Err(ErrorPaqueteRTP::ValorPaddingInvalido);
        }
        padding_bytes = longitud_padding;
    }
    Ok(padding_bytes)
}

///Valida que la extension no valga uno ya que la extension para el header se encuentra inabilitada.
pub fn validar_extension(extension: u8) -> Result<(), ErrorPaqueteRTP> {
    if extension == EXTENSION_ACTIVADA {
        eprintln!("4");
        return Err(ErrorPaqueteRTP::ExtensionInabilitada);
    }
    Ok(())
}

///Valida que la cantidad e bytes sea consistene con el conteo de csrc.
pub fn validar_cantidad_bytes(
    offset: usize,
    paquete_en_bytes: &[u8],
) -> Result<(), ErrorPaqueteRTP> {
    if paquete_en_bytes.len() < offset + CUATRO_BYTES {
        eprintln!("5");
        return Err(ErrorPaqueteRTP::BytesInsuficientes);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use crate::protocolos::rtp::constantes::*;
    use crate::protocolos::rtp::validaciones::*;

    #[test]
    fn validar_bytes_minimos_header_falla_si_insuficientes() {
        let bytes = vec![0u8; CANT_MIN_BYTES_HEADER - 1];
        let res = validar_bytes_minimos_header(&bytes);
        assert!(matches!(res, Err(ErrorPaqueteRTP::BytesInsuficientes)));
    }

    #[test]
    fn validar_bytes_minimos_header_ok_en_limite() {
        let bytes = vec![0u8; CANT_MIN_BYTES_HEADER];
        let res = validar_bytes_minimos_header(&bytes);
        assert!(res.is_ok());
    }

    #[test]
    fn validar_version_valida_y_no_valida() {
        let ok = validar_version(VERSION_VALIDA);
        assert!(ok.is_ok());

        // probar una version distinta
        let bad = validar_version(VERSION_VALIDA.wrapping_add(1));
        assert!(matches!(bad, Err(ErrorPaqueteRTP::VersionInvalida)));
    }

    #[test]
    fn obtener_bytes_padding_no_activado_devuelve_cero() {
        let bytes = vec![1u8, 2, 3];
        let res = obtener_bytes_padding(0, &bytes).unwrap();
        assert_eq!(res, 0);
    }

    #[test]
    fn obtener_bytes_padding_activado_valido() {
        // crear paquete con 5 bytes y último valor 3 -> padding 3
        let mut bytes = vec![10u8, 11, 12, 13, 3];
        let res = obtener_bytes_padding(PADDING_ACTIVADO, &bytes).unwrap();
        assert_eq!(res, 3);

        // también probar cuando padding ocupa todo el paquete (longitud == len)
        bytes = vec![4u8; 4];
        let res2 = obtener_bytes_padding(PADDING_ACTIVADO, &bytes).unwrap();
        assert_eq!(res2, 4);
    }

    #[test]
    fn obtener_bytes_padding_falla_con_paquete_vacio() {
        let bytes: Vec<u8> = vec![];
        let res = obtener_bytes_padding(PADDING_ACTIVADO, &bytes);
        assert!(matches!(res, Err(ErrorPaqueteRTP::ValorPaddingInvalido)));
    }

    #[test]
    fn obtener_bytes_padding_falla_con_longitud_cero_o_mayor() {
        // si el ultimo byte es 0 -> invalido
        let bytes = vec![1u8, 2, 0];
        let res = obtener_bytes_padding(PADDING_ACTIVADO, &bytes);
        assert!(matches!(res, Err(ErrorPaqueteRTP::ValorPaddingInvalido)));

        // si el ultimo byte > len -> invalido (ej: último = 10, len = 3)
        let bytes2 = vec![1u8, 2, 10];
        let res2 = obtener_bytes_padding(PADDING_ACTIVADO, &bytes2);
        assert!(matches!(res2, Err(ErrorPaqueteRTP::ValorPaddingInvalido)));
    }

    #[test]
    fn validar_extension_activada_falla_y_desactivada_ok() {
        let bad = validar_extension(EXTENSION_ACTIVADA);
        assert!(matches!(bad, Err(ErrorPaqueteRTP::ExtensionInabilitada)));

        let ok = validar_extension(0);
        assert!(ok.is_ok());
    }

    #[test]
    fn validar_cantidad_bytes_falla_si_no_hay_cuatro_bytes_extra() {
        let offset = 5usize;
        // paquete con longitud menor que offset + CUATRO_BYTES
        let bytes = vec![0u8; offset + CUATRO_BYTES - 1];
        let res = validar_cantidad_bytes(offset, &bytes);
        assert!(matches!(res, Err(ErrorPaqueteRTP::BytesInsuficientes)));
    }

    #[test]
    fn validar_cantidad_bytes_ok_si_hay_espacio() {
        let offset = 2usize;
        let bytes = vec![0u8; offset + CUATRO_BYTES];
        let res = validar_cantidad_bytes(offset, &bytes);
        assert!(res.is_ok());
    }
}
