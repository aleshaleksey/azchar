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

mod config;
mod database;
mod error;
mod roller;

use crate::database::LoadedDbs;

macro_rules! do_or_die {
    ($result:expr) => {
        match $result {
            Ok(r) => r,
            Err(e) => {
                println!("Big fail: {:?}", e);
                return;
            }
        }
    };
}

fn main() {
    let config = do_or_die!(config::Config::from_path("examples/example.toml"));
    let loaded_dbs = do_or_die!(LoadedDbs::from_config(config));

    println!("Root db: {:?}", loaded_dbs.root_connection());
    for ((name, uuid), connection) in loaded_dbs.character_connections().iter() {
        println!("char name: {}, uuid:{}, conn: {:?}", name, uuid, connection);
    }
}
