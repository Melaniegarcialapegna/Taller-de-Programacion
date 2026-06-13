use eframe::egui::{CentralPanel, Context, Image, RichText, Ui, include_image};

use crate::{
    aplicacion::EventoAplicacion,
    vista::pantallas::pantalla::{AccionPantalla, Pantalla},
};

#[derive(Default)]
pub struct PantallaInicio {}

/// Renderiza la pantalla principal del programa (logo + botones registrar o iniciar sesión).
///
/// Devuelve una AccionPantallaPrincipal, que puede ser un click en alguno de los dos botones, o Ninguna.
impl Pantalla for PantallaInicio {
    fn renderizar(&mut self, ctx: &Context) -> AccionPantalla {
        let mut accion = AccionPantalla::Ninguna;

        CentralPanel::default()
            .frame(eframe::egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    self.mostrar_logo(ui);
                    self.mostrar_titulo(ui);
                    accion = self.mostrar_botones(ui);
                });
            });

        accion
    }

    fn escuchar_evento(&mut self, _evento: EventoAplicacion) {
        // Esta pantalla no efectua cambios ante ningun evento
    }
}

impl PantallaInicio {
    fn mostrar_logo(&mut self, ui: &mut Ui) {
        let logo = include_image!("../assets/logo.png");
        ui.add(Image::new(logo).max_width(700.0).shrink_to_fit());
        ui.add_space(-100.0);
    }

    fn mostrar_titulo(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Bienvenid@ a RoomRTC").size(28.0).strong());
        ui.add_space(10.0);
    }

    fn mostrar_botones(&mut self, ui: &mut Ui) -> AccionPantalla {
        let mut accion = AccionPantalla::Ninguna;

        if ui
            .button(RichText::new("Iniciar sesión").size(22.0).strong())
            .clicked()
        {
            accion = AccionPantalla::IrALogin;
        }

        ui.add_space(10.0);

        if ui
            .button(RichText::new("Registrarme").size(22.0).strong())
            .clicked()
        {
            accion = AccionPantalla::IrARegistro;
        }

        accion
    }
}
