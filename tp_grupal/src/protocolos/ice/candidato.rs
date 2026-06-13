//! Módulo que define el struct `Candidato` y las funciones asociadas para validar y crear
//! candidatos ICE (Interactive Connectivity Establishment) utilizados en conexiones P2P.
//!
//! Los candidatos representan posibles rutas de comunicación entre dos peers en una sesión WebRTC,
//! pudiendo ser de tipo **host**, **server reflexive (srflx)** o **relay**.
//!
//! # Tipos de candidato
//! - `host`: dirección obtenida directamente desde una interfaz de red local (por ejemplo, Wi-Fi o Ethernet).
//! - `srflx`: dirección obtenida a través de un servidor STUN (server-reflexive).
//! - `relay`: dirección obtenida mediante un servidor TURN que retransmite el tráfico (relay).
//!
//! # Validaciones realizadas
//! Este módulo realiza validaciones exhaustivas sobre los campos que componen un candidato ICE:
//! - **Tipo de candidato**: debe ser uno de los tipos válidos (`host`, `srflx` o `relay`).
//! - **Dirección IP**: se valida su formato y se rechazan direcciones **no enrutable** como `link-local`,
//!   `multicast`, `broadcast` o `unspecified`, tanto para IPv4 como para IPv6.
//! - **Puerto**: debe ser mayor o igual a `1024`, excluyendo los puertos reservados del sistema.
//!
//! # Funciones principales
//! - `crear_candidato(tipo, ip, puerto) -> Result<Candidato, String>`: crea un nuevo candidato validando los parámetros.
//! - `getter_tipo(&self) -> &str`: devuelve el tipo de candidato (host, srflx o relay).
//! - `getter_ip(&self) -> &str`: devuelve la dirección IP del candidato.
//! - `getter_puerto(&self) -> u16`: devuelve el puerto asociado al candidato.
//!
//! # Contexto de uso
//! Este módulo se utiliza dentro del parser ICE y el proceso de negociación SDP del servidor RoomRTC.
//! Su objetivo es garantizar que los candidatos utilizados en la conexión cumplan con las condiciones
//! necesarias para establecer enlaces válidos y seguros en una red P2P.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

const CANDIDATO_LOCAL: &str = "host";
const CANDIDATO_REFLEXIVE: &str = "srflx";
const CANDIDATO_RELAY: &str = "relay";
const LINK_LOCAL_PRIMER_OCTETO: u8 = 169;
const LINK_LOCAL_SEGUNDO_OCTETO: u8 = 254;
const MULTICAST_INICIO: u8 = 224;
const MULTICAST_FIN: u8 = 239;
const OCTETOS_BROADCAST: [u8; 4] = [255, 255, 255, 255];
const UNSPECIFIED_IPV4: u8 = 0;
const PUERTO_MINIMO: u16 = 1024;

/// Representa un candidato ICE, conteniendo la información mínima necesaria para identificar un endpoint de red válido.
#[derive(Debug, Clone)]
pub struct Candidato {
    tipo: String,
    ip: String,
    puerto: u16,
    pub prioridad: u32,
}

impl Candidato {
    /// Crea un nuevo candidato ICE validando su tipo, dirección IP y puerto.
    ///
    /// # Arguments
    /// * `tipo` - Tipo de candidato (`"host"`, `"srflx"`, o `"relay"`).
    /// * `ip` - Dirección IP en formato string (IPv4 o IPv6).
    /// * `puerto` - Puerto asociado al candidato.
    ///
    /// # Returns
    /// Un `Result` que contiene:
    /// - `Ok(Candidato)` si todos los campos son válidos.
    /// - `Err(String)` si alguna validación falla, devolviendo el mensaje de error.
    ///
    /// # Ejemplo
    /// ```ignore
    /// let candidato = Candidato::crear_candidato(
    ///     "relay".to_string(),
    ///     "2001:0db8::1".to_string(),
    ///     5000
    /// );
    /// ```
    pub fn crear_candidato(tipo: String, ip: String, puerto: u16) -> Result<Self, String> {
        // validacion tipo de candidato
        Self::validar_tipo_candidato(&tipo)?;

        // validacion ip
        Self::validar_ip(&ip)?;

        // validacion puerto, no ponemos la de < 65535 porque viene incluida en el tipo de dato (u16 va hasta exactamente ese número)
        Self::validar_puerto(puerto)?;

        let prioridad = Self::calcular_prioridad(&tipo);

        Ok(Candidato {
            tipo,
            ip,
            puerto,
            prioridad,
        })
    }

    /// Devuelve el tipo de candidato (host, srflx o relay).
    pub fn getter_tipo(&self) -> &str {
        &self.tipo
    }

