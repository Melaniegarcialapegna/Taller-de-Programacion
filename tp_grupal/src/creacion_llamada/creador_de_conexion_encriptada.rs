use std::{
    net::SocketAddr,
    sync::mpsc::{self, Receiver},
};

use crate::{
    config_room_rtc::ConfigRoomRTC,
    creacion_llamada::{ConexionP2P, CreadorDeConexionP2P, ErrorCreadorDeConexion},
    logger::Logger,
    protocolos::{
        sctp::{
            config_sctp::{
                construir_client_config, construir_estado_sctp, construir_transport_config,
            },
            dcep::dcep_handshake::EstadoDcep,
            handshake_sctp::EstadoHandshakeSctp,
            rol_sctp::RolConexion,
        },
        sdp::{
            descripcion_de_sesion::DescripcionDeSesion,
            traits::{ParseableSdp, SerializableSdp},
        },
    },
    rtc::rtc_peer_connection::RTCPeerConnection,
    seguridad::dtls_protocolo::{
        dtls_contexto::RolDtls,
        dtls_utils::{
            agregar_atributo_setup_dtls, construir_linea_fingerprint_sdp,
            determinar_rol_dtls_desde_setup, generar_certificado_dtls, parsear_fingerprint_remoto,
            parsear_setup_dtls,
        },
    },
};

/// [CreadorDeConexionEncriptada] es un [CreadorDeConexionP2P] que permite generar offers y answers, y luego crear la conexion.
/// En particular, la conexión creada por este objeto estara encriptada mediante DTLS.
#[allow(dead_code)]
pub struct CreadorDeConexionEncriptada {
    rtc_peer: RTCPeerConnection,
    rx_termino_ice: Receiver<String>,
    par_exitoso: Option<(String, u16, u16)>,
    config: ConfigRoomRTC,
    logger: Logger,
    es_primera_llamada: bool,
}

impl CreadorDeConexionEncriptada {
    /// Crea un nuevo [CreadorDeConexionEncriptada] con la configuración dada.
    /// Este método también inicializa el [RTCPeerConnection] y el canal para recibir mensajes de cosas de ICE.
    pub fn iniciar_creador(
        config: ConfigRoomRTC,
        logger: Logger,
    ) -> Result<CreadorDeConexionEncriptada, ErrorCreadorDeConexion> {
        let (tx_termino_ice, rx_termino_ice) = mpsc::channel();
        let rtc_peer_connection =
            RTCPeerConnection::new(config.clone(), logger.clone(), tx_termino_ice)
                .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        Ok(CreadorDeConexionEncriptada {
            rtc_peer: rtc_peer_connection,
            rx_termino_ice,
            par_exitoso: None,
            config,
            logger,
            es_primera_llamada: true,
        })
    }
}

impl CreadorDeConexionP2P for CreadorDeConexionEncriptada {
    fn generar_offer(&mut self) -> Result<String, ErrorCreadorDeConexion> {
        if !self.es_primera_llamada {
            let socket_rtp = self
                .rtc_peer
                .get_socket_rtp()
                .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

            let socket_rtcp = self
                .rtc_peer
                .get_socket_rtcp()
                .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

            self.rtc_peer.agente_ice.reiniciar_agente_ice(
                &self.rtc_peer.logger,
                socket_rtp,
                socket_rtcp,
            );
        }

        let (certificado_generado, clave_privada, pkcs12) = generar_certificado_dtls()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        self.rtc_peer
            .dtls
            .guardar_certificado_local(certificado_generado.clone());
        self.rtc_peer
            .dtls
            .guardar_clave_privada_local(clave_privada);
        self.rtc_peer.dtls.setear_pkcs12_local(pkcs12);
        self.rtc_peer.dtls.establecer_rol(RolDtls::Indefinido);

        self.rtc_peer.logger.info(
            "Rol de offerer DTLS establecido como indefinido temporalmente por setup actpass",
            "Cliente",
        );

        let linea_sdp_huella = construir_linea_fingerprint_sdp(&certificado_generado)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        self.rtc_peer.logger.info(
            &format!(
                "Certificado DTLS local generado y huella agregada a la offer: {}",
                linea_sdp_huella
            ),
            "Cliente",
        );

        let mut offer = self
            .rtc_peer
            .generar_offer_y_registrar_candidatos()
            .map_err(|_| {
                ErrorCreadorDeConexion::ErrorInterno("Fallo al generar offer".to_string())
            })?;

        offer.agregar_atributo_a_todas_las_medias(linea_sdp_huella);
        // separ el setup x si hay q usar audio o algo de eso
        agregar_atributo_setup_dtls(&mut offer, true);

        let offer_str = offer.serializar();

        Ok(offer_str)
    }

