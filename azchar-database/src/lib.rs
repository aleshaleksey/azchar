//! The base package for the database.
#![allow(dead_code)]
#![allow(deprecated)]
#![allow(clippy::field_reassign_with_default)]
extern crate serde_json;
#[cfg(test)]
extern crate tempfile;
extern crate toml;
extern crate uuid_rs;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rusqlite;
extern crate fnv;

extern crate azchar_config;
extern crate azchar_error;

use azchar_config::Config;
use azchar_error::ma;

pub mod character;
pub mod root_db;
mod shared;
use rusqlite::Connection as SqliteConnection;

pub use root_db::{CharacterDbRef, LoadedDbs};

pub(self) const MIGRATIONS_MAIN: &str = "migrations_main";
pub(self) const MIGRATIONS_ROOT: &str = "migrations_root_db";

pub(self) const CHARS_TABLE: &str = "characters";
pub(self) const ATTRS_TABLE: &str = "attributes";
pub(self) const CHARACTER_DBS_TABLE: &str = "character_dbs";
pub(self) const PERMITTED_CHARS_TABLE: &str = "permitted_parts";
pub(self) const PERMITTED_ATTRS_TABLE: &str = "permitted_attributes";

/// Represents a basic connection to an sqlite database.
pub struct BasicConnection {
    db_path: String,
    connection: Option<SqliteConnection>,
}

/// A check to see if an input string is OK to use in SQL.
pub(crate) fn check_name_string(input: String) -> Result<String, String> {
    if input
        .chars()
        .all(|c| c != '\\' && (c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '.'))
    {
        Ok(input)
    } else {
        Err("Not a valid name string.".to_string())
    }
}

/// To do when a sheet is created.
pub fn set_pragma(c: &SqliteConnection) -> Result<(), String> {
    c.execute_batch(
        "
BEGIN;
pragma analysis_limit=500;
pragma foreign_keys=off;
pragma journal_mode = WAL;
pragma synchronous = off;
pragma temp_store = memory;
pragma wal_checkpoint(TRUNCATE);
pragma locking_mode=EXCLUSIVE;
pragma wal_autocheckpoint = 2000;
pragma optimize;
COMMIT;",
    )
    .map_err(ma)
}

impl std::fmt::Debug for BasicConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("BasicConnection")
            .field("db_path", &self.db_path)
            .field("connection", &self.connection.is_some())
            .finish()
    }
}

impl BasicConnection {
    pub(crate) fn path(&self) -> &str {
        &self.db_path
    }
    /// Create a connection but do not connect.
    pub fn new(path: &str) -> Self {
        BasicConnection {
            db_path: path.to_owned(),
            connection: None,
        }
    }

    /// Try to connect to an Sqlite Database.
    pub fn connect(&mut self) -> Result<&mut SqliteConnection, String> {
        if let Some(ref mut con) = self.connection {
            return Ok(con);
        }

        let c = SqliteConnection::open(&self.db_path).map_err(ma)?;
        set_pragma(&c)?;
        self.connection = Some(c);
        Ok(self.connection.as_mut().expect("Is there."))
    }

    pub fn drop_connection(&mut self) {
        if let Err(e) = Self::tidy_up(&self.connection) {
            println!(
                "Error {:?} when tidying database for {:?} on close.",
                e, self.db_path
            );
        }
        self.connection = None;
    }

    /// Gets a reference to the inner connection so we can actually use it.
    pub fn get_inner(&mut self) -> Option<&mut SqliteConnection> {
        self.connection.as_mut()
    }

    /// Drops the inner connection.
    pub fn drop_inner(&mut self) {
        self.connection = None;
    }

    /// Used to get all db_refs.
    pub fn get_all_char_refs(&self) -> Result<Vec<CharacterDbRef>, String> {
        match self.connection {
            Some(ref conn) => CharacterDbRef::get_all(conn),
            None => Err("Not connected!".to_string()),
        }
    }

    /// Do the thing where you tidy up before closing.
    fn tidy_up(conn: &Option<SqliteConnection>) -> Result<(), String> {
        if let Some(c) = conn {
            c.execute_batch("pragma optimize;\nvacuum;").map_err(ma)?;
        }
        Ok(())
    }
}

impl Drop for BasicConnection {
    fn drop(&mut self) {
        if let Err(e) = BasicConnection::tidy_up(&self.connection) {
            println!(
                "Error {:?} when tidying database for {:?} on close.",
                e, self.db_path
            );
        }
    }
}
