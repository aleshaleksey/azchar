//! I am dumping all the tests here for now.
use super::*;
use crate::requests::{Request, Response};

use std::path::PathBuf;
use tempfile::TempDir;

use azchar_error::ma;

const DND_TOML: &str = "examples/dnd5e.toml";
const DND_TOML2: &str = "../examples/dnd5e.toml";

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

#[test]
fn create_euridice_and_load() {
    println!("Help?");
    let (mut frame, _db_dir) = create_dnd().expect("Couldn't create.");
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
}
