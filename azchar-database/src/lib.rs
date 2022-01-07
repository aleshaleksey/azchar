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
extern crate fnv;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

extern crate azchar_error;
extern crate azchar_config;

use azchar_config::Config;
use azchar_error::ma;
use diesel::{Connection, SqliteConnection};

pub mod character;
pub mod root_db;
mod shared;

pub use root_db::{CharacterDbRef, LoadedDbs};

pub(self) const MIGRATIONS_MAIN: &str = "migrations_main";
pub(self) const MIGRATIONS_ROOT: &str = "migrations_root_db";

/// Represents a basic connection to an sqlite database.
pub struct BasicConnection {
    db_path: String,
    connection: Option<SqliteConnection>,
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
    pub fn connect(&mut self) -> Result<&SqliteConnection, String> {
        if let Some(ref con) = self.connection {
            return Ok(con);
        }

        let c = SqliteConnection::establish(&self.db_path).map_err(ma)?;
        self.connection = Some(c);
        Ok(self.connection.as_ref().expect("Is there."))
    }

    /// Gets a reference to the inner connection so we can actually use it.
    pub fn get_inner(&mut self) -> Option<&SqliteConnection> {
        self.connection.as_ref()
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
}
