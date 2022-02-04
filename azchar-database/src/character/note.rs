use diesel::result::Error as DbError;
use diesel::SqliteConnection;
use diesel::*;
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};

use azchar_error::ma;

table! {
    notes(id) {
        id -> BigInt,
        date -> Text,
        title -> Text,
        content -> Nullable<Text>,
    }
}

/// Represent global notes.
#[derive(Clone, Debug, Insertable, Default, Deserialize, Serialize)]
#[table_name = "notes"]
pub struct InputNote {
    /// Title of the note. Must exist.
    pub title: String,
    /// Content is optional.
    pub content: Option<String>,
}

/// Represents an complete note.
#[derive(Debug, Clone, PartialEq, Identifiable, Queryable, Insertable, Deserialize, Serialize)]
#[table_name = "notes"]
pub struct Note {
    pub id: i64,
    pub date: String,
    pub title: String,
    pub content: Option<String>,
}

impl InputNote {
    /// Just in case.
    pub fn new_note(title: String, content: Option<String>) -> Self {
        Self { title, content }
    }

    /// A convenience function.
    pub(crate) fn insert_new(self, conn: &SqliteConnection) -> Result<usize, String> {
        use self::notes::dsl::*;
        insert_into(notes).values(&self).execute(conn).map_err(ma)
    }
}

impl Note {
    /// Get all character notes.
    pub fn load_all(conn: &SqliteConnection) -> Result<Vec<Self>, DbError> {
        use self::notes::dsl::*;
        notes.order_by(date.desc()).load(conn)
    }

    /// A convenience function.
    pub(crate) fn update(&self, conn: &SqliteConnection) -> Result<usize, String> {
        use self::notes::dsl::*;
        replace_into(notes).values(self).execute(conn).map_err(ma)
    }
}

#[cfg(test)]
mod notes_tests {
    use super::*;
    use crate::root_db::characters::character_tests::*;
    use crate::root_db::tests;

    #[test]
    fn create_and_get_test() {
        let mut test_setup = tests::setup(tests::TestSystem::DnD5);
        let conn = create_char_with_name_and_connect(&mut test_setup, "Euridice");
        let inner_conn = conn.connect().expect("yeah, yeah, eyah...");

        let title = "My first ever note.".to_string();
        let content = Some("Today I went on an adventure.".to_string());
        let note = InputNote::new_note(title.clone(), content.clone());
        note.insert_new(&inner_conn).expect("could not insert.");
        let loaded_notes = Note::load_all(&inner_conn).expect("Could not load.");

        assert_eq!(loaded_notes.len(), 1);
        assert_eq!(loaded_notes[0].title, title);
        assert_eq!(loaded_notes[0].content, content);
    }
}
