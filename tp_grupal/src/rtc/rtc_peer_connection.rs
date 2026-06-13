//! Módulo `rtc_peer_connection`
//!
//! Este módulo define `RTCPeerConnection`, que representa un peer en una comunicación
//! WebRTC simplificada.
//!
//! Funcionalidades principales:
//! - Generar un Offer SDP y guardarlo en archivo.
//! - Recibir un Offer y generar un Answer.
//! - Recibir un Answer y registrarlo localmente.
//! - Generar y procesar candidatos ICE (tipo host).
//!
//! Internamente utiliza `DescripcionDeSesion`, `SesionSdp` y `DescripcionDeMedia`
//! para construir y analizar los SDP intercambiados entre peers.
use crate::config_room_rtc::ConfigRoomRTC;
use crate::logger::Logger;
use crate::protocolos::ice::agente_ice::AgenteICE;
use crate::protocolos::ice::generacion_candidatos::generar_candidatos;
use crate::protocolos::sctp::dcep::dcep_handshake::EstadoDcep;
use crate::protocolos::sctp::error_sctp::ErrorSctp;
use crate::protocolos::sctp::estado_sctp::EstadoSctp;
use crate::protocolos::sctp::evento_sctp::EventoSctp;
use crate::protocolos::sctp::handshake_sctp::{EstadoHandshakeSctp, iniciar_handshake_saliente};
use crate::protocolos::sctp::loop_sctp::spawnear_loop_sctp;
use crate::protocolos::sctp::rol_sctp::RolConexion;
use crate::protocolos::sdp::descripcion_de_sesion::DescripcionDeSesion;
use crate::rtc::logs::{EventoRTC, LogRTC};
use crate::rtc::transporte::demux::canal_demux::DemuxDtlsChannel;
use crate::rtc::transporte::demux::demux_post_ice::{DemuxPostIce, spawnear_demux_post_ice};
use crate::rtc::transporte::sockets::Sockets;
use crate::seguridad::dtls_protocolo::dtls_contexto::{DtlsContexto, RolDtls};
use crate::seguridad::dtls_protocolo::dtls_utils::validar_fingerprint_dtls;
use crate::seguridad::dtls_protocolo::errores::ErrorDTLSProtocolo;
use crate::seguridad::srtp::claves_srtp::ClavesSRTP;
use crate::seguridad::srtp::srtp_contexto::SRTPContexto;
use crate::sesion_rtp::socket_udp::SocketUDP;
use bytes::Bytes;
use sctp_proto::ClientConfig;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use udp_dtls::{DtlsAcceptor, DtlsConnector, DtlsStream, Identity, SrtpProfile};
const CONTEXTO_LOG: &str = "RTCPeerConnection";
type StreamDTLS = DtlsStream<DemuxDtlsChannel>;

/// Representa la conexión de un peer en una sesión WebRTC simplificada.
///
/// Contiene los SDP locales y remotos y métodos para generar y procesar
/// Offers y Answers.
pub struct RTCPeerConnection {
    pub config: ConfigRoomRTC,
    pub logger: Logger,
    sdp_local: Option<DescripcionDeSesion>,
    sdp_remoto: Option<DescripcionDeSesion>,
    sockets: Sockets,
    flag_video_activo: Option<Arc<AtomicBool>>, //todo arreglar para videollamada
    pub agente_ice: AgenteICE,
    // de ultima dsp lo ponemos privado y agregamos getters/setters si hace falta
    pub dtls: DtlsContexto,
    stream_dtls: Option<StreamDTLS>,
    contexto_srtp_tx: Option<SRTPContexto>,
    contexto_srtp_rx: Option<SRTPContexto>,
    // cosas de sctp:
    pub estado_sctp: Option<EstadoSctp>, // uso option porque este campo se inicializa cuando inicio la asociacion, algo similar a lo que pasaba con dtls
    demux_thread: Option<JoinHandle<()>>,
    rx_dtls: Option<Receiver<Vec<u8>>>,
    rx_srtp: Option<Receiver<Vec<u8>>>,
    loop_sctp_thread: Option<JoinHandle<()>>,
    send_socket_dtls: Option<UdpSocket>,
    tx_datos_sctp: Option<Sender<Bytes>>,
    rx_eventos_sctp: Option<Receiver<EventoSctp>>,
}

impl RTCPeerConnection {
    /// Crea una nueva instancia de RTCPeerConnection con la configuración dada.
    pub fn new(
        config: ConfigRoomRTC,
        logger: Logger,
        tx_termino_ice: Sender<String>,
    ) -> Result<Self, String> {
        let direccion = "0.0.0.0"; // escuchamos en todas las interfaces

        // delegamos inicialización de sockets al módulo sockets
        let puerto_rtp = config.getter_port_rtp_local();

        // delegamos inicialización de threads ICE y flags al módulo agente_ice
        let mut sockets = Sockets::new(&logger, direccion, puerto_rtp)?;
        let agente_ice = AgenteICE::new(
            &logger,
            sockets.clonar_socket_rtp()?,
            sockets.clonar_socket_rtcp()?,
            tx_termino_ice,
        );

        Ok(Self {
            config,
            logger,
            sdp_local: None,
            sdp_remoto: None,
            sockets,
            flag_video_activo: None,
            agente_ice,
            dtls: DtlsContexto::new(),
            stream_dtls: None,
            contexto_srtp_tx: None,
            contexto_srtp_rx: None,
            estado_sctp: None,
            demux_thread: None,
            rx_dtls: None,
            rx_srtp: None,
            loop_sctp_thread: None,
            send_socket_dtls: None,
            tx_datos_sctp: None,
            rx_eventos_sctp: None,
        })
    }

