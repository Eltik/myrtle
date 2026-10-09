use std::collections::{HashMap, HashSet};

use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        gamedata::types::{GameData, shop::SkinListing, skin::Skin},
        release::{
            BatchForecast, NewSkin, RerunForecast, Resolution, ReviewOutfit, ReviewWindow,
            SkinGroupArt, SkinTile, SkinsResponse, estimate, ledger, prices, resolve,
            resolve_reviews,
            skins::{self, CadenceModel, GroupHistory, RerunBasis},
            skipped::{self, SkippedTags},
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
                skin_group_name_auto: translate::resolve_group(
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
    skipped: &SkippedTags,
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
            name_en_auto: translate::resolve_group(&names.memory, &b.name),
            cn_start: b.start_time,
            cn_end: b.end_time,
            resolution: skipped::unless_passed(
                estimate::estimate(&p.models.general, b.start_time),
                skipped.passed(b.tag_id.as_deref(), b.start_time),
            ),
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

/// `RELEASE_POOLED_CADENCE=1` restores the pre-fix cadence rule: anchor on
/// the latest window of any kind (reviews included) plus the pooled median.
fn pooled_cadence() -> bool {
    std::env::var("RELEASE_POOLED_CADENCE").ok().as_deref() == Some("1")
}

/// A group's next EN rerun by its EN listing rhythm; `anchor` stands in when
/// the group has no EN listing of its own to date from.
fn by_cadence(
    en_g: Option<&GroupHistory>,
    cadence: &CadenceModel,
    anchor: Option<i64>,
) -> Option<(i64, Resolution, Option<f64>)> {
    if pooled_cadence() {
        let en_last = en_g.and_then(GroupHistory::last_seen).or(anchor)?;
        Some((en_last, skins::next_by_cadence(en_last, cadence), None))
    } else {
        en_g.and_then(|e| skins::next_by_own_cadence(e, cadence))
            .or_else(|| anchor.map(|a| (a, skins::next_by_cadence(a, cadence), None)))
    }
}

fn by_group(groups: &[GroupHistory]) -> HashMap<&str, &GroupHistory> {
    groups
        .iter()
        .map(|g| (g.skin_group_id.as_str(), g))
        .collect()
}

/// `cn_kept` is the CN histories without the listings EN has passed (see
/// `release::skipped`), `None` when there are none. A pending CN window that
/// only those listings made is never dated from CN: EN's own rhythm dates
/// the group as it does after a matched rerun, and a group with no EN
/// listing to date from is Unlisted.
fn reruns(
    p: &Planner,
    names: &Names<'_>,
    en_groups: &[GroupHistory],
    cn_groups: &[GroupHistory],
    cn_kept: Option<&[GroupHistory]>,
    art: &mut GroupArt<'_>,
) -> Vec<RerunForecast> {
    let runs = estimate::StageRuns::build(&p.ctx.cn);
    let en_by_group = by_group(en_groups);
    let cn_by_group = by_group(cn_groups);
    let kept_by_group = cn_kept.map(by_group);
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
        let passed = kept_by_group.as_ref().is_some_and(|k| {
            !skins::listed_near(
                k.get(g.skin_group_id.as_str()).copied(),
                pending.cn_window.start_time,
            )
        });
        let matched = skins::match_en_listing(en_g, resolved_start(&next), &mut consumed);
        let dated = match matched {
            Some(w) if w.end_time < p.now && cadence.n > 0 => {
                by_cadence(en_g, &cadence, Some(w.start_time))
            }
            Some(w) => {
                next = Resolution::Confirmed {
                    en_id: g.skin_group_id.clone(),
                    en_start: w.start_time,
                    en_end: w.end_time,
                };
                None
            }
            None if passed => {
                let dated = (cadence.n > 0)
                    .then(|| by_cadence(en_g, &cadence, None))
                    .flatten();
                if dated.is_none() {
                    next = Resolution::Unlisted;
                }
                dated
            }
            None => None,
        };
        if let Some((en_last, dated, own_gap_days)) = dated {
            next = dated;
            basis = RerunBasis::Cadence {
                en_last,
                n: cadence.n,
                median_days: cadence.median_days,
                p25_days: cadence.p25_days,
                p75_days: cadence.p75_days,
                own_gap_days,
            };
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
    one_guess_per_group(&mut out);
    out.sort_by(|a, b| {
        resolved_start(&a.next)
            .cmp(&resolved_start(&b.next))
            .then(a.skin_group_id.cmp(&b.skin_group_id))
    });
    out
}

/// Each pending CN window yields a forecast, so a group whose older CN rerun
/// EN already sold (dated by cadence) and whose newer one is still pending
/// came out twice, and the planner sold the same outfits on two cards. A
/// cadence date only stands in for a group nothing else dates: it yields to
/// any listing-dated forecast, and a group keeps one of them at most.
fn one_guess_per_group(out: &mut Vec<RerunForecast>) {
    let is_guess = |f: &RerunForecast| matches!(f.basis, RerunBasis::Cadence { .. });
    let dated: HashSet<String> = out
        .iter()
        .filter(|f| !is_guess(f) && !matches!(f.next, Resolution::Unlisted))
        .map(|f| f.skin_group_id.clone())
        .collect();
    let mut guessed: HashSet<String> = HashSet::new();
    out.retain(|f| {
        !is_guess(f)
            || (!dated.contains(&f.skin_group_id) && guessed.insert(f.skin_group_id.clone()))
    });
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

fn reviews(p: &Planner, names: &Names<'_>, skipped: &SkippedTags) -> Vec<ReviewWindow> {
    let cn = skins::review_windows(&p.ctx.cn);
    let en = skins::review_windows(&p.ctx.en);
    let passed = skipped.review_starts(&p.ctx.cn);
    let resolved = resolve_reviews(&cn, &en, &names.ov, &p.models.general, &passed);
    cn.iter()
        .zip(resolved)
        .map(|(&(cn_start, cn_end), resolution)| ReviewWindow {
            cn_start,
            cn_end,
            pool_cutoff: skins::review_pool_cutoff(cn_start),
            resolution,
        })
        .collect()
}

pub async fn get_skins(state: &AppState) -> Result<SkinsResponse, ApiError> {
    let key = cache_key("release:skins", estimate::window_from_env());
    if let Some(c) = state.cache.get::<SkinsResponse>(&key).await {
        return Ok(c);
    }
    let p = Planner::load(state).await?;
    let out = build(&p);
    state.cache.set(&key, &out).await;
    Ok(out)
}

fn build(p: &Planner) -> SkinsResponse {
    let (cn, en) = (&*p.ctx.cn, &*p.ctx.en);
    let names = Names::new(p, TranslationMemory::build(cn, en, &HashMap::new()));
    let mut art = GroupArt {
        ctx: &p.ctx,
        map: HashMap::new(),
    };
    let (en_groups, en_batches) = skins::group_histories(en, &skins::batch_families(en, cn));
    let cn_families = skins::batch_families(cn, en);
    let (cn_groups, cn_batches) = skins::group_histories(cn, &cn_families);
    let skipped = SkippedTags::build(cn, en);
    let is_passed = |l: &SkinListing| skipped.passed(l.tag_id.as_deref(), l.start_time);
    let cn_kept = cn
        .skin_listings
        .iter()
        .any(is_passed)
        .then(|| skins::group_histories_without(cn, &cn_families, is_passed).0);
    let review_pool = review_pool(p, &names);
    SkinsResponse {
        anniversaries: skins::anniversary_models(&en_groups, p.now),
        batches: batches(p, &names, &en_batches, &cn_batches, &skipped),
        new_skins: new_skins(p, &names, &mut art),
        rerun_forecasts: reruns(
            p,
            &names,
            &en_groups,
            &cn_groups,
            cn_kept.as_deref(),
            &mut art,
        ),
        reviews: reviews(p, &names, &skipped),
        review_pool,
        group_art: art.map,
        model: p.models.general.clone(),
        yearly: p.models.yearly.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forecast(group: &str, next: Resolution, basis: RerunBasis) -> RerunForecast {
        RerunForecast {
            skin_group_id: group.to_string(),
            skin_group_name: group.to_string(),
            skin_ids: vec![],
            skins: vec![],
            windows: vec![],
            last_seen: 0,
            next,
            basis,
        }
    }

    fn cadence() -> RerunBasis {
        RerunBasis::Cadence {
            en_last: 0,
            n: 1,
            median_days: 343.0,
            p25_days: 182.0,
            p75_days: 362.0,
            own_gap_days: None,
        }
    }

    fn listing() -> RerunBasis {
        RerunBasis::CnListing {
            cn_start: 0,
            cn_end: 0,
            anchor: None,
        }
    }

    fn estimated(en_start: i64) -> Resolution {
        Resolution::Estimated {
            en_start,
            lo: en_start,
            hi: en_start,
        }
    }

    fn kinds(out: &[RerunForecast]) -> Vec<(&str, bool)> {
        out.iter()
            .map(|f| {
                (
                    f.skin_group_id.as_str(),
                    matches!(f.basis, RerunBasis::Cadence { .. }),
                )
            })
            .collect()
    }

    #[test]
    fn cadence_yields_to_a_listing_dated_forecast() {
        // Achievement Star/III on 2026-10-09: cadence put it on a Mar 2 store
        // sale, the CN listing on the Feb 25 event, and Leizi sold twice.
        let mut out = vec![
            forecast("2024#game", estimated(1_803_992_400), cadence()),
            forecast("2024#game", estimated(1_804_107_600), listing()),
            forecast("2025#game", estimated(1_816_862_400), cadence()),
        ];
        one_guess_per_group(&mut out);
        assert_eq!(kinds(&out), vec![("2024#game", false), ("2025#game", true)]);
    }

    #[test]
    fn a_group_keeps_one_cadence_guess() {
        let mut out = vec![
            forecast("g", estimated(1), cadence()),
            forecast("g", estimated(1), cadence()),
        ];
        one_guess_per_group(&mut out);
        assert_eq!(kinds(&out), vec![("g", true)]);
    }

    #[test]
    fn an_unlisted_forecast_does_not_drop_the_guess() {
        let mut out = vec![
            forecast("g", Resolution::Unlisted, listing()),
            forecast("g", estimated(1), cadence()),
        ];
        one_guess_per_group(&mut out);
        assert_eq!(kinds(&out), vec![("g", false), ("g", true)]);
    }
}
