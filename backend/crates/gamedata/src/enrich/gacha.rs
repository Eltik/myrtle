use std::collections::HashSet;

use myrtle_gamedata_raw::tables::{
    gacha::{GachaPoolClient, RarityRate},
    gacha_detail::{GachaPoolDetail, PoolDetailFile, RARITY_RANK_5, RARITY_RANK_6},
};

use base64::Engine;
use bson::Bson;

pub fn enrich_banners(pools: &mut [GachaPoolClient], details: Option<&PoolDetailFile>) {
    for pool in pools {
        let (f6, f5) = extract_featured(pool);
        pool.featured6 = f6;
        pool.featured5 = f5;
        pool.featured_source = "static".to_owned();
        pool.linkage_loop_at = linkage_loop_at(pool);

        if let Some(detail) = details.and_then(|d| d.pools.get(&pool.gacha_pool_id)) {
            overlay_pool_detail(pool, detail);
        }
    }
}

fn extract_featured(pool: &GachaPoolClient) -> (Vec<String>, Vec<String>) {
    let limit = pool.limit_param.as_ref().and_then(decode_blob);
    let dyn_meta = pool.dyn_meta.as_ref().and_then(decode_blob);

    let mut f6: Vec<String> = Vec::new();
    let mut f5: Vec<String> = Vec::new();

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

fn push_unique(out: &mut Vec<String>, s: &str) {
    if !out.iter().any(|existing| existing == s) {
        out.push(s.to_owned());
    }
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

fn linkage_loop_at(pool: &GachaPoolClient) -> Option<u32> {
    let doc = pool.linkage_param.as_ref().and_then(decode_blob)?;
    let n = match doc.get("loopTarget6Count")? {
        Bson::Int32(n) => i64::from(*n),
        Bson::Int64(n) => *n,
        _ => return None,
    };
    u32::try_from(n).ok().filter(|n| *n > 0)
}

fn overlay_pool_detail(pool: &mut GachaPoolClient, detail: &GachaPoolDetail) {
    let info = &detail.detail_info;
    let mut used_api = false;

    if let Some(up) = info.up_char_info.as_ref() {
        pool.declared_share6 = up
            .per_char_list
            .iter()
            .find(|e| e.rarity_rank == RARITY_RANK_6 && !e.char_id_list.is_empty())
            .and_then(|e| share_fraction(e.percent));
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

fn share_fraction(percent: f64) -> Option<f64> {
    let share = if percent > 1.0 {
        percent / 100.0
    } else {
        percent
    };
    (share.is_finite() && share > 0.0 && share <= 1.0).then_some(share)
}

pub fn mark_returning_pools(pools: &mut [GachaPoolClient], returning: &HashSet<String>) {
    for pool in pools {
        pool.returning = returning.contains(&pool.gacha_pool_id);
    }
}
