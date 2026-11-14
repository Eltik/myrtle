#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::{HashMap, HashSet};

use myrtle_engine_release::{Switches, skins::*, types::Resolution};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::{
    character::{Brand, BrandGroup, Skin},
    item::{ListingKind, SkinListing, SkinWindow},
};

#[test]
fn reviews_pair_in_order_within_the_lag_band() {
    const D: i64 = 86_400;
    let cn = [
        (0, 27 * D),
        (90 * D, 117 * D),
        (180 * D, 207 * D),
        (270 * D, 297 * D),
    ];
    let en = [(180 * D, 208 * D), (272 * D, 300 * D)];
    let paired = myrtle_engine_release::skins::pair_reviews(&cn, &en);
    assert_eq!(paired[0], Some(en[0]), "180 d lag pairs");
    assert_eq!(
        paired[1],
        Some(en[1]),
        "182 d lag pairs with the next EN review"
    );
    assert_eq!(paired[2], None, "no EN review 90 to 270 d after it");
    assert_eq!(paired[3], None);
}

#[test]
fn review_pool_is_plain_price_brand_outfits_two_years_old() {
    let mut brands = HashMap::new();
    brands.insert(
        "crossover".to_owned(),
        Brand {
            brand_id: "crossover".into(),
            group_list: vec![BrandGroup {
                skin_group_id: "2021#rainbow6".into(),
                publish_time: 0,
            }],
            ..Default::default()
        },
    );
    let store = |group: &str, tag: Option<&str>| {
        let mut s = skin("a@x#1", group, 100 * D);
        s.display_skin.obtain_approach = Some("采购中心".into());
        s.display_skin.display_tag_id = tag.map(str::to_owned);
        s
    };
    assert!(review_eligible(&store("2021#epoque", None), &brands));
    assert!(
        !review_eligible(&store("2021#rainbow6", None), &brands),
        "a crossover group named by the brand roster"
    );
    assert!(
        !review_eligible(&store("2019#winter", Some("活动获得")), &brands),
        "the event-tagged store outfits never entered a review"
    );
    let mut dynamic = store("2021#epoque", None);
    dynamic.dyn_illust_id = Some("dyn".into());
    assert!(
        !review_eligible(&dynamic, &brands),
        "21 OP outfits rerun on their own"
    );
    assert_eq!(review_pool_cutoff(1000 * D), 270 * D);
}

const D: i64 = 86_400;

fn skin(id: &str, group: &str, get_time: i64) -> Skin {
    let mut s = Skin {
        skin_id: id.into(),
        ..Default::default()
    };
    s.display_skin.skin_group_id = group.into();
    s.display_skin.skin_group_name = format!("Coast/{}", group.trim_start_matches('g'));
    s.display_skin.get_time = get_time;
    s.display_skin.obtain_approach = Some("Store".into());
    s
}

const fn listing(start: i64, kind: ListingKind) -> SkinListing {
    SkinListing {
        start_time: start,
        end_time: start + 14 * D,
        tag_id: None,
        kind,
    }
}