    // getters
    pub fn get_sdp_local(&self) -> Option<DescripcionDeSesion> {
        self.sdp_local.clone()
    }

    pub fn get_sdp_remoto(&self) -> Option<DescripcionDeSesion> {
        self.sdp_remoto.clone()
    }

    pub fn get_config(&self) -> &ConfigRoomRTC {
        &self.config
    }

    pub fn get_flag_video(&self) -> Option<Arc<AtomicBool>> {
        self.flag_video_activo.clone()
    }

    pub fn get_socket_rtp(&mut self) -> Result<Box<dyn SocketUDP>, String> {
        self.sockets.clonar_socket_rtp()
    }

    pub fn get_socket_rtcp(&mut self) -> Result<Box<dyn SocketUDP>, String> {
        self.sockets.clonar_socket_rtcp()
    }

    pub fn get_contexto_srtp_tx(&self) -> Option<SRTPContexto> {
        self.contexto_srtp_tx.clone()
    }

    pub fn get_contexto_srtp_rx(&self) -> Option<SRTPContexto> {
        self.contexto_srtp_rx.clone()
    }

    // setters
    pub fn set_flag_video(&mut self, flag: Option<Arc<AtomicBool>>) {
        self.flag_video_activo = flag;
    }

    pub fn registrar_sdp_local(&mut self, sdp: &DescripcionDeSesion) {
        self.sdp_local = Some(sdp.clone());
    }

    pub fn registrar_sdp_remoto(&mut self, sdp: &DescripcionDeSesion) {
        self.sdp_remoto = Some(sdp.clone());
    }

    /// Negocia usando internamente todas las interfaces locales y devuelve el candidato seleccionado.
    ///
    /// Usa los SDP locales y remotos para determinar las medias activas, ejecuta los checks de connectividad ICE a través de los sockets, y retorna los datos del candidato seleccionado.
    ///
    /// Si falta alguno de los SDP o si la negociación falla, devuelve Err(String).
    pub fn negociar_candidatos(&mut self) -> Result<Vec<String>, String> {
        let sdp_local = match &self.get_sdp_local() {
            Some(sdp) => sdp.clone(),
            None => return Err("No hay SDP local".into()),
        };
        let sdp_remoto = match &self.get_sdp_remoto() {
            Some(sdp) => sdp.clone(),
            None => return Err("No hay SDP remoto".into()),
        };

        let mut socket_rtp = self.get_socket_rtp()?;
        let mut socket_rtcp = self.get_socket_rtcp()?;

        let (tipo, ip_remota, puerto_rtp, puerto_rtcp) = self.ejecutar_negociacion(
            &sdp_local,
            &sdp_remoto,
            &mut *socket_rtp,
            &mut *socket_rtcp,
        )?;

        Ok(vec![
            tipo,
            ip_remota,
            puerto_rtp.to_string(),
            puerto_rtcp.to_string(),
        ])
    }

    /// Ejecuta la negociación ICE sobre los sockets RTP y RTCP.
    ///
    /// Llama internamente a negociar_y_obtener_candidatos() para seleccionar uno válido y marcar la finalización de la etapa ICE. Retorna los datos del candidato seleccionado.
    fn ejecutar_negociacion(
        &mut self,
        sdp_local: &DescripcionDeSesion,
        sdp_remoto: &DescripcionDeSesion,
        socket_rtp: &mut dyn SocketUDP,
        socket_rtcp: &mut dyn SocketUDP,
    ) -> Result<(String, String, u16, u16), String> {
        let rx_ice = match self.agente_ice.rx.take() {
            Some(rx) => rx,
            None => {
                return Err("El Receiver ICE ya fue consumido, error.".to_string());
            }
        };
        let resultado = crate::rtc::negociacion::negociacion_medias::negociar_y_obtener_candidato(
            &self.logger,
            (sdp_local, sdp_remoto),
            (socket_rtp, socket_rtcp),
            (
                rx_ice,
                (&self.agente_ice.flags, self.agente_ice.es_controlling()),
            ),
        )
        .map_err(|e| e.to_string())?;

        self.logger
            .info("Este peer terminó su etapa ICE", CONTEXTO_LOG);
        Ok(resultado)
    }

