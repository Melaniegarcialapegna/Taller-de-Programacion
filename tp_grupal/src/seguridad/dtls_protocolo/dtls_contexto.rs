//! Módulo 'dtls_contexto.rs'
//!
//! Este módulo define la estructura `DtlsContexto`, que encapsula el contexto necesario para manejar
//! una conexión DTLS (Datagram Transport Layer Security). Proporciona funcionalidades para almacenar
//! certificados, claves privadas, roles y derivar claves SRTP (Secure Real-time Transport Protocol)
//! a partir del contexto DTLS.
//!
//! Una vez establecido el contexto DTLS, se pueden exportar las claves SRTP necesarias para cifrar
//! y descifrar los flujos de medios seguros.
//!
//! # Structs y enums
//!
//! - `DtlsContexto`: Estructura principal que almacena el contexto DTLS.
//! - `RolDtls`: Enum que define los roles posibles en una conexión DTLS (Cliente, Servidor, Indefinido).
//!
//! # Funcionalidades principales
//!
//! - Almacenamiento y recuperación de certificados locales y remotos.
//! - Almacenamiento y recuperación de claves privadas locales.
//! - Establecimiento y obtención del rol DTLS.
//! - Exportación de claves SRTP derivadas del contexto DTLS.

use crate::seguridad::dtls_protocolo::errores::ErrorDTLSProtocolo;
use crate::seguridad::srtp::claves_srtp::{ClavesSRTP, ClavesSRTPBrutas};
use openssl::pkey::PKey;
use openssl::ssl::SslRef; // SslRef de OpenSSL para referencias SSL
use std::fmt::Debug;
use udp_dtls::Certificate;

// Valores estándar para SRTP_AES128_CM_HMAC_SHA1_80
const SRTP_MASTER_KEY_LEN: usize = 16;
const SRTP_MASTER_SALT_LEN: usize = 14;
const SRTP_EXPORT_LEN: usize = 2 * (SRTP_MASTER_KEY_LEN + SRTP_MASTER_SALT_LEN);
// Label definido en RFC 5764 / WebRTC
const DTLS_SRTP_EXPORTER_LABEL: &str = "EXTRACTOR-dtls_srtp";

type ClavesSRTPResultado = (
    Vec<u8>, // clave_tx
    Vec<u8>, // clave_rx
    Vec<u8>, // salt_tx
    Vec<u8>, // salt_rx
);

/// Enum que define los roles posibles en una conexión DTLS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolDtls {
    Cliente,    // el que inicia handshake (active)
    Servidor,   // el que responde (passive)
    Indefinido, // para el actpass
}

/// Struct que encapsula el contexto DTLS.
#[derive(Clone)]
pub struct DtlsContexto {
    certificado_local: Option<Certificate>,
    certificado_remoto: Option<Certificate>,
    huella_remota: Option<String>,
    rol: RolDtls,
    clave_privada_local: Option<PKey<openssl::pkey::Private>>,
    pkcs12_local: Option<Vec<u8>>, // Nuevo campo para almacenar el PKCS#12 -> Pkcs es un contenedor que incluye certificado y clave privada
    claves_srtp: Option<ClavesSRTP>,
}

/// Implementación del trait Default para DtlsContexto.
impl Default for DtlsContexto {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementación de métodos para DtlsContexto.
impl DtlsContexto {
    /// Crea un nuevo contexto DTLS vacío.
    pub fn new() -> Self {
        Self {
            certificado_local: None,
            certificado_remoto: None,
            huella_remota: None,
            rol: RolDtls::Indefinido,
            clave_privada_local: None,
            pkcs12_local: None,
            claves_srtp: None,
        }
    }

    /// Guarda el certificado local.
    pub fn guardar_certificado_local(&mut self, certificado: Certificate) {
        self.certificado_local = Some(certificado);
    }

    /// Guarda la huella digital del certificado remoto.
    pub fn guardar_huella_remota(&mut self, fp: String) {
        self.huella_remota = Some(fp);
    }

    /// Establece el rol DTLS.
    pub fn establecer_rol(&mut self, rol: RolDtls) {
        self.rol = rol;
    }

    /// Obtiene el rol DTLS.
    pub fn obtener_rol(&self) -> RolDtls {
        self.rol
    }

    /// Obtiene el certificado local.
    pub fn obtener_certificado_local(&self) -> Option<&Certificate> {
        self.certificado_local.as_ref()
    }

    /// Obtiene la huella digital del certificado remoto.
    pub fn obtener_huella_remota(&self) -> Option<&String> {
        self.huella_remota.as_ref()
    }

    /// Guarda la clave privada local.
    pub fn guardar_clave_privada_local(&mut self, key: PKey<openssl::pkey::Private>) {
        self.clave_privada_local = Some(key);
    }

    /// Obtiene la clave privada local.
    pub fn obtener_clave_privada_local(&self) -> Option<&PKey<openssl::pkey::Private>> {
        self.clave_privada_local.as_ref()
    }