fn game() -> GameModel {
    let mut gd = GameModel::default();
    for (id, g, t) in [
        ("a@x#1", "g1", 100 * D),
        ("b@x#1", "g1", 100 * D),
        ("c@x#2", "g2", 200 * D),
        ("d@x#3", "g3", 900 * D),
    ] {
        gd.skins.char_skins.insert(id.into(), skin(id, g, t));
    }
    gd.skins.brand_list.insert(
        "coast".into(),
        Brand {
            brand_id: "coast".into(),
            brand_name: "Coast".into(),
            brand_capital_name: "COAST".into(),
            group_list: vec![
                BrandGroup {
                    skin_group_id: "g1".into(),
                    publish_time: 0,
                },
                BrandGroup {
                    skin_group_id: "g2".into(),
                    publish_time: 0,
                },
                BrandGroup {
                    skin_group_id: "g3".into(),
                    publish_time: 0,
                },
            ],
            ..Default::default()
        },
    );
    gd.skin_listings = vec![
        listing(
            100 * D,
            ListingKind::Group {
                name: "Coast/1".into(),
                skin_ids: vec![],
                img_id: None,
            },
        ),
        listing(
            200 * D,
            ListingKind::Group {
                name: "whatever".into(),
                skin_ids: vec!["c@x#2".into()],
                img_id: None,
            },
        ),
        listing(900 * D, ListingKind::Review),
        listing(
            465 * D,
            ListingKind::Group {
                name: "Coast/I".into(),
                skin_ids: vec![],
                img_id: None,
            },
        ),
        listing(
            1000 * D,
            ListingKind::Group {
                name: "COAST".into(),
                skin_ids: vec![],
                img_id: None,
            },
        ),
    ];
    gd.skin_windows = vec![SkinWindow {
        skin_id: "a@x#1".into(),
        start_time: 100 * D,
        end_time: 114 * D,
    }];
    gd
}

#[test]
fn batch_families_need_an_orphan_on_both_servers() {
    let l = |name: &str, img: &str, start: i64| SkinListing {
        start_time: start,
        end_time: start + 14 * D,
        tag_id: None,
        kind: ListingKind::Group {
            name: name.into(),
            skin_ids: vec![],
            img_id: Some(img.into()),
        },
    };
    let mut en = game();
    en.skin_listings = vec![
        l("Multi-theme Outfit/I", "tag610_a", 100 * D),
        l("Multi-theme Outfit/II", "tag733_a", 200 * D),
        l("Coast/IX", "tag1_a", 300 * D), // a group base on EN
        l("Naming Variant/II", "tag9_a", 400 * D), // orphan on EN, a group on CN
    ];
    let mut cn = game();
    cn.skin_listings = vec![
        l("多主题时装/I", "tag610_a", 90 * D),
        l("多主题时装/VIII", "tag1035_a", 900 * D),
        l("Coast/II", "tag9_a", 390 * D),
    ];
    assert_eq!(
        batch_families(&en, &cn),
        std::iter::once("multi-theme outfit".to_string()).collect()
    );
    assert_eq!(
        batch_families(&cn, &en),
        std::iter::once("多主题时装".to_string()).collect()
    );
    assert_eq!(name_base("epoque/xxxiv"), "epoque");
    assert_eq!(name_base("rhodes fashion review"), "rhodes fashion review");
}

#[test]
fn names_normalise_numerals_and_case() {
    assert_eq!(normalize_name("Coral Coast/1"), "coral coast/i");
    assert_eq!(normalize_name("Coral Coast/IX"), "coral coast/ix");
    assert_eq!(normalize_name(" EPOQUE/34 "), "epoque/xxxiv");
    assert_eq!(normalize_name("Test Collection"), "test collection");
}

#[test]
fn histories_merge_every_source() {
    let (gs, _) = group_histories(&game(), &HashSet::new(), Switches::SHIPPED);
    let g1 = gs.iter().find(|g| g.skin_group_id == "g1").unwrap();
    let g2 = gs.iter().find(|g| g.skin_group_id == "g2").unwrap();
    let g3 = gs.iter().find(|g| g.skin_group_id == "g3");
    let kinds = |g: &GroupHistory| {
        g.windows
            .iter()
            .map(|w| (w.start_time / D, w.kind))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        kinds(g1),
        vec![
            (100, SaleKind::Listing),
            (465, SaleKind::Listing),
            (900, SaleKind::Review),
            (1000, SaleKind::Listing)
        ]
    );
    assert_eq!(
        kinds(g2),
        vec![(200, SaleKind::Listing)],
        "a bare brand name is group one, not the whole brand; and at 200 d \
             g2 is too young for the review at 900 d (pool cutoff 170 d)"
    );
    assert!(g3.is_none(), "never listed, so no history");
    assert_eq!(g1.last_seen(), Some(1000 * D));
}

