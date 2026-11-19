//! Release planner: when CN content reaches EN, from both extracts and the stored overrides.

mod assets;
mod banners;
mod events;
pub mod jobs;
mod overrides;
mod skins;

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use myrtle_db::repos::release;
use myrtle_engine_release::estimate::{self, DEFAULT_HALF_LIFE};
use myrtle_engine_release::kinds::KIND_ACTIVITY;
use myrtle_engine_release::translate::{self, AutoName, TranslationMemory};
use myrtle_engine_release::{
    EventAnchor, LagModel, OverrideRow, Resolution, Switches, YearlyModel, override_index,
    override_name, resolve,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::character::Skin;
use myrtle_types::Server;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

pub use banners::{event_banner, get_banners};
pub use events::{get_events, get_lag};
pub use overrides::{
    delete_override, get_plan, list_overrides, override_row, put_override, put_plan, wire_override,
};
pub use skins::{get_skins, one_guess_per_group};

/// Every release body; override writes and art mirrors flush it
pub const CACHE_PREFIX: &str = "static:cn:release:";
const CACHE_TTL: Duration = Duration::from_hours(24);
/// Names the ledger and override rows a body was built from, so a build that read older rows lands on a dead key
const REVISION_KEY: &str = "release:revision";
const REVISION_TTL: Duration = Duration::from_hours(24 * 30);

/// Retires every release body, in this process and the others
pub async fn revise(ctx: &AppContext) {
    let revision = uuid::Uuid::new_v4().simple().to_string();
    ctx.cache.set(REVISION_KEY, &revision, REVISION_TTL).await;
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;
}
const LOOKBACK_GRACE_SECS: i64 = 60 * 86_400;

/// The lag model folds into the key so a changed window or half-life never serves the other model
#[must_use]
pub fn cache_key(resource: &str, switches: &Switches) -> String {
    let hash = switches.lag_window as u64 ^ switches.lag_half_life.to_bits().rotate_left(1);
    format!("{CACHE_PREFIX}{resource}:{hash}")
}

#[must_use]
pub fn switches(config: &Config) -> Switches {
    let flags = &config.switches;
    let half_life = config.release.lag_half_life;
    Switches {
        lag_window: config.release.lag_window,
        lag_half_life: if !half_life.is_finite() {
            DEFAULT_HALF_LIFE
        } else if half_life > 0.0 {
            half_life
        } else {
            0.0
        },
        keep_unsold_skins: flags.release_keep_unsold_skins.enabled,
        reviews_on_every_group: flags.release_reviews_all_groups.enabled,
        trust_skipped_tags: flags.release_trust_skipped_tags.enabled,
    }
}

async fn cached<T, F, Fut>(ctx: &AppContext, resource: &str, build: F) -> Result<T, AppError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(AppContext, Planner) -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, AppError>> + Send + 'static,
{
    let cn = ctx
        .snapshots
        .get(Server::CN)
        .ok_or(AppError::NotFound(None))?;
    let en = ctx.snapshots.require(Server::EN)?;
    let revision = ctx.cache.get(REVISION_KEY).await.unwrap_or_default();
    let resource = format!("{resource}:{revision}");
    let key = cn.key(&en.key(&cache_key(&resource, &switches(&ctx.config))));
    let owner = ctx.clone();
    let body = ctx
        .cache
        .json_detached(&key, CACHE_TTL, move || async move {
            let planner = Planner::load(&owner, cn, en).await?;
            let out = build(owner, planner).await?;
            serde_json::to_string(&out).map_err(AppError::internal)
        })
        .await?;
    serde_json::from_str(&body.body).map_err(AppError::internal)
}

struct Models {
    pairs: Vec<estimate::ActivityPair>,
    general: LagModel,
    yearly: YearlyModel,
}

impl Models {
    fn build(cn: &GameModel, en: &GameModel, switches: &Switches) -> Self {
        let all = estimate::activity_pairs(cn, en);
        let types = estimate::yearly_types(&all);
        let (general_pairs, yearly_pairs) = estimate::split_yearly(&all, &types);
        Self {
            general: estimate::build_lag_model(
                &general_pairs,
                switches.lag_window,
                switches.lag_half_life,
            ),
            yearly: YearlyModel {
                types,
                model: estimate::build_yearly_model(&yearly_pairs),
            },
            pairs: general_pairs,
        }
    }

    fn is_yearly(&self, activity_type: &str) -> bool {
        self.yearly.types.iter().any(|t| t == activity_type)
    }

    fn lookback_secs(&self) -> i64 {
        let longest = if self.yearly.model.n > 0 {
            self.yearly.model.median_days.max(self.general.p75_days)
        } else {
            self.general.p75_days
        };
        (longest * 86_400.0).round() as i64 + LOOKBACK_GRACE_SECS
    }

    fn for_type(&self, activity_type: &str) -> &LagModel {
        if self.is_yearly(activity_type) {
            &self.yearly.model
        } else {
            &self.general
        }
    }
}

struct Planner {
    cn: Arc<Snapshot>,
    en: Arc<Snapshot>,
    brand_by_group: HashMap<String, String>,
    models: Models,
    rows: Vec<OverrideRow>,
    switches: Switches,
    pooled_cadence: bool,
    now: i64,
}

impl Planner {
    async fn load(
        ctx: &AppContext,
        cn: Arc<Snapshot>,
        en: Arc<Snapshot>,
    ) -> Result<Self, AppError> {
        let mut conn = ctx.db.conn().await?;
        let rows = release::list_overrides(&mut conn)
            .await?
            .iter()
            .map(override_row)
            .collect();
        drop(conn);
        let switches = switches(&ctx.config);
        let mut brand_by_group: HashMap<String, String> = HashMap::new();
        for b in cn.model.skins.brand_list.values() {
            for g in &b.group_list {
                brand_by_group.insert(g.skin_group_id.clone(), b.brand_id.clone());
            }
            for k in &b.kv_img_id_list {
                brand_by_group
                    .entry(k.linked_skin_group_id.clone())
                    .or_insert_with(|| b.brand_id.clone());
            }
        }
        Ok(Self {
            models: Models::build(&cn.model, &en.model, &switches),
            cn,
            en,
            brand_by_group,
            rows,
            switches,
            pooled_cadence: ctx.config.switches.release_pooled_cadence.enabled,
            now: Utc::now().timestamp(),
        })
    }

    fn cn(&self) -> &GameModel {
        &self.cn.model
    }

    fn en(&self) -> &GameModel {
        &self.en.model
    }

    fn since(&self) -> i64 {
        self.now - self.models.lookback_secs()
    }

    fn names(&self, memory: TranslationMemory) -> Names<'_> {
        Names {
            memory,
            ov: override_index(&self.rows),
            en_idx: estimate::en_activity_index(self.en()),
        }
    }

    fn resolve_anchored(&self, hit: &estimate::Anchored<'_>, names: &Names<'_>) -> Resolution {
        let r = self.resolve_activity(hit.activity, names);
        if hit.offset_secs == 0 {
            return r;
        }
        estimate::shift_resolution(r, hit.offset_secs)
    }

    fn resolve_activity(&self, a: &ActivityBasicInfo, names: &Names<'_>) -> Resolution {
        let hit = names.en_idx.get(a.id.as_str());
        resolve(
            KIND_ACTIVITY,
            &a.id,
            hit.map(|(s, e, _)| (a.id.as_str(), *s, *e)),
            &names.ov,
            self.models.for_type(&a.activity_type),
            a.start_time,
        )
    }

    fn obtain_label(&self, sk: &Skin, names: &Names<'_>) -> String {
        let cn = sk.display_skin.obtain_approach.as_deref().unwrap_or("");
        self.en()
            .skins
            .char_skins
            .get(&sk.skin_id)
            .and_then(|e| e.display_skin.obtain_approach.clone())
            .or_else(|| names.memory.get(cn).map(str::to_string))
            .unwrap_or_else(|| cn.to_string())
    }

    fn event_name_auto(&self, a: &ActivityBasicInfo, names: &Names<'_>) -> Option<AutoName> {
        if names.en_idx.contains_key(a.id.as_str()) {
            return None;
        }
        translate::resolve(&names.memory, &a.name)
            .or_else(|| self.event_name_by_id(&a.id))
            .or_else(|| self.event_zone_name(&a.id))
            .or_else(|| override_name(KIND_ACTIVITY, &a.id, &names.ov))
    }

    fn event_anchor(&self, hit: &estimate::Anchored<'_>, names: &Names<'_>) -> EventAnchor {
        let a = hit.activity;
        EventAnchor {
            cn_id: a.id.clone(),
            name_cn: a.name.clone(),
            cn_start: a.start_time,
            cn_end: a.end_time,
            offset_secs: hit.offset_secs,
            name_en: names
                .en_idx
                .get(a.id.as_str())
                .map(|(_, _, n)| (*n).to_string()),
            name_en_auto: self.event_name_auto(a, names),
        }
    }
}

struct Names<'a> {
    memory: TranslationMemory,
    ov: HashMap<(String, String), &'a OverrideRow>,
    en_idx: HashMap<&'a str, (i64, i64, &'a str)>,
}

const fn resolved_start(r: &Resolution) -> i64 {
    match r {
        Resolution::Confirmed { en_start, .. }
        | Resolution::Override { en_start, .. }
        | Resolution::Estimated { en_start, .. } => *en_start,
        Resolution::Unlisted | Resolution::Unmodelled | Resolution::Independent => i64::MAX,
    }
}
