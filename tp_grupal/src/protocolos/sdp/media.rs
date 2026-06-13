//! Módulo `media`
//!
//! Contiene la estructura `DescripcionDeMedia` que representa un bloque de media
//! en un SDP, con líneas `m=` y atributos `a=`.
//!
//! Proporciona:
//! - Constructor genérico `new` y wrappers predefinidos (`video_basico`).
//! - Métodos para serializar a SDP y parsear desde líneas.
//! - Función auxiliar `parsear_multiples` para manejar varias secciones de media.
use crate::protocolos::sdp::traits::ParseableSdp;

use super::traits::SerializableSdp;

/// Representa una sección de media (líneas m= y sus atributos a=)
/// Los atributos que no son candidatos ICE se mantienen en `atributos`.
/// Los candidatos ICE locales se generan por separado en `candidatos_ice_locales`.
/// Los candidatos remotos se agregan en `candidatos_ice_remotos`.
#[derive(Debug, Clone)]
pub struct DescripcionDeMedia {
    tipo: String,
    puerto: u16,
    puerto_rtcp: u16,
    protocolo: String,
    formato: u8,
    mid: u8,
    atributos: Vec<String>,
    candidatos_ice_locales: Vec<String>,
    candidatos_ice_remotos: Vec<String>,
}

impl DescripcionDeMedia {
    /// Constructor genérico de un bloque de media (m=)
    pub fn new(
        tipo: &str,
        puerto: u16,
        puerto_rtcp: u16,
        protocolo: &str,
        formato: u8,
        mid: u8,
        atributos: Vec<String>,
    ) -> Self {
        DescripcionDeMedia {
            tipo: tipo.to_string(),
            puerto,
            puerto_rtcp,
            protocolo: protocolo.to_string(),
            formato,
            mid,
            atributos,
            candidatos_ice_locales: Vec::new(),
            candidatos_ice_remotos: Vec::new(),
        }
    }

    // getters
    pub fn get_tipo(&self) -> &str {
        &self.tipo
    }

    pub fn get_puerto(&self) -> u16 {
        self.puerto
    }

    pub fn get_puerto_rtcp(&self) -> u16 {
        self.puerto_rtcp
    }

    pub fn get_protocolo(&self) -> &str {
        &self.protocolo
    }

    pub fn get_formato(&self) -> u8 {
        self.formato
    }

    pub fn get_mid(&self) -> u8 {
        self.mid
    }

    pub fn get_atributos(&self) -> &[String] {
        &self.atributos
    }

    pub fn get_candidatos_ice_locales(&self) -> &Vec<String> {
        &self.candidatos_ice_locales
    }

    pub fn get_candidatos_ice_remotos(&self) -> &Vec<String> {
        &self.candidatos_ice_remotos
    }

    // setters
    pub fn settear_puerto_a_cero(&mut self) {
        self.puerto = 0;
    }

    pub fn establecer_candidatos_remotos(&mut self, candidatos: Vec<String>) {
        self.candidatos_ice_remotos = candidatos;
    }

    pub fn limpiar_candidatos_locales(&mut self) {
        self.candidatos_ice_locales.clear();
    }

    pub fn agregar_candidato_local(&mut self, candidato: String) {
        if !self.candidatos_ice_locales.iter().any(|c| c == &candidato) {
            self.candidatos_ice_locales.push(candidato);
        }
    }

    /// Reemplaza todos los atributos (para limpiar candidatos)
    pub fn establecer_atributos(&mut self, nuevos_atributos: Vec<String>) {
        self.atributos = nuevos_atributos;
    }

    pub fn crear_media_video_h264(puerto_base: u16, mid: u8) -> Option<Self> {
        let tipo = "video";
        let protocolo = "RTP/AVP";
        let payload = 102;
        let puerto_rtp = puerto_base;
        let puerto_rtcp = puerto_base + 1;
        let atributos = vec![
            "a=sendrecv".to_string(),
            "a=rtpmap:102 H264/90000".to_string(),
        ];

        Some(Self::new(
            tipo,
            puerto_rtp,
            puerto_rtcp,
            protocolo,
            payload,
            mid,
            atributos,
        ))
    }

