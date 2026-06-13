#[derive(Debug)]
pub enum ErrorJitterBuffer {
    ObteniendoLock,
    EnviandoPorChannel,
    SinInformacionVideo,
    SinInformacionAudio,
}
