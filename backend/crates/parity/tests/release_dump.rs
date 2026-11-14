#![allow(clippy::unwrap_used)]

use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use myrtle_engine_release::align::{
    align_pools, independent_rules, rule_independence, tolerance_days,
};
use myrtle_engine_release::estimate::{
    StageRuns, activity_pairs, backtest, build_lag_model, build_yearly_model, en_activity_index,
    estimate, shift_resolution, split_yearly, yearly_types,
};
use myrtle_engine_release::operator_changes::cn_changes;
use myrtle_engine_release::prices::{
    own_voice, plain_store, skin_price, sold_in_store, store_price,
};
use myrtle_engine_release::skins::{
    Batch, GroupHistory, anniversary_models, batch_families, cadence_model, group_histories,
    next_by_own_cadence, orphan_families, pair_reviews, pending_cn_reruns, review_windows,
};
use myrtle_engine_release::skipped::SkippedTags;
use myrtle_engine_release::translate::{
    TranslationMemory, operator_name, resolve as resolve_name, resolve_group,
};
use myrtle_engine_release::{OverrideRow, Switches, kinds, override_index, resolve_reviews};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel as Data;
use myrtle_gamedata_raw::{DirSource, load as load_raw};
use myrtle_types::Server;

const KIND_REVIEW: &str = kinds::KIND_REVIEW;

fn load(root: &Path, dir: &str) -> Option<Data> {
    let root = root.join(dir);
    if !root.join("gamedata/excel").is_dir() {
        return None;
    }
    let server = match dir {
        "en" => Server::EN,
        "cn" => Server::CN,
        "jp" => Server::JP,
        _ => Server::KR,
    };
    let raw = load_raw(&DirSource { root }, server).unwrap();
    Some(compile(raw, &CompileOptions::new(server)).unwrap().model)
}

/// The switches as the app will read them: through `Switches::from_vars`.
fn switches(sw: Sw) -> Switches {
    Switches::from_vars(|key| sw.var(key))
}

fn window(sw: Sw) -> usize {
    switches(sw).lag_window
}

fn half_life(sw: Sw) -> f64 {
    switches(sw).lag_half_life
}

fn histories(gd: &Data, families: &HashSet<String>, sw: Sw) -> (Vec<GroupHistory>, Vec<Batch>) {
    group_histories(gd, families, switches(sw))
}

fn skipped_tags(cn: &Data, en: &Data, sw: Sw) -> SkippedTags {
    SkippedTags::build(cn, en, switches(sw))
}

fn ov(kind: &str, id: &str, start: Option<i64>) -> OverrideRow {
    OverrideRow {
        kind: kind.into(),
        cn_id: id.into(),
        en_id: None,
        en_name: None,
        en_start: start,
        en_end: None,
        featured_chars: None,
        source: "manual".into(),
        note: String::new(),
    }
}

/// A fixed "now", so the dump does not move with the clock: 2026-11-07 UTC.
const NOW: i64 = 1_794_000_000;
const DAY: i64 = 86_400;

/// One setting of the planner's switches, as the variables that produce it.
#[derive(Clone, Copy)]
struct Sw {
    name: &'static str,
    vars: &'static [(&'static str, &'static str)],
}

impl Sw {
    fn var(self, key: &str) -> Option<String> {
        self.vars
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| (*v).to_owned())
    }
}

const SHIPPED: Sw = Sw {
    name: "shipped",
    vars: &[],
};
const VARIANTS: [Sw; 5] = [
    SHIPPED,
    Sw {
        name: "lag",
        vars: &[("RELEASE_LAG_WINDOW", "6"), ("RELEASE_LAG_HALF_LIFE", "0")],
    },
    Sw {
        name: "unsold",
        vars: &[("RELEASE_KEEP_UNSOLD_SKINS", "1")],
    },
    Sw {
        name: "reviews",
        vars: &[("RELEASE_REVIEWS_ALL_GROUPS", "1")],
    },
    Sw {
        name: "tags",
        vars: &[("RELEASE_TRUST_SKIPPED_TAGS", "0")],
    },
];

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

fn sorted<T: Ord>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = items.into_iter().collect();
    v.sort();
    v
}

