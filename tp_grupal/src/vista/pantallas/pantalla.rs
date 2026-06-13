use eframe::egui::Context;
use std::path::PathBuf;

use crate::aplicacion::EventoAplicacion;

/// Acciones que resultan de apretar algun boton (o ejecutar una acción) en una [Pantalla].
pub enum AccionPantalla {
    /// Representa que no se apreto ningun boton
    Ninguna,
    /// Se desea ir a la [Pantalla] de inicio de sesion
    IrALogin,
    /// Se desea ir a la [Pantalla] de registro
    IrARegistro,
    /// Se desea iniciar sesion en la aplicacion con el usuario y contrasenia especificados
    IntentarLogin(String, String),
    /// Se desea volver desde las pantallas de inicio de sesion/registro a la pantalla de inicio.
    Volver,
    /// Se desea registrarse en la aplicacion con el usuario y contrasenia especificados
    IntentarRegistro(String, String),
    /// Se desea llamar al usuario con el nombre de usuario especificado
    Llamar(String),
    /// Se desea atender la llamada que se esta recibiendo.
    AtenderLlamada,
    /// Se desea rechazar la llamada que se esta recibiendo.
    RechazarLlamada,
    /// Se desea pedir un nuevo frame de las camaras de la videollamada
    NuevoFrame,
    /// Se desea cortar la llamada en curso
    CortarLlamada,
    /// Se desea actualizar la lista de usuarios actual
    PedirUsuarios,
    /// Se desea actualizar la lista de camaras disponibles
    PedirListaDeCamaras,
    /// Se desea cambiar la camara que se usara para las videollamadas
    CambiarCamara(String),
    /// Se desea cerrar la sesion actual
    CerrarSesion,
    /// Se desea mutear el microfono
    MutearMicrofono,
    /// Se desea desmutear el microfono
    DesmutearMicrofono,
    /// Se desea abrir el dialogo para seleccionar un archivo
    AbrirDialogoArchivo,
    /// Se desea enviar el archivo en el path especificado
    EnviarArchivo(PathBuf),
    /// Se desea aceptar la oferta de archivo recibida del peer
    AceptarArchivo,
    /// Se desea rechazar la oferta de archivo recibida del peer
    RechazarArchivo,
}

/// Este trait representa la interfaz minima necesaria para que una pantalla pueda ser renderizada por [VistaEframe](crate::vista::vista_eframe::VistaEframe).
/// El proposito del trait es que las pantallas tengan el menor acoplamiento entre si. [VistaEframe](crate::vista::vista_eframe::VistaEframe) contendra una
/// referencia a la pantalla actual que se esta mostrando (un [Box<dyn Pantalla>]). El funcionamiento implementado es el siguiente:
///
/// - El metodo [Pantalla::renderizar()] sera ejecutado por [VistaEframe](crate::vista::vista_eframe::VistaEframe) en cada actualización, mostrando en la
///   interfaz el contenido de la pantalla. Este metodo retorna una [AccionPantalla], que determinara si se hizo en la pantalla alguna acción.
///   La vista debera tener un handler asociado para cada variante de [AccionPantalla].
///
/// - El metodo [Pantalla::escuchar_evento] permite que el estado interno de una pantalla se actualice según un evento ocurrido en la aplicación. El caso tipico
///   de uso será para informar si una acción se ejecuto correctamente en la aplicación. Otro caso podría ser si se recibe una llamada; la aplicación enviara a la vista
///   un EventoAplicación, y ademas la pantalla actual de la vista debera cambiar su estado para mostrar la llamada. En este ultimo caso,
///   [VistaEframe](crate::vista::vista_eframe::VistaEframe) recibira el evento e inmediatamente se lo informara a la pantalla para que cambie su estado interno.
pub trait Pantalla {
    /// Recibe un contexto donde mostrar la pantalla. Muestra la pantalla según el estado interno,
    /// y devuelve una [AccionPantalla] indicando si se realizo algúna acción en la interfaz
    /// grafica.
    fn renderizar(&mut self, ctx: &Context) -> AccionPantalla;

    /// Escucha un evento de la aplicación, y de ser necesario actualiza el estado interno
    /// de la aplicación según ese evento.
    fn escuchar_evento(&mut self, evento: EventoAplicacion);
}
