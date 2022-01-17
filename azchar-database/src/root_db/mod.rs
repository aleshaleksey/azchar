//! This deals with the base connections for the root db and outer dbs.
use super::BasicConnection;
use crate::character::attribute::{Attribute, AttributeKey, AttributeValue, Attributes};
use crate::character::character::{Character, CharacterPart, CompleteCharacter};
use crate::root_db::system::{PermittedAttribute, PermittedPart};
use crate::shared::*;
use crate::Config;

use azchar_error::ma;
use fnv::FnvHashMap;

pub mod characters;
pub mod system;
pub mod system_config;
#[cfg(test)]
mod tests;

pub use characters::CharacterDbRef;

use rusqlite::Connection as SqliteConnection;
use uuid_rs::v4;

use std::fs::File;
use std::path::PathBuf;

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
    pub fn get_inner_root(&mut self) -> Result<&mut SqliteConnection, String> {
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
        let refr = CharacterDbRef::new(name.to_owned(), file_name, uuid.clone())?;

        // Clean up if we can't create the character sheet.
        let root_conn = self.get_inner_root()?;
        match root_conn
            .prepare_cached("INSERT INTO character_dbs (name, uuid, db_path) VALUES (?,?,?);")
            .map_err(ma)?
            .execute(params![refr.name, refr.uuid, refr.db_path])
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
        crate::set_pragma(sheet_conn)?;

        // Create all needed tables
        let then = std::time::Instant::now();
        //// Start, transaction.
        {
            sheet_conn.execute_batch(SHEET_MIGRATION).map_err(ma)?;
            let t1 = then.elapsed().as_micros();
            let sheet_tx = BasicConnection::default_tx(sheet_conn)?;
            // Create and place a new part.
            let mut main_new_part: Character = self
                .permitted_parts
                .iter()
                .find(|p| matches!(p.part_type, Part::Main))
                .map(Into::into)
                .expect("There's always a new part");
            main_new_part.name = name.to_owned();
            main_new_part.uuid = uuid.to_owned();

            main_new_part.insert_single(&sheet_tx).map_err(ma)?;
            let char_id = Character::get_latest_id(&sheet_tx)?;

            let new_subparts: Vec<Character> = self
                .permitted_parts
                .iter()
                .filter(|p| p.obligatory && !matches!(p.part_type, Part::Main))
                .map(|p| {
                    let mut p: Character = p.into();
                    p.belongs_to = Some(char_id);
                    p
                })
                .collect();

            // Da chunking!
            for chunk in new_subparts {
                chunk.insert_single(&sheet_tx)?;
            }

            // All character parts created here are obligatory.
            let identifiers: Vec<Character> = Character::load_all(&sheet_tx)?;

            for p in identifiers {
                let attr_iter = self
                    .permitted_attrs
                    .iter()
                    .filter(|a| {
                        a.obligatory
                            && a.part_name == p.character_type
                            && a.part_type == p.part_type
                    })
                    .map(|a| Attribute {
                        id: None,
                        key: a.key.to_owned(),
                        value_num: None,
                        value_text: None,
                        description: Some(a.attribute_description.to_owned()),
                        of: p.id.expect("It was inserted."),
                    });
                for a in attr_iter {
                    a.insert_single(&sheet_tx).map_err(ma)?;
                }
            }
            sheet_tx.commit().map_err(ma)?;
            let t2 = then.elapsed().as_micros();
            println!("migrations:{}us", t1);
            println!("insertions:{}us", t2 - t1);
        }
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
            let sheet_tx = BasicConnection::default_tx(conn.connect()?)?;
            character.save(&sheet_tx, (&self.permitted_attrs, &self.permitted_parts))?;
            sheet_tx.commit().map_err(ma)?;

            let x = then.elapsed().as_micros();
            conn.drop_inner();
            println!("drop-{}us", x);
            return Ok(());
        }
        let key = self.create_sheet(&key.0)?;
        let conn = self.connections.get_mut(&key).expect("Just created");
        let sheet_tx = BasicConnection::default_tx(conn.connect()?)?;

        character.save(&sheet_tx, (&self.permitted_attrs, &self.permitted_parts))?;
        sheet_tx.commit().map_err(ma)?;
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

    /// A function to load a character.
    pub fn load_character(&mut self, key: (String, String)) -> Result<CompleteCharacter, String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            let sheet_tx = BasicConnection::default_tx(conn.connect()?)?;
            let sheet = CompleteCharacter::load(&sheet_tx)?;
            sheet_tx.commit().map_err(ma)?;
            Ok(sheet)
        } else {
            Err(format!(
                "Character ({}, uuid = {}) not found in this database",
                key.0, key.1
            ))
        }
    }

    pub fn create_update_attribute(
        &mut self,
        attr_key: AttributeKey,
        attr_value: AttributeValue,
        key: (String, String),
    ) -> Result<usize, String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            let sheet_tx = BasicConnection::default_tx(conn.connect()?)?;
            let a = Attributes::insert_update_key_value(&attr_key, &attr_value, &sheet_tx)?;
            sheet_tx.commit().map_err(ma)?;
            Ok(a)
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
    ) -> Result<usize, String> {
        if let Some(ref mut conn) = self.connections.get_mut(&key) {
            let sheet_tx = BasicConnection::default_tx(conn.connect()?)?;
            let p = CompleteCharacter::insert_update_character_part(part, &sheet_tx)?;
            sheet_tx.commit().map_err(ma)?;
            Ok(p)
        } else {
            Err(format!(
                "Character with identifier {}-{} not found.",
                key.0, key.1
            ))
        }
    }
}

