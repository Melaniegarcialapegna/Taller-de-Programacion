use eframe::egui::{Button, CentralPanel, Color32, Context, RichText, TextEdit, Ui, vec2};

use crate::{
    aplicacion::EventoAplicacion,
    vista::pantallas::pantalla::{AccionPantalla, Pantalla},
};

#[derive(Default)]
pub struct PantallaRegistro {
    usuario: String,
    contrasenia: String,
    registro_exitoso: bool,
    error_registro: Option<String>,
}

impl Pantalla for PantallaRegistro {
    fn renderizar(&mut self, ctx: &Context) -> AccionPantalla {
        let mut accion = AccionPantalla::Ninguna;

        CentralPanel::default()
            .frame(eframe::egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    self.mostrar_titulo(ui);
                    ui.add_space(20.0);

                    self.mostrar_input_usuario(ui);
                    ui.add_space(10.0);

                    self.mostrar_input_password(ui);
                    ui.add_space(20.0);

                    accion = self.mostrar_botones(ui);
                    ui.add_space(10.0);
                });

                if self.registro_exitoso {
                    self.mostrar_registro_exitoso(ui);
                }

                if let Some(_error) = &self.error_registro {
                    self.mostrar_error(ui);
                }
            });

        accion
    }

    fn escuchar_evento(&mut self, evento: EventoAplicacion) {
        match evento {
            EventoAplicacion::RegistroExitoso => self.actualizacion_registro_exitoso(),
            EventoAplicacion::ErrorDeRegistro(error_str) => {
                self.actualizacion_error_registro(error_str)
            }
            _ => (),
        };
    }
}

impl PantallaRegistro {
    fn mostrar_titulo(&self, ui: &mut Ui) {
        ui.label(RichText::new("Crear Cuenta").size(32.0).strong());
    }

    fn mostrar_input_usuario(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Usuario:").size(18.0));

        ui.add(
            TextEdit::singleline(&mut self.usuario)
                .min_size(vec2(300.0, 35.0))
                .margin(vec2(8.0, 8.0)),
        );
    }

    fn mostrar_input_password(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Contraseña:").size(18.0));

        ui.add(
            TextEdit::singleline(&mut self.contrasenia)
                .password(true)
                .min_size(vec2(300.0, 35.0))
                .margin(vec2(8.0, 8.0)),
        );
    }

    fn mostrar_botones(&mut self, ui: &mut Ui) -> AccionPantalla {
        let usuario = self.usuario.trim().to_string();
        let pass = self.contrasenia.trim().to_string();

        if ui
            .add(
                Button::new(RichText::new("Registrarme").size(20.0).strong())
                    .min_size(vec2(200.0, 40.0)),
            )
            .clicked()
        {
            // Reinicio por si hubo resultados anteriores
            self.registro_exitoso = false;
            self.error_registro = None;

            return AccionPantalla::IntentarRegistro(usuario, pass);
        }

        ui.add_space(10.0);

        if ui
            .add(Button::new(RichText::new("Volver").size(18.0)).min_size(vec2(200.0, 35.0)))
            .clicked()
        {
            return AccionPantalla::Volver;
        }

        AccionPantalla::Ninguna
    }

    fn mostrar_error(&mut self, ui: &mut Ui) {
        ui.colored_label(
            Color32::from_rgb(255, 80, 80),
            RichText::new("Fallo al registrarse. Intente nuevamente")
                .size(18.0)
                .strong(),
        );
    }

    fn mostrar_registro_exitoso(&mut self, ui: &mut Ui) {
        ui.colored_label(
            Color32::from_rgb(38, 184, 28),
            RichText::new("Registro exitoso. Ya puede iniciar sesion")
                .size(18.0)
                .strong(),
        );
    }

    fn actualizacion_registro_exitoso(&mut self) {
        self.registro_exitoso = true;
    }

    fn actualizacion_error_registro(&mut self, error_str: String) {
        self.error_registro = Some(error_str);
    }
}
