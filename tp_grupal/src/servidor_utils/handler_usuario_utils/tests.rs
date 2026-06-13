// #[cfg(test)]
// use crate::protocolos::pca::{estado::EstadoUsuarioPCA, mensaje::MensajePCA, usuario::UsuarioPCA};
// #[cfg(test)]
// use crate::servidor_utils::handler_usuario_utils::handler_usuario::HandlerUsuario;
// #[cfg(test)]
// use crate::servidor_utils::mensajes::mensaje_servidor::MensajeServidor;
// #[cfg(test)]
// use crate::servidor_utils::mensajes::mensaje_usuario::MensajeUsuario;
// #[cfg(test)]
// use crate::servidor_utils::stream_tcp_utils::stream_tcp::StreamTCP;
// #[cfg(test)]
// use crate::servidor_utils::{
//     servidor_central_utils::estado_usuario::EstadoUsuario,
//     stream_tcp_utils::stream_tcp_mock::MockStreamTCP,
// };
// #[cfg(test)]
// use std::collections::HashMap;
// #[cfg(test)]
// use std::time::Duration;
// ///TEST de funcionalidad de [`handler_usuario`]
// #[cfg(test)]
// use std::{
//     sync::{
//         Arc, Mutex,
//         mpsc::{self},
//     },
//     thread,
// };

// //LA ETAPA DE LOGIN NO PUEDE TESTEARSE PORQUE HABLA CON EL SERVIDOR Y ESPERA SU RTA -> TEST INTEGRACION !TODO!

// //TESTS PETICIONES DEL SERVIDOR
// #[test]
// fn test01_se_escucha_llamada_del_servidor_y_se_notifica_usuario() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(None));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             referencia_usuario_procesado,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     let usuario_llama = "melaniegl".to_string();
//     tx_usuario
//         .send(MensajeUsuario::LlamadaEntrante(usuario_llama.clone()))
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let escritura_esperada = String::from(MensajePCA::Llamando(usuario_llama));

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes());

//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, Some("melaniegl".to_string()));
// }

// #[test]
// //Si me esta llamando otro usuario pero estoy procesando una llamada con otro no se me notifica
// //sobre esta nueva llamada -> se le rechaza inmediatamente la llamada al otro usuario
// fn test02_llamada_entrante_y_procesando_llamada_con_otro_usuario() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             usuario_procesando_llamada,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     let usuario_llama = "melaniegl".to_string();
//     tx_usuario
//         .send(MensajeUsuario::LlamadaEntrante(usuario_llama.clone()))
//         .expect("Fallo al enviar mensaje por channel");

//     let respuesta = rx_servidor
//         .recv()
//         .expect("Fallo al recibir mensaje por channel");

//     //a mano pq no puedo usar el partialEq pq tengo senders en algunos campos del enum para MensajeServidor
//     let respuesta_recibida = respuesta.to_string();
//     let respuesta_esperada = MensajeServidor::RechazarLlamada(usuario_llama).to_string();

//     assert_eq!(respuesta_recibida, respuesta_esperada);

//     let escritura_esperada = vec![];

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada)
// }

// #[test]
// fn test03_se_nos_rechaza_llamada() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             referencia_usuario_procesado,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     tx_usuario
//         .send(MensajeUsuario::LlamadaRechazada)
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let escritura_esperada = String::from(MensajePCA::Rechazo);

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes());

//     //volvemos a poder procesar otra llamada
//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, None);
// }

// #[test]
// fn test04_se_nos_pide_el_offer_sdp() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             usuario_procesando_llamada,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     tx_usuario
//         .send(MensajeUsuario::PedirOffer)
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let escritura_esperada = String::from(MensajePCA::PedirOffer);

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes())
// }

// #[test]
// fn test05_se_nos_pide_el_answer_sdp_enviandonos_su_offer() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             usuario_procesando_llamada,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     let offer_sdp = String::from("~offer~");

//     tx_usuario
//         .send(MensajeUsuario::PedirAnswer(offer_sdp.clone()))
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let escritura_esperada = String::from(MensajePCA::Offer(offer_sdp));

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes())
// }

// #[test]
// fn test06_se_nos_envia_el_answer() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             usuario_procesando_llamada,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     let answer_sdp = String::from("~ una answer sdp ~");

//     tx_usuario
//         .send(MensajeUsuario::EnviarAnswer(answer_sdp.clone()))
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let escritura_esperada = String::from(MensajePCA::Answer(answer_sdp));

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes())
// }

// #[test]
// fn test07_se_nos_envia_la_actualizacion_del_estado_de_un_usuario() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(None));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             usuario_procesando_llamada,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     let usuario = "melanie".to_string();

//     let mensaje_servidor =
//         MensajeUsuario::ActualizarEstadoUsuario(usuario.clone(), EstadoUsuario::Ocupado);