    /// Devuelve la dirección IP del candidato.
    pub fn getter_ip(&self) -> &str {
        &self.ip
    }

    /// Devuelve el puerto asociado al candidato.
    pub fn getter_puerto(&self) -> u16 {
        self.puerto
    }

    pub fn getter_prioridad(&self) -> u32 {
        self.prioridad
    }

    /// Devuelve la prioridad de un candidato determinada según la fórmula que propone RFC 8445.
    pub fn calcular_prioridad(tipo: &str) -> u32 {
        // mas alta es mejor (P)
        let preferencia_por_tipo = match tipo {
            "host" => 126,
            "srflx" => 100,
            "relay" => 0,
            _ => 0,
        };

        // 1 para RTP, 2 para RTCP
        let component_id = 1;

        // preferencia Local, usamos el valor máximo (2^16 - 1) para simplificar (L)
        let local_preference = 65535;

        // P = (2^24 * T) + (2^8 * C) + (L)
        (preferencia_por_tipo as u32) * (1 << 24)
            + (component_id as u32) * (1 << 8)
            + local_preference as u32
    }

    /// Valida que el tipo de candidato sea uno de los permitidos.
    fn validar_tipo_candidato(tipo: &str) -> Result<(), String> {
        match tipo.to_lowercase().as_str() {
            //localhost, STUN, TURN
            CANDIDATO_LOCAL | CANDIDATO_REFLEXIVE | CANDIDATO_RELAY => {}
            _ => {
                return Err(format!(
                    "El tipo de candidato ingresado es incorrecto: {}",
                    tipo
                ));
            }
        }

        Ok(())
    }

    /// Valida que la dirección IP tenga un formato correcto y sea adecuada para ICE.
    /// Rechaza direcciones no enrutable como link-local, multicast, broadcast o unspecified
    fn validar_ip(ip: &str) -> Result<(), String> {
        let ip_procesada = IpAddr::from_str(ip).map_err(|_| {
            format!(
                "La dirección IP propuesta no tiene un formato válido: {}",
                ip
            )
        })?;

        match ip_procesada {
            IpAddr::V4(addr) => Self::validar_ipv4(addr, ip)?,
            IpAddr::V6(addr) => Self::validar_ipv6(addr, ip)?,
        }

        Ok(())
    }

