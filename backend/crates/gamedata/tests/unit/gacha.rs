use base64::Engine;
use bson::Bson;
use myrtle_gamedata_raw::tables::{
    gacha::GachaPoolClient,
    gacha_detail::{GachaPoolDetail, PoolDetailFile},
};

use crate::enrich::gacha::*;

/// A LIMITED banner's `LimitParam.Base64`, decoding to
/// `{hasFreeChar, freeCount, limitedCharId, version}`: a single 6*.
const LIMITED_BLOB: &str = "WwAAAAhoYXNGcmVlQ2hhcgABEmZyZWVDb3VudAAsAQAAAAAAAAJsaW1pdGVkQ2hhcklkABEAAABjaGFyXzEwNDVfc3Zhc2gyABJ2ZXJzaW9uAAEAAAAAAAAAAA==";

fn pool(id: &str, rule: &str, limit_param: Option<&str>) -> GachaPoolClient {
    GachaPoolClient {
        gacha_pool_id: id.to_owned(),
        gacha_rule_type: rule.to_owned(),
        limit_param: limit_param.map(|b| serde_json::json!({ "Base64": b })),
        ..Default::default()
    }
}

fn file(id: &str, detail: serde_json::Value) -> PoolDetailFile {
    let detail: GachaPoolDetail = serde_json::from_value(detail).expect("detail deserializes");
    let mut f = PoolDetailFile::default();
    f.pools.insert(id.to_owned(), detail);
    f
}

#[test]
fn only_granted_pools_are_returning() {
    let mut pools = vec![
        pool("RETURN_EN_41_0_1", "BACKFLOW", None),
        pool("NORM_EN_41_0_1", "NORMAL", None),
    ];
    let granted = std::iter::once("RETURN_EN_41_0_1".to_owned()).collect();
    mark_returning_pools(&mut pools, &granted);

    assert!(pools[0].returning);
    assert!(!pools[1].returning);
}

/// Without a sidecar, behaviour is exactly the pre-existing blob decode.
#[test]
fn without_details_only_the_blob_is_used() {
    let mut pools = vec![pool("LIMITED_EN_39_0_1", "LIMITED", Some(LIMITED_BLOB))];
    enrich_banners(&mut pools, None);

    assert_eq!(pools[0].featured6, vec!["char_1045_svash2"]);
    assert!(pools[0].featured5.is_empty());
    assert!(pools[0].weight_up.is_empty());
    assert_eq!(pools[0].featured_source, "static");
}

/// A LIMITED banner's blob names one 6*, but the banner also features a
/// co-featured 6*, a 5*, and three rate-boosted 6*.
#[test]
fn limited_banner_gains_the_cofeatured_operators() {
    let id = "LIMITED_EN_39_0_1";
    let details = file(
        id,
        serde_json::json!({
            "detailInfo": {
                "upCharInfo": { "perCharList": [
                    { "rarityRank": 5, "percent": 0.35, "count": 2,
                      "charIdList": ["char_1045_svash2", "char_1046_sbell2"] },
                    { "rarityRank": 4, "percent": 0.5, "count": 1,
                      "charIdList": ["char_4211_snhunt"] }
                ]},
                "weightUpCharInfoList": [
                    { "charId": "char_245_cello",  "rarityRank": 5, "weight": 500 },
                    { "charId": "char_1035_wisdel", "rarityRank": 5, "weight": 500 },
                    { "charId": "char_1038_whitw2", "rarityRank": 5, "weight": 500 }
                ],
                "limitedChar": ["char_1045_svash2"],
                "availCharInfo": { "perAvailList": [
                    { "rarityRank": 5, "totalPercent": 0.02, "charIdList": ["a", "b", "c"] },
                    { "rarityRank": 4, "totalPercent": 0.08, "charIdList": ["d"] }
                ]},
                "gachaObjList": [
                    { "gachaObject": "UP_CHAR_WITH_LIMIT", "imageType": 0, "param": null, "type": 0 }
                ]
            },
            "hasRateUp": false
        }),
    );

    let mut pools = vec![pool(id, "LIMITED", Some(LIMITED_BLOB))];
    enrich_banners(&mut pools, Some(&details));
    let p = &pools[0];

    // The blob-derived operator is kept; the co-featured one is added.
    assert_eq!(p.featured6, vec!["char_1045_svash2", "char_1046_sbell2"]);
    assert_eq!(p.featured5, vec!["char_4211_snhunt"]);

    // Rate-boosted operators stay out of the rate-up list.
    assert_eq!(p.weight_up.len(), 3);
    assert!(!p.featured6.contains(&"char_245_cello".to_owned()));

    assert_eq!(p.avail_rates.len(), 2);
    assert!((p.avail_rates[0].total_percent - 0.02).abs() < f64::EPSILON);
    assert_eq!(p.avail_rates[0].pool_size, 3);

    assert!(p.pickup6.is_empty(), "not a pickup banner");
    assert_eq!(p.featured_source, "static+api");
    assert_eq!(p.declared_share6, Some(0.35), "per operator, not per band");
}

