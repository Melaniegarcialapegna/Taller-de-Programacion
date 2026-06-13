use eframe::egui::{
    self, CentralPanel, ColorImage, ImageSource, SidePanel, TextureHandle, TextureOptions,
    load::SizedTexture, vec2,
};

use crate::{
    aplicacion::EventoAplicacion,
    sesion_rtp::sesion::EstadisticasReceiver,
    vista::pantallas::{
        pantalla::{AccionPantalla, Pantalla},
        utils::iconos::IconoApp,
    },
};

pub struct PantallaLlamada {
    textura_imagen_externa: Option<TextureHandle>,
    textura_imagen_local: Option<TextureHandle>,
    estadisticas: Box<EstadisticasReceiver>,
    microfono_muteado: bool,
    mostrar_estadisticas: bool,
    oferta_archivo_pendiente: Option<(String, u64)>,
}

impl Default for PantallaLlamada {
    fn default() -> Self {
        PantallaLlamada {
            estadisticas: Box::new(EstadisticasReceiver::default()),
            textura_imagen_externa: None,
            textura_imagen_local: None,
            microfono_muteado: true,
            mostrar_estadisticas: false,
            oferta_archivo_pendiente: None,
        }
    }
}

impl Pantalla for PantallaLlamada {
    fn renderizar(&mut self, ctx: &eframe::egui::Context) -> AccionPantalla {
        if self.textura_imagen_externa.is_none() {
            self.textura_imagen_externa = Some(self.textura_imagen_por_defecto(ctx));
            self.textura_imagen_local = Some(self.textura_imagen_por_defecto(ctx));
        }

        let mut accion_pantalla = AccionPantalla::NuevoFrame;

        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("Room RTC - Sesión").size(32.0).strong());
                ui.label("Llamada en curso");
            });

            ui.vertical_centered(|ui| {
                self.mostrar_camara_local(ui);
                self.mostrar_botones_llamada(&mut accion_pantalla, ui);
                self.mostrar_camara_externa(ui);
            });

            SidePanel::right("Estadisticas")
                .min_width(300.0)
                .show(ctx, |ui| {
                    self.mostrar_estadisticas_rtcp(ui);
                });
        });

        if self.oferta_archivo_pendiente.is_some() {
            self.mostrar_popup_oferta_archivo(ctx, &mut accion_pantalla);
        }

        accion_pantalla
    }

    fn escuchar_evento(&mut self, evento: EventoAplicacion) {
        match evento {
            EventoAplicacion::NuevoFrame(frame) => {
                if let Some(imagen) = &mut self.textura_imagen_externa {
                    imagen.set(frame, TextureOptions::default());
                }
            }
            EventoAplicacion::NuevoFrameLocal(frame) => {
                if let Some(imagen) = &mut self.textura_imagen_local {
                    imagen.set(frame, TextureOptions::default());
                }
            }
            EventoAplicacion::MicrofonoMuteado => {
                self.microfono_muteado = true;
            }
            EventoAplicacion::MicrofonoDesmuteado => {
                self.microfono_muteado = false;
            }
            EventoAplicacion::NuevasEstadisticas(estadisticas) => self.estadisticas = estadisticas,
            EventoAplicacion::RecibendoOfertaArchivo { nombre, tamanio } => {
                self.oferta_archivo_pendiente = Some((nombre, tamanio));
            }
            _ => {}
        };
    }
}

impl PantallaLlamada {
    pub fn new() -> PantallaLlamada {
        PantallaLlamada {
            textura_imagen_externa: None,
            textura_imagen_local: None,
            microfono_muteado: false,
            estadisticas: Box::new(EstadisticasReceiver::default()),
            mostrar_estadisticas: false,
            oferta_archivo_pendiente: None,
        }
    }

    fn textura_imagen_por_defecto(&mut self, ctx: &eframe::egui::Context) -> TextureHandle {
        let imagen = ColorImage::example();

        ctx.load_texture("Frame remoto", imagen, TextureOptions::default())
    }

