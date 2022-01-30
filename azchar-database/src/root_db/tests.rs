//! This file contains the basic setup for most tests.
use crate::root_db::system_config::SystemConfig;
use crate::LoadedDbs;
use diesel::SqliteConnection;

use tempfile::TempDir;

pub(crate) const MEMORY_SPHERE: &str = "\
permitted_parts = [\
{ part_name = \"main\", part_type = \"Main\", obligatory = true },\
{ part_name = \"Memory Sphere\", part_type = \"InventoryItem\", obligatory = true },\
{ part_name = \"spell\", part_type = \"Ability\", obligatory = false },\
]
permitted_attributes = [\
{ key = \"race\", obligatory = true, attribute_type = 0, attribute_description = \"The character's race.\", part_name = \"main\", part_type = \"Main\" },\
{ key = \"class\", obligatory = true, attribute_type = 0, attribute_description = \"The character's class.\", part_name = \"main\", part_type = \"Main\" },\
{ key = \"character_alignment\", obligatory = true, attribute_type = 0, attribute_description = \"The character's alignment.\", part_name = \"main\", part_type = \"Main\" },\
{ key = \"mana_type\", obligatory = true, attribute_type = 0, attribute_description = \"The type of mana that the memory sphere consumes.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
{ key = \"mana_consumption\", obligatory = true, attribute_type = 0, attribute_description = \"The amount of mana the memory sphere consumes per recollection.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
{ key = \"memory_capacity\", obligatory = true, attribute_type = 0, attribute_description = \"The number of memories that the memory sphere can hold before it breaks.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
{ key = \"memory_sphere_alignment\", obligatory = true, attribute_type = 0, attribute_description = \"The alignment of the memory sphere determines the kind of memories it prefers.\", part_name = \"Memory Sphere\", part_type = \"InventoryItem\" },\
]";

pub(crate) enum TestSystem {
    MemorySphere,
    DnD5,
}

pub(crate) struct TestSetup {
    pub(crate) root_dir: TempDir,
    pub(crate) loaded_dbs: LoadedDbs,
}
/// Setup the test.
pub(crate) fn setup(ts: TestSystem) -> TestSetup {
    let system_name = "Memory Sphere";
    let root_dir = tempfile::Builder::new()
        .prefix("system_dir")
        .rand_bytes(10)
        .tempdir()
        .expect("Failed to create a tempfile.");
    let root_path = root_dir.path().to_string_lossy();
    let a = match ts {
        TestSystem::MemorySphere => MEMORY_SPHERE.to_string(),
        TestSystem::DnD5 => std::fs::read_to_string("../examples/dnd5e0.toml").expect("Yes."),
    };
    let sys_config: SystemConfig = toml::from_str(&a).expect("Could not toml");
    let system = sys_config
        .into_system(&root_path, system_name)
        .expect("Could not create system.");
    TestSetup {
        root_dir,
        loaded_dbs: system,
    }
}

pub(crate) fn get_inner_conn<'a>(
    setup: &'a TestSetup,
    params: &(String, String),
) -> &'a SqliteConnection {
    let conn = setup
        .loaded_dbs
        .character_connections()
        .get(params)
        .expect("The character exists. We inserted it.");
    conn.connection.as_ref().expect("yes.")
}
