use rand::Rng;
use std::convert::TryInto;
use std::net::Ipv4Addr;

/// Longitud del header STUN en bytes
const STUN_HEADER_SIZE: usize = 20;

/// "Magic Cookie" STUN en formato big endian (vale esto)
const MAGIC_COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];
/// Magic Cookie como u32 para XOR
const MAGIC_COOKIE_U32: u32 = 0x2112A442;

// Tipos de Mensaje STUN con sus valores según RFC
/// Binding Request (0x0001)
const TYPE_BINDING_REQUEST: [u8; 2] = [0x00, 0x01];
/// Binding Response (0x0101)
const TYPE_BINDING_RESPONSE: [u8; 2] = [0x01, 0x01];
/// Binding Error Response (0x0111)
const TYPE_BINDING_ERROR_RESPONSE: [u8; 2] = [0x01, 0x11];

// Atributos STUN con sus valores según RFC
/// USE_CANDIDATE (0x0025) es el que se usa para nominación
const ATTR_USE_CANDIDATE: [u8; 2] = [0x00, 0x25];
/// ICE_FINALIZADO (un atributo no estándar, que usamos para finalizar, le puse 0xFF00)
const ATTR_ICE_FINALIZADO: [u8; 2] = [0xFF, 0x00];
/// XOR_MAPPED_ADDRESS (0x0020)
const ATTR_XOR_MAPPED_ADDRESS: [u8; 2] = [0x00, 0x20];
/// MAPPED_ADDRESS (0x0001)
const ATTR_MAPPED_ADDRESS: [u8; 2] = [0x00, 0x01];

/// (tipo_bytes, longitud_bytes, offset_valor, longitud_valor, padding)
pub type StunAtributoHeader = ([u8; 2], [u8; 2], usize, usize, usize);

/// Representa la dirección y puerto mapeados públicamente por el servidor STUN.
pub struct DireccionMappeada {
    pub ip: String,
    pub puerto: u16,
}

/// Representa el mensaje binario STUN.
#[derive(Debug, Clone)]
pub struct MensajeStun {
    // 2 bytes: tipo de mensaje
    pub tipo_mensaje_stun: [u8; 2],
    // 2 bytes: longitud de atributos (payload)
    largo_mensaje_stun: [u8; 2],
    // 4 bytes: la magic cookie (q vale 0x2112A442
    magic_cookie: [u8; 4],
    // 12 bytes: el ID de transacción
    pub transaction_id: [u8; 12],
    // contenido variable, atributos
    pub payload: Vec<u8>,
}

impl MensajeStun {
    /// Crea un nuevo 'MensajeStun' con el tipo y atributos especificados.
    pub fn new(type_id: [u8; 2], atributos: Option<&[&[u8; 2]]>, tid: Option<[u8; 12]>) -> Self {
        let transaction_id = crear_o_usar_tid(tid);
        let payload = construir_payload(atributos);
        let largo = (payload.len() as u16).to_be_bytes();

        Self {
            tipo_mensaje_stun: type_id,
            largo_mensaje_stun: largo,
            magic_cookie: MAGIC_COOKIE,
            transaction_id,
            payload,
        }
    }

    /// Crea un STUN Binding Request base (el check estándar).
    pub fn binding_request() -> Self {
        Self::new(TYPE_BINDING_REQUEST, None, None)
    }

    /// Crea un STUN Binding Request con la "nominación" (USE_CANDIDATE).
    pub fn binding_request_use_candidate() -> Self {
        Self::new(TYPE_BINDING_REQUEST, Some(&[&ATTR_USE_CANDIDATE]), None)
    }

    /// Crea un STUN Binding Request para notificar la finalización de ICE.
    pub fn binding_request_ice_finalizado() -> Self {
        Self::new(TYPE_BINDING_REQUEST, Some(&[&ATTR_ICE_FINALIZADO]), None)
    }

    /// Crea un STUN Binding Response a partir de un Request, manteniendo el TID.
    pub fn binding_response_for_request(&self, ice_finalizado: bool) -> Self {
        let atributos = if ice_finalizado {
            Some(&[&ATTR_ICE_FINALIZADO][..])
        } else {
            None
        };
        // usa el TID de la Request ('self.transaction_id') para la Response.
        Self::new(TYPE_BINDING_RESPONSE, atributos, Some(self.transaction_id))
    }

    /// Serializa el mensaje STUN en un vector de bytes listo para ser enviado.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(STUN_HEADER_SIZE + self.payload.len());

