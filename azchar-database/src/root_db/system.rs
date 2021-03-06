//! This deals with tables on the root database that deal with
//! permitted character parts and permitted attributes.
use crate::shared::*;
use azchar_error::ma;

use crate::character::NewCharacter;

use diesel::backend::Backend;
use diesel::types::FromSqlRow;
use diesel::{BoolExpressionMethods, ExpressionMethods};
use diesel::{QueryDsl, Queryable, RunQueryDsl, SqliteConnection};

use std::default::Default;

table!(
    permitted_attributes(key) {
        key -> Text,
        attribute_type -> Integer,
        attribute_description -> Text,
        part_name -> Nullable<Text>,
        part_type -> Nullable<Integer>,
        obligatory -> Bool,
    }
);

table!(
    permitted_parts(id) {
        id -> BigInt,
        part_name -> Text,
        part_type -> Integer,
        obligatory -> Bool,
    }
);

/// This represents a part that is permitted and that will be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Identifiable, Queryable)]
#[table_name = "permitted_parts"]
pub struct PermittedPart {
    id: i64,
    pub(crate) part_name: String,
    #[diesel(deserialize_as = "i32")]
    pub(crate) part_type: Part,
    pub(crate) obligatory: bool,
}

/// This represents a permitted attribute, to be created on a new sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct PermittedAttribute {
    pub(crate) key: String,
    pub(crate) attribute_type: i32,
    pub(crate) attribute_description: String,
    pub(crate) part_name: Option<String>,
    pub(crate) part_type: Option<Part>,
    pub(crate) obligatory: bool,
}

impl<DB, ST> Queryable<ST, DB> for PermittedAttribute
where
    DB: Backend,
    (String, i32, String, Option<String>, Option<i32>, bool): FromSqlRow<ST, DB>,
{
    type Row = (String, i32, String, Option<String>, Option<i32>, bool);

    fn build(row: Self::Row) -> Self {
        PermittedAttribute {
            key: row.0,
            attribute_type: row.1,
            attribute_description: row.2,
            part_name: row.3,
            part_type: row.4.map(Into::into),
            obligatory: row.5,
        }
    }
}

impl From<&PermittedPart> for NewCharacter {
    fn from(pp: &PermittedPart) -> Self {
        use uuid_rs::v4;

        let mut new = NewCharacter::default();
        new.uuid = v4!();
        new.character_type = pp.part_name.to_string();
        new.part_type = pp.part_type;
        new
    }
}

impl PermittedPart {
    pub fn id(&self) -> i64 {
        self.id
    }
    /// Get all permitted parts.
    pub fn load_all(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        use self::permitted_parts::dsl::*;
        permitted_parts.load(root_conn).map_err(ma)
    }
    /// Get all permitted parts.
    pub fn load_obligatory(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        use self::permitted_parts::dsl::*;
        permitted_parts
            .filter(obligatory.eq(true))
            .load(root_conn)
            .map_err(ma)
    }
}

impl PermittedAttribute {
    /// A DRY function to check if a part is permitted for a character.
    pub(crate) fn permitted_for_part(&self, p_type: Part, c_type: &str) -> bool {
        self.part_type.map(|x| x == p_type).unwrap_or(true)
            && self.part_name.as_ref().map(|x| x == c_type).unwrap_or(true)
    }
    /// A DRY function to check if a part is permitted for a character.
    pub(crate) fn obligatory_for_part(&self, p_type: Part, c_type: &str) -> bool {
        self.obligatory && self.permitted_for_part(p_type, c_type)
    }

    /// Load permitted attributes for the part from the root database.
    /// Permitted attributes are:
    ///
    /// a) Attributes which are general (no part name or part type)
    ///
    /// b) type specific attributes (no part name, part type equals)
    ///
    /// c) Specific attribtues (both p-name and p-type are equal)
    pub(crate) fn load_for_part(
        part: &PermittedPart,
        root_conn: &SqliteConnection,
    ) -> Result<Vec<Self>, String> {
        use self::permitted_attributes::dsl::*;
        let filter = part_type
            .is_null()
            .or(part_type.eq(part.part_type).and(part_name.is_null()))
            .or(part_type
                .eq(part.part_type)
                .and(part_name.eq(&part.part_name)));
        permitted_attributes
            .filter(filter)
            .order_by(part_type.asc())
            .load(root_conn)
            .map_err(ma)
    }
    // Load permitted attributes for the part from the root database.
    pub(crate) fn load_obligatory_for_part(
        part: &PermittedPart,
        root_conn: &SqliteConnection,
    ) -> Result<Vec<Self>, String> {
        use self::permitted_attributes::dsl::*;
        let filter = part_type
            .is_null()
            .or(part_type.eq(part.part_type).and(part_name.is_null()))
            .or(part_type
                .eq(part.part_type)
                .and(part_name.eq(&part.part_name)));
        permitted_attributes
            .filter(filter.and(obligatory.eq(true)))
            .order_by(part_type.asc())
            .load(root_conn)
            .map_err(ma)
    }

