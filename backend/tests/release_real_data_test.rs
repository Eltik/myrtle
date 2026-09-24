use std::path::Path;

use backend::core::gamedata::assets::{AssetIndex, AssetKind};
use backend::core::{
    gamedata::{
        enrich::gacha::enrich_banners,
        tables::load_table,
        types::{
            GameData,
            activity::ActivityTableFile,
            event_shop::{EventShopFile, event_shop_path},
            gacha::GachaTableFile,
            gacha_detail::{PoolDetailFile, pool_detail_path},
            shop::ShopTableFile,
            skin::SkinTableFile,
        },
    },
    release::{AlignMethod, Resolution, align, estimate, skins},
    translate::{self, TranslationMemory},
};

fn root(server: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../assets/output/{server}"))
}

fn load(server: &str) -> Option<GameData> {
    let root = root(server);
    let dir = root.join("gamedata/excel");
    if !dir.join("activity_table.json").exists() {
        return None;
    }
    let mut gd = GameData::new();
    let chars: backend::core::gamedata::types::operator::CharacterTable =
        load_table(&dir, "character_table").ok()?;
    for (id, raw) in chars.characters {
        gd.operators.insert(
            id,
            backend::core::gamedata::types::operator::Operator {
                name: raw.name.clone(),
                appellation: raw.appellation.clone(),
                ..Default::default()
            },
        );
    }
    let act: ActivityTableFile = load_table(&dir, "activity_table").ok()?;
    gd.activities = act.basic_info;
    let gacha: GachaTableFile = load_table(&dir, "gacha_table").ok()?;
    gd.gacha = gacha.into_gacha_data();
    let details: Option<PoolDetailFile> = std::fs::read(pool_detail_path(&root))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());
    enrich_banners(&mut gd.gacha.gacha_pool_client, details.as_ref());
    let skin: SkinTableFile = load_table(&dir, "skin_table").ok()?;
    gd.skins = skin.into_skin_data();
    let shop: ShopTableFile = load_table(&dir, "shop_client_table").ok()?;
    gd.skin_listings = shop.into_skin_listings();
    gd.skin_windows = shop.into_skin_windows();
    gd.event_shops = std::fs::read(event_shop_path(&root))
        .ok()
        .and_then(|b| serde_json::from_slice::<EventShopFile>(&b).ok())
        .map(|f| f.shops)
        .unwrap_or_default();
    Some(gd)
}

/// Every activity that names a token shop and has started on the server,
/// split by whether the sidecar holds that shop.
fn shop_coverage(gd: &GameData, now: i64) -> (Vec<String>, Vec<String>) {
    let mut with = Vec::new();
    let mut without = Vec::new();
    for a in gd.activities.values() {
        if a.template_shop_id.is_none() || a.start_time <= 0 || a.start_time > now {
            continue;
        }
        if gd.event_shops.contains_key(&a.id) {
            with.push(a.id.clone());
        } else {
            without.push(a.id.clone());
        }
    }
    without.sort();
    (with, without)
}