    /// Paso 1 de la comunicación: La generación de un offer.sdp con información propia, y su guardado en un archivo. Retorna la DescripcionDeSesion (estructura SDP completa de tipo offer) lista para ser enviada a otro peer.
    /// El peer recibidor nunca hace uso de esta función. Es sólo para quien quiere iniciar una llamada, ya que no sólo genera el offer, también lo guarda, cosa que para el peer B no nos interesa que pase (también genera su sdp offer porque en base a él crea el answer, pero no nos interesa que dicho offer quede guardaro, sólo nos interesa guardar el answer final que genera).
    ///
    /// # Retorna
    /// `Result<DescripcionDeSesion, String>` con el SDP Offer o un mensaje de error
    /// si falla al generar candidatos o guardar el archivo.
    pub fn generar_offer_y_registrar_candidatos(&mut self) -> Result<DescripcionDeSesion, String> {
        self.agente_ice.set_controlling(true);
        let mut offer = DescripcionDeSesion::generar_offer(&self.config);

        self.generar_candidatos_ice(&mut offer)?;
        self.registrar_sdp_local(&offer);
        self.log_detalles_offer(&offer);
        self.log_estado_medias(&offer, "Peer A");
        self.log_evento(EventoRTC::OfferGenerado);

        offer
            .guardar_en_archivo(self.config.getter_sdp_offer_file())
            .map_err(|e| {
                self.log_evento(EventoRTC::Error(&format!("{}", e)));
                e.to_string()
            })?;

        self.log_evento(EventoRTC::OfferGuardado(
            self.config.getter_sdp_offer_file(),
        ));
        Ok(offer)
    }

    /// Paso 2 de la comunicación: el peer receptor recibe un SDP Offer,
    /// genera un SDP Answer con la información compatible local,
    /// lo guarda en un archivo y lo retorna.
    ///
    /// Este método se encarga de:
    /// - Registrar el offer remoto.
    /// - Filtrar las medias no soportadas o con codecs incompatibles.
    /// - Generar y agregar candidatos ICE locales.
    ///
    /// # Parámetros
    /// - `offer_remoto`: SDP Offer recibido.
    ///
    /// # Retorna
    /// `Result<DescripcionDeSesion, String>` con el SDP Answer o error si falla al guardar.
    pub fn recibir_offer_y_responder(
        &mut self,
        offer_remoto: DescripcionDeSesion,
    ) -> Result<DescripcionDeSesion, String> {
        self.agente_ice.set_controlling(false);
        self.log_evento(EventoRTC::OfferRecibido);

        let offer_clonado = offer_remoto.clone();
        self.registrar_sdp_remoto(&offer_clonado);

        let mut answer =
            DescripcionDeSesion::generar_answer_desde_offer(&offer_clonado, &self.config);
        self.generar_candidatos_ice(&mut answer)?;
        self.registrar_sdp_local(&answer);

        self.log_evento(EventoRTC::AnswerGuardado(
            self.config.getter_sdp_answer_file(),
        ));

        answer
            .guardar_en_archivo(self.config.getter_sdp_answer_file())
            .map_err(|e| format!("Error guardando Answer SDP: {}", e))?;

        Ok(answer)
    }

    /// Paso 3 de la comunicación: el peer que inició la comunicación recibe un SDP Answer del peer remoto y lo registra localmente.
    ///
    /// # Retorna
    /// `Result<(), String>` o error si no hay SDP local o hay inconsistencia de medias.
    pub fn recibir_answer(&mut self, mut answer: DescripcionDeSesion) -> Result<(), String> {
        self.log_evento(EventoRTC::AnswerRecibido);

        if let Err(error) = self.validar_y_actualizar_medias(&mut answer) {
            self.logger
                .error(&format!("Error validando medias: {}", error), CONTEXTO_LOG);
            return Err(error);
        }

        self.registrar_sdp_remoto(&answer);

        self.log_estado_medias_despues_de_answer();
        self.log_evento(EventoRTC::AnswerProcesado);

        Ok(())
    }

    /// Valida que el número de `medias` en el SDP remoto coincida con el local,
    /// y copia los candidatos ICE locales del Answer remoto como candidatos remotos
    /// dentro de cada media del SDP local.
    ///
    /// # Errores
    /// Retorna `Err` si no existe un SDP local registrado o si la cantidad de medias no coincide.
    fn validar_y_actualizar_medias(
        &mut self,
        answer: &mut DescripcionDeSesion,
    ) -> Result<(), String> {
        // verificar que haya un SDP local
        let logger = &self.logger;
        let local_opt = self.sdp_local.as_mut();
        let local = match local_opt {
            Some(l) => l,
            None => {
                let msg = "No hay SDP local registrado antes de recibir el Answer".to_string();
                logger.error(&msg, CONTEXTO_LOG);
                return Err(msg);
            }
        };

        // verificar que coincida la cantidad de medias
        let medias_remotas = answer.get_medias();
        if local.get_medias().len() != medias_remotas.len() {
            let msg = format!(
                "Cantidad de medias no coincide (local: {}, remoto: {})",
                local.get_medias().len(),
                medias_remotas.len()
            );
            logger.error(&msg, CONTEXTO_LOG);
            return Err(msg);
        }

        // actualizar candidatos remotos
        for (i, media_remota) in medias_remotas.iter().enumerate() {
            let candidatos_remotos = media_remota.get_candidatos_ice_locales().clone();
            local.get_medias_mut()[i].establecer_candidatos_remotos(candidatos_remotos);
        }

        Ok(())
    }

