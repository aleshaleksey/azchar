//! This deals with the attributes table.
use super::character::characters;
use azchar_error::ma;

use diesel::{Connection, SqliteConnection};
use diesel::{ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};

use fnv::FnvHashMap;
use std::iter::Iterator;

table! {
    // NB, key and of should be unique.
    attributes(id) {
        id -> BigInt,
        key -> Text,
        value_num -> Nullable<BigInt>,
        value_text -> Nullable<Text>,
        description -> Nullable<Text>,
        of -> BigInt,
    }
}

joinable!(attributes -> characters(of));

/// A structure to store a db ref.
#[derive(Debug, Clone, PartialEq, Queryable, Identifiable, Insertable)]
#[table_name = "attributes"]
pub struct Attribute {
    id: i64,
    pub key: String,
    pub value_num: Option<i64>,
    pub value_text: Option<String>,
    pub description: Option<String>,
    pub(crate) of: i64,
}

impl Attribute {
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn of(&self) -> i64 {
        self.id
    }
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "attributes"]
pub struct NewAttribute {
    pub(crate) key: String,
    pub(crate) value_num: Option<i64>,
    pub(crate) value_text: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) of: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttributeValue {
    id: Option<i64>,
    value_num: Option<i64>,
    value_text: Option<String>,
    description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AttributeKey {
    pub(crate) key: String,
    pub(super) of: i64,
}

enum NewOrOldAttribute {
    New(NewAttribute),
    Old(Attribute),
}

fn kv_into_attribute(k: &AttributeKey, v: &AttributeValue) -> NewOrOldAttribute {
    if let Some(id) = v.id {
        NewOrOldAttribute::Old(Attribute {
            id,
            key: k.key.clone(),
            value_num: v.value_num,
            value_text: v.value_text.clone(),
            description: v.description.clone(),
            of: k.of,
        })
    } else {
        NewOrOldAttribute::New(NewAttribute {
            key: k.key.clone(),
            value_num: v.value_num,
            value_text: v.value_text.clone(),
            description: v.description.clone(),
            of: k.of,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Attributes(pub FnvHashMap<AttributeKey, AttributeValue>);

impl Attributes {
    /// Get existing attributes for a character.
    pub fn get_for_character(char_id: i64, conn: &SqliteConnection) -> Result<Self, String> {
        use self::attributes::dsl::*;
        let attribute_vec: Vec<_> = attributes.filter(of.eq(char_id)).load(conn).map_err(ma)?;
        Ok(Attributes::from_vec(attribute_vec))
    }

    /// An inner function that exiists because
    /// characters need a vector.
    pub(crate) fn get_vec_for_characters(
        char_ids: &[i64],
        conn: &SqliteConnection,
    ) -> Result<Vec<Attribute>, String> {
        use self::attributes::dsl::*;

        let mut attribute_vec: Vec<_> = Vec::new();
        for chunk in char_ids.chunks(999) {
            let mut chunk = attributes.filter(of.eq_any(chunk)).load(conn).map_err(ma)?;
            attribute_vec.append(&mut chunk);
        }
        Ok(attribute_vec)
    }

    /// A utility function that is used in multiple places.
    pub fn from_vec(vector: Vec<Attribute>) -> Self {
        Self(
            vector
                .into_iter()
                .map(|a: Attribute| {
                    let att_key = AttributeKey {
                        key: a.key,
                        of: a.of,
                    };
                    let att_value = AttributeValue {
                        id: Some(a.id),
                        value_num: a.value_num,
                        value_text: a.value_text,
                        description: a.description,
                    };
                    (att_key, att_value)
                })
                .collect::<FnvHashMap<_, _>>(),
        )
    }

    pub fn key_val_vec(vector: Vec<Attribute>) -> Vec<(AttributeKey, AttributeValue)> {
        vector
            .into_iter()
            .map(|a: Attribute| {
                let att_key = AttributeKey {
                    key: a.key,
                    of: a.of,
                };
                let att_value = AttributeValue {
                    id: Some(a.id),
                    value_num: a.value_num,
                    value_text: a.value_text,
                    description: a.description,
                };
                (att_key, att_value)
            })
            .collect::<Vec<_>>()
    }

    pub fn from_key_val_vec(x: &[(AttributeKey, AttributeValue)]) -> Self {
        Self(x.iter().cloned().collect::<FnvHashMap<_, _>>())
    }

    /// Get existing attributes for a list of characters.
    /// Intended to be used to get attributes for a defined subset of
    /// inner characters.
    pub fn get_for_characters(char_ids: &[i64], conn: &SqliteConnection) -> Result<Self, String> {
        let attribute_vec = Self::get_vec_for_characters(char_ids, conn)?;
        Ok(Self::from_vec(attribute_vec))
    }

    // NB: This is not a transaction, which may allow us to speed up character updates.
    pub(crate) fn insert_update_vec<'a, I>(
        vec: I,
        conn: &SqliteConnection,
    ) -> Result<(), diesel::result::Error>
    where
        I: Iterator<Item = (&'a AttributeKey, &'a AttributeValue)>,
    {
        use self::attributes::dsl::*;

        let mut update_vec = Vec::new();
        let mut insert_vec = Vec::new();

        for (k, v) in vec {
            match kv_into_attribute(k, v) {
                NewOrOldAttribute::New(a) => insert_vec.push(a),
                NewOrOldAttribute::Old(a) => update_vec.push(a),
            }
        }
        for chunk in insert_vec.chunks(999) {
            diesel::insert_into(attributes)
                .values(chunk)
                .execute(conn)?;
        }
        for chunk in update_vec.chunks(999) {
            diesel::replace_into(attributes)
                .values(chunk)
                .execute(conn)?;
        }
        Ok(())
    }

    /// Insert or update a character's attributes.
    pub(crate) fn insert_update(
        &self,
        conn: &SqliteConnection,
    ) -> Result<(), diesel::result::Error> {
        conn.transaction::<_, diesel::result::Error, _>(|| {
            Self::insert_update_vec(self.0.iter(), conn)
        })
    }

    /// Insert a single key value.
    pub(crate) fn insert_update_key_value(
        k: &AttributeKey,
        v: &AttributeValue,
        conn: &SqliteConnection,
    ) -> Result<(), String> {
        use self::attributes::dsl::*;
        match kv_into_attribute(k, v) {
            NewOrOldAttribute::Old(a) => diesel::replace_into(attributes)
                .values(a)
                .execute(conn)
                .map(|_| ()),
            NewOrOldAttribute::New(a) => diesel::insert_into(attributes)
                .values(a)
                .execute(conn)
                .map(|_| ()),
        }
        .map_err(ma)
    }
}