#[test]
#[ignore = "needs the real gamedata extract"]
fn census_reproduces_on_the_live_extract() {
    let (Some(cn), Some(en)) = (load("cn"), load("en")) else {
        eprintln!("extract not present, skipping");
        return;
    };

    let all_pairs = estimate::activity_pairs(&cn, &en);
    let yearly_types = estimate::yearly_types(&all_pairs);
    let (pairs, yearly_pairs) = estimate::split_yearly(&all_pairs, &yearly_types);
    let yearly = estimate::build_yearly_model(&yearly_pairs);
    println!(
        "yearly types={:?} n={} median={:.1} p25={:.1} p75={:.1}",
        yearly_types, yearly.n, yearly.median_days, yearly.p25_days, yearly.p75_days
    );
    assert!(yearly_types.iter().any(|t| t == "APRIL_FOOL"));
    assert!((360.0..370.0).contains(&yearly.median_days));
    let model = estimate::build_lag_model(&pairs, 10);
    let backtest = estimate::backtest(&pairs, 10, 1_704_067_200);
    println!(
        "lag model n={} median={:.1} p25={:.1} p75={:.1}",
        model.n, model.median_days, model.p25_days, model.p75_days
    );
    println!(
        "backtest n={} median_abs_err={:.1} p75={:.1} p90={:.1} max={:.1} band_hit={:.3}",
        backtest.n,
        backtest.median_abs_err_days,
        backtest.p75_abs_err_days,
        backtest.p90_abs_err_days,
        backtest.max_abs_err_days,
        backtest.band_hit_rate
    );
    assert!(pairs.len() >= 130, "stage pairs = {}", pairs.len());
    assert!(all_pairs.len() > pairs.len());
    assert!(backtest.median_abs_err_days < 8.0);

    let aligned = align::align_pools(&cn, &en, &pairs, 10);
    println!(
        "content tolerance days = {:.1}",
        align::tolerance_days(&pairs, 10)
    );
    let all = estimate::backtest(&pairs, 10, 0);
    println!(
        "all-history backtest n={} median={:.1} p75={:.1} p90={:.1} max={:.1}",
        all.n,
        all.median_abs_err_days,
        all.p75_abs_err_days,
        all.p90_abs_err_days,
        all.max_abs_err_days
    );
    let content = aligned
        .values()
        .filter(|p| p.method == AlignMethod::Content)
        .count();
    let anchor = aligned
        .values()
        .filter(|p| p.method == AlignMethod::Anchor)
        .count();
    let cn_only_pools = cn.gacha.gacha_pool_client.len() - aligned.len();
    println!(
        "pools cn={} en={} aligned content={} anchor={} unaligned={}",
        cn.gacha.gacha_pool_client.len(),
        en.gacha.gacha_pool_client.len(),
        content,
        anchor,
        cn_only_pools
    );
    assert!(content >= 30, "content-aligned = {content}");
    let en_open: std::collections::HashMap<&str, i64> = en
        .gacha
        .gacha_pool_client
        .iter()
        .map(|p| (p.gacha_pool_id.as_str(), p.open_time))
        .collect();
    for method in [AlignMethod::Content, AlignMethod::Anchor] {
        let mut lags: Vec<f64> = cn
            .gacha
            .gacha_pool_client
            .iter()
            .filter_map(|p| aligned.get(&p.gacha_pool_id).map(|pr| (p, pr)))
            .filter(|(_, pr)| pr.method == method)
            .map(|(p, pr)| (en_open[pr.en_pool_id.as_str()] - p.open_time) as f64 / 86_400.0)
            .collect();
        lags.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |f: f64| lags[((lags.len() - 1) as f64 * f) as usize];
        println!(
            "{method:?} pair lag days: n={} min={:.1} p10={:.1} median={:.1} p90={:.1} max={:.1}",
            lags.len(),
            lags[0],
            q(0.1),
            q(0.5),
            q(0.9),
            lags[lags.len() - 1]
        );
        if method == AlignMethod::Content {
            for (p, pr) in cn
                .gacha
                .gacha_pool_client
                .iter()
                .filter_map(|p| aligned.get(&p.gacha_pool_id).map(|pr| (p, pr)))
                .filter(|(_, pr)| pr.method == method)
            {
                let lag = (en_open[pr.en_pool_id.as_str()] - p.open_time) as f64 / 86_400.0;
                if !(100.0..400.0).contains(&lag) {
                    println!(
                        "  OUTLIER {} -> {} lag={lag:.1} cn6={:?}",
                        p.gacha_pool_id, pr.en_pool_id, p.featured6
                    );
                }
            }
            assert!(lags[0] > 100.0 && lags[lags.len() - 1] < 400.0);
        }
    }

    let cn_only_acts = cn
        .activities
        .keys()
        .filter(|id| !en.activities.contains_key(*id))
        .count();
    let unmodelled = cn
        .activities
        .values()
        .filter(|a| !en.activities.contains_key(&a.id))
        .filter(|a| {
            matches!(
                estimate::estimate(&model, a.start_time),
                Resolution::Unmodelled
            )
        })
        .count();
    println!("cn-only activities={cn_only_acts} unmodelled={unmodelled}");
    assert_eq!(unmodelled, 0);

    let runs = estimate::StageRuns::build(&cn);
    let en_acts = estimate::en_activity_index(&en);
    let (mut anchored, mut held, mut cn_only_anchored, mut cn_only) =
        (0usize, 0usize, 0usize, 0usize);
    for s in cn.skins.char_skins.values() {
        let t = s.display_skin.get_time;
        if t <= 0 {
            continue;
        }
        let anchor = runs
            .anchor(t)
            .filter(|h| h.offset_secs == 0)
            .map(|h| h.activity);
        match en.skins.char_skins.get(&s.skin_id) {
            Some(e) if e.display_skin.get_time > 0 => {
                if let Some(a) = anchor {
                    anchored += 1;
                    if en_acts.get(a.id.as_str()).is_some_and(|(start, _, _)| {
                        (start - e.display_skin.get_time).abs() <= 3 * 86_400
                    }) {
                        held += 1;
                    }
                }
            }
            _ => {
                cn_only += 1;
                if anchor.is_some() {
                    cn_only_anchored += 1;
                }
            }
        }
    }
    println!(
        "skins anchored to a stage event: shared {anchored}, tie held on EN {held}; cn-only {cn_only_anchored}/{cn_only}"
    );
    assert!(anchored >= 300 && held * 100 / anchored >= 90);

    let now = 1_789_000_000;
    let (groups, en_batches) = skins::group_histories(&en, &skins::batch_families(&en, &cn));
    let (_, cn_batches) = skins::group_histories(&cn, &skins::batch_families(&cn, &en));
    let stats = skins::anniversary_models(&groups, now);
    assert!(
        stats.len() >= 2,
        "at least years 1 and 2 have enough groups"
    );
    for st in &stats {
        println!(
            "en anniversary {}: observed {}/{} dev p25={:.0} median={:.0} p75={:.0}",
            st.year, st.observed, st.eligible, st.dev_p25_days, st.dev_median_days, st.dev_p75_days
        );
    }
    assert!(stats[0].eligible >= 150 && stats[0].observed * 100 / stats[0].eligible >= 50);
    assert!(stats[1].eligible >= 100 && stats[1].observed * 100 / stats[1].eligible >= 40);
    assert_eq!(en_batches.len(), 7);
    assert_eq!(cn_batches.len(), 12);
    let (cn_groups, _) = skins::group_histories(&cn, &skins::batch_families(&cn, &en));
    let lookback = (model.median_days * 86_400.0) as i64 + 60 * 86_400;
    let pending = skins::pending_cn_reruns(&cn_groups, now, lookback);
    let en_by_group: std::collections::HashMap<&str, &skins::GroupHistory> = groups
        .iter()
        .map(|g| (g.skin_group_id.as_str(), g))
        .collect();
    let mut consumed = std::collections::HashSet::new();
    let (mut anchored, mut confirmed, mut estimated) = (0usize, 0usize, 0usize);
    for p in &pending {
        let anchor = runs.anchor(p.cn_window.start_time);
        let expected = anchor
            .as_ref()
            .and_then(|h| {
                en_acts
                    .get(h.activity.id.as_str())
                    .map(|(s, _, _)| *s + h.offset_secs)
            })
            .unwrap_or_else(
                || match estimate::estimate(&model, p.cn_window.start_time) {
                    Resolution::Estimated { en_start, .. } => en_start,
                    _ => 0,
                },
            );
        if anchor.is_some() {
            anchored += 1;
        }
        if skins::match_en_listing(
            en_by_group.get(p.group.skin_group_id.as_str()).copied(),
            expected,
            &mut consumed,
        )
        .is_some()
        {
            confirmed += 1;
        } else {
            estimated += 1;
        }
    }
    println!(
        "cn re-listings in the lookback={} anchored to a stage event={anchored} already on EN={confirmed} pending={estimated}",
        pending.len()
    );
    for p in pending.iter().take(40) {
        println!(
            "  {} {} cn {} .. {} anchor={:?}",
            p.group.skin_group_id,
            p.group.skin_group_name,
            chrono::DateTime::from_timestamp(p.cn_window.start_time, 0)
                .unwrap()
                .date_naive(),
            chrono::DateTime::from_timestamp(p.cn_window.end_time, 0)
                .unwrap()
                .date_naive(),
            runs.anchor(p.cn_window.start_time)
                .map(|h| h.activity.id.as_str())
        );
    }
    assert!(!pending.is_empty());

    let cn_reviews = skins::review_windows(&cn);
    let en_reviews = skins::review_windows(&en);
    let paired = skins::pair_reviews(&cn_reviews, &en_reviews);
    let confirmed = paired.iter().filter(|p| p.is_some()).count();
    let mut pool: Vec<i64> = cn
        .skins
        .char_skins
        .values()
        .filter(|s| s.display_skin.get_time > 0 && skins::review_eligible(s, &cn.skins.brand_list))
        .map(|s| s.display_skin.get_time)
        .collect();
    pool.sort_unstable();
    println!(
        "fashion reviews: cn={} en={} paired={} pool={} last cn={}",
        cn_reviews.len(),
        en_reviews.len(),
        confirmed,
        pool.len(),
        cn_reviews.last().map_or(0, |w| w.0)
    );
    // The review is cumulative: each edition stocks every eligible outfit
    // released up to its cutoff, so the stocked count never shrinks and the
    // newest editions hold the bulk of the catalogue that is old enough.
    let mut previous = 0;
    for &(start, _) in &cn_reviews {
        let cutoff = skins::review_pool_cutoff(start);
        let stocked = pool.partition_point(|&t| t <= cutoff);
        let added = stocked - previous;
        println!(
            "  cn review {}: stocks {stocked} outfits released up to {} (+{added})",
            chrono::DateTime::from_timestamp(start, 0)
                .unwrap()
                .date_naive(),
            chrono::DateTime::from_timestamp(cutoff, 0)
                .unwrap()
                .date_naive(),
        );
        assert!(stocked >= previous, "the review never drops an outfit");
        assert!(
            (1..=25).contains(&added),
            "each edition adds a quarter's worth of outfits"
        );
        previous = stocked;
    }
    assert!(
        (190..=230).contains(&previous),
        "the newest CN review stocks about 200 outfits, got {previous}"
    );
    assert!(cn_reviews.len() >= 20 && en_reviews.len() >= 18);

    assert_eq!(
        confirmed,
        en_reviews.len(),
        "every EN review pairs with a CN one"
    );
    assert!(
        cn_reviews.len() - confirmed >= 2,
        "the newest CN reviews are unmatched on EN"
    );

    for (label, gd) in [("cn", &cn), ("en", &en)] {
        let (with, without) = shop_coverage(gd, now);
        eprintln!(
            "{label} event shops: {} cached, {} of {} listed shops covered, missing: {}",
            gd.event_shops.len(),
            with.len(),
            with.len() + without.len(),
            without.join(", ")
        );
        if gd.event_shops.is_empty() {
            eprintln!("{label} event-shop sidecar not present, skipping coverage");
            continue;
        }
        assert!(
            without.is_empty(),
            "{label}: {} listed shops the server did not answer: {without:?}",
            without.len()
        );
    }
    let reviews = en
        .skin_listings
        .iter()
        .filter(|l| {
            matches!(
                l.kind,
                backend::core::gamedata::types::shop::ListingKind::Review
            )
        })
        .count();
    let listed_since_2025 = groups
        .iter()
        .filter(|g| g.listings().any(|w| w.start_time > 1_735_689_600))
        .count();
    let seen_since_2025 = groups
        .iter()
        .filter(|g| g.last_seen().is_some_and(|t| t > 1_735_689_600))
        .count();
    let stale = groups
        .iter()
        .filter(|g| g.last_seen().is_some_and(|t| now - t > 500 * 86_400))
        .count();
    println!(
        "en listings={} reviews={} groups={} listed since 2025: {} seen since 2025 (incl. reviews): {} last seen >500d ago: {}",
        en.skin_listings.len(),
        reviews,
        groups.len(),
        listed_since_2025,
        seen_since_2025,
        stale
    );
    assert_eq!(reviews, 18);

    let memory = TranslationMemory::build(&cn, &en, &aligned);
    let cn_only_acts: Vec<_> = cn
        .activities
        .values()
        .filter(|a| !en.activities.contains_key(&a.id))
        .collect();
    let acts_named = cn_only_acts
        .iter()
        .filter(|a| translate::resolve(&memory, &a.name).is_some())
        .count();
    let cn_only_skins: Vec<_> = cn
        .skins
        .char_skins
        .values()
        .filter(|s| !en.skins.char_skins.contains_key(&s.skin_id) && s.display_skin.get_time > 0)
        .collect();
    let skins_named = cn_only_skins
        .iter()
        .filter(|s| {
            translate::resolve(&memory, s.display_skin.skin_name.as_deref().unwrap_or("")).is_some()
        })
        .count();
    let groups_named = cn_only_skins
        .iter()
        .filter(|s| translate::resolve(&memory, &s.display_skin.skin_group_name).is_some())
        .count();
    let cn_only_pools: Vec<_> = cn
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|p| !aligned.contains_key(&p.gacha_pool_id))
        .collect();
    let pools_named = cn_only_pools
        .iter()
        .filter(|p| translate::resolve(&memory, &p.gacha_pool_name).is_some())
        .count();
    let acts_by_id = cn_only_acts
        .iter()
        .filter(|a| translate::resolve(&memory, &a.name).is_none())
        .filter(|a| {
            a.id.strip_suffix("sre")
                .map(|stem| format!("{stem}side"))
                .is_some_and(|side| en.activities.contains_key(&side))
                || cn.retro_acts.values().any(|r| {
                    r.linked_act_id.iter().any(|l| l == &a.id)
                        && en.retro_acts.contains_key(&r.retro_id)
                })
        })
        .count();
    let cn_only_chars: Vec<_> = cn
        .operators
        .keys()
        .filter(|k| k.starts_with("char_") && !en.operators.contains_key(*k))
        .collect();
    let chars_named = cn_only_chars
        .iter()
        .filter(|k| translate::operator_name(&cn, &en, k).is_some())
        .count();
    println!(
        "memory pairs={} | cn-only names resolved: activities {}+{} by id /{} skins {}/{} skin groups {}/{} unaligned pools {}/{} operators {}/{}",
        memory.len(),
        acts_named,
        acts_by_id,
        cn_only_acts.len(),
        skins_named,
        cn_only_skins.len(),
        groups_named,
        cn_only_skins.len(),
        pools_named,
        cn_only_pools.len(),
        chars_named,
        cn_only_chars.len()
    );
    assert!(memory.len() > 500);
    assert_eq!(chars_named, cn_only_chars.len());

    let cn_idx = AssetIndex::build(&root("cn"));
    let en_idx = AssetIndex::build(&root("en"));
    let stages_cn: backend::core::gamedata::types::stage::StageTableFile =
        load_table(&root("cn").join("gamedata/excel"), "stage_table").expect("stage_table");
    let acts_cn: backend::core::gamedata::types::activity::ActivityTableFile =
        load_table(&root("cn").join("gamedata/excel"), "activity_table").expect("activity_table");
    let op = backend::core::gamedata::types::activity::op_stages_by_activity(
        &stages_cn.stages,
        &acts_cn.zone_to_activity,
    );
    let total = |id: &str| {
        op.get(id)
            .map_or(0, |v| v.iter().map(|s| s.op).sum::<i32>())
    };
    let with_op = cn_only_acts
        .iter()
        .filter(|a| a.has_stage && total(&a.id) > 0)
        .count();
    println!(
        "op rewards by activity: {} activities, cn-only stage events with op {with_op}, act51side={} act53side={} act44side={}",
        op.len(),
        total("act51side"),
        total("act53side"),
        total("act44side")
    );
    let item_file: backend::core::gamedata::types::material::ItemTableFile =
        load_table(&root("cn").join("gamedata/excel"), "item_table").expect("item_table");
    let farm = backend::core::gamedata::types::activity::farm_stages_by_activity(
        &stages_cn.stages,
        &acts_cn.zone_to_activity,
        &item_file.items,
    );
    let pa: Vec<String> = farm
        .get("act51side")
        .map(|v| {
            v.iter()
                .map(|f| {
                    format!(
                        "{} {:?}",
                        f.code,
                        f.drops
                            .iter()
                            .map(|d| (&d.name, d.tier))
                            .collect::<Vec<_>>()
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    println!(
        "farm stages: {} activities; act51side={pa:?}; act21mini={:?}",
        farm.len(),
        farm.get("act21mini").map(Vec::len)
    );
    assert_eq!(
        farm.get("act51side")
            .map(|v| v.iter().map(|f| f.code.as_str()).collect::<Vec<_>>()),
        Some(vec!["PA-6", "PA-7", "PA-8"])
    );
    assert!(farm.get("act21mini").is_some_and(|v| v.len() <= 2));
    assert_eq!(total("act51side"), 27);
    assert_eq!(total("act53side"), 38);
    assert_eq!(total("act44side"), 40);
    let art = |idx: &AssetIndex, id: &str| idx.event_banner_path(id).is_some();
    let with_rerun = |id: &str| {
        let side = id.strip_suffix("sre").map(|s| format!("{s}side"));
        art(&cn_idx, id)
            || art(&en_idx, id)
            || side
                .as_deref()
                .is_some_and(|s| art(&cn_idx, s) || art(&en_idx, s))
    };
    let ev_img = cn_only_acts.iter().filter(|a| with_rerun(&a.id)).count();
    let story_only = cn_only_acts.iter().filter(|a| a.has_stage).count();
    println!(
        "cn-only stage events {story_only}, with art {}",
        cn_only_acts
            .iter()
            .filter(|a| a.has_stage && with_rerun(&a.id))
            .count()
    );
    let en_ev_img = en
        .activities
        .values()
        .filter(|a| art(&en_idx, &a.id))
        .count();
    let recent_pools: Vec<_> = cn
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|p| p.open_time > 1_750_000_000)
        .collect();
    let pool_img = recent_pools
        .iter()
        .filter(|p| cn_idx.gacha_banner_path(&p.gacha_pool_id).is_some())
        .count();
    let en_recent: Vec<_> = en
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|p| p.open_time > 1_750_000_000)
        .collect();
    let en_pool_img = en_recent
        .iter()
        .filter(|p| en_idx.gacha_banner_path(&p.gacha_pool_id).is_some())
        .count();
    let skin_img = cn_only_skins
        .iter()
        .filter(|s| {
            cn_idx
                .path(AssetKind::SkinPortrait, &s.portrait_id)
                .is_some()
        })
        .count();
    let kv = en
        .skins
        .brand_list
        .values()
        .flat_map(|b| b.kv_img_id_list.iter())
        .count();
    let kv_img = en
        .skins
        .brand_list
        .values()
        .flat_map(|b| b.kv_img_id_list.iter())
        .filter(|k| en_idx.brand_kv_path(&k.kv_img_id).is_some())
        .count();
    let logo_img = en
        .skins
        .brand_list
        .values()
        .filter(|b| en_idx.brand_logo_path(&b.brand_id).is_some())
        .count();
    println!(
        "images: cn-only event art {}/{} | en event art {}/{} | cn pools since 2025-06 {}/{} | en pools since 2025-06 {}/{} | cn-only skin portraits {}/{} | brand kv {}/{} | brand logos {}/{}",
        ev_img,
        cn_only_acts.len(),
        en_ev_img,
        en.activities.len(),
        pool_img,
        recent_pools.len(),
        en_pool_img,
        en_recent.len(),
        skin_img,
        cn_only_skins.len(),
        kv_img,
        kv,
        logo_img,
        en.skins.brand_list.len()
    );
}

/// The CN edition of 2026-04-23T20:00Z, the first one the EN extract has no
/// listing for (the announced one); its override key is this start.
const ANNOUNCED_CN_REVIEW: i64 = 1_776_974_400;

/// EN's shop table with its newest real Fashion Review listing cloned onto
/// new dates, as the next extract would carry the announced edition.
fn en_listings_with_review(start: i64, end: i64) -> Option<ShopTableFile> {
    let path = root("en").join("gamedata/excel/shop_client_table.json");
    let mut v: serde_json::Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    let list = v.get_mut("RecommendList")?.as_array_mut()?;
    let mut review = list
        .iter()
        .filter(|r| {
            r["TagName"]
                .as_str()
                .is_some_and(backend::core::gamedata::types::shop::is_review_name)
        })
        .max_by_key(|r| r["StartDatetime"].as_i64())?
        .clone();
    review["StartDatetime"] = start.into();
    review["EndDatetime"] = end.into();
    list.push(review);
    serde_json::from_value(v).ok()
}

#[test]
fn announced_review_override_yields_to_the_listing() {
    use backend::core::release::{ledger, override_index, resolve_reviews};
    use backend::database::queries::release::OverrideRow;

    let (Some(cn), Some(mut en)) = (load("cn"), load("en")) else {
        eprintln!("skipping: no extract");
        return;
    };
    let pairs = estimate::activity_pairs(&cn, &en);
    let yearly_types = estimate::yearly_types(&pairs);
    let (pairs, _) = estimate::split_yearly(&pairs, &yearly_types);
    let model = estimate::build_lag_model(&pairs, 10);
    let cn_windows = skins::review_windows(&cn);
    let at = cn_windows
        .iter()
        .position(|&(s, _)| s == ANNOUNCED_CN_REVIEW)
        .expect("the 2026-04-23 CN edition is in the extract");
    let status = |r: &[Resolution]| {
        r.iter()
            .map(|x| match x {
                Resolution::Confirmed { .. } => 'C',
                Resolution::Override { .. } => 'O',
                Resolution::Estimated { .. } => 'E',
                _ => '-',
            })
            .collect::<String>()
    };

    // 2026-09-24T15:00Z .. 2026-10-21T14:59:59Z, a 27-day run like EN's last ones.
    let (en_start, en_end) = (1_790_262_000, 1_792_594_799);
    let row = OverrideRow {
        kind: ledger::KIND_REVIEW.into(),
        cn_id: ANNOUNCED_CN_REVIEW.to_string(),
        en_id: None,
        en_name: None,
        en_start: Some(en_start),
        en_end: Some(en_end),
        featured_chars: None,
        source: "EN announcement".into(),
        note: String::new(),
        updated_at: chrono::Utc::now(),
    };
    let rows = vec![row];
    let idx = override_index(&rows);
    let none = override_index(&[]);

    let before = resolve_reviews(&cn_windows, &skins::review_windows(&en), &none, &model);
    eprintln!("no override:   {}", status(&before));
    assert!(
        before[..at]
            .iter()
            .all(|r| matches!(r, Resolution::Confirmed { .. }))
    );
    assert!(matches!(before[at], Resolution::Estimated { .. }));

    let announced = resolve_reviews(&cn_windows, &skins::review_windows(&en), &idx, &model);
    eprintln!("override:      {}", status(&announced));
    assert!(matches!(
        &announced[at],
        Resolution::Override { en_start: s, en_end: Some(e), source, .. }
            if *s == en_start && *e == en_end && source == "EN announcement"
    ));
    for (i, (a, b)) in before.iter().zip(&announced).enumerate() {
        if i != at {
            assert_eq!(a, b, "the override moves only its own edition");
        }
    }

    // The next extract carries the listing; nobody touches the override.
    let shop = en_listings_with_review(en_start, en_end).expect("EN shop table");
    en.skin_listings = shop.into_skin_listings();
    let landed = resolve_reviews(&cn_windows, &skins::review_windows(&en), &idx, &model);
    eprintln!("listing lands: {}", status(&landed));
    assert_eq!(
        landed[at],
        Resolution::Confirmed {
            en_id: format!("review:{en_start}"),
            en_start,
            en_end,
        }
    );
    assert_eq!(
        landed[..at],
        before[..at],
        "older editions keep their pairs"
    );
}