    pub fn crear_media_audio(puerto: u16, mid: u8) -> Self {
        let tipo = "audio";
        let protocolo = "RTP/AVP";
        // 11: Numero de RTP Payload type para audio PCM lineal de 16 bits,
        // sin comprimir  (el que capta ALSA en linux por defecto)
        let payload = 11;
        let puerto_rtp = puerto;
        let puerto_rtcp = puerto + 1;
        let atributos = vec!["a=sendrecv".to_string()];

        Self::new(
            tipo,
            puerto_rtp,
            puerto_rtcp,
            protocolo,
            payload,
            mid,
            atributos,
        )
    }

    /// Obtiene la lista de nombres de códecs declarados en esta descripción de media.
    ///
    /// Se analizan los atributos `a=rtpmap:` y se extraen los nombres normalizados en minúsculas.
    pub fn extraer_codecs(&self) -> Vec<String> {
        self.get_atributos()
            .iter()
            .filter_map(|linea| {
                if !linea.starts_with("a=rtpmap:") {
                    return None;
                }
                let partes: Vec<&str> = linea.split_whitespace().collect();
                if partes.len() < 2 {
                    return None;
                }
                partes[1].split('/').next().map(|s| s.to_lowercase())
            })
            .collect()
    }

    /// Filtra los codecs de esta media según `codecs_comunes`, preservando los demás atributos.
    pub fn filtrar_codecs(&mut self, codecs_comunes: &[String]) {
        let mut nuevos = Vec::new();
        let mut nuevo_formato = 0u8;
        for atributo in &self.atributos {
            if let Some(resto) = atributo.strip_prefix("a=rtpmap:") {
                if let Some((payload, codec)) = resto.split_once(' ') {
                    let payload = payload.trim();
                    let codec = codec.trim();

                    if let Some(nombre_codec) = codec.split('/').next()
                        && codecs_comunes
                            .iter()
                            .any(|c| c.eq_ignore_ascii_case(nombre_codec))
                    {
                        nuevos.push(atributo.clone());

                        if nuevo_formato == 0
                            && let Ok(valor) = payload.parse::<u8>()
                        {
                            nuevo_formato = valor;
                        }
                    }
                }
            } else {
                nuevos.push(atributo.clone());
            }
        }
        self.establecer_atributos(nuevos);
        self.formato = nuevo_formato;
    }

    /// A partir de las líneas (strings) del vector de candidatos que obtenemos del answer.sdp, llenamos el campo candidatos_remotos
    pub fn agregar_candidatos_remotos(&mut self, lineas_candidatos: &Vec<String>) {
        for linea in lineas_candidatos {
            self.candidatos_ice_remotos.push(linea.clone());
        }
    }

    /// Auxiliar que parsea múltiples secciones de media a partir de líneas SDP.
    ///
    /// # Parámetros
    /// - `lineas`: Slice de líneas de texto SDP.
    ///
    /// # Retorna
    /// - `Ok(Vec<DescripcionDeMedia>)`: Vector de descripciones de media parseadas.
    /// - `Err(String)`: Mensaje de error si el parseo falla.
    pub fn parsear_multiples(lineas: &[&str]) -> Result<Vec<Self>, String> {
        let mut secciones = Vec::new();
        let mut actual = Vec::new();

        for &linea in lineas {
            if linea.starts_with("m=") && !actual.is_empty() {
                secciones.push(std::mem::take(&mut actual));
            }
            actual.push(linea);
        }
        if !actual.is_empty() {
            secciones.push(actual);
        }

        let mut resultado = Vec::new();
        for seccion in secciones {
            resultado.push(Self::parsear(&seccion)?);
        }
        Ok(resultado)
    }