    pub(crate) fn load_all(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        use self::permitted_attributes::dsl::*;
        permitted_attributes
            .order_by(part_type.asc())
            .load(root_conn)
            .map_err(ma)
    }

    pub(crate) fn load_all_obligatory(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        use self::permitted_attributes::dsl::*;
        permitted_attributes
            .filter(obligatory.eq(true))
            .order_by(part_type.asc())
            .load(root_conn)
            .map_err(ma)
    }
}

/// This represents a part that is permitted and that will be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Insertable)]
#[table_name = "permitted_parts"]
pub(crate) struct NewPermittedPart {
    pub(crate) part_name: String,
    pub(crate) part_type: Part,
    pub(crate) obligatory: bool,
}

/// This represents a permitted attribute, to be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Insertable)]
#[table_name = "permitted_attributes"]
pub(crate) struct NewPermittedAttribute {
    pub(crate) key: String,
    pub(crate) attribute_type: i32,
    pub(crate) attribute_description: String,
    pub(crate) part_name: Option<String>,
    pub(crate) part_type: Option<Part>,
    pub(crate) obligatory: bool,
}

#[cfg(test)]
mod system_tests {
    use super::{PermittedAttribute, PermittedPart};
    use crate::root_db::tests;
    use crate::shared::Part;

    use diesel::SqliteConnection;

    /// The basic setup used.
    pub(crate) fn get_all_parts(
        setup: &mut tests::TestSetup,
    ) -> (&SqliteConnection, Vec<PermittedPart>) {
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
        assert_eq!(parts.len(), 3);
        assert_eq!(
            parts,
            vec![
                PermittedPart {
                    id: 1,
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedPart {
                    id: 2,
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
                PermittedPart {
                    id: 3,
                    part_name: String::from("spell"),
                    part_type: Part::Ability,
                    obligatory: false,
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
                id: 1,
                part_name: String::from("main"),
                part_type: Part::Main,
                obligatory: true,
            },
        );
        assert_eq!(
            parts[1],
            PermittedPart {
                id: 2,
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
                    part_name: Some(String::from("main")),
                    part_type: Some(Part::Main),
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("class"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's class."),
                    part_name: Some(String::from("main")),
                    part_type: Some(Part::Main),
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("character_alignment"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's alignment."),
                    part_name: Some(String::from("main")),
                    part_type: Some(Part::Main),
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
                    key: String::from("cha"),
                    attribute_type: 0,
                    attribute_description: String::from(
                        "Persuading someone to go to bed with you."
                    ),
                    part_name: Some(String::from("main")),
                    part_type: Some(Part::Main),
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("ac"),
                    attribute_type: 0,
                    attribute_description: String::from("But can you hit me?"),
                    part_name: Some(String::from("main")),
                    part_type: Some(Part::Main),
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("level"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's race."),
                    part_name: Some(String::from("main")),
                    part_type: Some(Part::Main),
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
                part_name: Some(String::from("Memory Sphere")),
                part_type: Some(Part::InventoryItem),
                obligatory: true,
            },
            PermittedAttribute {
                key: String::from("mana_consumption"),
                attribute_type: 0,
                attribute_description: String::from("The amount of mana the memory sphere consumes per recollection."),
                part_name: Some(String::from("Memory Sphere")),
                part_type: Some(Part::InventoryItem),
                obligatory: true,
            },
            PermittedAttribute {
                key: String::from("memory_capacity"),
                attribute_type: 0,
                attribute_description: String::from("The number of memories that the memory sphere can hold before it breaks."),
                part_name: Some(String::from("Memory Sphere")),
                part_type: Some(Part::InventoryItem),
                obligatory: true,
            },
            PermittedAttribute {
                key: String::from("memory_sphere_alignment"),
                attribute_type: 0,
                attribute_description: String::from("The alignment of the memory sphere determines the kind of memories it prefers."),
                part_name: Some(String::from("Memory Sphere")),
                part_type: Some(Part::InventoryItem),
                obligatory: true,
            },]
        );
    }
}
