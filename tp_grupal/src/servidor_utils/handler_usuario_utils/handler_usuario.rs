use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use crate::servidor_utils::mensajes::mensaje_servidor::MensajeServidor;
use crate::servidor_utils::mensajes::mensaje_usuario::MensajeUsuario;
use crate::servidor_utils::stream_tcp_utils::stream_tcp::StreamTCP;
use crate::{
    encriptacion::SistemaDeEncriptacion, servidor_utils::handler_usuario_utils::error::ErrorUsuario,
};
use crate::{
    protocolos::pca::{estado::EstadoUsuarioPCA, mensaje::MensajePCA, usuario::UsuarioPCA},
    servidor_utils::servidor_central_utils::estado_usuario::EstadoUsuario,
};

/// Hace de intermediario entre el SERVIDOR CENTRAL y el USUARIO
/// Gestiona y envia las peticiones del usuario al servidor central y viceversa
pub struct HandlerUsuario {
    tx_servidor: Sender<MensajeServidor>,
    encriptador: SistemaDeEncriptacion,
}

impl HandlerUsuario {
    //Se crea una nueva instancia de HandlerUsuario
    pub fn new(tx_servidor: Sender<MensajeServidor>, encriptador: SistemaDeEncriptacion) -> Self {
        HandlerUsuario {
            tx_servidor,
            encriptador,
        }
    }

    //Para inicial al HandlerUsuario se llama a esta funcion
    pub fn gestionar(
        mut self,
        mut reader_stream: Box<dyn StreamTCP>,
        mut writer_stream: Box<dyn StreamTCP>,
    ) -> Result<(), ErrorUsuario> {
        loop {
            // Creo una copia de los streams
            let mut clon_reader = reader_stream
                .clonar()
                .map_err(|_| ErrorUsuario::ErrorInterno)?;
            let mut clon_writer = writer_stream
                .clonar()
                .map_err(|_| ErrorUsuario::ErrorInterno)?;

            //Primero es necesario pasar por la etapa de login
            let (usuario, rx_usuario) = self.etapa_login(&mut clon_reader, &mut clon_writer)?;
            //Luego una vez logueado el usuario se lo pasa a conectado y se gestiona su conexion
            self.etapa_conexion(clon_reader, clon_writer, usuario, rx_usuario)?;
        }
    }

    fn leer_mensaje(
        &mut self,
        buffer: &mut String,
        reader_stream: &mut Box<dyn StreamTCP>,
    ) -> Result<usize, ErrorUsuario> {
        let bytes_leidos = reader_stream
            .leer_mensaje(buffer, &mut self.encriptador)
            .map_err(|_| ErrorUsuario::LecturaStream)?;

        Ok(bytes_leidos)
    }

    // Se encarga de que el usuario este logueado antes de pasar a la etapa de conexion.
    //Por decision de diseño es necesario loguearse para poder ir a la etapa de conexion, si se registra unicamente
    //se carga la informacion de ese nuevo usuario en el servidor pero no se conecta inmediatamente, es necesario
    //luego loguearse.
    pub fn etapa_login(
        &mut self,
        reader_stream: &mut Box<dyn StreamTCP>,
        writer_stream: &mut Box<dyn StreamTCP>,
    ) -> Result<(String, Receiver<MensajeUsuario>), ErrorUsuario> {
        let mut buffer = String::new();
        //por este medio se establece la conexion con el servidor
        let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
        loop {
            buffer.clear();

            let bytes_leidos = self.leer_mensaje(&mut buffer, reader_stream)?;
            if bytes_leidos == 0 {
                return Err(ErrorUsuario::ObteniendoLock);
            }

            let linea = buffer.trim();

            if linea.is_empty() {
                continue;
            }

            let mensaje = match MensajePCA::try_from(linea) {
                Ok(mensaje) => mensaje,
                Err(_) => continue, // mensaje mal formado, lo ignoramos
            };

            match mensaje {
                MensajePCA::Registrar(nombre_usuario, contrasenia) => {
                    self.usuario_pide_registrarse(
                        nombre_usuario,
                        contrasenia,
                        tx_usuario.clone(),
                        &rx_usuario,
                        writer_stream,
                    )?;
                    continue;
                }
                MensajePCA::Entrar(nombre_usuario, contrasenia) => {
                    let entrada_valida = self.usuario_pide_entrar(
                        nombre_usuario.clone(),
                        contrasenia,
                        tx_usuario.clone(),
                        &rx_usuario,
                        writer_stream,
                    )?;
                    if entrada_valida {
                        return Ok((nombre_usuario, rx_usuario));
                    }
                    continue;
                }
                _ => {
                    //hasta antes de loguearse no se puede enviar de otro tipo
                    println!("Recibimos algo que no corresponde a registro/login");
                    return Err(ErrorUsuario::MensajeFueraDeContexto);
                }
            }
        }
    }

