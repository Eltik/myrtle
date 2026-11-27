#![allow(clippy::unwrap_used)]

use myrtle_app::privacy::layout::*;
use myrtle_types::EntityKind;
use serde_json::{Value, json};

fn ids(layout: &ProfileLayout) -> Vec<(&'static str, bool)> {
    layout
        .tabs
        .iter()
        .map(|t| (t.id.as_str(), t.visible))
        .collect()
}

const CANONICAL: [(&str, bool); 9] = [
    ("showcase", true),
    ("stats", true),
    ("score", true),
    ("roster", true),
    ("plans", true),
    ("inventory", true),
    ("enemies", true),
    ("optimizer", true),
    ("lookback", true),
];

#[test]
fn null_is_the_canonical_layout() {
    assert_eq!(ids(&ProfileLayout::normalize(&json!(null))), CANONICAL);
    assert_eq!(ids(&ProfileLayout::default()), CANONICAL);
}

#[test]
fn an_empty_list_or_a_shapeless_value_is_the_canonical_layout() {
    for raw in [
        json!({ "tabs": [] }),
        json!({}),
        json!({ "tabs": "stats" }),
        json!([{ "id": "roster", "visible": false }]),
        json!(7),
    ] {
        assert_eq!(ids(&ProfileLayout::normalize(&raw)), CANONICAL, "{raw}");
    }
}

#[test]
fn unknown_ids_and_malformed_entries_are_dropped() {
    let raw = json!({ "tabs": [
        { "id": "achievements", "visible": false },
        { "id": "roster", "visible": false },
        { "id": 3, "visible": false },
        { "visible": false },
        "score",
    ] });
    let layout = ProfileLayout::normalize(&raw);
    assert_eq!(
        ids(&layout),
        [
            ("showcase", true),
            ("roster", false),
            ("stats", true),
            ("score", true),
            ("plans", true),
            ("inventory", true),
            ("enemies", true),
            ("optimizer", true),
            ("lookback", true),
        ]
    );
}

#[test]
fn a_duplicate_keeps_the_first_entry() {
    let raw = json!({ "tabs": [
        { "id": "plans", "visible": false },
        { "id": "stats", "visible": true },
        { "id": "plans", "visible": true },
    ] });
    let layout = ProfileLayout::normalize(&raw);
    assert_eq!(
        layout.tabs[1],
        LayoutTab {
            id: ProfileTabId::Plans,
            visible: false
        }
    );
    assert_eq!(
        layout
            .tabs
            .iter()
            .filter(|t| t.id == ProfileTabId::Plans)
            .count(),
        1
    );
    assert_eq!(layout.tabs.len(), ProfileTabId::ALL.len());
}

#[test]
fn missing_tabs_are_appended_visible_in_canonical_order() {
    let raw = json!({ "tabs": [
        { "id": "optimizer", "visible": false },
        { "id": "inventory", "visible": true },
    ] });
    assert_eq!(
        ids(&ProfileLayout::normalize(&raw)),
        [
            ("showcase", true),
            ("optimizer", false),
            ("inventory", true),
            ("stats", true),
            ("score", true),
            ("roster", true),
            ("plans", true),
            ("enemies", true),
            ("lookback", true),
        ]
    );
}

#[test]
fn a_missing_visible_flag_reads_as_visible() {
    let layout = ProfileLayout::normalize(&json!({ "tabs": [{ "id": "score" }] }));
    assert!(layout.is_visible(ProfileTabId::Score));
}

#[test]
fn sibling_keys_do_not_disturb_tabs() {
    let raw = json!({ "showcase": [1, 2], "tabs": [{ "id": "enemies", "visible": false }] });
    let layout = ProfileLayout::normalize(&raw);
    assert_eq!(layout.showcase, None);
    assert_eq!(
        layout.tabs[1],
        LayoutTab {
            id: ProfileTabId::Enemies,
            visible: false
        }
    );
}

#[test]
fn a_normalized_layout_is_a_fixed_point() {
    let raw = json!({ "tabs": [
        { "id": "roster", "visible": false },
        { "id": "nope" },
        { "id": "score", "visible": true },
    ] });
    let once = ProfileLayout::normalize(&raw);
    let twice = ProfileLayout::normalize(&serde_json::to_value(&once).unwrap());
    assert_eq!(once, twice);
}

