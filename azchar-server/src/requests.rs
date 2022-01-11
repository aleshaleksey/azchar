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
    CreateUpdateCharacter(CompleteCharacter),
    /// This needs no arguments and uses the current root.
    ListCharacters,
    /// The string a name and UUID.
    LoadCharacter(String, String),
    /// Represents a request to parse and run a roll.
    Roll(String),
    /// Shut down the server.
    Shutdown,
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
    /// Shut down the server.
    Shutdown,
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
                    let chars = dbs.list_characters();
                    Response::CreateUpdateCharacter(chars?)
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
            Self::Shutdown => Response::Shutdown,
            Self::Invalid(x) => Response::Invalid(format!("Invalid request received:({})", x)),
        };
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::requests::Request;
    use azchar_database::character::character::CompleteCharacter;
    use std::io::Read;

    #[test]
    fn make_list_characters_request() {
        let exp = "\"ListCharacters\"";
        assert_eq!(
            exp,
            &serde_json::to_string(&Request::ListCharacters).unwrap()
        );
    }

    #[test]
    fn make_create_system_request() {
        let exp = "{\"CreateSystem\":[\"name\",\"db_path\",\"cfg_path\"]}";
        assert_eq!(
            exp,
            &serde_json::to_string(&Request::CreateSystem(
                String::from("name"),
                String::from("db_path"),
                String::from("cfg_path"),
            ))
            .unwrap()
        );
    }

    #[test]
    fn make_initialise_from_path_request() {
        let exp = "{\"InitialiseFromPath\":\"/path/\"}";
        assert_eq!(
            exp,
            &serde_json::to_string(&Request::InitialiseFromPath(String::from("/path/"))).unwrap(),
        );
    }

    #[test]
    fn make_create_character_sheet() {
        let exp = "{\"CreateCharacterSheet\":\"Euridice\"}";
        assert_eq!(
            exp,
            &serde_json::to_string(&Request::CreateCharacterSheet(String::from("Euridice")))
                .unwrap(),
        );
    }

    #[test]
    fn make_create_update_character() {
        let mut ch = String::new();
        let mut file = if let Ok(f) = std::fs::File::open("../examples/dnd5e_minimal_sheet.json") {
            f
        } else {
            std::fs::File::open("examples/dnd5e_minimal_sheet.json").unwrap()
        };
        file.read_to_string(&mut ch).unwrap();
        ch.pop();

        let complete: CompleteCharacter = serde_json::from_str(&ch).unwrap();
        let exp = format!("{{\"CreateUpdateCharacter\":{}}}", ch);
        assert_eq!(
            exp,
            serde_json::to_string(&Request::CreateUpdateCharacter(complete)).unwrap(),
        );
    }

    #[test]
    fn make_create_roll() {
        let exp = "{\"Roll\":\"2d10dl1mx10+1d4+6\"}";
        assert_eq!(
            exp,
            serde_json::to_string(&Request::Roll(String::from("2d10dl1mx10+1d4+6"))).unwrap(),
        );
    }

    #[test]
    fn make_create_invalid() {
        let exp = "{\"Invalid\":\"PoopaScooottta!!!\"}";
        assert_eq!(
            exp,
            serde_json::to_string(&Request::Invalid(String::from("PoopaScooottta!!!"))).unwrap(),
        );
    }

    #[test]
    fn make_load_character() {
        let exp = "{\"LoadCharacter\":[\"Euridice\",\"5936ce00-2275-463c-106a-0f2edde38175\"]}";
        let eur = String::from("Euridice");
        let uuid = String::from("5936ce00-2275-463c-106a-0f2edde38175");
        assert_eq!(
            exp,
            serde_json::to_string(&Request::LoadCharacter(eur, uuid)).unwrap(),
        );
    }
}
