//! Módulo 'dtls_utils.rs'
//!
//! Este módulo proporciona utilidades para la generación y manejo de certificados DTLS,
//! cálculo de huellas digitales, y validación de fingerprints en el contexto de DTLS.
//! Incluye funciones para generar certificados auto firmados, convertir huellas a formato SDP,
//! y parsear atributos DTLS desde descripciones de sesión SDP.
//!
//! También maneja la lógica para determinar roles DTLS basados en atributos SDP.
use crate::seguridad::dtls_protocolo;
use crate::seguridad::dtls_protocolo::errores::ErrorDTLSProtocolo;
use openssl::hash::MessageDigest;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::{X509, X509Builder, X509Name, X509NameBuilder};
use udp_dtls::{Certificate, SignatureAlgorithm};
// uso sha2 para hashear la huella del certificado que voy a usar en el sdp
use crate::protocolos::sdp::descripcion_de_sesion::DescripcionDeSesion;
use dtls_protocolo::dtls_contexto::RolDtls;

// https://datatracker.ietf.org/doc/html/rfc8827
// https://www.ssl.com/guide/pem-der-crt-and-cer-x-509-encodings-and-conversions/

/// Genera un certificado DTLS auto firmado junto con su clave privada y PKCS#12.
///
/// # Returns
/// - `Result<(Certificate, PKey<openssl::pkey::Private>, Vec<u8>), ErrorDTLSProtocolo>`: Tupla con el certificado, clave privada y PKCS#12 o error si falla la generación.
pub fn generar_certificado_dtls()
-> Result<(Certificate, PKey<openssl::pkey::Private>, Vec<u8>), ErrorDTLSProtocolo> {
    let key = generar_clave_pkey()?;
    // construyo un nombre para el certificado
    let nombre = construir_nombre_certificado()?;

    // creo el certificado x.509
    let mut constructor_certificado =
        X509::builder().map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    crear_constructor_certificado(&nombre, &key, &mut constructor_certificado)?;
    setear_validez_certificado(&mut constructor_certificado, 365)?;

    // firmo el certificado
    constructor_certificado
        .sign(&key, MessageDigest::sha256())
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    let certificado_x509 = constructor_certificado.build();
    // lo convierto a certificate para poder trabajar con udp-dtls
    let certificado = Certificate(certificado_x509);
    let pkcs12 = generar_pkcs12(&certificado, &key, "1234")?;

    Ok((certificado, key, pkcs12))
}

fn generar_clave_pkey() -> Result<PKey<openssl::pkey::Private>, ErrorDTLSProtocolo> {
    // primero genero la clave privada RSA
    // RSA -> es el algoritmo de clave publica usado en certificados x.509
    // 2048 bits es un tamaño comun y seguro
    let rsa = Rsa::generate(2048).map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    // creo la clave privada a partir de la RSA
    let key = PKey::from_rsa(rsa).map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    Ok(key)
}

fn construir_nombre_certificado() -> Result<X509Name, ErrorDTLSProtocolo> {
    let mut constructor_nombre =
        X509NameBuilder::new().map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    constructor_nombre
        .append_entry_by_text("CN", "localhost")
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    let nombre = constructor_nombre.build();
    Ok(nombre)
}

fn setear_validez_certificado(
    certificado: &mut X509Builder,
    dias_validos: u32,
) -> Result<(), ErrorDTLSProtocolo> {
    let fecha_inicio = openssl::asn1::Asn1Time::days_from_now(0)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    certificado
        .set_not_before(&fecha_inicio)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    let fecha_final = openssl::asn1::Asn1Time::days_from_now(dias_validos)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    certificado
        .set_not_after(&fecha_final)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    Ok(())
}

fn crear_constructor_certificado(
    nombre: &X509Name,
    key: &PKey<openssl::pkey::Private>,
    constructor_certificado: &mut X509Builder,
) -> Result<(), ErrorDTLSProtocolo> {
    constructor_certificado
        .set_version(2)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    constructor_certificado
        .set_subject_name(nombre)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    constructor_certificado
        .set_issuer_name(nombre)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;
    constructor_certificado
        .set_pubkey(key)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    Ok(())
}

