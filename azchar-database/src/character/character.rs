//! This deals with the character columns.
use crate::character::attribute::NewAttribute;
use crate::character::image::{Image, NewImage};
use crate::character::note::{Note};
use crate::root_db::system::{PermittedAttribute, PermittedPart};
use crate::shared::Part;

use azchar_error::ma;

use diesel::result::Error as DbError;
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use diesel::{OptionalExtension, SqliteConnection};
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn character_type(&self) -> &str {
        &self.character_type
    }

    pub fn speed(&self) -> i32 {
        self.speed
    }

    pub fn weight(&self) -> Option<i32> {
        self.weight
    }

    pub fn size(&self) -> &Option<String> {
        &self.size
    }

    pub fn part_type(&self) -> Part {
        self.part_type
    }

    pub fn hp_total(&self) -> &Option<i32> {
        &self.hp_total
    }

    pub fn hp_current(&self) -> &Option<i32> {
        &self.hp_current
    }

    pub fn belongs_to(&self) -> &Option<i64> {
        &self.belongs_to
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
        permitted_attrs: &[PermittedAttribute],
        image: &Option<Image>,
    ) -> Result<(), String> {
        use self::characters::dsl::*;
        use crate::character::attribute::attributes::dsl as adsl;
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

        let pid = characters
            .order_by(id.desc())
            .select(id)
            .first(conn)
            .map_err(ma)?;
        let new_attributes = permitted_attrs
            .iter()
            .filter(|pa| pa.obligatory_for_part(self.part_type, &self.character_type))
            .map(|a| NewAttribute::from_permitted(pid, a))
            .collect::<Vec<_>>();

        diesel::insert_into(adsl::attributes)
            .values(&new_attributes)
            .execute(conn)
            .map_err(ma)?;
        if let Some(image) = image {
            NewImage {
                of: pid,
                format: image.format.clone(),
                content: image.content.clone(),
            }
            .insert_new(conn)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// This is used purely for creating new parts.
pub struct InputCharacter {
    pub name: String,
    pub character_type: String,
    pub speed: i32,
    pub weight: Option<i32>,
    pub size: Option<String>,
    pub hp_total: Option<i32>,
    pub hp_current: Option<i32>,
    pub belongs_to: Option<i64>,
    pub part_type: Part,
}

impl InputCharacter {
    pub fn test() -> Self {
        InputCharacter {
            name: "Memory Thief".to_string(),
            character_type: "spell".to_string(),
            speed: 40,
            weight: None,
            size: None,
            hp_total: Some(30),
            hp_current: Some(10),
            belongs_to: Some(1),
            part_type: Part::Ability,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn character_type(&self) -> &str {
        &self.character_type
    }

    pub fn speed(&self) -> i32 {
        self.speed
    }

    pub fn weight(&self) -> Option<i32> {
        self.weight
    }

    pub fn size(&self) -> &Option<String> {
        &self.size
    }

    pub fn part_type(&self) -> Part {
        self.part_type
    }

    pub fn hp_total(&self) -> &Option<i32> {
        &self.hp_total
    }

    pub fn hp_current(&self) -> &Option<i32> {
        &self.hp_current
    }

    pub fn belongs_to(&self) -> &Option<i64> {
        &self.belongs_to
    }
}

/// exists to make working with CompleteCharacter simpler.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CharacterPart {
    id: Option<i64>,
    pub(crate) name: String,
    uuid: String,
    pub(crate) character_type: String,
    pub speed: i32,
    pub weight: Option<i32>,
    pub size: Option<String>,
    pub hp_total: Option<i32>,
    pub hp_current: Option<i32>,
    pub(crate) part_type: Part,
    pub belongs_to: Option<i64>,
    pub attributes: Vec<(AttributeKey, AttributeValue)>,
    pub image: Option<Image>,
}

impl CharacterPart {
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn part_type(&self) -> Part {
        self.part_type
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn character_type(&self) -> &str {
        &self.character_type
    }

    pub fn test() -> Self {
        Self {
            id: Some(5),
            name: "Memory Thief".to_owned(),
            uuid: "5936ce00-2275-463c-106a-0f2edde38000".to_owned(),
            character_type: "spell".to_owned(),
            speed: 0,
            weight: Some(1),
            size: None,
            hp_total: None,
            hp_current: None,
            part_type: Part::Ability,
            belongs_to: Some(1),
            attributes: vec![],
            image: None,
        }
    }

    pub(crate) fn from_db_character(db_char: Character) -> Self {
        Self {
            id: Some(db_char.id),
            name: db_char.name,
            uuid: db_char.uuid,
            character_type: db_char.character_type,
            speed: db_char.speed,
            weight: db_char.weight,
            size: db_char.size,
            hp_total: db_char.hp_total,
            hp_current: db_char.hp_current,
            part_type: db_char.part_type,
            belongs_to: db_char.belongs_to,
            attributes: vec![],
            image: None,
        }
    }
}

impl CharacterPart {
    /// This is an incomplete comparison which does not take attributes into account.
    fn compare_part(&self, other: &Self) -> bool {
        self.id == other.id
            && self.part_type == other.part_type
            && self.speed == other.speed
            && self.weight == other.weight
            && self.size == other.size
            && self.hp_total == other.hp_total
            && self.hp_current == other.hp_current
            && self.belongs_to == other.belongs_to
            && self.character_type == other.character_type
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
    pub(crate) image: Option<Image>,
    pub(crate) notes: Vec<Note>,
}

impl CompleteCharacter {
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn weight(&self) -> Option<i32> {
        self.weight
    }

    pub fn parts(&self) -> &[CharacterPart] {
        &self.parts
    }

    pub fn attributes(&self) -> &[(AttributeKey, AttributeValue)] {
        &self.attributes
    }

    pub fn image(&self) -> &Option<Image> {
        &self.image
    }

    /// Compare the main parts of two complete characters: NB: Attributes not compared.
    fn compare_main(&self, other: &CompleteCharacter) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.uuid == other.uuid
            && self.character_type == other.character_type
            && self.speed == other.speed
            && self.weight == other.weight
            && self.size == other.size
            && self.hp_total == other.hp_total
            && self.hp_current == other.hp_current
            && self.image == other.image
    }

    // Used essentially in tests.
    pub fn to_bare_part(&self) -> CharacterPart {
        CharacterPart {
            id: self.id,
            name: self.name.to_owned(),
            uuid: self.uuid.to_owned(),
            character_type: self.character_type.to_owned(),
            speed: self.speed,
            weight: self.weight,
            size: self.size.to_owned(),
            hp_total: self.hp_total,
            hp_current: self.hp_current,
            part_type: Part::Main,
            belongs_to: None,
            attributes: vec![],
            image: self.image.clone(),
        }
    }

    pub fn load(conn: &SqliteConnection) -> Result<CompleteCharacter, String> {
        use self::characters::dsl::*;
        use super::attribute::attributes::dsl as attr_dsl;

        let then = std::time::Instant::now();

        let mut chars: Vec<Character> = characters.load(conn).map_err(ma)?;
        let notes = Note::load_all(conn).map_err(ma)?;
        let mut images = Image::load_all(conn).map_err(ma)?;
        let attrs: Vec<Attribute> = attr_dsl::attributes.load(conn).map_err(ma)?;

        let a = then.elapsed().as_micros();
        let mut attrs = Attributes::key_val_vec(attrs);
        chars.sort_unstable_by(|a, b| b.id.cmp(&a.id));
        attrs.sort_unstable_by(|a, b| b.0.of.cmp(&a.0.of));

        if chars.is_empty() {
            return Err("No parts found for character.".to_string());
        }
        let core = chars.pop().expect("We have at least one.");
        let core_image = if images.last().map(|i| i.of == core.id).unwrap_or(false) {
            images.pop()
        } else {
            None
        };

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
            let image = if images.last().map(|i| i.of == c.id).unwrap_or(false) {
                images.pop()
            } else {
                None
            };
            let mut char_part = CharacterPart::from_db_character(c);
            char_part.attributes = c_attrs;
            char_part.image = image;
            subs.push(char_part);
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
            image: core_image,
            notes,
        })
    }

    /// Store a character in an existing sheet.
    /// If the sheet is empty a new character is created, otherwise it is updated.
    /// NB: The sheet should already exist.
    /// NB2: We disallow characters lacking obligatory parts, or that have parts that are disallowed.
    pub fn save(
        mut self,
        conn: &SqliteConnection,
        (permitted_attrs, permitted_parts): (&[PermittedAttribute], &[PermittedPart]),
    ) -> Result<(), String> {
        use self::characters::dsl::*;
        use super::image::images::dsl::images;
        use super::note::notes::dsl::notes;
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
                error_string = format!("Save error: {:?}", e);
                DbError::NotFound
            })?;
            if old_complete == self {
                let b = then.elapsed().as_micros();
                println!("same ret: {}", b);
                println!("same ret check: {}", b - a);
                return Ok(());
            }

            let permitted_parts_map = permitted_parts
                .iter()
                .map(|p| ((p.part_name.as_ref(), p.part_type), p.obligatory))
                .collect::<FnvHashMap<(&str, Part), bool>>();

            let permitted_attrs_map = permitted_attrs
                .iter()
                .map(|a| (a.key.as_ref(), (a.obligatory, a.part_type, &a.part_name)))
                .collect::<FnvHashMap<&str, (bool, Option<Part>, &Option<String>)>>();
            let obligatory_attrs = permitted_attrs
                .iter()
                .filter(|a| a.obligatory)
                .collect::<Vec<_>>();

            // Do the work.
            if permitted_parts_map
                .get(&(self.character_type.as_ref(), Part::Main))
                .is_none()
            {
                error_string = "Main part type is not permitted in the system".to_owned();
                return Err(DbError::NotFound);
            }
            let mut own_oblig_part_count = 1;
            for p in self.parts.iter() {
                if let Some(val) =
                    permitted_parts_map.get(&(p.character_type.as_ref(), p.part_type))
                {
                    if *val {
                        own_oblig_part_count += 1;
                    }
                } else {
                    error_string = format!("Forbidden part found: '{}'", p.character_type);
                    return Err(DbError::NotFound);
                }
            }

            let opc = permitted_parts_map
                .into_iter()
                .filter(|(_, ob)| *ob)
                .count();
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
            let mut upd_images = Vec::new();
            // Insert or update main character.
            if existing.is_none() {
                new_chars.push((self.image.take(), NewCharacter::from_complete(&self)));
            } else if let Some(_own_id) = self.id {
                if !self.compare_main(&old_complete) {
                    upd_chars.push(Character::from_complete(&self));
                    if let Some(i) = self.image.take() {
                        upd_images.push(i);
                    }
                }
            } else {
                new_chars.push((self.image.take(), NewCharacter::from_complete(&self)));
            }

            // Insert or update sub-characters.
            for sub_char in self.parts.iter_mut() {
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
                            upd_chars.push(Character::from_part(sub_char));
                            if let Some(i) = sub_char.image.take() {
                                upd_images.push(i);
                            }
                        }
                    } else {
                        new_chars.push((sub_char.image.take(), NewCharacter::from_part(sub_char)));
                    }
                } else {
                    new_chars.push((sub_char.image.take(), NewCharacter::from_part(sub_char)));
                }
            }

            for (image, new_char) in new_chars.into_iter() {
                new_char
                    .checked_insert(conn, permitted_parts, permitted_attrs, &image)
                    .map_err(|e| {
                        error_string = e;
                        DbError::NotFound
                    })?;
            }
            for chunk in upd_chars.chunks(999) {
                diesel::replace_into(characters)
                    .values(chunk)
                    .execute(conn)?;
            }
            for chunk in upd_images.chunks(999) {
                diesel::replace_into(images).values(chunk).execute(conn)?;
            }
            for chunk in self.notes.chunks(999) {
                diesel::replace_into(notes).values(chunk).execute(conn)?;
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
        permitted_parts: &[PermittedPart],
        permitted_attrs: &[PermittedAttribute],
    ) -> Result<(), String> {
        use self::characters::dsl::*;
        // Insert/update the image if it exists.
        if let Some(ref i) = chp.image {
            i.update(conn).map_err(ma)?;
        }
        if let Some(ch_id) = chp.id {
            return diesel::update(characters.filter(id.eq(ch_id)))
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
                .map(|_| ())
                .map_err(ma);
        }
        NewCharacter::from_part(&chp).checked_insert(
            conn,
            permitted_parts,
            permitted_attrs,
            &chp.image,
        )
    }
}

