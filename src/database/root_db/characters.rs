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
#[derive(Debug, Clone, PartialEq, Queryable, Identifiable, Serialize, Deserialize)]
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

#[cfg(test)]
mod character_tests {
    use crate::database::character::character::CompleteCharacter;
    use crate::database::root_db::tests::*;
    use crate::database::BasicConnection;

    const NAME1: &str = "Test Character";
    const NAME2: &str = "Test Character 2";
    const NAME3: &str = "Test Character 3";

    pub(crate) fn create_char_with_name<'a>(
        setup: &'a mut TestSetup,
        name: &str,
    ) -> &'a BasicConnection {
        let (name, uuid) = setup
            .loaded_dbs
            .create_sheet(name)
            .expect("Could not create a character.");
        setup
            .loaded_dbs
            .character_connections()
            .get(&(name, uuid))
            .expect("The character exists. We inserted it.")
    }

    #[test]
    fn create_character() {
        let mut setup = setup(TestSystem::MemorySphere);
        create_char_with_name(&mut setup, NAME1);
    }

    #[test]
    fn create_character_dnd5e() {
        let mut setup = setup(TestSystem::DnD5);
        create_char_with_name(&mut setup, NAME1);
    }

    #[test]
    fn create_and_load_character() {
        let mut setup = setup(TestSystem::MemorySphere);
        let conn = create_char_with_name(&mut setup, NAME1);
        if let Some(ref conn) = conn.connection {
            let c = CompleteCharacter::load(conn).expect("I'm here.");
            println!("char:{:?}", c);
            println!("char:{:?}", serde_json::to_string(&c).expect("yes"));
            assert_eq!(&c.name, NAME1);
        }
    }

    #[test]
    fn create_and_load_character_dnd5() {
        let mut setup = setup(TestSystem::DnD5);
        let conn = create_char_with_name(&mut setup, NAME1);
        if let Some(ref conn) = conn.connection {
            let c = CompleteCharacter::load(conn).expect("I'm here.");
            println!("char:{:?}", c);
            println!("char:{:?}", serde_json::to_string(&c).expect("yes"));
            assert_eq!(&c.name, NAME1);
        }
    }

    #[test]
    fn create_multiple_characters() {
        let mut setup = setup(TestSystem::MemorySphere);
        create_char_with_name(&mut setup, NAME1);
        create_char_with_name(&mut setup, NAME2);
        create_char_with_name(&mut setup, NAME3);
        let char_conns = setup.loaded_dbs.character_connections();
        assert_eq!(char_conns.len(), 3);
    }

    #[test]
    fn create_multiple_characters_same_name() {
        let mut setup = setup(TestSystem::MemorySphere);
        create_char_with_name(&mut setup, NAME1);
        create_char_with_name(&mut setup, NAME1);
        create_char_with_name(&mut setup, NAME1);
        let char_conns = setup.loaded_dbs.character_connections();
        assert_eq!(char_conns.len(), 3);
    }
}