#[test]
fn a_skipped_listing_leaves_no_window_behind() {
    let mut gd = game();
    gd.skin_listings[3].tag_id = Some("tag_7".into());
    let skip = |l: &SkinListing| l.tag_id.as_deref() == Some("tag_7");
    let (all, _) = group_histories(&gd, &HashSet::new(), Switches::SHIPPED);
    let (kept, _) = group_histories_without(&gd, &HashSet::new(), Switches::SHIPPED, skip);
    let g1 = |gs: &[GroupHistory]| gs.iter().find(|g| g.skin_group_id == "g1").cloned();
    assert!(listed_near(g1(&all).as_ref(), 465 * D));
    assert!(
        !listed_near(g1(&kept).as_ref(), 465 * D),
        "Coast/I at 465 d came only from the skipped listing"
    );
    assert!(
        listed_near(g1(&kept).as_ref(), 100 * D),
        "the carousel stays"
    );
    assert!(!listed_near(None, 100 * D));
}

fn history(windows: &[(i64, SaleKind)]) -> GroupHistory {
    GroupHistory {
        skin_group_id: "g".into(),
        windows: windows
            .iter()
            .map(|&(start, kind)| SaleWindow {
                start_time: start * D,
                end_time: start * D + 14 * D,
                kind,
            })
            .collect(),
        ..Default::default()
    }
}

const fn pooled() -> CadenceModel {
    CadenceModel {
        n: 40,
        median_days: 343.0,
        p25_days: 150.0,
        p75_days: 400.0,
    }
}

#[test]
fn own_cadence_uses_the_last_own_gap_with_the_pooled_spread() {
    // 100 -> 280 (180 d) -> 290 merges into 280 -> 463 (183 d, the last gap).
    let g = history(&[
        (100, SaleKind::Listing),
        (280, SaleKind::Listing),
        (290, SaleKind::Listing),
        (463, SaleKind::Listing),
    ]);
    let (en_last, r, gap) = next_by_own_cadence(&g, &pooled()).unwrap();
    assert_eq!(en_last, 463 * D);
    assert_eq!(gap, Some(183.0));
    assert_eq!(
        r,
        Resolution::Estimated {
            en_start: 646 * D,
            lo: (646 - 193) * D,
            hi: (646 + 57) * D,
        }
    );
}

#[test]
fn own_cadence_falls_back_to_pooled_without_an_own_gap() {
    let g = history(&[(100, SaleKind::Listing), (110, SaleKind::Listing)]);
    let (en_last, r, gap) = next_by_own_cadence(&g, &pooled()).unwrap();
    assert_eq!(en_last, 100 * D);
    assert_eq!(gap, None);
    assert_eq!(r, next_by_cadence(100 * D, &pooled()));
    assert!(next_by_own_cadence(&history(&[]), &pooled()).is_none());
}

#[test]
fn own_cadence_clamps_the_own_gap_to_the_pooled_band() {
    let long = history(&[(100, SaleKind::Listing), (1100, SaleKind::Listing)]);
    let (_, r, gap) = next_by_own_cadence(&long, &pooled()).unwrap();
    assert_eq!(gap, Some(400.0));
    assert!(matches!(r, Resolution::Estimated { en_start, .. } if en_start == 1500 * D));
    let short = history(&[(100, SaleKind::Listing), (200, SaleKind::Listing)]);
    assert_eq!(
        next_by_own_cadence(&short, &pooled()).unwrap().2,
        Some(150.0)
    );
}

#[test]
fn own_cadence_never_anchors_on_a_review_window() {
    let g = history(&[
        (100, SaleKind::Listing),
        (280, SaleKind::Listing),
        (700, SaleKind::Review),
    ]);
    assert_eq!(g.last_seen(), Some(700 * D));
    let (en_last, r, gap) = next_by_own_cadence(&g, &pooled()).unwrap();
    assert_eq!(en_last, 280 * D);
    assert_eq!(gap, Some(180.0));
    assert!(matches!(r, Resolution::Estimated { en_start, .. } if en_start == 460 * D));
}