/// Genera un PKCS#12 que contiene el certificado y la clave privada.
///
/// # Args
/// - `certificado`: Referencia al certificado DTLS.
/// - `clave_privada`: Referencia a la clave privada asociada al certificado.
/// - `contrasena`: Contraseña para proteger el PKCS#12.
///
/// # Returns
/// - `Result<Vec<u8>, ErrorDTLSProtocolo>`: PKCS#12 en formato DER o error si falla la generación.
pub fn generar_pkcs12(
    certificado: &Certificate,
    clave_privada: &PKey<openssl::pkey::Private>,
    contrasena: &str,
) -> Result<Vec<u8>, ErrorDTLSProtocolo> {
    // el certificado en udp_dtls::Certificate envuelve un openssl::x509::X509 internamente
    let x509: &X509 = &certificado.0;

    // pkcs12 es como un contenedor que tiene el certificado y la clave privada (pkey) -> lo necesito para Identity del RTCPeerConnection
    let pkcs12 = Pkcs12::builder()
        // el nombre del contenedor es este x el rtcpeerconnection
        .name("rtc-peer")
        .pkey(clave_privada)
        .cert(x509)
        .build2(contrasena)
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

    pkcs12
        .to_der()
        .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)
}

/// Calcula la huella SHA-256 de un certificado DTLS.
///
/// # Args
/// - `certificado`: Referencia al certificado DTLS.
///
/// # Returns
/// - `Result<Vec<u8>, ErrorDTLSProtocolo>`: Huella SHA-256 o error si falla el cálculo.
pub fn calcular_fingerprint(certificado: &Certificate) -> Result<Vec<u8>, ErrorDTLSProtocolo> {
    // calculo la huella SHA-256 del certificado -> en este nuevo crate ya tengo la función
    let huella = certificado
        .fingerprint(SignatureAlgorithm::Sha256)
        .map_err(|_| ErrorDTLSProtocolo::ErrorCertificadoInvalido)?;

    Ok(huella.bytes.clone())
}

fn convertir_huella_a_formato_sdp(huella: &[u8]) -> String {
    huella
        .iter()
        // 02: dos digitos rellenando con cero si hace falta
        // X: hexadecimal en mayuscula
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<_>>()
        .join(":")
}

/// Obtiene la huella digital del certificado en formato adecuado para SDP.
///
/// # Args
/// - `certificado`: Referencia al certificado DTLS.
///
/// # Returns
/// - `String`: Huella digital en formato SDP (hexadecimal con dos puntos).
pub fn obtener_huella_certificado_para_sdp(
    certificado: &Certificate,
) -> Result<String, ErrorDTLSProtocolo> {
    let huella_sha256 = calcular_fingerprint(certificado)?;
    Ok(convertir_huella_a_formato_sdp(&huella_sha256))
}

/// Construye la línea SDP para el fingerprint del certificado DTLS.
///
/// # Args
/// - `certificado`: Referencia al certificado DTLS.
///
/// # Returns
/// - `String`: Línea SDP con el fingerprint.
pub fn construir_linea_fingerprint_sdp(
    certificado: &Certificate,
) -> Result<String, ErrorDTLSProtocolo> {
    let huella_formateada = obtener_huella_certificado_para_sdp(certificado)?;
    Ok(format!("a=fingerprint:SHA-256 {}", huella_formateada))
}

/// Agrega el atributo 'setup' DTLS a una descripción de sesión SDP.
///
/// # Args
/// - `sdp`: Referencia mutable a la descripción de sesión SDP.
/// - `es_offerer`: Indica si el rol es de offerer (true) o answerer (false).
pub fn agregar_atributo_setup_dtls(sdp: &mut DescripcionDeSesion, es_offerer: bool) {
    // esto va todo en comandos.rs implementado en generar_offer y generar_answer
    let valor = if es_offerer {
        // el offerer usa actpass porque actúa de cliente (según los roles DTLS)
        "a=setup:actpass".to_string()
    } else {
        // el answerer usa active porque actúa de servidor (según los roles DTLS)
        // puede usar también passive, pero active es más común en WebRTC (y más recomendado segun RFC 5763 por un tema de latencia)
        "a=setup:active".to_string()
    };

    for media in sdp.get_medias_mut() {
        // agrego el atributo setup a cada descripcion de media
        media.agregar_atributo(valor.clone());
    }
}

