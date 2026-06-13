/// Demultiplexor post-ICE
/// Este módulo define el `DemuxPostIce`, que es responsable de recibir paquetes del socket UDP después de que el proceso de ICE ha concluido, y de
/// demultiplexar esos paquetes para enviarlos al canal correcto (DTLS o SRTP) dentro del demux. El `DemuxPostIce` utiliza patrones de bytes para determinar
/// el tipo de cada paquete entrante, y luego envía el paquete al canal correspondiente a través de `Sender`s. Además, incluye una señal especial para
/// permitir que el demux sea cerrado de manera ordenada.
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::thread;
const MATAR_DEMUX: &[u8] = b"__MATAR_DEMUX__";

pub struct DemuxPostIce {
    pub dtls_tx: Sender<Vec<u8>>,
    pub srtp_tx: Sender<Vec<u8>>,
}

fn es_dtls(p: &[u8]) -> bool {
    matches!(p.first().copied(), Some(20..=23)) // 0x14..0x17
}

fn es_rtp(buf: &[u8]) -> bool {
    buf.len() >= 2 && (buf[0] & 0xC0) == 0x80 && buf[1] < 192 // RTCP types típicos arrancan en 192+
}

fn es_stun(p: &[u8]) -> bool {
    if p.len() < 20 {
        return false;
    }
    // primeros dos bytes: mensaje de solicitud o respuesta (0x0001, 0x0101, etc)
    if (p[0] & 0b1100_0000) != 0 {
        return false;
    }
    // magic cookie 0x2112A442 at bytes 4..8
    p[4..8] == [0x21, 0x12, 0xA4, 0x42]
}

pub fn spawnear_demux_post_ice(
    recv_socket: UdpSocket,
    demux: DemuxPostIce,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            let (longitud, _from) = match recv_socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let paquete = &buf[..longitud];

            if paquete == MATAR_DEMUX {
                break;
            }

            let paquete_vectorizado = paquete.to_vec();

            if paquete == b"END" {
                //lo pongo como caso del if xq va al srtp pero no entra en el estandar al ser un END
                let _ = demux.srtp_tx.send(paquete_vectorizado);
            } else if es_dtls(&paquete_vectorizado) {
                if demux.dtls_tx.send(paquete_vectorizado).is_err() {
                    break;
                }
            } else if es_rtp(&paquete_vectorizado) {
                if demux.srtp_tx.send(paquete_vectorizado).is_err() {
                    // -> misma logica q lo q habia dicho en es_dtls
                    break;
                }
            } else if es_stun(&paquete_vectorizado) {
                eprintln!("Soy un paquete STUN, lo ignoro");
                continue;
            } else {
                // paquete desconocido, vemos si ignoramos o tiramos error
            }
        }
    })
}
