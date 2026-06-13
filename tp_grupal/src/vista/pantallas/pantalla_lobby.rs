use eframe::egui::{
    self, Color32, Context, FontId, Frame, RichText, ScrollArea, Separator, SidePanel,
    TopBottomPanel, Ui,
};

use crate::{
    aplicacion::EventoAplicacion,
    protocolos::pca::{estado::EstadoUsuarioPCA, usuario::UsuarioPCA},
    vista::pantallas::{
        pantalla::{AccionPantalla, Pantalla},
        utils::iconos::IconoApp,
    },
};
use eframe::egui::CentralPanel;

#[derive(Default)]
pub struct PantallaLobby {
    usuarios: Vec<UsuarioPCA>,
    usuario_siendo_llamado: Option<String>,
    usuario_llamandome: Option<String>,
    lista_camaras: Vec<String>,
    se_pidieron_usuarios: bool,
    se_pidio_lista_de_camaras: bool,
    se_eligio_camara: bool,
    camara_actual: String,
}

impl Pantalla for PantallaLobby {
    fn renderizar(&mut self, ctx: &Context) -> AccionPantalla {
        let mut accion_pantalla = AccionPantalla::Ninguna;

        self.mostrar_boton_cerrar_sesion(ctx, &mut accion_pantalla);

        SidePanel::right("panel_camaras")
            .resizable(false)
            .min_width(410.0)
            .show(ctx, |ui| {
                let nombre_camara = &self.camara_actual;
                ui.label(RichText::new(format!("Camara actual: {nombre_camara}")).size(30.0));
                ui.add(Separator::default().horizontal());
                self.mostrar_lista_camaras(ui, &mut accion_pantalla);
            });

        CentralPanel::default()
            .frame(eframe::egui::Frame::NONE)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.mostrar_lista_usuarios(ui, &mut accion_pantalla);

                    if let Some(usuario) = &self.usuario_siendo_llamado {
                        self.mostrar_cartel_llamando_a_usuario(ctx, usuario);
                    };

                    if let Some(usuario) = &self.usuario_llamandome {
                        self.mostrar_cartel_recibiendo_llamada(usuario, &mut accion_pantalla, ctx);
                    }
                })
            });

        if !self.se_pidio_lista_de_camaras {
            accion_pantalla = AccionPantalla::PedirListaDeCamaras;
            self.se_pidio_lista_de_camaras = true;
        } else if !self.se_eligio_camara {
            // Esto es por si no hay camaras.
            // No se por que usarias una aplicacion de videollamadas si no tenes camara, pero bueno
            if !self.lista_camaras.is_empty() {
                accion_pantalla = AccionPantalla::CambiarCamara(self.lista_camaras[0].clone());
                self.se_eligio_camara = true;
            };
        } else if !self.se_pidieron_usuarios {
            accion_pantalla = AccionPantalla::PedirUsuarios;
            self.se_pidieron_usuarios = true;
        }

        accion_pantalla
    }

    fn escuchar_evento(&mut self, evento: EventoAplicacion) {
        match evento {
            EventoAplicacion::UsuariosNuevos(usuarios) => self.actualizar_usuarios(usuarios),
            EventoAplicacion::EnviandoLlamada(usuario) => self.actualizar_enviando_llamada(usuario),
            EventoAplicacion::RecibiendoLlamada(usuario) => {
                self.actualizacion_recibiendo_llamada(usuario)
            }
            EventoAplicacion::LlamadaExternaRechazada => {
                self.actualizacion_llamada_externa_rechazada()
            }
            EventoAplicacion::LlamadaRechazada => self.actualizacion_llamada_rechazada(),
            EventoAplicacion::NuevaListaDeCamarasDisponibles(lista_camaras) => {
                self.actualizacion_nueva_lista_camaras(lista_camaras)
            }
            EventoAplicacion::NuevaCamaraEnUso(nombre_camara) => self.camara_actual = nombre_camara,
            _ => {}
        }
    }
}

impl PantallaLobby {
    fn actualizar_usuarios(&mut self, usuarios: Vec<UsuarioPCA>) {
        self.usuarios = usuarios;
    }

    fn actualizar_enviando_llamada(&mut self, usuario: String) {
        self.usuario_siendo_llamado = Some(usuario);
    }

    fn actualizacion_recibiendo_llamada(&mut self, usuario: String) {
        self.usuario_llamandome = Some(usuario);
    }

    fn actualizacion_llamada_externa_rechazada(&mut self) {
        self.usuario_llamandome = None;
    }

    fn actualizacion_llamada_rechazada(&mut self) {
        self.usuario_siendo_llamado = None;
    }

    fn actualizacion_nueva_lista_camaras(&mut self, lista_camaras: Vec<String>) {
        self.lista_camaras = lista_camaras;
    }

