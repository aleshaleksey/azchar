use super::table::Row;
use super::*;

use azchar_database::character::attribute::{AttributeKey, AttributeValue};
use azchar_database::character::character::CharacterPart;
use azchar_database::character::image as dbimg;
use azchar_database::LoadedDbs;
use azchar_error::ma;

use fnv::FnvHashMap;
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
            self.main_attr_table = [
                Row::new("Name", loaded.name()),
                Row::new("Speed", &loaded.speed.to_string()),
                Row::new(
                    "Weight",
                    &loaded.weight.map(|x| x.to_string()).unwrap_or_default(),
                ),
                Row::new("Size", &loaded.size.to_owned().unwrap_or_default()),
                Row::new(
                    "HP",
                    &loaded.hp_current.map(|x| x.to_string()).unwrap_or_default(),
                ),
                Row::new(
                    "HP total",
                    &loaded.hp_total.map(|x| x.to_string()).unwrap_or_default(),
                ),
            ];
            let level = Self::get_attr_val_num(&loaded.attributes(), LEVEL);
            let proficiency = Self::get_attr_val_num(&loaded.attributes(), PROFICIENCY);
            self.main_level_pro_table = [
                Row::with_label("Level", &level.to_string(), LEVEL),
                Row::with_label("Proficiency", &proficiency.to_string(), PROFICIENCY),
            ];
            let str = Self::get_attr_val_num(&loaded.attributes(), STRENGTH);
            let re = Self::get_attr_val_num(&loaded.attributes(), REFLEX);
            let tou = Self::get_attr_val_num(&loaded.attributes(), TOUGHNESS);
            let end = Self::get_attr_val_num(&loaded.attributes(), ENDURANCE);
            let int = Self::get_attr_val_num(&loaded.attributes(), INTELLIGENCE);
            let jud = Self::get_attr_val_num(&loaded.attributes(), JUDGEMENT);
            let cha = Self::get_attr_val_num(&loaded.attributes(), CHARM);
            let wil = Self::get_attr_val_num(&loaded.attributes(), WILL);
            self.main_stat_table = [
                Row::with_label("STR", &str.to_string(), STRENGTH),
                Row::with_label("REF", &re.to_string(), REFLEX),
                Row::with_label("TOU", &tou.to_string(), TOUGHNESS),
                Row::with_label("END", &end.to_string(), ENDURANCE),
                Row::with_label("INT", &int.to_string(), INTELLIGENCE),
                Row::with_label("JUD", &jud.to_string(), JUDGEMENT),
                Row::with_label("CHA", &cha.to_string(), CHARM),
                Row::with_label("WIL", &wil.to_string(), WILL),
            ];
            self.images = imagemap;
            self.current = Some(loaded);
        }
        Ok(())
    }
    /// This function exists for dry.
    fn get_attr_val_num(attrs: &[(AttributeKey, AttributeValue)], needle: &str) -> i64 {
        attrs
            .iter()
            .find(|(k, _)| k.key() == needle)
            .expect("Is there.")
            .1
            .value_num()
            .unwrap_or_default()
    }
    // Reset an image.
    pub(super) fn set_image(
        dbs: &mut Option<LoadedDbs>,
        image: &mut Option<dbimg::Image>,
        imagemap: &mut FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
        name: String,
        uuid: String,
        id: i64,
        path: std::path::PathBuf,
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

    pub(super) fn update_main(
        dbs: &mut Option<LoadedDbs>,
        part: CharacterPart,
    ) -> Result<(), String> {
        if let Some(ref mut dbs) = dbs {
            let name = part.name().to_owned();
            let uuid = part.uuid().to_owned();
            dbs.create_update_part(part, (name, uuid))?;
        }
        Ok(())
    }

    // Update attributes.
    pub(super) fn update_attrs(
        dbs: &mut Option<LoadedDbs>,
        part: &mut CompleteCharacter,
        rows: &[Row],
    ) -> Result<(), String> {
        if let Some(ref mut dbs) = dbs {
            for r in rows.iter() {
                if let Some((ref mut k, ref mut v)) =
                    part.attributes_mut().iter_mut().find(|(k, _)| k.key() == &r.label)
                {
                    match r.value.parse() {
                        Ok(v1) if Some(v1) != v.value_num() => {
                            v.update_value_num_by_ref(Some(v1));
                            dbs.create_update_attribute(
                                k.to_owned(),
                                v.to_owned(),
                                (part.name.to_owned(), part.uuid().to_owned()),
                            )?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

fn process_image(image: &dbimg::Image) -> Result<egui_extras::RetainedImage, String> {
    let ret = egui_extras::RetainedImage::from_image_bytes(image.of.to_string(), &image.content)
        .map_err(ma)?;
    Ok(ret)
}
