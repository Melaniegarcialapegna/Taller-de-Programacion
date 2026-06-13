//! Módulo `descripcion_de_sesion`
//!
//! Este módulo define la estructura `DescripcionDeSesion` que representa un mensaje completo
//! del protocolo SDP (Offer o Answer). Contiene la sesión (`SesionSdp`) y
//! una lista de descripciones de media (`DescripcionDeMedia`).
//!
//! También incluye métodos para:
//! - Generar un offer o answer.
//! - Serializar a formato SDP.
//! - Parsear desde líneas de texto SDP de manera segura (con `Result`).
use super::traits::{ParseableSdp, SerializableSdp};
use crate::config_room_rtc::ConfigRoomRTC;
use crate::protocolos::sdp::{media::DescripcionDeMedia, sesion::SesionSdp};
use std::fs::File;
use std::io::{self, BufWriter, Write};

/// Representa un mensaje SDP completo.
///
/// Contiene el tipo (Offer/Answer), la sesión y una lista de descripciones de media.
#[derive(Debug, Clone)]
pub struct DescripcionDeSesion {
    sesion: SesionSdp,
    media: Vec<DescripcionDeMedia>,
}

impl DescripcionDeSesion {
    pub fn new(sesion: SesionSdp, media: Vec<DescripcionDeMedia>) -> Self {
        Self { sesion, media }
    }

    // getters
    pub fn get_sesion(&self) -> &SesionSdp {
        &self.sesion
    }

    pub fn get_medias(&self) -> &Vec<DescripcionDeMedia> {
        &self.media
    }

    /// Devuelve una referencia mutable al vector de descripciones de media, permitiendo modificarlo directamente
    pub fn get_medias_mut(&mut self) -> &mut Vec<DescripcionDeMedia> {
        &mut self.media
    }

    /// Retorna iterador sobre las desciripciones de media
    pub fn iter_medias(&self) -> impl Iterator<Item = &DescripcionDeMedia> {
        self.media.iter()
    }

    /// Retorna la cantidad de descripciones de media
    pub fn cantidad_medias(&self) -> usize {
        self.media.len()
    }

    /// Genera un SDP de tipo Offer con media básica de video.
    ///
    /// # Parámetros
    /// - `config`: Configuración local del peer que genera el offer.
    ///
    /// # Retorna
    /// Una nueva `DescripcionDeSesion` tipo offer lista para ser enviada al peer remoto.
    pub fn generar_offer(config: &ConfigRoomRTC) -> Self {
        let sesion = SesionSdp::new();

        // creamos una única media de video y con H264
        let mut medias = Vec::new();
        if let Some(video_media) = DescripcionDeMedia::crear_media_video_h264(
            config.getter_port_rtp_local(),
            0, // mid
        ) {
            medias.push(video_media);
        }
        // Creamos una media para el audio, con el formato estandar captando
        // por ALSA en linux
        medias.push(DescripcionDeMedia::crear_media_audio(
            config.getter_port_rtp_local(),
            1,
        ));

        Self {
            sesion,
            media: medias,
        }
    }

    /// Genera un answer a partir de un offer.
    ///
    /// Copia las secciones de media del offer recibido y crea nuevas instancias
    /// preparadas para representar la descripción de media de este peer.
    ///
    /// En esta etapa no se generan candidatos ICE ni se modifica la dirección de conexión.
    ///
    /// # Parámetros
    /// - `offer`: Descripción de sesión tipo offer recibida del peer remoto.
    /// - `_config`: Configuración local del peer (no se usa).
    ///
    /// # Retorna
    /// Una nueva `DescripcionDeSesion` tipo answer, con las medias clonadas y listas
    /// para registrar posteriormente candidatos o atributos adicionales.
    pub fn generar_answer_desde_offer(
        offer: &DescripcionDeSesion,
        _config: &ConfigRoomRTC,
    ) -> Self {
        let nuevas_medias: Vec<DescripcionDeMedia> = offer
            .get_medias()
            .iter()
            .map(Self::crear_media_answer)
            .collect();

        DescripcionDeSesion::new(SesionSdp::new(), nuevas_medias)
    }

