//! This deals with the base connections for the root db and outer dbs.
use crate::error::ma;

use diesel::RunQueryDsl;
use diesel::SqliteConnection;

table! {
    character_dbs(id) {
        id -> BigInt,
        name -> Text,
        uuid -> Text,
        db_path -> Text,
    }
}

/// A structure to store a db ref.
#[derive(Debug, Clone, PartialEq, Queryable, Identifiable)]
#[table_name = "character_dbs"]
pub struct CharacterDbRef {
    id: i64,
    pub(super) name: String,
    pub(super) uuid: String,
    pub(super) db_path: String,
}

impl CharacterDbRef {
    /// Get all in a db.
    pub fn get_all(conn: &SqliteConnection) -> Result<Vec<CharacterDbRef>, String> {
        use self::character_dbs::dsl::*;
        character_dbs.load(conn).map_err(ma)
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "character_dbs"]
pub struct NewCharacterDbRef {
    name: String,
    uuid: String,
    db_path: String,
}

impl NewCharacterDbRef {
    /// NB, this should also make sure that the DB exists.
    /// At the very least, it should not be used outside of a scoped transaction
    /// which creates or checks the existance of the character database.
    pub fn new(name: String, db_path: String, uuid: String) -> Self {
        Self {
            name,
            uuid,
            db_path,
        }
    }
}