    /// Genera candidatos ICE (host y srflx) y los agrega a cada media del SDP dado.
    fn generar_candidatos_ice(&mut self, sdp: &mut DescripcionDeSesion) -> Result<(), String> {
        let puerto_rtp_local = self.config.getter_port_rtp_local();
        let stun_server = self.config.getter_stun_server();

        let candidatos_generados =
            generar_candidatos(&self.logger, puerto_rtp_local, Some(stun_server))
                .map_err(|e| format!("Error al generar candidatos ICE: {}", e))?;

        for media in sdp.get_medias_mut() {
            if media.get_puerto() == 0 {
                // media rechazada, no agregar candidatos
                continue;
            }
            for c in &candidatos_generados {
                media.agregar_candidato_local(c.clone());
            }
        }

        Ok(())
    }

    // ------------------------------------------------- COSAS DE DTLS -------------------------------------------------

    /// Devuelve una referencia mutable a los sockets UDP utilizados para la comunicación DTLS.
    pub fn obtener_sockets_dtls(&mut self) -> &mut Sockets {
        &mut self.sockets
    }

    /// Inicia el proceso de establecimiento de la conexión DTLS después de finalizar la etapa ICE.
    pub fn iniciar_dtls_post_ice(&mut self) -> Result<(), ErrorDTLSProtocolo> {
        self.logger.info("Iniciando DTLS post-ICE...", "DTLS");
        self.agente_ice.detener_hilos_ice();
        self.logger.info("Hilos ICE detenidos", "DTLS");
        self.logger
            .info("Iniciando demux post-ICE...", "DTLS-NUEVO");
        self.iniciar_demux_post_ice()?;
        match self.dtls.obtener_rol() {
            RolDtls::Cliente => self.iniciar_dtls_como_cliente(),
            RolDtls::Servidor => self.iniciar_dtls_como_servidor(),
            RolDtls::Indefinido => Err(ErrorDTLSProtocolo::ErrorRolNoEstablecido),
        }
    }

    fn iniciar_demux_post_ice(&mut self) -> Result<(), ErrorDTLSProtocolo> {
        if let Some(handle) = self.demux_thread.take() {
            let _ = handle.join();
        }

        let raw = self.crear_socket_dtls()?;
        let recv_socket = raw
            .try_clone()
            .map_err(|_| ErrorDTLSProtocolo::ErrorObteniendoSocketRtpParaDtls)?;

        let (dtls_tx, dtls_rx) = mpsc::channel::<Vec<u8>>();
        let (srtp_tx, srtp_rx) = mpsc::channel::<Vec<u8>>();

        let handle = spawnear_demux_post_ice(recv_socket, DemuxPostIce { dtls_tx, srtp_tx });

        self.demux_thread = Some(handle);
        self.rx_dtls = Some(dtls_rx);
        self.rx_srtp = Some(srtp_rx);
        self.send_socket_dtls = Some(raw);
        Ok(())
    }

    fn validar_rol_dtls_esperado(&self, esperado: RolDtls) -> Result<(), ErrorDTLSProtocolo> {
        if self.dtls.obtener_rol() != esperado {
            self.logger.error(
                &format!(
                    "Error: rol DTLS no coincide con el negociado en SDP ({:?})",
                    esperado
                ),
                "DTLS",
            );
            return Err(ErrorDTLSProtocolo::ErrorRolIncorrecto);
        }
        self.logger.info(
            &format!("Rol DTLS validado correctamente ({:?})", esperado),
            "DTLS",
        );
        Ok(())
    }

    fn finalizar_dtls_y_srtp(&mut self, stream: StreamDTLS) -> Result<(), ErrorDTLSProtocolo> {
        self.stream_dtls = Some(stream);

        let ssl = {
            let stream_ref = self
                .stream_dtls
                .as_ref()
                .ok_or(ErrorDTLSProtocolo::ErrorHandshakeDTLS)?;
            stream_ref.0.ssl()
        };

        let claves_srtp = {
            let claves = self.dtls.exportar_claves_srtp(ssl)?;
            claves.clone()
        };

        self.logger
            .info(&format!("Claves SRTP generadas: {:?}", claves_srtp), "DTLS");

        self.crear_contextos_srtp(&claves_srtp)?;

        Ok(())
    }

    fn construir_direccion_remota_config(&self) -> Result<SocketAddr, ErrorDTLSProtocolo> {
        // construimos la dirección remota a partir del config cargado post-ICE donde vamos a hacer DTLS
        let dir_remota = format!(
            "{}:{}",
            self.config.getter_host_remoto(),
            self.config.getter_port_rtp_remoto()
        )
        .parse()
        .map_err(|_| ErrorDTLSProtocolo::ErrorSetupInvalido)?;

        self.logger
            .info(&format!("Dirección remota DTLS: {}", dir_remota), "DTLS");

        Ok(dir_remota)
    }