    /// Extrae el clock rate del atributo `a=rtpmap` si está presente.
    /// Devuelve `None` si no se encuentra o no se puede parsear.
    pub fn obtener_clock_rate(&self) -> Option<u32> {
        for atributo in &self.atributos {
            if atributo.starts_with("a=rtpmap:") {
                let partes: Vec<&str> = atributo.split_whitespace().collect();
                if partes.len() == 2 {
                    let rtpmap_partes: Vec<&str> = partes[1].split('/').collect();
                    if rtpmap_partes.len() == 2
                        && let Ok(clock_rate) = rtpmap_partes[1].parse::<u32>()
                    {
                        return Some(clock_rate);
                    }
                }
            }
        }
        None
    }

    /// Agrega un nuevo atributo `a=` a la descripción de media.
    pub fn agregar_atributo(&mut self, atributo: String) {
        self.atributos.push(atributo);
    }

    /// Parsea el nombre del codec desde una línea `a=rtpmap:`.
    /// Devuelve `None` si la línea no es válida o no es un `rtpmap`.
    pub fn parsear_codec_desde_linea(linea: &str) -> Option<String> {
        if !linea.starts_with("a=rtpmap:") {
            return None;
        }

        let partes: Vec<&str> = linea.split_whitespace().collect();
        if partes.len() < 2 {
            return None;
        }

        let codec_parte = partes[1].split('/').next()?;
        Some(codec_parte.to_lowercase())
    }

    /// Verifica si dos listas de codecs tienen al menos uno en común (ignorando mayúsculas).
    pub fn tienen_codec_en_comun(a: &[String], b: &[String]) -> bool {
        a.iter()
            .any(|codec| b.iter().any(|c| c.eq_ignore_ascii_case(codec)))
    }

    /// Verifica si esta media de tipo video es soportada localmente (H264).
    pub fn es_media_soportada(&self, port_local: u16) -> bool {
        if self.get_tipo() != "video" {
            return false;
        }

        let codecs_offer = self.extraer_codecs();

        let codecs_local =
            if let Some(local_proto) = DescripcionDeMedia::crear_media_video_h264(port_local, 0) {
                local_proto.extraer_codecs()
            } else {
                vec![]
            };

        DescripcionDeMedia::tienen_codec_en_comun(&codecs_offer, &codecs_local)
    }

    /// Genera una descripción de media que rechaza la oferta (puerto 0, sin candidatos).
    pub fn rechazar_media(media_offer: &DescripcionDeMedia) -> DescripcionDeMedia {
        let mut m = media_offer.clone();
        m.settear_puerto_a_cero();
        m.limpiar_candidatos_locales();
        m.establecer_candidatos_remotos(vec![]);
        m
    }

    /// Prepara una descripción de media para la respuesta, basada en la oferta y los codecs comunes.
    pub fn preparar_media_answer(
        media_offer: &DescripcionDeMedia,
        codecs_comunes: &[String],
        port_local: u16,
    ) -> Result<DescripcionDeMedia, String> {
        let tipo = media_offer.get_tipo();
        let mid = media_offer.get_mid();

        let mut media_answer = match tipo {
            "video" => DescripcionDeMedia::crear_media_video_h264(port_local, mid)
                .ok_or("Error creando media local base para video")?,
            _ => {
                return Err(format!("Tipo de media no soportado: {}", tipo));
            }
        };

        media_answer.filtrar_codecs(codecs_comunes);

        if media_offer.get_puerto() != 0 {
            let candidatos = media_offer.get_candidatos_ice_locales().clone();
            media_answer.establecer_candidatos_remotos(candidatos);
        }

        media_answer.limpiar_candidatos_locales();

        Ok(media_answer)
    }

    pub fn limpiar_setup(&mut self) {
        self.atributos.retain(|a| !a.starts_with("a=setup:"));
    }
}

