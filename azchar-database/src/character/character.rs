//! This deals with the character columns.
use crate::root_db::system::{PermittedAttribute, PermittedPart};
use crate::shared::Part;

use azchar_error::ma;

use fnv::{FnvHashMap, FnvHashSet};
use rusqlite::Connection as SqliteConnection;
use rusqlite::Error as RSqlError;
use rusqlite::Row as RSqlRow;

use super::attribute::{Attribute, AttributeKey, AttributeValue, Attributes};

// table! {
//     characters(id) {
//         // Obligatory
//         id -> BigInt,
//         name -> Text,
//         uuid -> Text,
//         character_type -> Text,
//         // Almost obligatory Fields.
//         speed -> Integer,
//         weight -> Nullable<Integer>,
//         size -> Nullable<Text>,
//         hp_total -> Nullable<Integer>,
//         hp_current -> Nullable<Integer>,
//         // References.
//         belongs_to -> Nullable<BigInt>,
//         part_type -> Integer,
//     }
// }

const INSERT_CHAR: &str = "INSERT INTO characters( \
name, \
uuid, \
character_type, \
speed, \
weight, \
size, \
hp_total, \
hp_current, \
belongs_to, \
part_type) VALUES (?,?,?,?,?,?,?,?,?,?);";

const REPLACE_CHAR: &str = "REPLACE INTO characters( \
id, \
name, \
uuid, \
character_type, \
speed, \
weight, \
size, \
hp_total, \
hp_current, \
belongs_to, \
part_type) VALUES (?,?,?,?,?,?,?,?,?,?,?);";

/// A structure to store a db ref.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Character {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) uuid: String,
    pub(crate) character_type: String,
    speed: i32,
    weight: Option<i32>,
    size: Option<String>,
    hp_total: Option<i32>,
    hp_current: Option<i32>,
    pub(crate) belongs_to: Option<i64>,
    pub(crate) part_type: Part,
}

impl Character {
    pub fn get_latest_id(conn: &SqliteConnection) -> Result<i64, String> {
        conn.prepare_cached("SELECT max(id) FROM characters;")
            .map_err(ma)?
            .query_map([], |row| row.get(0))
            .map_err(ma)?
            .next()
            .map(|x| x.map_err(ma))
            .unwrap_or(Ok(-1))
    }