    fn mostrar_lista_usuarios(&mut self, ui: &mut Ui, accion_pantalla: &mut AccionPantalla) {
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.label(RichText::new("Usuarios").font(FontId::proportional(36.0)));
            ui.add_space(12.0);
        });

        let ancho = ui.available_width() - 340.0; // le resto el ancho del side panel de camaras
        let columnas = if ancho > 900.0 { 3 } else { 2 };

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(0.0);
                Frame::NONE
                    .inner_margin(egui::Margin {
                        left: 51,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        egui::Grid::new("grid_usuarios")
                            .num_columns(columnas)
                            .min_col_width(260.0)
                            .max_col_width(260.0)
                            .spacing([16.0, 16.0])
                            .show(ui, |ui| {
                                for (i, usuario) in self.usuarios.iter().enumerate() {
                                    if self.mostrar_usuario(ui, usuario) {
                                        *accion_pantalla = AccionPantalla::Llamar(usuario.nombre());
                                    }
                                    if (i + 1) % columnas == 0 {
                                        ui.end_row();
                                    }
                                }
                                if !self.usuarios.is_empty()
                                    && !self.usuarios.len().is_multiple_of(columnas)
                                {
                                    ui.end_row();
                                }
                            });
                    });
            });
    }

    fn mostrar_boton_cerrar_sesion(&mut self, ctx: &Context, accion_pantalla: &mut AccionPantalla) {
        TopBottomPanel::top("barra_superior").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Cerrar sesion").clicked() {
                    *accion_pantalla = AccionPantalla::CerrarSesion;
                }
            });
            ui.add_space(8.0);
        });
    }

    fn mostrar_lista_camaras(&mut self, ui: &mut Ui, accion_pantalla: &mut AccionPantalla) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Camaras disponibles").font(FontId::proportional(30.0)));

            for camara in &self.lista_camaras {
                ui.group(|ui| {
                    ui.label(camara);

                    let boton_seleccionar = ui.button("Seleccionar");
                    if boton_seleccionar.clicked() {
                        *accion_pantalla = AccionPantalla::CambiarCamara(camara.clone())
                    }
                });
            }
        });
    }

    fn mostrar_usuario(&self, ui: &mut Ui, usuario: &UsuarioPCA) -> bool {
        let estado = usuario.estado();

        let (estado, estado_color) = match estado {
            EstadoUsuarioPCA::Disponible => ("● Disponible", Color32::LIGHT_GREEN),
            EstadoUsuarioPCA::Ocupado => ("● Ocupado", Color32::LIGHT_RED),
            EstadoUsuarioPCA::Desconectado => ("● Desconectado", Color32::GRAY),
        };

        let mut clicked_llamar = false;

        let tamano = egui::vec2(260.0, 140.0);

        ui.allocate_ui(tamano, |ui| {
            Frame::group(ui.style())
                .corner_radius(10.0)
                .inner_margin(egui::Margin::same(12))
                .stroke(egui::Stroke::new(2.0, Color32::from_rgb(216, 30, 91)))
                .show(ui, |ui| {
                    ui.set_min_size(tamano - egui::vec2(4.0, 4.0));

                    ui.label(RichText::new(usuario.nombre()).size(18.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(estado).color(estado_color));
                    ui.add_space(10.0);

                    let puede_llamar = estado == "● Disponible";
                    ui.add_enabled_ui(puede_llamar, |ui| {
                        if ui
                            .add_sized([110.0, 32.0], egui::Button::new("Llamar"))
                            .clicked()
                        {
                            clicked_llamar = true;
                        }
                    });
                });
        });

        clicked_llamar
    }

    fn mostrar_cartel_recibiendo_llamada(
        &self,
        usuario: &String,
        accion_pantalla: &mut AccionPantalla,
        ctx: &Context,
    ) {
        egui::Window::new("Llamada entrante")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    IconoApp::Usuario.boton(ui, 100.0);
                    ui.add_space(10.0);

                    ui.heading(
                        egui::RichText::new(format!("{} te está llamando", usuario))
                            .size(24.0)
                            .strong(),
                    );

                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(115.0);
                            if IconoApp::Llamar.boton(ui, 50.0).clicked() {
                                *accion_pantalla = AccionPantalla::AtenderLlamada;
                            }

                            ui.add_space(12.0);

                            if IconoApp::CortarLlamada.boton(ui, 50.0).clicked() {
                                *accion_pantalla = AccionPantalla::RechazarLlamada;
                            }
                        });
                    });
                    ui.add_space(15.0);
                });
            });
    }

    fn mostrar_cartel_llamando_a_usuario(&self, ctx: &Context, usuario: &String) {
        egui::Window::new("Llamando a usuario")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    IconoApp::Usuario.boton(ui, 100.0);
                    ui.add_space(10.0);

                    ui.heading(
                        egui::RichText::new(format!("Intentando llamar a {usuario}..."))
                            .size(24.0)
                            .strong(),
                    );
                    ui.add_space(15.0);
                });
            });
    }
}