#[test]
fn deserializing_normalizes() {
    let layout: ProfileLayout = serde_json::from_value(
        json!({ "tabs": [{ "id": "plans", "visible": false }, { "id": "x" }] }),
    )
    .unwrap();
    assert_eq!(layout.tabs.len(), ProfileTabId::ALL.len());
    assert_eq!(
        layout.tabs[1],
        LayoutTab {
            id: ProfileTabId::Plans,
            visible: false
        }
    );
}

#[test]
fn the_visitor_projection_drops_private_tabs_and_keeps_order() {
    let raw = json!({ "tabs": [
        { "id": "roster", "visible": true },
        { "id": "stats", "visible": false },
        { "id": "score", "visible": false },
    ] });
    let layout = ProfileLayout::normalize(&raw);
    let visitor = layout.visible_only();
    assert_eq!(
        ids(&visitor),
        [
            ("showcase", true),
            ("roster", true),
            ("plans", true),
            ("inventory", true),
            ("enemies", true),
            ("optimizer", true),
            ("lookback", true),
        ]
    );
    assert!(!layout.is_visible(ProfileTabId::Stats));
    assert!(layout.is_visible(ProfileTabId::Roster));
    assert!(!visitor.is_visible(ProfileTabId::Score));
}

fn kinds(layout: &ProfileLayout) -> Vec<&'static str> {
    layout
        .showcase_blocks()
        .iter()
        .map(|b| match b {
            ShowcaseBlock::Favourites { .. } => "favourites",
            ShowcaseBlock::Grid { .. } => "grid",
            ShowcaseBlock::TierList { .. } => "tier_list",
            ShowcaseBlock::Plan { .. } => "plan",
        })
        .collect()
}

const PLAN: &str = "6f1c2e7a-3b44-4d1e-9a52-0c8f7d2b1e90";

#[test]
fn a_layout_without_a_showcase_key_has_no_showcase() {
    assert_eq!(ProfileLayout::default().showcase, None);
    let layout = ProfileLayout::normalize(&json!({ "tabs": [] }));
    assert_eq!(layout.showcase, None);
    assert!(
        serde_json::to_value(&layout)
            .unwrap()
            .get("showcase")
            .is_none()
    );
    for raw in [json!({ "showcase": null }), json!({ "showcase": "x" })] {
        assert_eq!(ProfileLayout::normalize(&raw).showcase, None, "{raw}");
    }
    assert_eq!(
        ProfileLayout::normalize(&json!({ "showcase": {} })).showcase,
        Some(ProfileShowcase::default())
    );
}

#[test]
fn malformed_and_unknown_blocks_are_dropped() {
    let raw = json!({ "showcase": { "blocks": [
        { "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya"] },
        { "type": "favourites", "entity_kind": "pokemon", "ids": ["pikachu"] },
        { "type": "favourites", "entity_kind": "skin", "ids": [] },
        { "type": "favourites", "entity_kind": "skin", "ids": [" ", 4, null] },
        { "type": "favourites", "ids": ["char_002_amiya"] },
        { "type": "badge", "slug": "x" },
        { "slug": "no-type" },
        { "type": "grid", "slug": "  " },
        { "type": "grid" },
        { "type": "grid", "slug": "my-grid" },
        { "type": "tier_list", "slug": 7 },
        { "type": "tier_list", "slug": "meta" },
        { "type": "plan", "id": "not-a-uuid" },
        { "type": "plan", "id": PLAN },
        { "type": "grid", "slug": "x".repeat(121) },
        "grid",
        12,
    ] } });
    let layout = ProfileLayout::normalize(&raw);
    assert_eq!(kinds(&layout), ["favourites", "grid", "tier_list", "plan"]);
    assert_eq!(
        layout.showcase_blocks()[3],
        ShowcaseBlock::Plan {
            id: PLAN.parse().unwrap(),
            operator_id: None,
        }
    );
}

