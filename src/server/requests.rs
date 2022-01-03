//! This deals with requests.
use super::main_loop::MainLoop;
use crate::database::root_db::system_config::SystemConfig;
use crate::LoadedDbs;

/// A request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Request {
    /// String is a config file as a TOML.
    CreateSystem(String, String, String),
    /// This is the path of the system to load.
    InitialiseFromPath(String),
    /// This represents the character name.
    CreateCharacterSheet(String),
    /// The string is a CompleteCharacter JSON/TOML.
    CreateUpdateCharacter(String),
    /// This needs no arguments and uses the current root.
    ListCharacters,
    /// The string a name and UUID.
    LoadCharacter(String, String),
    /// Represents an invalid request.
    Invalid(String),
}


impl Request {
    /// Converts an incoming JSON string into a bona-fide request.
    pub(crate) fn convert(input: String) -> Self {
        match toml::from_str(&input) {
            Ok(r) => r,
            Err(_) => match serde_json::from_str(&input) {
                Ok(r) => r,
                Err(_) => Self::Invalid(input),
            }
        }
    }

    /// Run the request.
    pub(crate) fn execute(self, main_loop: &mut MainLoop) -> Result<String, String> {
        match self {
            Self::CreateSystem(name, path, system) => {
                let sys = SystemConfig::from_config(&system)?;
                let dbs = sys.into_system(&path, &name)?;
                main_loop.dbs = Some(dbs);
                Ok(format!("Created \"{}\" in \"{}\"", name, path))
            }
            Self::InitialiseFromPath(path) => {
                let dbs = LoadedDbs::custom(&path)?;
                main_loop.dbs = Some(dbs);
                Ok(format!("Opened system from \"{}\"", path))
            }
            Self::CreateCharacterSheet(name) => {
                match main_loop.dbs {
                    Some(ref mut dbs) => {
                        dbs.create_sheet(&name)?;
                        Ok(String::new())
                    }
                    None => Err(String::from("Load system first.")),
                }

            }
            Self::CreateUpdateCharacter(sheet) => {
                match main_loop.dbs {
                    Some(ref mut dbs) => {
                        dbs.create_or_update_character(sheet)?;
                        Ok(String::new())
                    }
                    None => Err(String::from("Load system first.")),
                }
            }
            Self::ListCharacters => {
                match main_loop.dbs {
                    Some(ref mut dbs) => {
                        let chars = dbs.list_characters_json()?;
                        Ok(chars)
                    }
                    None => Err(String::from("Load system first.")),
                }
            }
            Self::LoadCharacter(name, uuid) => {
                match main_loop.dbs {
                    Some(ref mut dbs) => {
                        let char = dbs.load_character_as_json((name, uuid))?;
                        Ok(char)
                    }
                    None => Err(String::from("Load system first.")),
                }
            }
            Self::Invalid(x) => Err(format!("Invalid request received:({})", x)),
        }
    }
}