impl SerializableSdp for DescripcionDeMedia {
    fn serializar(&self) -> String {
        let mut payloads: Vec<u8> = self
            .atributos
            .iter()
            .filter_map(|a| a.strip_prefix("a=rtpmap:"))
            .filter_map(|r| r.split_once(' '))
            .filter_map(|(p, _)| p.trim().parse::<u8>().ok())
            .collect();

        payloads.sort();
        payloads.dedup();
        if payloads.is_empty() && self.formato != 0 {
            payloads.push(self.formato);
        }
        let mut sdp = format!(
            "m={} {} {} {}\r\n",
            self.tipo,
            self.puerto,
            self.protocolo,
            payloads
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );

        sdp.push_str(&format!("a=mid:{}\r\n", self.mid));
        sdp.push_str(&format!("a=rtcp:{}\r\n", self.puerto_rtcp));

        for a in &self.atributos {
            sdp.push_str(a);
            sdp.push_str("\r\n");
        }
        for c in &self.candidatos_ice_locales {
            sdp.push_str(c);
            sdp.push_str("\r\n");
        }
        sdp
    }
}

impl ParseableSdp for DescripcionDeMedia {
    /// Parsea un bloque de líneas SDP de media.
    /// Devuelve error si la línea `m=` está incompleta o los números no se pueden parsear.
    fn parsear(lineas: &[&str]) -> Result<Self, String> {
        let (mut tipo, mut puerto, mut protocolo, mut formato) =
            (String::new(), 0, String::new(), 0);
        let mut candidatos_ice_locales: Vec<String> = Vec::new();
        let mut atributos = Vec::new();
        let mut mid: u8 = 0;
        let mut puerto_rtcp: u16 = 0;

        for &linea in lineas {
            if let Some((clave, valor)) = linea.trim().split_once('=') {
                match clave {
                    "m" => {
                        let partes: Vec<&str> = valor.split_whitespace().collect();
                        if partes.len() < 4 {
                            return Err("Línea 'm=' incompleta".to_string());
                        }
                        tipo = partes[0].to_string();
                        puerto = partes[1]
                            .parse()
                            .map_err(|e| format!("Puerto inválido: {}", e))?;
                        protocolo = partes[2].to_string();
                        formato = partes[3]
                            .parse()
                            .map_err(|e| format!("Formato inválido: {}", e))?;
                    }
                    "a" => {
                        if let Some(resto) = valor.strip_prefix("mid:") {
                            mid = resto.parse().map_err(|e| format!("MID inválido: {}", e))?;
                        } else if let Some(resto) = valor.strip_prefix("rtcp:") {
                            puerto_rtcp = resto
                                .parse()
                                .map_err(|e| format!("Puerto RTCP inválido: {}", e))?;
                        } else if valor.starts_with("candidate:") {
                            // mover directamente a candidatos_ice_locales
                            candidatos_ice_locales.push(format!("a={}", valor));
                        } else {
                            atributos.push(format!("a={}", valor));
                        }
                    }
                    _ => {}
                }
            }

            // si no viene puerto_rtcp explícito, va puerto + 1
            if puerto_rtcp == 0 {
                puerto_rtcp = puerto + 1;
            }
        }

        let mut media = Self::new(
            &tipo,
            puerto,
            puerto_rtcp,
            &protocolo,
            formato,
            mid,
            atributos,
        );
        media.candidatos_ice_locales = candidatos_ice_locales;
        Ok(media)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_asigna_campos_correctamente() {
        let atributos: Vec<String> = vec!["a=1".to_string()];
        let media =
            DescripcionDeMedia::new("video", 1234, 1235, "RTP/AVP", 96, 0, atributos.clone());

        assert_eq!(media.tipo, "video");
        assert_eq!(media.puerto, 1234);
        assert_eq!(media.protocolo, "RTP/AVP");
        assert_eq!(media.formato, 96);
        assert_eq!(media.atributos, atributos);
    }

    #[test]
    fn crea_descripcion_video_valida() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
        ];
        let media =
            DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos.clone());

        assert_eq!(media.tipo, "video");
        assert_eq!(media.puerto, 5000);
        assert_eq!(media.protocolo, "RTP/AVP");
        assert_eq!(media.formato, 96);
        assert_eq!(media.atributos, atributos);
    }

    #[test]
    fn crear_media_video_h264_funciona() {
        let puerto_base = 6000;
        let mid = 1;
        let media = DescripcionDeMedia::crear_media_video_h264(puerto_base, mid).unwrap();

        assert_eq!(media.tipo, "video");
        assert_eq!(media.puerto, puerto_base);
        assert_eq!(media.puerto_rtcp, puerto_base + 1);
        assert_eq!(media.protocolo, "RTP/AVP");
        assert_eq!(media.formato, 102);
        assert_eq!(media.mid, mid);
        assert!(
            media
                .atributos
                .contains(&"a=rtpmap:102 H264/90000".to_string())
        );
    }

    #[test]
    fn extraer_codecs_devuelve_lista_correcta() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
            "a=rtpmap:97 H264/90000".to_string(),
        ];
        let media = DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos);

        let codecs = media.extraer_codecs();
        assert_eq!(codecs.len(), 2);
        assert!(codecs.contains(&"vp8".to_string()));
        assert!(codecs.contains(&"h264".to_string()));
    }

    #[test]
    fn filtrar_codecs_elimina_no_comunes() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
            "a=rtpmap:97 H264/90000".to_string(),
        ];
        let mut media = DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos);

        let codecs_comunes = vec!["h264".to_string()];
        media.filtrar_codecs(&codecs_comunes);

        let codecs_filtrados = media.extraer_codecs();
        assert_eq!(codecs_filtrados.len(), 1);
        assert!(codecs_filtrados.contains(&"h264".to_string()));
        assert!(!codecs_filtrados.contains(&"vp8".to_string()));
    }

    #[test]
    fn rechazar_media_puerto_cero() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
        ];
        let media_offer = DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos);

        let media_rechazada = DescripcionDeMedia::rechazar_media(&media_offer);

        assert_eq!(media_rechazada.get_puerto(), 0);
        assert!(media_rechazada.get_candidatos_ice_locales().is_empty());
        assert!(media_rechazada.get_candidatos_ice_remotos().is_empty());
    }

    #[test]
    fn serializar_sdp_contiene_lineas_m_y_a() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
        ];
        let media =
            DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos.clone());

        let sdp = media.serializar();

        assert!(sdp.starts_with("m=video 5000 RTP/AVP 96"));
        for a in &media.atributos {
            assert!(sdp.contains(a));
        }
    }

    #[test]
    fn parsear_lineas_reconstruye_media() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
        ];
        let media =
            DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos.clone());

        let sdp = media.serializar();
        let sdp_lineas: Vec<&str> = sdp.lines().collect();
        let sdp_parseado = DescripcionDeMedia::parsear(&sdp_lineas).unwrap();

        assert_eq!(sdp_parseado.tipo, media.tipo);
        assert_eq!(sdp_parseado.puerto, media.puerto);
        assert_eq!(sdp_parseado.formato, media.formato);
        assert_eq!(sdp_parseado.atributos, media.atributos);
    }

    #[test]
    fn parsear_multiples_descripciones() {
        let atributos = vec![
            "a=rtcp-mux".to_string(),
            "a=sendrecv".to_string(),
            "a=rtpmap:96 VP8/90000".to_string(),
        ];
        let media1 =
            DescripcionDeMedia::new("video", 5000, 5001, "RTP/AVP", 96, 0, atributos.clone());
        let media2 =
            DescripcionDeMedia::new("video", 5002, 5003, "RTP/AVP", 96, 1, atributos.clone());
        let mut lineas = Vec::new();

        let sdp1 = media1.serializar();
        let sdp2 = media2.serializar();

        lineas.extend(sdp1.lines());
        lineas.extend(sdp2.lines());

        let sdp_media_parseado = DescripcionDeMedia::parsear_multiples(&lineas).unwrap();
        assert_eq!(sdp_media_parseado.len(), 2);
        assert_eq!(sdp_media_parseado[0].tipo, "video");
        assert_eq!(sdp_media_parseado[1].tipo, "video");
    }
}