#[test]
fn favourites_ids_are_trimmed_deduplicated_and_clamped() {
    let mut ids: Vec<Value> = vec![json!(" char_002_amiya "), json!("char_002_amiya")];
    ids.extend((0..40).map(|i| json!(format!("id_{i}"))));
    let raw = json!({ "showcase": { "blocks": [
        { "type": "favourites", "entity_kind": "enemy", "ids": ids, "title": "  Who I fear\n most  " },
    ] } });
    let layout = ProfileLayout::normalize(&raw);
    let ShowcaseBlock::Favourites {
        entity_kind,
        ids,
        title,
    } = &layout.showcase_blocks()[0]
    else {
        panic!("favourites");
    };
    assert_eq!(*entity_kind, EntityKind::Enemy);
    assert_eq!(ids.len(), SHOWCASE_MAX_IDS);
    assert_eq!(ids[0], "char_002_amiya");
    assert_eq!(ids[1], "id_0");
    assert_eq!(ids.last().map(String::as_str), Some("id_22"));
    assert_eq!(title.as_deref(), Some("Who I fear most"));
}

#[test]
fn titles_are_cut_to_forty_characters_and_a_blank_one_is_none() {
    let block = |title: Value| {
        let raw = json!({ "showcase": { "blocks": [
            { "type": "favourites", "entity_kind": "skin", "ids": ["a"], "title": title },
        ] } });
        match ProfileLayout::normalize(&raw).showcase_blocks()[0].clone() {
            ShowcaseBlock::Favourites { title, .. } => title,
            _ => panic!("favourites"),
        }
    };
    let long = "é".repeat(50);
    assert_eq!(
        block(json!(long)).map(|t| t.chars().count()),
        Some(SHOWCASE_TITLE_MAX)
    );
    assert_eq!(block(json!("   ")), None);
    assert_eq!(block(json!(3)), None);
    assert_eq!(block(json!("<b>hi</b>")).as_deref(), Some("<b>hi</b>"));
}

#[test]
fn blocks_are_clamped_and_a_repeated_referent_keeps_the_first() {
    let mut blocks: Vec<Value> = vec![
        json!({ "type": "grid", "slug": "a" }),
        json!({ "type": "grid", "slug": "a" }),
        json!({ "type": "tier_list", "slug": "a" }),
        json!({ "type": "plan", "id": PLAN }),
        json!({ "type": "plan", "id": PLAN.to_uppercase() }),
        json!({ "type": "favourites", "entity_kind": "operator", "ids": ["x"], "title": "one" }),
        json!({ "type": "favourites", "entity_kind": "operator", "ids": ["x"], "title": "two" }),
    ];
    blocks.extend((0..20).map(|i| json!({ "type": "grid", "slug": format!("g{i}") })));
    let layout = ProfileLayout::normalize(&json!({ "showcase": { "blocks": blocks } }));
    let got = kinds(&layout);
    assert_eq!(got.len(), SHOWCASE_MAX_BLOCKS);
    assert_eq!(
        &got[..5],
        ["grid", "tier_list", "plan", "favourites", "favourites"]
    );
    assert_eq!(
        layout.showcase_blocks()[5],
        ShowcaseBlock::Grid { slug: "g0".into() }
    );
}

