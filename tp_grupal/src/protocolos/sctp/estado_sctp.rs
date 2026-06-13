/// EstadoSctp representa el estado de la capa SCTP para una peer connection, incluyendo la asociación activa y su handle.
use crate::protocolos::sctp::error_sctp::ErrorSctp;
use crate::protocolos::sctp::rol_sctp::RolConexion;
use sctp_proto::{
    Association, AssociationHandle, ClientConfig, Endpoint, EndpointConfig, Error, ServerConfig,
};
use std::net::SocketAddr;
use std::sync::Arc;

pub struct EstadoSctp {
    pub endpoint: Endpoint,
    pub asociacion: Option<Association>,
    pub handle: Option<AssociationHandle>,
}

/// Funciones relacionadas con el estado de SCTP, como inicialización, inicio de asociación saliente y finalización de llamada.
impl EstadoSctp {
    /// Inicializa el estado de SCTP con la configuración del endpoint y, opcionalmente, la configuración del servidor según el rol de conexión.
    pub fn inicializar_sctp(
        rol: RolConexion,
        endpoint_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
    ) -> Result<EstadoSctp, Error> {
        let server_config_final = match rol {
            RolConexion::Acepta | RolConexion::Dual => server_config,
            RolConexion::Inicia => None,
        };

        let endpoint = Endpoint::new(endpoint_config, server_config_final);

        Ok(EstadoSctp {
            endpoint,
            asociacion: None,
            handle: None,
        })
    }

    /// Inicia una asociación SCTP saliente hacia el peer remoto especificado, utilizando la configuración del cliente.
    /// Retorna un error si ya existe una asociación activa o si ocurre un error al iniciar la asociación.
    pub fn iniciar_asociacion_saliente(
        estado: &mut EstadoSctp,
        client_config: ClientConfig,
        remote: SocketAddr,
    ) -> Result<(), ErrorSctp> {
        if estado.asociacion.is_some() {
            return Err(ErrorSctp::YaExisteAsociacionActiva);
        }

        let conexion_endpoint = estado.endpoint.connect(client_config, remote);
        match conexion_endpoint {
            Ok((handle, asociacion)) => {
                estado.asociacion = Some(asociacion);
                estado.handle = Some(handle);
            }
            Err(error) => {
                return Err(ErrorSctp::ErrorAlIniciarAsociacion(error.to_string()));
            }
        }

        Ok(())
    }

    /// Finaliza la llamada SCTP cerrando la asociación activa (si existe) y limpiando el handle.
    pub fn finalizar_llamada_sctp(estado: &mut EstadoSctp) {
        if let Some(mut asoc) = estado.asociacion.take() {
            let _ = asoc.shutdown();
        }
        estado.handle = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inicializar_sctp_inicia() {
        let endpoint_config = Arc::new(EndpointConfig::default());
        let result = EstadoSctp::inicializar_sctp(RolConexion::Inicia, endpoint_config, None);
        assert!(result.is_ok());
        let estado = result.unwrap();
        assert!(estado.asociacion.is_none());
        assert!(estado.handle.is_none());
    }

    #[test]
    fn test_inicializar_sctp_acepta() {
        let endpoint_config = Arc::new(EndpointConfig::default());
        let server_config = Some(Arc::new(ServerConfig::default()));
        let result =
            EstadoSctp::inicializar_sctp(RolConexion::Acepta, endpoint_config, server_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_finalizar_llamada_sctp_sin_asociacion() {
        let endpoint_config = Arc::new(EndpointConfig::default());
        let mut estado =
            EstadoSctp::inicializar_sctp(RolConexion::Inicia, endpoint_config, None).unwrap();
        EstadoSctp::finalizar_llamada_sctp(&mut estado);
        assert!(estado.handle.is_none());
    }
}