//     tx_usuario
//         .send(mensaje_servidor)
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let usuario_actualizado = UsuarioPCA::new(
//         usuario.clone(),
//         EstadoUsuarioPCA::from(EstadoUsuario::Ocupado),
//     );

//     let mensaje_usuario = MensajePCA::UsuarioEstado(usuario_actualizado);

//     let escritura_esperada = String::from(mensaje_usuario);

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes())
// }

// #[test]
// fn test08_se_nos_envia_los_usuarios_y_su_estado() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let (tx_usuario, rx_usuario) = mpsc::channel::<MensajeUsuario>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(None));

//     let mock_stream = MockStreamTCP::new(Vec::new());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     thread::spawn(move || {
//         HandlerUsuario::escuchar_servidor_central(
//             writer_stream,
//             tx_servidor,
//             rx_usuario,
//             usuario_procesando_llamada,
//         )
//         .expect("Error escuchando mensajes del servidor");
//     });

//     //no puedo testear de mas pq el dicc puede variar el orden:(
//     let usuario1 = "melanie".to_string();

//     let mut dicc_estado = HashMap::new();
//     dicc_estado.insert(usuario1.clone(), EstadoUsuario::Disponible);

//     let mensaje_servidor = MensajeUsuario::EstadoUsuarios(dicc_estado);

//     tx_usuario
//         .send(mensaje_servidor)
//         .expect("Fallo al enviar mensaje por channel");

//     thread::sleep(std::time::Duration::from_millis(10000));

//     let usuario_pca1 = UsuarioPCA::new(
//         usuario1.clone(),
//         EstadoUsuarioPCA::from(EstadoUsuario::Disponible),
//     );

//     let mensaje_usuario = MensajePCA::Usuarios(vec![usuario_pca1]);

//     let escritura_esperada = String::from(mensaje_usuario);

//     let lock_escritura = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     assert_eq!(*lock_escritura, escritura_esperada.as_bytes())
// }

// //TESTS PETICIONES DEL USUARIO
// #[test]
// fn test09_le_rechazamos_llamada_a_usuario() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");

//     let mensaje_usuario = String::from(MensajePCA::Rechazo);
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(reader_stream, usuario, referencia_usuario_procesado)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::RechazarLlamada("mirkito".to_string());

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     //ademas volvemos a estar disponibles para nuevas llamadas
//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, None);
// }

// #[test]
// fn test10_se_corta_llamada_y_se_esta_disponible_para_nuevas() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");

//     let mensaje_usuario = String::from(MensajePCA::Cortar);
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(reader_stream, usuario, referencia_usuario_procesado)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::EstadoDisponible("melaniegl".to_string());

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     //ademas volvemos a estar disponibles para nuevas llamadas
//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, None);
// }

// #[test]
// fn test11_se_desconecta_notificandole_al_servidor() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(None));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");

//     let mensaje_usuario = String::from(MensajePCA::Salir);
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(reader_stream, usuario, referencia_usuario_procesado)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::Desconectarse("melaniegl".to_string());

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());
// }

// #[test]
// fn test12_se_llama_a_usuario() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(None));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");
//     let usuario_llamado = String::from("mirkito");

//     let mensaje_usuario = String::from(MensajePCA::Llamar(usuario_llamado.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(
//                 reader_stream,
//                 String::from("melaniegl"),
//                 referencia_usuario_procesado,
//             )
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::Llamar(usuario, usuario_llamado);

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     //pasa a tener a ese usuario como procesando en llamda
//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, Some("mirkito".to_string()));
// }

// #[test]
// fn test13_se_llama_a_usuario_estando_procesando_otro() {
//     let (tx_servidor, _rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario_llamado = String::from("larita");

//     let mensaje_usuario = String::from(MensajePCA::Llamar(usuario_llamado.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(
//                 reader_stream,
//                 String::from("melaniegl"),
//                 referencia_usuario_procesado,
//             )
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, Some("mirkito".to_string()));
// }

// #[test]
// fn test14_usuario_acepta_llamada_a_usuario_en_proceso() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");

//     let mensaje_usuario = String::from(MensajePCA::Aceptar);
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(reader_stream, usuario, referencia_usuario_procesado)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::AceptarLlamada("mirkito".to_string());

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     //se actualiza a que ahora se esta procesando una llamada con ese usuario
//     assert_eq!(*lock_usuario_procesado, Some("mirkito".to_string()));
// }

// #[test]
// fn test15_servidor_nos_pidio_offer_y_se_lo_envia_usuario() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");
//     let datita_offer = String::from("~offer~");

//     let mensaje_usuario = String::from(MensajePCA::Offer(datita_offer.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(reader_stream, usuario, referencia_usuario_procesado)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::EnviarOffer("mirkito".to_string(), datita_offer);

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     assert_eq!(*lock_usuario_procesado, Some("mirkito".to_string()));
// }

