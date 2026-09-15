use std::collections::HashMap;

use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        gamedata::types::activity::FarmStage,
        release::{EventsResponse, LagResponse, ReleaseEvent, estimate},
        translate::TranslationMemory,
    },
};

use super::{Names, Planner, cache_key, resolved_start};

const BACKTEST_SINCE: i64 = 1_704_067_200;
const MINI_EVENT_TYPE: &str = "MINISTORY";

pub async fn get_lag(state: &AppState) -> Result<LagResponse, ApiError> {
    let key = cache_key("release:lag", estimate::window_from_env());
    if let Some(c) = state.cache.get::<LagResponse>(&key).await {
        return Ok(c);
    }
    let p = Planner::load(state).await?;
    let out = LagResponse {
        backtest: estimate::backtest(&p.models.pairs, p.window, BACKTEST_SINCE),
        model: p.models.general,
        yearly: p.models.yearly,
    };
    state.cache.set(&key, &out).await;
    Ok(out)
}

fn farm_stages(p: &Planner, act_id: &str) -> Vec<FarmStage> {
    let mut stages = p
        .ctx
        .cn
        .activity_farm_stages
        .get(act_id)
        .cloned()
        .unwrap_or_default();
    for st in &mut stages {
        for d in &mut st.drops {
            d.name_en = p
                .ctx
                .en
                .materials
                .items
                .get(&d.item_id)
                .map(|i| i.name.clone());
        }
    }
    stages
}

pub async fn get_events(state: &AppState) -> Result<EventsResponse, ApiError> {
    let key = cache_key("release:events", estimate::window_from_env());
    if let Some(c) = state.cache.get::<EventsResponse>(&key).await {
        return Ok(c);
    }
    let p = Planner::load(state).await?;
    let names = Names::new(
        &p,
        TranslationMemory::build(&p.ctx.cn, &p.ctx.en, &HashMap::new()),
    );
    let since = p.since();
    let mut events: Vec<ReleaseEvent> = p
        .ctx
        .cn
        .activities
        .values()
        .filter(|a| a.start_time > 0 && a.start_time >= since)
        .map(|a| ReleaseEvent {
            cn_id: a.id.clone(),
            name_cn: a.name.clone(),
            name_en: names
                .en_idx
                .get(a.id.as_str())
                .map(|(_, _, n)| (*n).to_string()),
            activity_type: a.activity_type.clone(),
            has_stage: a.has_stage,
            cn_start: a.start_time,
            cn_end: a.end_time,
            op_stages: p
                .ctx
                .cn
                .activity_op_stages
                .get(&a.id)
                .cloned()
                .unwrap_or_default(),
            farm_stages: if a.activity_type == MINI_EVENT_TYPE {
                Vec::new()
            } else {
                farm_stages(&p, &a.id)
            },
            name_en_auto: p.event_name_auto(a, &names),
            image_path: p.ctx.event_image(&a.id),
            resolution: p.resolve_activity(a, &names),
        })
        .collect();
    events.sort_by(|a, b| {
        resolved_start(&a.resolution)
            .cmp(&resolved_start(&b.resolution))
            .then(a.cn_start.cmp(&b.cn_start))
            .then(a.cn_id.cmp(&b.cn_id))
    });
    let out = EventsResponse {
        model: p.models.general.clone(),
        yearly: p.models.yearly.clone(),
        events,
    };
    state.cache.set(&key, &out).await;
    Ok(out)
}
