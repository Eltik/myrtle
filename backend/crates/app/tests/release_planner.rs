#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use myrtle_app::config::Config;
use myrtle_app::policy::Subject;
use myrtle_app::services::catalog::release::{
    self as rel, cache_key, jobs, one_guess_per_group, override_row, switches, wire_override,
};
use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::release;
use myrtle_engine_release::skins::RerunBasis;
use myrtle_engine_release::{PutOverride, PutReleasePlan, RerunForecast, Resolution, Switches};
use myrtle_gamedata::assets::AssetIndex;
use myrtle_gamedata::compile::CompileOptions;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::character::Skin;
use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;
use myrtle_gamedata_raw::tables::item::SkinWindow;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_types::{Role, Server};

fn config_with(window: usize, half_life: f64, flags: [bool; 3]) -> Config {
    let mut config = common::config();
    config.release.lag_window = window;
    config.release.lag_half_life = half_life;
    config.switches.release_keep_unsold_skins.enabled = flags[0];
    config.switches.release_reviews_all_groups.enabled = flags[1];
    config.switches.release_trust_skipped_tags.enabled = flags[2];
    config
}

#[test]
fn switches_follow_the_config_exactly_as_the_variables_did() {
    let flag = |on: bool| if on { "1" } else { "0" }.to_owned();
    for window in [0, 4, 10] {
        for half_life in [3.0, 1.5, 0.0, -0.0, -2.0, f64::NAN, f64::INFINITY] {
            for bits in 0..8u8 {
                let flags = [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0];
                let got = switches(&config_with(window, half_life, flags));
                let want = Switches::from_vars(|name| match name {
                    "RELEASE_LAG_WINDOW" => Some(window.to_string()),
                    "RELEASE_LAG_HALF_LIFE" => Some(half_life.to_string()),
                    "RELEASE_KEEP_UNSOLD_SKINS" => Some(flag(flags[0])),
                    "RELEASE_REVIEWS_ALL_GROUPS" => Some(flag(flags[1])),
                    "RELEASE_TRUST_SKIPPED_TAGS" => Some(flag(flags[2])),
                    _ => None,
                });
                assert_eq!(got, want, "window {window} half-life {half_life} {flags:?}");
                assert_eq!(got.lag_half_life.to_bits(), want.lag_half_life.to_bits());
            }
        }
    }
}

#[test]
fn shipped_config_is_the_shipped_switches() {
    assert_eq!(switches(&common::config()), Switches::SHIPPED);
}

#[test]
fn cache_key_keeps_the_static_text_and_folds_the_model_in() {
    let shipped = Switches::SHIPPED;
    let hash = u64::from(10_u32) ^ 3.0f64.to_bits().rotate_left(1);
    assert_eq!(
        cache_key("banners", &shipped),
        format!("static:cn:release:banners:{hash}")
    );
    let off = Switches {
        lag_half_life: 0.0,
        ..shipped
    };
    assert_eq!(cache_key("lag", &off), "static:cn:release:lag:10");
}

fn put_body(kind: &str, cn_id: &str) -> PutOverride {
    PutOverride {
        kind: kind.to_owned(),
        cn_id: cn_id.to_owned(),
        en_id: Some("act1".to_owned()),
        en_name: Some("  ".to_owned()),
        featured_chars: Some(Vec::new()),
        en_start: Some(1_800_000_000),
        en_end: None,
        source: None,
        note: None,
    }
}

