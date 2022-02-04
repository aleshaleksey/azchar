//! This deals with the base connections for the root db and outer dbs.
use super::BasicConnection;
use crate::character::attribute::{AttributeKey, AttributeValue, Attributes, NewAttribute};
use crate::character::character::InputCharacter;
use crate::character::character::{Character, CharacterPart, CompleteCharacter, NewCharacter};
use crate::root_db::system::{PermittedAttribute, PermittedPart};
use crate::shared::*;
use crate::Config;

use azchar_error::ma;
use diesel::result::Error as DsError;
use diesel::Connection;
use fnv::FnvHashMap;

pub mod attributes;
pub mod characters;
pub mod system;
pub mod system_config;
#[cfg(test)]
mod tests;

pub use characters::{CharacterDbRef, NewCharacterDbRef};

use diesel::{RunQueryDsl, SqliteConnection};
use uuid_rs::v4;

use std::fs::File;
use std::path::PathBuf;

embed_migrations!("migrations_main");

/// A structure that stores the root database connection and the character
/// files it refers to.
pub struct LoadedDbs {
    pub(crate) root_db: BasicConnection,
    // Connections to character sheets with the character name and connections.
    connections: FnvHashMap<(String, String), BasicConnection>,
    // This shows permitted attributes.
    pub(crate) permitted_attrs: Vec<PermittedAttribute>,
    /// This shows parts keys.
    pub(crate) permitted_parts: Vec<PermittedPart>,
    /// Keep the config around.
    root_path: String,
}

impl LoadedDbs {
    /// Load databases from standard configuration.
    pub fn from_config(cfg: Config) -> Result<Self, String> {
        Self::custom(cfg.get_root_db_path())
    }

    /// Load databases from a custom path.
    pub fn custom(path: &str) -> Result<Self, String> {
        let mut root_db = BasicConnection::new(path);
        let connections = CharacterDbRef::get_all(root_db.connect()?)?
            .into_iter()
            .map(|refs| ((refs.name, refs.uuid), BasicConnection::new(&refs.db_path)))
            .collect::<FnvHashMap<(String, String), BasicConnection>>();
        let permitted_attrs = PermittedAttribute::load_all(root_db.connect()?)?;
        let permitted_parts = PermittedPart::load_all(root_db.connect()?)?;
        Ok(LoadedDbs {
            root_db,
            connections,
            permitted_attrs,
            permitted_parts,
            root_path: path.to_string(),
        })
    }

    /// This function is used to refresh one's own status.
    pub fn refresh_and_list(&mut self) -> Result<Vec<CharacterDbRef>, String> {
        self.root_db.connect()?;
        let connections = CharacterDbRef::get_all(self.root_db.connect()?)?;

        self.connections = connections
            .iter()
            .cloned()
            .map(|refs| ((refs.name, refs.uuid), BasicConnection::new(&refs.db_path)))
            .collect::<FnvHashMap<(String, String), BasicConnection>>();
        Ok(connections)
    }

    /// A special case for creating a new system.
    /// NB, we do not load parts till later, because they do not exist yet!
    pub fn new_system(path: &str) -> Result<Self, String> {
        let root_db = BasicConnection::new(path);
        Ok(LoadedDbs {
            root_db,
            connections: FnvHashMap::default(),
            permitted_attrs: Vec::new(),
            permitted_parts: Vec::new(),
            root_path: path.to_string(),
        })
    }

    /// Reference to basic connection.
    pub fn root_connection(&self) -> &BasicConnection {
        &self.root_db
    }

    /// Needs to be connected.
    pub fn get_inner_root(&mut self) -> Result<&SqliteConnection, String> {
        self.root_db.connect()?;
        Ok(self.root_db.get_inner().expect("We just created it."))
    }

    /// Drop inner root.
    pub fn drop_inner_root(&mut self) {
        self.root_db.drop_inner()
    }

    /// Get other connections.
    pub fn character_connections(&self) -> &FnvHashMap<(String, String), BasicConnection> {
        &self.connections
    }