#[test]
fn tabs_and_showcase_round_trip() {
    let raw = json!({
        "tabs": [{ "id": "roster", "visible": true }, { "id": "showcase", "visible": true }],
        "showcase": { "blocks": [
            { "type": "favourites", "entity_kind": "main_story", "ids": ["main_0", "main_1"], "title": "Faves" },
            { "type": "favourites", "entity_kind": "skin", "ids": ["char_002_amiya@epoque#4"] },
            { "type": "grid", "slug": "my-grid" },
            { "type": "tier_list", "slug": "meta" },
            { "type": "plan", "id": PLAN },
        ] },
    });
    let once = ProfileLayout::normalize(&raw);
    let wire = serde_json::to_value(&once).unwrap();
    assert_eq!(
        wire["showcase"], raw["showcase"],
        "a clean showcase is stored as sent"
    );
    assert_eq!(wire["tabs"][0]["id"], json!("roster"));
    assert_eq!(wire["tabs"][1]["id"], json!("showcase"));
    let twice: ProfileLayout = serde_json::from_value(wire).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn the_old_tabs_only_shape_still_reads() {
    let raw = json!({ "tabs": [{ "id": "plans", "visible": false }] });
    let layout: ProfileLayout = serde_json::from_value(raw).unwrap();
    assert_eq!(layout.showcase, None);
    assert_eq!(layout.tabs[0].id, ProfileTabId::Showcase);
    assert_eq!(
        layout.tabs[1],
        LayoutTab {
            id: ProfileTabId::Plans,
            visible: false
        }
    );
    assert_eq!(layout.tabs.len(), ProfileTabId::ALL.len());
}

#[test]
fn the_visitor_projection_gates_the_showcase_on_its_tab_and_plans_on_theirs() {
    let showcase = json!({ "blocks": [
        { "type": "grid", "slug": "g" },
        { "type": "plan", "id": PLAN },
    ] });
    let with = |tabs: Value| {
        ProfileLayout::normalize(&json!({ "tabs": tabs, "showcase": showcase })).visible_only()
    };
    assert_eq!(kinds(&with(json!([]))), ["grid", "plan"]);
    assert_eq!(
        kinds(&with(json!([{ "id": "plans", "visible": false }]))),
        ["grid"]
    );
    let hidden = with(json!([{ "id": "showcase", "visible": false }]));
    assert_eq!(hidden.showcase, None);
    assert!(!hidden.is_visible(ProfileTabId::Showcase));
}

#[test]
fn wire_ids_round_trip_through_serde() {
    for tab in ProfileTabId::ALL {
        let wire = serde_json::to_value(tab).unwrap();
        assert_eq!(wire, json!(tab.as_str()));
        assert_eq!(ProfileTabId::from_id(tab.as_str()), Some(tab));
    }
}

fn background(raw: Value) -> Option<ProfileBackground> {
    ProfileLayout::normalize(&json!({ "background": raw })).background
}

#[test]
fn a_layout_without_a_background_has_none_and_serializes_without_the_key() {
    assert_eq!(ProfileLayout::default().background, None);
    let layout = ProfileLayout::normalize(&json!({ "tabs": [] }));
    assert_eq!(layout.background, None);
    assert!(
        serde_json::to_value(&layout)
            .unwrap()
            .get("background")
            .is_none()
    );
}

#[test]
fn a_background_keeps_its_kind_id_and_focus() {
    assert_eq!(
        background(
            json!({ "kind": "skin", "id": " char_002_amiya@epoque#4 ", "focus_x": 30, "focus_y": 12.4 })
        ),
        Some(ProfileBackground {
            kind: ProfileBackgroundKind::Skin,
            id: "char_002_amiya@epoque#4".into(),
            focus_x: Some(30),
            focus_y: Some(12),
            scale: None,
            elite: None,
        })
    );
    assert_eq!(
        background(json!({ "kind": "operator", "id": "char_003_kalts" })),
        Some(ProfileBackground {
            kind: ProfileBackgroundKind::Operator,
            id: "char_003_kalts".into(),
            focus_x: None,
            focus_y: None,
            scale: None,
            elite: None,
        })
    );
}

#[test]
fn a_gallery_background_keeps_its_case_sensitive_id_and_both_focus_axes() {
    let bg = background(
        json!({ "kind": "archive_pic", "id": " pic_rogue_1_KV1 ", "focus_x": 72, "focus_y": 40 }),
    );
    assert_eq!(
        bg,
        Some(ProfileBackground {
            kind: ProfileBackgroundKind::ArchivePic,
            id: "pic_rogue_1_KV1".into(),
            focus_x: Some(72),
            focus_y: Some(40),
            scale: None,
            elite: None,
        })
    );
    assert_eq!(
        serde_json::to_value(bg.unwrap()).unwrap(),
        json!({ "kind": "archive_pic", "id": "pic_rogue_1_KV1", "focus_x": 72, "focus_y": 40 })
    );
    assert_eq!(ProfileBackgroundKind::ArchivePic.entity_kind(), None);
    for kind in ProfileBackgroundKind::ALL {
        assert_eq!(ProfileBackgroundKind::from_id(kind.as_str()), Some(kind));
    }
    assert_eq!(
        background(json!({ "kind": "gallery", "id": "act13side_pic_0" })),
        None
    );
}

#[test]
fn a_story_art_background_keeps_its_id_as_sent_and_has_no_entity_kind() {
    for (wire, kind) in [
        ("story_cg", ProfileBackgroundKind::StoryCg),
        ("story_scene", ProfileBackgroundKind::StoryScene),
    ] {
        let bg = background(
            json!({ "kind": wire, "id": " avg_1_1 ", "focus_x": 20, "focus_y": 80, "scale": 140 }),
        );
        assert_eq!(
            bg,
            Some(ProfileBackground {
                kind,
                id: "avg_1_1".into(),
                focus_x: Some(20),
                focus_y: Some(80),
                scale: Some(140),
                elite: None,
            })
        );
        assert_eq!(
            serde_json::to_value(bg.unwrap()).unwrap(),
            json!({ "kind": wire, "id": "avg_1_1", "focus_x": 20, "focus_y": 80, "scale": 140 })
        );
        assert_eq!(kind.entity_kind(), None);
        assert_eq!(ProfileBackgroundKind::from_id(wire), Some(kind));
    }
    assert_eq!(
        background(json!({ "kind": "story_cg", "id": "AVG_1_1" })).map(|b| b.id),
        Some("AVG_1_1".into())
    );
    assert_eq!(
        background(json!({ "kind": "story_art", "id": "avg_1_1" })),
        None
    );
}

#[test]
fn a_bad_background_is_dropped_and_focus_is_clamped() {
    for raw in [
        json!(null),
        json!("char_002_amiya"),
        json!({ "kind": "story", "id": "main_0" }),
        json!({ "kind": "enemy", "id": "enemy_1007_slime" }),
        json!({ "id": "char_002_amiya" }),
        json!({ "kind": "operator" }),
        json!({ "kind": "operator", "id": "   " }),
        json!({ "kind": "operator", "id": 7 }),
        json!({ "kind": "operator", "id": "x".repeat(121) }),
    ] {
        assert_eq!(background(raw.clone()), None, "{raw}");
    }
    let bg = background(json!({ "kind": "operator", "id": "a", "focus_x": -40, "focus_y": 250 }))
        .unwrap();
    assert_eq!((bg.focus_x, bg.focus_y), (Some(0), Some(100)));
    let bg = background(json!({ "kind": "operator", "id": "a", "focus_x": "50", "focus_y": null }))
        .unwrap();
    assert_eq!((bg.focus_x, bg.focus_y), (None, None));
    let bg = background(json!({ "kind": "operator", "id": "a", "focus_x": 99.5, "focus_y": 0.49 }))
        .unwrap();
    assert_eq!((bg.focus_x, bg.focus_y), (Some(100), Some(0)));
}

#[test]
fn an_operator_background_keeps_elite_1_or_2_and_nothing_else_does() {
    let elite = |raw: Value| background(raw).unwrap().elite;
    assert_eq!(
        elite(json!({ "kind": "operator", "id": "a", "elite": 1 })),
        Some(1)
    );
    assert_eq!(
        elite(json!({ "kind": "operator", "id": "a", "elite": 2 })),
        Some(2)
    );
    for bad in [
        json!(0),
        json!(3),
        json!(1.5),
        json!("2"),
        json!(-1),
        json!(null),
    ] {
        assert_eq!(
            elite(json!({ "kind": "operator", "id": "a", "elite": bad })),
            None
        );
    }
    assert_eq!(elite(json!({ "kind": "operator", "id": "a" })), None);
    for kind in ["skin", "archive_pic", "story_cg", "story_scene"] {
        assert_eq!(
            elite(json!({ "kind": kind, "id": "a@b", "elite": 1 })),
            None
        );
    }
    let plain = background(json!({ "kind": "operator", "id": "a", "focus_y": 25 })).unwrap();
    assert_eq!(
        serde_json::to_value(plain).unwrap(),
        json!({ "kind": "operator", "id": "a", "focus_y": 25 })
    );
    let e1 = background(json!({ "kind": "operator", "id": "a", "elite": 1 })).unwrap();
    assert_eq!(
        serde_json::to_value(e1).unwrap(),
        json!({ "kind": "operator", "id": "a", "elite": 1 })
    );
}

#[test]
fn a_background_scale_is_rounded_clamped_and_absent_when_not_a_number() {
    let scale = |raw: Value| {
        background(json!({ "kind": "operator", "id": "a", "scale": raw }))
            .unwrap()
            .scale
    };
    assert_eq!(scale(json!(150)), Some(150));
    assert_eq!(scale(json!(149.5)), Some(150));
    assert_eq!(scale(json!(100)), Some(100));
    assert_eq!(scale(json!(300)), Some(300));
    assert_eq!(scale(json!(40)), Some(BACKGROUND_SCALE_MIN));
    assert_eq!(scale(json!(-7)), Some(BACKGROUND_SCALE_MIN));
    assert_eq!(scale(json!(1e9)), Some(BACKGROUND_SCALE_MAX));
    assert_eq!(scale(json!("150")), None);
    assert_eq!(scale(json!(null)), None);
    assert_eq!(scale(json!(true)), None);
    let bare = background(json!({ "kind": "operator", "id": "a" })).unwrap();
    assert_eq!(bare.scale, None);
    assert_eq!(
        serde_json::to_value(&bare).unwrap(),
        json!({ "kind": "operator", "id": "a" })
    );
    let zoomed = background(
        json!({ "kind": "archive_pic", "id": "p", "focus_x": 10, "focus_y": 90, "scale": 210 }),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&zoomed).unwrap(),
        json!({ "kind": "archive_pic", "id": "p", "focus_x": 10, "focus_y": 90, "scale": 210 })
    );
}

