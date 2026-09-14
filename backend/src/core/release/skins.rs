use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::{
    GameData,
    shop::{ListingKind, SkinWindow},
};

use super::estimate::percentile;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum RerunBasis {
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

pub fn normalize_name(name: &str) -> String {
    let name = name.trim();
    let normalized = match name.rsplit_once('/') {
        Some((head, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => {
            format!("{head}/{}", roman(tail.parse().unwrap_or(0)))
        }
        _ => name.to_string(),
    };
    normalized.to_lowercase()
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
    let mut brand_by_name: HashMap<String, String> = HashMap::new();
    for b in gd.skins.brand_list.values() {
        brand_by_name.insert(normalize_name(&b.brand_name), b.brand_id.clone());
        brand_by_name.insert(normalize_name(&b.brand_capital_name), b.brand_id.clone());
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
                    let brand_hit = brand_by_name.get(&key).or_else(|| {
                        let head = key.split('/').next().unwrap_or("").trim();
                        brand_by_name.get(head).or_else(|| {
                            brand_by_name
                                .iter()
                                .filter(|(n, _)| n.len() >= 3 && key.contains(n.as_str()))
                                .map(|(_, b)| b)
                                .next()
                        })
                    });
                    if batch_families.contains(name_base(&key)) {
                        batches.push(Batch {
                            name: name.clone(),
                            img_id: img_id.clone(),
                            start_time: l.start_time,
                            end_time: l.end_time,
                        });
                    } else if let Some(gs) = group_by_name.get(&key) {
                        t.extend(gs.iter().cloned());
                    } else if let Some(brand) = brand_hit {
                        t.extend(
                            groups
                                .values()
                                .filter(|g| {
                                    g.brand_id.as_deref() == Some(brand)
                                        && g.debut > 0
                                        && g.debut <= l.start_time
                                })
                                .map(|g| g.skin_group_id.clone()),
                        );
                    } else if !name.is_empty() {
                        tracing::debug!(name, "skin listing matched no group, brand or batch");
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
        let g3 = gs.iter().find(|g| g.skin_group_id == "g3").unwrap();
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
            vec![
                (200, SaleKind::Listing),
                (500, SaleKind::Review),
                (1000, SaleKind::Listing)
            ]
        );
        assert_eq!(kinds(g3), vec![(1000, SaleKind::Listing)]);
        assert_eq!(g1.last_seen(), Some(1000 * D));
    }
}
