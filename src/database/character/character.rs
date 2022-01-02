//! This deals with the character columns.
use crate::database::shared::Part;
use crate::error::ma;

use diesel::OptionalExtension;
use diesel::{Connection, SqliteConnection};
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};

use super::attribute::Attributes;

table! {
    characters(id) {
        // Obligatory
        id -> BigInt,
        name -> Text,
        uuid -> Text,
        character_type -> Text,
        // Almost obligatory Fields.
        speed -> Integer,
        weight -> Nullable<Integer>,
        size -> Nullable<Text>,
        hp_total -> Nullable<Integer>,
        hp_current -> Nullable<Integer>,
        // References.
        belongs_to -> Nullable<BigInt>,
        part_type -> Integer,
    }
}

/// A structure to store a db ref.
#[derive(Debug, Clone, PartialEq, Identifiable, Queryable)]
#[table_name = "characters"]
pub struct Character {
    id: i64,
    name: String,
    uuid: String,
    character_type: String,
    speed: i32,
    weight: Option<i32>,
    size: Option<String>,
    hp_total: Option<i32>,
    hp_current: Option<i32>,
    belongs_to: Option<i64>,
    #[diesel(deserialize_as = "i32")]
    part_type: Part,
}

#[derive(Debug, Clone, Insertable, Default)]
#[table_name = "characters"]
pub struct NewCharacter {
    pub(crate) name: String,
    pub(crate) uuid: String,
    pub(crate) character_type: String,
    pub(crate) speed: i32,
    pub(crate) weight: Option<i32>,
    pub(crate) size: Option<String>,
    pub(crate) hp_total: Option<i32>,
    pub(crate) hp_current: Option<i32>,
    pub(crate) belongs_to: Option<i64>,
    pub(crate) part_type: Part,
}

impl NewCharacter {
    fn from_part(part: &CharacterPart) -> Self {
        NewCharacter {
            name: part.name.clone(),
            uuid: part.uuid.clone(),
            character_type: part.character_type.clone(),
            speed: part.speed,
            weight: part.weight,
            size: part.size.clone(),
            hp_total: part.hp_total,
            hp_current: part.hp_current,
            belongs_to: part.belongs_to,
            part_type: part.part_type,
        }
    }

    fn from_complete(main: &CompleteCharacter) -> Self {
        NewCharacter {
            name: main.name.clone(),
            uuid: main.uuid.clone(),
            character_type: main.character_type.clone(),
            speed: main.speed,
            weight: main.weight,
            size: main.size.clone(),
            hp_total: main.hp_total,
            hp_current: main.hp_current,
            belongs_to: None,
            part_type: Part::Main,
        }
    }
}

/// exists to make working with CompleteCharacter simpler.
#[derive(Clone, Debug)]
pub struct CharacterPart {
    id: Option<i64>,
    pub(crate) name: String,
    uuid: String,
    pub(crate) character_type: String,
    pub(crate) speed: i32,
    pub(crate) weight: Option<i32>,
    pub(crate) size: Option<String>,
    pub(crate) hp_total: Option<i32>,
    pub(crate) hp_current: Option<i32>,
    part_type: Part,
    pub(crate) belongs_to: Option<i64>,
    pub(crate) attributes: super::attribute::Attributes,
}

/// This represents a complete character.
#[derive(Clone, Debug)]
pub struct CompleteCharacter {
    id: Option<i64>,
    pub(crate) name: String,
    uuid: String,
    pub(crate) character_type: String,
    pub(crate) speed: i32,
    pub(crate) weight: Option<i32>,
    pub(crate) size: Option<String>,
    pub(crate) hp_total: Option<i32>,
    pub(crate) hp_current: Option<i32>,
    pub(crate) parts: Vec<CharacterPart>,
    pub(crate) attributes: super::attribute::Attributes,
}

