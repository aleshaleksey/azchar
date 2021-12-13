use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use crate::error::ma;

#[derive(Debug, Deserialize)]
/// The base configuration file for the app.
pub struct Config {
    root_database: String,
}

impl Config {
    /// Creates a config.
    pub fn from_path(path: &str) -> Result<Self, String> {
        let mut file = BufReader::new(File::open(path).map_err(ma)?);
        let mut input = String::new();

        file.read_to_string(&mut input).map_err(ma)?;
        toml::from_str(&input).map_err(ma)
    }

    /// Get the path without modifying it.
    pub fn get_root_db_path(&self) -> &str {
        &self.root_database
    }
}
