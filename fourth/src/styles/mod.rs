use egui::style::*;
use egui::Color32;
use egui::Rounding;

use std::collections::BTreeMap;

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
