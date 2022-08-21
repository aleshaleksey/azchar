use egui::containers::Frame;
use egui::style::*;
use egui::Color32;

pub(crate) fn style() -> Style {
    let mut visuals = Visuals::dark();
    visuals.faint_bg_color = Color32::from_rgb(99, 88, 77);
    visuals.extreme_bg_color = egui::Color32::from_rgb(55, 44, 33);
    let interaction = Interaction {
        resize_grab_radius_side: 15.,
        resize_grab_radius_corner: 20.,
        show_tooltips_only_when_still: false,
    };

    let mut style = Style::default();
    style.override_text_style = Some(TextStyle::Body);
    style.wrap = Some(true);
    style.interaction = interaction;
    style.visuals = visuals;
    style
}

pub(crate) fn default_frame() -> Frame {
    let margin = Margin {
        left: 5.,
        right: 5.,
        top: 5.,
        bottom: 5.,
    };

    Frame::none()
        .fill(egui::Color32::from_rgb(110, 99, 88))
        .stroke(egui::Stroke::new(3., egui::Color32::from_rgb(33, 22, 11)))
        .inner_margin(margin)
        .outer_margin(margin)
}