/// Valida que la huella digital recibida en SDP coincida con la del certificado DTLS.
///
/// # Args
/// - `certificado`: Referencia al certificado DTLS.
/// - `huella_recibida_sdp`: Huella digital recibida en SDP.
///
/// # Returns
/// - `Result<(), ErrorDTLSProtocolo>`: Ok si coinciden, error si no coinciden.
pub fn validar_fingerprint_dtls(
    certificado: &Certificate,
    huella_recibida_sdp: &str,
) -> Result<(), ErrorDTLSProtocolo> {
    // calculo la huella del certificado recibido por DTLS
    let huella_calculada_local = obtener_huella_certificado_para_sdp(certificado)?;
    // normalizo la huella recibida del sdp (saco espacios y pongo mayusculas)
    let huella_remota_normalizada = huella_recibida_sdp.trim().to_uppercase();

    // comparo huellas
    if huella_calculada_local == huella_remota_normalizada {
        Ok(())
    } else {
        // el estandar rfc dice que si falla cortamos sesión al toque -> cuando integremos
        Err(ErrorDTLSProtocolo::ErrorFingerprintNoCoincide {
            esperado: huella_calculada_local,
            recibido: huella_remota_normalizada,
        })?
    }
}

/// Parsea la línea de fingerprint DTLS desde SDP.
///
/// # Args
/// - `linea`: Línea SDP con el fingerprint.
///
/// # Returns
/// - `Result<String, ErrorDTLSProtocolo>`: Huella digital en formato adecuado o error si falla el parseo.
pub fn parsear_fingerprint_remoto(linea: &str) -> Result<String, ErrorDTLSProtocolo> {
    // prefijo EXACTO
    let prefijo = "a=fingerprint:";
    if !linea.starts_with(prefijo) {
        Err(ErrorDTLSProtocolo::ErrorFingerprintInvalida)?;
    }

    let huella = &linea[prefijo.len()..];
    let partes: Vec<&str> = huella.split_whitespace().collect();

    // solo usamos el algoritmo de hasheo SHA-256 asi que validamos
    let algoritmo = partes[0].to_uppercase();
    if algoritmo != "SHA-256" {
        Err(ErrorDTLSProtocolo::ErrorFingerprintAlgoritmoNoSoportado)?;
    }

    // debe tener solamente la huella
    if partes.len() != 2 {
        Err(ErrorDTLSProtocolo::ErrorFingerprintInvalida)?;
    }

    let huella_formateada = partes[1].trim().to_uppercase();
    Ok(huella_formateada)
}

/// Parsea el atributo 'setup' DTLS desde SDP.
///
/// # Args
/// - `linea`: Línea SDP con el atributo 'setup'.
///
/// # Returns
/// - `Result<String, ErrorDTLSProtocolo>`: Valor del atributo 'setup' o error si es inválido.
pub fn parsear_setup_dtls(linea: &str) -> Result<String, ErrorDTLSProtocolo> {
    let linea = linea.trim();

    let valor = if let Some(resto) = linea.strip_prefix("a=setup:") {
        resto.trim().to_lowercase()
    } else {
        linea.to_lowercase()
    };

    match valor.as_str() {
        "active" | "passive" | "actpass" => Ok(valor.to_string()),
        _ => Err(ErrorDTLSProtocolo::ErrorSetupInvalido)?,
    }
}

