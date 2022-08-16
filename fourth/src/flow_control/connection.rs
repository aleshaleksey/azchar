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
            self.current = Some(loaded);
        }
        Ok(())
    }
}

fn process_image(image: &dbimg::Image) -> Result<egui_extras::RetainedImage, String> {
    // let decoded = image::io::Reader::new(Cursor::new(image.content))
    //     .with_guessed_format()
    //     .map_err(ma)?
    //     .decode()
    //     .map_err(ma)?
    //     .to_rgba8();
    let ret = egui_extras::RetainedImage::from_image_bytes(image.of.to_string(), &image.content)
        .map_err(ma)?;
    Ok(ret)
}
