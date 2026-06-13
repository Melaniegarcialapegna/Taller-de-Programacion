//! # Sistema de Encriptación basado en RSA
//!
//! Para encriptar un stream se debe usar [`SistemaDeEncriptacion::encriptar_conexion`]. Una vez que la conexión esta encriptada,
//! se deberan encriptar los mensajes antes de enviarlos con [`SistemaDeEncriptacion::encriptar_mensaje`]. Cuando se reciban mensajes, se
//! podran desencriptar usando [`SistemaDeEncriptacion::desencriptar_mensaje`].

pub mod error;
#[cfg(test)]
pub mod mock_socket;
pub mod test;

use error::ErrorEncriptacion;
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::{Read, Write},
};

/// Tamanio de la clave privada en bytes
pub const TAMANIO_CLAVE_PRIVADA: usize = 1024;

/// Tamaño del buffer para los intercambios de mensajes en el handshake
pub const TAMANIO_CLAVE_PUBLICA: usize = 251;

/// Tamaño del buffer para los intercambios de mensajes en el handshake
pub const TAMANIO_CLAVE_HASH: usize = 4;

/// Tamanio de cada bloque que se encripta por separado
pub const TAMANIO_BLOQUE: usize = 100;

/// Representa un Sistema de Encriptación para un Stream TCP.
/// Permite encriptar y desencriptar mensajes
#[derive(Clone, Debug)]
pub struct SistemaDeEncriptacion {
    clave_hash: String,
    clave_hash_externa: String,
    clave_publica_externa: RsaPublicKey,
    clave_privada: RsaPrivateKey,
}

impl SistemaDeEncriptacion {
    /// Encripta la conexión recibida con el Sistema de Encriptación
    ///
    /// PRE: `stream` representa un stream que aun no esta encriptado
    ///
    /// POST: El stream queda encriptado, y el usuario externo solo va a aceptar mensajes bien encriptados.
    pub fn encriptar_conexion<S: Read + Write>(
        mut stream: S,
    ) -> Result<SistemaDeEncriptacion, ErrorEncriptacion> {
        let mut rng = rand::thread_rng();

        // Creo par clave privada-clave publica
        let clave_privada = RsaPrivateKey::new(&mut rng, TAMANIO_CLAVE_PRIVADA)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;
        let clave_publica = RsaPublicKey::from(&clave_privada);

        Self::enviar_clave_publica_propia(&mut stream, &clave_publica)?;
        let clave_hash = Self::enviar_clave_hash_propia(&mut stream)?;

        let (clave_publica_externa, clave_hash_externa) = Self::leer_clave_publica_externa(stream)?;
        Ok(SistemaDeEncriptacion {
            clave_hash,
            clave_hash_externa,
            clave_publica_externa,
            clave_privada,
        })
    }

    /// Encripta un mensaje para que pueda ser recibido correctamente por el otro extremo del stream encriptado y lo devuelve
    pub fn encriptar_mensaje(&mut self, mensaje: &str) -> Result<Vec<u8>, ErrorEncriptacion> {
        let mensaje_a_enviar = self.obtener_mensaje_con_hash(mensaje);

        // Obtengo la cantidad de bloques y la cantidad de bytes que quedan en el ultimo
        // bloque, que va a estar incompleto
        let largo_mensaje = mensaje_a_enviar.len();
        let cantidad_bloques_enteros = largo_mensaje / TAMANIO_BLOQUE;
        let cantidad_bytes_ultimo_bloque = largo_mensaje % TAMANIO_BLOQUE;
        let mensaje_encriptado = self.encriptar_mensaje_en_bloques(
            mensaje_a_enviar,
            largo_mensaje,
            cantidad_bloques_enteros,
        )?;

        // Agrego el header (la cantidad de bloques y los bytes del bloque incompleto) al mensaje
        let cantidad_bloques_enteros_64 = cantidad_bloques_enteros as u64;
        let cantidad_bytes_ultimo_bloque_64 = cantidad_bytes_ultimo_bloque as u64;
        let mut bytes_a_enviar = cantidad_bloques_enteros_64.to_be_bytes().to_vec();
        bytes_a_enviar.extend_from_slice(&cantidad_bytes_ultimo_bloque_64.to_be_bytes());

        // Agrego los bytes del mensaje encriptado despues de los bytes del tamanio
        bytes_a_enviar.extend_from_slice(&mensaje_encriptado);

        Ok(bytes_a_enviar)
    }

