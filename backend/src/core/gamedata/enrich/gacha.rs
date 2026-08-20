//! Decode the base64-wrapped BSON blobs that Yostar emits on each
//! `GachaPoolClient` (in `LimitParam` and `DynMeta`) into structured
//! lists of featured 5★/6★ `char_ids`.
//!
//! The blobs are standard `MongoDB` BSON. Their shape varies by
//! `gacha_rule_type`:
//!
//! | Rule              | Field      | Where the `char_ids` live
//! |-------------------|------------|----------------------------------------------------
//! | LIMITED           | `LimitParam` | `limitedCharId` (one 6★)
//! | CLASSIC           | `DynMeta`    | `main6RarityCharId` + `sub6RarityCharId`, `rare5CharList`
//! | `CLASSIC_DOUBLE`    | `DynMeta`    | same as CLASSIC
//! | `CLASSIC_ATTAIN`    | `DynMeta`    | `attainRare6CharList`
//! | ATTAIN            | `LimitParam` | `attainRare6CharList`
//! | FESCLASSIC        | `DynMeta`    | `rarityPickCharDict.TIER_5` + `TIER_6`
//! | SPECIAL           | `DynMeta`    | same as FESCLASSIC
//! | LINKAGE / SINGLE / NORMAL / DOUBLE | (no blob - rate-ups come from elsewhere)
//!
//! Decoding is best-effort: a malformed blob never errors out, it just
//! leaves the banner's featured lists empty (the UI gracefully falls back).
//!
//! # The static blobs are not the whole story
//!
//! On several rule types the blob is a strict *subset* of the real rate-up set.
//! A LIMITED banner's `limitedCharId` names one 6*, for instance, while the
//! banner may also feature a co-featured standard 6* and several rate-boosted
//! ones. Those exist only in the `gacha/getPoolDetail` response, which
//! the pool-detail refresh job caches to a sidecar.
//!
//! When that sidecar is present, [`enrich_banners`] overlays it on top of the
//! blob-derived lists. The merge is a union, never a replacement: FESCLASSIC,
//! SPECIAL and `CLASSIC_ATTAIN` return an *empty* `upCharInfo` and are carried
//! entirely by the blobs, so replacing would lose data. See
//! [`GachaPoolDetail`] for why.

use base64::Engine;
use bson::Bson;

use crate::core::gamedata::types::gacha::GachaPoolClient;
use crate::core::gamedata::types::gacha_detail::{
    GachaPoolDetail, PoolDetailFile, RARITY_RANK_5, RARITY_RANK_6, RarityRate,
};

/// Populate each banner's derived fields.
///
/// `details` is the optional `gacha/getPoolDetail` sidecar. Passing `None`
/// reproduces the blob-only behaviour exactly, which is what every offline
/// path (tests, the `resync-gacha` bin, a fresh checkout with no service
/// account) gets.
pub fn enrich_banners(pools: &mut [GachaPoolClient], details: Option<&PoolDetailFile>) {
    for pool in pools {
        let (f6, f5) = extract_featured(pool);
        pool.featured6 = f6;
        pool.featured5 = f5;
        pool.featured_source = "static".to_owned();

        if let Some(detail) = details.and_then(|d| d.pools.get(&pool.gacha_pool_id)) {
            overlay_pool_detail(pool, detail);
        }
    }
}

