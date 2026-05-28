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

use base64::Engine;
use bson::Bson;

use crate::core::gamedata::types::gacha::GachaPoolClient;

pub fn enrich_banners(pools: &mut [GachaPoolClient]) {
    for pool in pools {
        let (f6, f5) = extract_featured(pool);
        pool.featured6 = f6;
        pool.featured5 = f5;
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
