//! This deals with the base connections for the root db and outer dbs.
use super::BasicConnection;
use crate::database::root_db::system::PermittedPart;
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

use diesel::result::Error as DError;
use diesel::{RunQueryDsl, SqliteConnection};
use uuid_rs::v4;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::database::MIGRATIONS_MAIN;

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
    pub fn create_sheet(&mut self, name: &str) -> Result<(), String> {
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
        let file_path = PathBuf::from(&self.root_path).join(&file_name);
        if file_path.exists() {
            return Err(format!(
                "{:?} already exists as a file! Try again.",
                file_path
            ));
        }

        // Create the file.
        let _sheet_db = File::create(file_path.clone()).map_err(ma)?;
        let reference = NewCharacterDbRef::new(name.to_owned(), file_name, uuid);

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
        embed_migrations!("/home/alesha/Code/rustcodes/azchar/migrations_main");
        embedded_migrations::run_with_output(sheet_conn);

        // Create all obligatory parts.
        PermittedPart::create_basic(root_conn, sheet_conn)?;

        Ok(())
    }
}
