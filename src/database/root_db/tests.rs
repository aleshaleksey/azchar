//! This file contains the basic setup for most tests.
use crate::database::root_db::system_config::SystemConfig;
use crate::LoadedDbs;

use tempfile::TempDir;

pub(crate) struct TestSetup {
    pub(crate) root_dir: TempDir,
    pub(crate) loaded_dbs: LoadedDbs,
}
/// Setup the test.
pub(crate) fn setup() -> TestSetup {
    let system_name = "Memory Sphere";
    let root_dir = tempfile::Builder::new()
        .prefix("system_dir")
        .rand_bytes(10)
        .tempdir()
        .expect("Failed to create a tempfile.");
    let root_path = root_dir.path().to_string_lossy();
    let a = "\
permitted_parts = [\
{ part_name = \"main\", part_type = \"Main\" },\
{ part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
]
permitted_attributes = [\
{ key = \"race\", attribute_type = 0, attribute_description = \"The character's race.\", part_name = \"main\", part_type = \"Main\" },\
{ key = \"class\", attribute_type = 0, attribute_description = \"The character's class.\", part_name = \"main\", part_type = \"Main\" },\
{ key = \"character_alignment\", attribute_type = 0, attribute_description = \"The character's alignment.\", part_name = \"main\", part_type = \"Main\" },\
{ key = \"mana_type\", attribute_type = 0, attribute_description = \"The type of mana that the memory sphere consumes.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
{ key = \"mana_consumption\", attribute_type = 0, attribute_description = \"The amount of mana the memory sphere consumes per recollection.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
{ key = \"memory_capacity\", attribute_type = 0, attribute_description = \"The number of memories that the memory sphere can hold before it breaks.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
{ key = \"memory_sphere_alignment\", attribute_type = 0, attribute_description = \"The alignment of the memory sphere determines the kind of memories it prefers.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
]";
    let sys_config: SystemConfig = toml::from_str(&a).expect("Could not toml");
    let system = sys_config
        .into_system(&root_path, system_name)
        .expect("Could not create system.");
    TestSetup {
        root_dir,
        loaded_dbs: system,
    }
}
