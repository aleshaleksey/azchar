//! This file deals with encoding and decoing the TOML files needed for the root db.
// TODO: test conversion into a new system.
use crate::root_db::system::PermittedAttribute as DbPermittedAttribute;
use crate::root_db::system::PermittedPart as DbPermittedPart;
use crate::root_db::system::{NewPermittedAttribute, NewPermittedPart};
use crate::shared::*;
use crate::LoadedDbs;
use azchar_error::ma;

use diesel::result::Error as DsError;
use diesel::RunQueryDsl;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

// Create all needed tables
embed_migrations!("migrations_root_db");

/// This represents a part that is permitted and that will be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct PermittedPart {
    part_name: String,
    part_type: Part,
    obligatory: bool,
}

impl From<PermittedPart> for NewPermittedPart {
    fn from(p: PermittedPart) -> Self {
        Self {
            part_name: p.part_name,
            part_type: p.part_type,
            obligatory: p.obligatory,
        }
    }
}

/// This represents a permitted attribute, to be created on a new sheet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct PermittedAttribute {
    key: String,
    attribute_type: i32,
    attribute_description: String,
    part_name: String,
    part_type: Part,
    obligatory: bool,
}

impl From<PermittedAttribute> for NewPermittedAttribute {
    fn from(a: PermittedAttribute) -> Self {
        Self {
            key: a.key,
            attribute_type: a.attribute_type,
            attribute_description: a.attribute_description,
            part_name: a.part_name,
            part_type: a.part_type,
            obligatory: a.obligatory,
        }
    }
}

/// This structure can recreate the system configuration for a game system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemConfig {
    permitted_parts: Vec<PermittedPart>,
    permitted_attributes: Vec<PermittedAttribute>,
}

impl SystemConfig {
    // Create an instance of `SystemConfig` from a config toml.
    pub fn from_config(system_config_path: &str) -> Result<Self, String> {
        let mut config_file = std::fs::File::open(system_config_path).map_err(ma)?;
        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).map_err(ma)?;
        toml::from_str(&config_string).map_err(ma)
    }

    /// This function exists to:
    // a) Create the root database with all three tables.
    // b) Insert permitted attributes and parts into it.
    pub fn into_system(self, path: &str, system_name: &str) -> Result<LoadedDbs, String> {
        // The required tables.
        use crate::root_db::system::permitted_attributes::dsl as pa_dsl;
        use crate::root_db::system::permitted_parts::dsl as pp_dsl;

        let file_name = format!("{}.db", system_name);
        let file_path = PathBuf::from(path).join(&file_name);
        if file_path.exists() {
            return Err(format!(
                "{:?} already exists as a file! Try again.",
                file_path
            ));
        }

        let _sheet_db = File::create(file_path.clone()).map_err(ma)?;
        let file_path_string = file_path.to_string_lossy();

        let mut loaded_dbs = LoadedDbs::new_system(&file_path_string)?;
        let new_root = loaded_dbs.get_inner_root()?;
        crate::set_pragma(new_root)?;

        let Self {
            permitted_parts,
            permitted_attributes,
        } = self;
        let permitted_attributes: Vec<NewPermittedAttribute> =
            permitted_attributes.into_iter().map(Into::into).collect();
        let permitted_parts: Vec<NewPermittedPart> =
            permitted_parts.into_iter().map(Into::into).collect();

        embedded_migrations::run(new_root).map_err(ma)?;
        new_root
            .immediate_transaction::<_, DsError, _>(|| {
                // Insert values as needed.
                diesel::insert_into(pp_dsl::permitted_parts)
                    .values(&permitted_parts)
                    .execute(new_root)?;
                diesel::insert_into(pa_dsl::permitted_attributes)
                    .values(&permitted_attributes)
                    .execute(new_root)?;
                Ok(())
            })
            .map_err(ma)?;

        let pp = DbPermittedPart::load_all(new_root)?;
        let pa = DbPermittedAttribute::load_all(new_root)?;
        loaded_dbs.permitted_parts = pp;
        loaded_dbs.permitted_attrs = pa;

        Ok(loaded_dbs)
    }
}

#[cfg(test)]
mod system_config_tests {
    use super::{PermittedAttribute, PermittedPart, SystemConfig};
    use crate::root_db::tests::MEMORY_SPHERE;
    use crate::shared::*;

