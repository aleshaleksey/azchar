use crate::AZCharFourth;

use azchar_database::character::image::Image;
use azchar_database::character::character::{InputCharacter, CharacterPart, CompleteCharacter};
use azchar_database::character::attribute::AttributeKey;
use azchar_database::LoadedDbs;
use azchar_error::ma;

use eframe;
use eframe::egui::Widget;
use std::fs::File;

fn get_file() -> Result<Option<File>, String> {
    if let Some(p) = rfd::FileDialog::new()
        .add_filter("Export", &["json", "toml"])
        .pick_file() {
            if !p.exists() {
                return Err(format!("Ara-Ara! The File ({:?}) doens't actually exist.", p));
            }
            File::open(p).map(Some).map_err(ma)
        } else {
            Ok(None)
        }
}

pub(super) fn import_part(
    dbs: &mut LoadedDbs,
    char: &mut CompleteCharacter,
    mut file: File,
) -> Result<(), String> {
    let initial_part: CharacterPart = serde_json::from_reader(file).map_err(ma)?;
    let new_part:InputCharacter = (&initial_part).into();
    let keys = (char.name().to_owned(), char.uuid().to_owned());
    let updated_character = dbs.create_part(new_part, keys.to_owned())?;

    // Now we need to find the new part, because someone made a silly in'erface.
    let ids: Vec<Option<i64>> = char.parts.iter().map(|p| p.id()).collect();
    let new_part_id = updated_character
        .parts
        .iter()
        .find(|p| !ids.contains(&p.id()))
        .expect("This is a paradox. The new part is always there.")
        .id()
        .expect("It's been through the database so an id exists.");

    for (k, v) in initial_part.attributes {
        let k = AttributeKey::new(k.key().to_owned(), new_part_id);
        // TODO: Make some kind of reporter for failed attribute import.
        if let Err(_) = dbs.create_update_attribute(k, v, keys.to_owned()) {}
    }
    *char = updated_character;
    Ok(())
}
