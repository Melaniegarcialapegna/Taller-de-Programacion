use std::io::Write;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    sync::mpsc::{Receiver, Sender},
};
use std::{fs, path::Path};

use crate::servidor_utils::mensajes::mensaje_servidor::MensajeServidor;
use crate::servidor_utils::mensajes::mensaje_usuario::MensajeUsuario;
use crate::servidor_utils::servidor_central_utils::error::ErrorServidorCentral;
use crate::servidor_utils::servidor_central_utils::estado_usuario::EstadoUsuario;

type DiccionarioString = HashMap<String, String>;
type DiccionarioEstados = HashMap<String, EstadoUsuario>;

///Tiene informacion y gestiona a todos los usuarios dentro de el
pub struct ServidorCentral {
    pub ruta_archivo_usuarios: String,
    pub limite_usuarios: usize,
    pub usuarios: HashMap<String, String>,
    pub estado_usuarios: HashMap<String, EstadoUsuario>,
    pub usuarios_conectados: HashMap<String, Sender<MensajeUsuario>>,
    //Se ponen en set para :
    //-corroborar estado en O(1) y se usa como publisher/suscriber
    //     -> si un usuario cambia de estado se le notifica unicamente a los usuarios disponibles
    pub usuarios_disponibles: HashSet<String>,
}

impl ServidorCentral {
    //se crea una nueva instancia de ServidorCentral
    pub fn crear(
        ruta_archivo_usuarios: String,
        limite_usuarios: usize,
    ) -> Result<Self, ErrorServidorCentral> {
        //Se levantan usuarios de archivo y se los pone como desconectados
        let (usuarios, estado_usuarios) = Self::crear_registro_usuarios(&ruta_archivo_usuarios)?;

        Ok(ServidorCentral {
            ruta_archivo_usuarios,
            limite_usuarios,
            usuarios,
            estado_usuarios,
            usuarios_conectados: HashMap::new(),
            usuarios_disponibles: HashSet::new(),
        })
    }

    pub fn iniciar_escucha(
        mut self,
        rx_servidor: Receiver<MensajeServidor>,
    ) -> Result<(), ErrorServidorCentral> {
        for mensaje in rx_servidor {
            match mensaje {
                MensajeServidor::Registrar(usuario, contrasenia, tx_usuario) => {
                    self.registrar_usuario(usuario, contrasenia, tx_usuario)?;
                }
                MensajeServidor::Loguear(usuario, contrasenia, tx_usuario) => {
                    self.loguear_usuario(usuario, contrasenia, tx_usuario)?;
                }
                MensajeServidor::Llamar(usuario_llama, usuario_llamado) => {
                    self.usuario_llama_otro(usuario_llama, usuario_llamado)?;
                }
                MensajeServidor::AceptarLlamada(usuario_se_le_acepto) => {
                    self.aceptar_llamada(usuario_se_le_acepto)?;
                }
                MensajeServidor::RechazarLlamada(usuario_rechazado) => {
                    self.rechazar_llamada(usuario_rechazado)?;
                }
                MensajeServidor::EnviarOffer(usuario_receptor, offer_sdp) => {
                    self.enviar_offer(usuario_receptor, offer_sdp)?;
                }
                MensajeServidor::EnviarAnswer(usuario_emisor, usuario_receptor, answer_sdp) => {
                    self.enviar_answer(usuario_emisor, usuario_receptor, answer_sdp)?;
                }
                MensajeServidor::EstadoDisponible(usuario) => {
                    self.cambiar_estado_usuario_a_disponible(usuario)?;
                }
                MensajeServidor::Desconectarse(usuario) => {
                    self.cambiar_estado_usuario_a_desconectado(usuario)?;
                }
            }
        }
        Ok(())
    }

    //Levanta la informacion sobre los usuarios persistidos devolviendo:
    //- diccionario con la informarcion de estos (usuario:contraseña)
    //- diccionario de estados, inicialmente al levantar el servidor central todos los usuarios estan desconectados
    fn crear_registro_usuarios(
        ruta_archivo: &str,
    ) -> Result<(DiccionarioString, DiccionarioEstados), ErrorServidorCentral> {
        let lista: Vec<(String, String)> = Self::cargar_desde_archivo(ruta_archivo)?;

        let mut dicc_info = HashMap::new();
        let mut dicc_estado = HashMap::new();
        for (user, pass) in lista {
            let user_clone = user.clone();
            dicc_info.insert(user, pass);
            dicc_estado.insert(user_clone, EstadoUsuario::Desconectado);
        }
        Ok((dicc_info, dicc_estado))
    }