    pub fn get_main_identifiers(
        conn: &SqliteConnection,
    ) -> Result<Option<(i64, String, String)>, String> {
        // Part zero is main.
        let r = conn
            .prepare_cached("SELECT id,name,uuid FROM characters WHERE part_type=0;")
            .map_err(ma)?
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let uuid: String = row.get(2)?;
                Ok((id, name, uuid))
            })
            .map_err(ma)?
            .next()
            .map(|x| x.map_err(ma));
        match r {
            Some(Ok(r)) => Ok(Some(r)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn insert_single(&self, conn: &SqliteConnection) -> Result<usize, String> {
        conn.prepare_cached(INSERT_CHAR)
            .map_err(ma)?
            .execute(params![
                self.name,
                self.uuid,
                self.character_type,
                self.speed,
                self.weight,
                self.size,
                self.hp_total,
                self.hp_current,
                self.belongs_to,
                self.part_type,
            ])
            .map_err(ma)
    }

    pub fn update_single(&self, conn: &SqliteConnection) -> Result<usize, String> {
        if let Some(id) = self.id {
            conn.prepare_cached(REPLACE_CHAR)
                .map_err(ma)?
                .execute(params![
                    id,
                    self.name,
                    self.uuid,
                    self.character_type,
                    self.speed,
                    self.weight,
                    self.size,
                    self.hp_total,
                    self.hp_current,
                    self.belongs_to,
                    self.part_type,
                ])
                .map_err(ma)
        } else {
            Err(format!(
                "Character has no id and cannot be updated ({}, {})",
                self.name, self.uuid,
            ))
        }
    }

    pub(crate) fn from_row(row: &RSqlRow) -> Result<Self, RSqlError> {
        let c = Character {
            id: row.get(0)?,
            name: row.get(1)?,
            uuid: row.get(2)?,
            character_type: row.get(3)?,
            speed: row.get(4)?,
            weight: row.get(5)?,
            size: row.get(6)?,
            hp_total: row.get(7)?,
            hp_current: row.get(8)?,
            belongs_to: row.get(9)?,
            part_type: row.get(10)?,
        };
        Ok(c)
    }

    pub fn load_all(conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        conn.prepare_cached("SELECT * from characters;")
            .map_err(ma)?
            .query_map([], |row| Character::from_row(row))
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }

    fn from_part(part: &CharacterPart) -> Self {
        Character {
            id: part.id,
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
            id: main.id,
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
        let then = std::time::Instant::now();

        let mut chars: Vec<Character> = Character::load_all(conn).map_err(ma)?;
        let attrs: Vec<Attribute> = Attribute::load_all(conn).map_err(ma)?;

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
                if Some(x.0.of) == core.id {
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
                    if Some(x.0.of) == c.id {
                        c_attrs.push(x);
                    } else {
                        next_vec.push(x);
                    }
                } else {
                    break;
                }
            }
            subs.push(CharacterPart {
                id: c.id,
                name: c.name,
                uuid: c.uuid,
                character_type: c.character_type,
                speed: c.speed,
                weight: c.weight,
                size: c.size,
                hp_total: c.hp_total,
                hp_current: c.hp_current,
                part_type: c.part_type,
                belongs_to: core.id,
                attributes: c_attrs,
            });
        }
        let b = then.elapsed().as_micros();
        println!("db-get:{}us, rust-sort: {}us", a, b - a);
        Ok(CompleteCharacter {
            id: core.id,
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
        let then = std::time::Instant::now();
        // A check to see if the existing character already exists here.
        println!("get main ident");
        let existing: Option<(i64, String, String)> = Character::get_main_identifiers(conn)?;
        let a = then.elapsed().as_micros();
        // If the current sheet is already occupied by a different character, return early.
        if let Some(other) = &existing {
            if self.id != Some(other.0) {
                return Err(format!(
                    "A character already exists on this sheet: name:{}, uuid:{}",
                    other.1, other.2,
                ));
            }
        }
        println!("getting to main.");
        let old_complete = CompleteCharacter::load(conn)?;
        if &old_complete == self {
            let b = then.elapsed().as_micros();
            println!("same ret: {}", b);
            println!("same ret check: {}", b - a);
            return Ok(());
        }
        println!("got to main");
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
            return Err("Main part type is not permitted in the system".to_owned());
        }
        let mut own_oblig_part_count = 1;
        for p in self.parts.iter() {
            if let Some(val) = permitted_parts.get(&(p.character_type.as_ref(), p.part_type)) {
                if *val {
                    own_oblig_part_count += 1;
                }
            } else {
                return Err(format!("Forbidden part found: '{}'", p.character_type));
            }
        }

        let opc = permitted_parts.into_iter().filter(|(_, ob)| *ob).count();
        if own_oblig_part_count < opc {
            return Err("Obligatory part missing.".to_string());
        }

        if let Err(e) = check_attributes_vs_db(
            &self.attributes,
            &permitted_attrs_map,
            (&self.character_type, Part::Main),
            &obligatory_attrs,
        ) {
            return Err(e);
        };
        let mut attribute_refs: Vec<_> = Vec::with_capacity(1000); // Why not.
        attribute_refs.extend(self.attributes.iter().map(|(k, v)| (k, v)));
        let mut new_chars = Vec::new();
        let mut upd_chars = Vec::new();
        // Insert or update main character.
        if existing.is_none() {
            new_chars.push(Character::from_complete(self));
        } else if let Some(_own_id) = self.id {
            if !self.compare_main(&old_complete) {
                upd_chars.push(Character::from_complete(self));
            }
        } else {
            new_chars.push(Character::from_complete(self));
        }

        // Insert or update sub-characters.
        for sub_char in self.parts.iter() {
            if let Err(e) = check_attributes_vs_db(
                &sub_char.attributes,
                &permitted_attrs_map,
                (&sub_char.character_type, sub_char.part_type),
                &obligatory_attrs,
            ) {
                return Err(e);
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
                    new_chars.push(Character::from_part(sub_char));
                }
            } else {
                new_chars.push(Character::from_part(sub_char));
            }
        }

        for c in new_chars.into_iter() {
            c.insert_single(conn)?;
        }
        conn.cache_flush().map_err(ma)?;
        for c in upd_chars.into_iter() {
            c.update_single(conn)?;
        }
        conn.cache_flush().map_err(ma)?;
        let d = then.elapsed().as_micros();
        Attributes::insert_update_vec(attribute_refs.into_iter(), conn)?;
        println!("inserted attributes");
        let e = then.elapsed().as_micros();
        println!("insert to chars {}us, to attrs: {}us.", d, e);
        Ok(())
    }

    /// Insert a single key value.
    pub(crate) fn insert_update_character_part(
        chp: CharacterPart,
        conn: &SqliteConnection,
    ) -> Result<usize, String> {
        let new_part = Character::from_part(&chp);
        match new_part.id {
            Some(_) => new_part.update_single(conn),
            None => new_part.insert_single(conn),
        }
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