/// The lag model, its backtest and everything that reads them.
fn lag_section(tag: &str, cn: &Data, en: &Data, sw: Sw, text: &mut String) {
    let window = window(sw);
    let half_life = half_life(sw);
    writeln!(text, "{tag} switches\t{window} {half_life:?}").unwrap();
    let mut pairs = activity_pairs(cn, en);
    pairs.sort_by(|a, b| (a.cn_start, &a.cn_id).cmp(&(b.cn_start, &b.cn_id)));
    let yearly = yearly_types(&pairs);
    let (general, yearly_pairs) = split_yearly(&pairs, &yearly);
    writeln!(text, "{tag} yearly types\t{yearly:?}").unwrap();
    let lag = build_lag_model(&general, window, half_life);
    writeln!(text, "{tag} lag\t{}", json(&lag)).unwrap();
    writeln!(
        text,
        "{tag} lag yearly\t{}",
        json(&build_yearly_model(&yearly_pairs))
    )
    .unwrap();
    for since in [0, NOW - 730 * DAY, NOW - 365 * DAY] {
        writeln!(
            text,
            "{tag} backtest {since}\t{}",
            json(&backtest(&general, window, half_life, since))
        )
        .unwrap();
    }
    for cn_start in [0, NOW - 400 * DAY, NOW - 30 * DAY, NOW] {
        let r = estimate(&lag, cn_start);
        writeln!(
            text,
            "{tag} estimate {cn_start}\t{}\t{}",
            json(&r),
            json(&shift_resolution(r.clone(), 3 * DAY))
        )
        .unwrap();
    }
    writeln!(
        text,
        "{tag} tolerance\t{:?}",
        tolerance_days(&pairs, window, half_life)
    )
    .unwrap();
    let aligned = align_pools(cn, en, &pairs, window, half_life);
    for cn_pool in sorted(aligned.keys()) {
        writeln!(text, "{tag} pool {cn_pool}\t{:?}", aligned[cn_pool]).unwrap();
    }

    // Reviews resolve through the lag model, an override and the skipped tags.
    let cn_reviews = review_windows(cn);
    let en_reviews = review_windows(en);
    writeln!(
        text,
        "{tag} review pairs\t{:?}",
        pair_reviews(&cn_reviews, &en_reviews)
    )
    .unwrap();
    let rows: Vec<OverrideRow> = cn_reviews
        .iter()
        .rev()
        .take(2)
        .enumerate()
        .map(|(i, (start, _))| {
            ov(
                KIND_REVIEW,
                &start.to_string(),
                (i == 0).then_some(start + 170 * DAY),
            )
        })
        .collect();
    let overrides = override_index(&rows);
    let tags = skipped_tags(cn, en, sw);
    let passed = tags.review_starts(cn);
    writeln!(
        text,
        "{tag} review starts passed\t{:?}",
        sorted(passed.iter())
    )
    .unwrap();
    writeln!(
        text,
        "{tag} reviews\t{}",
        json(&resolve_reviews(
            &cn_reviews,
            &en_reviews,
            &overrides,
            &lag,
            &passed
        ))
    )
    .unwrap();
}

/// Outfit sale histories and the models read off them.
fn skin_section(tag: &str, cn: &Data, en: &Data, sw: Sw, text: &mut String) {
    let cn_families = batch_families(cn, en);
    let en_families = batch_families(en, cn);
    writeln!(text, "{tag} families cn\t{:?}", sorted(cn_families.iter())).unwrap();
    writeln!(text, "{tag} families en\t{:?}", sorted(en_families.iter())).unwrap();
    let (cn_groups, cn_batches) = histories(cn, &cn_families, sw);
    let (en_groups, en_batches) = histories(en, &en_families, sw);
    for (side, groups, batches) in [
        ("cn", &cn_groups, &cn_batches),
        ("en", &en_groups, &en_batches),
    ] {
        for g in groups {
            writeln!(text, "{tag} group {side} {}\t{g:?}", g.skin_group_id).unwrap();
        }
        for b in batches {
            writeln!(text, "{tag} batch {side}\t{b:?}").unwrap();
        }
        writeln!(
            text,
            "{tag} anniversary {side}\t{}",
            json(&anniversary_models(groups, NOW))
        )
        .unwrap();
    }
    let cadence = cadence_model(&en_groups);
    writeln!(text, "{tag} cadence\t{cadence:?}").unwrap();
    for g in &en_groups {
        if let Some(next) = next_by_own_cadence(g, &cadence) {
            writeln!(text, "{tag} next {}\t{next:?}", g.skin_group_id).unwrap();
        }
    }
    for rerun in pending_cn_reruns(&cn_groups, NOW, 400 * DAY) {
        writeln!(
            text,
            "{tag} pending {}\t{:?}",
            rerun.group.skin_group_id, rerun.cn_window
        )
        .unwrap();
    }
}

/// Which CN listings EN's calendar has already passed.
fn skipped_section(tag: &str, cn: &Data, en: &Data, sw: Sw, text: &mut String) {
    let tags = skipped_tags(cn, en, sw);
    let mut passed = Vec::new();
    for l in &cn.skin_listings {
        if tags.passed(l.tag_id.as_deref(), l.start_time) {
            passed.push((l.start_time, l.tag_id.clone()));
        }
    }
    passed.sort();
    writeln!(
        text,
        "{tag} listings passed\t{} of {}\t{passed:?}",
        passed.len(),
        cn.skin_listings.len()
    )
    .unwrap();
    writeln!(
        text,
        "{tag} review starts\t{:?}",
        sorted(tags.review_starts(cn))
    )
    .unwrap();
}

