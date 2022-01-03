//! This deals with the base connections for the root db and outer dbs.
use super::BasicConnection;
use crate::database::root_db::system::PermittedPart;
use crate::database::character::character::CompleteCharacter;
use crate::database::Config;
use crate::error::ma;

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

/// A structure that stores the root database connection and the character
/// files it refers to.
pub struct LoadedDbs {
    root_db: BasicConnection,
    // Connections to character sheets with the character name and connections.
    connections: FnvHashMap<(String, String), BasicConnection>,
    // This shows permitted attributes.
    attribute_keys: u8,
    /// This shows parts keys.
    permitted_parts: u8,
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
        root_db.connect()?;
        let connections = root_db
            .get_all_char_refs()?
            .into_iter()
            .map(|refs| ((refs.name, refs.uuid), BasicConnection::new(&refs.db_path)))
            .collect::<FnvHashMap<(String, String), BasicConnection>>();
        Ok(LoadedDbs {
            root_db,
            connections,
            attribute_keys: 0,
            permitted_parts: 0,
            root_path: path.to_string(),
        })
    }

    /// A special case for creating a new system.
    pub fn new_system(path: &str) -> Result<Self, String> {
        let mut root_db = BasicConnection::new(path);
        root_db.connect()?;
        Ok(LoadedDbs {
            root_db,
            connections: FnvHashMap::default(),
            attribute_keys: 0,
            permitted_parts: 0,
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

    /// Get other connections.
    pub fn character_connections(&self) -> &FnvHashMap<(String, String), BasicConnection> {
        &self.connections
    }

    /// Create a new character sheet database.
    /// Returns the character name and uuid.
    pub fn create_sheet(&mut self, name: &str) -> Result<(String, String), String> {
        use crate::database::root_db::characters::character_dbs::dsl::character_dbs;
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
        let reference = NewCharacterDbRef::new(name.to_owned(), file_name, uuid.clone());

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
        let file_path = file_path.to_string_lossy();
        let mut sheet_conn_outer = BasicConnection::new(&file_path);
        let sheet_conn = sheet_conn_outer.connect()?;
        // Create all needed tables
        embed_migrations!("migrations_main");
        embedded_migrations::run(sheet_conn).map_err(ma)?;
        // Create all obligatory parts.
        PermittedPart::create_basic(root_conn, sheet_conn, name)?;
        self.connections
            .insert((name.to_owned(), uuid.clone()), sheet_conn_outer);
        Ok((name.to_string(), uuid))
    }

    /// Create or update character.
    /// Take a JSON and either a) create a character or b) update a character
    /// Depending on whether the character exists in the current instance.
    pub fn create_or_update_character(&mut self, c: String) -> Result<(), String> {
        // Create a character.
        let character: CompleteCharacter = match toml::from_str(&c) {
            Err(_) => serde_json::from_str(&c).map_err(ma)?,
            Ok(c) => c,
        };
        let key = (character.name.to_owned(), character.uuid().to_owned());
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            character.save(conn.connect()?)?;
        } else {
            let key = self.create_sheet(&key.0)?;
            let conn = self.connections.get_mut(&key).expect("Just created");
            character.save(conn.connect()?)?;
        }
        Ok(())
    }

    /// This is used to get a list of characters.
    /// These are the keys to the database.
    pub fn list_characters(&mut self) -> Result<Vec<CharacterDbRef>, String> {
        CharacterDbRef::get_all(self.root_db.connect()?)
    }

    /// This is used to get the character list as a JSON string.
    pub fn list_characters_json(&mut self) -> Result<String, String> {
        serde_json::to_string(&self.list_characters()?).map_err(ma)
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
}