    /// Crea una `DescripcionDeMedia` para el answer a partir de la media del offer.
    ///
    /// Esta función auxiliar encapsula la lógica de:
    /// - Copiar la DescripcionDeMedia del offer
    /// - Guardar candidatos remotos (los del offer)
    /// - Generar candidatos locales limpios (del peer que está generando el answer)
    fn crear_media_answer(media_offer: &DescripcionDeMedia) -> DescripcionDeMedia {
        let mut media_answer = media_offer.clone();

        let candidatos_offer = media_offer.get_candidatos_ice_locales().clone();

        media_answer.establecer_candidatos_remotos(candidatos_offer);
        media_answer.limpiar_candidatos_locales();

        media_answer
    }

    /// Guarda el mensaje SDP en un archivo.
    ///
    /// # Parámetros
    /// - `ruta`: Ruta del archivo donde guardar el SDP.
    ///
    /// # Retorna
    /// `io::Result<()>` indicando éxito o error en la operación de escritura.
    pub fn guardar_en_archivo(&self, ruta: &str) -> io::Result<()> {
        let mut escritor = BufWriter::new(File::create(ruta)?);
        escritor.write_all(self.serializar().as_bytes())?;
        let _ = escritor.flush();
        Ok(())
    }

    pub fn agregar_atributo_a_todas_las_medias(&mut self, atributo: String) {
        for media in self.get_medias_mut() {
            media.agregar_atributo(atributo.clone());
        }
    }

    pub fn buscar_fingerprint(&self) -> Option<String> {
        for media in self.get_medias() {
            for attr in media.get_atributos() {
                if attr.starts_with("a=fingerprint:") {
                    return Some(attr.replace("\r\n", ""));
                }
            }
        }
        None
    }

    pub fn buscar_setup(&self) -> Option<String> {
        let mut ultimo = None;

        for media in self.get_medias() {
            for attr in media.get_atributos() {
                if let Some(stripped) = attr.strip_prefix("a=setup:") {
                    ultimo = Some(stripped.trim().to_string());
                }
            }
        }

        ultimo
    }

    /// Obtiene los codecs comunes entre la oferta y la configuración local.
    pub fn obtener_codecs_comunes(
        media_offer: &DescripcionDeMedia,
        port_local: u16,
    ) -> Vec<String> {
        let codecs_offer = media_offer.extraer_codecs();

        let local_proto = DescripcionDeMedia::crear_media_video_h264(port_local, 0);

        let codecs_local = local_proto.map(|m| m.extraer_codecs());

        codecs_offer
            .into_iter()
            .filter(|c| {
                codecs_local
                    .as_ref()
                    .is_some_and(|vec| vec.iter().any(|lc| lc.eq_ignore_ascii_case(c)))
            })
            .collect()
    }

    /// Genera las descripciones de media para un answer basado en un offer.
    pub fn generar_medias_answer(
        offer: &DescripcionDeSesion,
        port_local: u16,
    ) -> Result<Vec<DescripcionDeMedia>, String> {
        let mut medias = Vec::new();

        for media_offer in offer.get_medias().iter() {
            let media_answer = if media_offer.es_media_soportada(port_local) {
                let comunes = Self::obtener_codecs_comunes(media_offer, port_local);
                if comunes.is_empty() {
                    DescripcionDeMedia::rechazar_media(media_offer)
                } else {
                    DescripcionDeMedia::preparar_media_answer(media_offer, &comunes, port_local)?
                }
            } else {
                DescripcionDeMedia::rechazar_media(media_offer)
            };

            medias.push(media_answer);
        }

        Ok(medias)
    }
}

