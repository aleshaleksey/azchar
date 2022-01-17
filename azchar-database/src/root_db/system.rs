//! This deals with tables on the root database that deal with
//! permitted character parts and permitted attributes.
use crate::shared::*;
use azchar_error::ma;

use crate::character::Character;
use rusqlite::Connection as SqliteConnection;
use rusqlite::Error as RSqlError;
use rusqlite::Row as RSqlRow;

use std::default::Default;

// table!(
//     permitted_attributes(key) {
//         key -> Text,
//         attribute_type -> Integer,
//         attribute_description -> Text,
//         part_name -> Text,
//         part_type -> Integer,
//         obligatory -> Bool,
//     }
// );
//
// table!(
//     permitted_parts(id) {
//         id -> BigInt,
//         part_name -> Text,
//         part_type -> Integer,
//         obligatory -> Bool,
//     }
// );

/// This represents a part that is permitted and that will be created on a new sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct PermittedPart {
    pub(crate) id: Option<i64>,
    pub(crate) part_name: String,
    pub(crate) part_type: Part,
    pub(crate) obligatory: bool,
}

/// This represents a permitted attribute, to be created on a new sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct PermittedAttribute {
    pub(crate) key: String,
    pub(crate) attribute_type: i32,
    pub(crate) attribute_description: String,
    pub(crate) part_name: String,
    pub(crate) part_type: Part,
    pub(crate) obligatory: bool,
}

impl From<&PermittedPart> for Character {
    fn from(pp: &PermittedPart) -> Self {
        use uuid_rs::v4;

        let mut new = Character::default();
        new.uuid = v4!();
        new.character_type = pp.part_name.to_string();
        new.part_type = pp.part_type;
        new
    }
}

impl PermittedPart {
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    fn from_row(row: &RSqlRow) -> Result<Self, RSqlError> {
        let r = PermittedPart {
            id: row.get(0)?,
            part_name: row.get(1)?,
            part_type: row.get(2)?,
            obligatory: row.get(3)?,
        };
        Ok(r)
    }
    /// Get all permitted parts.
    pub fn load_all(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        root_conn
            .prepare_cached("SELECT * from permitted_parts;")
            .map_err(ma)?
            .query_map([], |row| PermittedPart::from_row(row))
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }
    /// Get all permitted parts.
    pub fn load_obligatory(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        root_conn
            .prepare_cached("SELECT * from permitted_parts WHERE obligatory=true;")
            .map_err(ma)?
            .query_map([], |row| PermittedPart::from_row(row))
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }

    pub fn insert_single(&self, conn: &SqliteConnection) -> Result<usize, String> {
        conn.prepare_cached(
            "INSERT INTO permitted_parts(part_name, part_type, obligatory) VALUES (?,?,?);",
        )
        .map_err(ma)?
        .execute(params![self.part_name, self.part_type, self.obligatory,])
        .map_err(ma)
    }
}

impl PermittedAttribute {
    fn from_row(row: &RSqlRow) -> Result<Self, RSqlError> {
        let r = PermittedAttribute {
            key: row.get(0)?,
            attribute_type: row.get(1)?,
            attribute_description: row.get(2)?,
            part_name: row.get(3)?,
            part_type: row.get(4)?,
            obligatory: row.get(5)?,
        };
        Ok(r)
    }

    pub fn insert_single(&self, conn: &SqliteConnection) -> Result<usize, String> {
        conn
            .prepare_cached("INSERT INTO permitted_attributes(key, attribute_type, attribute_description, part_name, part_type, obligatory) VALUES (?,?,?,?,?,?);")
            .map_err(ma)?
            .execute(params![
                self.key,
                self.attribute_type,
                self.attribute_description,
                self.part_name,
                self.part_type,
                self.obligatory,
                ])
            .map_err(ma)
    }
    // Load permitted attributes for the part from the root database.
    pub(crate) fn load_for_part(
        part: &PermittedPart,
        root_conn: &SqliteConnection,
    ) -> Result<Vec<Self>, String> {
        root_conn
            .prepare_cached("SELECT * from permitted_attributes WHERE part_name=:n AND part_type=:t ORDER BY part_type ASC;")
            .map_err(ma)?
            .query_map(
                &[(":n", &part.part_name), (":t", &(part.part_type as i64).to_string())],
                |row| PermittedAttribute::from_row(row)
            )
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }

    // Load permitted attributes for the part from the root database.
    pub(crate) fn load_obligatory_for_part(
        part: &PermittedPart,
        root_conn: &SqliteConnection,
    ) -> Result<Vec<Self>, String> {
        root_conn
            .prepare_cached("SELECT * from permitted_attributes WHERE part_name=:n AND part_type=:t AND obligatory=true ORDER BY part_type ASC;")
            .map_err(ma)?
            .query_map(
                &[(":n", &part.part_name), (":t", &(part.part_type as i64).to_string())],
                |row| PermittedAttribute::from_row(row)
            )
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }

