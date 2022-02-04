//! This deals with requests.
use azchar_database::character::attribute::{AttributeKey, AttributeValue, InputAttribute};
use azchar_database::character::character::InputCharacter;
use azchar_database::character::character::{CharacterPart, CompleteCharacter};
use azchar_database::root_db::system_config::SystemConfig;
use azchar_database::CharacterDbRef;
use azchar_error::ma;

use azchar_database::LoadedDbs;

/// A request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Request {
    /// String is a config file as a TOML.
    // The strings are name && uuid
    CreateSystem(String, String, String),
    /// This is the path of the system to load.
    InitialiseFromPath(String),
    /// This represents the character name.
    CreateCharacterSheet(String),
    /// The string is a CompleteCharacter JSON/TOML.
    CreateUpdateCharacter(CompleteCharacter),
    /// This needs no arguments and uses the current root. [Need identifier]
    // The strings are name && uuid
    UpdateAttribute(String, String, AttributeKey, AttributeValue),
    /// Purely for creating an attribute.
    // The strings are name && uuid
    CreateAttribute(String, String, InputAttribute),
    /// Update a single character part. [Need identifier]
    // The strings are name && uuid
    UpdatePart(String, String, CharacterPart),
    /// A function particularly for adding new parts.
    // The strings are name && uuid
    CreatePart(String, String, InputCharacter),
    /// Delete a character.
    // The strings are name && uuid
    DeleteCharacter(String, String),
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
    /// This needs no arguments and uses the current root.
    UpdateAttribute,
    /// Update a single character part.
    UpdatePart,
    /// We must update the whole character when create an utterly new_attribute o part.
    CreateAttributePart(CompleteCharacter),
    /// Delete Character, return the list.
    DeleteCharacter(Vec<CharacterDbRef>),
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
    Err(String, String),
}