    /// Validaciones específicas para direcciones IPv4.
    fn validar_ipv4(addr: Ipv4Addr, ip: &str) -> Result<(), String> {
        let octetos = addr.octets();

        // la dejo comentada porque en pruebas locales en misma máquina ESTÁ BIEN aceptar loopback, vemos después que dirección toma lo que hay que implementar
        // if octetos[0] == 127 {
        //     return Err(format!("La IP {} es loopback, no valido para ICE", ip))
        // }

        //link-local -> no enrutable
        Self::rechazar_link_local_ipv4(&octetos, ip)?;

        //multicast -> no p2p
        Self::rechazar_multicast_ipv4(&octetos, ip)?;

        //broadcast -> transmisión de un paquete a varios usuarios
        Self::rechazar_broadcast_ipv4(&octetos, ip)?;

        // unspecified -> no representa nada, ninguna interfaz
        Self::rechazar_unspecified_ipv4(&octetos, ip)?;

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo link-local.
    fn rechazar_link_local_ipv4(octetos: &[u8; 4], ip: &str) -> Result<(), String> {
        if octetos[0] == LINK_LOCAL_PRIMER_OCTETO && octetos[1] == LINK_LOCAL_SEGUNDO_OCTETO {
            return Err(format!("La IP {} es link-local, no válida para ICE", ip));
        }

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo multicast.
    fn rechazar_multicast_ipv4(octetos: &[u8; 4], ip: &str) -> Result<(), String> {
        if (MULTICAST_INICIO..=MULTICAST_FIN).contains(&octetos[0]) {
            return Err(format!("La IP {} es multicast, no válida para ICE", ip));
        }

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo broadcast.
    fn rechazar_broadcast_ipv4(octetos: &[u8; 4], ip: &str) -> Result<(), String> {
        if *octetos == OCTETOS_BROADCAST {
            return Err(format!("La IP {} es broadcast, no válida para ICE", ip));
        }

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo unspecified.
    fn rechazar_unspecified_ipv4(octetos: &[u8; 4], ip: &str) -> Result<(), String> {
        if octetos[0] == UNSPECIFIED_IPV4 {
            return Err(format!("La IP {} no representa ninguna interfaz", ip));
        }

        Ok(())
    }

    /// Validaciones específicas para direcciones IPv6.
    fn validar_ipv6(addr: Ipv6Addr, ip: &str) -> Result<(), String> {
        // la dejo comentada porque en pruebas locales en misma máquina ESTÁ BIEN aceptar loopback, vemos después que dirección toma lo que hay que implementar
        // if addr.is_loopback() {
        //     return Err(format!("La dirección {} IPV6 es loopback, no valido para ICE", ip))
        // }

        //multicast -> no p2p
        Self::rechazar_multicast_ipv6(&addr, ip)?;

        // unspecified -> no representa nada, ninguna interfaz
        Self::rechazar_unspecified_ipv6(&addr, ip)?;

        //link-local -> no enrutable
        Self::rechazar_link_local_ipv6(&addr, ip)?;

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo multicast.
    fn rechazar_multicast_ipv6(addr: &Ipv6Addr, ip: &str) -> Result<(), String> {
        if addr.is_multicast() {
            return Err(format!(
                "La IP {} es multicast IPV6, no válida para ICE",
                ip
            ));
        }

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo unspecified.
    fn rechazar_unspecified_ipv6(addr: &Ipv6Addr, ip: &str) -> Result<(), String> {
        if addr.is_unspecified() {
            return Err(format!("La IP {} no representa ninguna interfaz", ip));
        }

        Ok(())
    }

    /// Valida que la dirección IP no sea de tipo link-local.
    fn rechazar_link_local_ipv6(addr: &Ipv6Addr, ip: &str) -> Result<(), String> {
        if addr.is_unicast_link_local() {
            return Err(format!(
                "La IP {} es link-local IPV6, no válida para ICE",
                ip
            ));
        }

        Ok(())
    }

    /// Valida que el puerto sea mayor o igual a 1024.
    fn validar_puerto(puerto: u16) -> Result<(), String> {
        if puerto < PUERTO_MINIMO {
            return Err(format!(
                "El puerto propuesto no es válido para crear el candidato: {}",
                puerto
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_candidato_valido_ipv4() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "192.168.1.1".to_string(), 12345);
        assert!(candidato.is_ok());
        let candidato_test = candidato.unwrap();
        assert_eq!(candidato_test.getter_tipo(), "host");
        assert_eq!(candidato_test.getter_ip(), "192.168.1.1");
        assert_eq!(candidato_test.getter_puerto(), 12345);
    }

    #[test]
    fn test_crear_candidato_valido_ipv6() {
        let candidato =
            Candidato::crear_candidato("srflx".to_string(), "2001:db8::1".to_string(), 23456);
        assert!(candidato.is_ok());
        let candidato_test = candidato.unwrap();
        assert_eq!(candidato_test.getter_tipo(), "srflx");
        assert_eq!(candidato_test.getter_ip(), "2001:db8::1");
        assert_eq!(candidato_test.getter_puerto(), 23456);
    }

    #[test]
    fn test_tipo_candidato_invalido() {
        let candidato =
            Candidato::crear_candidato("invalid".to_string(), "192.168.1.1".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("El tipo de candidato ingresado es incorrecto")
        );
    }

    #[test]
    fn test_ip_invalida() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "999.999.999.999".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("La dirección IP propuesta no tiene un formato válido")
        );
    }

    #[test]
    fn test_puerto_invalido() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "192.168.1.1".to_string(), 80);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("El puerto propuesto no es válido para crear el candidato")
        );
    }

    #[test]
    fn test_ipv4_link_local() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "169.254.10.20".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("es link-local, no válida para ICE")
        );
    }

    #[test]
    fn test_ipv4_multicast() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "224.0.0.1".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("es multicast, no válida para ICE")
        );
    }

    #[test]
    fn test_ipv4_broadcast() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "255.255.255.255".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("es broadcast, no válida para ICE")
        );
    }

    #[test]
    fn test_ipv4_unspecified() {
        let candidato =
            Candidato::crear_candidato("host".to_string(), "0.0.0.0".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("no representa ninguna interfaz")
        );
    }

    #[test]
    fn test_ipv6_multicast() {
        let candidato =
            Candidato::crear_candidato("relay".to_string(), "ff02::1".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("es multicast IPV6, no válida para ICE")
        );
    }

    #[test]
    fn test_ipv6_unspecified() {
        let candidato = Candidato::crear_candidato("relay".to_string(), "::".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("no representa ninguna interfaz")
        );
    }

    #[test]
    fn test_ipv6_link_local() {
        let candidato =
            Candidato::crear_candidato("relay".to_string(), "fe80::1".to_string(), 12345);
        assert!(candidato.is_err());
        assert!(
            candidato
                .err()
                .unwrap()
                .contains("es link-local IPV6, no válida para ICE")
        );
    }
}
