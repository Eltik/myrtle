use std::collections::{HashMap, HashSet};

use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        gamedata::types::{GameData, skin::Skin},
        release::{
            BatchForecast, NewSkin, RerunForecast, Resolution, ReviewOutfit, ReviewWindow,
            SkinGroupArt, SkinTile, SkinsResponse, estimate, ledger, prices, resolve,
            skins::{self, GroupHistory, RerunBasis},
        },
        translate::{self, TranslationMemory},
    },
};

use super::{Names, Planner, cache_key, resolved_start};

fn is_yearly_reward(p: &Planner, skin: &Skin) -> bool {
    let cn = &p.ctx.cn;
    let get_time = skin.display_skin.get_time;
    if get_time <= 0 || !cn.activity_skin_refs.contains(&skin.skin_id) {
        return false;
    }
    let Some(day) = estimate::cn_day(get_time) else {
        return false;
    };
    cn.activities.values().any(|a| {
        a.start_time > 0
            && p.models.is_yearly(&a.activity_type)
            && estimate::cn_day(a.start_time) == Some(day)
    })
}

fn en_first_windows(en: &GameData) -> HashMap<&str, (i64, i64)> {
    let mut out: HashMap<&str, (i64, i64)> = HashMap::new();
    for w in &en.skin_windows {
        let e = out
            .entry(w.skin_id.as_str())
            .or_insert((w.start_time, w.end_time));
        if w.start_time < e.0 {
            *e = (w.start_time, w.end_time);
        }
    }
    out
}

fn tile(p: &Planner, names: &Names<'_>, sk: &Skin) -> SkinTile {
    let (cn, en) = (&p.ctx.cn, &p.ctx.en);
    let on_en = en.skins.char_skins.contains_key(&sk.skin_id);
    SkinTile {
        skin_id: sk.skin_id.clone(),
        char_id: sk.char_id.clone(),
        skin_name: en
            .skins
            .char_skins
            .get(&sk.skin_id)
            .and_then(|e| e.display_skin.skin_name.clone())
            .or_else(|| sk.display_skin.skin_name.clone())
            .unwrap_or_default(),
        char_name: translate::operator_name(cn, en, &sk.char_id),
        portrait_path: p.ctx.skin_portrait(&sk.char_id, &sk.portrait_id, on_en),
        colors: colors(&sk.display_skin.color_list),
        price: prices::skin_price(sk, p.obtain_label(sk, names)),
    }
}

fn colors(list: &[String]) -> Vec<String> {
    list.iter()
        .filter(|c| c.len() == 7 && c.starts_with('#'))
        .cloned()
        .collect()
}

fn tiles(p: &Planner, names: &Names<'_>, g: &GroupHistory) -> Vec<SkinTile> {
    g.skin_ids
        .iter()
        .filter_map(|id| p.ctx.cn.skins.char_skins.get(id))
        .map(|sk| tile(p, names, sk))
        .collect()
}

struct GroupArt<'a> {
    ctx: &'a super::ctx::Ctx,
    map: HashMap<String, SkinGroupArt>,
}

impl GroupArt<'_> {
    fn touch(&mut self, gid: &str) {
        if !gid.is_empty()
            && !self.map.contains_key(gid)
            && let Some(a) = self.ctx.group_art(gid)
        {
            self.map.insert(gid.to_string(), a);
        }
    }
}