    //luego de la etapa de login se pasa a la etapa de conexion.
    //en esta se escucha tanto peticiones del usuario como del servidor central, ya que
    //entre otras cosas el cliente podria querer llamar a alguien pero tambien alguien podria querer llamarnos.
    fn etapa_conexion(
        &mut self,
        reader_stream: Box<dyn StreamTCP>,
        writer_stream: Box<dyn StreamTCP>,
        usuario: String,
        rx_usuario: Receiver<MensajeUsuario>,
    ) -> Result<(), ErrorUsuario> {
        //en un momento determinado unicamente se puede estar procesando una llamada con un unico usuario
        let usuario_procesando_llamada: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let referencia_usuario = Arc::clone(&usuario_procesando_llamada);
        let clon_usuario = usuario.clone();

        let ref_tx_servidor = self.tx_servidor.clone();
        let ref_encriptador = self.encriptador.clone();
        //Se escuchan peticiones del servidor
        thread::spawn(move || {
            if Self::escuchar_servidor_central(
                writer_stream,
                ref_tx_servidor,
                rx_usuario,
                referencia_usuario,
                ref_encriptador,
                clon_usuario,
            )
            .is_err()
            {
                eprint!("Error al escuchar mensajes del servidor central")
            }
        });

        //Se escuchan peticiones del usuario
        self.escuchar_usuario(reader_stream, usuario, usuario_procesando_llamada)?;
        Ok(())
    }