        buffer.extend_from_slice(&self.tipo_mensaje_stun); // 1. Tipo
        buffer.extend_from_slice(&self.largo_mensaje_stun); // 2. Longitud
        buffer.extend_from_slice(&self.magic_cookie); // 3. Magic Cookie
        buffer.extend_from_slice(&self.transaction_id); // 4. ID de Transacción (TID)
        buffer.extend_from_slice(&self.payload); // 5. Atributos

        buffer
    }

    /// Deserializa un vector de bytes en un mensaje STUN.
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        if buffer.len() < STUN_HEADER_SIZE {
            return Err("Longitud de mensaje STUN insuficiente.");
        }
        if buffer[4..8] != MAGIC_COOKIE {
            return Err("Magic Cookie STUN inválida.");
        }

        let tipo_mensaje_stun: [u8; 2] = [buffer[0], buffer[1]];
        let largo_mensaje_stun: [u8; 2] = [buffer[2], buffer[3]];
        let largo_payload_u16 = u16::from_be_bytes(largo_mensaje_stun);
        let largo_payload = largo_payload_u16 as usize;
        if buffer.len() != STUN_HEADER_SIZE + largo_payload {
            return Err("La longitud del mensaje no coincide con el valor del header.");
        }

        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&buffer[8..20]);

        let payload = buffer[STUN_HEADER_SIZE..].to_vec();

        Ok(Self {
            tipo_mensaje_stun,
            largo_mensaje_stun,
            magic_cookie: MAGIC_COOKIE,
            transaction_id,
            payload,
        })
    }

    /// Obtiene la longitud mínima del mensaje (sólo la header).
    pub fn min_size() -> usize {
        STUN_HEADER_SIZE
    }

    /// Verifica si el mensaje es un STUN Binding Request.
    pub fn es_request(&self) -> bool {
        self.tipo_mensaje_stun == TYPE_BINDING_REQUEST
    }

    /// Verifica si el mensaje es un STUN Binding Response.
    pub fn es_response(&self) -> bool {
        self.tipo_mensaje_stun == TYPE_BINDING_RESPONSE
    }

    /// Verifica si el mensaje es un STUN Binding Error Response.
    pub fn es_error_response(&self) -> bool {
        self.tipo_mensaje_stun == TYPE_BINDING_ERROR_RESPONSE
    }

    /// Verifica si el mensaje contiene el atributo USE_CANDIDATE.
    pub fn contiene_use_candidate(&self) -> bool {
        self.payload
            .windows(2)
            .any(|window| window == ATTR_USE_CANDIDATE)
    }

    /// Verifica si el mensaje contiene el atributo ICE_FINALIZADO.
    pub fn contiene_ice_finalizado(&self) -> bool {
        self.payload
            .windows(2)
            .any(|window| window == ATTR_ICE_FINALIZADO)
    }

    /// Itera sobre los atributos en el payload y extrae el XOR-MAPPED-ADDRESS o MAPPED-ADDRESS.
    pub fn get_direccion_mappeada(&self) -> Result<DireccionMappeada, String> {
        let mut offset = 0;
        let payload_len = self.payload.len();

        while offset < payload_len {
            let (attr_type_bytes, _attr_len_bytes, value_offset, attr_len_usize, padding) =
                parsear_header_atributo(&self.payload, offset)?;

            offset = value_offset + attr_len_usize + padding;
            let attr_type = u16::from_be_bytes(attr_type_bytes);

            if attr_type == u16::from_be_bytes(ATTR_XOR_MAPPED_ADDRESS)
                || attr_type == u16::from_be_bytes(ATTR_MAPPED_ADDRESS)
            {
                if attr_len_usize != 8 {
                    return Err(format!(
                        "Longitud de atributo de dirección inesperada: {} bytes",
                        attr_len_usize
                    ));
                }

                let value_slice =
                    match self
                        .payload
                        .get(value_offset..value_offset + attr_len_usize)
                    {
                        Some(s) => s,
                        None => return Err(
                            "Payload corrupto: No hay bytes suficientes para el Valor de Atributo."
                                .to_string(),
                        ),
                    };

                return procesar_direccion_mappeada(attr_type, value_slice, self.transaction_id);
            }
        }

        Err("No se encontró el atributo XOR-MAPPED-ADDRESS ni MAPPED-ADDRESS.".to_string())
    }
}

