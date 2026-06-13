use eframe::egui::{self, Align2, Color32, RichText, Ui, Vec2};

use crate::{
    aplicacion::EventoAplicacion,
    vista::pantallas::pantalla::{AccionPantalla, Pantalla},
};

#[derive(Default)]
pub struct PantallaLogin {
    usuario: String,
    contrasenia: String,
    error_iniciando_sesion: bool,
}

impl Pantalla for PantallaLogin {
    fn renderizar(&mut self, ctx: &eframe::egui::Context) -> AccionPantalla {
        let mut accion = AccionPantalla::Ninguna;

        egui::Window::new("Iniciar Sesión")
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    ui.label(RichText::new("Ingresá tus datos").strong().size(18.0));
                    ui.add_space(15.0);

                    ui.label("Usuario:");
                    ui.text_edit_singleline(&mut self.usuario);

                    ui.add_space(10.0);

                    ui.label("Contraseña:");
                    ui.add(egui::TextEdit::singleline(&mut self.contrasenia).password(true));

                    ui.add_space(10.0);

                    // if let Some(err) = app.get_login_error() {
                    //     ui.colored_label(Color32::RED, err);
                    // }

                    ui.add_space(15.0);

                    if ui
                        .button(RichText::new("Iniciar sesión").strong().size(16.0))
                        .clicked()
                        && !self.usuario.is_empty()
                        && !self.contrasenia.is_empty()
                    {
                        accion = AccionPantalla::IntentarLogin(
                            String::from(&self.usuario),
                            String::from(&self.contrasenia),
                        );
                    }

                    ui.add_space(10.0);

                    if ui.button(RichText::new("Volver").size(15.0)).clicked() {
                        accion = AccionPantalla::Volver;
                    }

                    if self.error_iniciando_sesion {
                        self.mostrar_error(ui);
                    }
                });
            });

        accion
    }

    fn escuchar_evento(&mut self, evento: crate::aplicacion::EventoAplicacion) {
        if let EventoAplicacion::ErrorIniciandoSesion(_error_str) = evento {
            self.error_iniciando_sesion = true;
        }
    }
}

impl PantallaLogin {
    fn mostrar_error(&mut self, ui: &mut Ui) {
        ui.colored_label(
            Color32::from_rgb(255, 80, 80),
            RichText::new("Fallo al iniciar sesion. Intente nuevamente")
                .size(18.0)
                .strong(),
        );
    }
}