#[test]
fn a_background_round_trips_and_survives_the_visitor_projection() {
    let raw = json!({
        "tabs": [{ "id": "showcase", "visible": false }],
        "background": { "kind": "skin", "id": "char_002_amiya@epoque#4", "focus_x": 40, "focus_y": 20 },
    });
    let once = ProfileLayout::normalize(&raw);
    let wire = serde_json::to_value(&once).unwrap();
    assert_eq!(wire["background"], raw["background"]);
    assert_eq!(ProfileLayout::normalize(&wire), once);
    assert_eq!(once.visible_only().background, once.background);
}

#[test]
fn a_patch_carries_only_the_keys_sent() {
    let patch = ProfileLayoutPatch::normalize(
        &json!({ "background": { "kind": "operator", "id": "char_003_kalts" } }),
    );
    assert_eq!(patch.tabs, None);
    assert_eq!(patch.showcase, None);
    assert!(matches!(patch.background, Some(Some(_))));
    assert_eq!(
        patch.merge_json(),
        json!({ "background": { "kind": "operator", "id": "char_003_kalts" } })
    );
    assert!(patch.removed_keys().is_empty());

    for raw in [json!(7), json!({}), json!({ "tabs": "x", "showcase": [1] })] {
        assert_eq!(
            ProfileLayoutPatch::normalize(&raw),
            ProfileLayoutPatch::default(),
            "{raw}"
        );
    }

    let patch =
        ProfileLayoutPatch::normalize(&json!({ "tabs": [{ "id": "roster", "visible": false }] }));
    assert_eq!(
        patch.tabs.as_ref().map(Vec::len),
        Some(ProfileTabId::ALL.len())
    );
    assert_eq!(patch.background, None);

    for raw in [json!(null), json!({ "kind": "story", "id": "x" })] {
        let patch = ProfileLayoutPatch::normalize(&json!({ "background": raw }));
        assert_eq!(patch.background, Some(None), "{raw}");
        assert_eq!(patch.merge_json(), json!({}));
        assert_eq!(patch.removed_keys(), ["background"]);
    }
}