// #[test]
// fn test15_servidor_nos_pidio_answer_y_se_lo_envia_usuario() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();
//     let usuario_procesando_llamada = Arc::new(Mutex::new(Some("mirkito".to_string())));

//     let referencia_usuario_procesado = Arc::clone(&usuario_procesando_llamada);

//     let usuario = String::from("melaniegl");
//     let datita_answer = String::from("~answer~");

//     let mensaje_usuario = String::from(MensajePCA::Answer(datita_answer.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .escuchar_usuario(reader_stream, usuario, referencia_usuario_procesado)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::EnviarAnswer(
//         "melaniegl".to_string(),
//         "mirkito".to_string(),
//         datita_answer,
//     );

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     let lock_usuario_procesado = usuario_procesando_llamada
//         .lock()
//         .expect("Fallo al obtener lock");
//     assert_eq!(*lock_usuario_procesado, Some("mirkito".to_string()));
// }

// #[test]
// fn test16_pedimos_registrarnos() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

//     let usuario = String::from("melaniegl");
//     let contrasenia = String::from("clairo123");

//     let mensaje_usuario = String::from(MensajePCA::Registrar(usuario.clone(), contrasenia.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let mut writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream.clone());
//     let mut reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream);
//     //let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .etapa_login(&mut reader_stream, &mut writer_stream)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let (tx_juguete, _rx_juguete) = mpsc::channel::<MensajeUsuario>();

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::Registrar(
//         "melaniegl".to_string(),
//         "clairo123".to_string(),
//         tx_juguete.clone(),
//     );

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     let _ = match mensaje_recibido {
//         MensajeServidor::Registrar(_, _, tx_usuario) => tx_usuario,
//         _ => tx_juguete,
//     };
// }

// #[test]
// fn test17_pedimos_ingresar() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

//     let usuario = String::from("melaniegl");
//     let contrasenia = String::from("clairo123");

//     //TODO ver creo q rompe
//     let mensaje_usuario = String::from(MensajePCA::Entrar(usuario.clone(), contrasenia.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let mut writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream.clone());
//     let mut reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream.clone());

//     let mut handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .etapa_login(&mut reader_stream, &mut writer_stream)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     let (tx_juguete, _rx_juguete) = mpsc::channel::<MensajeUsuario>();

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::Loguear(
//         "melaniegl".to_string(),
//         "clairo123".to_string(),
//         tx_juguete.clone(),
//     );

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());
// }

// #[test]
// fn test18_ingresamos_salimos_etapa_login_nos_envian_usuarios() {
//     let (tx_servidor, rx_servidor) = mpsc::channel::<MensajeServidor>();

//     let usuario = String::from("melaniegl");
//     let contrasenia = String::from("clairo123");

//     //TODO ver creo q rompe
//     let mensaje_usuario = String::from(MensajePCA::Entrar(usuario.clone(), contrasenia.clone()));
//     let mensaje_usuario_bytes = Vec::from(mensaje_usuario);

//     let mock_stream = MockStreamTCP::new(mensaje_usuario_bytes);
//     let writer_stream: Box<dyn StreamTCP> = Box::new(mock_stream.clone());
//     let reader_stream: Box<dyn StreamTCP> = Box::new(mock_stream.clone());
//     let bytes_escritos = Arc::clone(&mock_stream.bytes_escritos);

//     let handler_usuario = HandlerUsuario::new(tx_servidor);

//     thread::spawn(move || {
//         handler_usuario
//             .gestionar(reader_stream, writer_stream)
//             .expect("Error escuchando mensajes del usuario");
//     });

//     thread::sleep(Duration::from_millis(10000));

//     let (tx_juguete, _rx_juguete) = mpsc::channel::<MensajeUsuario>();

//     let mensaje_recibido = rx_servidor.recv().expect("Fallo al recibir mensaje");

//     let mensaje_recibido_string = mensaje_recibido.to_string();
//     let mensaje_esperado = MensajeServidor::Loguear(
//         "melaniegl".to_string(),
//         "clairo123".to_string(),
//         tx_juguete.clone(),
//     );

//     assert_eq!(mensaje_recibido_string, mensaje_esperado.to_string());

//     let dicc: HashMap<String, EstadoUsuario> = HashMap::new();

//     if let MensajeServidor::Loguear(_, _, tx_usuario) = mensaje_recibido {
//         tx_usuario
//             .send(MensajeUsuario::Ok)
//             .expect("Fallo enviando mensaje");

//         tx_usuario
//             .send(MensajeUsuario::EstadoUsuarios(dicc))
//             .expect("Fallo enviando mensaje");
//     }

//     thread::sleep(Duration::from_millis(10000));

//     let lock_bytes_escritos = bytes_escritos.lock().expect("Fallo obteniendo el lock");

//     let mensaje = String::from(MensajePCA::Usuarios(Vec::new()));

//     assert_eq!(*lock_bytes_escritos, mensaje.as_bytes())
// }