    //Se escuchan y gestionan peticiones del servidor central comunicandoselo al usuario por medio del Stream.
    pub fn escuchar_servidor_central(
        mut stream_escritura: Box<dyn StreamTCP>,
        tx_servidor: Sender<MensajeServidor>,
        rx_usuario: Receiver<MensajeUsuario>,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
        mut encriptador: SistemaDeEncriptacion,
        usuario: String,
    ) -> Result<(), ErrorUsuario> {
        for mensaje in rx_usuario {
            match mensaje {
                MensajeUsuario::LlamadaEntrante(usuario_entrante) => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    Self::servidor_notifica_llamada_entrante(
                        usuario_entrante,
                        referencia,
                        tx_servidor.clone(),
                        &mut stream_escritura,
                        &mut encriptador,
                        usuario.clone(),
                    )?;
                }

                MensajeUsuario::LlamadaRechazada => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    Self::servidor_notifica_llamada_rechazada(
                        &mut stream_escritura,
                        referencia,
                        &mut encriptador,
                        tx_servidor.clone(),
                        usuario.clone(),
                    )?;
                }

                MensajeUsuario::PedirOffer => {
                    Self::servidor_notifica_se_pide_offer(&mut stream_escritura, &mut encriptador)?;
                }

                MensajeUsuario::PedirAnswer(offer_sdp) => {
                    Self::servidor_notifica_se_pide_answer(
                        offer_sdp,
                        &mut stream_escritura,
                        &mut encriptador,
                    )?;
                }

                MensajeUsuario::EnviarAnswer(answer_sdp) => {
                    Self::servidor_envia_answer_del_otro_peer(
                        answer_sdp,
                        &mut stream_escritura,
                        &mut encriptador,
                    )?;
                }

                MensajeUsuario::ActualizarEstadoUsuario(usuario_cambio_estado, estado) => {
                    Self::servidor_notifica_sobre_actualizacion_de_estado_usuario(
                        usuario_cambio_estado,
                        estado,
                        &mut stream_escritura,
                        &mut encriptador,
                        tx_servidor.clone(),
                        usuario.clone(),
                    )?;
                }

                MensajeUsuario::EstadoUsuarios(usuarios) => {
                    Self::servidor_envia_estado_usuarios(
                        usuarios,
                        &mut stream_escritura,
                        &mut encriptador,
                        tx_servidor.clone(),
                        usuario.clone(),
                    )?;
                }
                MensajeUsuario::Ok => {
                    //si llega a este es pq se corto la llamada
                    Self::servidor_nos_desconecto(&mut stream_escritura, &mut encriptador)?;
                    return Ok(());
                }
                _ => {
                    //No deberia
                    println!("Recibimos algo que no corresponde a la etapa de conexion");
                    return Err(ErrorUsuario::MensajeFueraDeContexto);
                }
            }
        }
        Ok(())
    }

    //Se escuchan peticiones del usuario por medio del stream y se las comunica al servidor central
    //por medio de un channel
    pub fn escuchar_usuario(
        &mut self,
        mut reader_stream: Box<dyn StreamTCP>,
        usuario: String,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
    ) -> Result<(), ErrorUsuario> {
        let mut buffer = String::new();
        loop {
            buffer.clear();

            let bytes_leidos = self.leer_mensaje(&mut buffer, &mut reader_stream)?;
            if bytes_leidos == 0 {
                break;
            }

            let linea = buffer.trim();

            if linea.is_empty() {
                continue;
            }

            let mensaje = match MensajePCA::try_from(linea) {
                Ok(mensaje) => mensaje,
                Err(_) => {
                    // Habria que loguear
                    dbg!("INFO: Se recibio un mensaje invalido (fallo al parsearlo)");
                    continue;
                } // mensaje mal formado, lo ignoramos
            };

            match mensaje {
                MensajePCA::Rechazo => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    self.usuario_rechaza_llamada(referencia)?;
                }

                MensajePCA::Cortar => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    self.usuario_corta_llamada(usuario.clone(), referencia)?;
                }

                MensajePCA::Salir => {
                    self.usuario_sale(usuario.clone())?;
                    return Ok(());
                }

                MensajePCA::Llamar(usuario_llamado) => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    self.usuario_llama_a_otro_usuario(
                        usuario.clone(),
                        usuario_llamado,
                        referencia,
                    )?;
                }

                MensajePCA::Aceptar => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    self.usuario_le_acepta_llamada(referencia)?;
                }

                MensajePCA::Offer(offer_sdp) => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    self.usuario_envia_su_offer(referencia, offer_sdp)?;
                }

                MensajePCA::Answer(answer_sdp) => {
                    let referencia = Arc::clone(&usuario_procesando_llamada);
                    self.usuario_envia_su_answer(usuario.clone(), referencia, answer_sdp)?;
                }

                _ => {
                    //no deberiamos recibir de otro tipo en esta instancia
                    println!("Recibimos algo que no corresponde a la etapa de conexion");
                    return Err(ErrorUsuario::MensajeFueraDeContexto);
                }
            }
        }
        Ok(())
    }

    //usuario pide registrarse por lo que se le notifica al servidor por medio del mensaje MensajeServidor::Registrar
    fn usuario_pide_registrarse(
        &mut self,
        nombre_usuario: String,
        contrasenia: String,
        tx_usuario: Sender<MensajeUsuario>,
        rx_usuario: &Receiver<MensajeUsuario>,
        writer_stream: &mut Box<dyn StreamTCP>,
    ) -> Result<(), ErrorUsuario> {
        //se le notifica al servidor
        self.tx_servidor
            .send(MensajeServidor::Registrar(
                nombre_usuario.clone(),
                contrasenia,
                tx_usuario,
            ))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;

        //se escucha de manera bloqueante la respuesta del servidor
        let respuesta = rx_usuario
            .recv()
            .map_err(|_| ErrorUsuario::RecibiendoMensajeServidorCentral)?;

        match respuesta {
            MensajeUsuario::Ok => {
                writer_stream
                    .enviar_mensaje(MensajePCA::Registrado, &mut self.encriptador)
                    .map_err(|_| ErrorUsuario::EnviandoMensajeUsuarioPorStream)?;
                Ok(())
            }
            MensajeUsuario::Error(error) => {
                writer_stream
                    .enviar_mensaje(MensajePCA::ErrorPCA(error), &mut self.encriptador)
                    .map_err(|_| ErrorUsuario::EnviandoMensajeUsuarioPorStream)?;
                Ok(())
            }
            _ => Err(ErrorUsuario::MensajeFueraDeContexto),
        }
    }

    //usuario pide loguearse por lo que se le notifica al servidor por medio del mensaje MensajeServidor::Loguear
    fn usuario_pide_entrar(
        &mut self,
        nombre_usuario: String,
        contrasenia: String,
        tx_usuario: Sender<MensajeUsuario>,
        rx_usuario: &Receiver<MensajeUsuario>,
        writer_stream: &mut Box<dyn StreamTCP>,
    ) -> Result<bool, ErrorUsuario> {
        //se le notifica al servidor5
        self.tx_servidor
            .send(MensajeServidor::Loguear(
                nombre_usuario.clone(),
                contrasenia,
                tx_usuario,
            ))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;

        //esperamos respuesta del servidor
        let respuesta = rx_usuario
            .recv()
            .map_err(|_| ErrorUsuario::RecibiendoMensajeServidorCentral)?;
        match respuesta {
            MensajeUsuario::Ok => {
                //sale de la etapa de login
                Ok(true)
            }
            MensajeUsuario::Error(error) => {
                writer_stream
                    .enviar_mensaje(MensajePCA::ErrorPCA(error), &mut self.encriptador)
                    .map_err(|_| ErrorUsuario::EnviandoMensajeUsuarioPorStream)?;
                Ok(false)
            }
            _ => Err(ErrorUsuario::MensajeFueraDeContexto),
        }
    }

    //usuario pide rechazar una llamada entrante
    fn usuario_rechaza_llamada(
        &mut self,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
    ) -> Result<(), ErrorUsuario> {
        //avisar a servidor que se rechazo la llamada y eliminamos al usuario procesado
        let mut usuario_rechazado = String::from("momentaneo");
        {
            let mut lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            if let Some(ref usuario) = *lock_usuario {
                usuario_rechazado = usuario.clone();
            }

            *lock_usuario = None;
        } //libero lock

        self.tx_servidor
            .send(MensajeServidor::RechazarLlamada(usuario_rechazado))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //usuario nos avisa que corto la llamada por lo tanto vuelve a estar disponible
    fn usuario_corta_llamada(
        &mut self,
        usuario: String,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
    ) -> Result<(), ErrorUsuario> {
        {
            let mut lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            *lock_usuario = None;
        } //libero lock

        self.tx_servidor
            .send(MensajeServidor::EstadoDisponible(usuario))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //usuario le notifica al servidor que saldra de la aplicacion por lo que pasara a estar desconectado
    fn usuario_sale(&mut self, usuario: String) -> Result<(), ErrorUsuario> {
        //avisar que nos desconectamos
        self.tx_servidor
            .send(MensajeServidor::Desconectarse(usuario))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //usuario pide llamar a otro usuario.
    //unicamente se puede hacer si no estamos procesando otra llamada
    fn usuario_llama_a_otro_usuario(
        &mut self,
        usuario: String,
        usuario_llamado: String,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
    ) -> Result<(), ErrorUsuario> {
        {
            let mut lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            match *lock_usuario {
                Some(_) => {
                    //estamos procesando una llamada
                    return Ok(()); //sigo con proximo mensaje
                }
                None => {
                    //lo pongo como usuario con el que procesare una llamada
                    *lock_usuario = Some(usuario_llamado.clone());
                }
            }
        } //libero lock

        self.tx_servidor
            .send(MensajeServidor::Llamar(usuario.clone(), usuario_llamado))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //usuario notifica que acepto la llamada entrante
    fn usuario_le_acepta_llamada(
        &mut self,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
    ) -> Result<(), ErrorUsuario> {
        let mut usuario_llamada = String::from("momentaneo");
        {
            let lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            if let Some(ref usuario) = *lock_usuario {
                usuario_llamada = usuario.clone();
            }
        } //libero lock

        self.tx_servidor
            .send(MensajeServidor::AceptarLlamada(usuario_llamada))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //usuario envia su offer para que el servidor se la envie al otro usuario
    fn usuario_envia_su_offer(
        &mut self,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
        offer_sdp: String,
    ) -> Result<(), ErrorUsuario> {
        let mut usuario_llamada = String::from("momentaneo");
        {
            let lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            if let Some(ref usuario) = *lock_usuario {
                usuario_llamada = usuario.clone();
            }
        } //libero lock

        self.tx_servidor
            .send(MensajeServidor::EnviarOffer(usuario_llamada, offer_sdp))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //usuario nos envia su answer para enviarsela al otro usuario.
    //(ya recibimos su offer previamente)
    fn usuario_envia_su_answer(
        &mut self,
        usuario: String,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
        answer_sdp: String,
    ) -> Result<(), ErrorUsuario> {
        let mut usuario_llamada = String::from("momentaneo");
        {
            let lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            if let Some(ref usuario) = *lock_usuario {
                usuario_llamada = usuario.clone();
            }
        } //libero lock

        self.tx_servidor
            .send(MensajeServidor::EnviarAnswer(
                usuario.clone(),
                usuario_llamada,
                answer_sdp,
            ))
            .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        Ok(())
    }

    //el servidor notifica que hay un usuario que quiere llamarnos, se lo notificamos al usuario poro medio del stream.
    //Si estamos procesando una llamada con otro usuario le rechazamos la llamada entrante inmediatamente
    fn servidor_notifica_llamada_entrante(
        usuario_entrante: String,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
        tx_servidor: Sender<MensajeServidor>,
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
        usuario: String,
    ) -> Result<(), ErrorUsuario> {
        let clon_usuario_entrante = usuario_entrante.clone();
        {
            let mut lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            match *lock_usuario {
                Some(_) => {
                    //le rechazo la llamada
                    tx_servidor
                        .send(MensajeServidor::RechazarLlamada(usuario_entrante))
                        .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
                    return Ok(()); //sigo con proximo mensaje
                }
                None => {
                    //lo pongo como usuario con el que procesare una llamada
                    *lock_usuario = Some(usuario_entrante.clone());
                }
            }
        } //libero lock

        //avisarle al usuario por medio del stream que un usuario_entrante quiere comunicarse con nosotros y ponerlo en usuario procesando llamda, si el lugar esta ocupado no se puede procesar ya que estmos procesando otra llamada
        if stream_escritura
            .enviar_mensaje(MensajePCA::Llamando(usuario_entrante), encriptador)
            .is_err()
        {
            //si cuando intentamos enviar un mensaje no se puede ya que el usuario se desconecto repentinamente
            //avisamos al servidor que lo ponga como desconectado
            tx_servidor
                .send(MensajeServidor::Desconectarse(usuario))
                .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;

            //ademas se le rechaza la llamada al usuario que nos llamo
            tx_servidor
                .send(MensajeServidor::RechazarLlamada(clon_usuario_entrante))
                .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
            return Ok(()); //sigo con proximo mensaje
        }
        Ok(())
    }

    //servidor nos notifica que usuario al que le pedimos hacer llamada nos la rechazo
    fn servidor_notifica_llamada_rechazada(
        stream_escritura: &mut Box<dyn StreamTCP>,
        usuario_procesando_llamada: Arc<Mutex<Option<String>>>,
        encriptador: &mut SistemaDeEncriptacion,
        tx_servidor: Sender<MensajeServidor>,
        usuario: String,
    ) -> Result<(), ErrorUsuario> {
        //avisar que el usuario con el que queriamos hacer llamada nos la rechazo y poner en none al usuario de
        //llamada procesada

        if stream_escritura
            .enviar_mensaje(MensajePCA::Rechazo, encriptador)
            .is_err()
        {
            //si cuando intentamos enviar un mensaje no se puede ya que el usuario se desconecto repentinamente
            //avisamos al servidor que lo ponga como desconectado
            tx_servidor
                .send(MensajeServidor::Desconectarse(usuario))
                .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        }

        {
            let mut lock_usuario = usuario_procesando_llamada
                .lock()
                .map_err(|_| ErrorUsuario::ObteniendoLock)?;
            *lock_usuario = None;
        } //libero lock

        Ok(())
    }

    //El servidor nos pide el offer para enviarsela al otro usuario
    fn servidor_notifica_se_pide_offer(
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<(), ErrorUsuario> {
        stream_escritura
            .enviar_mensaje(MensajePCA::PedirOffer, encriptador)
            .map_err(|_| ErrorUsuario::EnviandoMensajeUsuarioPorStream)?;
        Ok(())
    }

    //servidor nos pide el answer para enviarsela al otro usuario enviandonos tambien el offer de este
    fn servidor_notifica_se_pide_answer(
        offer_sdp: String,
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<(), ErrorUsuario> {
        stream_escritura
            .enviar_mensaje(MensajePCA::Offer(offer_sdp), encriptador)
            .map_err(|_| ErrorUsuario::EnviandoMensajeUsuarioPorStream)?;

        Ok(())
    }
    fn servidor_envia_answer_del_otro_peer(
        answer_sdp: String,
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<(), ErrorUsuario> {
        //le enviamos la answer del otro usuario (fuimos nosotros los que enviamos la offer)
        stream_escritura
            .enviar_mensaje(MensajePCA::Answer(answer_sdp), encriptador)
            .map_err(|_| ErrorUsuario::EnviandoMensajeUsuarioPorStream)?;
        Ok(())
    }

    //servidor nos envia una notificacion con el cambio de estado de un usuario en particular
    fn servidor_notifica_sobre_actualizacion_de_estado_usuario(
        usuario_a_actualizar: String,
        estado: EstadoUsuario,
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
        tx_servidor: Sender<MensajeServidor>,
        usuario: String,
    ) -> Result<(), ErrorUsuario> {
        let usuario_a_actualizar =
            UsuarioPCA::new(usuario_a_actualizar, EstadoUsuarioPCA::from(estado));
        if stream_escritura
            .enviar_mensaje(MensajePCA::UsuarioEstado(usuario_a_actualizar), encriptador)
            .is_err()
        {
            //si cuando intentamos enviar un mensaje no se puede ya que el usuario se desconecto repentinamente
            //avisamos al servidor que lo ponga como desconectado
            tx_servidor
                .send(MensajeServidor::Desconectarse(usuario))
                .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        }
        Ok(())
    }

    //servidor nos envia el estado de todos los usuarios registrados en el
    fn servidor_envia_estado_usuarios(
        usuarios: HashMap<String, EstadoUsuario>,
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
        tx_servidor: Sender<MensajeServidor>,
        usuario: String,
    ) -> Result<(), ErrorUsuario> {
        let mut vector_usuarios = Vec::new();

        for (usuario, estado) in usuarios.iter() {
            let clon_usuario = usuario.clone();
            let clon_estado = estado.clone();
            vector_usuarios.push(UsuarioPCA::new(
                clon_usuario,
                EstadoUsuarioPCA::from(clon_estado),
            ));
        }
        if stream_escritura
            .enviar_mensaje(MensajePCA::Usuarios(vector_usuarios), encriptador)
            .is_err()
        {
            //si cuando intentamos enviar un mensaje no se puede ya que el usuario se desconecto repentinamente
            //avisamos al servidor que lo ponga como desconectado
            tx_servidor
                .send(MensajeServidor::Desconectarse(usuario))
                .map_err(|_| ErrorUsuario::EnviandoMensajeServidorCentral)?;
        }
        Ok(())
    }

    //servidor nos desconecto
    fn servidor_nos_desconecto(
        stream_escritura: &mut Box<dyn StreamTCP>,
        encriptador: &mut SistemaDeEncriptacion,
    ) -> Result<(), ErrorUsuario> {
        if stream_escritura
            .enviar_mensaje(MensajePCA::Salio, encriptador)
            .is_err()
        {
            //no me interesa
        }
        Ok(())
    }
}