#[test]
fn a_background_save_keeps_tabs_and_showcase_and_null_clears_it() {
    let stored = json!({
        "tabs": [{ "id": "roster", "visible": false }],
        "showcase": { "blocks": [{ "type": "grid", "slug": "g" }] },
    });
    let set = ProfileLayoutPatch::normalize(
        &json!({ "background": { "kind": "skin", "id": "s@a#1", "focus_y": 30 } }),
    );
    let after = set.apply(Some(&stored)).unwrap();
    assert_eq!(after["tabs"], stored["tabs"]);
    assert_eq!(after["showcase"], stored["showcase"]);
    assert_eq!(
        after["background"],
        json!({ "kind": "skin", "id": "s@a#1", "focus_y": 30 })
    );

    let tabs = ProfileLayoutPatch::normalize(&json!({ "tabs": [] }));
    let kept = tabs.apply(Some(&after)).unwrap();
    assert_eq!(kept["background"], after["background"]);
    assert_eq!(kept["showcase"], stored["showcase"]);

    let clear = ProfileLayoutPatch::normalize(&json!({ "background": null }));
    let cleared = clear.apply(Some(&after)).unwrap();
    assert_eq!(cleared, stored);

    let alone = set.apply(None).unwrap();
    assert_eq!(alone.as_object().map(serde_json::Map::len), Some(1));
    assert_eq!(clear.apply(Some(&alone)), None);
    assert_eq!(
        ProfileLayout::normalize(&alone).tabs.len(),
        ProfileTabId::ALL.len()
    );
}