/// FESCLASSIC returns an EMPTY `upCharInfo`; the blobs own this rule type.
/// A merge that replaced instead of unioning would silently drop the list.
#[test]
fn empty_up_char_info_does_not_clobber_blob_data() {
    let id = "FESCLASSIC_EN_39_0_1";
    // rarityPickCharDict.TIER_6 = ["char_test_six"], TIER_5 = ["char_test_five"]
    let mut doc = bson::Document::new();
    let mut dict = bson::Document::new();
    dict.insert("TIER_6", vec![Bson::String("char_test_six".into())]);
    dict.insert("TIER_5", vec![Bson::String("char_test_five".into())]);
    doc.insert("rarityPickCharDict", dict);
    let mut bytes = Vec::new();
    doc.to_writer(&mut bytes).unwrap();
    let blob = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let details = file(
        id,
        serde_json::json!({
            "detailInfo": {
                "upCharInfo": { "perCharList": [] },
                "availCharInfo": { "perAvailList": [
                    { "rarityRank": 5, "totalPercent": 0.02, "charIdList": ["x"] }
                ]},
                "gachaObjList": [
                    { "gachaObject": "FES_CLASSIC_UP_CHAR", "imageType": 0, "param": null, "type": 0 }
                ]
            }
        }),
    );

    let mut p = pool(id, "FESCLASSIC", None);
    p.dyn_meta = Some(serde_json::json!({ "Base64": blob }));
    let mut pools = vec![p];
    enrich_banners(&mut pools, Some(&details));

    assert_eq!(pools[0].featured6, vec!["char_test_six"]);
    assert_eq!(pools[0].featured5, vec!["char_test_five"]);
}

/// On a player-pick banner the 6* "available" list is the candidate set, so
/// it is kept whole. Detection is by layout discriminator, not rule type.
#[test]
fn pickup_banner_keeps_the_candidate_six_stars() {
    let id = "SPECIAL_EN_39_0_3";
    let details = file(
        id,
        serde_json::json!({
            "detailInfo": {
                "upCharInfo": { "perCharList": [] },
                "availCharInfo": { "perAvailList": [
                    { "rarityRank": 5, "totalPercent": 0.02,
                      "charIdList": ["char_4072_ironmn", "char_1042_phatm2"] },
                    { "rarityRank": 4, "totalPercent": 0.08, "charIdList": ["z"] }
                ]},
                "gachaObjList": [
                    { "gachaObject": "SPECIAL_PICKUP_SELECT_CHAR", "imageType": 0, "param": null, "type": 0 }
                ]
            }
        }),
    );

    let mut pools = vec![pool(id, "SPECIAL", None)];
    enrich_banners(&mut pools, Some(&details));

    assert_eq!(
        pools[0].pickup6,
        vec!["char_4072_ironmn", "char_1042_phatm2"]
    );
}

/// A sidecar that knows nothing about this pool must be inert.
#[test]
fn unknown_pool_is_left_alone() {
    let details = file("SOME_OTHER_POOL", serde_json::json!({ "detailInfo": {} }));
    let mut pools = vec![pool("LIMITED_EN_39_0_1", "LIMITED", Some(LIMITED_BLOB))];
    enrich_banners(&mut pools, Some(&details));

    assert_eq!(pools[0].featured6, vec!["char_1045_svash2"]);
    assert_eq!(pools[0].featured_source, "static");
}

/// `LINKAGE_74_0_1`'s `LinkageParam` (`loopTarget6Count: 120`) against
/// `LINKAGE_17_0_1`'s (`guaranteeTarget6Count: 120`, once only).
#[test]
fn only_the_loop_rule_repeats_the_collab_handover() {
    let blob = |b: &str| Some(serde_json::json!({ "Base64": b }));
    let mut looping = pool("LINKAGE_74_0_1", "LINKAGE", None);
    looping.linkage_param = blob(
        "SgAAABJsb29wVGFyZ2V0NkNvdW50AHgAAAAAAAAAAmd1YXJhbnRlZVRhcmdldDZDaGFyABEAAABjaGFyXzEwNDhfb3JjaGQyAAA=",
    );
    let mut once = pool("LINKAGE_17_0_1", "LINKAGE", None);
    once.linkage_param = blob("JAAAABJndWFyYW50ZWVUYXJnZXQ2Q291bnQAeAAAAAAAAAAA");
    let mut pools = vec![looping, once];
    enrich_banners(&mut pools, None);

    assert_eq!(pools[0].linkage_loop_at, Some(120));
    assert_eq!(pools[1].linkage_loop_at, None);
    assert_eq!(
        pools[0].declared_share6, None,
        "no sidecar, no declared share"
    );
}

#[test]
fn a_percentage_share_is_read_as_a_fraction() {
    assert_eq!(share_fraction(0.35), Some(0.35));
    assert_eq!(share_fraction(35.0), Some(0.35));
    assert_eq!(share_fraction(0.0), None);
    assert_eq!(share_fraction(f64::NAN), None);
}