const ROOT_MIGRATION: &str = "
-- Create the root table.
create table character_dbs(
	id INTEGER primary key AUTOINCREMENT,
	name TEXT NOT NULL,
	uuid TEXT NOT NULL,
	db_path TEXT NOT NULL
);

create table permitted_attributes(
	key TEXT NOT NULL primary key,
	attribute_type INTEGER NOT NULL,
	attribute_description TEXT NOT NULL,
	part_name TEXT NOT NULL,
	part_type INTEGER NOT NULL,
	obligatory BOOLEAN NOT NULL
	-- UNIQUE(part_name, part_type)
);

create table permitted_parts(
	id INTEGER primary key AUTOINCREMENT,
	part_name TEXT NOT NULL,
	-- Should be an enum which is shared with characters..
	part_type INTEGER NOT NULL,
	obligatory BOOLEAN NOT NULL
);
";

const SHEET_MIGRATION: &str = "
-- Create the root table
-- NB: Everything is a character, including, bodyparts, items and spells.
-- All things have a name, type, weight,
create table characters(
  -- The basic fields.
  id INTEGER primary key AUTOINCREMENT,
  name TEXT NOT NULL,
  uuid TEXT NOT NULL UNIQUE,
  character_type TEXT NOT NULL, -- NB: Text for maximum flexibility.
  -- Basic, vital attributes for characters.
  -- NB: Might not be relevant to all things.
  speed INTEGER NOT NULL,
  weight INTEGER,
  size TEXT,
  -- Not all things have hitpoints, mp. etc.
  hp_total INTEGER,
  hp_current INTEGER,
  -- What, if anything does this character belong to?
  belongs_to BIGINT references characters(id),
  -- What kind of part is it. See appropriate type.
  part_type INTEGER
);

-- A basic set of keys and values.

create table attributes(
  id INTEGER primary key AUTOINCREMENT,
  key TEXT NOT NULL,
  value_num INTEGER,
  value_text TEXT,
  description TEXT,
  of BIGINT NOT NULL references characters(id),
  UNIQUE(key, of)
);

create index if not exists attributes_idx on attributes(id);
create index if not exists attributes_ofx on attributes(of);
create index if not exists attributes_keyx on attributes(key);
create index if not exists characters_idx on characters(id);
create index if not exists characters_belonx on characters(belongs_to);
create index if not exists characters_belonx on characters(part_type);
";
