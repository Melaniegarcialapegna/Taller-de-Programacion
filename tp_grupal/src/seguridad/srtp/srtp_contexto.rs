//! Módulo 'srtp_contexto.rs'
//! Este módulo define la estructura y funciones necesarias para manejar el contexto SRTP (Secure Real-time Transport Protocol)
//! en el contexto de DTLS (Datagram Transport Layer Security).

use std::collections::HashMap;

use crate::seguridad::srtp::errores::ErrorSRTP;
use crate::seguridad::srtp::srtp_clave_salt::SRTPClaveSalt;
use openssl::{
    hash::MessageDigest,
    pkey::PKey,
    sign::Signer,
    symm::{Cipher, Crypter, Mode},
};

#[derive(Debug, Clone, Default)]
pub struct EstadisticasSRTP {
    pub roc: u32, // rollover counter -> es un contador de cuántas veces se desbordó la secuencia de 16 bits del RTP, srtp no usa secuencias de 32
    //bits sino que usa el seq de rtp (un u16 que va de 0 a 65535) y cada vez que se desborda (pasa de 65535 a 0) se incrementa este contador
    pub seq_mas_alto: u16, // secuencia más alta vista hasta ahora (u16) -> lo usamos para ver si hubo rollover o reordenamiento (comparamos valores con seq_recibido)
    pub replay_window: u64, // ventana de reproducción para evitar ataques de repetición -> comparación de indices de paquetes recibidos
}

/// Struct que encapsula el contexto SRTP.
#[derive(Debug, Clone)]
pub struct SRTPContexto {
    pub clave_salt: SRTPClaveSalt, // las claves y salt para SRTP (rx o tx) que obtuvimos del handshake DTLS
    pub estadisticas_ssrc: HashMap<u32, EstadisticasSRTP>,
}

/// Implementación de métodos para SRTPContexto.
impl SRTPContexto {
    /// Crea un nuevo contexto SRTP con la clave y el salt proporcionados.
    pub fn new(clave_srtp: Vec<u8>, salt: Vec<u8>) -> Self {
        SRTPContexto {
            clave_salt: SRTPClaveSalt {
                clave: clave_srtp,
                salt,
            },
            estadisticas_ssrc: HashMap::new(),
        }
    }

    fn calcular_indice_paquete(&mut self, ssrc: u32, seq_recibido: u16) -> Result<u64, ErrorSRTP> {
        let estadisticas = self.obtener_estadisticas_srtp(ssrc)?;

        let seq_mas_alto = estadisticas.seq_mas_alto;
        let mut roc_candidato = estadisticas.roc;

        // Caso 1: Secuencia nueva y mayor (normal)
        // hacemos seq_recibido > seq_mas_alto porque:
        // - el número de secuencia aumentó como es debido.
        // - no hubo rollover (rollover solo ocurre cuando seq baja)
        if seq_recibido > seq_mas_alto {
            // mismo roc que antes
            // RESPONSABILIDAD DE CAMBIO EN OTRA FUNCION
        }
        // Caso 2: rollover ascendente válido
        // - si la diferencia es MAYOR que la mitad del espacio circular, es muy improbable que estés recibiendo un paquete retrasado
        // -> es mucho más probable que el contador pasó de 65535 a 0.
        // - no es reordenamiento porque seq_recibido es menor que seq_mas_alto
        else if seq_mas_alto.wrapping_sub(seq_recibido) > 32768 {
            // 32768 es la mitad de 65536
            roc_candidato = roc_candidato.wrapping_add(1);
            // Responsabilidad de cambio en otra funcion
        }
        // Caso 3: reordenamiento normal (jitter, retrasos)
        // → no actualizamos seq_mas_alto ni roc, solo usamos los valores actuales

        // Construir índice final de 48 bits: combinamos un seq de 16 bits con el roc de 32 bits para generar un índice de paquete único de 48 bits
        Ok(((roc_candidato as u64) << 16) | (seq_recibido as u64))

        // uso roc_candidato en vwz de self.roc porque en este punto no actualizamos nada (sino explota)
    }

