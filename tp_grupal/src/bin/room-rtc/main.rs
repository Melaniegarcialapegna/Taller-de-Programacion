use eframe::{NativeOptions, egui};
use egui_extras::install_image_loaders;
use room_rtc_2c_25::{
    aplicacion::Aplicacion,
    comunicacion::comunicador_tcp::ComunicadorTCP,
    config_room_rtc::ConfigRoomRTC,
    logger::Logger,
    vista::{customizacion::configurar_theme, vista_eframe::VistaEframe},
};

fn main() {
    if let Err(error_str) = inicializar_aplicacion() {
        eprintln!("{}", error_str);
    };
}

fn inicializar_aplicacion() -> Result<(), String> {
    let parametros = std::env::args().collect::<Vec<String>>();

    if parametros.len() != 2 {
        return Err(
            "El programa espera únicamente el archivo de configuración por parámetro.".to_string(),
        );
    }

    let ruta = &parametros[1];

    let config = ConfigRoomRTC::almacenar_config(ruta)
        .map_err(|_| "Error cargando archivo de configuracion")?;

    // Creo la aplicacion
    let aplicacion = obtener_aplicacion(config)?;

    // Creo la vista
    let vista =
        VistaEframe::crear_vista(aplicacion).map_err(|_| "Fallo creando la vista".to_string())?;

    // Muestro ventana de la aplicacion
    let native_options = obtener_native_options();
    eframe::run_native(
        "Room RTC",
        native_options,
        Box::new(|ctx| {
            install_image_loaders(&ctx.egui_ctx);
            configurar_theme(&ctx.egui_ctx);
            Ok(Box::new(vista))
        }),
    )
    .map_err(|_| "Fallo al renderizar la vista".to_string())?;

    Ok(())
}

fn obtener_aplicacion(config: ConfigRoomRTC) -> Result<Aplicacion, String> {
    let comunicador = ComunicadorTCP::crear_comunicador(config.getter_direccion_signaling())
        .map_err(|_| "Error creando comunicador".to_string())?;

    let logger = Logger::new(config.getter_log_file());

    let aplicacion = Aplicacion::new(Box::new(comunicador), config, logger)
        .map_err(|_| "Error creando aplicacion")?;
    Ok(aplicacion)
}

fn obtener_native_options() -> NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Room RTC - Cliente")
            .with_maximized(false)
            .with_inner_size([1280.0, 900.0])
            .with_resizable(true),
        ..Default::default()
    }
}