    #[test]
    fn permitted_part_from_toml1() {
        let a = "\
    part_name = \"Test Part\"
    part_type = \"Body\"
    obligatory = true
    ";
        let expected = PermittedPart {
            part_name: String::from("Test Part"),
            part_type: Part::Body,
            obligatory: true,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn permitted_part_from_toml2() {
        let a = "\
    part_name = \"Spellbook\"
    part_type = \"InventoryItem\"
    obligatory = false
    ";
        let expected = PermittedPart {
            part_name: String::from("Spellbook"),
            part_type: Part::InventoryItem,
            obligatory: false,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn permitted_part_from_toml3() {
        let a = "\
    part_name = \"Giant Cupcake\"
    part_type = \"Summon\"
    obligatory = true
    ";
        let expected = PermittedPart {
            part_name: String::from("Giant Cupcake"),
            part_type: Part::Summon,
            obligatory: true,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn permitted_attributes_from_toml1() {
        let a = "\
    key = \"spell_level\"
    attribute_type = 0
    attribute_description = \"The level of the spell.\"
    part_name = \"spell\"
    part_type = \"Ability\"
    obligatory = true
    ";
        let expected = PermittedAttribute {
            key: String::from("spell_level"),
            attribute_type: 0,
            attribute_description: String::from("The level of the spell."),
            part_name: String::from("spell"),
            part_type: Part::Ability,
            obligatory: true,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn permitted_attributes_from_toml2() {
        let a = "\
    key = \"attack_power\"
    attribute_type = -4
    attribute_description = \"The giant cupcake's attack power.\"
    part_name = \"Giant Cupcake\"
    part_type = \"Summon\"
    obligatory = true
    ";
        let expected = PermittedAttribute {
            key: String::from("attack_power"),
            attribute_type: -4,
            attribute_description: String::from("The giant cupcake's attack power."),
            part_name: String::from("Giant Cupcake"),
            part_type: Part::Summon,
            obligatory: true,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn permitted_attributes_from_toml3() {
        let a = "\
    key = \"range\"
    attribute_type = 0
    attribute_description = \"The range of the weapon, in hexes.\"
    part_name = \"ranged_weapon\"
    part_type = \"InventoryItem\"
    obligatory = true
    ";
        let expected = PermittedAttribute {
            key: String::from("range"),
            attribute_type: 0,
            attribute_description: String::from("The range of the weapon, in hexes."),
            part_name: String::from("ranged_weapon"),
            part_type: Part::InventoryItem,
            obligatory: true,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn permitted_attributes_from_toml4() {
        let a = "\
    key = \"coordinates\"
    attribute_type = 0
    attribute_description = \"The coordinates of the base.\"
    part_name = \"base\"
    part_type = \"Asset\"
    obligatory = true
    ";
        let expected = PermittedAttribute {
            key: String::from("coordinates"),
            attribute_type: 0,
            attribute_description: String::from("The coordinates of the base."),
            part_name: String::from("base"),
            part_type: Part::Asset,
            obligatory: true,
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn system_config_from_toml1() {
        let a = MEMORY_SPHERE;
        let expected = SystemConfig {
            permitted_parts: vec![
                PermittedPart {
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedPart {
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
                PermittedPart {
                    part_name: String::from("spell"),
                    part_type: Part::Ability,
                    obligatory: false,
                },
            ],
            permitted_attributes: vec![
                PermittedAttribute {
                    key: String::from("race"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's race."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("class"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's class."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("character_alignment"),
                    attribute_type: 0,
                    attribute_description: String::from("The character's alignment."),
                    part_name: String::from("main"),
                    part_type: Part::Main,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("mana_type"),
                    attribute_type: 0,
                    attribute_description: String::from("The type of mana that the memory sphere consumes."),
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("mana_consumption"),
                    attribute_type: 0,
                    attribute_description: String::from("The amount of mana the memory sphere consumes per recollection."),
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("memory_capacity"),
                    attribute_type: 0,
                    attribute_description: String::from("The number of memories that the memory sphere can hold before it breaks."),
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
                PermittedAttribute {
                    key: String::from("memory_sphere_alignment"),
                    attribute_type: 0,
                    attribute_description: String::from("The alignment of the memory sphere determines the kind of memories it prefers."),
                    part_name: String::from("Memory Sphere"),
                    part_type: Part::InventoryItem,
                    obligatory: true,
                },
            ],
        };
        assert_eq!(expected, toml::from_str(&a).expect("Could not toml"));
    }

    #[test]
    fn dnd5_config_from_toml() {
        let text = std::fs::read_to_string("../examples/dnd5e.toml").expect("Yes.");
        let dnd5toml: SystemConfig = toml::from_str(&text).expect("ho ho ho");

        assert_eq!(dnd5toml.permitted_parts.len(), 10);
        assert_eq!(dnd5toml.permitted_attributes.len(), 126);
    }
}
