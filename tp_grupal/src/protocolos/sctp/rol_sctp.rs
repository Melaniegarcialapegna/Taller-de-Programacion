/// Rol de conexión SCTP
/// Define el rol que un peer asume en la conexión SCTP, lo cual afecta cómo se comporta en términos de iniciar o aceptar asociaciones.
/// Este rol se determina a partir del rol DTLS,
use crate::seguridad::dtls_protocolo::dtls_contexto::RolDtls;

#[derive(Clone, Copy, Debug)]
pub enum RolConexion {
    Inicia, // “cliente” en el sentido: yo llamo connect()
    Acepta, // “server” en el sentido: espero INIT entrante y creo asociaciones
    Dual, // opcional: acepto y también puedo iniciar salientes (necesario por RolDtls::Indefinido)
}

// Tenemos en cuenta la relación 1:1 entre roles DTLS y roles de conexión SCTP
impl From<RolDtls> for RolConexion {
    fn from(rol: RolDtls) -> Self {
        match rol {
            RolDtls::Cliente => RolConexion::Inicia,
            RolDtls::Servidor => RolConexion::Acepta,
            RolDtls::Indefinido => RolConexion::Dual,
        }
    }
}