    fn obtener_estadisticas_srtp(&mut self, ssrc: u32) -> Result<&mut EstadisticasSRTP, ErrorSRTP> {
        let estadisticas = self.estadisticas_ssrc.entry(ssrc).or_default();
        Ok(estadisticas)
    }

    fn derivar_iv(&self, ssrc: u32, indice_paquete: u64) -> Result<[u8; 16], ErrorSRTP> {
        // IV es un valor de 128 bits donde ciertas posiciones se llenan con SSRC e índice, con dos bits menos significativos
        // en cero, y luego se XOR con el “salting key” (nuestro salt de 112 bits).
        // - Los primeros bits (salt) dan aleatoriedad por sesión.
        // - Los campos con `SSRC` e `index` hacen que cambie por flujo y por paquete.
        // - Dejar algunos bits en 0 garantiza ciertas propiedades de seguridad (evitar colisiones en IVs, especialmente con mismo salt).
        // tooodo lo que se hace en esta funcion toma como base la forma de armar IV -> IV = SALT XOR (SSRC || indice_paquete)
        let mut iv = [0u8; 16];
        // Aseguramos que el salt tenga 14 bytes
        if self.clave_salt.salt.len() != 14 {
            Err(ErrorSRTP::ErrorClaveSaltInvalida(
                "El salt debe tener exactamente 14 bytes".to_string(),
            ))?
        }
        // copio salt en bytes [2..16)
        // los primeros 2 bytes quedan en 0
        iv[2..16].copy_from_slice(&self.clave_salt.salt);
        // XOR con SSRC en [4..8)
        let ssrc_big_endian = ssrc.to_be_bytes(); // 4 bytes
        for i in 0..4 {
            iv[4 + i] ^= ssrc_big_endian[i];
        }
        // XOR con indice_paquete en [8..16)
        let index_big_endian = indice_paquete.to_be_bytes(); // 8 bytes
        for i in 0..8 {
            iv[8 + i] ^= index_big_endian[i]; //^= es XOR combinado con asignar variable
        }
        Ok(iv)
    }

    fn proteger_rtp(&mut self, ssrc: u32, paquete: &mut [u8]) -> Result<(), ErrorSRTP> {
        Self::chequear_minimo_bytes_paquete(paquete)?;
        let seq = u16::from_be_bytes([paquete[2], paquete[3]]); // leo el número de secuencia del header RTP (bytes 2 y 3)
        let indice_paquete = self.calcular_indice_paquete(ssrc, seq)?; // calculo el packet index usando la secuencia
        let iv = self.derivar_iv(ssrc, indice_paquete)?; // derivo el IV para este paquete
        let header_len = 12; // longitud fija del header RTP
        let (_header, payload) = paquete.split_at_mut(header_len); // separo header y payload
        let cipher = Cipher::aes_128_ctr(); // configuro cifrador AES-128-CTR
        let mut crypter = self.crear_crypter_aes_ctr(&iv, Mode::Encrypt)?; // creamos el crypter para cifrar
        let mut out_buf = vec![0u8; payload.len() + cipher.block_size()]; // buffer temporal para el payload cifrado

        // AES-CTR es un modo de cifra tipo stream: no usa padding y la cantidad de bytes cifrados es exactamente igual a la del payload. update() escribe el resultado directamente en out_buf.
        let count = crypter
            .update(payload, &mut out_buf)
            .map_err(|_| ErrorSRTP::ErrorCifradoAES("Error en update() de AES-CTR".to_string()))?;
        // En AES-CTR, finalize() no agrega más datos (retorna 0), pero debe llamarse para completar el ciclo del Crypter.
        let rest = crypter.finalize(&mut out_buf[count..]).map_err(|_| {
            ErrorSRTP::ErrorCifradoAES("Error en finalize() de AES-CTR".to_string())
        })?;
        let total = count + rest;
        out_buf.truncate(total);
        Self::asegurar_estabilidad_longitud_payload_post_operaciones_encriptacion(
            out_buf.len(),
            payload.len(),
        )?;
        payload.copy_from_slice(&out_buf); // escribo el payload cifrado de vuelta en el paquete RTP

        Ok(())
    }