#[tokio::test]
async fn override_writes_are_super_admin_only() {
    let (ctx, db) = common::context().await;
    let id = db.user(1, Server::EN).await;
    for role in [Role::User, Role::Translator, Role::TierListAdmin] {
        let s = common::user(&ctx, id, role).await;
        assert!(matches!(
            rel::put_override(&ctx, &s, put_body("activity", "act1")).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            rel::list_overrides(&ctx, &s).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            rel::delete_override(&ctx, &s, "activity", "act1").await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        rel::list_overrides(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));

    let admin = common::user(&ctx, id, Role::SuperAdmin).await;
    let stored = rel::put_override(&ctx, &admin, put_body("activity", " act1 "))
        .await
        .unwrap();
    assert_eq!(stored.cn_id, "act1");
    assert_eq!(stored.en_name, None);
    assert_eq!(stored.featured_chars, None);
    assert_eq!(stored.source, "manual");
    assert_eq!(stored.note, "");
    let listed = rel::list_overrides(&ctx, &admin).await.unwrap();
    assert_eq!(listed, vec![stored]);
    db.close().await;
}

#[tokio::test]
async fn override_validation() {
    let (ctx, db) = common::context().await;
    let admin = common::user(&ctx, db.user(1, Server::EN).await, Role::SuperAdmin).await;
    let bad_kind = rel::put_override(&ctx, &admin, put_body("char", "x")).await;
    assert!(
        matches!(bad_kind, Err(AppError::BadRequest(m)) if m == "kind must be one of activity, pool, skin, retro, review")
    );
    let blank = rel::put_override(&ctx, &admin, put_body("pool", "  ")).await;
    assert!(matches!(blank, Err(AppError::BadRequest(m)) if m == "cn_id is required"));
    assert!(matches!(
        rel::delete_override(&ctx, &admin, "pool", "missing").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn override_writes_retire_the_release_bodies() {
    let (ctx, db) = common::context().await;
    let admin = common::user(&ctx, db.user(1, Server::EN).await, Role::SuperAdmin).await;
    assert_eq!(ctx.cache.get("release:revision").await, None);
    rel::put_override(&ctx, &admin, put_body("skin", "char_1@x#1"))
        .await
        .unwrap();
    let put = ctx.cache.get("release:revision").await.unwrap();
    rel::delete_override(&ctx, &admin, "skin", "char_1@x#1")
        .await
        .unwrap();
    assert_ne!(ctx.cache.get("release:revision").await.unwrap(), put);
    assert!(rel::list_overrides(&ctx, &admin).await.unwrap().is_empty());
    db.close().await;
}

#[tokio::test]
async fn override_rows_convert_field_for_field() {
    let (ctx, db) = common::context().await;
    let mut conn = ctx.db.conn().await.unwrap();
    let chars = vec!["char_1".to_owned()];
    let row = release::put_override(
        &mut conn,
        &release::OverrideInput {
            kind: "pool",
            cn_id: "LIMITED_1",
            en_id: Some("LIMITED_1_EN"),
            en_name: Some("Name"),
            en_start: Some(5),
            en_end: Some(9),
            featured_chars: Some(&chars),
            source: "wiki",
            note: "n",
            updated_by: None,
        },
    )
    .await
    .unwrap();
    drop(conn);
    let engine = override_row(&row);
    assert_eq!(
        engine,
        myrtle_engine_release::OverrideRow {
            kind: "pool".into(),
            cn_id: "LIMITED_1".into(),
            en_id: Some("LIMITED_1_EN".into()),
            en_name: Some("Name".into()),
            en_start: Some(5),
            en_end: Some(9),
            featured_chars: Some(chars.clone()),
            source: "wiki".into(),
            note: "n".into(),
        }
    );
    let updated_at = row.updated_at.timestamp();
    let wire = wire_override(row);
    assert_eq!(wire.updated_at, updated_at);
    assert_eq!(wire.featured_chars, Some(chars));
    assert_eq!(wire.en_end, Some(9));
    db.close().await;
}

#[tokio::test]
async fn a_plan_round_trips_for_its_owner_only() {
    let (ctx, db) = common::context().await;
    let alice = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let bob = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    assert_eq!(rel::get_plan(&ctx, &alice).await.unwrap(), None);
    let body = PutReleasePlan {
        initial: -5,
        initial_manual: true,
        picks: vec!["LIMITED_1".into()],
        stages: HashMap::from([("act1".to_owned(), HashMap::from([("s1".to_owned(), true)]))]),
    };
    let saved = rel::put_plan(&ctx, &alice, body.clone()).await.unwrap();
    assert_eq!(saved.initial, 0);
    assert_eq!(saved.picks, body.picks);
    assert_eq!(saved.stages, body.stages);
    assert_eq!(rel::get_plan(&ctx, &alice).await.unwrap(), Some(saved));
    assert_eq!(rel::get_plan(&ctx, &bob).await.unwrap(), None);
    assert!(matches!(
        rel::get_plan(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        rel::put_plan(&ctx, &Subject::Anonymous, body).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

fn small_model() -> GameModel {
    let mut gd = GameModel::default();
    gd.activities.insert(
        "act1side".into(),
        ActivityBasicInfo {
            id: "act1side".into(),
            start_time: 100,
            end_time: 0,
            ..Default::default()
        },
    );
    gd.gacha.gacha_pool_client.push(GachaPoolClient {
        gacha_pool_id: "DOUBLE_1".into(),
        open_time: 50,
        end_time: 60,
        ..Default::default()
    });
    let mut skin = Skin {
        skin_id: "char_a@x#1".into(),
        ..Default::default()
    };
    skin.display_skin.get_time = 0;
    gd.skins.char_skins.insert(skin.skin_id.clone(), skin);
    for _ in 0..2 {
        gd.skin_windows.push(SkinWindow {
            skin_id: "char_a@x#1".into(),
            start_time: 70,
            end_time: 80,
        });
    }
    gd.operators.insert("char_a".into(), Operator::default());
    gd
}

#[test]
fn sightings_cover_every_kind_and_zero_dates_are_null() {
    let rows = jobs::sightings(&small_model());
    let kinds: Vec<&str> = rows.iter().map(|r| r.0).collect();
    assert_eq!(kinds, ["activity", "char", "pool", "skin", "skin_window"]);
    assert_eq!(rows[0].3, None);
    assert_eq!(rows[3].2, None);
    assert_eq!(rows[4].1, "char_a@x#1@70");
}

#[tokio::test]
async fn the_ledger_records_sightings_and_links_debuts() {
    let (mut ctx, db) = common::context().await;
    let mut snapshot = Snapshot::from_model(small_model(), SandboxUniverse::default());
    snapshot.res_version = Some("v1".into());
    snapshot.pipeline_seq = 1;
    ctx.snapshots.install(Server::EN, snapshot);
    let first = jobs::record(&ctx, Server::EN).await.unwrap();
    assert_eq!(first.as_deref(), Some("en: 5 inserted, 0 refreshed as v1"));
    let again = jobs::record(&ctx, Server::EN).await.unwrap();
    assert_eq!(
        again.as_deref(),
        Some("en: 0 inserted, 5 refreshed"),
        "a repeat stamps nothing"
    );
    let mut conn = ctx.db.conn().await.unwrap();
    assert_eq!(
        release::debut_pairs(&mut conn, Server::EN).await.unwrap(),
        vec![("DOUBLE_1".to_owned(), "char_a".to_owned())]
    );
    assert!(
        release::debut_pairs(&mut conn, Server::CN)
            .await
            .unwrap()
            .is_empty()
    );
    drop(conn);

    let mut off = common::config();
    off.switches.release_ledger.enabled = false;
    ctx.config = Arc::new(off);
    assert_eq!(
        jobs::record_ledger(&ctx, Server::EN, 0).await.unwrap(),
        "en: nothing written, the ledger is off or ahead of this data"
    );
    db.close().await;
}

#[test]
fn mirrors_parse_the_config_and_default_when_empty() {
    assert_eq!(jobs::mirrors(&[]).len(), jobs::DEFAULT_MIRRORS.len());
    let parsed = jobs::mirrors(&[
        " gacha:cn = https://m/{id}.png ".to_owned(),
        "event:en=https://m/no-placeholder.png".to_owned(),
        "=https://m/{id}.png".to_owned(),
        "garbage".to_owned(),
    ]);
    assert_eq!(
        parsed,
        vec![("gacha:cn".to_owned(), "https://m/{id}.png".to_owned())]
    );
}

#[test]
fn archive_copies_once_and_never_blanks_from_a_stub() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    let gacha = "textures/spritepack/ui_home_act_banner_gacha_h2_0/picLimited_76_0_1.png";
    let event = "textures/arts/ui/stage/[uc]homeentry/act54side.png";
    for rel in [gacha, event] {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"png").unwrap();
    }
    let mut gd = GameModel {
        assets: AssetIndex::from_paths(&[gacha.to_owned(), event.to_owned()]),
        ..Default::default()
    };
    gd.gacha.gacha_pool_client.push(GachaPoolClient {
        gacha_pool_id: "LIMITED_76_0_1".into(),
        ..Default::default()
    });
    gd.activities.insert(
        "act54side".into(),
        ActivityBasicInfo {
            id: "act54side".into(),
            ..Default::default()
        },
    );
    let counts = |s: jobs::ArchiveStats| (s.copied, s.present, s.failed);
    assert_eq!(counts(jobs::archive(root, &gd)), (2, 0, 0));
    assert_eq!(counts(jobs::archive(root, &gd)), (0, 2, 0));
    std::fs::write(root.join(event), b"png2").unwrap();
    assert_eq!(counts(jobs::archive(root, &gd)), (1, 1, 0));
    std::fs::write(root.join(event), b"").unwrap();
    assert_eq!(counts(jobs::archive(root, &gd)), (0, 1, 1));
    assert_eq!(
        std::fs::read(root.join("derived/release-art/event/act54side.png")).unwrap(),
        b"png2"
    );

    let archived = GameModel {
        assets: AssetIndex::from_paths(&[
            "derived/release-art/gacha/LIMITED_76_0_1.png".to_owned(),
            "derived/release-art/event/act54side.png".to_owned(),
        ]),
        ..gd
    };
    assert_eq!(counts(jobs::archive(root, &archived)), (0, 2, 0));
}

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

const fn cadence() -> RerunBasis {
    RerunBasis::Cadence {
        en_last: 0,
        n: 1,
        median_days: 343.0,
        p25_days: 182.0,
        p75_days: 362.0,
        own_gap_days: None,
    }
}

const fn listing() -> RerunBasis {
    RerunBasis::CnListing {
        cn_start: 0,
        cn_end: 0,
        anchor: None,
    }
}

const fn estimated(en_start: i64) -> Resolution {
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

fn cn_snapshot() -> Option<Arc<Snapshot>> {
    static SNAPSHOT: OnceLock<Option<Arc<Snapshot>>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            let root = std::env::var_os("MYRTLE_TEST_ASSETS").map_or_else(
                || {
                    PathBuf::from(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../../assets/output"
                    ))
                },
                PathBuf::from,
            );
            let root = root.join("cn");
            if !root.join("gamedata/excel/character_table.json").is_file() {
                return None;
            }
            Snapshot::load(&root, &CompileOptions::new(Server::CN))
                .ok()
                .map(Arc::new)
        })
        .clone()
}

async fn context_with_cn() -> Option<(AppContext, myrtle_testkit::db::TestDb)> {
    let (en, cn) = (common::snapshot()?, cn_snapshot()?);
    let (mut ctx, db) = common::context().await;
    let mut config = common::config();
    config.server.servers = vec![Server::EN, Server::CN];
    let registry = SnapshotRegistry::new(&config);
    registry.install_shared(Server::EN, en);
    registry.install_shared(Server::CN, cn);
    ctx.snapshots = Arc::new(registry);
    Some((ctx, db))
}

#[tokio::test]
async fn without_cn_data_the_planner_is_not_found() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        rel::get_events(&ctx).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn the_release_index_builds_from_both_extracts() {
    let Some((ctx, db)) = context_with_cn().await else {
        return;
    };
    let events = rel::get_events(&ctx).await.unwrap();
    assert!(!events.events.is_empty());
    assert!(events.model.n > 0);
    assert!(
        events
            .events
            .iter()
            .any(|e| matches!(e.resolution, Resolution::Estimated { .. }))
    );
    let banners = rel::get_banners(&ctx).await.unwrap();
    assert!(!banners.banners.is_empty());
    let skins = rel::get_skins(&ctx).await.unwrap();
    assert!(!skins.new_skins.is_empty());
    let lag = rel::get_lag(&ctx).await.unwrap();
    assert!(lag.backtest.n > 0);

    let (cn, en) = (
        ctx.snapshots.get(Server::CN).unwrap(),
        ctx.snapshots.require(Server::EN).unwrap(),
    );
    let revision = ctx.cache.get("release:revision").await.unwrap();
    let key = cn.key(&en.key(&cache_key(
        &format!("events:{revision}"),
        &Switches::SHIPPED,
    )));
    assert!(ctx.cache.get(&key).await.is_some());
    let cached = rel::get_events(&ctx).await.unwrap();
    assert_eq!(cached, events);

    let admin = common::user(&ctx, db.user(1, Server::EN).await, Role::SuperAdmin).await;
    let target = events
        .events
        .iter()
        .find(|e| matches!(e.resolution, Resolution::Estimated { .. }))
        .unwrap()
        .cn_id
        .clone();
    rel::put_override(&ctx, &admin, put_body("activity", &target))
        .await
        .unwrap();
    assert_ne!(
        ctx.cache.get("release:revision").await.unwrap(),
        revision,
        "the override retires every body"
    );
    let rebuilt = rel::get_events(&ctx).await.unwrap();
    let overridden = rebuilt.events.iter().find(|e| e.cn_id == target).unwrap();
    assert!(matches!(
        overridden.resolution,
        Resolution::Override {
            en_start: 1_800_000_000,
            ..
        }
    ));
    db.close().await;
}

#[tokio::test]
async fn a_release_build_raced_by_an_invalidation_is_not_stored() {
    let (mut ctx, db) = common::context().await;
    let mut config = common::config();
    config.server.servers = vec![Server::EN, Server::CN];
    config.cpu.task_permits = 1;
    config.cpu.task_wait_ms = 30_000;
    let registry = SnapshotRegistry::new(&config);
    for server in [Server::EN, Server::CN] {
        let model = GameModel::default();
        registry.install(
            server,
            Snapshot::from_model(model, SandboxUniverse::default()),
        );
    }
    ctx.snapshots = Arc::new(registry);
    ctx.cpu = myrtle_app::limits::CpuLimiter::new(&config.cpu);
    ctx.config = Arc::new(config);

    let held = ctx.cpu.admit("test").await.unwrap();
    let building = {
        let ctx = ctx.clone();
        tokio::spawn(async move { rel::get_lag(&ctx).await })
    };
    while ctx.cpu.waiting() == 0 {
        tokio::task::yield_now().await;
    }
    ctx.cache.invalidate_prefix("static:cn:").await;
    drop(held);
    building.await.unwrap().unwrap();
    let (cn, en) = (
        ctx.snapshots.get(Server::CN).unwrap(),
        ctx.snapshots.require(Server::EN).unwrap(),
    );
    let revision = ctx.cache.get("release:revision").await.unwrap();
    let key = cn.key(&en.key(&cache_key(
        &format!("lag:{revision}"),
        &switches(&ctx.config),
    )));
    assert_eq!(
        ctx.cache.get(&key).await,
        None,
        "built before the invalidation"
    );
    rel::get_lag(&ctx).await.unwrap();
    assert!(ctx.cache.get(&key).await.is_some());

    rel::revise(&ctx).await;
    rel::get_lag(&ctx).await.unwrap();
    let revised = ctx.cache.get("release:revision").await.unwrap();
    let moved = cn.key(&en.key(&cache_key(
        &format!("lag:{revised}"),
        &switches(&ctx.config),
    )));
    assert!(
        ctx.cache.get(&moved).await.is_some(),
        "readers build on the new revision"
    );
    assert!(
        ctx.cache.get(&key).await.is_some(),
        "the old body only expires"
    );

    ctx.cache.invalidate("release:revision").await;
    rel::get_lag(&ctx).await.unwrap();
    let fresh = ctx.cache.get("release:revision").await.unwrap();
    assert_ne!(fresh, revised, "an evicted revision is never read as empty");
    ctx.cache.invalidate("release:revision").await;
    rel::get_lag(&ctx).await.unwrap();
    assert_eq!(
        ctx.cache.get("release:revision").await.unwrap(),
        fresh,
        "reads with the key absent share one revision"
    );
    db.close().await;
}

#[tokio::test]
async fn a_revision_changes_on_every_revise() {
    let (ctx, db) = common::context().await;
    assert_eq!(ctx.cache.get("release:revision").await, None);
    let first = rel::revise(&ctx).await;
    assert_eq!(ctx.cache.get("release:revision").await, Some(first.clone()));
    assert_ne!(rel::revise(&ctx).await, first);
    db.close().await;
}