fn new_skins(p: &Planner, names: &Names<'_>, art: &mut GroupArt<'_>) -> Vec<NewSkin> {
    let (cn, en) = (&*p.ctx.cn, &*p.ctx.en);
    let first_windows = en_first_windows(en);
    let runs = estimate::StageRuns::build(cn);
    let since = p.since();
    let mut out: Vec<NewSkin> = cn
        .skins
        .char_skins
        .values()
        .filter(|s| s.display_skin.get_time > 0 && s.display_skin.get_time >= since)
        .map(|s| {
            let confirmed = en.skins.char_skins.get(&s.skin_id).and_then(|e| {
                let (start, end) = first_windows
                    .get(s.skin_id.as_str())
                    .copied()
                    .unwrap_or((e.display_skin.get_time, 0));
                (start > 0).then_some((s.skin_id.as_str(), start, end))
            });
            let model = if is_yearly_reward(p, s) {
                &p.models.yearly.model
            } else {
                &p.models.general
            };
            let mut resolution = resolve(
                ledger::KIND_SKIN,
                &s.skin_id,
                confirmed,
                &names.ov,
                model,
                s.display_skin.get_time,
            );
            let anchor = runs.anchor(s.display_skin.get_time).map(|hit| {
                let event = p.resolve_anchored(&hit, names);
                if matches!(resolution, Resolution::Estimated { .. })
                    && !matches!(event, Resolution::Unmodelled | Resolution::Independent)
                {
                    resolution = event;
                }
                p.event_anchor(&hit, names)
            });
            let skin_name = s.display_skin.skin_name.clone().unwrap_or_default();
            art.touch(&s.display_skin.skin_group_id);
            NewSkin {
                skin_id: s.skin_id.clone(),
                char_id: s.char_id.clone(),
                skin_name_auto: translate::resolve(&names.memory, &skin_name),
                skin_group_name_auto: translate::resolve(
                    &names.memory,
                    &s.display_skin.skin_group_name,
                ),
                char_name: translate::operator_name(cn, en, &s.char_id),
                portrait_path: p.ctx.skin_portrait(
                    &s.char_id,
                    &s.portrait_id,
                    en.skins.char_skins.contains_key(&s.skin_id),
                ),
                colors: colors(&s.display_skin.color_list),
                price: prices::skin_price(s, p.obtain_label(s, names)),
                skin_name,
                skin_group_id: s.display_skin.skin_group_id.clone(),
                skin_group_name: s.display_skin.skin_group_name.clone(),
                is_buy_skin: s.is_buy_skin,
                obtain_approach: s.display_skin.obtain_approach.clone(),
                cn_get_time: s.display_skin.get_time,
                anchor,
                resolution,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        resolved_start(&a.resolution)
            .cmp(&resolved_start(&b.resolution))
            .then(a.cn_get_time.cmp(&b.cn_get_time))
            .then(a.skin_id.cmp(&b.skin_id))
    });
    out
}

fn batches(
    p: &Planner,
    names: &Names<'_>,
    en_batches: &[skins::Batch],
    cn_batches: &[skins::Batch],
) -> Vec<BatchForecast> {
    let en_seen: HashSet<&str> = en_batches
        .iter()
        .filter_map(|b| b.img_id.as_deref())
        .collect();
    let mut out: Vec<BatchForecast> = cn_batches
        .iter()
        .filter(|b| b.img_id.as_deref().is_some_and(|i| !en_seen.contains(i)))
        .map(|b| BatchForecast {
            name: b.name.clone(),
            name_en_auto: translate::resolve(&names.memory, &b.name),
            cn_start: b.start_time,
            cn_end: b.end_time,
            resolution: estimate::estimate(&p.models.general, b.start_time),
        })
        .chain(
            en_batches
                .iter()
                .filter(|b| b.start_time > p.now)
                .map(|b| BatchForecast {
                    name: b.name.clone(),
                    name_en_auto: None,
                    cn_start: 0,
                    cn_end: 0,
                    resolution: Resolution::Confirmed {
                        en_id: b.name.clone(),
                        en_start: b.start_time,
                        en_end: b.end_time,
                    },
                }),
        )
        .collect();
    out.sort_by_key(|b| resolved_start(&b.resolution));
    out
}

fn reruns(
    p: &Planner,
    names: &Names<'_>,
    en_groups: &[GroupHistory],
    cn_groups: &[GroupHistory],
    art: &mut GroupArt<'_>,
) -> Vec<RerunForecast> {
    let runs = estimate::StageRuns::build(&p.ctx.cn);
    let en_by_group: HashMap<&str, &GroupHistory> = en_groups
        .iter()
        .map(|g| (g.skin_group_id.as_str(), g))
        .collect();
    let cn_by_group: HashMap<&str, &GroupHistory> = cn_groups
        .iter()
        .map(|g| (g.skin_group_id.as_str(), g))
        .collect();
    let cadence = skins::cadence_model(en_groups);
    let mut consumed: HashSet<(String, i64)> = HashSet::new();
    let mut out: Vec<RerunForecast> = Vec::new();
    for pending in skins::pending_cn_reruns(cn_groups, p.now, p.models.lookback_secs()) {
        let g = pending.group;
        let en_g = en_by_group.get(g.skin_group_id.as_str()).copied();
        let anchor = runs.anchor(pending.cn_window.start_time);
        let mut next = anchor
            .as_ref()
            .map(|hit| p.resolve_anchored(hit, names))
            .filter(|r| !matches!(r, Resolution::Unmodelled | Resolution::Independent))
            .unwrap_or_else(|| estimate::estimate(&p.models.general, pending.cn_window.start_time));
        let mut basis = RerunBasis::CnListing {
            cn_start: pending.cn_window.start_time,
            cn_end: pending.cn_window.end_time,
            anchor: anchor.as_ref().map(|hit| p.event_anchor(hit, names)),
        };
        if let Some(w) = skins::match_en_listing(en_g, resolved_start(&next), &mut consumed) {
            if w.end_time < p.now && cadence.n > 0 {
                let en_last = en_g
                    .and_then(GroupHistory::last_seen)
                    .unwrap_or(w.start_time);
                next = skins::next_by_cadence(en_last, &cadence);
                basis = RerunBasis::Cadence {
                    en_last,
                    n: cadence.n,
                    median_days: cadence.median_days,
                    p25_days: cadence.p25_days,
                    p75_days: cadence.p75_days,
                };
            } else {
                next = Resolution::Confirmed {
                    en_id: g.skin_group_id.clone(),
                    en_start: w.start_time,
                    en_end: w.end_time,
                };
            }
        }
        art.touch(&g.skin_group_id);
        out.push(RerunForecast {
            skin_group_id: g.skin_group_id.clone(),
            skin_group_name: en_g
                .map_or_else(|| g.skin_group_name.clone(), |e| e.skin_group_name.clone()),
            skin_ids: g.skin_ids.clone(),
            skins: tiles(p, names, g),
            windows: en_g.map(|e| e.windows.clone()).unwrap_or_default(),
            last_seen: en_g.and_then(GroupHistory::last_seen).unwrap_or(0),
            next,
            basis,
        });
    }
    for g in en_groups {
        for w in g.listings() {
            if w.start_time <= p.now || consumed.contains(&(g.skin_group_id.clone(), w.start_time))
            {
                continue;
            }
            art.touch(&g.skin_group_id);
            out.push(RerunForecast {
                skin_group_id: g.skin_group_id.clone(),
                skin_group_name: g.skin_group_name.clone(),
                skin_ids: g.skin_ids.clone(),
                skins: cn_by_group
                    .get(g.skin_group_id.as_str())
                    .map(|c| tiles(p, names, c))
                    .unwrap_or_default(),
                windows: g.windows.clone(),
                last_seen: g.last_seen().unwrap_or(0),
                next: Resolution::Confirmed {
                    en_id: g.skin_group_id.clone(),
                    en_start: w.start_time,
                    en_end: w.end_time,
                },
                basis: RerunBasis::None,
            });
        }
    }
    out.sort_by(|a, b| {
        resolved_start(&a.next)
            .cmp(&resolved_start(&b.next))
            .then(a.skin_group_id.cmp(&b.skin_group_id))
    });
    out
}

fn review_pool(p: &Planner, names: &Names<'_>) -> Vec<ReviewOutfit> {
    let (cn, en) = (&*p.ctx.cn, &*p.ctx.en);
    let mut out: Vec<ReviewOutfit> = cn
        .skins
        .char_skins
        .values()
        .filter(|s| s.display_skin.get_time > 0 && skins::review_eligible(s, &cn.skins.brand_list))
        .map(|s| ReviewOutfit {
            tile: tile(p, names, s),
            cn_get_time: s.display_skin.get_time,
            en_get_time: en
                .skins
                .char_skins
                .get(&s.skin_id)
                .map(|e| e.display_skin.get_time)
                .filter(|t| *t > 0),
        })
        .collect();
    out.sort_by(|a, b| {
        a.cn_get_time
            .cmp(&b.cn_get_time)
            .then_with(|| a.tile.skin_id.cmp(&b.tile.skin_id))
    });
    out
}

fn reviews(p: &Planner) -> Vec<ReviewWindow> {
    let cn = skins::review_windows(&p.ctx.cn);
    let en = skins::review_windows(&p.ctx.en);
    cn.iter()
        .zip(skins::pair_reviews(&cn, &en))
        .map(|(&(cn_start, cn_end), en_window)| ReviewWindow {
            cn_start,
            cn_end,
            pool_cutoff: skins::review_pool_cutoff(cn_start),
            resolution: match en_window {
                Some((en_start, en_end)) => Resolution::Confirmed {
                    en_id: format!("review:{en_start}"),
                    en_start,
                    en_end,
                },
                None => estimate::estimate(&p.models.general, cn_start),
            },
        })
        .collect()
}

pub async fn get_skins(state: &AppState) -> Result<SkinsResponse, ApiError> {
    let key = cache_key("release:skins", estimate::window_from_env());
    if let Some(c) = state.cache.get::<SkinsResponse>(&key).await {
        return Ok(c);
    }
    let p = Planner::load(state).await?;
    let (cn, en) = (&*p.ctx.cn, &*p.ctx.en);
    let names = Names::new(&p, TranslationMemory::build(cn, en, &HashMap::new()));
    let mut art = GroupArt {
        ctx: &p.ctx,
        map: HashMap::new(),
    };
    let (en_groups, en_batches) = skins::group_histories(en, &skins::batch_families(en, cn));
    let (cn_groups, cn_batches) = skins::group_histories(cn, &skins::batch_families(cn, en));
    let review_pool = review_pool(&p, &names);
    let out = SkinsResponse {
        anniversaries: skins::anniversary_models(&en_groups, p.now),
        batches: batches(&p, &names, &en_batches, &cn_batches),
        new_skins: new_skins(&p, &names, &mut art),
        rerun_forecasts: reruns(&p, &names, &en_groups, &cn_groups, &mut art),
        reviews: reviews(&p),
        review_pool,
        group_art: art.map,
        model: p.models.general.clone(),
        yearly: p.models.yearly.clone(),
    };
    state.cache.set(&key, &out).await;
    Ok(out)
}