    fn encriptar_mensaje_en_bloques(
        &mut self,
        mensaje_a_enviar: String,
        largo_mensaje: usize,
        cantidad_bloques_enteros: usize,
    ) -> Result<Vec<u8>, ErrorEncriptacion> {
        let bytes_bloque_sin_encriptar = mensaje_a_enviar.as_bytes();
        let mut mensaje_encriptado = vec![];
        for i in 0..cantidad_bloques_enteros {
            // Obtengo los indices de los bytes de este bloque
            let indice_inicio_bloque = TAMANIO_BLOQUE * i;
            let indice_fin_bloque = TAMANIO_BLOQUE * (i + 1);
            let cadena_bloque =
                &bytes_bloque_sin_encriptar[indice_inicio_bloque..indice_fin_bloque];

            // Encripto los bytes de este bloque
            let bytes_bloque_encriptado = self
                .clave_publica_externa
                .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, cadena_bloque)
                .map_err(|e| {
                    dbg!(e);
                    ErrorEncriptacion::ErrorEncriptandoMensaje
                })?;
            mensaje_encriptado.extend_from_slice(&bytes_bloque_encriptado);
        }

        // Encripto los bytes del ultimo bloque
        let indice_inicio_bloque = TAMANIO_BLOQUE * cantidad_bloques_enteros;
        let indice_fin_bloque = largo_mensaje;
        let cadena_bloque = &bytes_bloque_sin_encriptar[indice_inicio_bloque..indice_fin_bloque];
        let bytes_bloque_encriptado = self
            .clave_publica_externa
            .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, cadena_bloque)
            .map_err(|e| {
                dbg!(e);
                ErrorEncriptacion::ErrorEncriptandoMensaje
            })?;
        mensaje_encriptado.extend_from_slice(&bytes_bloque_encriptado);

        Ok(mensaje_encriptado)
    }

    pub fn leer_desencriptando_mensaje<S: Read + Write>(
        &mut self,
        mut stream: S,
    ) -> Result<String, ErrorEncriptacion> {
        let mut buffer_cantidad_bloques: [u8; 8] = [0; 8];
        let mut buffer_cantidad_bytes_ultimo_bloque: [u8; 8] = [0; 8];

        // Leer 64 bits y pasarlo a entero -> Eso va a ser la
        // cantidad de bloques enteros a leer
        stream
            .read_exact(&mut buffer_cantidad_bloques)
            .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

        // Leer 64 bits y pasarlo a entero -> Eso va a ser la
        // bytes del ultimo bloque (que no esta entero)
        stream
            .read_exact(&mut buffer_cantidad_bytes_ultimo_bloque)
            .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

        // Paso a entero la cantidad de bloques y la cantidad de bytes del ultimo bloque
        let cantidad_bloques_enteros = u64::from_be_bytes(buffer_cantidad_bloques);
        let mut mensaje_desencriptado = vec![];

        let mut buffer_bloque = vec![0; 128];
        for _ in 0..cantidad_bloques_enteros {
            stream
                .read_exact(&mut buffer_bloque)
                .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

            let bloque_desencriptado = self
                .clave_privada
                .decrypt(Pkcs1v15Encrypt, &buffer_bloque[..])
                .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

            mensaje_desencriptado.extend_from_slice(&bloque_desencriptado);
            buffer_bloque = vec![0; 128]
        }

        let mut buffer_ultimo_bloque = vec![0; 128];
        stream
            .read_exact(&mut buffer_ultimo_bloque)
            .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

        let ultimo_bloque_desencriptado = self
            .clave_privada
            .decrypt(Pkcs1v15Encrypt, &buffer_ultimo_bloque[..])
            .map_err(|e| {
                dbg!(e);
                ErrorEncriptacion::ErrorDesencriptandoMensaje
            })?;

        mensaje_desencriptado.extend_from_slice(&ultimo_bloque_desencriptado);
        let mensaje_desencriptado_str = String::from_utf8(mensaje_desencriptado)
            .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

        let (mensaje_original, clave_hash_leida) = self
            .obtener_campos_mensaje(&mensaje_desencriptado_str)
            .map_err(|_| ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

        if clave_hash_leida != self.obtener_hash_esperado(&mensaje_original) {
            return Err(ErrorEncriptacion::ErrorDesencriptandoMensaje);
        }

        Ok(mensaje_original)
    }

    /// Devuelve el hash que se espera que este al final del mensaje recibido, según la clave hash que se le proporciono al otro extremo.
    fn obtener_hash_esperado(&mut self, mensaje_sin_hash: &String) -> String {
        let mut hasher = DefaultHasher::new();
        format!("{}***{}", mensaje_sin_hash, self.clave_hash).hash(&mut hasher);
        let hash_esperado_str = format!("{}", hasher.finish());
        hash_esperado_str
    }

    /// Dado un mensaje original, agrega al final un hash que corresponde a hashear el mensaje original junto a la
    /// clave hash propocionada por el otro extremo.
    fn obtener_mensaje_con_hash(&mut self, mensaje: &str) -> String {
        let mut hasher = DefaultHasher::new();
        let mensaje_con_clave = format!("{}***{}", mensaje, self.clave_hash_externa);
        mensaje_con_clave.hash(&mut hasher);
        let mensaje_hasheado = hasher.finish();
        let mensaje_hasheado_str = format!("{}", mensaje_hasheado);

        let mensaje_a_enviar = format!("{}***{}", mensaje, mensaje_hasheado_str);
        mensaje_a_enviar
    }

    /// Recibe un mensaje encriptado.
    ///
    /// Devuelve el mensaje junto a su hash adjunto, en una tupla (String, String).
    fn obtener_campos_mensaje(
        &mut self,
        mensaje_desencriptado_str: &str,
    ) -> Result<(String, String), ErrorEncriptacion> {
        let mensaje_desencriptado_lineas: Vec<&str> =
            mensaje_desencriptado_str.split("***").collect();

        let mensaje_sin_hash = *mensaje_desencriptado_lineas
            .first()
            .ok_or(ErrorEncriptacion::ErrorDesencriptandoMensaje)?;
        let hash_mensaje = *mensaje_desencriptado_lineas
            .get(1)
            .ok_or(ErrorEncriptacion::ErrorDesencriptandoMensaje)?;

        Ok((String::from(mensaje_sin_hash), String::from(hash_mensaje)))
    }

    /// Lee la clave publica que debe enviar el otro extremo del stream.
    fn leer_clave_publica_externa<S: Read + Write>(
        mut socket: S,
    ) -> Result<(RsaPublicKey, String), ErrorEncriptacion> {
        let mut buffer_clave_publica = vec![0; TAMANIO_CLAVE_PUBLICA];
        socket
            .read_exact(&mut buffer_clave_publica)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;

        // Leo clave publica externa
        let bytes_ascii_leidos = String::from_utf8(buffer_clave_publica)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;
        let clave_publica_externa = RsaPublicKey::from_pkcs1_pem(&bytes_ascii_leidos)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;

        let mut buffer_clave_hash = vec![0; TAMANIO_CLAVE_HASH];

        // Leo clave hash externa
        socket
            .read_exact(&mut buffer_clave_hash)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;
        let clave_hash = String::from_utf8(buffer_clave_hash)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;

        Ok((clave_publica_externa, clave_hash))
    }

    /// Envia la publica propia para que el otro extremo del stream pueda encriptar y enviar mensajes
    fn enviar_clave_publica_propia<S: Read + Write>(
        socket: &mut S,
        clave_publica: &RsaPublicKey,
    ) -> Result<(), ErrorEncriptacion> {
        let clave_publica_str = clave_publica
            .to_pkcs1_pem(LineEnding::LF)
            .map_err(|_| ErrorEncriptacion::ErrorCreandoClavePublica)?;

        socket
            .write(clave_publica_str.as_bytes())
            .map_err(|_| ErrorEncriptacion::ErrorEnviandoClavePublica)?;
        Ok(())
    }

    /// Envia la clave hash propia, que el otro extremo usara para confirmar su identidad en cada mensaje.
    fn enviar_clave_hash_propia<S: Read + Write>(
        socket: &mut S,
    ) -> Result<String, ErrorEncriptacion> {
        let clave_hash = Self::generar_clave_random()?;

        socket
            .write(clave_hash.as_bytes())
            .map_err(|_| ErrorEncriptacion::ErrorEnviandoClaveHash)?;
        Ok(clave_hash)
    }

    fn generar_clave_random() -> Result<String, ErrorEncriptacion> {
        // let mut bytes_random: Vec<u8> = vec![];

        // for _ in 0..TAMANIO_CLAVE_HASH {
        //     let byte_random: u8 = rand::random();
        //     bytes_random.push((byte_random % 50) + 50);
        // }

        // let cadena = String::from_utf8(bytes_random)
        //     .map_err(|_| ErrorEncriptacion::ErrorCreandoClaveHash)?;

        // Ok(cadena)
        Ok("HOLA".to_string())
    }
}