    /// Obtiene el PKCS#12 local.
    pub fn obtener_pkcs12_local(&self) -> Option<&Vec<u8>> {
        self.pkcs12_local.as_ref()
    }

    /// Establece el PKCS#12 local.
    pub fn setear_pkcs12_local(&mut self, pkcs12: Vec<u8>) {
        self.pkcs12_local = Some(pkcs12);
    }

    /// Establece el certificado remoto.
    pub fn establecer_certificado_remoto(&mut self, certificado: Certificate) {
        self.certificado_remoto = Some(certificado);
    }

    /// Obtiene el certificado remoto.
    pub fn obtener_certificado_remoto(&self) -> Option<&Certificate> {
        self.certificado_remoto.as_ref()
    }

    /// Obtiene las claves SRTP.
    pub fn obtener_claves_srtp(&self) -> Option<&ClavesSRTP> {
        self.claves_srtp.as_ref()
    }

    /// Exporta las claves SRTP derivadas del contexto DTLS.
    ///
    /// # Args
    /// - `ssl`: Referencia al contexto SSL.
    ///
    /// # Returns
    /// - `Result<&ClavesSRTP, ErrorDTLSProtocolo>`: Claves SRTP o error si no están disponibles.
    pub fn exportar_claves_srtp(
        &mut self,
        ssl: &SslRef,
    ) -> Result<&ClavesSRTP, ErrorDTLSProtocolo> {
        if self.claves_srtp.is_none() {
            let brutas = Self::generar_claves_srtp_derivadas(ssl)
                .map_err(|_| ErrorDTLSProtocolo::ClavesSRTPNoDisponibles)?;

            let (clave_tx, clave_rx, salt_tx, salt_rx) =
                Self::ordenar_claves_por_rol(self.rol, &brutas)?;

            self.claves_srtp = Some(ClavesSRTP {
                clave_tx,
                clave_rx,
                salt_tx,
                salt_rx,
            });
        }

        self.claves_srtp
            .as_ref()
            .ok_or(ErrorDTLSProtocolo::ClavesSRTPNoDisponibles)
    }

    fn ordenar_claves_por_rol(
        rol: RolDtls,
        brutas: &ClavesSRTPBrutas,
    ) -> Result<ClavesSRTPResultado, ErrorDTLSProtocolo> {
        match rol {
            RolDtls::Cliente => Ok((
                brutas.client_key.to_vec(),
                brutas.server_key.to_vec(),
                brutas.client_salt.to_vec(),
                brutas.server_salt.to_vec(),
            )),
            RolDtls::Servidor => Ok((
                brutas.server_key.to_vec(),
                brutas.client_key.to_vec(),
                brutas.server_salt.to_vec(),
                brutas.client_salt.to_vec(),
            )),
            RolDtls::Indefinido => Err(ErrorDTLSProtocolo::ErrorRolNoEstablecido),
        }
    }

    fn generar_claves_srtp_derivadas(ssl: &SslRef) -> Result<ClavesSRTPBrutas, ErrorDTLSProtocolo> {
        let keying_material = Self::exportar_keying_material(ssl)?;
        let mut offset = 0;

        let mut clave_escritura_cliente = [0u8; SRTP_MASTER_KEY_LEN];
        clave_escritura_cliente
            .copy_from_slice(&keying_material[offset..offset + SRTP_MASTER_KEY_LEN]);
        offset += SRTP_MASTER_KEY_LEN;
        let mut clave_escritura_remota = [0u8; SRTP_MASTER_KEY_LEN];
        clave_escritura_remota
            .copy_from_slice(&keying_material[offset..offset + SRTP_MASTER_KEY_LEN]);
        offset += SRTP_MASTER_KEY_LEN;
        let mut salt_escritura_cliente = [0u8; SRTP_MASTER_SALT_LEN];
        salt_escritura_cliente
            .copy_from_slice(&keying_material[offset..offset + SRTP_MASTER_SALT_LEN]);
        offset += SRTP_MASTER_SALT_LEN;
        let mut salt_escritura_remota = [0u8; SRTP_MASTER_SALT_LEN];
        salt_escritura_remota
            .copy_from_slice(&keying_material[offset..offset + SRTP_MASTER_SALT_LEN]);

        Ok(ClavesSRTPBrutas {
            client_key: clave_escritura_cliente,
            server_key: clave_escritura_remota,
            client_salt: salt_escritura_cliente,
            server_salt: salt_escritura_remota,
        })
    }

    fn exportar_keying_material(ssl: &SslRef) -> Result<Vec<u8>, ErrorDTLSProtocolo> {
        let total_len = SRTP_EXPORT_LEN;
        let mut keying_material = vec![0u8; total_len];

        ssl.export_keying_material(&mut keying_material, DTLS_SRTP_EXPORTER_LABEL, None)
            .map_err(|_| ErrorDTLSProtocolo::ClavesSRTPNoDisponibles)?;

        Ok(keying_material)
    }
}