/// Genera un nuevo ID de transacción (TID) o usa el provisto.
fn crear_o_usar_tid(tid: Option<[u8; 12]>) -> [u8; 12] {
    match tid {
        Some(t) => t,
        None => {
            let mut rng = rand::thread_rng();
            let mut tid = [0u8; 12];
            rng.fill(&mut tid[..]); // llenar con bytes aleatorios
            tid
        }
    }
}

/// Construye el payload del mensaje STUN a partir de la lista de atributos.
fn construir_payload(atributos: Option<&[&[u8; 2]]>) -> Vec<u8> {
    let mut payload: Vec<u8> = Vec::new();

    if let Some(attrs) = atributos {
        for attr in attrs.iter() {
            // un atributo simple solo tiene el tipo (2 bytes) y la longitud 0 (2 bytes)
            payload.extend_from_slice(*attr); // agregar el tipo de atributo (2 bytes)
            payload.extend_from_slice(&[0x00, 0x00]); // agregar la longitud 0 (2 bytes)
        }
    }
    payload
}

/// Parsea el Tipo y Longitud de un Atributo STUN.
fn parsear_header_atributo(payload: &[u8], offset: usize) -> Result<StunAtributoHeader, String> {
    let type_slice = match payload.get(offset..offset + 2) {
        Some(s) => s,
        None => {
            return Err(
                "Payload corrupto: No hay bytes suficientes para el Tipo de Atributo.".to_string(),
            );
        }
    };
    let attr_type_bytes: [u8; 2] = match type_slice.try_into() {
        Ok(a) => a,
        Err(_) => {
            return Err("Error de conversión interna de tipo para Tipo de Atributo.".to_string());
        }
    };

    let len_slice = match payload.get(offset + 2..offset + 4) {
        Some(s) => s,
        None => {
            return Err(
                "Payload corrupto: No hay bytes suficientes para la Longitud de Atributo."
                    .to_string(),
            );
        }
    };
    let attr_len_bytes: [u8; 2] = match len_slice.try_into() {
        Ok(a) => a,
        Err(_) => {
            return Err(
                "Error de conversión interna de tipo para Longitud de Atributo.".to_string(),
            );
        }
    };

    let attr_len_usize = u16::from_be_bytes(attr_len_bytes) as usize;
    let padding = (4 - (attr_len_usize % 4)) % 4;
    let value_offset = offset + 4;

    Ok((
        attr_type_bytes,
        attr_len_bytes,
        value_offset,
        attr_len_usize,
        padding,
    ))
}

/// Aplica la lógica de XOR al puerto y la IP si el atributo es XOR-MAPPED-ADDRESS.
fn aplicar_xor(
    mut puerto: u16,
    mut ip_bytes: [u8; 4],
    attr_type: u16,
    _tid: [u8; 12],
) -> (u16, [u8; 4]) {
    if attr_type == u16::from_be_bytes(ATTR_XOR_MAPPED_ADDRESS) {
        let cookie_bits = (MAGIC_COOKIE_U32 >> 16) as u16;
        puerto ^= cookie_bits;

        let cookie_bytes: [u8; 4] = MAGIC_COOKIE_U32.to_be_bytes();
        for i in 0..4 {
            ip_bytes[i] ^= cookie_bytes[i];
        }
    }
    (puerto, ip_bytes)
}

/// Extrae y decodifica la dirección y puerto mapeados.
fn procesar_direccion_mappeada(
    attr_type: u16,
    value: &[u8],
    transaction_id: [u8; 12],
) -> Result<DireccionMappeada, String> {
    // byte 0: Reservado, byte 1: Familia (0x01 para IPv4)
    if value[1] != 0x01 {
        return Err("Familia de dirección inesperada (no IPv4).".to_string());
    }

    let port_bytes_slice = match value[2..4].try_into() {
        Ok(b) => b,
        Err(_) => return Err("Error de conversión para bytes de puerto.".to_string()),
    };

    let ip_bytes_slice = match value[4..8].try_into() {
        Ok(b) => b,
        Err(_) => return Err("Error de conversión para bytes de IP.".to_string()),
    };

    let puerto_decodificado = u16::from_be_bytes(port_bytes_slice);

    let (puerto_srflx, ip_srflx_bytes) = aplicar_xor(
        puerto_decodificado,
        ip_bytes_slice,
        attr_type,
        transaction_id,
    );

    let srflx_ip = Ipv4Addr::new(
        ip_srflx_bytes[0],
        ip_srflx_bytes[1],
        ip_srflx_bytes[2],
        ip_srflx_bytes[3],
    );

    Ok(DireccionMappeada {
        ip: srflx_ip.to_string(),
        puerto: puerto_srflx,
    })
}