    fn generar_answer(&mut self, offer: &str) -> Result<String, ErrorCreadorDeConexion> {
        if !self.es_primera_llamada {
            let socket_rtp = self
                .rtc_peer
                .get_socket_rtp()
                .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

            let socket_rtcp = self
                .rtc_peer
                .get_socket_rtcp()
                .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

            self.rtc_peer.agente_ice.reiniciar_agente_ice(
                &self.rtc_peer.logger,
                socket_rtp,
                socket_rtcp,
            );
        }

        let mut lineas_offer: Vec<&str> = offer.split("\n").collect();
        lineas_offer = lineas_offer.iter().map(|linea| linea.trim()).collect();
        let offer = DescripcionDeSesion::parsear(&lineas_offer[..])
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;
        let linea_setup = offer
            .buscar_setup()
            .ok_or(ErrorCreadorDeConexion::ErrorInterno(
                "Fallo al buscar setup".to_string(),
            ))?;

        let setup_value = parsear_setup_dtls(&linea_setup)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        let rol_local = determinar_rol_dtls_desde_setup(&setup_value, false)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        self.rtc_peer.dtls.establecer_rol(rol_local);

        self.rtc_peer.logger.info(
            &format!(
                "El answerer eligió como rol local a partir del setup del offer: {:?}",
                rol_local
            ),
            "Cliente",
        );

        let fingerprint_remota =
            offer
                .buscar_fingerprint()
                .ok_or(ErrorCreadorDeConexion::ErrorInterno(
                    "No existe fingerprint".to_string(),
                ))?;

        let fingerprint_limpio = parsear_fingerprint_remoto(&fingerprint_remota)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        self.rtc_peer
            .dtls
            .guardar_huella_remota(fingerprint_limpio.clone());

        self.rtc_peer.logger.info(
            &format!("Fingerprint remota guardada: {}", fingerprint_remota),
            "Cliente",
        );

        let (certificado_local, clave_privada_local, pkcs12_local) = generar_certificado_dtls()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        self.rtc_peer
            .dtls
            .guardar_certificado_local(certificado_local.clone());
        self.rtc_peer
            .dtls
            .guardar_clave_privada_local(clave_privada_local);
        self.rtc_peer.dtls.setear_pkcs12_local(pkcs12_local);

        self.rtc_peer
            .logger
            .info("Certificado DTLS local generado y guardado", "Cliente");

        let linea_fingerprint_local = construir_linea_fingerprint_sdp(&certificado_local)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

        let mut answer = self
            .rtc_peer
            .recibir_offer_y_responder(offer)
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        for media in answer.get_medias_mut() {
            media.limpiar_setup();
        }

        answer.agregar_atributo_a_todas_las_medias(linea_fingerprint_local);
        agregar_atributo_setup_dtls(&mut answer, false);

        Ok(answer.serializar())
    }