    /// Create a new character sheet database.
    /// Returns the character name and uuid.
    pub fn create_sheet(&mut self, name: &str) -> Result<(String, String), String> {
        use crate::character::attribute::attributes::dsl as at_dsl;
        use crate::character::character::characters::dsl as ch_dsl;
        use crate::root_db::characters::character_dbs::dsl::character_dbs;

        let uuid = v4!();
        // Sanity check.
        if self
            .connections
            .get(&(uuid.clone(), name.to_owned()))
            .is_some()
        {
            return Err(format!("{} already exists as a file! Try again.", name));
        }
        let file_name = format!("{}_{}.db", name, uuid);
        let file_path = PathBuf::from(&self.root_path)
            .parent()
            .expect("Root path is file. Has parent.")
            .join(&file_name);
        if file_path.exists() {
            return Err(format!(
                "{:?} already exists as a file! Try again.",
                file_path
            ));
        }

        // Create the file.
        let _sheet_db = File::create(file_path.clone()).map_err(ma)?;
        let file_path = file_path.to_string_lossy().to_owned().to_string();
        let reference = NewCharacterDbRef::new(name.to_owned(), file_path.clone(), uuid.clone());

        // Clean up if we can't create the character sheet.
        let root_conn = self.get_inner_root()?;
        match diesel::insert_into(character_dbs)
            .values(&vec![reference])
            .execute(root_conn)
            .map_err(ma)
        {
            Ok(_) => {}
            Err(e) => {
                std::fs::remove_file(file_path).map_err(ma)?;
                return Err(e);
            }
        }

        // Connect to the new character sheet.
        let mut sheet_conn_outer = BasicConnection::new(&file_path);
        let sheet_conn = sheet_conn_outer.connect()?;
        crate::set_pragma(sheet_conn)?;

        // Create all needed tables
        let then = std::time::Instant::now();
        embedded_migrations::run(sheet_conn).map_err(ma)?;
        let t1 = then.elapsed().as_micros();
        // Create and place a new part.
        let mut main_new_part: NewCharacter = self
            .permitted_parts
            .iter()
            .find(|p| matches!(p.part_type, Part::Main))
            .map(Into::into)
            .expect("There's always a new part");
        main_new_part.name = name.to_owned();
        main_new_part.uuid = uuid.to_owned();

        sheet_conn
            .transaction::<_, DsError, _>(|| {
                diesel::insert_into(ch_dsl::characters)
                    .values(main_new_part)
                    .execute(sheet_conn)?;
                let char_id = Character::get_latest_id(sheet_conn)?;

                let new_subparts: Vec<NewCharacter> = self
                    .permitted_parts
                    .iter()
                    .filter(|p| p.obligatory && !matches!(p.part_type, Part::Main))
                    .map(|p| {
                        let mut p: NewCharacter = p.into();
                        p.belongs_to = Some(char_id);
                        p
                    })
                    .collect();

                // Da chunking!
                for chunk in new_subparts.chunks(999) {
                    diesel::insert_into(ch_dsl::characters)
                        .values(chunk)
                        .execute(sheet_conn)?;
                }

                // All character parts created here are obligatory.
                let identifiers: Vec<Character> = ch_dsl::characters.load(sheet_conn)?;

                let mut new_attributes = Vec::new();
                for p in identifiers {
                    let attr_iter = self
                        .permitted_attrs
                        .iter()
                        .filter(|a| a.obligatory_for_part(p.part_type, &p.character_type))
                        .map(|a| NewAttribute::from_permitted(p.id, a));
                    new_attributes.extend(attr_iter);
                }

                // Da chunking!
                for chunk in new_attributes.chunks(999) {
                    diesel::insert_into(at_dsl::attributes)
                        .values(chunk)
                        .execute(sheet_conn)?;
                }
                Ok(())
            })
            .map_err(ma)?;

        let t2 = then.elapsed().as_micros();
        println!("migrations:{}us", t1);
        println!("insertions:{}us", t2 - t1);

        self.connections
            .insert((name.to_owned(), uuid.clone()), sheet_conn_outer);

        Ok((name.to_string(), uuid))
    }

