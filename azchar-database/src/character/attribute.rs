//! This deals with the attributes table.
use azchar_error::ma;

use fnv::FnvHashMap;
use rusqlite::Connection as SqliteConnection;
use rusqlite::Error as RSqlError;
use rusqlite::Row as RSqlRow;
use std::iter::Iterator;

// table! {
//     // NB, key and of should be unique.
//     attributes(id) {
//         id -> BigInt,
//         key -> Text,
//         value_num -> Nullable<BigInt>,
//         value_text -> Nullable<Text>,
//         description -> Nullable<Text>,
//         of -> BigInt,
//     }
// }racters(of));

/// A structure to store a db ref.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub(crate) id: Option<i64>,
    pub key: String,
    pub value_num: Option<i64>,
    pub value_text: Option<String>,
    pub description: Option<String>,
    pub(crate) of: i64,
}

impl Attribute {
    pub fn id(&self) -> Option<i64> {
        self.id
    }
    pub fn of(&self) -> i64 {
        self.of
    }

    fn into_key_val(self) -> (AttributeKey, AttributeValue) {
        let att_key = AttributeKey {
            key: self.key,
            of: self.of,
        };
        let att_value = AttributeValue {
            id: self.id,
            value_num: self.value_num,
            value_text: self.value_text,
            description: self.description,
        };
        (att_key, att_value)
    }

    pub(crate) fn insert_single(&self, conn: &SqliteConnection) -> Result<usize, String> {
        conn.prepare_cached(
            "INSERT INTO attributes(key, value_num, value_text, description, of) VALUES (?);",
        )
        .map_err(ma)?
        .execute(params![
            self.key,
            self.value_num,
            self.value_text.as_ref(),
            self.description.as_ref(),
            self.of,
        ])
        .map_err(ma)
    }

    pub(crate) fn update_single(&self, conn: &SqliteConnection) -> Result<usize, String> {
        conn.prepare_cached(
            "REPLACE INTO attributes(id, key, value_num, value_text, description, of) VALUES (?);",
        )
        .map_err(ma)?
        .execute(params![
            self.id.unwrap(),
            self.key,
            self.value_num,
            self.value_text.as_ref(),
            self.description.as_ref(),
            self.of,
        ])
        .map_err(ma)
    }

    pub(crate) fn insert_update_key_val(
        k: &AttributeKey,
        v: &AttributeValue,
        conn: &SqliteConnection,
    ) -> Result<usize, String> {
        if let Some(id) = v.id {
            conn.prepare_cached(
                "REPLACE INTO attributes(key, value_num, value_text, description, of) VALUES (?);",
            )
            .map_err(ma)?
            .execute(params![
                id,
                k.key,
                v.value_num,
                v.value_text.as_ref(),
                v.description.as_ref(),
                k.of,
            ])
            .map_err(ma)
        } else {
            conn.prepare_cached(
                "INSERT INTO attributes(key, value_num, value_text, description, of) VALUES (?);",
            )
            .map_err(ma)?
            .execute(params![
                k.key,
                v.value_num,
                v.value_text.as_ref(),
                v.description.as_ref(),
                k.of,
            ])
            .map_err(ma)
        }
    }

    pub(crate) fn from_row(row: &RSqlRow) -> Result<Self, RSqlError> {
        let a = Attribute {
            id: row.get(0)?,
            key: row.get(1)?,
            value_num: row.get(2)?,
            value_text: row.get(3)?,
            description: row.get(4)?,
            of: row.get(5)?,
        };
        Ok(a)
    }

    pub fn load_all(conn: &SqliteConnection) -> Result<Vec<Self>, String> {
        conn.prepare_cached("SELECT * from attributes;")
            .map_err(ma)?
            .query_map([], |row| Attribute::from_row(row))
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)
    }
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

#[derive(Clone, Debug)]
pub struct Attributes(pub FnvHashMap<AttributeKey, AttributeValue>);

impl Attributes {
    /// Get existing attributes for a character.
    pub fn get_for_character(char_id: i64, conn: &SqliteConnection) -> Result<Self, String> {
        let attribute_vec: Vec<_> = conn
            .prepare_cached("SELECT * from attributes WHERE of=:of;")
            .map_err(ma)?
            .query_map([":of", &char_id.to_string()], |row| {
                Attribute::from_row(row)
            })
            .map_err(ma)?
            .collect::<Result<Vec<_>, RSqlError>>()
            .map_err(ma)?;
        Ok(Attributes::from_vec(attribute_vec))
    }

    /// A utility function that is used in multiple places.
    pub fn from_vec(vector: Vec<Attribute>) -> Self {
        Self(
            vector
                .into_iter()
                .map(|a: Attribute| a.into_key_val())
                .collect::<FnvHashMap<_, _>>(),
        )
    }

    pub fn key_val_vec(vector: Vec<Attribute>) -> Vec<(AttributeKey, AttributeValue)> {
        vector
            .into_iter()
            .map(|a: Attribute| a.into_key_val())
            .collect::<Vec<_>>()
    }

    pub fn from_key_val_vec(x: &[(AttributeKey, AttributeValue)]) -> Self {
        Self(x.iter().cloned().collect::<FnvHashMap<_, _>>())
    }

    // NB: This is not a transaction, which may allow us to speed up character updates.
    pub(crate) fn insert_update_vec<'a, I>(vec: I, conn: &SqliteConnection) -> Result<(), String>
    where
        I: Iterator<Item = (&'a AttributeKey, &'a AttributeValue)>,
    {
        for (k, v) in vec {
            Self::insert_update_key_value(k, v, conn)?;
        }
        Ok(())
    }

    /// Insert or update a character's attributes.
    pub(crate) fn insert_update(&self, conn: &SqliteConnection) -> Result<(), String> {
        for (k, v) in self.0.iter() {
            Attribute::insert_update_key_val(k, v, conn)?;
        }
        Ok(())
    }

    /// Insert a single key value.
    pub(crate) fn insert_update_key_value(
        k: &AttributeKey,
        v: &AttributeValue,
        conn: &SqliteConnection,
    ) -> Result<usize, String> {
        Attribute::insert_update_key_val(k, v, conn)
    }
}
