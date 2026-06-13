/// Canal de comunicación para DTLS dentro del demux
/// Este módulo define el `DemuxDtlsChannel`, que es un canal de comunicación específico para manejar la comunicación DTLS dentro del demux.
/// Este canal utiliza un socket UDP para enviar datos al peer remoto, y un `Receiver` para recibir datos que han sido procesados por el contexto DTLS
use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;

#[derive(Debug)]
pub struct DemuxDtlsChannel {
    pub send_socket: UdpSocket,
    pub remote: SocketAddr,
    pub dtls_rx: Receiver<Vec<u8>>,
}

impl io::Read for DemuxDtlsChannel {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let data = match self
            .dtls_rx
            .recv_timeout(std::time::Duration::from_millis(100))
        {
            Ok(data) => data,
            Err(RecvTimeoutError::Timeout) => {
                return Err(io::Error::new(io::ErrorKind::WouldBlock, "timeout"));
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "canal cerrado"));
            }
        };

        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        Ok(n)
    }
}

impl Write for DemuxDtlsChannel {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send_socket.send_to(buf, self.remote)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
