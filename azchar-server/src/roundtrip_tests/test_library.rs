//! I am dumping all the tests here for now.
use super::*;
use crate::requests::{Request, Response};
use azchar_database::character::attribute::InputAttribute;
use azchar_database::character::character::{CompleteCharacter, InputCharacter};
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
        FrameReply::Success(x) => return Err(format!("Wrong success: {:?}", x)),
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
    assert!(
        maybe_sword.is_some(),
        "We lost a magical sword. Not good. {:?}",
        maybe_sword
    );

    let sword = maybe_sword.unwrap();
    assert_eq!(sword.name(), "+1 Scimitar");
    assert_eq!(sword.speed, 0);
    assert_eq!(sword.weight, Some(3));
    assert_eq!(sword.size, Some("medium".to_owned()));
    assert_eq!(sword.hp_total, None);
    assert_eq!(sword.hp_current, None);
}

#[test]
fn create_euridice_and_set_some_stats() {
    let (mut frame, _dir, euridice) = create_euridice_and_load_inner();

    let uuid = euridice.uuid().to_owned();
    let name = euridice.name().to_owned();
    let (sk, sv) = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "str")
        .cloned()
        .unwrap();
    let (dk, dv) = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "dex")
        .cloned()
        .unwrap();
    let (ck, cv) = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "con")
        .cloned()
        .unwrap();
    let (ik, iv) = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "int")
        .cloned()
        .unwrap();
    let (wk, wv) = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "wis")
        .cloned()
        .unwrap();
    let (hk, hv) = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "cha")
        .cloned()
        .unwrap();

    assert_eq!(sv.value_num(), None);
    assert_eq!(dv.value_num(), None);
    assert_eq!(cv.value_num(), None);
    assert_eq!(iv.value_num(), None);
    assert_eq!(wv.value_num(), None);
    assert_eq!(hv.value_num(), None);
    let new_str = sv.update_value_num(Some(10));
    let new_dex = dv.update_value_num(Some(16));
    let new_con = cv.update_value_num(Some(10));
    let new_int = iv.update_value_num(Some(14));
    let new_wis = wv.update_value_num(Some(12));
    let new_cha = hv.update_value_num(Some(14));
    let str_req = Request::UpdateAttribute(name.to_owned(), uuid.to_owned(), sk, new_str);
    let dex_req = Request::UpdateAttribute(name.to_owned(), uuid.to_owned(), dk, new_dex);
    let con_req = Request::UpdateAttribute(name.to_owned(), uuid.to_owned(), ck, new_con);
    let int_req = Request::UpdateAttribute(name.to_owned(), uuid.to_owned(), ik, new_int);
    let wis_req = Request::UpdateAttribute(name.to_owned(), uuid.to_owned(), wk, new_wis);
    let cha_req = Request::UpdateAttribute(name.to_owned(), uuid.to_owned(), hk, new_cha);
    match frame.send_and_receive(str_req) {
        FrameReply::Success(Response::UpdateAttribute) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdateAttribute`, got {:?}", r),
    };
    match frame.send_and_receive(dex_req) {
        FrameReply::Success(Response::UpdateAttribute) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdateAttribute`, got {:?}", r),
    };
    match frame.send_and_receive(con_req) {
        FrameReply::Success(Response::UpdateAttribute) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdateAttribute`, got {:?}", r),
    };
    match frame.send_and_receive(int_req) {
        FrameReply::Success(Response::UpdateAttribute) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdateAttribute`, got {:?}", r),
    };
    match frame.send_and_receive(wis_req) {
        FrameReply::Success(Response::UpdateAttribute) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdateAttribute`, got {:?}", r),
    };
    match frame.send_and_receive(cha_req) {
        FrameReply::Success(Response::UpdateAttribute) => {}
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdateAttribute`, got {:?}", r),
    };

    let summon_euridice_request = Request::LoadCharacter(name, uuid);
    let loaded_euridice = match frame.send_and_receive(summon_euridice_request) {
        FrameReply::Success(Response::LoadCharacter(c)) => c,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::UpdatePart`, got {:?}", r),
    };

    let (_, sv) = loaded_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "str")
        .cloned()
        .unwrap();
    let (_, dv) = loaded_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "dex")
        .cloned()
        .unwrap();
    let (_, cv) = loaded_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "con")
        .cloned()
        .unwrap();
    let (_, iv) = loaded_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "int")
        .cloned()
        .unwrap();
    let (_, wv) = loaded_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "wis")
        .cloned()
        .unwrap();
    let (_, hv) = loaded_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "cha")
        .cloned()
        .unwrap();
    assert_eq!(sv.value_num(), Some(10));
    assert_eq!(dv.value_num(), Some(16));
    assert_eq!(cv.value_num(), Some(10));
    assert_eq!(iv.value_num(), Some(14));
    assert_eq!(wv.value_num(), Some(12));
    assert_eq!(hv.value_num(), Some(14));
}

#[test]
fn create_euridice_and_give_her_some_experience() {
    let (mut frame, _dir, euridice) = create_euridice_and_load_inner();

    let uuid = euridice.uuid().to_owned();
    let name = euridice.name().to_owned();
    let maybe_exp = euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "exp_current");
    assert!(
        maybe_exp.is_none(),
        "Non-obligatory attributes shouldn't be here yet."
    );

    let experience = InputAttribute {
        key: "exp_current".to_owned(),
        value_num: Some(-999),
        value_text: Some("Himitsu-desu!".to_string()),
        description: Some("Mostly running away from dragons and getting drunk.".to_string()),
        of: euridice.id().unwrap_or(1),
    };
    let experience_req = Request::CreateAttribute(name.to_owned(), uuid.to_owned(), experience);
    let experienced_euridice = match frame.send_and_receive(experience_req) {
        FrameReply::Success(Response::CreateAttributePart(c)) => c,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::CreateAttribute`, got {:?}", r),
    };

    let (_, exp_value) = experienced_euridice
        .attributes()
        .iter()
        .find(|(k, _)| k.key() == "exp_current")
        .cloned()
        .expect("We just created the exp!");
    assert_eq!(exp_value.value_num(), Some(-999));
}