    fn crear_identidad_sobre_pkcs12(&self) -> Result<Identity, ErrorDTLSProtocolo> {
        // obtenemos el certificado PKCS12 local guardado previamente generado en negociación
        let identidad_p12 = self
            .dtls
            .obtener_pkcs12_local()
            .ok_or(ErrorDTLSProtocolo::ErrorCertificadoInvalido)?;

        self.logger
            .info("Certificado PKCS12 local obtenido para DTLS", "DTLS");

        // creamos la identidad a partir del PKCS12, usamos contraseña fija "1234" para simplificar
        // podriamos cambiarla (?)
        let identidad = Identity::from_pkcs12(identidad_p12, "1234")
            .map_err(|_| ErrorDTLSProtocolo::ErrorGenerandoCertificado)?;

        self.logger
            .info("Identidad DTLS creada a partir del PKCS12", "DTLS");

        Ok(identidad)
    }

    fn crear_dtls_connector(
        &self,
        identidad: Identity,
    ) -> Result<DtlsConnector, ErrorDTLSProtocolo> {
        // creamos el dtls connector con la identidad y perfiles SRTP del lado del cliente, los perfiles se negocian y despues permiten
        // crear las claves SRTP que vamos a necesitar

        // -AES-GCM = más seguro, pero no todos los navegadores lo soportan
        // -AES-CM = más compatible, pero menos seguro
        let mut binding = DtlsConnector::builder();
        let builder_conector = binding
            .identity(identidad)
            .add_srtp_profile(SrtpProfile::AeadAes256Gcm)
            .add_srtp_profile(SrtpProfile::Aes128CmSha180);
        // con esto evito el error de unknown ca ->  esta  validación no es necesaria en WebRTC asi que no hay drama
        builder_conector.danger_accept_invalid_certs(true);
        builder_conector.danger_accept_invalid_hostnames(true);
        self.logger.info("DTLS Connector creado", "DTLS");

        // creo finalmente el conector
        let conector = builder_conector
            .build()
            .map_err(|_| ErrorDTLSProtocolo::ErrorHandshakeDTLS)?;

        self.logger.info("DTLS Connector final construido", "DTLS");
        Ok(conector)
    }

    fn crear_socket_dtls(&self) -> Result<UdpSocket, ErrorDTLSProtocolo> {
        // creo socket para DTLS
        let socket = self
            .sockets
            .obtener_raw_rtp()
            .map_err(|_| ErrorDTLSProtocolo::ErrorObteniendoSocketRtpParaDtls)?;

        self.logger
            .info("Socket UDP RTP obtenido para DTLS", "DTLS");

        Ok(socket)
    }

    fn ejecutar_handshake_dtls_como_cliente(&mut self) -> Result<StreamDTLS, ErrorDTLSProtocolo> {
        self.logger.info("Iniciando DTLS como CLIENTE...", "DTLS");
        let addr_remoto = self.construir_direccion_remota_config()?;
        let identidad = self.crear_identidad_sobre_pkcs12()?;
        let conector = self.crear_dtls_connector(identidad)?;

        let send_socket = self
            .send_socket_dtls
            .take()
            .ok_or(ErrorDTLSProtocolo::ErrorObteniendoSocketRtpParaDtls)?;
        let dtls_rx = self
            .rx_dtls
            .take()
            .ok_or(ErrorDTLSProtocolo::ErrorHandshakeDTLS)?;
        let canal = DemuxDtlsChannel {
            send_socket,
            remote: addr_remoto,
            dtls_rx,
        };
        let stream = conector
            .connect("rtc-peer", canal)
            .map_err(|_| ErrorDTLSProtocolo::ErrorHandshakeDTLS)?;

        self.logger
            .info("Handshake DTLS CLIENTE COMPLETADO", "DTLS");

        Ok(stream)
    }

    fn obtener_certificado_remoto_handshake_dtls(
        &mut self,
        stream: &StreamDTLS,
    ) -> Result<udp_dtls::Certificate, ErrorDTLSProtocolo> {
        let certificado_remoto = stream
            .peer_certificate()
            .map_err(|_| ErrorDTLSProtocolo::ErrorHandshakeDTLS)?
            .ok_or(ErrorDTLSProtocolo::ErrorCertificadoRemotoInexistente)?;

        self.logger
            .info("Certificado remoto DTLS obtenido del handshake", "DTLS");

        Ok(certificado_remoto)
    }

    fn obtener_fingerprint_remoto_sdp(&self) -> Result<&str, ErrorDTLSProtocolo> {
        let huella_remota = self
            .dtls
            .obtener_huella_remota()
            .ok_or(ErrorDTLSProtocolo::ErrorFingerprintRemotoInexistente)?;

        self.logger
            .info("Fingerprint remoto DTLS obtenido del SDP", "DTLS");

        Ok(huella_remota)
    }