    fn recibir_answer(&mut self, answer: &str) -> Result<(), ErrorCreadorDeConexion> {
        let mut lineas_answer: Vec<&str> = answer.split("\n").collect();

        lineas_answer = lineas_answer.iter().map(|linea| linea.trim()).collect();

        let answer = DescripcionDeSesion::parsear(&lineas_answer[..])
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        let linea_setup = answer
            .buscar_setup()
            .ok_or(ErrorCreadorDeConexion::ErrorInterno(
                "No existe setup".to_string(),
            ))?;

        let setup_value = parsear_setup_dtls(&linea_setup)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        let rol = determinar_rol_dtls_desde_setup(&setup_value, true)
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;

        self.rtc_peer.dtls.establecer_rol(rol);

        self.rtc_peer.logger.info(
            &format!(
                "El offerer eligió como rol local a partir del setup del answer: {:?}",
                rol
            ),
            "Cliente",
        );

        if let Some(linea_fp) = answer.buscar_fingerprint() {
            let fp_limpio = parsear_fingerprint_remoto(&linea_fp)
                .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(String::from(e)))?;
            self.rtc_peer.dtls.guardar_huella_remota(fp_limpio.clone());

            self.rtc_peer.logger.info(
                &format!(
                    "Fingerprint remota (limpia) guardada en offerer: {}",
                    fp_limpio
                ),
                "Cliente",
            );
        } else {
            return Err(ErrorCreadorDeConexion::ErrorInterno(
                "No existe fingerprint".to_string(),
            ));
        }
        self.rtc_peer
            .recibir_answer(answer)
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        Ok(())
    }

    fn conectar(&mut self) -> Result<(), ErrorCreadorDeConexion> {
        let par_exitoso = self
            .rtc_peer
            .negociar_candidatos()
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        let parametros = setear_direcciones_externas(&mut self.rtc_peer, par_exitoso)?;
        self.par_exitoso = Some(parametros);
        esperar_y_entrar_a_llamada(&mut self.rtc_peer, &self.rx_termino_ice)?;

        let _ = limpiar_sockets_sin_cortar_llamada(&mut self.rtc_peer);

        self.es_primera_llamada = false;

        Ok(())
    }

    fn obtener_sockets(&mut self) -> Result<ConexionP2P, ErrorCreadorDeConexion> {
        let socket_rtp = self
            .rtc_peer
            .get_socket_rtp()
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        let socket_rtcp = self
            .rtc_peer
            .get_socket_rtcp()
            .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

        let host_sockets_receptor = self.rtc_peer.config.getter_host_remoto();
        let puerto_rtp = self.rtc_peer.config.getter_port_rtp_remoto();
        let puerto_rtcp = self.rtc_peer.config.getter_port_rtcp_remoto();

        let direccion_rtp_receptor = format!("{}:{}", host_sockets_receptor, puerto_rtp);
        let direccion_rtcp_receptor = format!("{}:{}", host_sockets_receptor, puerto_rtcp);

        let contexto_srtp_tx = self.rtc_peer.get_contexto_srtp_tx();
        let contexto_srtp_rx = self.rtc_peer.get_contexto_srtp_rx();

        let srtp_rx = self.rtc_peer.tomar_srtp_rx();

        let mut conexion = ConexionP2P::new(
            socket_rtp,
            socket_rtcp,
            direccion_rtp_receptor,
            direccion_rtcp_receptor,
            contexto_srtp_tx,
            contexto_srtp_rx,
            srtp_rx,
        );

        if let Some(tx) = self.rtc_peer.tomar_tx_datos_sctp() {
            conexion.set_tx_datos_sctp(tx);
        }

        if let Some(rx) = self.rtc_peer.tomar_rx_eventos_sctp() {
            conexion.set_rx_eventos_sctp(rx);
        }

        Ok(conexion)
    }
}