#[test]
fn create_euridice_and_delete_euridice() {
    let (mut frame, _dir, euridice) = create_euridice_and_load_inner();
    let list = match frame.send_and_receive(Request::ListCharacters) {
        FrameReply::Success(Response::ListCharacters(list)) => list,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::ListCharacters`, got {:?}", r),
    };
    assert_eq!(list.len(), 1);

    let uuid = euridice.uuid().to_owned();
    let name = euridice.name().to_owned();
    let get_rid_of_ce = Request::DeleteCharacter(name, uuid);
    let list = match frame.send_and_receive(get_rid_of_ce) {
        FrameReply::Success(Response::DeleteCharacter(list)) => list,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::DeleteCharacter`, got {:?}", r),
    };
    assert!(list.is_empty(), "Die! Die! Why won't you die?");
}

#[test]
fn create_euridice_and_create_more_characters_than_barney() {
    let (mut frame, _dir, _) = create_euridice_and_load_inner();
    for x in [
        "Saloth", "Zeb", "Nil", "Drauchir", "Narli", "Narumi", "Eirei", "Morgil", "Ezel",
    ] {
        match frame.send_and_receive(Request::CreateCharacterSheet(x.to_owned())) {
            FrameReply::Success(Response::CreateCharacterSheet(_)) => {}
            _ => panic!("Expected `Response::CreateCharacterSheet`, got some kind of crap."),
        }
    }
    let mut list = match frame.send_and_receive(Request::ListCharacters) {
        FrameReply::Success(Response::ListCharacters(list)) => list,
        FrameReply::Fail(e) => panic!("Failed to send and receive: {}", e),
        FrameReply::Success(r) => panic!("Expect `Response::ListCharacters`, got {:?}", r),
    };
    assert_eq!(list.len(), 10);
    list.sort_by(|a, b| a.name().cmp(b.name()));
    let name_list = list.iter().map(|x| x.name()).collect::<Vec<_>>();
    assert_eq!(
        &name_list,
        &[
            "Drauchir", "Eirei", "Euridice", "Ezel", "Morgil", "Narli", "Narumi", "Nil", "Saloth",
            "Zeb"
        ],
    );
}

#[test]
fn create_euridice_and_fiddle_with_euridice() {
    let (mut frame, _dir, euridice) = create_euridice_and_load_inner();
    assert_eq!(euridice.weight(), None, "Euridice should be weightless!");
    for part in euridice.parts() {
        assert_eq!(
            part.weight,
            None,
            "{} should be weightless too..",
            part.name()
        );
    }
    let mut stringridice = serde_json::to_string(&euridice).expect("Yes me can.");
    stringridice = stringridice.replace("\"weight\":null", "\"weight\":999");

    let newridice: CompleteCharacter = serde_json::from_str(&stringridice).expect("We can string.");

    let update_euridice_req = Request::CreateUpdateCharacter(newridice.clone());
    match frame.send_and_receive(update_euridice_req) {
        FrameReply::Success(Response::CreateUpdateCharacter(_)) => {}
        _ => panic!("`Response::CreateUpdateCharacter`."),
    }

    let nrc_req = Request::LoadCharacter(euridice.name().to_owned(), euridice.uuid().to_owned());
    std::thread::sleep(std::time::Duration::from_millis(500));
    let newridice_loaded = match frame.send_and_receive(nrc_req) {
        FrameReply::Success(Response::LoadCharacter(newridice)) => newridice,
        _ => panic!("`Response::CreateUpdateCharacter`."),
    };

    assert!(
        newridice.compare_main_test(&newridice_loaded),
        "We loaded not what we saved!"
    );
    assert_eq!(newridice, newridice_loaded, "We loaded not what we saved!");
    assert_ne!(
        euridice, newridice,
        "Euridice should not be the same as Neuridice."
    );
    assert_ne!(
        euridice, newridice_loaded,
        "Euridice should not be the same as NeuridiceL."
    );
    assert_eq!(newridice_loaded.weight(), Some(999));
    for part in newridice_loaded.parts() {
        assert_eq!(
            part.weight,
            Some(999),
            "{} should not be weightless..",
            part.name()
        );
    }
}
