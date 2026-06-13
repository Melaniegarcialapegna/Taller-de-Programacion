/// Funciones relacionadas con el handshake de SCTP, como iniciar el handshake saliente y avanzar el estado del handshake según los eventos recibidos.
use sctp_proto::{ClientConfig, Event};
use std::net::SocketAddr;

use crate::logger::Logger;
use crate::protocolos::sctp::error_sctp::ErrorSctp;
use crate::protocolos::sctp::estado_sctp::EstadoSctp;

#[derive(Debug)]
/// Estados del handshake de SCTP, que reflejan el progreso de la asociación SCTP: Inactivo (no se ha iniciado), Conectando (handshake en progreso),
/// Establecido (asociación activa) y Fallido (handshake o asociación fallida).
pub enum EstadoHandshakeSctp {
    Inactivo,
    Conectando,
    Establecido,
    Fallido,
}

/// Inicia el handshake SCTP saliente hacia el peer remoto especificado, utilizando la configuración del cliente.
/// Retorna un error si ya existe una asociación activa o si ocurre un error al iniciar la asociación.
pub fn iniciar_handshake_saliente(
    estado: &mut EstadoHandshakeSctp,
    estado_sctp: &mut EstadoSctp,
    remote: SocketAddr,
    client_config: ClientConfig,
) -> Result<(), ErrorSctp> {
    match estado {
        EstadoHandshakeSctp::Inactivo => {
            EstadoSctp::iniciar_asociacion_saliente(estado_sctp, client_config, remote)?;
            *estado = EstadoHandshakeSctp::Conectando;
            Ok(())
        }
        _ => Err(ErrorSctp::ErrorAlIniciarDesdeElEstadoActual(format!(
            "{:?}",
            estado
        ))),
    }
}

/// Avanza el estado del handshake SCTP según los eventos recibidos. Si se recibe un evento Connected, el estado pasa a Establecido.
/// Si se recibe un evento HandshakeFailed o AssociationLost, el estado pasa a Fallido. Otros eventos no afectan el estado del handshake.
pub fn avanzar_handshake_sctp(
    estado: &mut EstadoHandshakeSctp,
    eventos: &Vec<Event>,
    logger: &Logger,
) {
    logger.info("Avanzando handshake SCTP", "Handshake SCTP");

    for evento in eventos {
        match evento {
            Event::Connected => {
                *estado = EstadoHandshakeSctp::Establecido;
                eprintln!("[Handshake SCTP] Asociación establecida exitosamente");
                return;
            }
            Event::HandshakeFailed { .. } => {
                *estado = EstadoHandshakeSctp::Fallido;
                eprintln!("[Handshake SCTP] Handshake fallido");
                return;
            }
            Event::AssociationLost { .. } => {
                *estado = EstadoHandshakeSctp::Fallido;
                eprintln!("[Handshake SCTP] Asociación perdida");
                return;
            }
            _ => {
                eprintln!("[Handshake SCTP] Evento ignorado");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocolos::sctp::rol_sctp::RolConexion;
    use sctp_proto::{ClientConfig, Event};
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };

    fn remote_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000)
    }

    #[test]
    fn test_iniciar_handshake_desde_estado_invalido_falla() {
        // Si el estado no es Inactivo, debe retornar error
        let mut estado = EstadoHandshakeSctp::Conectando;
        let endpoint_config = Arc::new(sctp_proto::EndpointConfig::default());
        let mut estado_sctp =
            EstadoSctp::inicializar_sctp(RolConexion::Inicia, endpoint_config, None).unwrap();
        let client_config = ClientConfig::default();

        let resultado =
            iniciar_handshake_saliente(&mut estado, &mut estado_sctp, remote_addr(), client_config);

        assert!(resultado.is_err());
        assert!(matches!(
            resultado.unwrap_err(),
            ErrorSctp::ErrorAlIniciarDesdeElEstadoActual(_)
        ));
    }

    #[test]
    fn test_avanzar_handshake_evento_connected_establece() {
        let mut estado = EstadoHandshakeSctp::Conectando;
        let eventos = vec![Event::Connected];
        let logger = Logger::dummy_logger();

        avanzar_handshake_sctp(&mut estado, &eventos, &logger);

        assert!(matches!(estado, EstadoHandshakeSctp::Establecido));
    }
}
