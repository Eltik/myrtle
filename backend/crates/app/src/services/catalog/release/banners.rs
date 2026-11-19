use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use myrtle_db::repos::release;
use myrtle_engine_release::event_banner;
use myrtle_engine_release::kinds::KIND_POOL;
use myrtle_engine_release::translate::{self, AutoName, TranslationMemory};
use myrtle_engine_release::{
    AlignMethod, BannersResponse, PoolAlignment, ReleaseBanner, Resolution, align, override_name,
    resolve,
};
use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;
use myrtle_types::Server;
use tokio::sync::Semaphore;

use super::{Planner, cached, resolved_start};
use crate::services::story::{modified, with_default_fallback, write_atomically};
use crate::{AppContext, AppError};

/// Pool id to the operators that debuted with it and EN has not shipped; a failed read is no links
async fn debut_links(ctx: &AppContext, p: &Planner) -> HashMap<String, Vec<String>> {
    let pairs = match ctx.db.conn().await {
        Ok(mut conn) => release::debut_pairs(&mut conn, Server::CN).await,
        Err(e) => Err(e),
    };
    let mut debut: HashMap<String, Vec<String>> = HashMap::new();
    match pairs {
        Ok(pairs) => {
            for (pool_id, char_id) in pairs {
                if !p.en().operators.contains_key(&char_id) {
                    debut.entry(pool_id).or_default().push(char_id);
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "release ledger debut query failed"),
    }
    debut
}

pub async fn get_banners(ctx: &AppContext) -> Result<BannersResponse, AppError> {
    cached(ctx, "banners", |ctx, p| async move {
        let debut = debut_links(&ctx, &p).await;
        ctx.cpu
            .run("release.banners", move || build(&p, debut))
            .await
    })
    .await
}

fn build(p: &Planner, mut debut: HashMap<String, Vec<String>>) -> BannersResponse {
    let (cn, en) = (p.cn(), p.en());
    let (window, half_life) = (p.switches.lag_window, p.switches.lag_half_life);
    let aligned = align::align_pools(cn, en, &p.models.pairs, window, half_life);
    let rule_independence = align::rule_independence(cn, en);
    let independent: HashSet<&str> = rule_independence
        .iter()
        .filter(|r| r.independent)
        .map(|r| r.rule_type.as_str())
        .collect();
    let names = p.names(TranslationMemory::build(cn, en, &aligned));
    let en_pools: HashMap<&str, &GachaPoolClient> = en
        .gacha
        .gacha_pool_client
        .iter()
        .map(|pool| (pool.gacha_pool_id.as_str(), pool))
        .collect();
    let shared: HashSet<&str> = cn
        .activities
        .keys()
        .filter(|id| en.activities.contains_key(*id))
        .map(String::as_str)
        .collect();
    let anchors = align::anchors(&cn.gacha.gacha_pool_client, cn, &shared);
    let mut char_names: BTreeMap<String, AutoName> = BTreeMap::new();
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
                KIND_POOL,
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
                .get(&(KIND_POOL.to_string(), pool.gacha_pool_id.clone()))
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
                .map(|e| AutoName::memory(e.gacha_pool_name.clone()))
                .or_else(|| translate::resolve(&names.memory, &pool.gacha_pool_name))
                .or_else(|| override_name(KIND_POOL, &pool.gacha_pool_id, &names.ov));
            ReleaseBanner {
                cn_pool_id: pool.gacha_pool_id.clone(),
                rule_type: pool.gacha_rule_type.clone(),
                name_cn: pool.gacha_pool_name.clone(),
                cn_open: pool.open_time,
                cn_end: pool.end_time,
                returning: pool.returning,
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
                image_path: p.banner_image(
                    en_pool.map(|e| e.gacha_pool_id.as_str()),
                    &pool.gacha_pool_id,
                ),
                resolution,
                declared_share_each: en_pool
                    .and_then(|e| e.declared_share6)
                    .or(pool.declared_share6),
                linkage_loop_at: en_pool.map_or(pool.linkage_loop_at, |e| e.linkage_loop_at),
            }
        })
        .collect();
    banners.sort_by(|a, b| {
        resolved_start(&a.resolution)
            .cmp(&resolved_start(&b.resolution))
            .then(a.cn_open.cmp(&b.cn_open))
            .then(a.cn_pool_id.cmp(&b.cn_pool_id))
    });
    BannersResponse {
        model: p.models.general.clone(),
        yearly: p.models.yearly.clone(),
        banners,
        char_names,
        rule_independence,
    }
}

/// Banner renders at once; a cold pool page asks for dozens, and they wait rather than shed
static RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));

/// The made banner for `act_id`, rendered when missing or older than its art; 404 sends the caller to the cover
pub async fn event_banner(
    ctx: &AppContext,
    server: Server,
    act_id: &str,
) -> Result<PathBuf, AppError> {
    if act_id.is_empty() || act_id.contains(['/', '\\', '\0']) || act_id.starts_with('.') {
        return Err(AppError::NotFound(None));
    }
    let snapshots = &ctx.snapshots;
    for srv in with_default_fallback(ctx, server.data_server()) {
        let Some((background, title)) = snapshots
            .get(srv)
            .and_then(|s| event_banner::sources(&s.model, act_id))
        else {
            continue;
        };
        let root = snapshots.art_dir(srv); // a text-only server shares the art tree and its banners
        let background = root.join(background.trim_start_matches('/'));
        let title = root.join(title.trim_start_matches('/'));
        let target = root.join(event_banner::cache_rel_path(act_id));
        if fresh(&target, &background, &title) {
            return Ok(target);
        }
        if is_stub(&background) || is_stub(&title) {
            continue; // evicted to the object store; the cover stands in
        }
        let _permit = RENDERS
            .acquire()
            .await
            .map_err(|_| AppError::internal("event banner render pool closed"))?;
        if fresh(&target, &background, &title) {
            return Ok(target);
        }
        let made = ctx
            .cpu
            .offload("event_banner", move || -> Result<PathBuf, String> {
                let read = |p: &Path| std::fs::read(p).map_err(|e| e.to_string());
                let bytes = event_banner::render(&read(&background)?, &read(&title)?)
                    .map_err(|e| e.to_string())?;
                write_atomically(&target, &bytes).map_err(|e| e.to_string())?;
                Ok(target)
            })
            .await?;
        match made {
            Ok(path) => return Ok(path),
            Err(e) => tracing::warn!(act_id, error = %e, "event banner failed"),
        }
    }
    Err(AppError::NotFound(None))
}

fn fresh(target: &Path, background: &Path, title: &Path) -> bool {
    match (modified(target), modified(background), modified(title)) {
        (Some(t), Some(b), Some(n)) => t >= b && t >= n,
        _ => false,
    }
}

fn is_stub(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.len() == 0)
}