    fn iniciar_dtls_como_cliente(&mut self) -> Result<(), ErrorDTLSProtocolo> {
        let stream = self.ejecutar_handshake_dtls_como_cliente()?;
        // antes de seguir, valido que el rol sea correcto (por correcto me refiero a que el rol del peer debe ser cliente)
        self.validar_rol_dtls_esperado(RolDtls::Cliente)?;
        self.logger
            .info("Rol DTLS validado correctamente (cliente)", "DTLS");
        // fase 2 -> validacion del fingerprint remoto
        // el fingerprint SDP debe compararse con el certificado remoto despues de el handshake
        // el certificado remoto se obtiene del stream DTLS luego del handshake
        let certificado_remoto = self.obtener_certificado_remoto_handshake_dtls(&stream)?;
        // obtengo el fingerprint remoto que vino desde el SDP
        let huella_remota = self.obtener_fingerprint_remoto_sdp()?;
        // validamos que coincida el fingerprint remoto con el certificado recibido en el handshake
        validar_fingerprint_dtls(&certificado_remoto, huella_remota)?;
        self.logger
            .info("Fingerprint remoto DTLS validado correctamente", "DTLS");
        self.dtls
            .establecer_certificado_remoto(certificado_remoto.clone());
        self.logger
            .info("Certificado remoto DTLS almacenado correctamente", "DTLS");
        self.finalizar_dtls_y_srtp(stream)?;
        self.logger
            .info("Contextos SRTP creados correctamente (cliente)", "SRTP");
        self.logger
            .info("Handshake DTLS CLIENTE COMPLETADO", "DTLS");
        Ok(())
    }

    fn crear_dtls_acceptor(&self, identidad: Identity) -> Result<DtlsAcceptor, ErrorDTLSProtocolo> {
        // creamos el dtls acceptor con la identidad y perfiles SRTP del lado del servidor, los perfiles se negocian y despues permiten
        // crear las claves SRTP que vamos a necesitar
        // -AES-GCM = más seguro, pero no todos los navegadores lo soportan
        // -AES-CM = más compatible, pero menos seguro
        let acceptor = DtlsAcceptor::builder(identidad)
            .build()
            .map_err(|_| ErrorDTLSProtocolo::ErrorHandshakeDTLS)?;

        self.logger.info("DTLS Acceptor creado", "DTLS");

        Ok(acceptor)
    }

    fn ejecutar_handshake_dtls_como_servidor(&mut self) -> Result<StreamDTLS, ErrorDTLSProtocolo> {
        let addr_remoto = self.construir_direccion_remota_config()?;
        let identidad = self.crear_identidad_sobre_pkcs12()?;
        let acceptor = self.crear_dtls_acceptor(identidad)?;

        let send_socket = self
            .send_socket_dtls
            .take()
            .ok_or(ErrorDTLSProtocolo::ErrorObteniendoSocketRtpParaDtls)?;

        // receiver con SOLO datagramas DTLS (lo alimenta el demux)
        let dtls_rx = self
            .rx_dtls
            .take()
            .ok_or(ErrorDTLSProtocolo::ErrorHandshakeDTLS)?;

        let canal = DemuxDtlsChannel {
            send_socket,
            remote: addr_remoto,
            dtls_rx,
        };

        let stream = acceptor.accept(canal).map_err(|e| {
            self.logger.error(
                &format!("Error en handshake DTLS servidor: {:?}", e),
                "DTLS",
            );
            ErrorDTLSProtocolo::ErrorHandshakeDTLS
        })?;

        self.logger
            .info("Handshake DTLS SERVIDOR COMPLETADO", "DTLS");
        Ok(stream)
    }

    fn validar_recepcion_certificado_remoto_servidor(&mut self, stream: &StreamDTLS) {
        match stream.peer_certificate() {
            Ok(Some(cert)) => {
                self.dtls.establecer_certificado_remoto(cert.clone());
                self.logger.info(
                    "Certificado remoto recibido en handshake (servidor)",
                    "DTLS",
                );
            }
            _ => {
                self.logger.info(
                    "Certificado remoto NO disponible en handshake (servidor)",
                    "DTLS",
                );
            }
        }
    }

    fn iniciar_dtls_como_servidor(&mut self) -> Result<(), ErrorDTLSProtocolo> {
        self.logger.info("Iniciando DTLS como SERVIDOR...", "DTLS");
        let stream = self.ejecutar_handshake_dtls_como_servidor()?;
        // antes de seguir, valido que el rol sea correcto (por correcto me refiero a que el rol del peer debe ser servidor)
        self.validar_rol_dtls_esperado(RolDtls::Servidor)?;
        self.logger
            .info("Rol DTLS validado correctamente (servidor)", "DTLS");
        // de aca saco la validacion fingerprint xq webrtc no la hace en el servidor solo en cliente
        self.validar_recepcion_certificado_remoto_servidor(&stream);
        self.finalizar_dtls_y_srtp(stream)?;
        self.logger
            .info("Contextos SRTP creados correctamente (servidor)", "SRTP");
        self.logger
            .info("Handshake DTLS SERVIDOR COMPLETADO", "DTLS");
        Ok(())
    }

