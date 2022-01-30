//! This deals with the base connections for the root db and outer dbs.
use azchar_error::ma;

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
    use crate::character::attribute::Attribute;
    use crate::character::character::CompleteCharacter;
    use crate::root_db::tests::*;
    use crate::root_db::{Character, CharacterPart, InputCharacter, NewAttribute};
    use crate::BasicConnection;

    const NAME1: &str = "Test Character";
    const NAME2: &str = "Test Character 2";
    const NAME3: &str = "Test Character 3";

    pub(crate) fn create_char_with_name<'a>(
        setup: &'a mut TestSetup,
        name: &str,
    ) -> (String, String) {
        setup
            .loaded_dbs
            .create_sheet(name)
            .expect("Could not create a character.")
    }

    pub(crate) fn create_char_with_name_and_connect<'a>(
        setup: &'a mut TestSetup,
        name: &str,
    ) -> &'a BasicConnection {
        let (name, uuid) = create_char_with_name(setup, name);
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
        let conn = create_char_with_name_and_connect(&mut setup, NAME1);
        if let Some(ref conn) = conn.connection {
            let c = CompleteCharacter::load(conn).expect("I'm here.");
            println!("char:{:?}", c);
            println!("char:{:?}", serde_json::to_string(&c).expect("yes"));
            assert_eq!(&c.name, NAME1);
        } else {
            panic!("Connection should exist.");
        }
    }

    #[test]
    fn create_part() {
        use crate::character::character::characters::dsl::*;
        use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

        let mut setup = setup(TestSystem::MemorySphere);
        let params = create_char_with_name(&mut setup, NAME1);
        let c = {
            let inner_conn = get_inner_conn(&setup, &params);
            CompleteCharacter::load(&inner_conn).expect("I'm here.")
        };
        let new_part = InputCharacter::test();
        let key = (c.name.to_owned(), c.uuid().to_owned());
        {
            setup
                .loaded_dbs
                .create_part(new_part.clone(), key)
                .expect("We can create part.");
        }
        {
            let inner_conn = get_inner_conn(&setup, &params);
            let c2 = CompleteCharacter::load(&inner_conn).expect("I'm here.");
            //
            assert_eq!(&c.name, NAME1);
            assert_eq!(&c2.name, NAME1);
            assert_ne!(c, c2);
            //
            let new_loaded: Character = characters
                .filter(name.eq(&new_part.name()))
                .first(inner_conn)
                .expect("Could not load created part.");

            assert_eq!(new_loaded.name(), new_part.name());
            assert_eq!(new_loaded.character_type(), new_part.character_type());
            assert_eq!(new_loaded.speed(), new_part.speed());
            assert_eq!(new_loaded.weight(), new_part.weight());
            assert_eq!(new_loaded.size(), new_part.size());
            assert_eq!(new_loaded.hp_total(), new_part.hp_total());
            assert_eq!(new_loaded.hp_current(), new_part.hp_current());
            assert_eq!(new_loaded.belongs_to(), new_part.belongs_to());
            assert_eq!(new_loaded.part_type(), new_part.part_type());
        }
    }

    #[test]
    fn update_part() {
        use crate::character::character::characters::dsl::*;
        use crate::diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

        let mut setup = setup(TestSystem::MemorySphere);
        let params = create_char_with_name(&mut setup, NAME1);
        let c = {
            let inner_conn = get_inner_conn(&setup, &params);
            CompleteCharacter::load(&inner_conn).expect("I'm here.")
        };
        let new_part = InputCharacter::test();
        let key = (c.name.to_owned(), c.uuid().to_owned());
        {
            setup
                .loaded_dbs
                .create_part(new_part.clone(), key.to_owned())
                .expect("We can create part.");
        }
        let part_to_update: CharacterPart = {
            let inner_conn = get_inner_conn(&setup, &params);
            let part_to_update: Character = characters
                .filter(name.eq(&new_part.name()))
                .first(inner_conn)
                .expect("Could not load created part.");
            let mut part_to_update = CharacterPart::from_db_character(part_to_update);
            part_to_update.speed = 9999;
            part_to_update.hp_total = Some(1111);
            part_to_update.name = "Still the Same part".to_owned();
            part_to_update
        };
        {
            setup
                .loaded_dbs
                .create_update_part(part_to_update.clone(), key.to_owned())
                .expect("Can update.");
        }
        {
            let inner_conn = get_inner_conn(&setup, &params);
            let c2 = CompleteCharacter::load(&inner_conn).expect("I'm here.");
            //
            assert_eq!(&c.name, NAME1);
            assert_eq!(&c2.name, NAME1);
            assert_ne!(c, c2);
            //
            let old_loaded: Option<Character> = characters
                .filter(name.eq(&new_part.name()))
                .first(inner_conn)
                .optional()
                .expect("Could not load created part.");
            assert!(old_loaded.is_none());

            let new_loaded: Character = characters
                .filter(name.eq(&part_to_update.name))
                .first(inner_conn)
                .expect("Could not load created part.");
            assert!(old_loaded.is_none());

            assert_eq!(new_loaded.name(), &part_to_update.name);
            assert_eq!(new_loaded.character_type(), &part_to_update.character_type);
            assert_eq!(new_loaded.speed(), part_to_update.speed);
            assert_eq!(new_loaded.weight(), part_to_update.weight);
            assert_eq!(new_loaded.size(), &part_to_update.size);
            assert_eq!(new_loaded.hp_total(), &part_to_update.hp_total);
            assert_eq!(new_loaded.hp_current(), &part_to_update.hp_current);
            assert_eq!(new_loaded.belongs_to(), &part_to_update.belongs_to);
            assert_eq!(new_loaded.part_type(), part_to_update.part_type);
        }
    }

    #[test]
    fn create_attribute() {
        use crate::character::attribute::attributes::dsl::*;
        use crate::diesel::BoolExpressionMethods;
        use crate::diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

        let mut setup = setup(TestSystem::MemorySphere);
        let params = create_char_with_name(&mut setup, NAME1);
        let c = {
            let conn = setup
                .loaded_dbs
                .character_connections()
                .get(&params)
                .expect("The character exists. We inserted it.");
            let inner_conn = conn.connection.as_ref().expect("yes.");
            CompleteCharacter::load(&inner_conn).expect("I'm here.")
        };
        let char_key = (c.name.to_owned(), c.uuid().to_owned());
        let new_attribute = NewAttribute::test();
        {
            let inner_conn = get_inner_conn(&setup, &params);
            let non_existing: Option<Attribute> = attributes
                .filter(key.eq(&new_attribute.key).and(of.eq(new_attribute.of)))
                .first(inner_conn)
                .optional()
                .expect("Can load.");
            assert!(non_existing.is_none());

            setup
                .loaded_dbs
                .create_attribute(new_attribute.clone(), char_key)
                .expect("We can create part.");
        }
        {
            let inner_conn = get_inner_conn(&setup, &params);

            let existing: Attribute = attributes
                .filter(key.eq(&new_attribute.key).and(of.eq(new_attribute.of)))
                .first(inner_conn)
                .expect("Can load.");
            assert_eq!(existing.of, new_attribute.of);
            assert_eq!(existing.key, new_attribute.key);
            assert_eq!(existing.value_num, new_attribute.value_num);
            assert_eq!(existing.value_text, new_attribute.value_text);
            assert_eq!(existing.description, new_attribute.description);
        }
    }

    #[test]
    fn update_attribute() {
        use crate::character::attribute::attributes::dsl::*;
        use crate::diesel::BoolExpressionMethods;
        use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

        let mut setup = setup(TestSystem::MemorySphere);
        let params = create_char_with_name(&mut setup, NAME1);
        let c = {
            let conn = setup
                .loaded_dbs
                .character_connections()
                .get(&params)
                .expect("The character exists. We inserted it.");
            let inner_conn = conn.connection.as_ref().expect("yes.");
            CompleteCharacter::load(&inner_conn).expect("I'm here.")
        };
        let char_key = (c.name.to_owned(), c.uuid().to_owned());
        let new_attribute = NewAttribute::test();
        setup
            .loaded_dbs
            .create_attribute(new_attribute.clone(), char_key.clone())
            .expect("We can create part.");

        let mut attribute: Attribute = {
            let inner_conn = get_inner_conn(&setup, &params);
            attributes
                .filter(key.eq(&new_attribute.key).and(of.eq(new_attribute.of)))
                .first(inner_conn)
                .expect("Can load.")
        };

        attribute.value_num = Some(65);
        attribute.value_text = Some("Can store 65 minor memories".to_owned());
        attribute.description = Some("Each memory sphere has its limit.".to_owned());
        let (k, v) = attribute.clone().into_key_value();
        {
            setup
                .loaded_dbs
                .create_update_attribute(k, v, char_key)
                .expect("Yes we can!");
        }
        {
            let inner_conn = get_inner_conn(&setup, &params);

            let existing: Attribute = attributes
                .filter(key.eq(&new_attribute.key).and(of.eq(new_attribute.of)))
                .first(inner_conn)
                .expect("Can load.");
            assert_eq!(existing.of, attribute.of);
            assert_eq!(existing.key, attribute.key);
            assert_eq!(existing.value_num, attribute.value_num);
            assert_eq!(existing.value_text, attribute.value_text);
            assert_eq!(existing.description, attribute.description);
        }
    }

    #[test]
    fn create_and_load_character_dnd5() {
        let mut setup = setup(TestSystem::DnD5);
        let conn = create_char_with_name_and_connect(&mut setup, NAME1);
        if let Some(ref conn) = conn.connection {
            let c = CompleteCharacter::load(conn).expect("I'm here.");
            println!("char:{:?}", c);
            println!("char:{:?}", serde_json::to_string(&c).expect("yes"));
            // assert_eq!(&c.name, "");
            assert_eq!(&c.name, NAME1);
        } else {
            panic!("Connection should exist.");
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
