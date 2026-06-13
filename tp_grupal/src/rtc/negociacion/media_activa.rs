/// Representa una descripción de media aceptada por el peer B y que por ende tanto A como B pueden considerar al momento de buscar candidatos ICE
#[derive(Clone)]
pub struct MediaActiva {
    pub mid: String,
    pub tipo: String,
    pub puerto_local_rtp: u16,
    pub puerto_local_rtcp: u16,
    pub puerto_remoto_rtp: u16,
    pub puerto_remoto_rtcp: u16,
    pub candidatos_remotos: Vec<String>,
    pub candidatos_locales: Vec<String>,
}