fn header(raw: Value) -> Option<ProfileHeader> {
    ProfileLayout::normalize(&json!({ "header": raw })).header
}

fn wire(fields: &[ProfileHeaderField]) -> Vec<&'static str> {
    fields.iter().map(|f| f.as_str()).collect()
}

#[test]
fn a_layout_without_a_header_has_none_and_serializes_without_the_key() {
    assert_eq!(ProfileLayout::default().header, None);
    let layout = ProfileLayout::normalize(&json!({ "tabs": [], "background": null }));
    assert_eq!(layout.header, None);
    assert_eq!(layout.visible_only().header, None);
    assert!(
        serde_json::to_value(&layout)
            .unwrap()
            .get("header")
            .is_none()
    );
    for field in ProfileHeaderField::ALL {
        assert!(layout.header_shows(field), "{}", field.as_str());
    }
}

#[test]
fn a_header_that_is_not_an_object_is_none_and_shapeless_lists_are_empty() {
    for raw in [
        json!(null),
        json!("originite"),
        json!(["originite"]),
        json!(3),
    ] {
        assert_eq!(header(raw.clone()), None, "{raw}");
    }
    assert_eq!(header(json!({})), Some(ProfileHeader::default()));
    assert_eq!(
        header(json!({ "hidden": "originite", "strip": { "0": "lmd" } })),
        Some(ProfileHeader::default())
    );
}

#[test]
fn hidden_drops_unknown_ids_and_repeats_and_keeps_canonical_order() {
    let got = header(json!({ "hidden": [
        "originite", "uid", "nickname", 7, null, "lmd", "originite", "level", " lmd",
    ] }))
    .unwrap();
    assert_eq!(wire(&got.hidden), ["level", "originite", "lmd"]);
    assert!(got.is_hidden(ProfileHeaderField::Originite));
    assert!(!got.is_hidden(ProfileHeaderField::Orundum));
}

#[test]
fn the_strip_drops_unknown_repeated_and_non_card_ids_and_is_cut_to_five() {
    let got = header(json!({ "strip": [
        "lmd", "registered", "lmd", "nope", "level", "originite", "sign_in",
        "ten_pull_permits", "operators", "skins", "items", "orundum",
    ] }))
    .unwrap();
    assert_eq!(
        wire(got.strip.as_deref().unwrap()),
        ["lmd", "originite", "ten_pull_permits", "operators", "skins"]
    );
    assert_eq!(got.strip.as_ref().map(Vec::len), Some(HEADER_STRIP_MAX));
    assert_eq!(header(json!({ "strip": [] })).unwrap().strip, Some(vec![]));
    assert_eq!(header(json!({ "strip": "lmd" })).unwrap().strip, None);
    assert_eq!(
        wire(&header(json!({})).unwrap().strip_cards()),
        ["operators", "skins", "items", "orundum", "originite"]
    );
}

#[test]
fn a_normalized_header_is_a_fixed_point_and_round_trips() {
    let raw = json!({
        "tabs": [{ "id": "roster", "visible": false }],
        "header": { "hidden": ["lmd", "originite", "lmd"], "strip": ["lmd", "x", "orundum"] },
    });
    let once = ProfileLayout::normalize(&raw);
    let wire_once = serde_json::to_value(&once).unwrap();
    assert_eq!(
        wire_once["header"],
        json!({ "hidden": ["originite", "lmd"], "strip": ["lmd", "orundum"] })
    );
    assert_eq!(ProfileLayout::normalize(&wire_once), once);
    let bare = header(json!({ "hidden": ["level"] })).unwrap();
    assert_eq!(
        serde_json::to_value(bare).unwrap(),
        json!({ "hidden": ["level"] })
    );
}

