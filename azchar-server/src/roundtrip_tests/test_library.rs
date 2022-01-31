//! I am dumping all the tests here for now.
use super::*;
use crate::requests::{Request, Response};
use azchar_database::character::character::{InputCharacter, CompleteCharacter};
use azchar_database::shared::Part;

use std::path::PathBuf;
use tempfile::TempDir;

use azchar_error::ma;

const DND_TOML: &str = "examples/dnd5e0.toml";
const DND_TOML2: &str = "../examples/dnd5e0.toml";

pub(super) fn create_dnd() -> Result<(Frame, TempDir), String> {
    let new_dir = TempDir::new().map_err(ma)?;
    let storage_path = new_dir.path();
    let sp_s = storage_path.to_string_lossy().to_owned();

    let mut fram = Frame::create(storage_path.to_path_buf(), "127.0.0.1:55555");
    println!("Frame::create done.");

    let toml = if PathBuf::from(DND_TOML).exists() {
        DND_TOML
    } else if PathBuf::from(DND_TOML2).exists() {
        DND_TOML2
    } else {
        panic!("Oh no! No path for system config file.");
    };
    if !storage_path.exists() {
        panic!("storage path doesn't exist: {:?}", storage_path);
    }
    let create_request =
        Request::CreateSystem("dnd5e_test".to_string(), sp_s.to_string(), toml.to_owned());

    match fram.send_and_receive(create_request) {
        FrameReply::Success(Response::CreateSystem(s)) => {
            println!("Successfully created system: {:?}", s)
        }
        FrameReply::Success(x) => return Err(format!("{:?}", x)),
        FrameReply::Fail(e) => return Err(e),
    }

    Ok((fram, new_dir))
}

/// An inner function which we basically use for everything.
fn create_euridice_and_load_inner() -> (Frame, TempDir, CompleteCharacter) {
    println!("Help?");
    let (mut frame, db_dir) = create_dnd().expect("Couldn't create.");
    println!("Created frame with system.");

    let create_euridice_request = Request::CreateCharacterSheet("Euridice".to_string());
    let reply = match frame.send_and_receive(create_euridice_request) {
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => r,
    };
    println!("Created character sheet. {:?}", reply);
    //     /
    assert!(matches!(reply, Response::CreateCharacterSheet(_)));
    let list = match reply {
        Response::CreateCharacterSheet(list) => list,
        _ => panic!("We expected a 'Response::CreateCharacterSheet', we really did."),
    };
    assert_eq!(list.len(), 1, "We should have one character for now.");
    assert_eq!(list[0].name(), "Euridice");

    let name = list[0].name().to_owned();
    let uuiudi = list[0].uuid().to_owned();
    let load_euridice_request = Request::LoadCharacter(name, uuiudi);
    let loaded_euridice = match frame.send_and_receive(load_euridice_request) {
        FrameReply::Success(Response::LoadCharacter(c)) => c,
        _ => panic!("We expected a 'Response::LoadCharacter', we really did."),
    };

    assert_eq!(
        list[0].name(),
        loaded_euridice.name(),
        "This is not a Euridice."
    );
    assert_eq!(
        list[0].uuid(),
        loaded_euridice.uuid(),
        "This is not the Euridice we are looking for."
    );
    (frame, db_dir, loaded_euridice)
}

#[test]
fn create_euridice_and_load() {
    create_euridice_and_load_inner();
}

#[test]
fn create_euridice_and_edit_main_character() {
    let (mut frame, _dir, euridice) = create_euridice_and_load_inner();

    let e_uuid = euridice.uuid().to_owned();
    let e_name = euridice.name().to_owned();

    let mut mutable_euridice = euridice.to_bare_part();
    assert_eq!(mutable_euridice.speed, 0);
    assert_eq!(mutable_euridice.weight, None);
    assert_eq!(mutable_euridice.size, None);
    assert_eq!(mutable_euridice.hp_total, None);
    assert_eq!(mutable_euridice.hp_current, None);
    mutable_euridice.speed = 30;
    mutable_euridice.weight = Some(130);
    mutable_euridice.size = Some("medium".to_owned());
    mutable_euridice.hp_total = Some(63);
    mutable_euridice.hp_current = Some(63);

    let modify_euridice_request = Request::UpdatePart(e_name, e_uuid, mutable_euridice);
    match frame.send_and_receive(modify_euridice_request) {
        FrameReply::Success(Response::UpdatePart) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdatePart`, got {:?}", r),
    };

    let summon_euridice_request =
        Request::LoadCharacter(euridice.name().to_owned(), euridice.uuid().to_owned());

    let loaded_euridice = match frame.send_and_receive(summon_euridice_request) {
        FrameReply::Success(Response::LoadCharacter(c)) => c,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdatePart`, got {:?}", r),
    };

    let accessible_euridice = loaded_euridice.to_bare_part();
    assert_eq!(accessible_euridice.speed, 30);
    assert_eq!(accessible_euridice.weight, Some(130));
    assert_eq!(accessible_euridice.size, Some("medium".to_owned()));
    assert_eq!(accessible_euridice.hp_total, Some(63));
    assert_eq!(accessible_euridice.hp_current, Some(63));
}

#[test]
fn create_euridice_and_give_her_a_sword() {
    let (mut frame, _dir, euridice) = create_euridice_and_load_inner();

    let e_uuid = euridice.uuid().to_owned();
    let e_name = euridice.name().to_owned();
    for p in euridice.parts() {
        assert_ne!(
            (p.part_type(), p.character_type()),
            (Part::InventoryItem, "weapon"),
        );
    }

    let scimitar = InputCharacter {
        name: "+1 Scimitar".to_string(),
        character_type: "weapon".to_string(),
        speed: 0,
        weight: Some(3),
        size: Some("medium".to_owned()),
        hp_total: None,
        hp_current: None,
        belongs_to: euridice.id(),
        part_type: Part::InventoryItem,
    };

    let sword_request = Request::CreatePart(e_name.to_owned(), e_uuid.to_owned(), scimitar);
    let armed_euridice = match frame.send_and_receive(sword_request) {
        FrameReply::Success(Response::CreateAttributePart(c)) => c,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `CreateAttributePart`, got {:?}", r),
    };

    let maybe_sword = armed_euridice
        .parts()
        .iter()
        .find(|p| (p.part_type(), p.character_type()) == (Part::InventoryItem, "weapon"));
    assert!(maybe_sword.is_some(), "We lost a magical sword. Not good. {:?}", maybe_sword);

    let sword = maybe_sword.unwrap();
    assert_eq!(sword.name(), "+1 Scimitar");
    assert_eq!(sword.speed, 0);
    assert_eq!(sword.weight, Some(3));
    assert_eq!(sword.size, Some("medium".to_owned()));
    assert_eq!(sword.hp_total, None);
    assert_eq!(sword.hp_current, None);
}
