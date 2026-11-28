#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use std::sync::Arc;

use myrtle_app::services::tier_lists::entities::{
    ArtRef, EntitySummary, FacetValue, StoryCatalogues, catalogue, known, resolve, validate,
};
use myrtle_engine_story::dto::{
    GalleryGroup, GalleryPicture, StoryArtGallery, StoryArtGroup, StoryArtKind, StoryArtPicture,
    StoryCategory, StoryGallery,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::handbook::{BasicInfo, OperatorProfile, OperatorRace};
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_testkit::fixtures::has_story_avg;
use myrtle_types::{EntityKind, Server};

const NONE: &StoryCatalogues = &StoryCatalogues::NONE;
const PICTURE_KINDS: [EntityKind; 3] = [
    EntityKind::StoryCg,
    EntityKind::StoryScene,
    EntityKind::ArchivePic,
];

fn model_with(id: &str, op: Operator) -> GameModel {
    let mut model = GameModel::default();
    model.operators.insert(id.to_owned(), op);
    model
}

fn named(name: &str, obtainable: bool) -> Operator {
    Operator {
        name: name.to_owned(),
        is_not_obtainable: !obtainable,
        ..Default::default()
    }
}

fn with_race(race: OperatorRace) -> Operator {
    Operator {
        name: "Exusiai".to_owned(),
        profile: Some(OperatorProfile {
            basic_info: BasicInfo {
                race,
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn one<'a>(s: &'a EntitySummary, key: &str) -> Option<&'a str> {
    match s.facets.get(key)? {
        FacetValue::One(v) => Some(v),
        FacetValue::Many(_) => None,
    }
}

fn operator(op: Operator) -> EntitySummary {
    resolve(
        &model_with("char_x", op),
        NONE,
        EntityKind::Operator,
        "char_x",
    )
    .unwrap()
}

#[test]
fn an_unknown_id_does_not_resolve() {
    let model = model_with("char_002_amiya", named("Amiya", true));
    assert!(resolve(&model, NONE, EntityKind::Operator, "char_002_amiya").is_some());
    assert!(resolve(&model, NONE, EntityKind::Operator, "char_999_nobody").is_none());
    assert!(!known(
        &model,
        NONE,
        EntityKind::Operator,
        "char_999_nobody"
    ));
}

#[test]
fn placing_an_id_no_loaded_server_knows_is_a_bad_request() {
    let en = model_with("char_002_amiya", named("Amiya", true));
    let cn = model_with("char_4999_cnonly", named("Someone", true));
    let servers = [(&en, NONE), (&cn, NONE)];
    assert!(validate(servers, EntityKind::Operator, "char_002_amiya").is_ok());
    assert!(validate(servers, EntityKind::Operator, "char_4999_cnonly").is_ok());
    match validate(servers, EntityKind::Operator, "char_999_nobody") {
        Err(AppError::BadRequest(msg)) => assert!(msg.contains("char_999_nobody"), "{msg}"),
        other => panic!("expected BadRequest, got {other:?}"),
    }
}

#[test]
fn an_unobtainable_operator_resolves_but_is_not_offered() {
    let model = model_with("char_1001_amiya2", named("Amiya", false));
    assert!(resolve(&model, NONE, EntityKind::Operator, "char_1001_amiya2").is_some());
    assert!(catalogue(&model, NONE, EntityKind::Operator).is_empty());
}

#[test]
fn an_operator_carries_its_avatar_link_and_facets() {
    let s = operator(named("Amiya", true));
    assert_eq!(s.icon.as_deref(), Some("/avatar/char_x"));
    assert_eq!(s.href.as_deref(), Some("/operators/char_x"));
    assert_eq!(one(&s, "rarity"), Some("1"));
    assert!(s.facets.contains_key("profession"));
}

#[test]
fn race_is_sent_in_the_operator_index_spelling() {
    assert_eq!(
        one(&operator(with_race(OperatorRace::Sankta)), "race"),
        Some("Sankta")
    );
    assert_eq!(
        one(&operator(with_race(OperatorRace::CautusChimera)), "race"),
        Some("Cautus/Chimera")
    );
}

#[test]
fn an_unknown_or_missing_race_sends_no_facet() {
    assert!(
        !operator(with_race(OperatorRace::Unknown))
            .facets
            .contains_key("race")
    );
    assert!(!operator(Operator::default()).facets.contains_key("race"));
    assert_eq!(
        one(&operator(with_race(OperatorRace::Undisclosed)), "race"),
        Some("Undisclosed")
    );
}

#[test]
fn group_and_team_are_sent_with_their_names() {
    let s = operator(Operator {
        nation_id: "kazimierz".to_owned(),
        group_id: Some("pinus".to_owned()),
        group_name: Some("Pinus Sylvestris".to_owned()),
        team_id: Some(String::new()),
        ..Default::default()
    });
    assert_eq!(one(&s, "nation_id"), Some("kazimierz"));
    assert_eq!(one(&s, "group_id"), Some("pinus"));
    assert_eq!(one(&s, "group_name"), Some("Pinus Sylvestris"));
    assert!(!s.facets.contains_key("team_id"));
    assert!(!s.facets.contains_key("team_name"));
}

#[test]
fn a_skill_key_needs_both_halves_to_match() {
    let model = model_with("char_x", Operator::default());
    assert!(!known(&model, NONE, EntityKind::Skill, "char_x"));
    assert!(!known(&model, NONE, EntityKind::Skill, "char_x:skchr_x_1"));
}

fn art(kind: StoryArtKind, ids: &[&str]) -> ArtRef {
    ArtRef::Own(Arc::new(StoryArtGallery {
        kind,
        groups: vec![StoryArtGroup {
            id: "act13side".to_owned(),
            name: "Near Light".to_owned(),
            category: StoryCategory::Side,
            pictures: ids
                .iter()
                .map(|id| StoryArtPicture {
                    id: (*id).to_owned(),
                    width: 1600,
                    height: 900,
                })
                .collect(),
        }],
    }))
}

fn archive() -> Arc<StoryGallery> {
    let pic = |id: &str, title: &str, url: Option<&str>| GalleryPicture {
        id: id.to_owned(),
        title: title.to_owned(),
        description: None,
        picture_type: "ROGUE_IMAGE".to_owned(),
        url: url.map(str::to_owned),
    };
    Arc::new(StoryGallery {
        groups: vec![GalleryGroup {
            id: "rogue_1".to_owned(),
            name: "Phantom & Crimson Solitaire".to_owned(),
            pictures: vec![
                pic(
                    "pic_rogue_1_1",
                    "Warm Fire",
                    Some("/textures/avg/imgs/pic_rogue_1_1.png"),
                ),
                pic("pic_rogue_1_KV1", "", None),
            ],
        }],
    })
}

fn full() -> StoryCatalogues {
    StoryCatalogues {
        cg: Some(art(StoryArtKind::Cg, &["avg_1_1", "avg_1_2"])),
        scene: Some(art(StoryArtKind::Scene, &["bg_corridor"])),
        archive: Some(archive()),
    }
}

#[test]
fn a_story_cg_is_its_story_with_the_thumb_route() {
    let (model, story) = (GameModel::default(), full());
    let s = resolve(&model, &story, EntityKind::StoryCg, "avg_1_2").unwrap();
    assert_eq!(s.kind, EntityKind::StoryCg);
    assert_eq!(s.name, "Near Light");
    assert_eq!(
        s.icon.as_deref(),
        Some("/story/art-gallery/cg/avg_1_2/thumb")
    );
    assert_eq!(one(&s, "group"), Some("Near Light"));
    assert_eq!(one(&s, "group_id"), Some("act13side"));
    assert_eq!(one(&s, "category"), Some("side"));
    let ids: Vec<String> = catalogue(&model, &story, EntityKind::StoryCg)
        .into_iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(ids, ["avg_1_1", "avg_1_2"]);
}

#[test]
fn cg_and_scene_ids_are_separate_spaces() {
    let (model, story) = (GameModel::default(), full());
    assert!(known(&model, &story, EntityKind::StoryScene, "bg_corridor"));
    assert!(!known(&model, &story, EntityKind::StoryCg, "bg_corridor"));
    assert!(!known(&model, &story, EntityKind::StoryScene, "avg_1_1"));
    let s = resolve(&model, &story, EntityKind::StoryScene, "bg_corridor").unwrap();
    assert_eq!(
        s.icon.as_deref(),
        Some("/story/art-gallery/scene/bg_corridor/thumb")
    );
    assert!(!known(&model, &story, EntityKind::StoryCg, "AVG_1_1"));
}

#[test]
fn an_archive_picture_carries_its_group_and_type() {
    let (model, story) = (GameModel::default(), full());
    let s = resolve(&model, &story, EntityKind::ArchivePic, "pic_rogue_1_1").unwrap();
    assert_eq!(s.name, "Warm Fire");
    assert_eq!(
        s.icon.as_deref(),
        Some("/story/gallery/pic_rogue_1_1/thumb")
    );
    assert_eq!(one(&s, "group"), Some("Phantom & Crimson Solitaire"));
    assert_eq!(one(&s, "group_id"), Some("rogue_1"));
    assert_eq!(one(&s, "picture_type"), Some("ROGUE_IMAGE"));
    let bare = resolve(&model, &story, EntityKind::ArchivePic, "pic_rogue_1_KV1").unwrap();
    assert_eq!(bare.name, "pic_rogue_1_KV1");
    assert_eq!(bare.icon, None);
}

#[test]
fn a_missing_catalogue_lists_nothing_and_does_not_validate() {
    let model = GameModel::default();
    for kind in PICTURE_KINDS {
        assert!(catalogue(&model, NONE, kind).is_empty(), "{kind:?}");
    }
    assert!(resolve(&model, NONE, EntityKind::StoryCg, "avg_1_1").is_none());
    assert!(validate([(&model, NONE)], EntityKind::StoryCg, "avg_1_1").is_err());
    let story = full();
    assert!(
        validate(
            [(&model, NONE), (&model, &story)],
            EntityKind::StoryCg,
            "avg_1_1"
        )
        .is_ok()
    );
}

#[test]
fn the_summary_wire_shape_is_unchanged() {
    let s = resolve(
        &GameModel::default(),
        &full(),
        EntityKind::StoryCg,
        "avg_1_1",
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&s).unwrap(),
        serde_json::json!({
            "kind": "story_cg",
            "id": "avg_1_1",
            "name": "Near Light",
            "icon": "/story/art-gallery/cg/avg_1_1/thumb",
            "href": null,
            "facets": { "category": "side", "group": "Near Light", "group_id": "act13side" },
        })
    );
}

#[test]
fn sprite_and_main_story_ids_the_data_lacks_are_unknown() {
    let model = GameModel::default();
    for kind in [EntityKind::StorySprite, EntityKind::MainStory] {
        assert!(
            resolve(&model, NONE, kind, "avg_npc_935").is_none(),
            "{kind:?}"
        );
        assert!(catalogue(&model, NONE, kind).is_empty(), "{kind:?}");
        assert!(
            validate([(&model, NONE)], kind, "main_14").is_err(),
            "{kind:?}"
        );
    }
}

#[tokio::test]
async fn every_catalogue_entry_resolves_to_itself_on_en() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let model = &snapshot.model;
    let story = StoryCatalogues::gather(&ctx, Server::EN, &EntityKind::ALL).await;
    for kind in EntityKind::ALL {
        let entries = catalogue(model, &story, kind);
        let needs_avg = matches!(kind, EntityKind::StorySprite) && !has_story_avg();
        if !matches!(kind, EntityKind::StoryCg | EntityKind::StoryScene) && !needs_avg {
            assert!(!entries.is_empty(), "{kind:?} offers nothing");
        }
        for entry in &entries {
            assert_eq!(entry.kind, kind);
            assert_eq!(
                resolve(model, &story, kind, &entry.id).as_ref(),
                Some(entry),
                "{kind:?}"
            );
        }
    }
    assert_eq!(catalogue(model, &story, EntityKind::Class).len(), 8);
    db.close().await;
}

#[tokio::test]
async fn story_sprites_and_main_story_resolve_on_en() {
    if !has_story_avg() {
        eprintln!("skipping story_sprites_and_main_story_resolve_on_en: no en/textures/avg");
        return;
    }
    let Some((_, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let model = &snapshot.model;
    let kalts = resolve(model, NONE, EntityKind::StorySprite, "char_003_kalts").unwrap();
    assert_eq!(kalts.name, "Kal'tsit");
    assert_eq!(one(&kalts, "source"), Some("operator"));
    assert_eq!(one(&kalts, "char_id"), Some("char_003_kalts"));
    assert_eq!(kalts.href.as_deref(), Some("/operators/char_003_kalts"));
    assert_eq!(
        kalts.icon.as_deref(),
        Some("/story-sprite-thumb/char_003_kalts")
    );
    let sprites = catalogue(model, NONE, EntityKind::StorySprite);
    let first_npc = sprites.iter().position(|s| one(s, "source") == Some("npc"));
    assert!(
        first_npc.is_some_and(|i| sprites[..i]
            .iter()
            .all(|s| one(s, "source") == Some("operator"))),
        "operators sort first"
    );

    let chapters = catalogue(model, NONE, EntityKind::MainStory);
    assert!(chapters.len() >= 15, "{} chapters", chapters.len());
    let first = &chapters[0];
    assert_eq!(first.kind, EntityKind::MainStory);
    assert_eq!(one(first, "episode"), Some("0"));
    assert!(validate([(model, NONE)], EntityKind::MainStory, &first.id).is_ok());
    assert!(validate([(model, NONE)], EntityKind::MainStory, "main_999").is_err());
    db.close().await;
}

#[tokio::test]
async fn amiya_and_her_first_skill_resolve_on_en() {
    let Some((_, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let model = &snapshot.model;
    let amiya = resolve(model, NONE, EntityKind::Operator, "char_002_amiya").unwrap();
    assert_eq!(amiya.name, "Amiya");
    let skill_id = &model.operators["char_002_amiya"].skills[0].skill_id;
    let skill = resolve(
        model,
        NONE,
        EntityKind::Skill,
        &format!("char_002_amiya:{skill_id}"),
    )
    .unwrap();
    assert_eq!(one(&skill, "slot"), Some("1"));
    assert_eq!(one(&skill, "operator"), Some("Amiya"));
    assert!(validate([(model, NONE)], EntityKind::Skill, skill_id).is_err());
    db.close().await;
}