    fn mostrar_botones_llamada(&mut self, accion_pantalla: &mut AccionPantalla, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // si uso horizontal_centered me los centra mal, calculo a mano
            let ancho_botones = 64.0 * 3.0 + 5.0 * 2.0; // 3 botones + espacios
            let espacio = (ui.available_width() - ancho_botones).max(0.0) / 2.0;
            ui.add_space(espacio);

            if IconoApp::CortarLlamada.boton(ui, 64.0).clicked() {
                *accion_pantalla = AccionPantalla::CortarLlamada;
            }
            ui.add_space(5.0);

            if self.microfono_muteado {
                if IconoApp::MicrofonoInactivo.boton(ui, 64.0).clicked() {
                    *accion_pantalla = AccionPantalla::DesmutearMicrofono;
                }
            } else if IconoApp::MicrofonoActivo.boton(ui, 64.0).clicked() {
                *accion_pantalla = AccionPantalla::MutearMicrofono;
            }
            ui.add_space(5.0);

            if IconoApp::AdjuntarArchivo.boton(ui, 64.0).clicked() {
                *accion_pantalla = AccionPantalla::AbrirDialogoArchivo;
            }
        });
    }

    fn mostrar_camara_externa(&mut self, ui: &mut egui::Ui) {
        if let Some(textura) = &self.textura_imagen_externa {
            ui.add_space(20.0);
            let textura_con_tamanio = SizedTexture::new(textura.id(), vec2(300.0, 300.0));
            ui.image(ImageSource::Texture(textura_con_tamanio));
        };
    }

    fn mostrar_camara_local(&mut self, ui: &mut egui::Ui) {
        if let Some(textura) = &self.textura_imagen_local {
            ui.add_space(20.0);
            let textura_con_tamanio = SizedTexture::new(textura.id(), vec2(300.0, 300.0));
            ui.image(ImageSource::Texture(textura_con_tamanio));
            ui.add_space(20.0);
        }
    }

    fn mostrar_popup_oferta_archivo(
        &mut self,
        ctx: &eframe::egui::Context,
        accion_pantalla: &mut AccionPantalla,
    ) {
        let Some((nombre, tamanio)) = &self.oferta_archivo_pendiente else {
            return;
        };

        let tamanio_legible = if *tamanio >= 1_048_576 {
            format!("{:.1} MB", *tamanio as f64 / 1_048_576.0)
        } else if *tamanio >= 1_024 {
            format!("{:.1} KB", *tamanio as f64 / 1_024.0)
        } else {
            format!("{} bytes", tamanio)
        };

        let titulo = format!("📎 {}", nombre);
        let descripcion = format!(
            "El otro peer quiere enviarte un archivo\nTamaño: {}",
            tamanio_legible
        );

        let mut aceptar = false;
        let mut rechazar = false;

        egui::Window::new(&titulo)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(&descripcion);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    let ancho_boton = 100.0;
                    let espacio = (ui.available_width() - ancho_boton * 2.0 - 10.0).max(0.0) / 2.0;
                    ui.add_space(espacio);
                    if ui
                        .add_sized([ancho_boton, 32.0], egui::Button::new("Aceptar"))
                        .clicked()
                    {
                        aceptar = true;
                    }
                    ui.add_space(10.0);
                    if ui
                        .add_sized([ancho_boton, 32.0], egui::Button::new("Rechazar"))
                        .clicked()
                    {
                        rechazar = true;
                    }
                });
                ui.add_space(8.0);
            });

        if aceptar {
            self.oferta_archivo_pendiente = None;
            *accion_pantalla = AccionPantalla::AceptarArchivo;
        } else if rechazar {
            self.oferta_archivo_pendiente = None;
            *accion_pantalla = AccionPantalla::RechazarArchivo;
        }
    }

    fn mostrar_estadisticas_rtcp(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            if ui.button("Mostrar/Ocultar estadisticas").clicked() {
                self.mostrar_estadisticas = !self.mostrar_estadisticas;
            };

            if self.mostrar_estadisticas {
                ui.heading("Estadisticas de video");

                let cantidad_paquetes_recibidos = self.estadisticas.cantidad_paquetes_recibidos;
                ui.label(format!(
                    "Cantidad de paquetes recibidos: {}",
                    cantidad_paquetes_recibidos
                ));

                let cantidad_paquetes_perdidos =
                    self.estadisticas.contenido_report.cant_paquetes_perdidos & (!(0xFF << 24));
                ui.label(format!(
                    "Cantidad de paquetes perdidos: {}",
                    cantidad_paquetes_perdidos
                ));

                let perdida_de_paquetes_str = (cantidad_paquetes_perdidos as f32
                    / cantidad_paquetes_recibidos as f32)
                    * 100.0;
                ui.label(format!("Perdida de paquetes: {}%", perdida_de_paquetes_str));

                let tiempo_est_entre_paquetes_str = self
                    .estadisticas
                    .contenido_report
                    .tiempo_est_entre_paquetes
                    .to_string();
                ui.label(format!(
                    "Tiempo estimado entre paquetes: {}",
                    tiempo_est_entre_paquetes_str
                ));
            };
        });
    }
}