    //se levanta la informacion del archivo indicado
    fn cargar_desde_archivo(
        ruta_archivo: &str,
    ) -> Result<Vec<(String, String)>, ErrorServidorCentral> {
        //Si el archivo no existe se lo crea
        if !Path::new(ruta_archivo).exists() {
            let _ = File::create(ruta_archivo)
                .map_err(|_| ErrorServidorCentral::CreandoArchivoUsuarios)?;
        }

        let contenido = match fs::read_to_string(ruta_archivo) {
            Ok(c) => c,
            Err(_) => return Err(ErrorServidorCentral::LeyendoArchivoUsuarios),
        };

        let info_usuarios = contenido
            .lines()
            .filter_map(|linea| {
                let partes: Vec<&str> = linea.trim().split(';').collect();

                if partes.len() != 2 {
                    // LOGUEAR: línea mal formateada
                    return None; // por ahora ignoramos lineas mal formateadas (no debería pasar porque las escribimos nosotros, las logueamos)
                }

                Some((partes[0].to_string(), partes[1].to_string()))
            })
            .collect();

        Ok(info_usuarios)
    }

    //se persiste un nuevo registro de usuario en el archivo indicado
    fn persistir_usuario_en_archivo(
        ruta_archivo: &str,
        usuarios: &[(String, String)],
    ) -> Result<(), ErrorServidorCentral> {
        let mut buffer = String::new();

        for (nombre, pass) in usuarios {
            buffer.push_str(&format!("{};{}", nombre, pass));
        }

        let mut f = File::options()
            .append(true)
            .open(ruta_archivo)
            .map_err(|_| ErrorServidorCentral::ErrorPersistiendoUsuario)?;
        writeln!(&mut f, "{}", buffer)
            .map_err(|_| ErrorServidorCentral::ErrorPersistiendoUsuario)?;
        Ok(())
        // LOGUEAR: si hay error o si escribe bien
    }