    /// Create or update character.
    /// Take a JSON and either a) create a character or b) update a character
    /// Depending on whether the character exists in the current instance.
    pub fn create_or_update_character(
        &mut self,
        character: CompleteCharacter,
    ) -> Result<(), String> {
        let then = std::time::Instant::now();
        let key = (character.name.to_owned(), character.uuid().to_owned());
        println!("{:?}", key);
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            character.save(
                conn.connect()?,
                (&self.permitted_attrs, &self.permitted_parts),
            )?;
            let x = then.elapsed().as_micros();
            conn.drop_inner();
            println!("drop-{}us", x);
            return Ok(());
        }
        let key = self.create_sheet(&key.0)?;
        let conn = self.connections.get_mut(&key).expect("Just created");
        character.save(
            conn.connect()?,
            (&self.permitted_attrs, &self.permitted_parts),
        )?;
        conn.drop_inner();
        let x = then.elapsed().as_micros();
        println!("drop-{}us", x);
        Ok(())
    }

    /// This is used to get a list of characters.
    /// These are the keys to the database.
    pub fn list_characters(&mut self) -> Result<Vec<CharacterDbRef>, String> {
        self.refresh_and_list()
    }

    /// This is used to get the character list as a JSON string.
    pub fn list_characters_json(&mut self) -> Result<String, String> {
        serde_json::to_string(&self.refresh_and_list()?).map_err(ma)
    }

    /// A function to load a character.
    pub fn load_character(&mut self, key: (String, String)) -> Result<CompleteCharacter, String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            CompleteCharacter::load(conn.connect()?)
        } else {
            Err(format!(
                "Character ({}, uuid = {}) not found in this database",
                key.0, key.1
            ))
        }
    }

    /// Load a character as a string. Ready for consumption.
    pub fn load_character_as_json(&mut self, key: (String, String)) -> Result<String, String> {
        serde_json::to_string(&self.load_character(key)?).map_err(ma)
    }

    pub fn create_attribute(
        &mut self,
        new_attr: NewAttribute,
        key: (String, String),
    ) -> Result<CompleteCharacter, String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            let c = conn.connect()?;
            new_attr.checked_insert(c, &self.permitted_attrs)?;
            CompleteCharacter::load(c)
        } else {
            Err(format!(
                "Character with identifier {}-{} not found.",
                key.0, key.1
            ))
        }
    }

    pub fn create_update_attribute(
        &mut self,
        attr_key: AttributeKey,
        attr_value: AttributeValue,
        key: (String, String),
    ) -> Result<(), String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            Attributes::insert_update_key_value(&attr_key, &attr_value, conn.connect()?)
        } else {
            Err(format!(
                "Character with identifier {}-{} not found.",
                key.0, key.1
            ))
        }
    }

    pub fn create_update_part(
        &mut self,
        part: CharacterPart,
        key: (String, String),
    ) -> Result<(), String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            CompleteCharacter::insert_update_character_part(
                part,
                conn.connect()?,
                &self.permitted_parts,
                &self.permitted_attrs,
            )
        } else {
            Err(format!(
                "Character with identifier {}-{} not found.",
                key.0, key.1
            ))
        }
    }

    pub fn create_part(
        &mut self,
        new_part: InputCharacter,
        key: (String, String),
    ) -> Result<CompleteCharacter, String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            let c = conn.connect()?;
            NewCharacter::from_input(new_part).checked_insert(
                c,
                &self.permitted_parts,
                &self.permitted_attrs,
            )?;
            CompleteCharacter::load(c)
        } else {
            Err(format!(
                "Character with identifier {}-{} not found.",
                key.0, key.1
            ))
        }
    }

    pub fn delete_character(&mut self, char_name: String, char_uuid: String) -> Result<(), String> {
        use crate::diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
        use crate::root_db::characters::character_dbs::dsl::*;

        let key = (char_name.clone(), char_uuid.clone());
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            ::diesel::delete(character_dbs.filter(name.eq(&char_name).and(uuid.eq(&char_uuid))))
                .execute(self.root_db.connect()?)
                .map_err(ma)?;
            match std::fs::remove_file(&conn.db_path) {
                Ok(()) => Ok(()),
                Err(_) => match std::fs::remove_file(&conn.db_path) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("{}", e)),
                },
            }
        } else {
            Err(format!(
                "Character with identifier {}-{} not found.",
                key.0, key.1
            ))
        }
    }
}