impl Response {
    fn load_db_error(r: Request) -> Self {
        Response::Err(
            String::from("Load system first."),
            serde_json::to_string(&r).unwrap(),
        )
    }
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
        let a = std::time::Instant::now();
        let res = match self {
            Self::CreateSystem(name, path, system) => {
                let sys = if std::path::PathBuf::from(&system).exists() {
                    SystemConfig::from_config(&system)?
                } else {
                    toml::from_str(&system).map_err(ma)?
                };
                let dbs = sys.into_system(&path, &name);
                if let Err(ref e) = dbs {
                    println!("Error in creation: {:?}", e);
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
                None => Response::load_db_error(Self::CreateCharacterSheet(name)),
            },
            Self::CreateUpdateCharacter(sheet) => match main_loop {
                Some(ref mut dbs) => {
                    dbs.create_or_update_character(sheet)?;
                    let chars = dbs.list_characters();
                    Response::CreateUpdateCharacter(chars?)
                }
                None => Response::load_db_error(Self::CreateUpdateCharacter(sheet)),
            },
            Self::UpdateAttribute(name, uuid, attr_k, attr_v) => match main_loop {
                Some(ref mut dbs) => {
                    dbs.create_update_attribute(attr_k, attr_v, (name, uuid))?;
                    Response::UpdateAttribute
                }
                None => Response::load_db_error(Self::UpdateAttribute(name, uuid, attr_k, attr_v)),
            },
            Self::UpdatePart(name, uuid, character) => match main_loop {
                Some(ref mut dbs) => {
                    dbs.create_update_part(character, (name, uuid))?;
                    Response::UpdatePart
                }
                None => Response::load_db_error(Self::UpdatePart(name, uuid, character)),
            },
            Self::CreatePart(name, uuid, part) => match main_loop {
                Some(ref mut dbs) => {
                    Response::CreateAttributePart(dbs.create_part(part, (name, uuid))?)
                }
                None => Response::load_db_error(Self::CreatePart(name, uuid, part)),
            },
            Self::CreateAttribute(name, uuid, attr) => match main_loop {
                Some(ref mut dbs) => {
                    let res = dbs.create_attribute(attr, (name, uuid))?;
                    Response::CreateAttributePart(res)
                }
                None => Response::load_db_error(Self::CreateAttribute(name, uuid, attr)),
            },
            Self::DeleteCharacter(name, uuid) => match main_loop {
                Some(ref mut dbs) => {
                    dbs.delete_character(name, uuid)?;
                    Response::DeleteCharacter(dbs.list_characters()?)
                }
                None => Response::load_db_error(Self::LoadCharacter(name, uuid)),
            },
            Self::ListCharacters => match main_loop {
                Some(ref mut dbs) => {
                    let chars = dbs.list_characters()?;
                    Response::ListCharacters(chars)
                }
                None => Response::load_db_error(Self::ListCharacters),
            },
            Self::LoadCharacter(name, uuid) => match main_loop {
                Some(ref mut dbs) => {
                    let char = dbs.load_character((name, uuid));
                    Response::LoadCharacter(char?)
                }
                None => Response::load_db_error(Self::LoadCharacter(name, uuid)),
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
        let b = a.elapsed().as_micros();
        println!("inner exec: {}us", b);
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

    #[test]
    fn make_update_attribute() {
        use azchar_database::character::attribute::{AttributeKey, AttributeValue};

        let eur = "Euridice".to_string();
        let uuid = "5936ce00-2275-463c-106a-0f2edde38175".to_string();
        let key = "{\"key\":\"attack_power\",\"of\":1}".to_string();
        let value = "{\"id\":null,\"value_num\":null,\"value_text\":\"no\",\"description\":null}"
            .to_string();

        let k1 = AttributeKey::test();
        let v1 = AttributeValue::test();

        let exp = format!(
            "{{\"UpdateAttribute\":[\
        \"{}\",\
        \"{}\",\
        {},\
        {}\
        ]}}",
            eur, uuid, key, value
        );
        assert_eq!(
            exp,
            serde_json::to_string(&Request::UpdateAttribute(eur, uuid, k1, v1)).unwrap(),
        );
    }

    #[test]
    fn make_create_part() {
        use azchar_database::character::character::InputCharacter;

        let eur = "Euridice".to_string();
        let uuid = "5936ce00-2275-463c-106a-0f2edde38175".to_string();
        let part = "{\"name\":\"Memory Thief\",\
            \"character_type\":\"spell\",\
            \"speed\":40,\
            \"weight\":null,\
            \"size\":null,\
            \"hp_total\":30,\
            \"hp_current\":10,\
            \"belongs_to\":1,\
            \"part_type\":\"Ability\"}";

        let p1 = InputCharacter::test();

        let exp = format!(
            "{{\"CreatePart\":[\
        \"{}\",\
        \"{}\",\
        {}\
        ]}}",
            eur, uuid, part
        );
        assert_eq!(
            exp,
            serde_json::to_string(&Request::CreatePart(eur, uuid, p1)).unwrap(),
        );
    }

    #[test]
    fn make_create_attribute() {
        use azchar_database::character::attribute::InputAttribute;
        let eur = "Euridice".to_string();
        let uuid = "5936ce00-2275-463c-106a-0f2edde38175".to_string();

        let part = "{\"key\":\"memory_capacity\",\
            \"value_num\":9999,\
            \"value_text\":\"It's over nine thousand.\",\
            \"description\":null,\
            \"of\":1}"
            .to_string();
        let p1 = InputAttribute::test();

        let exp = format!(
            "{{\"CreateAttribute\":[\"{}\",\"{}\",{}]}}",
            eur, uuid, part
        );
        assert_eq!(
            exp,
            serde_json::to_string(&Request::CreateAttribute(eur, uuid, p1)).unwrap(),
        );
    }

    #[test]
    fn make_update_part() {
        use azchar_database::character::character::CharacterPart;
        let eur = "Euridice".to_string();
        let uuid = "5936ce00-2275-463c-106a-0f2edde38175".to_string();

        let p1 = CharacterPart::test();
        let part = "{\"id\":5,\
            \"name\":\"Memory Thief\",\
            \"uuid\":\"5936ce00-2275-463c-106a-0f2edde38000\",\
            \"character_type\":\"spell\",\
            \"speed\":0,\
            \"weight\":1,\
            \"size\":null,\
            \"hp_total\":null,\
            \"hp_current\":null,\
            \"part_type\":\"Ability\",\
            \"belongs_to\":1,\
            \"attributes\":[]}"
            .to_string();

        let exp = format!("{{\"UpdatePart\":[\"{}\",\"{}\",{}]}}", eur, uuid, part,);
        assert_eq!(
            exp,
            serde_json::to_string(&Request::UpdatePart(eur, uuid, p1)).unwrap()
        )
    }
    // UpdatePart(String, String, CharacterPart),

    #[test]
    fn make_delete_character() {
        let eur = "Euridice".to_string();
        let uuid = "5936ce00-2275-463c-106a-0f2edde38175".to_string();
        let exp = format!(
            "{{\"DeleteCharacter\":[\
        \"{}\",\
        \"{}\"]}}",
            eur, uuid
        );
        assert_eq!(
            exp,
            serde_json::to_string(&Request::DeleteCharacter(eur, uuid)).unwrap(),
        );
    }
}
