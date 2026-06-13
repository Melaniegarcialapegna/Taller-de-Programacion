use eframe::egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke,
    TextStyle, Visuals, style::WidgetVisuals, vec2,
};
use std::sync::Arc;

pub fn configurar_theme(ctx: &Context) {
    configurar_fuentes_personalizadas(ctx);
    aplicar_colores_roomrtc(ctx);
    configurar_estilos_base(ctx);
}

fn configurar_fuentes_personalizadas(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "CabalFont".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "assets/PoppinsLight-l4Zw.otf"
        ))),
    );

    for family in [&FontFamily::Proportional, &FontFamily::Monospace] {
        match fonts.families.get_mut(family) {
            Some(f) => f.insert(0, "CabalFont".to_owned()),
            None => {
                fonts
                    .families
                    .insert((*family).clone(), vec!["CabalFont".to_owned()]);
            }
        }
    }

    ctx.set_fonts(fonts);
}

fn configurar_estilos_base(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.button_padding = vec2(20.0, 12.0);
    style.spacing.interact_size = vec2(100.0, 40.0);

    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(28.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(18.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(16.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(16.0, FontFamily::Monospace),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
    ]
    .into();

    ctx.set_style(style);
}

fn aplicar_colores_roomrtc(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(15, 16, 18);
    visuals.panel_fill = Color32::from_rgb(21, 23, 27);
    visuals.override_text_color = Some(Color32::WHITE);

    let corner_radius_global = CornerRadius::same(16);

    visuals.widgets.inactive = widget_inactivo(corner_radius_global);
    visuals.widgets.hovered = widget_hover(corner_radius_global);
    visuals.widgets.active = widget_activo(corner_radius_global);
    visuals.window_corner_radius = CornerRadius::same(10);

    ctx.set_visuals(visuals);
}

fn widget_inactivo(radio: CornerRadius) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: Color32::from_rgb(45, 45, 55),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(55, 55, 65)),
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        corner_radius: radio,
        ..Visuals::dark().widgets.inactive
    }
}

fn widget_hover(radio: CornerRadius) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: Color32::from_rgb(216, 30, 91),
        weak_bg_fill: Color32::from_rgb(216, 30, 91),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(216, 30, 91)),
        fg_stroke: Stroke::new(1.2, Color32::WHITE),
        corner_radius: radio,
        ..Visuals::dark().widgets.hovered
    }
}

fn widget_activo(radio: CornerRadius) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: Color32::from_rgb(180, 24, 76),
        weak_bg_fill: Color32::from_rgb(180, 24, 76),
        bg_stroke: Stroke::new(1.0, Color32::from_rgb(180, 24, 76)),
        fg_stroke: Stroke::new(1.2, Color32::WHITE),
        corner_radius: radio,
        ..Visuals::dark().widgets.active
    }
}
