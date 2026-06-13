/// Este módulo se encarga de construir las configuraciones necesarias para inicializar el estado del protocolo SCTP, incluyendo la configuración de
/// transporte, endpoint y servidor. Proporciona funciones para crear estas configuraciones de manera modular y reutilizable, facilitando la inicialización
/// del estado SCTP según el rol de conexión (aceptar, iniciar o dual).
use crate::protocolos::sctp::error_sctp::ErrorSctp;
use crate::protocolos::sctp::estado_sctp::EstadoSctp;
use crate::protocolos::sctp::rol_sctp::RolConexion;
use sctp_proto::{ClientConfig, EndpointConfig, ServerConfig, TransportConfig};
use std::sync::Arc;

/// Construye el estado inicial del protocolo SCTP basado en el rol de conexión especificado. Dependiendo del rol,
/// se configuran las opciones necesarias para el endpoint y el servidor, y se inicializa el estado SCTP con estas configuraciones.
pub fn construir_estado_sctp(rol: RolConexion) -> Result<EstadoSctp, ErrorSctp> {
    let transport_config = construir_transport_config();
    let endpoint_config = construir_endpoint_config();
    let server_config = match rol {
        RolConexion::Acepta | RolConexion::Dual => {
            Some(construir_server_config(transport_config.clone()))
        }
        RolConexion::Inicia => None,
    };

    EstadoSctp::inicializar_sctp(rol, endpoint_config, server_config)
        .map_err(|e| ErrorSctp::ErrorAlIniciarAsociacion(e.to_string()))
}

/// Construye la configuración de transporte para SCTP. Esta configuración puede incluir opciones como el número máximo de flujos entrantes, el tamaño de
/// la ventana, etc.
pub fn construir_transport_config() -> Arc<TransportConfig> {
    let config = TransportConfig::default();
    // meti el default pero podemos achicar campos, x ej el with_max_inbound_streams(1024) o algo asi pero igual el default nos va bien
    Arc::new(config) // arc para compartir entre endpoint y server/client configs
}

/// Construye la configuración del endpoint para SCTP. Esta configuración puede incluir opciones como el número de puerto local, la dirección IP, etc.
pub fn construir_endpoint_config() -> Arc<EndpointConfig> {
    let config = EndpointConfig::default();
    // lo mismo que el transport_config
    Arc::new(config)
}

/// Construye la configuración del servidor para SCTP. Esta configuración se utiliza para aceptar conexiones entrantes y puede incluir opciones específicas
pub fn construir_server_config(transport_config: Arc<TransportConfig>) -> Arc<ServerConfig> {
    let mut config = ServerConfig::default();
    config.transport = transport_config; // asigno el transport_config al server_config
    Arc::new(config)
}

/// Construye la configuración del cliente para SCTP. Esta configuración se utiliza para iniciar conexiones salientes y puede incluir opciones específicas
pub fn construir_client_config(transport_config: Arc<TransportConfig>) -> ClientConfig {
    ClientConfig {
        transport: transport_config,
    }
}