    fn chequear_minimo_bytes_paquete(paquete: &[u8]) -> Result<(), ErrorSRTP> {
        if paquete.len() < 12 {
            return Err(ErrorSRTP::ErrorPaqueteDemasiadoCorto(
                "RTP header debe tener al menos 12 bytes".to_string(),
            ));
        }
        Ok(())
    }

    fn asegurar_clave_longitud_igual_a_cipher(
        clave: &[u8],
        cipher: &Cipher,
    ) -> Result<(), ErrorSRTP> {
        if clave.len() != cipher.key_len() {
            return Err(ErrorSRTP::ErrorClaveSaltInvalida(format!(
                "Longitud de clave SRTP inválida: se esperaban {} bytes, se recibieron {}",
                cipher.key_len(),
                clave.len()
            )));
        }
        Ok(())
    }

    fn crear_crypter_aes_ctr(&self, iv: &[u8], mode: Mode) -> Result<Crypter, ErrorSRTP> {
        let cipher = Cipher::aes_128_ctr();

        Self::asegurar_clave_longitud_igual_a_cipher(&self.clave_salt.clave, &cipher)?;

        let mut crypter =
            Crypter::new(cipher, mode, &self.clave_salt.clave, Some(iv)).map_err(|_| {
                let accion = match mode {
                    Mode::Encrypt => "cifrado",
                    Mode::Decrypt => "descifrado",
                };
                ErrorSRTP::ErrorCifradoAES(format!(
                    "No se pudo crear Crypter AES-CTR para {}",
                    accion
                ))
            })?;

        // En CTR no hay padding -> desactivamos
        crypter.pad(false);

        Ok(crypter)
    }

    fn asegurar_estabilidad_longitud_payload_post_operaciones_encriptacion(
        out_buf_len: usize,
        payload_len: usize,
    ) -> Result<(), ErrorSRTP> {
        if out_buf_len != payload_len {
            return Err(ErrorSRTP::ErrorCifradoAES(
                "AES-CTR devolvió longitud inesperada".to_string(),
            ));
        }
        Ok(())
    }

    fn generar_tag_autenticacion(
        &mut self,
        ssrc: u32,
        paquete: &[u8],
    ) -> Result<[u8; 10], ErrorSRTP> {
        let roc_bytes;
        {
            let estadisticas = self.obtener_estadisticas_srtp(ssrc)?;
            roc_bytes = estadisticas.roc.to_be_bytes();
        }
        let mut signer = self.crear_signer_hmac_sha1()?;

        // el input del MAC es: RTP (header+payload cifrado) || ROC
        signer.update(paquete).map_err(|_| {
            ErrorSRTP::ErrorHMAC("Error en update() de HMAC-SHA1 (RTP)".to_string())
        })?;

        signer.update(&roc_bytes).map_err(|_| {
            ErrorSRTP::ErrorHMAC("Error en update() de HMAC-SHA1 (ROC)".to_string())
        })?;

        let mac_completo = signer
            .sign_to_vec()
            .map_err(|_| ErrorSRTP::ErrorHMAC("Error en sign_to_vec() de HMAC-SHA1".to_string()))?;

        // verifico que haya al menos 10 bytes para poder generar el tag de 80 bits
        Self::verificacion_al_menos_10_bytes(&mac_completo)?;

        // solo tomo los primeros 80 bits (10 bytes que pedi en verificacion)
        let mut tag80 = [0u8; 10];
        tag80.copy_from_slice(&mac_completo[..10]);

        Ok(tag80)
    }

