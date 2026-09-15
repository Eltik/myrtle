use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::{
    GameData,
    shop::{ListingKind, SkinWindow},
    skin::{Brand, Skin},
};

use super::{estimate::percentile, types::Resolution};

const SECS_PER_DAY: f64 = 86_400.0;
const SAME_WINDOW_SECS: i64 = 20 * 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SaleKind {
    Listing,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SaleWindow {
    #[ts(type = "number")]
    pub start_time: i64,
    #[ts(type = "number")]
    pub end_time: i64,
    pub kind: SaleKind,
}

pub fn name_base(normalized: &str) -> &str {
    match normalized.rsplit_once('/') {
        Some((head, tail)) if !tail.is_empty() && tail.chars().all(|c| "ivxlcdm".contains(c)) => {
            head.trim_end()
        }
        _ => normalized,
    }
}

pub fn orphan_families(gd: &GameData) -> HashMap<String, Vec<String>> {
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

pub fn batch_families(this: &GameData, other: &GameData) -> HashSet<String> {
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AnniversaryStats {
    pub year: u32,
    pub eligible: usize,
    pub observed: usize,
    pub dev_p25_days: f64,
    pub dev_median_days: f64,
    pub dev_p75_days: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum RerunBasis {
    Cadence {
        #[ts(type = "number")]
        en_last: i64,
        n: usize,
        median_days: f64,
        p25_days: f64,
        p75_days: f64,
    },
    CnListing {
        #[ts(type = "number")]
        cn_start: i64,
        #[ts(type = "number")]
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

/// A listing's name, its group-one spelling and its bare spelling: brands
/// name group one both ways (`斗争血脉/I` but `闪耀阶梯`).
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

pub fn group_histories(
    gd: &GameData,
    batch_families: &HashSet<String>,
) -> (Vec<GroupHistory>, Vec<Batch>) {
    let mut batches: Vec<Batch> = Vec::new();
    let mut groups: BTreeMap<String, GroupHistory> = BTreeMap::new();
    let mut skin_to_group: HashMap<&str, &str> = HashMap::new();
    for s in gd.skins.char_skins.values() {
        let gid = s.display_skin.skin_group_id.as_str();
        if gid.is_empty() {
            continue;
        }
        skin_to_group.insert(s.skin_id.as_str(), gid);
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

    for l in &gd.skin_listings {
        let window = SaleWindow {
            start_time: l.start_time,
            end_time: l.end_time,
            kind: SaleKind::Listing,
        };
        let targets: Vec<String> = match &l.kind {
            ListingKind::Review => {
                for g in groups.values_mut() {
                    if g.debut > 0 && g.debut <= l.start_time {
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
                        });
                    } else if let Some(gs) = name_variants(&key)
                        .iter()
                        .find_map(|k| group_by_name.get(k))
                    {
                        t.extend(gs.iter().cloned());
                    } else if !name.is_empty() {
                        tracing::debug!(name, "skin listing matched no group or batch");
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
    batches.dedup();
    (out, batches)
}

const YEAR_SECS: f64 = 365.25 * 86_400.0;
const ANNIVERSARY_GRACE_SECS: i64 = 60 * 86_400;
pub const ANNIVERSARY_MIN_ELIGIBLE: usize = 20;
pub const ANNIVERSARY_MAX_YEARS: u32 = 6;

fn anniversary_at(debut: i64, k: u32) -> i64 {
    debut + (f64::from(k) * YEAR_SECS).round() as i64
}

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

pub fn anniversary_models(groups: &[GroupHistory], now: i64) -> Vec<AnniversaryStats> {
    (1..=ANNIVERSARY_MAX_YEARS)
        .map(|k| anniversary_stats(groups, k, now))
        .take_while(|st| st.eligible >= ANNIVERSARY_MIN_ELIGIBLE)
        .collect()
}

pub const RERUN_MATCH_SECS: i64 = 31 * 86_400;
pub const REVIEW_LAG_MIN_SECS: i64 = 90 * 86_400;
pub const REVIEW_LAG_MAX_SECS: i64 = 270 * 86_400;
/// How old an outfit is when a Fashion Review first stocks it. The review is
/// cumulative: every edition adds the outfits that turned about two years old
/// since the last one and keeps everything older. Measured against the
/// twenty CN editions on record (2021-11 to 2026-07): the newest outfit each
/// edition added was 723 to 821 days old, the oldest it left out 693 to 785,
/// so a fixed two years is right to within one release batch per edition.
pub const REVIEW_POOL_AGE_SECS: i64 = 730 * 86_400;
/// Store outfits that were never in a review (Fang's Cross-Cantabile and
/// Hibiscus's Nian) carry this tag despite their store obtain approach.
const REVIEW_BARRED_TAG: &str = "活动获得";
const CROSSOVER_BRAND: &str = "crossover";

pub fn review_windows(gd: &GameData) -> Vec<(i64, i64)> {
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

/// The newest release date a review starting at `cn_start` stocks.
pub const fn review_pool_cutoff(cn_start: i64) -> i64 {
    cn_start - REVIEW_POOL_AGE_SECS
}

/// Whether an outfit ever enters the Fashion Review: a plain-price store
/// outfit from a real brand. Crossover membership is the brand list's own
/// group roster (`BrandList.crossover.GroupList`); the group id suffix names
/// the partner, not the brand.
pub fn review_eligible(skin: &Skin, brands: &HashMap<String, Brand>) -> bool {
    let ds = &skin.display_skin;
    super::prices::store_price(skin) == super::prices::STORE
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

pub fn next_by_cadence(en_last: i64, model: &CadenceModel) -> Resolution {
    let shift = |d: f64| en_last + (d * SECS_PER_DAY).round() as i64;
    Resolution::Estimated {
        en_start: shift(model.median_days),
        lo: shift(model.p25_days),
        hi: shift(model.p75_days),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRerun<'a> {
    pub group: &'a GroupHistory,
    pub cn_window: SaleWindow,
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn reviews_pair_in_order_within_the_lag_band() {
        const D: i64 = 86_400;
        let cn = [
            (0, 27 * D),
            (90 * D, 117 * D),
            (180 * D, 207 * D),
            (270 * D, 297 * D),
        ];
        let en = [(180 * D, 208 * D), (272 * D, 300 * D)];
        let paired = super::pair_reviews(&cn, &en);
        assert_eq!(paired[0], Some(en[0]), "180 d lag pairs");
        assert_eq!(
            paired[1],
            Some(en[1]),
            "182 d lag pairs with the next EN review"
        );
        assert_eq!(paired[2], None, "no EN review 90 to 270 d after it");
        assert_eq!(paired[3], None);
    }

    #[test]
    fn review_pool_is_plain_price_brand_outfits_two_years_old() {
        let mut brands = HashMap::new();
        brands.insert(
            "crossover".to_owned(),
            Brand {
                brand_id: "crossover".into(),
                group_list: vec![BrandGroup {
                    skin_group_id: "2021#rainbow6".into(),
                    publish_time: 0,
                }],
                ..Default::default()
            },
        );
        let store = |group: &str, tag: Option<&str>| {
            let mut s = skin("a@x#1", group, 100 * D);
            s.display_skin.obtain_approach = Some("采购中心".into());
            s.display_skin.display_tag_id = tag.map(str::to_owned);
            s
        };
        assert!(review_eligible(&store("2021#epoque", None), &brands));
        assert!(
            !review_eligible(&store("2021#rainbow6", None), &brands),
            "a crossover group named by the brand roster"
        );
        assert!(
            !review_eligible(&store("2019#winter", Some("活动获得")), &brands),
            "the event-tagged store outfits never entered a review"
        );
        let mut dynamic = store("2021#epoque", None);
        dynamic.dyn_illust_id = Some("dyn".into());
        assert!(
            !review_eligible(&dynamic, &brands),
            "21 OP outfits rerun on their own"
        );
        assert_eq!(review_pool_cutoff(1000 * D), 270 * D);
    }

    use super::*;
    use crate::core::gamedata::types::{
        shop::SkinListing,
        skin::{Brand, BrandGroup, Skin},
    };

    const D: i64 = 86_400;

    fn skin(id: &str, group: &str, get_time: i64) -> Skin {
        let mut s = Skin {
            skin_id: id.into(),
            ..Default::default()
        };
        s.display_skin.skin_group_id = group.into();
        s.display_skin.skin_group_name = format!("Coast/{}", group.trim_start_matches('g'));
        s.display_skin.get_time = get_time;
        s
    }

    fn listing(start: i64, kind: ListingKind) -> SkinListing {
        SkinListing {
            start_time: start,
            end_time: start + 14 * D,
            kind,
        }
    }

    fn game() -> GameData {
        let mut gd = GameData::new();
        for (id, g, t) in [
            ("a@x#1", "g1", 100 * D),
            ("b@x#1", "g1", 100 * D),
            ("c@x#2", "g2", 200 * D),
            ("d@x#3", "g3", 900 * D),
        ] {
            gd.skins.char_skins.insert(id.into(), skin(id, g, t));
        }
        gd.skins.brand_list.insert(
            "coast".into(),
            Brand {
                brand_id: "coast".into(),
                brand_name: "Coast".into(),
                brand_capital_name: "COAST".into(),
                group_list: vec![
                    BrandGroup {
                        skin_group_id: "g1".into(),
                        publish_time: 0,
                    },
                    BrandGroup {
                        skin_group_id: "g2".into(),
                        publish_time: 0,
                    },
                    BrandGroup {
                        skin_group_id: "g3".into(),
                        publish_time: 0,
                    },
                ],
                ..Default::default()
            },
        );
        gd.skin_listings = vec![
            listing(
                100 * D,
                ListingKind::Group {
                    name: "Coast/1".into(),
                    skin_ids: vec![],
                    img_id: None,
                },
            ),
            listing(
                200 * D,
                ListingKind::Group {
                    name: "whatever".into(),
                    skin_ids: vec!["c@x#2".into()],
                    img_id: None,
                },
            ),
            listing(500 * D, ListingKind::Review),
            listing(
                465 * D,
                ListingKind::Group {
                    name: "Coast/I".into(),
                    skin_ids: vec![],
                    img_id: None,
                },
            ),
            listing(
                1000 * D,
                ListingKind::Group {
                    name: "COAST".into(),
                    skin_ids: vec![],
                    img_id: None,
                },
            ),
        ];
        gd.skin_windows = vec![SkinWindow {
            skin_id: "a@x#1".into(),
            start_time: 100 * D,
            end_time: 114 * D,
        }];
        gd
    }

    #[test]
    fn batch_families_need_an_orphan_on_both_servers() {
        let l = |name: &str, img: &str, start: i64| SkinListing {
            start_time: start,
            end_time: start + 14 * D,
            kind: ListingKind::Group {
                name: name.into(),
                skin_ids: vec![],
                img_id: Some(img.into()),
            },
        };
        let mut en = game();
        en.skin_listings = vec![
            l("Multi-theme Outfit/I", "tag610_a", 100 * D),
            l("Multi-theme Outfit/II", "tag733_a", 200 * D),
            l("Coast/IX", "tag1_a", 300 * D), // a group base on EN
            l("Naming Variant/II", "tag9_a", 400 * D), // orphan on EN, a group on CN
        ];
        let mut cn = game();
        cn.skin_listings = vec![
            l("多主题时装/I", "tag610_a", 90 * D),
            l("多主题时装/VIII", "tag1035_a", 900 * D),
            l("Coast/II", "tag9_a", 390 * D),
        ];
        assert_eq!(
            batch_families(&en, &cn),
            ["multi-theme outfit".to_string()].into_iter().collect()
        );
        assert_eq!(
            batch_families(&cn, &en),
            ["多主题时装".to_string()].into_iter().collect()
        );
        assert_eq!(name_base("epoque/xxxiv"), "epoque");
        assert_eq!(name_base("rhodes fashion review"), "rhodes fashion review");
    }

    #[test]
    fn names_normalise_numerals_and_case() {
        assert_eq!(normalize_name("Coral Coast/1"), "coral coast/i");
        assert_eq!(normalize_name("Coral Coast/IX"), "coral coast/ix");
        assert_eq!(normalize_name(" EPOQUE/34 "), "epoque/xxxiv");
        assert_eq!(normalize_name("Test Collection"), "test collection");
    }

    #[test]
    fn histories_merge_every_source() {
        let (gs, _) = group_histories(&game(), &HashSet::new());
        let g1 = gs.iter().find(|g| g.skin_group_id == "g1").unwrap();
        let g2 = gs.iter().find(|g| g.skin_group_id == "g2").unwrap();
        let g3 = gs.iter().find(|g| g.skin_group_id == "g3");
        let kinds = |g: &GroupHistory| {
            g.windows
                .iter()
                .map(|w| (w.start_time / D, w.kind))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            kinds(g1),
            vec![
                (100, SaleKind::Listing),
                (465, SaleKind::Listing),
                (500, SaleKind::Review),
                (1000, SaleKind::Listing)
            ]
        );
        assert_eq!(
            kinds(g2),
            vec![(200, SaleKind::Listing), (500, SaleKind::Review)],
            "a bare brand name is group one, not the whole brand"
        );
        assert!(g3.is_none(), "never listed, so no history");
        assert_eq!(g1.last_seen(), Some(1000 * D));
    }
}
