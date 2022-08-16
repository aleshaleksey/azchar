use super::AZCharFourth;
use azchar_database::character::image as dbimg;
use azchar_database::LoadedDbs;
use azchar_error::ma;

use fnv::FnvHashMap;
use image;
use std::io::Cursor;
// pub(crate) struct AZCharFourth {
//     db_path: String,
//     cfg_path: String,
//     dbs: Option<LoadedDbs>,
//     char_list: Vec<CharacterDbRef>,
//     current: Option<CompleteCharacter>,
// }

impl AZCharFourth {
    pub(super) fn load_system(&mut self) -> Result<(), String> {
        let mut dbs = LoadedDbs::custom(&self.db_path)?;
        self.char_list = dbs.list_characters()?;
        self.dbs = Some(dbs);
        Ok(())
    }

    pub(super) fn load_character(&mut self, name: &str, uuid: &str) -> Result<(), String> {
        if let Some(ref mut dbs) = self.dbs {
            let loaded = dbs.load_character((name.to_owned(), uuid.to_owned()))?;
            let mut imagemap = FnvHashMap::default();
            // Insert primary image.
            if let Some(ref data) = loaded.image().as_ref() {
                let processed = process_image(data)?;
                imagemap.insert(loaded.id(), processed);
            }
            // Insert part images.
            for c in loaded.parts().iter() {
                if let Some(ref data) = c.image.as_ref() {
                    let processed = process_image(data)?;
                    imagemap.insert(loaded.id(), processed);
                }
            }
            self.images = imagemap;
            self.current = Some(loaded);
        }
        Ok(())
    }

    // Reset an image.
    pub(super) fn set_image(
        dbs: &mut Option<LoadedDbs>,
        image: &mut Option<dbimg::Image>,
        imagemap: &mut FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
        name: String,
        uuid: String,
        id: i64,
        path: std::path::PathBuf
    ) -> Result<(), String> {
        if let Some(ref mut dbs) = dbs {
            let input = dbimg::InputImage {
                of: id,
                link: path.into_os_string().into_string().map_err(ma)?,
            };
            let output = dbs.create_update_image(name, uuid, input)?;
            let processed = process_image(&output)?;
            *image = Some(output);
            imagemap.insert(Some(id), processed);
        }
        Ok(())
    }
}

fn process_image(image: &dbimg::Image) -> Result<egui_extras::RetainedImage, String> {
    let ret = egui_extras::RetainedImage::from_image_bytes(image.of.to_string(), &image.content)
        .map_err(ma)?;
    Ok(ret)
}