/// Merge one `gacha/getPoolDetail` body into an already blob-enriched banner.
fn overlay_pool_detail(pool: &mut GachaPoolClient, detail: &GachaPoolDetail) {
    let info = &detail.detail_info;
    let mut used_api = false;

    // The rate-ups proper. Union rather than replace - the blob-derived entry
    // is a subset here, and empty on the rule types the blobs own outright.
    if let Some(up) = info.up_char_info.as_ref() {
        for entry in &up.per_char_list {
            let target = match entry.rarity_rank {
                RARITY_RANK_6 => &mut pool.featured6,
                RARITY_RANK_5 => &mut pool.featured5,
                _ => continue,
            };
            for char_id in &entry.char_id_list {
                if !char_id.is_empty() {
                    push_unique(target, char_id);
                    used_api = true;
                }
            }
        }
    }

    if let Some(weights) = info.weight_up_char_info_list.as_ref()
        && !weights.is_empty()
    {
        pool.weight_up = weights.clone();
        used_api = true;
    }

    if let Some(avail) = info.avail_char_info.as_ref()
        && !avail.per_avail_list.is_empty()
    {
        pool.avail_rates = avail
            .per_avail_list
            .iter()
            .map(|e| RarityRate {
                rarity_rank: e.rarity_rank,
                total_percent: e.total_percent,
                pool_size: e.char_id_list.len(),
            })
            .collect();

        // On player-pick banners the 6* "available" list IS the candidate set
        // (six operators, not the ~54 of a normal pool), so it is worth keeping
        // whole. Everywhere else it is the entire pool, so only counts are kept.
        if detail.is_pickup()
            && let Some(six) = avail
                .per_avail_list
                .iter()
                .find(|e| e.rarity_rank == RARITY_RANK_6)
        {
            pool.pickup6 = six.char_id_list.clone();
        }
        used_api = true;
    }

    if used_api {
        pool.featured_source = "static+api".to_owned();
    }
}

fn extract_featured(pool: &GachaPoolClient) -> (Vec<String>, Vec<String>) {
    let limit = pool.limit_param.as_ref().and_then(decode_blob);
    let dyn_meta = pool.dyn_meta.as_ref().and_then(decode_blob);

    let mut f6: Vec<String> = Vec::new();
    let mut f5: Vec<String> = Vec::new();

    // LIMITED's `limitedCharId` lives in LimitParam; everything else lives in
    // DynMeta. We probe both regardless of rule_type - Yostar has shuffled
    // fields across rule types in the past, and the cost of a missing-key
    // lookup is negligible.
    for doc in [&limit, &dyn_meta].iter().copied().flatten() {
        if let Some(s) = doc.get_str("limitedCharId").ok().filter(|s| !s.is_empty()) {
            push_unique(&mut f6, s);
        }
        if let Some(s) = doc
            .get_str("main6RarityCharId")
            .ok()
            .filter(|s| !s.is_empty())
        {
            push_unique(&mut f6, s);
        }
        if let Some(s) = doc
            .get_str("sub6RarityCharId")
            .ok()
            .filter(|s| !s.is_empty())
        {
            push_unique(&mut f6, s);
        }
        collect_str_array(doc, "attainRare6CharList", &mut f6);
        collect_str_array(doc, "rare5CharList", &mut f5);

        // FESCLASSIC / SPECIAL nest tiered char lists. The TIER_5 array on
        // these pools is the *full selectable pool* (~24 ops), not just the
        // active rate-ups, but it's still useful for the hover card.
        if let Ok(dict) = doc.get_document("rarityPickCharDict") {
            collect_str_array(dict, "TIER_6", &mut f6);
            collect_str_array(dict, "TIER_5", &mut f5);
        }
    }

    (f6, f5)
}

fn decode_blob(value: &serde_json::Value) -> Option<bson::Document> {
    let b64 = value.get("Base64").and_then(|v| v.as_str())?;
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    bson::Document::from_reader(bytes.as_slice()).ok()
}

fn collect_str_array(doc: &bson::Document, key: &str, out: &mut Vec<String>) {
    let Ok(arr) = doc.get_array(key) else { return };
    for v in arr {
        if let Bson::String(s) = v
            && !s.is_empty()
        {
            push_unique(out, s);
        }
    }
}

fn push_unique(out: &mut Vec<String>, s: &str) {
    if !out.iter().any(|existing| existing == s) {
        out.push(s.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::gacha_detail::GachaPoolDetail;

    /// A LIMITED banner's `LimitParam.Base64`, decoding to
    /// `{hasFreeChar, freeCount, limitedCharId, version}` - a single 6*.
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
}
