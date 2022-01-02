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
        let mut new = NewCharacter::default();

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
    ) -> Result<(), String> {
        use crate::database::character::attribute::attributes::dsl as at_dsl;
        use crate::database::character::character::characters::dsl as ch_dsl;

        for part in PermittedPart::load_all(root_conn)?.into_iter() {
            // Create and insert an empty part and then create obligatory attributes.
            let mut new_part: NewCharacter = (&part).into();
            // TODO Belonging properly.
            if !matches!(part.part_type, Part::Main) {
                new_part.belongs_to = Some(
                    ch_dsl::characters
                        .filter(ch_dsl::part_type.eq(Part::Main))
                        .select(ch_dsl::id)
                        .first(sheet_conn)
                        .map_err(ma)?,
                );
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