    pub(crate) fn load_all(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        root_conn
            .prepare_cached("SELECT * from permitted_attributes;")
            .map_err(ma)?
            .query_map([], |row| PermittedAttribute::from_row(row))
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }

    pub(crate) fn load_all_obligatory(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        root_conn
            .prepare_cached(
                "SELECT * from permitted_attributes WHERE obligatory=true ORDER BY part_type ASC;",
            )
            .map_err(ma)?
            .query_map([], |row| PermittedAttribute::from_row(row))
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }
}

#[cfg(test)]
mod system_tests {
    use super::{PermittedAttribute, PermittedPart};
    use crate::root_db::tests;
    use crate::shared::Part;

    use rusqlite::Connection as SqliteConnection;

    /// The basic setup used.
    fn get_all_parts(setup: &mut tests::TestSetup) -> (&SqliteConnection, Vec<PermittedPart>) {
        let root_db = setup
            .loaded_dbs
            .get_inner_root()
            .expect("Could not get inner root.");
        (
            root_db,
            PermittedPart::load_all(root_db).expect("couldn't get permitted parts."),
        )
    }

    #[test]
    fn test_system_setup() {
        tests::setup(tests::TestSystem::MemorySphere);
    }

    #[test]
    fn test_system_setup_5e() {
        tests::setup(tests::TestSystem::DnD5);
    }

    #[test]
    fn load_all_permitted_parts() {
        let mut setup = tests::setup(tests::TestSystem::MemorySphere);
        let (_, parts) = get_all_parts(&mut setup);
        assert_eq!(parts.len(), 2);
        assert_eq!(
            parts,
            vec![
                PermittedPart {
                    id: Some(1),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedPart {
                    id: Some(2),
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
            ]
        );
    }

    #[test]
    fn load_all_permitted_parts_dnd5e() {
        let mut setup = tests::setup(tests::TestSystem::DnD5);
        let (_, parts) = get_all_parts(&mut setup);
        assert_eq!(parts.len(), 10);
        assert_eq!(
            parts[0],
            PermittedPart {
                id: Some(1),
                part_name: String::from("main"),
                part_type: Part::Main,
                obligatory: true,
            },
        );
        assert_eq!(
            parts[1],
            PermittedPart {
                id: Some(2),
                part_name: String::from("spell"),
                part_type: Part::Ability,
                obligatory: false,
            },
        );
    }

    #[test]
    fn load_permitted_attributes_for_main() {
        let mut setup = tests::setup(tests::TestSystem::MemorySphere);
        let (root_db, parts) = get_all_parts(&mut setup);
        let main_attributes = PermittedAttribute::load_for_part(&parts[0], root_db)
            .expect("Could not get attributes for `main`.");
        assert_eq!(main_attributes.len(), 3);
        assert_eq!(
            main_attributes,
            vec![
                PermittedAttribute {
                    key: String::from("race"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's race."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("class"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's class."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("character_alignment"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's alignment."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
            ]
        );
    }

    #[test]
    fn load_permitted_attributes_for_main_dnd5e() {
        let mut setup = tests::setup(tests::TestSystem::DnD5);
        let (root_db, parts) = get_all_parts(&mut setup);
        let main_attributes = PermittedAttribute::load_for_part(&parts[0], root_db)
            .expect("Could not get attributes for `main`.");
        assert_eq!(main_attributes.len(), 14);
        assert_eq!(
            &main_attributes[11..14],
            &[
                PermittedAttribute {
                    key: String::from("char"),
                    attribute_type: 0,
                    attribute_description: String::from(
                        "Persuading someone to go to bed with you."
                    ),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("ac"),
                    attribute_type: 0,
                    attribute_description: String::from("But can you hit me?"),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("level"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's race."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
            ]
        );
    }

    #[test]
    fn load_permitted_attributes_for_sphere() {
        let mut setup = tests::setup(tests::TestSystem::MemorySphere);
        let (root_db, parts) = get_all_parts(&mut setup);
        let sphere_attributes = PermittedAttribute::load_for_part(&parts[1], root_db)
            .expect("Could not get attributes for `Memory Sphere`.");
        assert_eq!(sphere_attributes.len(), 4);
        assert_eq!(
            sphere_attributes,
            vec![PermittedAttribute {
                key: String::from("mana_type"),
                attribute_type: 0,
                attribute_description: String::from("The type of mana that the memory sphere consumes."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
                obligatory: true,
            },
            PermittedAttribute {
                key: String::from("mana_consumption"),
                attribute_type: 0,
                attribute_description: String::from("The amount of mana the memory sphere consumes per recollection."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
                obligatory: true,
            },
            PermittedAttribute {
                key: String::from("memory_capacity"),
                attribute_type: 0,
                attribute_description: String::from("The number of memories that the memory sphere can hold before it breaks."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
                obligatory: true,
            },
            PermittedAttribute {
                key: String::from("memory_sphere_alignment"),
                attribute_type: 0,
                attribute_description: String::from("The alignment of the memory sphere determines the kind of memories it prefers."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
                obligatory: true,
            },]
        );
    }
}