/// The parts no switch reaches.
fn fixed_section(tag: &str, cn: &Data, en: &Data, text: &mut String) {
    let mut rules = rule_independence(cn, en);
    rules.sort_by(|a, b| a.rule_type.cmp(&b.rule_type));
    writeln!(text, "{tag} rule independence\t{}", json(&rules)).unwrap();
    writeln!(
        text,
        "{tag} independent rules\t{:?}",
        sorted(independent_rules(cn, en))
    )
    .unwrap();
    let mut pairs = activity_pairs(cn, en);
    pairs.sort_by(|a, b| (a.cn_start, &a.cn_id).cmp(&(b.cn_start, &b.cn_id)));
    for p in &pairs {
        writeln!(text, "{tag} pair\t{p:?}").unwrap();
    }
    for (side, gd) in [("cn", cn), ("en", en)] {
        for (base, mut images) in sorted(orphan_families(gd)) {
            images.sort();
            writeln!(text, "{tag} orphans {side} {base}\t{images:?}").unwrap();
        }
        writeln!(
            text,
            "{tag} review windows {side}\t{:?}",
            review_windows(gd)
        )
        .unwrap();
    }
    for (id, entry) in sorted(en_activity_index(en)) {
        writeln!(text, "{tag} en activity {id}\t{entry:?}").unwrap();
    }

    // Names: the translation memory over the aligned pools, then each lookup.
    let aligned = align_pools(cn, en, &pairs, window(SHIPPED), half_life(SHIPPED));
    let memory = TranslationMemory::build(cn, en, &aligned);
    writeln!(text, "{tag} memory\t{}", memory.len()).unwrap();
    let mut names: Vec<&str> = cn.activities.values().map(|a| a.name.as_str()).collect();
    names.extend(
        cn.gacha
            .gacha_pool_client
            .iter()
            .map(|p| p.gacha_pool_name.as_str()),
    );
    names.extend(
        cn.skins
            .char_skins
            .values()
            .map(|s| s.display_skin.skin_group_name.as_str()),
    );
    let mut names = sorted(names);
    names.dedup();
    for name in names {
        let plain = resolve_name(&memory, name);
        let group = resolve_group(&memory, name);
        if plain.is_some() || group.is_some() {
            writeln!(text, "{tag} name {name}\t{plain:?}\t{group:?}").unwrap();
        }
    }
    for id in sorted(cn.operators.keys()) {
        if let Some(name) = operator_name(cn, en, id) {
            writeln!(text, "{tag} operator name {id}\t{name:?}").unwrap();
        }
    }

    // Where each CN pool's opening sits against the CN stage events.
    let runs = StageRuns::build(cn);
    let mut pools: Vec<(&str, i64)> = cn
        .gacha
        .gacha_pool_client
        .iter()
        .map(|p| (p.gacha_pool_id.as_str(), p.open_time))
        .collect();
    pools.sort_unstable();
    for (pool, open) in pools {
        if let Some(hit) = runs.anchor(open) {
            writeln!(
                text,
                "{tag} anchor {pool}\t{} {}",
                hit.activity.id, hit.offset_secs
            )
            .unwrap();
        }
    }

    // Outfit prices, CN side, and what changed on each operator.
    for id in sorted(cn.skins.char_skins.keys()) {
        let skin = &cn.skins.char_skins[id];
        writeln!(
            text,
            "{tag} price {id}\t{} {} {} {}\t{}",
            sold_in_store(skin),
            own_voice(skin),
            store_price(skin),
            plain_store(skin),
            json(&skin_price(skin, "obtain".to_owned()))
        )
        .unwrap();
    }
    for changed in cn_changes(en, cn) {
        writeln!(
            text,
            "{tag} changes {}\t{}",
            changed.id,
            json(&changed.changes)
        )
        .unwrap();
    }
}

fn dump(cn: &Data, other: &Data) -> String {
    let mut text = String::new();
    fixed_section("fixed", cn, other, &mut text);
    for sw in VARIANTS {
        let tag = sw.name;
        match tag {
            "shipped" => {
                lag_section(tag, cn, other, sw, &mut text);
                skin_section(tag, cn, other, sw, &mut text);
                skipped_section(tag, cn, other, sw, &mut text);
            }
            "lag" => lag_section(tag, cn, other, sw, &mut text),
            "tags" => {
                skipped_section(tag, cn, other, sw, &mut text);
                lag_section(tag, cn, other, sw, &mut text);
            }
            _ => skin_section(tag, cn, other, sw, &mut text),
        }
    }
    text
}

#[test]
fn release_dump() {
    let Ok(out) = std::env::var("RELEASE_DUMP") else {
        return;
    };
    let root = PathBuf::from(std::env::var("RELEASE_ROOT").unwrap());
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    let Some(cn) = load(&root, "cn") else {
        return;
    };
    for dir in ["en", "jp", "kr"] {
        let Some(other) = load(&root, dir) else {
            continue;
        };
        std::fs::write(out.join(format!("cn-{dir}.tsv")), dump(&cn, &other)).unwrap();
    }
}
