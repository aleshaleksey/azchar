//! Contains shared elements.
use std::hash::Hash;
use rusqlite::ToSql;
use rusqlite::types::*;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// This is used for both the character and system
/// database and is convertable to i16 to be storable
// on an Sqlite database.
pub enum Part {
    // Self.
    Main = 0,
    // Intrinsic parts.
    Body = 1,
    Mechanical = 2,
    // Physical associated items.
    InventoryItem = 3,
    Asset = 4,
    // Abilities (spells, manoeuvres, etc).
    Ability = 5,
    // Other Characters belonging to the character.
    Summon = 6,
    // Minions.
    Minion = 7,
    // Other
    Other = 8,
}

impl Default for Part {
    fn default() -> Self {
        Self::Other
    }
}

impl From<i32> for Part {
    // Here because diesel derive refuses to work.
    fn from(n: i32) -> Self {
        match n {
            0 => Self::Main,
            1 => Self::Body,
            2 => Self::Mechanical,
            3 => Self::InventoryItem,
            4 => Self::Asset,
            5 => Self::Ability,
            6 => Self::Summon,
            7 => Self::Minion,
            _ => Self::Other,
        }
    }
}

impl Into<i64> for Part {
    // Here because diesel derive refuses to work.
    fn into(self) -> i64 {
        match self {
            Self::Main => 0,
            Self::Body => 1,
            Self::Mechanical => 2,
            Self::InventoryItem => 3,
            Self::Asset => 4,
            Self::Ability => 5,
            Self::Summon => 6,
            Self::Minion => 7,
            Self::Other => 255,
        }
    }
}

impl ToSql for Part {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer((*self).into())))
    }
}

impl FromSql for Part {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(Self::Other),
            ValueRef::Integer(i) => Ok(Self::from(i as i32)),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToString for Part {
    fn to_string(&self) -> String {
        match self {
            Self::Main => "Character".to_string(),
            Self::Body => "body part".to_string(),
            Self::Mechanical => "mechanical part".to_string(),
            Self::InventoryItem => "inventory item".to_string(),
            Self::Asset => "character asset".to_string(),
            Self::Ability => "character ability".to_string(),
            Self::Summon => "Summon".to_string(),
            Self::Minion => "Minion".to_string(),
            Self::Other => "other thing".to_string(),
        }
    }
}