    fn crear_signer_hmac_sha1(&'_ self) -> Result<Signer<'_>, ErrorSRTP> {
        let clave = PKey::hmac(&self.clave_salt.clave).map_err(|_| {
            ErrorSRTP::ErrorHMAC("No se pudo crear PKey HMAC para SRTP".to_string())
        })?;

        let signer = Signer::new(MessageDigest::sha1(), &clave).map_err(|_| {
            // objeto que calcula HMAC-SHA1
            ErrorSRTP::ErrorHMAC("No se pudo crear Signer HMAC-SHA1 para SRTP".to_string())
        })?;

        Ok(signer)
    }

    fn verificacion_al_menos_10_bytes(tag: &[u8]) -> Result<(), ErrorSRTP> {
        if tag.len() < 10 {
            return Err(ErrorSRTP::ErrorHMAC(
                "Tag de autenticación SRTP debe tener al menos 10 bytes".to_string(),
            ));
        }
        Ok(())
    }

    // protege un paquete RTP completo:
    // 1. actualiza índices (seq, ROC)
    // 2. cifra el payload con AES-CTR
    // 3. calculamos HMAC-SHA1-80 y lo agregamos al final del Vec (SRTP)

    /// Protege y firma un paquete RTP para convertirlo en SRTP.
    ///
    /// # Arguments
    /// * `ssrc` - El SSRC del flujo RTP.
    /// * `paquete` - El paquete RTP a proteger y firmar.
    ///
    /// # Returns
    /// * `Result<(), ErrorSRTP>` - Resultado de la operación, con error en caso de fallo.
    pub fn proteger_y_firmar_rtp(
        &mut self,
        ssrc: u32,
        paquete: &mut Vec<u8>,
    ) -> Result<(), ErrorSRTP> {
        // ciframos el payload
        self.proteger_rtp(ssrc, paquete.as_mut_slice())?;
        // generamos tag de autenticación HMAC-SHA1-80
        let tag = self.generar_tag_autenticacion(ssrc, paquete)?;
        // agregamos el tag al final del paquete -> queda SRTP
        paquete.extend_from_slice(&tag);

        Ok(())
    }

    fn verificar_replay_y_actualizar(
        &mut self,
        ssrc: u32,
        index_recibido: u64,
    ) -> Result<(), ErrorSRTP> {
        let roc: u32;
        let seq_mas_alto: u16;
        let replay_window: u64;
        {
            let estadisticas = self.obtener_estadisticas_srtp(ssrc)?;
            roc = estadisticas.roc;
            seq_mas_alto = estadisticas.seq_mas_alto;
            replay_window = estadisticas.replay_window;
        }

        let index_mas_alto = ((roc as u64) << 16) | (seq_mas_alto as u64);

        // caso 1: index_recibido nuevo y mayor que index_mas_alto
        // - actualizamos la ventana de replay
        // - actualizamos index_mas_alto (seq_mas_alto y roc)
        if index_recibido > index_mas_alto {
            self.actualizar_por_paquete_nuevo(ssrc, index_recibido, index_mas_alto)?;
            return Ok(());
        }

        // caso 2: index_recibido demasiado viejo (fuera de ventana)
        // si está más de 63 por debajo del highest, rechazamos -> es un replay o un paquete que se quedo muy atras
        let desplazamiento_viejo = (index_mas_alto - index_recibido) as u32;
        self.verificar_paquete_viejo(desplazamiento_viejo)?;

        // caso 3: dentro de ventana: índice válido o reordenado
        let desplazamiento = (index_mas_alto - index_recibido) as u32; // 0..63

        // verificamos si ya recibimos este paquete (bit=1). Si ya lo recibimos, es un replay attack, hay que rechazarlo
        self.verificar_paquete_repetido(desplazamiento, replay_window)?;

        // caso valido es cuando recibimos en bit que era 0 -> aceptamos y marcamos bit como recibido (1)
        let estadisticas = self.obtener_estadisticas_srtp(ssrc)?;
        estadisticas.replay_window |= 1u64 << desplazamiento;

        Ok(())
    }

    fn actualizar_por_paquete_nuevo(
        &mut self,
        ssrc: u32,
        index_recibido: u64,
        index_mas_alto: u64,
    ) -> Result<(), ErrorSRTP> {
        let estadisticas = self.obtener_estadisticas_srtp(ssrc)?;
        let desplazamiento = index_recibido - index_mas_alto;

        // si avanzamos más de 64, reseteamos ventana porque solo queda el más nuevo
        if desplazamiento >= 64 {
            estadisticas.replay_window = 1; // solo el más nuevo
        } else {
            // desplazamos hacia la izquierda y marcamos bit 0
            estadisticas.replay_window <<= desplazamiento; // <<= es left-shift combinado con asignar variable, hacemos left-shift para mover los bits de la ventana
            estadisticas.replay_window |= 1; // marcamos bit
        }

        // actualizamos index_mas_alto
        estadisticas.seq_mas_alto = (index_recibido & 0xFFFF) as u16;
        estadisticas.roc = (index_recibido >> 16) as u32;
        Ok(())
    }

    fn verificar_paquete_viejo(&self, desplazamiento: u32) -> Result<(), ErrorSRTP> {
        if desplazamiento >= 64 {
            return Err(ErrorSRTP::ErrorReplay(
                "Paquete SRTP fuera de ventana anti-replay (muy viejo)".to_string(),
            ));
        }
        Ok(())
    }

    fn verificar_paquete_repetido(
        &self,
        desplazamiento: u32,
        replay_window: u64,
    ) -> Result<(), ErrorSRTP> {
        if (replay_window >> desplazamiento) & 1 == 1 {
            return Err(ErrorSRTP::ErrorReplay(
                "Paquete SRTP repetido (replay attack)".to_string(),
            ));
        }
        Ok(())
    }

    // ESTA TRABAJA SOBRE PAYLOAD
    fn descifrar_rtp(
        &self,
        ssrc: u32,
        indice_paquete: u64,
        paquete: &mut [u8],
    ) -> Result<(), ErrorSRTP> {
        let iv = self.derivar_iv(ssrc, indice_paquete)?; // derivo el IV para este paquete
        let payload = Self::obtener_payload_rtp(paquete)?;
        let cipher = Cipher::aes_128_ctr();
        Self::asegurar_clave_longitud_igual_a_cipher(&self.clave_salt.clave, &cipher)?;
        let mut crypter = self.crear_crypter_aes_ctr(&iv, Mode::Decrypt)?; // creamos el crypter para descifrar

        // descifro el payload en un buffer temporal y copio de vuelta
        let mut out_buf = vec![0u8; payload.len() + cipher.block_size()];
        self.descifrar_payload_update_in_place(payload, &mut out_buf, &mut crypter)?;
        Ok(())
    }

    fn obtener_payload_rtp(paquete: &mut [u8]) -> Result<&mut [u8], ErrorSRTP> {
        Self::chequear_minimo_bytes_paquete(paquete)?;
        let header_len = 12;
        let (_header, payload) = paquete.split_at_mut(header_len);
        Ok(payload)
    }

    fn descifrar_payload_update_in_place(
        &self,
        payload: &mut [u8],
        out_buf: &mut Vec<u8>,
        crypter: &mut Crypter,
    ) -> Result<(), ErrorSRTP> {
        // aca el update hace su magia -> descifra el payload
        let count = crypter.update(payload, out_buf).map_err(|_| {
            ErrorSRTP::ErrorCifradoAES("Error en update() de AES-CTR (desencriptado)".to_string())
        })?;

        // finalize() para completar el ciclo del Crypter (cierra el proceso)
        let rest = crypter.finalize(&mut out_buf[count..]).map_err(|_| {
            ErrorSRTP::ErrorCifradoAES("Error en finalize() de AES-CTR (desencriptado)".to_string())
        })?;

        let total = count + rest;
        out_buf.truncate(total);

        // aseguramos que el descifrado no cambió la longitud del payload (si pasa esto deberia explotar)
        Self::asegurar_estabilidad_longitud_payload_post_operaciones_encriptacion(
            out_buf.len(),
            payload.len(),
        )?;
        payload.copy_from_slice(out_buf);

        Ok(())
    }

    /// Verifica y desprotege un paquete SRTP para obtener el paquete RTP original.
    ///
    /// # Arguments
    /// * `ssrc` - El SSRC del flujo RTP.
    /// * `paquete_srtp` - El paquete SRTP a verificar y desproteger.
    ///
    /// # Returns
    /// * `Result<(), ErrorSRTP>` - Resultado de la operación, con error en caso de fallo.
    pub fn verificar_y_desproteger_rtp(
        &mut self,
        ssrc: u32,
        paquete_srtp: &mut Vec<u8>,
    ) -> Result<(), ErrorSRTP> {
        let len_rtp = Self::calcular_longitud_rtp(paquete_srtp)?; // calcula longitud RTP sin tag
        // split_at_mut nos da dos slices mutables: [RTP | TAG]
        let (rtp_bytes, tag_recibido_slice) = paquete_srtp.split_at_mut(len_rtp);
        let mut tag_recibido = [0u8; 10];
        tag_recibido.copy_from_slice(tag_recibido_slice);
        Self::validar_longitud_rtp_seq(rtp_bytes)?;
        let seq = u16::from_be_bytes([rtp_bytes[2], rtp_bytes[3]]);

        // calculamos index de paquete (esto puede actualizar roc / seq_mas_alto)
        let indice_paquete = self.calcular_indice_paquete(ssrc, seq)?;

        // decidimos si aceptamos este index (sobre replay window)
        // TODO: deberia chequearse el indice para este ssrc en especifico, pero se tiene un contador en general.
        // Es hacer un diccionario ssrc:(indice, roc), pero para chequear lo comento a este chequeo.
        self.verificar_replay_y_actualizar(ssrc, indice_paquete)?;

        // recalculamos HMAC-SHA1-80 sobre RTP || ROC y lo comparamos
        let tag_esperado = self.generar_tag_autenticacion(ssrc, rtp_bytes)?;
        Self::validar_igualdad_tags(&tag_esperado, &tag_recibido)?;

        // si pasó autenticación + anti-replay, desciframos el payload in-place
        self.descifrar_rtp(ssrc, indice_paquete, rtp_bytes)?;
        // truncamos el paquete SRTP para dejar solo RTP (removemos tag)
        paquete_srtp.truncate(len_rtp);

        Ok(())
    }

    fn calcular_longitud_rtp(paquete_srtp: &[u8]) -> Result<usize, ErrorSRTP> {
        Self::validar_longitud_header_tag(paquete_srtp)?;
        Ok(paquete_srtp.len() - 10) // resto 10 bytes del tag HMAC-SHA1-80
    }

    fn validar_longitud_header_tag(paquete_srtp: &[u8]) -> Result<(), ErrorSRTP> {
        // necesitamos al menos header RTP (12) + tag (10)
        if paquete_srtp.len() < 12 + 10 {
            return Err(ErrorSRTP::ErrorPaqueteDemasiadoCorto(
                "SRTP demasiado corto: falta header o tag".to_string(),
            ));
        }
        Ok(())
    }

    fn validar_longitud_rtp_seq(paquete_rtp: &[u8]) -> Result<(), ErrorSRTP> {
        if paquete_rtp.len() < 12 {
            return Err(ErrorSRTP::ErrorPaqueteDemasiadoCorto(
                "RTP header debe tener al menos 12 bytes".to_string(),
            ));
        }
        Ok(())
    }

    fn validar_igualdad_tags(tag_esperado: &[u8], tag_recibido: &[u8]) -> Result<(), ErrorSRTP> {
        if tag_esperado != tag_recibido {
            return Err(ErrorSRTP::ErrorHMAC(
                "Tag de autenticación SRTP inválido (HMAC no coincide)".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seguridad::srtp::errores::ErrorSRTP;

    fn contexto_srtp_prueba_basico() -> SRTPContexto {
        // clave de 16 bytes (AES-128) y salt de 14 bytes (SRTP)
        let clave = vec![0u8; 16];
        let salt = vec![0u8; 14];
        SRTPContexto::new(clave, salt)
    }

    #[test]
    fn test_new_inicializa_campos_en_cero() {
        let ctx = contexto_srtp_prueba_basico();
        let estadisticas = EstadisticasSRTP::default();

        assert_eq!(estadisticas.roc, 0);
        assert_eq!(estadisticas.seq_mas_alto, 0);
        assert_eq!(estadisticas.replay_window, 0);
        assert_eq!(ctx.clave_salt.clave.len(), 16);
        assert_eq!(ctx.clave_salt.salt.len(), 14);
    }

    #[test]
    fn test_derivar_iv_formato_basico_y_dependencia_de_campos() {
        // salt conocido de 14 bytes
        let clave: Vec<u8> = (0u8..16u8).collect();
        let salt: Vec<u8> = (1u8..15u8).collect(); // 1..=14

        let ctx = SRTPContexto::new(clave, salt.clone());

        let ssrc = 0x11223344;
        let indice_paquete = 0x0000_0000_0000_A1B2u64;

        let iv = ctx.derivar_iv(ssrc, indice_paquete).unwrap();

        assert_eq!(iv.len(), 16);
        assert_eq!(iv[0], 0);
        assert_eq!(iv[1], 0);

        // bytes [2..4) = primeros bytes del salt (no se xorean con nada)
        assert_eq!(&iv[2..4], &salt[0..2]);

        // cambiar el índice debe cambiar el IV
        let iv_otro_indice = ctx.derivar_iv(ssrc, indice_paquete + 1).unwrap();
        assert_ne!(iv, iv_otro_indice);

        // cambiar el SSRC también debe cambiar el IV
        let iv_otro_ssrc = ctx.derivar_iv(0x55667788, indice_paquete).unwrap();
        assert_ne!(iv, iv_otro_ssrc);
    }

    #[test]
    fn test_derivar_iv_falla_con_salt_invalido() {
        let clave = vec![0u8; 16];
        let salt_invalido = vec![0u8; 13]; // no son 14 bytes
        let ctx = SRTPContexto::new(clave, salt_invalido);

        let res = ctx.derivar_iv(0x12345678, 1);
        assert!(matches!(res, Err(ErrorSRTP::ErrorClaveSaltInvalida(_))));
    }

    #[test]
    fn test_proteger_y_desproteger_rtp_roundtrip() {
        // clave y salt fijos para tener algo determinista
        let clave: Vec<u8> = (0u8..16u8).collect();
        let salt: Vec<u8> = (100u8..114u8).collect(); // 14 bytes

        let mut ctx_tx = SRTPContexto::new(clave.clone(), salt.clone());
        let mut ctx_rx = SRTPContexto::new(clave, salt);

        let ssrc = 0x12345678;

        let mut paquete_rtp: Vec<u8> = vec![
            0x80, 0x60, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x12, 0x34, 0x56, 0x78,
        ];
        let payload_original: Vec<u8> = vec![0x10, 0x20, 0x30, 0x40, 0x50];
        paquete_rtp.extend_from_slice(&payload_original);

        let len_original_rtp = paquete_rtp.len();

        ctx_tx
            .proteger_y_firmar_rtp(ssrc, &mut paquete_rtp)
            .expect("Error protegiendo RTP en TX");

        assert_eq!(paquete_rtp.len(), len_original_rtp + 10);

        let payload_cifrado = &paquete_rtp[12..len_original_rtp];
        assert_ne!(payload_cifrado, &payload_original[..]);

        ctx_rx
            .verificar_y_desproteger_rtp(ssrc, &mut paquete_rtp)
            .expect("Error verificando/descifrando SRTP en RX");

        assert_eq!(paquete_rtp.len(), len_original_rtp);

        assert_eq!(
            &paquete_rtp[0..12],
            &[
                0x80, 0x60, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x12, 0x34, 0x56, 0x78
            ]
        );

        assert_eq!(&paquete_rtp[12..], &payload_original[..]);
    }

    #[test]
    fn test_verificar_replay_y_actualizar_rechaza_repetidos() {
        let mut ctx = contexto_srtp_prueba_basico();

        ctx.verificar_replay_y_actualizar(1, 1)
            .expect("El primer índice debería aceptarse");
        let estadisticas = ctx.estadisticas_ssrc.get(&1).unwrap();
        assert_eq!(estadisticas.replay_window, 1);

        let res = ctx.verificar_replay_y_actualizar(1, 1);
        assert!(matches!(res, Err(ErrorSRTP::ErrorReplay(_))));
    }

    #[test]
    fn test_verificar_replay_y_actualizar_fuera_de_ventana() {
        let mut ctx = contexto_srtp_prueba_basico();

        ctx.verificar_replay_y_actualizar(1, 100)
            .expect("Índice 100 debería aceptarse");

        let res = ctx.verificar_replay_y_actualizar(1, 100 - 64);
        assert!(matches!(res, Err(ErrorSRTP::ErrorReplay(_))));
    }

    #[test]
    fn test_verificar_y_desproteger_rtp_falla_con_hmac_invalido() {
        let clave: Vec<u8> = (0u8..16u8).collect();
        let salt: Vec<u8> = (50u8..64u8).collect();

        let mut ctx_tx = SRTPContexto::new(clave.clone(), salt.clone());
        let mut ctx_rx = SRTPContexto::new(clave, salt);

        let ssrc = 0xCAFEBABE;

        let mut paquete_rtp: Vec<u8> = vec![
            0x80, 0x60, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0xCA, 0xFE, 0xBA, 0xBE,
        ];
        paquete_rtp.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);

        ctx_tx
            .proteger_y_firmar_rtp(ssrc, &mut paquete_rtp)
            .expect("Error protegiendo RTP en TX");

        // Corrompemos el último byte del tag
        let last_index = paquete_rtp.len() - 1;
        paquete_rtp[last_index] ^= 0xFF;

        let res = ctx_rx.verificar_y_desproteger_rtp(ssrc, &mut paquete_rtp);
        assert!(matches!(res, Err(ErrorSRTP::ErrorHMAC(_))));
    }

    #[test]
    fn test_descifrar_rtp_solo_sobre_payload() {
        let clave: Vec<u8> = (0u8..16u8).collect();
        let salt: Vec<u8> = (10u8..24u8).collect();

        let mut ctx = SRTPContexto::new(clave, salt);

        let ssrc = 0x01020304;

        let mut paquete: Vec<u8> = vec![
            0x80, 0x60, 0x00, 0x10, // seq=16
            0x00, 0x00, 0x00, 0x10, 0x01, 0x02, 0x03, 0x04,
        ];
        let payload_original = vec![1u8, 2, 3, 4, 5, 6];
        paquete.extend_from_slice(&payload_original);

        let seq = 0x0010;
        let indice_paquete = ctx.calcular_indice_paquete(ssrc, seq).unwrap();

        ctx.proteger_rtp(ssrc, paquete.as_mut_slice())
            .expect("No se pudo cifrar RTP");

        let payload_cifrado = paquete[12..].to_vec();
        assert_ne!(payload_cifrado, payload_original);

        ctx.descifrar_rtp(ssrc, indice_paquete, paquete.as_mut_slice())
            .expect("No se pudo descifrar RTP");

        assert_eq!(&paquete[12..], &payload_original[..]);
    }
}
