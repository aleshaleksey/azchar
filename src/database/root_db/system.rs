//! This deals with tables on the root database that deal with
//! permitted character parts and permitted attributes.
use crate::database::shared::*;
use crate::error::ma;

use crate::database::character::NewAttribute;
use crate::database::character::NewCharacter;

use diesel::{BoolExpressionMethods, ExpressionMethods};
use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};

use std::default::Default;

table!(
    permitted_attributes(key) {
        key -> Text,
        attribute_type -> Integer,
        attribute_description -> Text,
        part_name -> Text,
        part_type -> Integer,
    }
);

table!(
    permitted_parts(id) {
        id -> BigInt,
        part_name -> Text,
        part_type -> Integer,
    }
);

/// This represents a part that is permitted and that will be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Identifiable, Queryable)]
#[table_name = "permitted_parts"]
pub(crate) struct PermittedPart {
    id: i64,
    part_name: String,
    #[diesel(deserialize_as = "i32")]
    part_type: Part,
}

/// This represents a permitted attribute, to be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Queryable)]
// #[table_name = "permitted_attributes"]
pub(crate) struct PermittedAttribute {
    key: String,
    attribute_type: i32,
    attribute_description: String,
    part_name: String,
    #[diesel(deserialize_as = "i32")]
    part_type: Part,
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
    /// Get all permitted parts.
    pub fn load_all(root_conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        use self::permitted_parts::dsl::*;
        permitted_parts.load(root_conn).map_err(ma)
    }

    // Create the basic attributes and parts for the sheet.
    pub fn create_basic(
        root_conn: &SqliteConnection,
        sheet_conn: &SqliteConnection,
        name: &str,
    ) -> Result<(), String> {
        use crate::database::character::attribute::attributes::dsl as at_dsl;
        use crate::database::character::character::characters::dsl as ch_dsl;

        for part in PermittedPart::load_all(root_conn)?.into_iter() {
            // Create and insert an empty part and then create obligatory attributes.
            let mut new_part: NewCharacter = (&part).into();
            println!("new_part:{:?}", new_part);
            // TODO Belonging properly.
            if !matches!(part.part_type, Part::Main) {
                new_part.belongs_to = Some(
                    ch_dsl::characters
                        .filter(ch_dsl::part_type.eq(Part::Main))
                        .select(ch_dsl::id)
                        .first(sheet_conn)
                        .map_err(ma)?,
                );
            } else {
                new_part.name = name.to_owned();
            };

            diesel::insert_into(ch_dsl::characters)
                .values(new_part)
                .execute(sheet_conn)
                .map_err(ma)?;
            let char_id: i64 = ch_dsl::characters
                .order_by(ch_dsl::id.desc())
                .select(ch_dsl::id)
                .first(sheet_conn)
                .map_err(ma)?;

            // create attributes for the given part.
            let attributes = PermittedAttribute::load_for_part(&part, root_conn)
                .map_err(ma)?
                .into_iter()
                .map(|at| NewAttribute {
                    key: at.key,
                    value_num: None,
                    value_text: None,
                    description: Some(at.attribute_description),
                    of: char_id,
                })
                .collect::<Vec<_>>();
            diesel::insert_into(at_dsl::attributes)
                .values(&attributes)
                .execute(sheet_conn)
                .map_err(ma)?;
        }
        Ok(())
    }
}

impl PermittedAttribute {
    // Load permitted attributes for the part from the root database.
    fn load_for_part(
        part: &PermittedPart,
        root_conn: &SqliteConnection,
    ) -> Result<Vec<Self>, String> {
        use self::permitted_attributes::dsl::*;
        permitted_attributes
            .filter(
                part_name
                    .eq(&part.part_name)
                    .and(part_type.eq(part.part_type)),
            )
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
}

/// This represents a permitted attribute, to be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Insertable)]
#[table_name = "permitted_attributes"]
pub(crate) struct NewPermittedAttribute {
    pub(crate) key: String,
    pub(crate) attribute_type: i32,
    pub(crate) attribute_description: String,
    pub(crate) part_name: String,
    pub(crate) part_type: Part,
}

#[cfg(test)]
mod system_tests {
    use super::{PermittedAttribute, PermittedPart};
    use crate::database::root_db::tests;
    use crate::database::shared::Part;

    use diesel::SqliteConnection;

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
        tests::setup();
    }

    #[test]
    fn load_all_permitted_parts() {
        let mut setup = tests::setup();
        let (_, parts) = get_all_parts(&mut setup);
        assert_eq!(parts.len(), 2);
        assert_eq!(
            parts,
            vec![
                PermittedPart {
                    id: 1,
                    part_name: String::from("main"),
                    part_type: Part::Main,
                },
                PermittedPart {
                    id: 2,
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                },
            ]
        );
    }

    #[test]
    fn load_permitted_attributes_for_main() {
        let mut setup = tests::setup();
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
                },
                PermittedAttribute {
                    key: String::from("class"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's class."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                },
                PermittedAttribute {
                    key: String::from("character_alignment"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's alignment."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                },
            ]
        );
    }

    #[test]
    fn load_permitted_attributes_for_sphere() {
        let mut setup = tests::setup();
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
            },
            PermittedAttribute {
                key: String::from("mana_consumption"),
                attribute_type: 0,
                attribute_description: String::from("The amount of mana the memory sphere consumes per recollection."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
            },
            PermittedAttribute {
                key: String::from("memory_capacity"),
                attribute_type: 0,
                attribute_description: String::from("The number of memories that the memory sphere can hold before it breaks."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
            },
            PermittedAttribute {
                key: String::from("memory_sphere_alignment"),
                attribute_type: 0,
                attribute_description: String::from("The alignment of the memory sphere determines the kind of memories it prefers."),
                part_name: String::from("Memory Sphere"),
                part_type: Part::InventoryItem,
            },]
        );
    }
}
