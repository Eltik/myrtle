use std::collections::HashMap;

use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        gamedata::types::{activity::FarmStage, event_shop::EventShopData, skin::Skin},
        release::{
            EventShop, EventsResponse, LagResponse, ReleaseEvent, ShopGood, ShopGoodKind, ShopItem,
            estimate,
        },
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

/// The event's token shop from whichever server has fetched it, CN first.
fn event_shop(p: &Planner, act_id: &str) -> Option<EventShop> {
    let (cn, en) = (&p.ctx.cn, &p.ctx.en);
    let (server, data): (&str, &EventShopData) = cn
        .event_shops
        .get(act_id)
        .map(|s| ("cn", s))
        .or_else(|| en.event_shops.get(act_id).map(|s| (p.ctx.en_server, s)))?;
    Some(EventShop {
        server: server.to_owned(),
        shop_name: data.shop_name.clone(),
        token: shop_item(p, "MATERIAL", &data.token_id),
        start_time: data.start_time,
        end_time: data.end_time,
        max_price: data.max_price,
        goods: data
            .goods
            .iter()
            .map(|g| ShopGood {
                good_id: g.good_id.clone(),
                item_type: g.item_type.clone(),
                kind: ShopGoodKind::of(&g.item_type),
                item: shop_item(p, &g.item_type, &g.item_id),
                count: g.count,
                price: g.price,
                avail_count: g.avail_count,
            })
            .collect(),
    })
}

/// Name and icon for a shop good from the table that knows the item type:
/// outfits from the skin table, furniture from the building catalogue,
/// everything else from the item table. EN's name when EN has the item.
fn shop_item(p: &Planner, item_type: &str, item_id: &str) -> ShopItem {
    let (cn, en) = (&p.ctx.cn, &p.ctx.en);
    match ShopGoodKind::of(item_type) {
        ShopGoodKind::Outfit => {
            let on_en = en.skins.char_skins.contains_key(item_id);
            let name = |sk: &Skin| sk.display_skin.skin_name.clone().unwrap_or_default();
            let skin = cn.skins.char_skins.get(item_id);
            ShopItem {
                item_id: item_id.to_owned(),
                name: skin.map_or_else(|| item_id.to_owned(), name),
                name_en: en.skins.char_skins.get(item_id).map(name),
                icon_path: skin.and_then(|sk| p.ctx.avatar(&sk.avatar_id, on_en)),
                rarity: 0,
            }
        }
        ShopGoodKind::Furniture => {
            let on_en = en.building.custom_data.furnitures.contains_key(item_id);
            let piece = cn.building.custom_data.furnitures.get(item_id);
            ShopItem {
                item_id: item_id.to_owned(),
                name: piece.map_or_else(|| item_id.to_owned(), |f| f.name.clone()),
                name_en: en
                    .building
                    .custom_data
                    .furnitures
                    .get(item_id)
                    .map(|f| f.name.clone()),
                icon_path: piece.and_then(|f| p.ctx.furniture_icon(&f.icon_id, on_en)),
                rarity: piece.map_or(0, |f| f.rarity),
            }
        }
        _ => {
            let on_en = en.materials.items.contains_key(item_id);
            let item = cn.materials.items.get(item_id);
            ShopItem {
                item_id: item_id.to_owned(),
                name: item.map_or_else(|| item_id.to_owned(), |i| i.name.clone()),
                name_en: en.materials.items.get(item_id).map(|i| i.name.clone()),
                icon_path: item.and_then(|i| p.ctx.item_icon(&i.icon_id, on_en)),
                rarity: item.map_or(0, |i| i32::from(i.rarity.tier())),
            }
        }
    }
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
            mission_tokens: p
                .ctx
                .cn
                .activity_mission_tokens
                .get(&a.id)
                .copied()
                .unwrap_or(0),
            shop: event_shop(&p, &a.id),
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
