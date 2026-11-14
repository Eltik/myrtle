use std::collections::{BTreeMap, HashMap, HashSet};

use myrtle_gamedata::model::GameModel;

use serde::{Deserialize, Serialize};

use myrtle_gamedata_raw::tables::{
    character::{Brand, Skin},
    item::{ListingKind, SkinListing, SkinWindow},
};

use super::{Switches, estimate::percentile, types::Resolution};

const SECS_PER_DAY: f64 = 86_400.0;
const SAME_WINDOW_SECS: i64 = 20 * 86_400;

fn is_skin_shop_sold(skin: &Skin, shop_skin_ids: &HashSet<&str>) -> bool {
    shop_skin_ids.contains(skin.skin_id.as_str())
        || matches!(
            skin.display_skin.obtain_approach.as_deref(),
            Some("Store" | "采购中心")
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaleKind {
    Listing,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleWindow {
    pub start_time: i64,
    pub end_time: i64,
    pub kind: SaleKind,
}

#[must_use]
pub fn name_base(normalized: &str) -> &str {
    match normalized.rsplit_once('/') {
        Some((head, tail)) if !tail.is_empty() && tail.chars().all(|c| "ivxlcdm".contains(c)) => {
            head.trim_end()
        }
        _ => normalized,
    }
}

#[must_use]
pub fn orphan_families(gd: &GameModel) -> HashMap<String, Vec<String>> {
    let mut group_bases: HashSet<String> = HashSet::new();
    for s in gd.skins.char_skins.values() {
        if !s.display_skin.skin_group_id.is_empty() {
            group_bases
                .insert(name_base(&normalize_name(&s.display_skin.skin_group_name)).to_string());
        }
    }
    let mut brand_names: HashSet<String> = HashSet::new();
    for b in gd.skins.brand_list.values() {
        brand_names.insert(normalize_name(&b.brand_name));
        brand_names.insert(normalize_name(&b.brand_capital_name));
    }
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for l in &gd.skin_listings {
        if let ListingKind::Group { name, img_id, .. } = &l.kind {
            let key = normalize_name(name);
            let base = name_base(&key);
            if base != key && !group_bases.contains(base) && !brand_names.contains(base) {
                let e = out.entry(base.to_string()).or_default();
                if let Some(i) = img_id {
                    e.push(i.clone());
                }
            }
        }
    }
    out
}

#[must_use]
pub fn batch_families(this: &GameModel, other: &GameModel) -> HashSet<String> {
    let mine = orphan_families(this);
    let theirs: HashSet<String> = orphan_families(other).into_values().flatten().collect();
    mine.into_iter()
        .filter(|(_, imgs)| imgs.iter().any(|i| theirs.contains(i)))
        .map(|(base, _)| base)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub name: String,
    pub img_id: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub tag_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnniversaryStats {
    pub year: u32,
    pub eligible: usize,
    pub observed: usize,
    pub dev_p25_days: f64,
    pub dev_median_days: f64,
    pub dev_p75_days: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RerunBasis {
    Cadence {
        en_last: i64,
        n: usize,
        median_days: f64,
        p25_days: f64,
        p75_days: f64,
        own_gap_days: Option<f64>,
    },
    CnListing {
        cn_start: i64,
        cn_end: i64,
        anchor: Option<super::types::EventAnchor>,
    },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupHistory {
    pub skin_group_id: String,
    pub skin_group_name: String,
    pub brand_id: Option<String>,
    pub skin_ids: Vec<String>,
    pub debut: i64,
    pub windows: Vec<SaleWindow>,
}

impl GroupHistory {
    pub fn listings(&self) -> impl Iterator<Item = &SaleWindow> {
        self.windows.iter().filter(|w| w.kind == SaleKind::Listing)
    }

    #[must_use]
    pub fn last_seen(&self) -> Option<i64> {
        self.windows.iter().map(|w| w.start_time).max()
    }
}

fn roman(mut n: u32) -> String {
    const T: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (v, s) in T {
        while n >= *v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

const UNICODE_ROMAN: &[(char, &str)] = &[
    ('Ⅰ', "I"),
    ('Ⅱ', "II"),
    ('Ⅲ', "III"),
    ('Ⅳ', "IV"),
    ('Ⅴ', "V"),
    ('Ⅵ', "VI"),
    ('Ⅶ', "VII"),
    ('Ⅷ', "VIII"),
    ('Ⅸ', "IX"),
    ('Ⅹ', "X"),
    ('Ⅺ', "XI"),
    ('Ⅻ', "XII"),
];

fn is_roman(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| "ivxlcdm".contains(c))
}

#[must_use]
pub fn normalize_name(name: &str) -> String {
    let mut name: String = name.trim().replace('™', "");
    for (u, ascii) in UNICODE_ROMAN {
        name = name.replace(*u, ascii);
    }
    let name = name.trim();
    let normalized = match name.rsplit_once('/') {
        Some((head, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => {
            format!("{}/{}", head.trim(), roman(tail.parse().unwrap_or(0)))
        }
        Some((head, tail)) => format!("{}/{}", head.trim(), tail.trim()),
        None => {
            let tail: String = name
                .chars()
                .rev()
                .take_while(|c| "IVXLCDM".contains(*c))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let head = name[..name.len() - tail.len()].trim();
            if is_roman(&tail.to_lowercase())
                && !head.is_empty()
                && !head.ends_with(|c: char| c.is_ascii_alphabetic())
            {
                format!("{head}/{tail}")
            } else {
                name.to_string()
            }
        }
    };
    normalized.to_lowercase()
}

fn name_variants(key: &str) -> Vec<String> {
    let mut out = vec![key.to_string()];
    match key.rsplit_once('/') {
        Some((head, "i")) => out.push(head.to_string()),
        Some((_, tail)) if is_roman(tail) => {}
        _ => out.push(format!("{key}/i")),
    }
    out
}

fn push_window(windows: &mut Vec<SaleWindow>, w: SaleWindow) {
    if let Some(existing) = windows
        .iter_mut()
        .find(|e| e.kind == w.kind && (e.start_time - w.start_time).abs() < SAME_WINDOW_SECS)
    {
        existing.start_time = existing.start_time.min(w.start_time);
        existing.end_time = existing.end_time.max(w.end_time);
        return;
    }
    windows.push(w);
}

#[must_use]
pub fn group_histories(
    gd: &GameModel,
    batch_families: &HashSet<String>,
    switches: Switches,
) -> (Vec<GroupHistory>, Vec<Batch>) {
    group_histories_without(gd, batch_families, switches, |_| false)
}

pub fn group_histories_without(
    gd: &GameModel,
    batch_families: &HashSet<String>,
    switches: Switches,
    skip: impl Fn(&SkinListing) -> bool,
) -> (Vec<GroupHistory>, Vec<Batch>) {
    let mut batches: Vec<Batch> = Vec::new();
    let mut groups: BTreeMap<String, GroupHistory> = BTreeMap::new();
    let mut skin_to_group: HashMap<&str, &str> = HashMap::new();
    let mut shop_skin_ids: HashSet<&str> = gd
        .skin_windows
        .iter()
        .map(|window| window.skin_id.as_str())
        .collect();
    for listing in &gd.skin_listings {
        if let ListingKind::Group { skin_ids, .. } = &listing.kind {
            shop_skin_ids.extend(skin_ids.iter().map(String::as_str));
        }
    }
    let keep_unsold = switches.keep_unsold_skins;
    let mut review_from: HashMap<&str, i64> = HashMap::new();
    for s in gd.skins.char_skins.values() {
        let gid = s.display_skin.skin_group_id.as_str();
        if gid.is_empty() || (!keep_unsold && !is_skin_shop_sold(s, &shop_skin_ids)) {
            continue;
        }
        skin_to_group.insert(s.skin_id.as_str(), gid);
        let t = s.display_skin.get_time;
        if t > 0 && review_eligible(s, &gd.skins.brand_list) {
            review_from
                .entry(gid)
                .and_modify(|from| *from = (*from).min(t))
                .or_insert(t);
        }
        let g = groups
            .entry(gid.to_string())
            .or_insert_with(|| GroupHistory {
                skin_group_id: gid.to_string(),
                skin_group_name: s.display_skin.skin_group_name.clone(),
                ..Default::default()
            });
        g.skin_ids.push(s.skin_id.clone());
        let t = s.display_skin.get_time;
        if t > 0 && (g.debut == 0 || t < g.debut) {
            g.debut = t;
        }
    }
    let mut group_brand: HashMap<String, String> = HashMap::new();
    for b in gd.skins.brand_list.values() {
        for g in &b.group_list {
            group_brand.insert(g.skin_group_id.clone(), b.brand_id.clone());
        }
        for k in &b.kv_img_id_list {
            group_brand
                .entry(k.linked_skin_group_id.clone())
                .or_insert_with(|| b.brand_id.clone());
        }
    }
    for (gid, g) in &mut groups {
        g.brand_id = group_brand.get(gid).cloned();
    }
    let group_by_name: HashMap<String, Vec<String>> = {
        let mut m: HashMap<String, Vec<String>> = HashMap::new();
        for g in groups.values() {
            m.entry(normalize_name(&g.skin_group_name))
                .or_default()
                .push(g.skin_group_id.clone());
        }
        m
    };

    for l in gd.skin_listings.iter().filter(|l| !skip(l)) {
        let window = SaleWindow {
            start_time: l.start_time,
            end_time: l.end_time,
            kind: SaleKind::Listing,
        };
        let targets: Vec<String> = match &l.kind {
            ListingKind::Review => {
                let every_group = switches.reviews_on_every_group;
                for (gid, g) in &mut groups {
                    let stocked = if every_group {
                        g.debut > 0 && g.debut <= l.start_time
                    } else {
                        review_from
                            .get(gid.as_str())
                            .is_some_and(|from| *from <= review_pool_cutoff(l.start_time))
                    };
                    if stocked {
                        push_window(
                            &mut g.windows,
                            SaleWindow {
                                kind: SaleKind::Review,
                                ..window
                            },
                        );
                    }
                }
                continue;
            }
            ListingKind::Group {
                name,
                skin_ids,
                img_id,
            } => {
                let mut t: HashSet<String> = skin_ids
                    .iter()
                    .filter_map(|s| skin_to_group.get(s.as_str()))
                    .map(|g| (*g).to_string())
                    .collect();
                if t.is_empty() {
                    let key = normalize_name(name);
                    if batch_families.contains(name_base(&key)) {
                        batches.push(Batch {
                            name: name.clone(),
                            img_id: img_id.clone(),
                            start_time: l.start_time,
                            end_time: l.end_time,
                            tag_id: l.tag_id.clone(),
                        });
                    } else if let Some(gs) = name_variants(&key)
                        .iter()
                        .find_map(|k| group_by_name.get(k))
                    {
                        t.extend(gs.iter().cloned());
                    }
                }
                t.into_iter().collect()
            }
        };
        for gid in targets {
            if let Some(g) = groups.get_mut(&gid) {
                push_window(&mut g.windows, window);
            }
        }
    }
    let mut by_skin: HashMap<&str, Vec<&SkinWindow>> = HashMap::new();
    for w in &gd.skin_windows {
        by_skin.entry(w.skin_id.as_str()).or_default().push(w);
    }
    for (skin, gid) in &skin_to_group {
        if let (Some(ws), Some(g)) = (by_skin.get(skin), groups.get_mut(*gid)) {
            for w in ws {
                push_window(
                    &mut g.windows,
                    SaleWindow {
                        start_time: w.start_time,
                        end_time: w.end_time,
                        kind: SaleKind::Listing,
                    },
                );
            }
        }
    }

    let mut out: Vec<GroupHistory> = groups
        .into_values()
        .filter(|g| !g.windows.is_empty())
        .collect();
    for g in &mut out {
        g.skin_ids.sort();
        g.windows.sort_by_key(|w| (w.start_time, w.end_time));
    }
    batches.sort_by_key(|b| (b.start_time, b.name.clone()));
    batches.dedup_by(|a, b| {
        (&a.name, &a.img_id, a.start_time, a.end_time)
            == (&b.name, &b.img_id, b.start_time, b.end_time)
    });
    (out, batches)
}

const YEAR_SECS: f64 = 365.25 * 86_400.0;
const ANNIVERSARY_GRACE_SECS: i64 = 60 * 86_400;
pub const ANNIVERSARY_MIN_ELIGIBLE: usize = 20;
pub const ANNIVERSARY_MAX_YEARS: u32 = 6;

fn anniversary_at(debut: i64, k: u32) -> i64 {
    debut + (f64::from(k) * YEAR_SECS).round() as i64
}

#[must_use]
pub fn anniversary_stats(groups: &[GroupHistory], k: u32, now: i64) -> AnniversaryStats {
    let mut eligible = 0usize;
    let mut devs: Vec<f64> = Vec::new();
    for g in groups {
        if g.debut <= 0 {
            continue;
        }
        let target = anniversary_at(g.debut, k);
        if target + ANNIVERSARY_GRACE_SECS >= now {
            continue;
        }
        eligible += 1;
        if let Some(w) = g
            .listings()
            .filter(|w| (w.start_time - target).abs() < ANNIVERSARY_GRACE_SECS)
            .min_by_key(|w| (w.start_time - target).abs())
        {
            devs.push((w.start_time - target) as f64 / SECS_PER_DAY);
        }
    }
    devs.sort_by(|a, b| a.partial_cmp(b).expect("finite deviation"));
    AnniversaryStats {
        year: k,
        eligible,
        observed: devs.len(),
        dev_p25_days: percentile(&devs, 0.25),
        dev_median_days: percentile(&devs, 0.5),
        dev_p75_days: percentile(&devs, 0.75),
    }
}

#[must_use]
pub fn anniversary_models(groups: &[GroupHistory], now: i64) -> Vec<AnniversaryStats> {
    (1..=ANNIVERSARY_MAX_YEARS)
        .map(|k| anniversary_stats(groups, k, now))
        .take_while(|st| st.eligible >= ANNIVERSARY_MIN_ELIGIBLE)
        .collect()
}

pub const RERUN_MATCH_SECS: i64 = 31 * 86_400;
pub const REVIEW_LAG_MIN_SECS: i64 = 90 * 86_400;
pub const REVIEW_LAG_MAX_SECS: i64 = 270 * 86_400;
pub const REVIEW_POOL_AGE_SECS: i64 = 730 * 86_400;
const REVIEW_BARRED_TAG: &str = "活动获得";
const CROSSOVER_BRAND: &str = "crossover";

#[must_use]
pub fn review_windows(gd: &GameModel) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = gd
        .skin_listings
        .iter()
        .filter(|l| l.kind == ListingKind::Review)
        .map(|l| (l.start_time, l.end_time))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[must_use]
pub fn pair_reviews(cn: &[(i64, i64)], en: &[(i64, i64)]) -> Vec<Option<(i64, i64)>> {
    let mut next = 0;
    cn.iter()
        .map(|&(cn_start, _)| {
            let hit = en[next..].iter().position(|&(en_start, _)| {
                (REVIEW_LAG_MIN_SECS..=REVIEW_LAG_MAX_SECS).contains(&(en_start - cn_start))
            });
            hit.map(|i| {
                next += i + 1;
                en[next - 1]
            })
        })
        .collect()
}

#[must_use]
pub const fn review_pool_cutoff(cn_start: i64) -> i64 {
    cn_start - REVIEW_POOL_AGE_SECS
}

#[must_use]
pub fn review_eligible(skin: &Skin, brands: &HashMap<String, Brand>) -> bool {
    let ds = &skin.display_skin;
    super::prices::plain_store(skin)
        && ds.display_tag_id.as_deref() != Some(REVIEW_BARRED_TAG)
        && !brands.get(CROSSOVER_BRAND).is_some_and(|b| {
            b.group_list
                .iter()
                .any(|g| g.skin_group_id == ds.skin_group_id)
        })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CadenceModel {
    pub n: usize,
    pub median_days: f64,
    pub p25_days: f64,
    pub p75_days: f64,
}

#[must_use]
pub fn cadence_model(groups: &[GroupHistory]) -> CadenceModel {
    let mut gaps: Vec<f64> = Vec::new();
    for g in groups {
        let mut last: Option<i64> = None;
        for w in g.listings() {
            if let Some(prev) = last
                && w.start_time - prev > SAME_WINDOW_SECS
            {
                gaps.push((w.start_time - prev) as f64 / SECS_PER_DAY);
            }
            if last.is_none_or(|prev| w.start_time - prev > SAME_WINDOW_SECS) {
                last = Some(w.start_time);
            }
        }
    }
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if gaps.is_empty() {
        return CadenceModel {
            n: 0,
            median_days: 0.0,
            p25_days: 0.0,
            p75_days: 0.0,
        };
    }
    CadenceModel {
        n: gaps.len(),
        median_days: percentile(&gaps, 0.5),
        p25_days: percentile(&gaps, 0.25),
        p75_days: percentile(&gaps, 0.75),
    }
}

#[must_use]
pub fn next_by_cadence(en_last: i64, model: &CadenceModel) -> Resolution {
    let shift = |d: f64| en_last + (d * SECS_PER_DAY).round() as i64;
    Resolution::Estimated {
        en_start: shift(model.median_days),
        lo: shift(model.p25_days),
        hi: shift(model.p75_days),
    }
}

fn merged_listing_starts(group: &GroupHistory) -> Vec<i64> {
    let mut starts: Vec<i64> = Vec::new();
    for w in group.listings() {
        if starts
            .last()
            .is_none_or(|prev| w.start_time - prev > SAME_WINDOW_SECS)
        {
            starts.push(w.start_time);
        }
    }
    starts
}

#[must_use]
pub fn next_by_own_cadence(
    group: &GroupHistory,
    model: &CadenceModel,
) -> Option<(i64, Resolution, Option<f64>)> {
    let starts = merged_listing_starts(group);
    let en_last = *starts.last()?;
    let own_gap = starts
        .windows(2)
        .last()
        .map(|p| ((p[1] - p[0]) as f64 / SECS_PER_DAY).clamp(model.p25_days, model.p75_days));
    let resolution = match own_gap {
        Some(gap) => {
            let shift = |d: f64| en_last + (d * SECS_PER_DAY).round() as i64;
            Resolution::Estimated {
                en_start: shift(gap),
                lo: shift(gap - (model.median_days - model.p25_days)),
                hi: shift(gap + (model.p75_days - model.median_days)),
            }
        }
        None => next_by_cadence(en_last, model),
    };
    Some((en_last, resolution, own_gap))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRerun<'a> {
    pub group: &'a GroupHistory,
    pub cn_window: SaleWindow,
}

#[must_use]
pub fn pending_cn_reruns(
    cn_groups: &[GroupHistory],
    now: i64,
    lookback_secs: i64,
) -> Vec<PendingRerun<'_>> {
    let mut out = Vec::new();
    for g in cn_groups {
        if g.debut <= 0 {
            continue;
        }
        for w in g.listings() {
            if w.start_time >= now - lookback_secs && w.start_time > g.debut + SAME_WINDOW_SECS {
                out.push(PendingRerun {
                    group: g,
                    cn_window: *w,
                });
            }
        }
    }
    out.sort_by(|a, b| {
        a.cn_window
            .start_time
            .cmp(&b.cn_window.start_time)
            .then(a.group.skin_group_id.cmp(&b.group.skin_group_id))
    });
    out
}

#[must_use]
pub fn listed_near(group: Option<&GroupHistory>, start: i64) -> bool {
    group.is_some_and(|g| {
        g.listings()
            .any(|w| (w.start_time - start).abs() < SAME_WINDOW_SECS)
    })
}

pub fn match_en_listing(
    en: Option<&GroupHistory>,
    expected_start: i64,
    consumed: &mut HashSet<(String, i64)>,
) -> Option<SaleWindow> {
    let g = en?;
    let w = g
        .listings()
        .filter(|w| (w.start_time - expected_start).abs() <= RERUN_MATCH_SECS)
        .find(|w| !consumed.contains(&(g.skin_group_id.clone(), w.start_time)))
        .copied()?;
    consumed.insert((g.skin_group_id.clone(), w.start_time));
    Some(w)
}
