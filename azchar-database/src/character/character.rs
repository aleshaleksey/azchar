//! This deals with the character columns.
use crate::root_db::system::{PermittedAttribute, PermittedPart};
use crate::shared::Part;

use azchar_error::ma;

use diesel::result::Error as DbError;
use diesel::OptionalExtension;
use diesel::SqliteConnection;
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use fnv::{FnvHashMap, FnvHashSet};

use super::attribute::{Attribute, AttributeKey, AttributeValue, Attributes};

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
#[derive(Debug, Clone, PartialEq, Identifiable, Queryable, QueryableByName, Insertable)]
#[table_name = "characters"]
pub struct Character {
    pub(crate) id: i64,
    name: String,
    uuid: String,
    pub(crate) character_type: String,
    speed: i32,
    weight: Option<i32>,
    size: Option<String>,
    hp_total: Option<i32>,
    hp_current: Option<i32>,
    belongs_to: Option<i64>,
    #[diesel(deserialize_as = "i32")]
    pub(crate) part_type: Part,
}

impl Character {
    pub fn get_latest_id(conn: &SqliteConnection) -> Result<i64, DbError> {
        use self::characters::dsl::*;
        characters.order_by(id.desc()).select(id).first(conn)
    }

    fn from_part(part: &CharacterPart) -> Self {
        Character {
            id: part.id.unwrap(),
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
        Character {
            id: main.id.unwrap(),
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

    pub(crate) fn from_input(input: InputCharacter) -> Self {
        use uuid_rs::v4;
        NewCharacter {
            name: input.name,
            uuid: v4!(),
            character_type: input.character_type,
            speed: input.speed,
            weight: input.weight,
            size: input.size,
            hp_total: input.hp_total,
            hp_current: input.hp_current,
            belongs_to: input.belongs_to,
            part_type: input.part_type,
        }
    }

    pub(crate) fn checked_insert(
        self,
        conn: &SqliteConnection,
        permitted_parts: &[PermittedPart],
    ) -> Result<(), String> {
        use self::characters::dsl::*;
        // First check if this is allowed.
        if !permitted_parts
            .iter()
            .any(|p| p.part_name == self.character_type && p.part_type == self.part_type)
        {
            let m = format!(
                "Part {}-({:?},{}) not permitted in this system",
                self.name, self.part_type, self.character_type
            );
            return Err(m);
        }
        // Next check if it chains with the character.
        let parts: Vec<Character> = characters.load(conn).map_err(ma)?;
        if self.belongs_to.is_none()
            || (matches!(self.part_type, Part::Main)
                && parts.iter().any(|p| matches!(p.part_type, Part::Main)))
        {
            let m = format!(
                "Part {} has a \"Main\" typ, but one already exists on this sheet.",
                self.name
            );
            return Err(m);
        }
        if !parts.iter().any(|p| Some(p.id) == self.belongs_to) {
            let m = format!(
                "Part {}-({:?},{}) doesn't belong.",
                self.name, self.part_type, self.character_type
            );
            return Err(m);
        }
        diesel::insert_into(characters)
            .values(&self)
            .execute(conn)
            .map_err(ma)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// This is used purely for creating new parts.
pub struct InputCharacter {
    name: String,
    character_type: String,
    speed: i32,
    weight: Option<i32>,
    size: Option<String>,
    hp_total: Option<i32>,
    hp_current: Option<i32>,
    belongs_to: Option<i64>,
    part_type: Part,
}

/// exists to make working with CompleteCharacter simpler.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    pub(crate) attributes: Vec<(AttributeKey, AttributeValue)>,
}

impl CharacterPart {
    /// This is an incomplete comparison which does not take attributes into account.
    fn compare_part(&self, other: &Self) -> bool {
        self.id == other.id
            && self.part_type == other.part_type
            && self.speed == other.speed
            && self.weight == other.weight
            && self.hp_total == other.hp_total
            && self.hp_current == other.hp_current
            && self.belongs_to == other.belongs_to
            && self.character_type == other.character_type
            && self.size == other.size
            && self.name == other.name
            && self.uuid == other.uuid
    }
}

/// This represents a complete character.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    pub(crate) attributes: Vec<(AttributeKey, AttributeValue)>,
}

impl CompleteCharacter {
    pub(crate) fn id(&self) -> Option<i64> {
        self.id
    }

    pub(crate) fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Compare the main parts of two complete characters: NB: Attributes not compared.
    fn compare_main(&self, other: &CompleteCharacter) -> bool {
        self.id == other.id
            && self.speed == other.speed
            && self.weight == other.weight
            && self.hp_total == other.hp_current
            && self.hp_current == other.hp_current
            && self.name == other.name
            && self.uuid == other.uuid
            && self.character_type == other.character_type
            && self.size == other.size
    }

    pub fn load(conn: &SqliteConnection) -> Result<CompleteCharacter, String> {
        use self::characters::dsl::*;
        use super::attribute::attributes::dsl as attr_dsl;
        let then = std::time::Instant::now();

        let mut chars: Vec<Character> = characters.load(conn).map_err(ma)?;
        let attrs: Vec<Attribute> = attr_dsl::attributes.load(conn).map_err(ma)?;

        let a = then.elapsed().as_micros();
        let mut attrs = Attributes::key_val_vec(attrs);
        chars.sort_unstable_by(|a, b| b.id.cmp(&a.id));
        attrs.sort_unstable_by(|a, b| b.0.of.cmp(&a.0.of));

        if chars.is_empty() {
            return Err("No parts found for character.".to_string());
        }
        let core = chars.pop().expect("We have at least one.");

        let mut core_attrs = Vec::with_capacity(50);
        let mut next_vec = Vec::with_capacity(50);
        while next_vec.is_empty() {
            if let Some(x) = attrs.pop() {
                if x.0.of == core.id {
                    core_attrs.push(x);
                } else {
                    next_vec.push(x);
                }
            } else {
                break;
            }
        }

        let mut subs = Vec::with_capacity(chars.len());
        while let Some(c) = chars.pop() {
            let mut c_attrs = std::mem::replace(&mut next_vec, Vec::with_capacity(50));
            while next_vec.is_empty() {
                if let Some(x) = attrs.pop() {
                    if x.0.of == c.id {
                        c_attrs.push(x);
                    } else {
                        next_vec.push(x);
                    }
                } else {
                    break;
                }
            }
            subs.push(CharacterPart {
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
                belongs_to: Some(core.id),
                attributes: c_attrs,
            });
        }
        let b = then.elapsed().as_micros();
        println!("db-get:{}us, rust-sort: {}us", a, b - a);
        Ok(CompleteCharacter {
            id: Some(core.id),
            name: core.name,
            uuid: core.uuid,
            character_type: core.character_type,
            speed: core.speed,
            weight: core.weight,
            size: core.size,
            hp_total: core.hp_total,
            hp_current: core.hp_current,
            parts: subs,
            attributes: core_attrs,
        })
    }

    /// Store a character in an existing sheet.
    /// If the sheet is empty a new character is created, otherwise it is updated.
    /// NB: The sheet should already exist.
    /// NB2: We disallow characters lacking obligatory parts, or that have parts that are disallowed.
    pub fn save(
        &self,
        conn: &SqliteConnection,
        (permitted_attrs, permitted_parts): (&[PermittedAttribute], &[PermittedPart]),
    ) -> Result<(), String> {
        use self::characters::dsl::*;
        let then = std::time::Instant::now();
        let mut error_string = "DbError::NotFound".to_string();

        let res = conn.immediate_transaction::<_, DbError, _>(|| {
            // A check to see if the existing character already exists here.
            let existing: Option<(i64, String, String)> = characters
                .filter(part_type.eq(Part::Main))
                .select((id, name, uuid))
                .first(conn)
                .optional()?;
            let a = then.elapsed().as_micros();
            // If the current sheet is already occupied by a different character, return early.
            if let Some(other) = &existing {
                if self.id != Some(other.0) {
                    error_string = format!(
                        "A character already exists on this sheet: name:{}, uuid:{}",
                        other.1, other.2,
                    );
                    return Err(DbError::NotFound);
                }
            }

            let old_complete = CompleteCharacter::load(conn).map_err(|e| {
                error_string = format!("{:?}", e);
                DbError::NotFound
            })?;
            if &old_complete == self {
                let b = then.elapsed().as_micros();
                println!("same ret: {}", b);
                println!("same ret check: {}", b - a);
                return Ok(());
            }

            let permitted_parts = permitted_parts
                .iter()
                .map(|p| ((p.part_name.as_ref(), p.part_type), p.obligatory))
                .collect::<FnvHashMap<(&str, Part), bool>>();

            let permitted_attrs_map = permitted_attrs
                .iter()
                .map(|a| {
                    (
                        a.key.as_ref(),
                        (a.obligatory, a.part_type, a.part_name.as_ref()),
                    )
                })
                .collect::<FnvHashMap<&str, (bool, Part, &str)>>();
            let obligatory_attrs = permitted_attrs
                .iter()
                .filter(|a| a.obligatory)
                .collect::<Vec<_>>();

            // Do the work.
            if permitted_parts
                .get(&(self.character_type.as_ref(), Part::Main))
                .is_none()
            {
                error_string = "Main part type is not permitted in the system".to_owned();
                return Err(DbError::NotFound);
            }
            let mut own_oblig_part_count = 1;
            for p in self.parts.iter() {
                if let Some(val) = permitted_parts.get(&(p.character_type.as_ref(), p.part_type)) {
                    if *val {
                        own_oblig_part_count += 1;
                    }
                } else {
                    error_string = format!("Forbidden part found: '{}'", p.character_type);
                    return Err(DbError::NotFound);
                }
            }

            let opc = permitted_parts.into_iter().filter(|(_, ob)| *ob).count();
            if own_oblig_part_count < opc {
                error_string = "Obligatory part missing.".to_string();
                return Err(DbError::NotFound);
            }

            if let Err(e) = check_attributes_vs_db(
                &self.attributes,
                &permitted_attrs_map,
                (&self.character_type, Part::Main),
                &obligatory_attrs,
            ) {
                error_string = e;
                return Err(DbError::NotFound);
            };
            let mut attribute_refs: Vec<_> = Vec::with_capacity(1000); // Why not.
            attribute_refs.extend(self.attributes.iter().map(|(k, v)| (k, v)));
            let mut new_chars = Vec::new();
            let mut upd_chars = Vec::new();
            // Insert or update main character.
            if existing.is_none() {
                new_chars.push(NewCharacter::from_complete(self));
            } else if let Some(_own_id) = self.id {
                if !self.compare_main(&old_complete) {
                    upd_chars.push(Character::from_complete(self));
                }
            } else {
                new_chars.push(NewCharacter::from_complete(self));
            }

            // Insert or update sub-characters.
            for sub_char in self.parts.iter() {
                if let Err(e) = check_attributes_vs_db(
                    &sub_char.attributes,
                    &permitted_attrs_map,
                    (&sub_char.character_type, sub_char.part_type),
                    &obligatory_attrs,
                ) {
                    error_string = e;
                    return Err(DbError::NotFound);
                };
                attribute_refs.extend(sub_char.attributes.iter().map(|(k, v)| (k, v)));

                // A mystery wrapped in an enigma wrapped in an onion.
                if let Some(_part_id) = sub_char.id {
                    // A quick to check if we need to update. Maybe inefficient
                    let p = old_complete.parts.iter().find(|p| p.id == sub_char.id);
                    if let Some(part) = p {
                        if !part.compare_part(sub_char) {
                            upd_chars.push(Character::from_complete(self));
                        }
                    } else {
                        new_chars.push(NewCharacter::from_part(sub_char));
                    }
                } else {
                    new_chars.push(NewCharacter::from_part(sub_char));
                }
            }

            for chunk in new_chars.chunks(999) {
                diesel::insert_into(characters)
                    .values(chunk)
                    .execute(conn)?;
            }
            for chunk in new_chars.chunks(999) {
                diesel::replace_into(characters)
                    .values(chunk)
                    .execute(conn)?;
            }

            Attributes::insert_update_vec(attribute_refs.into_iter(), conn)?;
            Ok(())
        });
        let d = then.elapsed().as_micros();
        println!("transaction: {}", d);

        match res {
            Ok(r) => Ok(r),
            Err(DbError::NotFound) => Err(error_string),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Insert a single key value.
    pub(crate) fn insert_update_character_part(
        chp: CharacterPart,
        conn: &SqliteConnection,
    ) -> Result<(), String> {
        use self::characters::dsl::*;
        if let Some(ch_id) = chp.id {
            diesel::update(characters.filter(id.eq(ch_id)))
                .set((
                    name.eq(&chp.name),
                    uuid.eq(&chp.uuid),
                    character_type.eq(&chp.character_type),
                    speed.eq(chp.speed),
                    weight.eq(chp.weight),
                    size.eq(&chp.size),
                    hp_total.eq(chp.hp_total),
                    hp_current.eq(chp.hp_current),
                    part_type.eq(chp.part_type),
                    belongs_to.eq(chp.belongs_to),
                ))
                .execute(conn)
        } else {
            let new_part = NewCharacter::from_part(&chp);
            diesel::insert_into(characters)
                .values(&new_part)
                .execute(conn)
        }
        .map(|_| ())
        .map_err(ma)
    }
}

fn check_attributes_vs_db(
    own_attributes: &[(AttributeKey, AttributeValue)],
    permitted: &FnvHashMap<&str, (bool, Part, &str)>,
    (part_name, part_type): (&str, Part),
    obligatory: &[&PermittedAttribute],
) -> Result<(), String> {
    // First attribute check.
    for (ak, _) in own_attributes.iter() {
        if let Some(v) = permitted.get(&ak.key.as_ref()) {
            if (part_type != v.1) && (part_name != v.2) {
                let msg = format!(
                    "Can't save sheet: Attribute '{}' not allowed for '{}'",
                    ak.key, part_name
                );
                return Err(msg);
            }
        } else {
            let msg = format!("Can't save sheet: Illegal attribute '{}'.", ak.key);
            return Err(msg);
        }
    }
    let attrs = own_attributes
        .iter()
        .map(|a| &a.0.key)
        .collect::<FnvHashSet<_>>();
    for a in obligatory
        .iter()
        .filter(|pa| pa.part_name == part_name && pa.part_type == part_type)
    {
        if !attrs.contains(&a.key) {
            let m = format!("Can't save sheet: Obligatory attribute missing '{}'", a.key);
            return Err(m);
        }
    }
    Ok(())
}

#[cfg(test)]
mod character_tests {
    // use crate::database::root_db::tests::*;
    // use crate:;database::root_db::characters::character_tests;
}
