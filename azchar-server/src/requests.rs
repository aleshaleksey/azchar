//! This deals with requests.
use azchar_database::character::character::CompleteCharacter;
use azchar_database::root_db::system_config::SystemConfig;
use azchar_database::CharacterDbRef;
use azchar_error::ma;

use azchar_database::LoadedDbs;

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
    /// Represents a request to parse and run a roll.
    Roll(String),
    /// Represents an invalid request.
    Invalid(String),
}

/// A request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Response {
    /// Returns a useless message.
    CreateSystem(String),
    /// Returns a list of characters, because what else?
    InitialiseFromPath(Vec<CharacterDbRef>),
    /// Returns an updated list of characters
    CreateCharacterSheet(Vec<CharacterDbRef>),
    /// Returns updated list of characters.
    CreateUpdateCharacter(Vec<CharacterDbRef>),
    /// Returns a list of characters.
    ListCharacters(Vec<CharacterDbRef>),
    /// The Complete Character.
    LoadCharacter(CompleteCharacter),
    /// The roll for each dice group and the total.
    Roll(Vec<i64>, i64),
    /// Represents an invalid request.
    Invalid(String),
    /// Represents an error.
    Err(String),
}

impl Request {
    /// Converts an incoming JSON string into a bona-fide request.
    pub(crate) fn convert(input: String) -> Self {
        match toml::from_str(&input) {
            Ok(r) => r,
            Err(_) => match serde_json::from_str(&input) {
                Ok(r) => r,
                Err(_) => Self::Invalid(input),
            },
        }
    }

    /// Run the request and give a response.
    /// NB: An error case should be unwrapped
    pub(crate) fn execute(self, main_loop: &mut Option<LoadedDbs>) -> Result<Response, String> {
        let res = match self {
            Self::CreateSystem(name, path, system) => {
                let sys = if std::path::PathBuf::from(&system).exists() {
                    SystemConfig::from_config(&system)?
                    // s
                } else {
                    toml::from_str(&system).map_err(ma)?
                };
                let dbs = sys.into_system(&path, &name);
                if let Err(ref e) = dbs {
                    println!("{:?}", e);
                }
                *main_loop = Some(dbs?);
                Response::CreateSystem(format!("Created \"{}\" in \"{}\"", name, path))
            }
            Self::InitialiseFromPath(path) => {
                let mut dbs = LoadedDbs::custom(&path)?;
                let chars = dbs.list_characters()?;
                *main_loop = Some(dbs);
                Response::InitialiseFromPath(chars)
            }
            Self::CreateCharacterSheet(name) => match main_loop {
                Some(ref mut dbs) => {
                    dbs.create_sheet(&name)?;
                    let chars = dbs.list_characters()?;
                    Response::CreateCharacterSheet(chars)
                }
                None => Response::Err(String::from("Load system first.")),
            },
            Self::CreateUpdateCharacter(sheet) => match main_loop {
                Some(ref mut dbs) => {
                    dbs.create_or_update_character(sheet)?;
                    let chars = dbs.list_characters()?;
                    Response::CreateUpdateCharacter(chars)
                }
                None => Response::Err(String::from("Load system first.")),
            },
            Self::ListCharacters => match main_loop {
                Some(ref mut dbs) => {
                    let chars = dbs.list_characters()?;
                    Response::ListCharacters(chars)
                }
                None => Response::Err(String::from("Load system first.")),
            },
            Self::LoadCharacter(name, uuid) => match main_loop {
                Some(ref mut dbs) => {
                    let char = dbs.load_character((name, uuid));
                    Response::LoadCharacter(char?)
                }
                None => Response::Err(String::from("Load system first.")),
            },
            Self::Roll(dice) => {
                let roll = libazdice::parse::parse(dice)?.roll();
                let totals = roll
                    .get_dice_groups()
                    .iter()
                    .map(|r| r.total())
                    .collect::<Vec<_>>();
                let bonus = roll.get_bonus().total();
                Response::Roll(totals, bonus)
            }
            Self::Invalid(x) => Response::Invalid(format!("Invalid request received:({})", x)),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::requests::Request;
    #[test]
    fn make_create_list_request() {
        println!("{:?}", serde_json::to_string(&Request::ListCharacters));
        println!(
            "{:?}",
            serde_json::to_string(&Request::CreateSystem(
                String::from("a"),
                String::from("b"),
                String::from("c")
            ))
        );
        panic!()
    }
}
