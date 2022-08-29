use azchar_database::character::attribute::AttributeKey;
use azchar_database::character::character::{CharacterPart, CompleteCharacter, InputCharacter};
use azchar_database::{CharacterDbRef, LoadedDbs};
use azchar_error::ma;

use std::fs::File;

fn get_file() -> Result<Option<File>, String> {
    if let Some(p) = rfd::FileDialog::new()
        .add_filter("Export", &["json", "toml"])
        .pick_file()
    {
        if !p.exists() {
            return Err(format!(
                "Ara-Ara! The File ({:?}) doens't actually exist.",
                p
            ));
        }
        File::open(p).map(Some).map_err(ma)
    } else {
        Ok(None)
    }
}

fn import_part_inner(
    dbs: &mut LoadedDbs,
    char: &mut CompleteCharacter,
    file: File,
) -> Result<(), String> {
    let initial_part: CharacterPart = serde_json::from_reader(file).map_err(ma)?;
    let new_part: InputCharacter = (&initial_part).into();
    let keys = (char.name().to_owned(), char.uuid().to_owned());
    let mut updated_character = dbs.create_part(new_part, keys.to_owned())?;

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
        if dbs.create_update_attribute(k, v, keys.to_owned()).is_err() {}
    }
    updated_character.create_attribute_map();
    *char = updated_character;
    Ok(())
}

fn import_character_inner(dbs: &mut LoadedDbs, file: File) -> Result<Vec<CharacterDbRef>, String> {
    let character: CompleteCharacter = serde_json::from_reader(file).map_err(ma)?;
    dbs.create_or_update_character(character)?;
    dbs.refresh_and_list()
}

pub(super) fn import_part(dbs: &mut LoadedDbs, char: &mut CompleteCharacter) -> Result<(), String> {
    if let Some(f) = get_file()? {
        import_part_inner(dbs, char, f)?
    }
    Ok(())
}

pub(super) fn import_character(dbs: &mut LoadedDbs) -> Result<(), String> {
    if let Some(f) = get_file()? {
        import_character_inner(dbs, f)?;
    }
    Ok(())
}