/// Determina el rol DTLS basado en el atributo 'setup' y si es offerer o answerer.
///
/// # Args
/// - `setup`: Valor del atributo 'setup'.
/// - `soy_offer`: Indica si es offerer (true) o answerer (false).
///
/// # Returns
/// - `Result<RolDtls, ErrorDTLSProtocolo>`: Rol DTLS o error si el setup es inválido.
pub fn determinar_rol_dtls_desde_setup(
    setup: &str,
    soy_offer: bool,
) -> Result<RolDtls, ErrorDTLSProtocolo> {
    match setup {
        "active" => {
            if soy_offer {
                Ok(RolDtls::Servidor)
            } else {
                Err(ErrorDTLSProtocolo::ErrorSetupInvalido)
            }
        }
        "passive" => {
            if soy_offer {
                Ok(RolDtls::Cliente)
            } else {
                Err(ErrorDTLSProtocolo::ErrorSetupInvalido)
            }
        }
        "actpass" => {
            if soy_offer {
                // el offerer con actpass es indefinido
                Ok(RolDtls::Servidor)
            } else {
                // soy answerer y leo offerer con actpass -> soy active (cliente) (decisión de diseño, segun rfc se puede ser active o passive)
                Ok(RolDtls::Cliente)
            }
        }
        _ => Err(ErrorDTLSProtocolo::ErrorSetupInvalido)?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generar_certificado_y_huella() {
        let (certificado, _key, _pkcs12) =
            generar_certificado_dtls().expect("Error generando certificado DTLS");
        let huella =
            obtener_huella_certificado_para_sdp(&certificado).expect("Error obteniendo huella");
        assert!(!huella.is_empty(), "La huella no debe estar vacía");
    }

    #[test]
    fn test_convertir_huella_a_formato_sdp() {
        let huella_bytes = vec![0xAB, 0xCD, 0xEF, 0x01, 0x23];
        let huella_formateada = convertir_huella_a_formato_sdp(&huella_bytes);
        assert_eq!(huella_formateada, "AB:CD:EF:01:23");
    }

    #[test]
    fn test_parsear_fingerprint_remoto_valido() {
        let linea = "a=fingerprint:SHA-256 AB:CD:EF:01:23";
        let huella = parsear_fingerprint_remoto(linea).expect("Error parseando fingerprint válido");
        assert_eq!(huella, "AB:CD:EF:01:23");
    }

    #[test]
    fn test_parsear_fingerprint_remoto_invalido() {
        let linea_invalida = "a=fingerprint:SHA-1 AB:CD:EF:01:23";
        assert!(
            parsear_fingerprint_remoto(linea_invalida).is_err(),
            "Debería fallar al parsear fingerprint inválido"
        );
    }

    #[test]
    fn test_parsear_fingerprint_con_espacios_extras() {
        let linea = "a=fingerprint:   SHA-256    AA:BB:CC:DD:EE";
        let huella = parsear_fingerprint_remoto(linea).unwrap();
        assert_eq!(huella, "AA:BB:CC:DD:EE");
    }

    #[test]
    fn test_parsear_fingerprint_con_tabs() {
        let linea = "a=fingerprint:\tSHA-256\tAA:BB:CC";
        let huella = parsear_fingerprint_remoto(linea).unwrap();
        assert_eq!(huella, "AA:BB:CC");
    }

    #[test]
    fn test_parsear_fingerprint_con_minusculas() {
        let linea = "a=fingerprint:SHA-256 aa:bb:cc:dd";
        let huella = parsear_fingerprint_remoto(linea).unwrap();
        assert_eq!(huella, "AA:BB:CC:DD");
    }

    #[test]
    fn test_parsear_fingerprint_sin_prefijo() {
        let linea = "fingerprint:SHA-256 AB:CD";
        assert!(parsear_fingerprint_remoto(linea).is_err());
    }

    #[test]
    fn test_parsear_fingerprint_sin_algoritmo() {
        let linea = "a=fingerprint: AB:CD:EF";
        assert!(parsear_fingerprint_remoto(linea).is_err());
    }

    #[test]
    fn test_parsear_fingerprint_sin_valor() {
        let linea = "a=fingerprint:SHA-256 ";
        assert!(parsear_fingerprint_remoto(linea).is_err());
    }

    #[test]
    fn test_parsear_fingerprint_algoritmo_incorrecto() {
        let linea = "a=fingerprint:SHA-1 AB:CD";
        assert!(parsear_fingerprint_remoto(linea).is_err());
    }

    #[test]
    fn test_parsear_fingerprint_con_token_extra() {
        let linea = "a=fingerprint:SHA-256 AB:CD EF:12"; // demasiado tokens
        assert!(parsear_fingerprint_remoto(linea).is_err());
    }

    #[test]
    fn test_cadena_completa_generar_parsear_validar() {
        let (cert, _key, _pkcs12) = generar_certificado_dtls().unwrap();
        let linea = construir_linea_fingerprint_sdp(&cert).unwrap();
        let huella = parsear_fingerprint_remoto(&linea).unwrap();
        assert!(validar_fingerprint_dtls(&cert, &huella).is_ok());
    }
}
