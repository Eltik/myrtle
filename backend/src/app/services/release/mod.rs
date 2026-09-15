mod banners;
mod ctx;
mod events;
mod models;
mod overrides;
mod plans;
mod skins;

use std::collections::HashMap;

use chrono::Utc;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    core::{
        gamedata::types::{activity::ActivityBasicInfo, skin::Skin},
        release::{EventAnchor, Resolution, estimate, ledger, override_name, resolve},
        translate::{self, AutoName, TranslationMemory},
    },
    database::queries::release as q,
};

pub use banners::get_banners;
pub use events::{get_events, get_lag};
pub use overrides::{delete_override, list_overrides, put_override};
pub use plans::{get_plan, put_plan};
pub use skins::get_skins;

use ctx::Ctx;
use models::Models;

const CACHE_PREFIX: &str = "static:cn:release:";

const fn cache_key(resource: &'static str, window: usize) -> CacheKey<'static> {
    CacheKey::StaticData {
        resource,
        server: "cn",
        fields_hash: window as u64,
        page: 0,
    }
}

type Overrides<'a> = HashMap<(String, String), &'a q::OverrideRow>;
type EnActivities<'a> = HashMap<&'a str, (i64, i64, &'a str)>;

struct Planner {
    ctx: Ctx,
    models: Models,
    rows: Vec<q::OverrideRow>,
    window: usize,
    now: i64,
}

impl Planner {
    async fn load(state: &AppState) -> Result<Self, ApiError> {
        let ctx = Ctx::load(state)?;
        let window = estimate::window_from_env();
        let models = Models::build(&ctx.cn, &ctx.en, window);
        let rows = q::list_overrides(&state.db).await?;
        Ok(Self {
            ctx,
            models,
            rows,
            window,
            now: Utc::now().timestamp(),
        })
    }

    fn since(&self) -> i64 {
        self.now - self.models.lookback_secs()
    }
}

struct Names<'a> {
    memory: TranslationMemory,
    ov: Overrides<'a>,
    en_idx: EnActivities<'a>,
}

impl<'a> Names<'a> {
    fn new(p: &'a Planner, memory: TranslationMemory) -> Self {
        Self {
            memory,
            ov: crate::core::release::override_index(&p.rows),
            en_idx: estimate::en_activity_index(&p.ctx.en),
        }
    }
}

impl Planner {
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
            ledger::KIND_ACTIVITY,
            &a.id,
            hit.map(|(s, e, _)| (a.id.as_str(), *s, *e)),
            &names.ov,
            self.models.for_type(&a.activity_type),
            a.start_time,
        )
    }

    fn obtain_label(&self, sk: &Skin, names: &Names<'_>) -> String {
        let cn = sk.display_skin.obtain_approach.as_deref().unwrap_or("");
        self.ctx
            .en
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
            .or_else(|| self.ctx.event_name_by_id(&a.id))
            .or_else(|| override_name(ledger::KIND_ACTIVITY, &a.id, &names.ov))
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

const fn resolved_start(r: &Resolution) -> i64 {
    match r {
        Resolution::Confirmed { en_start, .. }
        | Resolution::Override { en_start, .. }
        | Resolution::Estimated { en_start, .. } => *en_start,
        Resolution::Unmodelled | Resolution::Independent => i64::MAX,
    }
}
