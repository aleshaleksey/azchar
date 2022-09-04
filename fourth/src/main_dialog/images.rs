use eframe::egui::Widget;
use fnv::FnvHashMap;

use azchar_database::character::image as dbimg;
use azchar_error::ma;

#[allow(clippy::too_many_arguments)]
pub(crate) fn set_image(
    default_image: &egui_extras::RetainedImage,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    part_id: i64,
    images: &mut FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
    flow_state: &mut crate::flow_control::For,
) -> Result<(), String> {
    let portrait = images.get(&Some(part_id)).unwrap_or(default_image);
    let ib = egui::ImageButton::new(portrait.texture_id(ctx), [136., 136.]);
    if ib.ui(ui).clicked() {
        *flow_state = crate::flow_control::For::ImportImage(part_id);
    }
    super::separator(ui);
    Ok(())
}

pub(crate) fn process_image(image: &dbimg::Image) -> Result<egui_extras::RetainedImage, String> {
    let ret = egui_extras::RetainedImage::from_image_bytes(image.of.to_string(), &image.content)
        .map_err(ma)?;
    Ok(ret)
}