impl CompleteCharacter {
    pub fn load(conn: &SqliteConnection) -> Result<CompleteCharacter, String> {
        use self::characters::dsl::*;

        let core_char: Character = characters
            .filter(belongs_to.is_null())
            .first(conn)
            .map_err(ma)?;
        let bare_chars: Vec<Character> = characters
            .filter(belongs_to.is_not_null())
            .load(conn)
            .map_err(ma)?;

        let ids: Vec<_> = bare_chars.iter().map(|c| c.id).collect();
        let mut attributes = Attributes::get_vec_for_characters(&ids, conn)?;
        // This should speed up sorting by character.
        attributes.sort_by(|a, b| a.of.cmp(&b.of));
        let attrs = attributes.iter().filter(|a| a.of == core_char.id).cloned();
        let main_attributes = Attributes::from_vec(attrs.collect::<Vec<_>>());

        let subchars: Vec<CharacterPart> = bare_chars
            .into_iter()
            .map(|c| {
                let attrs = attributes.iter().filter(|a| a.of == c.id).cloned();
                let attributes = Attributes::from_vec(attrs.collect::<Vec<_>>());
                CharacterPart {
                    id: Some(c.id),
                    name: c.name,
                    uuid: c.uuid,
                    character_type: c.character_type,
                    speed: c.speed,
                    weight: c.weight,
                    size: c.size,
                    hp_total: c.hp_total,
                    hp_current: c.hp_current,
                    part_type: c.part_type,
                    belongs_to: Some(core_char.id),
                    attributes,
                }
            })
            .collect();

        Ok(CompleteCharacter {
            id: Some(core_char.id),
            name: core_char.name,
            uuid: core_char.uuid,
            character_type: core_char.character_type,
            speed: core_char.speed,
            weight: core_char.weight,
            size: core_char.size,
            hp_total: core_char.hp_total,
            hp_current: core_char.hp_current,
            parts: subchars,
            attributes: main_attributes,
        })
    }

    /// Store a character in an existing sheet.
    /// If the sheet is empty a new character is created, otherwise it is updated.
    /// NB: The sheet should already exist.
    pub fn save(&self, conn: &SqliteConnection) -> Result<(), String> {
        use self::characters::dsl::*;

        let existing: Option<Character> = characters
            .filter(part_type.eq(Part::Main))
            .first(conn)
            .optional()
            .map_err(ma)?;
        // If the current sheet is already occupied by a different character, return early.
        if let Some(other) = &existing {
            if self.id != Some(other.id) {
                return Err(format!(
                    "A character already exists on this sheet: name:{}, uuid:{}",
                    other.name, other.uuid,
                ));
            }
        }

        // Do the work.
        conn.transaction::<_, diesel::result::Error, _>(|| {
            // Insert or update attributes for main character.
            self.attributes.insert_update(conn)?;
            let mut new_chars = Vec::new();
            let mut uuids = vec![&self.uuid];
            // Insert or update main character.
            if let Some(own_id) = self.id {
                diesel::update(characters.filter(id.eq(own_id)))
                    .set((
                        name.eq(&self.name),
                        uuid.eq(&self.uuid),
                        speed.eq(self.speed),
                        character_type.eq(&self.character_type),
                        weight.eq(self.weight),
                        size.eq(&self.size),
                        hp_total.eq(self.hp_total),
                        hp_current.eq(self.hp_current),
                        part_type.eq(Part::Main),
                        belongs_to.eq::<Option<i64>>(None),
                    ))
                    .execute(conn)?;
            } else {
                new_chars.push(NewCharacter::from_complete(self));
            }

            // Insert or update sub-characters.
            for sub_char in self.parts.iter() {
                uuids.push(&sub_char.uuid);
                sub_char.attributes.insert_update(conn)?;
                if let Some(part_id) = sub_char.id {
                    diesel::update(characters.filter(id.eq(part_id)))
                        .set((
                            name.eq(&sub_char.name),
                            uuid.eq(&sub_char.uuid),
                            character_type.eq(&sub_char.character_type),
                            speed.eq(sub_char.speed),
                            weight.eq(sub_char.weight),
                            size.eq(&sub_char.size),
                            hp_total.eq(sub_char.hp_total),
                            hp_current.eq(sub_char.hp_current),
                            part_type.eq(sub_char.part_type),
                            belongs_to.eq(sub_char.belongs_to),
                        ))
                        .execute(conn)?;
                } else {
                    new_chars.push(NewCharacter::from_part(sub_char));
                }
            }
            // Perform the main insertion, but first delete anything that is not here.
            for chunk in uuids.chunks(999) {
                diesel::delete(characters.filter(uuid.ne_any(chunk))).execute(conn)?;
            }
            for chunk in new_chars.chunks(999) {
                diesel::insert_into(characters)
                    .values(chunk)
                    .execute(conn)?;
            }

            Ok(())
        })
        .map_err(ma)
    }
}