impl SerializableSdp for DescripcionDeSesion {
    fn serializar(&self) -> String {
        let mut sdp_str = self.get_sesion().serializar();

        // generación de la línea group:BUNDLE con todos los mids
        let mids: Vec<String> = self
            .get_medias()
            .iter()
            .map(|media| media.get_mid().to_string())
            .collect();

        if !mids.is_empty() {
            sdp_str.push_str(&format!("a=group:BUNDLE {}\r\n", mids.join(" ")));
        }

        if let Some(bundle) = &self.get_sesion().get_linea_group_bundle() {
            sdp_str.push_str(&format!("a=group:BUNDLE {}\r\n", bundle));
        }

        // descrpiciones de media
        for desc_media in self.get_medias().iter() {
            sdp_str.push_str(&desc_media.serializar());
        }
        sdp_str
    }
}

impl ParseableSdp for DescripcionDeSesion {
    fn parsear(lineas: &[&str]) -> Result<Self, String> {
        let mut lineas_sesion = Vec::new();
        let mut lineas_media = Vec::new();
        let mut en_media = false;

        for &linea in lineas {
            if linea.starts_with("m=") {
                en_media = true;
            }

            if en_media {
                lineas_media.push(linea);
            } else {
                lineas_sesion.push(linea);
            }
        }

        let sesion = SesionSdp::parsear(&lineas_sesion)?;
        let media = DescripcionDeMedia::parsear_multiples(&lineas_media)?;

        Ok(Self { sesion, media })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_room_rtc::{ConfigRoomRTC, Direcciones};
    use std::fs;

    // genera un Config para usar en los tests y no tener que parsea un .conf
    fn generar_config() -> ConfigRoomRTC {
        let locales = Direcciones::new("192.168.1.1".to_string(), 9090);
        let remotas = Direcciones::new("192.168.1.1".to_string(), 9090);

        ConfigRoomRTC::crear_struct(
            locales,
            remotas,
            "/tmp/log.log".to_string(),
            "./sdp_test/offer.sdp".to_string(),
            "./sdp_test/answer.sdp".to_string(),
            "stun_server: stun.cloudflare.com:3478;".to_string(),
            "0.0.0.0:1818".to_string(),
        )
    }

    #[test]
    fn generar_offer_crea_mensaje_valido() {
        let config = generar_config();
        let offer = DescripcionDeSesion::generar_offer(&config);

        assert_eq!(offer.get_medias().len(), 2);
        assert_eq!(offer.get_medias()[0].get_tipo(), "video");
        assert!(!offer.get_sesion().get_origen().is_empty());
    }

    #[test]
    fn generar_answer_desde_offer_crea_answer_valido() {
        let config = generar_config();
        let offer = DescripcionDeSesion::generar_offer(&config);
        let answer = DescripcionDeSesion::generar_answer_desde_offer(&offer, &config);

        assert_eq!(answer.get_medias().len(), 2);
    }

    #[test]
    fn serializar_sdp_contiene_sesion_y_media() {
        let config = generar_config();
        let offer = DescripcionDeSesion::generar_offer(&config);
        let sdp = offer.serializar();

        assert!(sdp.contains("v="));
        assert!(sdp.contains("o="));
        assert!(sdp.contains("m=video"));
        assert!(sdp.contains("a=rtpmap:102 H264/90000"));
    }

    #[test]
    fn parsear_desde_lineas_reconstituye_mensaje() {
        let config = generar_config();
        let offer = DescripcionDeSesion::generar_offer(&config);
        let binding = offer.serializar();
        let lineas: Vec<&str> = binding.lines().collect();

        let sdp_parseado = DescripcionDeSesion::parsear(&lineas).unwrap();
        assert_eq!(sdp_parseado.get_medias().len(), offer.get_medias().len());
    }

    #[test]
    fn guardar_en_archivo_crea_archivo() {
        let config = generar_config();
        let offer = DescripcionDeSesion::generar_offer(&config);

        let ruta = "test_sdp_offer.sdp";
        let resultado = offer.guardar_en_archivo(ruta);
        assert!(resultado.is_ok());

        let contenido_sdp = fs::read_to_string(ruta).unwrap();
        assert!(contenido_sdp.contains("v="));
        assert!(contenido_sdp.contains("m=video"));

        let _ = fs::remove_file(ruta);
    }
}