fn esperar_y_entrar_a_llamada(
    rtc_peer: &mut RTCPeerConnection,
    rx_termino_ice: &Receiver<String>,
) -> Result<(), ErrorCreadorDeConexion> {
    // Se termina hilo ice RTP
    rx_termino_ice
        .recv()
        .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

    // Se termina hilo ice RTCP
    rx_termino_ice
        .recv()
        .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

    // Espero que DTLS encripte los sockets
    rtc_peer
        .logger
        .info("Lanzando handshake DTLS post-ICE", "DTLS");

    match rtc_peer.iniciar_dtls_post_ice() {
        Ok(_) => {
            rtc_peer.logger.info("Handshake DTLS COMPLETO", "DTLS");

            let rol = RolConexion::from(rtc_peer.dtls.obtener_rol());

            rtc_peer.logger.info(
                &format!("Rol SCTP determinado a partir del rol DTLS: {:?}", rol),
                "SCTP",
            );

            let estado_sctp = construir_estado_sctp(rol)
                .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{:?}", e)))?;

            rtc_peer.logger.info(
                &format!("Estado SCTP construido con rol {:?} para handshake", rol),
                "SCTP",
            );

            if let Err(e) = rtc_peer.inicializar_sctp_post_dtls(estado_sctp) {
                rtc_peer
                    .logger
                    .error(&format!("Error inicializando SCTP: {:?}", e), "SCTP");
            }

            // la addr remota ya la tiene el config post-ICE
            let remote_addr: SocketAddr = format!(
                "{}:{}",
                rtc_peer.config.getter_host_remoto(),
                rtc_peer.config.getter_port_rtp_remoto()
            )
            .parse()
            .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;

            rtc_peer.logger.info(
                &format!(
                    "Iniciando handshake SCTP con dirección remota: {}",
                    remote_addr
                ),
                "SCTP",
            );

            let client_config = construir_client_config(construir_transport_config());
            let mut estado_handshake = EstadoHandshakeSctp::Inactivo;
            let estado_dcep = EstadoDcep::Inactivo;

            if let Err(e) = rtc_peer.iniciar_sctp_si_corresponde(
                &mut estado_handshake,
                remote_addr,
                client_config,
            ) {
                rtc_peer
                    .logger
                    .error(&format!("Error iniciando handshake SCTP: {:?}", e), "SCTP");
            }

            let es_dtls_client = matches!(rol, RolConexion::Inicia);

            if let Err(e) = rtc_peer.spawnear_loop_sctp(
                estado_handshake,
                es_dtls_client,
                "data".to_string(),
                remote_addr,
                estado_dcep,
            ) {
                rtc_peer
                    .logger
                    .error(&format!("Error lanzando loop SCTP: {:?}", e), "SCTP");
            }
        }
        Err(e) => {
            rtc_peer
                .logger
                .error(&format!("Error en DTLS: {:?}", e), "DTLS");
        }
    }

    Ok(())
}

fn setear_direcciones_externas(
    rtc_peer: &mut RTCPeerConnection,
    par_exitoso: Vec<String>,
) -> Result<(String, u16, u16), ErrorCreadorDeConexion> {
    let ip_remota = par_exitoso[1].clone();
    let puerto_rtp = par_exitoso[2]
        .parse::<u16>()
        .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;
    let puerto_rtcp = par_exitoso[3]
        .parse::<u16>()
        .map_err(|e| ErrorCreadorDeConexion::ErrorInterno(format!("{e}")))?;
    rtc_peer
        .config
        .establecer_informacion_conexion(ip_remota.clone(), puerto_rtp, puerto_rtcp);

    Ok((ip_remota, puerto_rtcp, puerto_rtp))
}

fn limpiar_sockets_sin_cortar_llamada(
    rtc_peer: &mut RTCPeerConnection,
) -> Result<(), ErrorCreadorDeConexion> {
    let mut socket_rtp = rtc_peer
        .get_socket_rtp()
        .map_err(ErrorCreadorDeConexion::ErrorInterno)?;
    let mut socket_rtcp = rtc_peer
        .get_socket_rtcp()
        .map_err(ErrorCreadorDeConexion::ErrorInterno)?;

    let mut buffer: [u8; 50000] = [0; 50000];
    if socket_rtp.mutar_no_bloqueante().is_err() {
        eprintln!("ERROR AL HACER NO BLOQUEANTE")
    }
    while let Ok((tamanio_mensaje, _)) = socket_rtp.recibir(&mut buffer) {
        if tamanio_mensaje == 0 {
            break;
        }
    }
    if socket_rtp.mutar_bloqueante().is_err() {
        eprintln!("ERROR AL HACER BLOQUEANTE")
    }
    if socket_rtcp.mutar_no_bloqueante().is_err() {
        eprintln!("ERROR AL HACER NO BLOQUEANTE")
    }
    while let Ok((tamanio_mensaje, _)) = socket_rtcp.recibir(&mut buffer) {
        if tamanio_mensaje == 0 {
            break;
        }
    }
    if socket_rtcp.mutar_bloqueante().is_err() {
        eprintln!("ERROR AL HACER BLOQUEANTE")
    }

    Ok(())
}
