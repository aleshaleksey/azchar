//! This deals with requests.
use azchar_database::character::attribute::{AttributeKey, AttributeValue, InputAttribute};
use azchar_database::character::character::InputCharacter;
use azchar_database::character::character::{CharacterPart, CompleteCharacter};
use azchar_database::character::image::{Image, InputImage};
use azchar_database::character::note::{InputNote, Note};
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
    /// A function particularly for removing a part.
    /// The strings are name && uuid of the character, the id is the part id.
    DeletePart(String, String, i64),
    /// Inserting an image requires the (name, uuid) and main character,
    /// as well as the InputImage (an id and path).
    InsertUpdateImage(String, String, InputImage),
    /// Adds a new note. Requires the (name, uuid) of the character it belongs to.
    InsertNote(String, String, InputNote),
    /// Update Note. Requires the (name, uuid) of the character it belongs to.
    UpdateNote(String, String, Note),
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
    /// Same applies when we destroy a part.
    CreateDeleteAttributePart(CompleteCharacter),
    /// We need the actual image data here.
    InsertUpdateImage(Image),
    /// We do not need to retrieve anything for this.
    UpdateNote,
    /// When creating a note we need to return the id and date.
    InsertNote(Note),
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
    pub(crate) fn convert(input: &str) -> Self {
        match toml::from_str(input) {
            Ok(r) => r,
            Err(_) => match serde_json::from_str(input) {
                Ok(r) => r,
                Err(e) => {
                    println!("parse error: {:?}", e);
                    Self::Invalid(input.to_owned())
                }
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
            Self::DeletePart(name, uuid, part_id) => match main_loop {
                Some(ref mut dbs) => {
                    let updated = dbs.delete_part(part_id, (name, uuid))?;
                    Response::CreateDeleteAttributePart(updated)
                }
                None => Response::load_db_error(Self::DeletePart(name, uuid, part_id)),
            },
            Self::InsertUpdateImage(name, uuid, input_image) => match main_loop {
                Some(ref mut dbs) => {
                    Response::InsertUpdateImage(dbs.create_update_image(name, uuid, input_image)?)
                }
                None => Response::load_db_error(Self::InsertUpdateImage(name, uuid, input_image)),
            },
            Self::InsertNote(name, uuid, new_note) => match main_loop {
                Some(ref mut dbs) => Response::InsertNote(dbs.add_note(name, uuid, new_note)?),
                None => Response::load_db_error(Self::InsertNote(name, uuid, new_note)),
            },
            Self::UpdateNote(name, uuid, mut note) => match main_loop {
                Some(ref mut dbs) => {
                    if let Some(ref mut c) = note.content {
                        *c = c.replace("[[enter]]", "\n");
                    }
                    dbs.update_note(name, uuid, note)?;
                    Response::UpdateNote
                }
                None => Response::load_db_error(Self::UpdateNote(name, uuid, note)),
            },
            Self::CreatePart(name, uuid, part) => match main_loop {
                Some(ref mut dbs) => {
                    Response::CreateDeleteAttributePart(dbs.create_part(part, (name, uuid))?)
                }
                None => Response::load_db_error(Self::CreatePart(name, uuid, part)),
            },
            Self::CreateAttribute(name, uuid, attr) => match main_loop {
                Some(ref mut dbs) => {
                    let res = dbs.create_attribute(attr, (name, uuid))?;
                    Response::CreateDeleteAttributePart(res)
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
            \"attributes\":[],\
            \"image\":null}"
            .to_string();

        let exp = format!("{{\"UpdatePart\":[\"{}\",\"{}\",{}]}}", eur, uuid, part,);
        assert_eq!(
            exp,
            serde_json::to_string(&Request::UpdatePart(eur, uuid, p1)).unwrap()
        )
    }

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

    #[test]
    fn make_delete_character_part() {
        let eur = "Euridice".to_string();
        let uuid = "5936ce00-2275-463c-106a-0f2edde38175".to_string();
        let id = 42;
        let exp = format!(
            "{{\"DeletePart\":[\
        \"{}\",\
        \"{}\",\
        {}]}}",
            eur, uuid, id
        );
        assert_eq!(
            exp,
            serde_json::to_string(&Request::DeletePart(eur, uuid, id)).unwrap(),
        );
    }
}
