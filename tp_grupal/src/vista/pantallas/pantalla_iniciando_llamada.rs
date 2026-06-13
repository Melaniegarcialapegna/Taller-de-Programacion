use eframe::egui::CentralPanel;

use crate::{
    aplicacion::EventoAplicacion,
    vista::pantallas::pantalla::{AccionPantalla, Pantalla},
};

#[derive(Default)]
pub struct PantallaIniciandoLlamada {
    // Aca deberia guardarse el ultimo frame recibido para mostrarse
}

impl Pantalla for PantallaIniciandoLlamada {
    fn renderizar(&mut self, ctx: &eframe::egui::Context) -> AccionPantalla {
        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.spinner();
            })
        });

        AccionPantalla::Ninguna
    }

    fn escuchar_evento(&mut self, _evento: EventoAplicacion) {}
}