    //se registra un nuevo usuario
    fn registrar_usuario(
        &mut self,
        usuario: String,
        contrasenia: String,
        tx_usuario: Sender<MensajeUsuario>,
    ) -> Result<(), ErrorServidorCentral> {
        //Chequeos previos al registro del usuario
        //el usuario no puede tener un nombre de usuario vacio
        let usuario = String::from(usuario.trim());
        if usuario.is_empty() {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("Usuario invalido")),
            );
            return Ok(());
        }

        //no puede repetirse un nombre de usuario -> son unicos
        if self.usuarios.contains_key(&usuario) {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("Nombre usuario no disponible")),
            );
            return Ok(());
        }

        //la contraseña no puede ser vacia
        let contrasenia = String::from(contrasenia.trim());
        if contrasenia.is_empty() {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("Contraseña invalida")),
            );
            return Ok(());
        }

        //Si todo OK se le notifica al usuario que su registro fue exitoso
        Self::enviar_channel_sin_accion_ante_desconexion(&tx_usuario, MensajeUsuario::Ok);

        //Se guarda la informacion de este usuario
        self.usuarios.insert(usuario.clone(), contrasenia.clone());

        //Se lo pone como desconectado en el dicc de estados
        self.estado_usuarios
            .insert(usuario.clone(), EstadoUsuario::Desconectado);

        //Se lo persiste en el archivo
        let info_usuario = vec![(usuario.clone(), contrasenia)];
        Self::persistir_usuario_en_archivo(&self.ruta_archivo_usuarios, &info_usuario)?;

        let mut usuarios_desconectados_repentinamente: HashSet<String> = HashSet::new();

        //Se le notifica a todos los usuarios conectados sobre este nuevo usuario junto con su estado(desconectado)
        for usuario_disponible in &self.usuarios_disponibles {
            if let Some(tx_usuario) = self.usuarios_conectados.get(usuario_disponible)
                && tx_usuario
                    .send(MensajeUsuario::ActualizarEstadoUsuario(
                        usuario.clone(),
                        EstadoUsuario::Desconectado,
                    ))
                    .is_err()
            {
                //En caso de que un usuario se desconecte repentinamente
                usuarios_desconectados_repentinamente.insert(usuario_disponible.clone());
            }
        }

        self.desconectar_usuarios(usuarios_desconectados_repentinamente);

        Ok(())
    }

    //se loguea un nuevo usuario
    fn loguear_usuario(
        &mut self,
        usuario: String,
        contrasenia: String,
        tx_usuario: Sender<MensajeUsuario>,
    ) -> Result<(), ErrorServidorCentral> {
        //se chequea que el usuario exista en la base
        let usuario = String::from(usuario.trim());
        if !self.usuarios.contains_key(&usuario) {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("Nombre usuario inexistente")),
            );
            return Ok(());
        }

        //se verifica que la contraseña sea correcta
        if let Some(contrasenia_usuario) = self.usuarios.get(&usuario)
            && *contrasenia_usuario != contrasenia
        {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("Contraseña incorrecta")),
            );
            return Ok(());
        }

        //se fija si otro peer no esta ya logueado con esta cuenta
        if self.usuarios_conectados.contains_key(&usuario) {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("Usuario ya logueado")),
            );
            return Ok(());
        }

        //se fija que no se exceda el limite de usuarios conectados
        //si la cantidad de usuarios contectados esta al tope
        //(en realidad con el "==" solo ya esta pero porlas!, nunca va a ser mayor je)
        if self.limite_usuarios <= self.usuarios_conectados.len() {
            Self::enviar_channel_sin_accion_ante_desconexion(
                &tx_usuario,
                MensajeUsuario::Error(String::from("No hay lugar para una nueva conexion")),
            );
            return Ok(());
        }

        //Si todo OK se le avisa asi sale de la etapa de login
        Self::enviar_channel_sin_accion_ante_desconexion(&tx_usuario, MensajeUsuario::Ok);

        //se lo pone dentro de los usuarios conectados
        self.usuarios_conectados
            .insert(usuario.clone(), tx_usuario.clone());

        //se le envia al usuario el dicc con todos los usuarios y su estado
        //lo elimino para no enviar estado propio en diccionario
        self.estado_usuarios.remove(&usuario);

        //se le envia la informacion del estado del resto de los usuarios
        if tx_usuario
            .send(MensajeUsuario::EstadoUsuarios(self.estado_usuarios.clone()))
            .is_err()
        {
            //Si se desconecta repentinamente se lo deja como desconectado
            self.estado_usuarios
                .insert(usuario.clone(), EstadoUsuario::Desconectado);
            return Ok(());
        }

        //lo pone en estado disponible dentro de este
        self.estado_usuarios
            .insert(usuario.clone(), EstadoUsuario::Disponible);

        let mut usuarios_desconectados_repentinamente: HashSet<String> = HashSet::new();

        //se le notifica sobre esta nueva conexion a los usuarios en el set de disponibles
        for usuario_disponible in &self.usuarios_disponibles {
            if let Some(tx_usuario) = self.usuarios_conectados.get(usuario_disponible)
                && tx_usuario
                    .send(MensajeUsuario::ActualizarEstadoUsuario(
                        usuario.clone(),
                        EstadoUsuario::Disponible,
                    ))
                    .is_err()
            {
                //En caso de que un usuario se desconecte repentinamente
                usuarios_desconectados_repentinamente.insert(usuario_disponible.clone());
            }
        }
        //se lo pone dentro de los usuarios disponibles
        self.usuarios_disponibles.insert(usuario.clone());

        self.desconectar_usuarios(usuarios_desconectados_repentinamente);

        Ok(())
    }

    //al servidor le llega que un usuario quiere llamar a otro
    fn usuario_llama_otro(
        &mut self,
        usuario_llama: String,
        usuario_llamado: String,
    ) -> Result<(), ErrorServidorCentral> {
        //no deberia pasar jamas pero si intenta autollamarse(¿¿)
        if usuario_llama == usuario_llamado
            && let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_llama)
        {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();
            if tx_usuario.send(MensajeUsuario::LlamadaRechazada).is_err() {
                usuario_desconectado_repentinamente.insert(usuario_llama);
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
            return Ok(());
        }

        //se le notifica al usuario llamado que esta siendo llamado y por quien
        if self.usuarios_disponibles.contains(&usuario_llamado)
            && let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_llamado)
        {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();
            if tx_usuario
                .send(MensajeUsuario::LlamadaEntrante(usuario_llama))
                .is_err()
            {
                usuario_desconectado_repentinamente.insert(usuario_llamado);
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
            return Ok(());
        };

        //si al usuario que quiere llamar no esta disponible
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_llama) {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();

            if tx_usuario.send(MensajeUsuario::LlamadaRechazada).is_err() {
                usuario_desconectado_repentinamente.insert(usuario_llama);
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
        }
        Ok(())
    }

    //servidor recibe notificacion de que un usuario acepto una llamada
    fn aceptar_llamada(
        &mut self,
        usuario_se_le_acepto: String,
    ) -> Result<(), ErrorServidorCentral> {
        //se le pide el offer al usuario que arranco la llamada
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_se_le_acepto) {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();

            if tx_usuario.send(MensajeUsuario::PedirOffer).is_err() {
                usuario_desconectado_repentinamente.insert(usuario_se_le_acepto);
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
        }
        Ok(())
    }
    //servidor recibe notificacion de que un usuario rechazo una llamada
    fn rechazar_llamada(&mut self, usuario_rechazado: String) -> Result<(), ErrorServidorCentral> {
        //se le notifica al usuario que su llamada fue rechazada
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_rechazado) {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();

            if tx_usuario.send(MensajeUsuario::LlamadaRechazada).is_err() {
                usuario_desconectado_repentinamente.insert(usuario_rechazado);
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
        }
        Ok(())
    }

    //un usuario nos envio su offer para que la redireccionemos al usuario con el cual quiere realizar la llamada
    fn enviar_offer(
        &mut self,
        usuario_receptor: String,
        offer_sdp: String,
    ) -> Result<(), ErrorServidorCentral> {
        //se le envia al usuario receptor el offer
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_receptor) {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();

            if tx_usuario
                .send(MensajeUsuario::PedirAnswer(offer_sdp))
                .is_err()
            {
                usuario_desconectado_repentinamente.insert(usuario_receptor);
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
        }
        Ok(())
    }

    //un usuario nos envio su answer para que la redireccionemos al usuario con el cual quiere realizar la llamada
    fn enviar_answer(
        &mut self,
        usuario_emisor: String,
        usuario_receptor: String,
        answer_sdp: String,
    ) -> Result<(), ErrorServidorCentral> {
        //se le envia al usuario el answer
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario_receptor) {
            let mut usuario_desconectado_repentinamente: HashSet<String> = HashSet::new();

            if tx_usuario
                .send(MensajeUsuario::EnviarAnswer(answer_sdp))
                .is_err()
            {
                usuario_desconectado_repentinamente.insert(usuario_receptor.clone());
                self.desconectar_usuarios(usuario_desconectado_repentinamente);
            }
        }

        //OBS: se asume que para este punto ya realizaran la llamada
        //por lo tanto se los pone como usuarios en estado ocupado

        //Ahora pasan a estar ocupados, se les notifica al resto de usuarios disponibles al respecto
        //se los elimina del set de disponibles
        self.usuarios_disponibles.remove(&usuario_emisor);
        self.usuarios_disponibles.remove(&usuario_receptor);

        //se los pone en estado ocupado
        self.estado_usuarios
            .entry(usuario_receptor.clone())
            .and_modify(|estado| *estado = EstadoUsuario::Ocupado);

        self.estado_usuarios
            .entry(usuario_emisor.clone())
            .and_modify(|estado| *estado = EstadoUsuario::Ocupado);

        let mut usuarios_desconectado_repentinamente: HashSet<String> = HashSet::new();

        //se les notifica al resto de usuarios disponibles sobre el cambio de estado de estos dos usuarios
        for usuario in &self.usuarios_disponibles {
            if let Some(tx_usuario) = self.usuarios_conectados.get(usuario) {
                if tx_usuario
                    .send(MensajeUsuario::ActualizarEstadoUsuario(
                        usuario_emisor.clone(),
                        EstadoUsuario::Ocupado,
                    ))
                    .is_err()
                {
                    usuarios_desconectado_repentinamente.insert(usuario.clone());
                }
                if tx_usuario
                    .send(MensajeUsuario::ActualizarEstadoUsuario(
                        usuario_receptor.clone(),
                        EstadoUsuario::Ocupado,
                    ))
                    .is_err()
                {
                    //ya se agrego previamente
                }
            }
        }
        self.desconectar_usuarios(usuarios_desconectado_repentinamente);
        Ok(())
    }

    //usuario notifica que cambia su estado a disponible (corto una llamada)
    fn cambiar_estado_usuario_a_disponible(
        &mut self,
        usuario: String,
    ) -> Result<(), ErrorServidorCentral> {
        self.estado_usuarios.remove(&usuario);

        //se le envia el resto de estados de los usuarios
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario)
            && tx_usuario
                .send(MensajeUsuario::EstadoUsuarios(self.estado_usuarios.clone()))
                .is_err()
        {
            //Si se desconecta repentinamente pasa a estar desconectado
            self.cambiar_estado_usuario_a_desconectado(usuario.clone())?;
            return Ok(());
        };

        //se le actualiza el estado a disponible
        self.estado_usuarios
            .insert(usuario.clone(), EstadoUsuario::Disponible);

        let mut usuarios_desconectado_repentinamente: HashSet<String> = HashSet::new();

        //se le notifica a los usuarios disponibles sobre esta actualizacion en el estado del usuario
        for usuario_disponible in &self.usuarios_disponibles {
            if let Some(tx_usuario) = self.usuarios_conectados.get(usuario_disponible)
                && tx_usuario
                    .send(MensajeUsuario::ActualizarEstadoUsuario(
                        usuario.clone(),
                        EstadoUsuario::Disponible,
                    ))
                    .is_err()
            {
                usuarios_desconectado_repentinamente.insert(usuario_disponible.clone());
            }
        }

        //se lo pone dentro del set de usuarios disponibles
        self.usuarios_disponibles.insert(usuario.clone());

        self.desconectar_usuarios(usuarios_desconectado_repentinamente);

        Ok(())
    }

    //usuario notifica que se desconectara
    fn cambiar_estado_usuario_a_desconectado(
        &mut self,
        usuario: String,
    ) -> Result<(), ErrorServidorCentral> {
        //usuario se desconecta por lo que se lo pone en desconectado

        //se lo elimina del set de usuarios disponibles
        self.usuarios_disponibles.remove(&usuario);

        //se actualiza su estado a desconectado
        self.estado_usuarios
            .entry(usuario.clone())
            .and_modify(|estado| *estado = EstadoUsuario::Desconectado);

        let mut usuarios_desconectado_repentinamente: HashSet<String> = HashSet::new();

        //se le notifica al resto de usuarios disponibles sobre esta actualizacion
        for usuario_disponible in &self.usuarios_disponibles {
            if let Some(tx_usuario) = self.usuarios_conectados.get(usuario_disponible)
                && tx_usuario
                    .send(MensajeUsuario::ActualizarEstadoUsuario(
                        usuario.clone(),
                        EstadoUsuario::Desconectado,
                    ))
                    .is_err()
            {
                usuarios_desconectado_repentinamente.insert(usuario_disponible.clone());
            }
        }

        self.desconectar_usuarios(usuarios_desconectado_repentinamente);

        //se le notifica al usuario que se lo desconecto del servidor correctamente
        if let Some(tx_usuario) = self.usuarios_conectados.get(&usuario) {
            //Si se desconecta de manera repentina no pasa nada
            Self::enviar_channel_sin_accion_ante_desconexion(tx_usuario, MensajeUsuario::Ok);
        }

        //se lo elimina de usuarios conectados
        self.usuarios_conectados.remove(&usuario);

        Ok(())
    }

    fn enviar_channel_sin_accion_ante_desconexion(
        tx_usuario: &Sender<MensajeUsuario>,
        mensaje: MensajeUsuario,
    ) {
        if tx_usuario.send(mensaje).is_err() {
            // en caso de error por desconexion repentina del usuario no se hace nada
        }
    }

    //Se pone en desconectados a los usuarios que se desconectan de manera repentina
    //Ademas se le notifica sobre este cambio de estado al resto de los usuarios, haciendo que esta sea una funcion recursiva
    fn desconectar_usuarios(&mut self, usuarios_a_desconectar: HashSet<String>) {
        if usuarios_a_desconectar.is_empty() {
            return;
        }

        let mut nuevos_desconectados: HashSet<String> = HashSet::new();

        for usuario in &usuarios_a_desconectar {
            //se le cambia estado
            self.usuarios_conectados.remove(usuario);
            self.estado_usuarios
                .insert(usuario.clone(), EstadoUsuario::Desconectado);

            //se le notifica al resto sobre este cambio
            for usuario_disponible in &self.usuarios_disponibles {
                if let Some(tx_usuario) = self.usuarios_conectados.get(usuario_disponible)
                    && tx_usuario
                        .send(MensajeUsuario::ActualizarEstadoUsuario(
                            usuario.clone(),
                            EstadoUsuario::Desconectado,
                        ))
                        .is_err()
                {
                    //En caso de que un usuario se desconecte repentinamente
                    if usuarios_a_desconectar.contains(&usuario_disponible.clone()) {
                        continue;
                    }
                    nuevos_desconectados.insert(usuario_disponible.clone());
                }
            }
        }

        self.desconectar_usuarios(nuevos_desconectados)
    }
}
