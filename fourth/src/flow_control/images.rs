use crate::AZCharFourth;

use azchar_database::character::image::Image;
use azchar_database::LoadedDbs;

use eframe;
use eframe::egui::Widget;
use fnv::FnvHashMap;

pub(crate) fn set_image(
    default_image: &egui_extras::RetainedImage,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    dbs: &mut LoadedDbs,
    char_image: &mut Option<Image>,
    (char_name, char_uuid): (String, String),
    part_id: i64,
    images: &mut FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
) -> Result<(), String> {
    let portrait = images.get(&Some(part_id)).unwrap_or(default_image);
    let ib = egui::ImageButton::new(portrait.texture_id(ctx), [136., 136.]);
    if ib.ui(ui).clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("image", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
        {
            println!("Picked: {:?}", path);
            let res = AZCharFourth::set_image(
                dbs, char_image, images, char_name, char_uuid, part_id, path,
            );
            if let Err(e) = res {
                return Err(format!("Couldn't set image: {:?}", e));
            }
        } else {
            return Err("Failed to pick a file.".to_string());
        }
    }
    super::separator(ui);
    Ok(())
}