    /// Crea los contextos SRTP de transmisión y recepción a partir de las claves exportadas del handshake DTLS.
    pub fn crear_contextos_srtp(&mut self, claves: &ClavesSRTP) -> Result<(), ErrorDTLSProtocolo> {
        match self.dtls.obtener_rol() {
            RolDtls::Indefinido => Err(ErrorDTLSProtocolo::ErrorRolNoEstablecido),
            RolDtls::Cliente | RolDtls::Servidor => {
                self.contexto_srtp_tx = Some(SRTPContexto::new(
                    claves.clave_tx.clone(),
                    claves.salt_tx.clone(),
                ));
                self.contexto_srtp_rx = Some(SRTPContexto::new(
                    claves.clave_rx.clone(),
                    claves.salt_rx.clone(),
                ));
                Ok(())
            }
        }
    }

    // ------------------------------------------------- COSAS DE SCTP --------------------------------------------------

    /// Inicializa el estado de SCTP después de completar el handshake DTLS, preparando la capa SCTP para comenzar su propio proceso de handshake
    /// según el rol de conexión.
    pub fn inicializar_sctp_post_dtls(&mut self, estado_sctp: EstadoSctp) -> Result<(), ErrorSctp> {
        self.estado_sctp = Some(estado_sctp);
        self.logger.info("SCTP inicializado post-DTLS", "SCTP");
        Ok(())
    }

    /// Inicia el proceso de handshake de SCTP si el rol de conexión lo requiere (es decir, si es el iniciador).
    pub fn iniciar_sctp_si_corresponde(
        &mut self,
        estado_handshake_sctp: &mut EstadoHandshakeSctp,
        remote: SocketAddr,
        client_cfg: ClientConfig,
    ) -> Result<(), ErrorSctp> {
        let sctp = self
            .estado_sctp
            .as_mut()
            .ok_or(ErrorSctp::ErrorIniciarPostDTLS)?;

        self.logger.info("Obteniendo rol de conexión SCTP", "SCTP");
        let rol_sctp: RolConexion = self.dtls.obtener_rol().into();

        match rol_sctp {
            RolConexion::Inicia => {
                // el que inicia es el que hace el primer envío, así que acá arranco el handshake de SCTP
                iniciar_handshake_saliente(estado_handshake_sctp, sctp, remote, client_cfg)?;
            }
            RolConexion::Acepta | RolConexion::Dual => {
                // este rol no inicia pero tiene que estar esperando
                // lo pongo en conectando para avanzar el handshake segun eventos
                *estado_handshake_sctp = EstadoHandshakeSctp::Conectando;
            }
        }
        Ok(())
    }

    /// Después de completar el handshake de SCTP, este método inicia el loop principal de SCTP en un hilo separado, proporcionando canales para enviar
    /// datos SCTP a la red y recibir eventos de aplicación SCTP.
    pub fn spawnear_loop_sctp(
        &mut self,
        estado_handshake: EstadoHandshakeSctp,
        es_dtls_client: bool,
        label_canal: String,
        remote: SocketAddr,
        estado_dcep: EstadoDcep,
    ) -> Result<(), ErrorSctp> {
        let stream = self
            .stream_dtls
            .take()
            .ok_or(ErrorSctp::ErrorIniciarPostDTLS)?;
        let estado = self
            .estado_sctp
            .take()
            .ok_or(ErrorSctp::ErrorIniciarPostDTLS)?;
        let estados_completos = (estado, estado_handshake, estado_dcep);

        // El spawnear_loop_sctp que se llama acá corresponde a loop_sctp.rs, que es quien realmente lanza el thread
        let (handle, tx_datos, rx_eventos) = spawnear_loop_sctp(
            stream,
            estados_completos,
            es_dtls_client,
            label_canal,
            remote,
            self.logger.clone(),
        );

        self.loop_sctp_thread = Some(handle);
        self.tx_datos_sctp = Some(tx_datos);
        self.rx_eventos_sctp = Some(rx_eventos);
        self.logger.info("Loop SCTP lanzado", "SCTP");
        Ok(())
    }

    /// Toma el Receiver de SRTP para recibir datos SRTP desencriptados desde el demux post-ICE
    pub fn tomar_srtp_rx(&mut self) -> Option<mpsc::Receiver<Vec<u8>>> {
        self.rx_srtp.take()
    }

    /// Toma el Sender de SRTP para enviar datos SRTP a través del demux post-ICE
    pub fn tomar_tx_datos_sctp(&mut self) -> Option<Sender<Bytes>> {
        self.tx_datos_sctp.take()
    }

