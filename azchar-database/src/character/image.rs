use diesel::result::Error as DbError;
use diesel::SqliteConnection;
use diesel::*;
use diesel::{Insertable, Queryable, RunQueryDsl};
use std::io::Read;

use crate::character::character::characters;
use azchar_error::ma;

table! {
    images(id) {
        id -> BigInt,
        of -> BigInt,
        format -> Text,
        content -> Blob,
    }
}
allow_tables_to_appear_in_same_query!(characters, images);
// joinable!(images -> characters(of));

/// A link to a character image.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct InputImage {
    /// The character to which it belongs to.
    pub of: i64,
    /// The link to the image.
    pub link: String,
}

impl InputImage {
    pub(crate) fn convert_to_new(self) -> Result<NewImage, String> {
        let path = std::path::PathBuf::from(&self.link);
        let ext = match path.extension() {
            Some(e) => e,
            _ => return Err(format!("Link '{:?}' not a valid image.", path)),
        }
        .to_string_lossy()
        .to_owned();
        // Open and read new image.
        let mut new_image = std::fs::File::open(&path).map_err(ma)?;
        let mut bytes = Vec::new();
        new_image.read_to_end(&mut bytes).map_err(ma)?;

        Ok(NewImage {
            of: self.of,
            format: ext.to_string(),
            content: bytes,
        })
    }
}

/// The actual image to be imported.
#[derive(Clone, Debug, Insertable, Default, Deserialize, Serialize)]
#[table_name = "images"]
pub struct NewImage {
    pub of: i64,
    pub format: String,
    pub content: Vec<u8>,
}

impl NewImage {
    /// A convenience function.
    pub(crate) fn insert_new(self, conn: &SqliteConnection) -> Result<usize, String> {
        use self::images::dsl::*;
        replace_into(images).values(&self).execute(conn).map_err(ma)
    }
}

#[derive(Debug, Clone, PartialEq, Identifiable, Queryable, Insertable, Deserialize, Serialize)]
#[table_name = "images"]
pub struct Image {
    pub id: i64,
    pub of: i64,
    pub format: String,
    pub content: Vec<u8>,
}

impl Image {
    /// Get all character images.
    pub fn load_all(conn: &SqliteConnection) -> Result<Vec<Self>, DbError> {
        use self::images::dsl::*;
        images.order_by(of.desc()).load(conn)
    }

    /// Get the latest after insertion.
    pub fn get_latest(conn: &SqliteConnection) -> Result<Self, String> {
        use self::images::dsl::*;
        images.order_by(id.desc()).first(conn).map_err(ma)
    }

    /// A convenience function.
    pub(crate) fn update(&self, conn: &SqliteConnection) -> Result<usize, String> {
        use self::images::dsl::*;
        replace_into(images).values(self).execute(conn).map_err(ma)
    }
}

#[cfg(test)]
mod notes_tests {
    use super::*;
    use crate::root_db::characters::character_tests::*;
    use crate::root_db::tests;

    #[test]
    fn convert_input_to_new() {
        let inm = InputImage {
            of: 1,
            link: "../examples/c-euri-2021b.png".to_string(),
        };

        let mut image_file = std::fs::File::open("../examples/c-euri-2021b.png").expect("yah..");
        let new_image: NewImage = inm
            .convert_to_new()
            .expect("Could not convert to new image.");

        assert_eq!(new_image.of, 1);
        assert_eq!(&new_image.format, "png");

        let mut bytes = Vec::new();
        image_file.read_to_end(&mut bytes).expect("yah yah");
        assert_eq!(new_image.content, bytes);
    }

    #[test]
    fn create_and_get_test() {
        let mut test_setup = tests::setup(tests::TestSystem::DnD5);
        let conn = create_char_with_name_and_connect(&mut test_setup, "Euridice");
        let inner_conn = conn.connect().expect("yeah, yeah, eyah...");

        let inm = InputImage {
            of: 1,
            link: "../examples/c-euri-2021b.png".to_string(),
        };
        let new_image: NewImage = inm
            .convert_to_new()
            .expect("Could not convert to new image.");

        new_image.insert_new(&inner_conn).expect("Can insert");
        let loaded_images = Image::load_all(&inner_conn).expect("can lkoad all");

        assert_eq!(loaded_images.len(), 1);

        let mut image_file = std::fs::File::open("../examples/c-euri-2021b.png").expect("yah..");
        let mut bytes = Vec::new();
        image_file.read_to_end(&mut bytes).expect("yah yah");

        assert_eq!(loaded_images[0].of, 1);
        assert_eq!(&loaded_images[0].format, "png");
        assert_eq!(loaded_images[0].content, bytes);
    }
}
