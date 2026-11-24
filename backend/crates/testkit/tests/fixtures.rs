#![allow(clippy::unwrap_used)]

use myrtle_testkit::fixtures::{character_model, root};
use myrtle_types::Server;

#[test]
fn the_character_model_carries_what_the_dps_and_base_suites_read() {
    let Some(model) = character_model(Server::EN) else {
        eprintln!("skip: no EN extract at {}", root(Server::EN).display());
        return;
    };
    let amiya = &model.operators["char_002_amiya"];
    assert!(!amiya.phases.is_empty());
    assert!(amiya.skills.iter().all(|s| s.static_data.is_some()));
    assert!(!amiya.modules.is_empty());
    assert!(amiya.profile.is_some());
    assert!(model.operators.contains_key("char_1001_amiya2"));
    assert!(!model.building.rooms.is_empty());
    assert!(!model.building.layout_terms.is_empty());
    assert!(model.stages.is_empty());
    assert!(std::ptr::eq(model, character_model(Server::EN).unwrap()));
}

#[test]
fn a_server_without_an_extract_is_none() {
    if root(Server::TW).exists() {
        return;
    }
    assert!(character_model(Server::TW).is_none());
}