fn check_attributes_vs_db(
    own_attributes: &[(AttributeKey, AttributeValue)],
    permitted: &FnvHashMap<&str, (bool, Option<Part>, &Option<String>)>,
    (part_name, part_type): (&str, Part),
    obligatory: &[&PermittedAttribute],
) -> Result<(), String> {
    // First attribute check.
    for (ak, _) in own_attributes.iter() {
        if let Some(v) = permitted.get(&ak.key.as_ref()) {
            let part_type_ok = v.1.map(|x| x == part_type).unwrap_or(true);
            let part_name_ok = v.2.as_ref().map(|x| x == part_name).unwrap_or(true);
            if !part_type_ok || !part_name_ok {
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
        .filter(|pa| pa.permitted_for_part(part_type, part_name))
    {
        if !attrs.contains(&a.key) {
            let m = format!("Can't save sheet: Obligatory attribute missing '{}'", a.key);
            return Err(m);
        }
    }
    Ok(())
}

impl CompleteCharacter {
    /// Compare the main parts of two complete characters: NB: Attributes not compared.
    pub fn compare_main_test(&self, other: &CompleteCharacter) -> bool {
        let mut result = true;
        if self.id != other.id {
            println!("self.id={:?}, other.id={:?}", self.id, other.id);
            result = false;
        }
        if self.speed != other.speed {
            println!("self.speed={}, other.speed={}", self.speed, other.speed);
            result = false;
        }
        if self.weight != other.weight {
            println!(
                "self.weight={:?}, other.weight={:?}",
                self.weight, other.weight
            );
            result = false;
        }
        if self.hp_total != other.hp_total {
            println!(
                "self.hp_total={:?}, other.hp_total={:?}",
                self.hp_total, other.hp_total
            );
            result = false;
        }
        if self.hp_current != other.hp_current {
            println!(
                "self.hp_current={:?}, other.hp_current={:?}",
                self.hp_current, other.hp_current
            );
            result = false;
        }
        if self.name != other.name {
            println!("self.name={:?}, other.name={:?}", self.name, other.name);
            result = false;
        }
        if self.uuid != other.uuid {
            println!("self.uuid={:?}, other.uuid={:?}", self.uuid, other.uuid);
            result = false;
        }
        if self.character_type != other.character_type {
            println!(
                "self.character_type={:?}, other.character_type={:?}",
                self.character_type, other.character_type
            );
            result = false;
        }
        if self.size != other.size {
            println!("self.size={:?}, other.size={:?}", self.size, other.size);
            result = false;
        }
        if self.image != other.image {
            println!("Images don't match.");
            result = false;
        }
        result
    }
}

#[cfg(test)]
mod character_tests {
    // use crate::database::root_db::tests::*;
    // use crate:;database::root_db::characters::character_tests;
}
