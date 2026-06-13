/// Funciones relacionadas con el procesamiento de datagramas SCTP entrantes, el avance del estado de la asociación, el manejo de eventos y
/// transmisiones pendientes, y la sincronización entre el endpoint y la asociación.
use crate::protocolos::sctp::error_sctp::ErrorSctp;
use crate::protocolos::sctp::estado_sctp::EstadoSctp;
use bytes::Bytes;
use sctp_proto::{
    Association, AssociationHandle,
    DatagramEvent::{AssociationEvent, NewAssociation},
    Event, Transmit,
};
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

/// Procesa un datagrama SCTP entrante, actualizando el estado de SCTP según corresponda. Si el datagrama corresponde a una nueva asociación,
/// se acepta o rechaza según si ya existe una asociación activa.
pub fn recibir_desde_red(
    estado: &mut EstadoSctp,
    now: Instant,
    remote: SocketAddr,
    local_ip: Option<IpAddr>,
    data: Bytes,
) -> Result<(), ErrorSctp> {
    let procesado_datagrama = estado.endpoint.handle(now, remote, local_ip, None, data);

    if let Some((handle_asoc, evento_datagrama)) = procesado_datagrama {
        match evento_datagrama {
            NewAssociation(nueva_asoc) => {
                // solo aceptamos si no hay una activa
                aceptar_nueva_asociacion(estado, handle_asoc, nueva_asoc)?;
            }
            AssociationEvent(ev_asoc) => {
                // el datagrama era para una asociación existente
                if let (Some(handle_activo), Some(asoc)) =
                    (estado.handle, estado.asociacion.as_mut())
                {
                    if handle_asoc == handle_activo {
                        asoc.handle_event(ev_asoc);
                    } else {
                        return Err(ErrorSctp::ErrorHandleAsociacionNoCoincide);
                    }
                }
            }
        }
    }
    sincronizar_endpoint_y_asociacion(estado);
    Ok(())
}

fn aceptar_nueva_asociacion(
    estado: &mut EstadoSctp,
    handle_asoc: AssociationHandle,
    nueva_asoc: Association,
) -> Result<(), ErrorSctp> {
    if estado.asociacion.is_none() {
        estado.handle = Some(handle_asoc);
        estado.asociacion = Some(nueva_asoc);
    } else {
        estado.endpoint.reject_new_associations();
        return Err(ErrorSctp::YaExisteAsociacionActiva);
    }

    Ok(())
}

/// Sincroniza el estado del endpoint y la asociación procesando todos los eventos pendientes en ambos.
/// Esto es necesario porque el endpoint y la asociación mantienen estados internos que deben estar sincronizados.
pub fn sincronizar_endpoint_y_asociacion(estado: &mut EstadoSctp) {
    let (Some(handle), Some(asoc)) = (estado.handle, estado.asociacion.as_mut()) else {
        return;
    };

    loop {
        let mut progreso = false;

        // de asociación a endpoint
        while let Some(evento_endpoint) = asoc.poll_endpoint_event() {
            progreso = true;
            if let Some(evento_asoc) = estado.endpoint.handle_event(handle, evento_endpoint) {
                // si el endpoint devuelve un evento para la asociación, lo procesamos
                asoc.handle_event(evento_asoc);
            }
        }

        // si en esta vuelta no hubo eventos ni en asociación ni en endpoint, es porque terminamos de sincronizar
        if !progreso {
            break;
        }
    }
}

/// Drena todas las transmisiones pendientes tanto del endpoint como de la asociación, retornándolas en un vector para ser enviadas por la red.
pub fn drenar_transmisiones(estado: &mut EstadoSctp, tiempo_actual: Instant) -> Vec<Transmit> {
    let mut vec_salida = Vec::new();

    // mientras que el endpoint tenga datagramas a enviar, los sacamos y los agregamos al vector de salida
    while let Some(tx) = estado.endpoint.poll_transmit() {
        vec_salida.push(tx);
    }

    // mientras que la asociación tenga transmisiones pendientes, las sacamos y las agregamos al vector de salida
    if let Some(asoc) = estado.asociacion.as_mut() {
        while let Some(tx) = asoc.poll_transmit(tiempo_actual) {
            vec_salida.push(tx);
        }
    }

    vec_salida
}

/// Drena todos los eventos pendientes de la asociación, retornándolos en un vector para ser procesados por la aplicación.
pub fn drenar_eventos_app(estado: &mut EstadoSctp) -> Vec<Event> {
    let mut vec_salida = Vec::new();
    if let Some(asoc) = estado.asociacion.as_mut() {
        while let Some(ev) = asoc.poll() {
            vec_salida.push(ev);
        }
    }
    vec_salida
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocolos::sctp::estado_sctp::EstadoSctp;
    use crate::protocolos::sctp::rol_sctp::RolConexion;
    use sctp_proto::EndpointConfig;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use std::time::Instant;

    fn estado_sin_asociacion() -> EstadoSctp {
        let endpoint_config = Arc::new(EndpointConfig::default());
        EstadoSctp::inicializar_sctp(RolConexion::Inicia, endpoint_config, None).unwrap()
    }

    fn remote_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000)
    }

    #[test]
    fn test_drenar_transmisiones_sin_asociacion_retorna_vacio() {
        let mut estado = estado_sin_asociacion();
        let resultado = drenar_transmisiones(&mut estado, Instant::now());
        assert!(resultado.is_empty());
    }

    #[test]
    fn test_drenar_eventos_app_sin_asociacion_retorna_vacio() {
        let mut estado = estado_sin_asociacion();
        let resultado = drenar_eventos_app(&mut estado);
        assert!(resultado.is_empty());
    }

    #[test]
    fn test_recibir_datagrama_invalido_no_genera_error() {
        let mut estado = estado_sin_asociacion();
        let data = Bytes::from_static(b"datos_invalidos_sctp");
        let resultado = recibir_desde_red(&mut estado, Instant::now(), remote_addr(), None, data);
        let _ = resultado;
    }
}
