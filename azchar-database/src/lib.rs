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

extern crate azchar_config;
extern crate azchar_error;

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

/// To do when a sheet is created.
pub fn set_pragma(c: &SqliteConnection) -> Result<(), String> {
    c.execute("pragma analysis_limit=500;").map_err(ma)?;
    c.execute("pragma foreign_keys=off;").map_err(ma)?;
    c.execute("pragma journal_mode = WAL;").map_err(ma)?;
    c.execute("pragma synchronous = off;").map_err(ma)?;
    c.execute("pragma temp_store = memory;").map_err(ma)?;
    c.execute("pragma wal_checkpoint(TRUNCATE);").map_err(ma)?;
    c.execute("pragma locking_mode=EXCLUSIVE;").map_err(ma)?;
    c.execute("pragma wal_autocheckpoint = 2000;").map_err(ma)?;
    c.execute("pragma optimize;").map_err(ma)?;
    Ok(())
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
        set_pragma(&c)?;
        self.connection = Some(c);
        Ok(self.connection.as_ref().expect("Is there."))
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

    /// Do the thing where you tidy up before closing.
    fn tidy_up(conn: &Option<SqliteConnection>) -> Result<(), String> {
        if let Some(c) = conn {
            c.execute("pragma optimize;").map_err(ma)?;
            c.execute("vacuum;").map_err(ma)?;
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
