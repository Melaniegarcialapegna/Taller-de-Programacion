//! Módulo que define los íconos utilizados en la aplicación RoomRTC.
//! Cada ícono se representa como una variante del enum `IconoApp`, y se carga desde archivos SVG ubicados en la carpeta "assets/".
use eframe::egui::{Color32, Image, ImageSource, Response, Sense, Ui, Vec2, include_image};

pub enum IconoApp {
    Llamar,
    OpcionLlamarInicio,
    CortarLlamada,
    PrevisualizarVideoLocal,
    Logout,
    Usuario,
    MicrofonoActivo,
    MicrofonoInactivo,
    AdjuntarArchivo,
}

// Todos los iconos se cargan desde archivos SVG ubicados en la carpeta "assets/"

/// Cada variante del enum corresponde a un ícono específico.
impl IconoApp {
    /// Devuelve la fuente de imagen correspondiente al ícono.
    fn image(&self) -> ImageSource<'static> {
        match self {
            IconoApp::Llamar => include_image!("../../assets/call.svg"),
            IconoApp::OpcionLlamarInicio => include_image!("../../assets/call.svg"),
            IconoApp::CortarLlamada => include_image!("../../assets/call_end.svg"),
            IconoApp::PrevisualizarVideoLocal => {
                include_image!("../../assets/video_camera_front.svg")
            }
            IconoApp::Logout => include_image!("../../assets/logout.svg"),
            IconoApp::Usuario => include_image!("../../assets/usuario.png"),
            IconoApp::MicrofonoActivo => include_image!("../../assets/mic.svg"),
            IconoApp::MicrofonoInactivo => include_image!("../../assets/mic_off.svg"),
            IconoApp::AdjuntarArchivo => include_image!("../../assets/adjuntar.svg"),
        }
    }

    /// Dibuja el ícono como un botón con estilo compacto y color de fondo personalizado.
    ///
    /// # Arguments
    /// * `ui` - Referencia mutable a la interfaz de usuario donde se dibujará el ícono.
    /// * `size` - Tamaño del botón en píxeles.
    ///
    /// # Returns
    /// * `Response` - Respuesta del botón dibujado.
    pub fn boton(&self, ui: &mut Ui, size: f32) -> Response {
        let old_style = (**ui.style()).clone(); // Tipo: Style
        let mut compact_style = old_style.clone(); // Tipo: Style (mutable)

        compact_style.spacing.button_padding = Vec2::splat(2.0);
        compact_style.spacing.interact_size = Vec2::splat(size);

        //aplicar temporalmente el estilo compacto
        ui.set_style(compact_style);

        //dibujar el ícono
        let response = match self {
            IconoApp::CortarLlamada => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(220, 50, 50))
            }
            IconoApp::OpcionLlamarInicio => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(52, 53, 65))
            }
            IconoApp::Llamar => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(0, 200, 0))
            }
            IconoApp::PrevisualizarVideoLocal => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(255, 102, 0))
            }
            IconoApp::Logout => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(45, 45, 55))
            }
            IconoApp::Usuario => self.dibujar_simple(ui, size),
            IconoApp::MicrofonoActivo => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(0, 200, 0))
            }
            IconoApp::MicrofonoInactivo => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(220, 50, 50))
            }
            IconoApp::AdjuntarArchivo => {
                self.dibujar_icono_color_redondeado(ui, size, Color32::from_rgb(255, 187, 51))
            }
        };

        //restaurar el estilo original
        ui.set_style(old_style);

        response
    }

    /// Dibuja el ícono con un fondo de color redondeado.
    ///
    /// # Arguments
    /// * `ui` - Referencia mutable a la interfaz de usuario donde se dibujará el ícono.
    /// * `size` - Tamaño del área del ícono en píxeles.
    /// * `color` - Color de fondo del ícono.
    ///
    /// # Returns
    /// * `Response` - Respuesta del área dibujada.
    fn dibujar_icono_color_redondeado(&self, ui: &mut Ui, size: f32, color: Color32) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, 8.0, color);

        let image = Image::new(self.image()).fit_to_exact_size(Vec2::splat(size * 0.7));
        image.paint_at(ui, rect.shrink(size * 0.15));

        response
    }

    fn dibujar_simple(&self, ui: &mut Ui, size: f32) -> Response {
        let image = Image::new(self.image()).fit_to_exact_size(Vec2::splat(size));
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        image.paint_at(ui, rect);
        response
    }
}
