use azchar_database::character::attribute::AttributeKey;
use azchar_database::character::character::{CharacterPart, CompleteCharacter, InputCharacter};
use azchar_database::{CharacterDbRef, LoadedDbs};
use azchar_error::ma;

use std::fs::File;
use std::path::PathBuf;

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

pub(crate) fn part(
    dbs: &mut LoadedDbs,
    char: &mut CompleteCharacter,
    path: PathBuf,
) -> Result<(), String> {
    if let Ok(f) = std::fs::File::open(path) {
        import_part_inner(dbs, char, f)?
    }
    Ok(())
}

pub(crate) fn character(dbs: &mut LoadedDbs, path: PathBuf) -> Result<(), String> {
    if let Ok(f) = std::fs::File::open(path) {
        import_character_inner(dbs, f)?;
    }
    Ok(())
}