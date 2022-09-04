use azchar_database::character::character::CharacterPart;
use azchar_database::LoadedDbs;
use azchar_error::ma;

use std::path::PathBuf;

fn get_dir() -> Result<Option<PathBuf>, String> {
    if let Some(p) = rfd::FileDialog::new().pick_folder() {
        if !p.exists() {
            return Err(format!(
                "Ara-Ara! The Folder ({:?}) doesn't actually exist.",
                p
            ));
        }
        Ok(Some(p))
    } else {
        Ok(None)
    }
}

pub(crate) fn character(
    dbs: &mut LoadedDbs,
    c_name: &str,
    c_uuid: &str,
    dir: PathBuf,
) -> Result<(), String> {
    if let Ok(char) = dbs.load_character((c_name.to_string(), c_uuid.to_string())) {
        let name = format!("{}-{}.json", char.name(), char.uuid());
        let path = dir.join(name);
        println!("Exporting to: {:?}", path);
        let file = std::fs::File::create(path).map_err(ma)?;
        serde_json::to_writer_pretty(file, &char).map_err(ma)?;
    }
    Ok(())
}

pub(crate) fn part(part: &CharacterPart, dir: PathBuf) -> Result<(), String> {
    let name = format!(
        "{}-({})-{}.json",
        part.name(),
        part.character_type(),
        part.uuid()
    );

    let file = std::fs::File::create(dir.join(name)).map_err(ma)?;
    serde_json::to_writer_pretty(file, &part).map_err(ma)
}
