use std::collections::{HashMap, HashSet};

use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        gamedata::types::gacha::GachaPoolClient,
        hypergryph::constants::Server,
        release::{
            AlignMethod, BannersResponse, PoolAlignment, ReleaseBanner, Resolution,
            STANDING_POOL_SECS, align, estimate, ledger, override_name, resolve,
        },
        translate::{self, AutoName, AutoNameSource, TranslationMemory},
    },
    database::queries::release as q,
};

use super::{Names, Planner, cache_key, resolved_start};

async fn debut_links(state: &AppState, p: &Planner) -> HashMap<String, Vec<String>> {
    let mut debut: HashMap<String, Vec<String>> = HashMap::new();
    match q::debut_pairs(&state.db, ledger::ledger_server(Server::CN).index() as i16).await {
        Ok(pairs) => {
            for (pool_id, char_id) in pairs {
                if !p.ctx.en.operators.contains_key(&char_id) {
                    debut.entry(pool_id).or_default().push(char_id);
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "release ledger debut query failed"),
    }
    debut
}

pub async fn get_banners(state: &AppState) -> Result<BannersResponse, ApiError> {
    let key = cache_key("release:banners", estimate::window_from_env());
    if let Some(c) = state.cache.get::<BannersResponse>(&key).await {
        return Ok(c);
    }
    let p = Planner::load(state).await?;
    let mut debut = debut_links(state, &p).await;
    let (cn, en) = (&*p.ctx.cn, &*p.ctx.en);
    let aligned = align::align_pools(cn, en, &p.models.pairs, p.window);
    let rule_independence = align::rule_independence(cn, en);
    let independent: HashSet<&str> = rule_independence
        .iter()
        .filter(|r| r.independent)
        .map(|r| r.rule_type.as_str())
        .collect();
    let names = Names::new(&p, TranslationMemory::build(cn, en, &aligned));
    let en_pools: HashMap<&str, &GachaPoolClient> = en
        .gacha
        .gacha_pool_client
        .iter()
        .map(|p| (p.gacha_pool_id.as_str(), p))
        .collect();
    let shared: HashSet<&str> = cn
        .activities
        .keys()
        .filter(|id| en.activities.contains_key(*id))
        .map(String::as_str)
        .collect();
    let anchors = align::anchors(&cn.gacha.gacha_pool_client, cn, &shared);
    let mut char_names: HashMap<String, AutoName> = HashMap::new();
    let mut name_char = |id: &str| {
        if !char_names.contains_key(id)
            && let Some(n) = translate::operator_name(cn, en, id)
        {
            char_names.insert(id.to_string(), n);
        }
    };
    let since = p.since();

    let mut banners: Vec<ReleaseBanner> = cn
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|pool| pool.open_time > 0 && pool.open_time >= since)
        .map(|pool| {
            let rule_independent = independent.contains(pool.gacha_rule_type.as_str());
            let pair = aligned.get(&pool.gacha_pool_id);
            let en_pool = pair.and_then(|pr| en_pools.get(pr.en_pool_id.as_str()).copied());
            let anchor = if rule_independent {
                None
            } else {
                anchors.get(pool.gacha_pool_id.as_str()).copied()
            };
            let anchor_type = anchor
                .and_then(|a| cn.activities.get(a))
                .map_or("", |a| a.activity_type.as_str());
            let mut resolution = resolve(
                ledger::KIND_POOL,
                &pool.gacha_pool_id,
                en_pool.map(|e| (e.gacha_pool_id.as_str(), e.open_time, e.end_time)),
                &names.ov,
                p.models.for_type(anchor_type),
                pool.open_time,
            );
            if rule_independent
                && matches!(
                    resolution,
                    Resolution::Estimated { .. } | Resolution::Unmodelled
                )
            {
                resolution = Resolution::Independent;
            }
            let debut_chars = debut.remove(&pool.gacha_pool_id).unwrap_or_default();
            let override_featured: Vec<String> = names
                .ov
                .get(&(ledger::KIND_POOL.to_string(), pool.gacha_pool_id.clone()))
                .and_then(|o| o.featured_chars.clone())
                .unwrap_or_default();
            for id in pool
                .featured6
                .iter()
                .chain(&pool.featured5)
                .chain(&debut_chars)
                .chain(&override_featured)
                .chain(en_pool.iter().flat_map(|e| e.featured6.iter()))
            {
                name_char(id);
            }
            let name_en_auto = en_pool
                .map(|e| AutoName {
                    text: e.gacha_pool_name.clone(),
                    source: AutoNameSource::Memory,
                })
                .or_else(|| translate::resolve(&names.memory, &pool.gacha_pool_name))
                .or_else(|| override_name(ledger::KIND_POOL, &pool.gacha_pool_id, &names.ov));
            ReleaseBanner {
                cn_pool_id: pool.gacha_pool_id.clone(),
                rule_type: pool.gacha_rule_type.clone(),
                name_cn: pool.gacha_pool_name.clone(),
                cn_open: pool.open_time,
                cn_end: pool.end_time,
                standing: pool.end_time - pool.open_time > STANDING_POOL_SECS,
                featured6: pool.featured6.clone(),
                featured5: pool.featured5.clone(),
                debut_chars,
                anchor_activity: anchor.map(str::to_string),
                alignment: PoolAlignment {
                    en_pool_id: pair.map(|pr| pr.en_pool_id.clone()),
                    method: pair.map_or(AlignMethod::None, |pr| pr.method),
                },
                en_featured6: en_pool.map(|e| e.featured6.clone()).unwrap_or_default(),
                override_featured,
                name_en_auto,
                image_path: p.ctx.banner_image(
                    en_pool.map(|e| e.gacha_pool_id.as_str()),
                    &pool.gacha_pool_id,
                ),
                resolution,
            }
        })
        .collect();
    banners.sort_by(|a, b| {
        resolved_start(&a.resolution)
            .cmp(&resolved_start(&b.resolution))
            .then(a.cn_open.cmp(&b.cn_open))
            .then(a.cn_pool_id.cmp(&b.cn_pool_id))
    });
    let out = BannersResponse {
        model: p.models.general.clone(),
        yearly: p.models.yearly.clone(),
        banners,
        char_names,
        rule_independence,
    };
    state.cache.set(&key, &out).await;
    Ok(out)
}