#[test]
fn the_visitor_header_resolves_the_strip_and_drops_hidden_cards() {
    let visitor = |raw: Value| {
        ProfileLayout::normalize(&json!({ "header": raw }))
            .visible_only()
            .header
            .unwrap()
    };
    let got = visitor(json!({ "hidden": ["originite", "level"] }));
    assert_eq!(
        wire(got.strip.as_deref().unwrap()),
        ["operators", "skins", "items", "orundum"]
    );
    assert_eq!(wire(&got.hidden), ["level", "originite"]);
    let got = visitor(json!({ "hidden": ["lmd"], "strip": ["lmd", "orundum"] }));
    assert_eq!(wire(got.strip.as_deref().unwrap()), ["orundum"]);
    let got = visitor(json!({ "hidden": ["lmd"], "strip": ["lmd"] }));
    assert_eq!(got.strip, Some(vec![]));
    assert_eq!(
        visitor(json!({})).strip.as_deref(),
        Some(&ProfileHeaderField::DEFAULT_STRIP[..])
    );
    let owner =
        ProfileLayout::normalize(&json!({ "header": { "hidden": ["lmd"], "strip": ["lmd"] } }));
    assert_eq!(
        owner.header.unwrap().strip,
        Some(vec![ProfileHeaderField::Lmd])
    );
}

#[test]
fn header_wire_ids_round_trip_and_the_sql_predicate_matches_the_stored_shape() {
    for field in ProfileHeaderField::ALL {
        assert_eq!(serde_json::to_value(field).unwrap(), json!(field.as_str()));
        assert_eq!(ProfileHeaderField::from_id(field.as_str()), Some(field));
    }
    assert!(
        ProfileHeaderField::DEFAULT_STRIP
            .iter()
            .all(|f| f.is_strip_card())
    );
    let stored = serde_json::to_value(ProfileHeader {
        hidden: vec![ProfileHeaderField::Originite],
        strip: None,
    })
    .unwrap();
    assert_eq!(stored["hidden"].to_string(), r#"["originite"]"#);
}

#[test]
fn a_private_tab_serializes_to_the_literal_the_sql_predicates_contain() {
    let stored = serde_json::to_string(&LayoutTab {
        id: ProfileTabId::Roster,
        visible: false,
    })
    .unwrap();
    assert_eq!(stored, r#"{"id":"roster","visible":false}"#);
}

#[test]
fn a_header_save_keeps_the_other_keys_and_theirs_keep_it() {
    let stored = json!({
        "tabs": [{ "id": "roster", "visible": false }],
        "showcase": { "blocks": [{ "type": "grid", "slug": "g" }] },
        "background": { "kind": "operator", "id": "char_003_kalts" },
    });
    let set = ProfileLayoutPatch::normalize(
        &json!({ "header": { "hidden": ["originite", "nope"], "strip": ["lmd"] } }),
    );
    assert_eq!(
        (
            set.tabs.as_ref(),
            set.showcase.as_ref(),
            set.background.as_ref()
        ),
        (None, None, None)
    );
    assert_eq!(
        set.merge_json(),
        json!({ "header": { "hidden": ["originite"], "strip": ["lmd"] } })
    );
    let after = set.apply(Some(&stored)).unwrap();
    for key in ["tabs", "showcase", "background"] {
        assert_eq!(after[key], stored[key], "{key}");
    }
    assert_eq!(after["header"]["hidden"], json!(["originite"]));

    for other in [
        json!({ "tabs": [] }),
        json!({ "showcase": { "blocks": [] } }),
        json!({ "background": null }),
        json!({ "background": { "kind": "skin", "id": "a@b#1" } }),
    ] {
        let kept = ProfileLayoutPatch::normalize(&other)
            .apply(Some(&after))
            .unwrap();
        assert_eq!(kept["header"], after["header"], "{other}");
        assert_eq!(
            ProfileLayoutPatch::normalize(&other).header,
            None,
            "{other}"
        );
    }

    for raw in [json!(null), json!(["originite"])] {
        let clear = ProfileLayoutPatch::normalize(&json!({ "header": raw }));
        assert_eq!(clear.header, Some(None), "{raw}");
        assert_eq!(clear.removed_keys(), ["header"]);
        assert_eq!(clear.apply(Some(&after)), Some(stored.clone()));
    }
    let alone = set.apply(None).unwrap();
    let clear = ProfileLayoutPatch::normalize(&json!({ "header": null }));
    assert_eq!(clear.apply(Some(&alone)), None);
    let both = ProfileLayoutPatch::normalize(&json!({ "header": null, "background": null }));
    assert_eq!(both.removed_keys(), ["background", "header"]);
}