    /// Toma el Receiver de eventos SCTP para recibir eventos de aplicación SCTP desde el loop de SCTP
    pub fn tomar_rx_eventos_sctp(&mut self) -> Option<Receiver<EventoSctp>> {
        self.rx_eventos_sctp.take()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;
    use crate::config_room_rtc::{ConfigRoomRTC, Direcciones};

    fn generar_config(puerto_base: u16) -> ConfigRoomRTC {
        let locales = Direcciones::new("0.0.0.0".to_string(), puerto_base);
        let remotas = Direcciones::new("0.0.0.0".to_string(), puerto_base);

        ConfigRoomRTC::crear_struct(
            locales,
            remotas,
            "/tmp/log.log".to_string(),
            format!("./archivos_test/offer_{}.sdp", puerto_base),
            format!("./archivos_test/answer_{}.sdp", puerto_base),
            "stun_server: stun.cloudflare.com:3478;".to_string(),
            "0.0.0.0:1818".to_string(),
        )
    }

    #[test]
    fn new_crea_peer_sin_sdps() -> Result<(), Box<dyn std::error::Error>> {
        let config = generar_config(9100);
        let logger = Logger::dummy_logger();
        let (tx_termino, _rx_termino) = mpsc::channel();
        let peer = RTCPeerConnection::new(config, logger, tx_termino)?;

        assert!(peer.sdp_local.is_none());
        assert!(peer.sdp_remoto.is_none());
        Ok(())
    }

    #[test]
    fn funcion_inicial_genera_offer_y_guarda() -> Result<(), Box<dyn std::error::Error>> {
        let config = generar_config(9200);
        let logger = Logger::dummy_logger();
        let (tx_termino, _rx_termino) = mpsc::channel();
        let mut peer = RTCPeerConnection::new(config, logger, tx_termino)?;

        let offer = peer
            .generar_offer_y_registrar_candidatos()
            .expect("No debería fallar al generar Offer");

        let sdp_local = peer.get_sdp_local().expect("SDP local no registrado");
        assert_eq!(sdp_local.get_medias().len(), offer.get_medias().len());
        Ok(())
    }

    #[test]
    fn recibir_offer_y_responder_registra_sdp_local_y_remoto()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = generar_config(9300);
        let logger = Logger::dummy_logger();
        let (tx_termino, _rx_termino) = mpsc::channel();
        let mut peer = RTCPeerConnection::new(config, logger, tx_termino)?;

        let mut offer = DescripcionDeSesion::generar_offer(&peer.config);
        for media in offer.get_medias_mut() {
            media.limpiar_candidatos_locales();
            media.agregar_candidato_local(
                "candidate:1 1 udp 2122260223 0.0.0.0 1234 typ host".to_string(),
            );
        }

        let mut answer = peer
            .recibir_offer_y_responder(offer.clone())
            .expect("Error generando Answer");

        // aseguramos que cada media del Answer tenga al menos un candidato local
        for media in answer.get_medias_mut() {
            if media.get_candidatos_ice_locales().is_empty() {
                media.agregar_candidato_local(format!(
                    "candidate:1 1 udp 2122260223 0.0.0.0 {} typ host",
                    media.get_puerto()
                ));
            }
        }

        peer.sdp_local = Some(answer.clone());

        let sdp_remoto = peer.get_sdp_remoto().expect("SDP remoto no registrado");
        assert_eq!(sdp_remoto.get_medias().len(), offer.get_medias().len());

        let sdp_local = peer
            .get_sdp_local()
            .expect("SDP local no generado en Answer");
        assert_eq!(sdp_local.get_medias().len(), answer.get_medias().len());

        for media in sdp_local.get_medias() {
            assert!(!media.get_candidatos_ice_locales().is_empty());
        }
        Ok(())
    }

    #[test]
    fn recibir_answer_registra_sdp_remoto() -> Result<(), Box<dyn std::error::Error>> {
        let config = generar_config(9400);
        let logger = Logger::dummy_logger();
        let (tx_termino, _rx_termino) = mpsc::channel();
        let mut peer = RTCPeerConnection::new(config, logger, tx_termino)?;

        let mut offer = peer
            .generar_offer_y_registrar_candidatos()
            .expect("No debería fallar al generar Offer");
        for media in offer.get_medias_mut() {
            media.agregar_candidato_local(
                "candidate:1 1 udp 2122260223 0.0.0.0 1234 typ host".to_string(),
            );
        }

        let answer = peer
            .recibir_offer_y_responder(offer.clone())
            .expect("Error generando Answer");

        peer.recibir_answer(answer.clone())
            .expect("Fallo al recibir Answer: debería contener candidatos ICE");

        let sdp_remoto = peer.get_sdp_remoto().expect("SDP remoto no registrado");
        assert_eq!(sdp_remoto.get_medias().len(), answer.get_medias().len());
        Ok(())
    }

    #[test]
    fn registrar_sdp_remoto_actualiza_estado_correctamente()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = generar_config(9500);
        let logger = Logger::dummy_logger();
        let (tx_termino, _rx_termino) = mpsc::channel();
        let mut peer = RTCPeerConnection::new(config, logger, tx_termino)?;

        let offer = DescripcionDeSesion::generar_offer(&peer.config);

        peer.registrar_sdp_remoto(&offer);

        let sdp_remoto = peer.get_sdp_remoto().unwrap();
        assert_eq!(sdp_remoto.get_medias().len(), offer.get_medias().len());

        for media in sdp_remoto.get_medias() {
            assert!(media.get_candidatos_ice_locales().is_empty());
            assert!(media.get_candidatos_ice_remotos().is_empty());
        }
        Ok(())
    }
}
