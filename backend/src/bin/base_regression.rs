//! Turn every synced base into a regression check on the base optimizer.
//!
//! For each user with a synced base, value the base AS THE PLAYER STATIONS
//! IT (their saved preset shifts averaged, else the live stationing) and the
//! plan the Optimizer tab would give the same roster on the same rooms,
//! both by the sustained yield the grade uses. A player whose own base out-
//! yields our plan is a modelling gap: the plan is supposed to be the best
//! staffing of that roster, so anything a player beats it with is a squad,
//! skill or coupling the model prices wrong. Those accounts are the bug
//! reports before anyone files them.
//!
//! Prints a table sorted by how far the player is ahead, with the rooms
//! where their crew scores above ours, and writes the full table as JSON.
//!
//! Usage:
//!   cargo run --release --bin base-regression                       # every synced base
//!   cargo run --release --bin base-regression -- --limit 200        # the first 200
//!   cargo run --release --bin base-regression -- --concurrency 8
//!   cargo run --release --bin base-regression -- --out /tmp/base-regression.json
//!   cargo run --release --bin base-regression -- --min-gain 0.02    # report >= 2% ahead

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use anyhow::{Context, Result};
use backend::{
    app::state::{default_bin_server_from_env, derive_assets_dir, derive_game_data_dir},
    core::{
        gamedata::{init_game_data, types::GameData},
        grade::base::{
            assignment::{
                align_rooms_to_current, compute_live_assignment, sustained_assignment_value,
            },
            context::BaseContext,
            is_production_room,
            pools::{optimal_with_bundles, search_economy},
            sustain_sim::synced_live_morale,
            types::{BaseAssignment, UserBuilding},
            yield_model::room_yield,
        },
    },
    database::queries::{building::get_building, roster::get_roster},
};
use dotenv::dotenv;
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::Semaphore;
use uuid::Uuid;

struct Args {
    limit: Option<i64>,
    concurrency: usize,
    out: String,
    min_gain: f64,
    uid: Option<String>,
}

fn parse_args() -> Result<Args> {
    let mut a = Args {
        limit: None,
        concurrency: 4,
        out: "base-regression.json".into(),
        min_gain: 0.0,
        uid: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--limit" => a.limit = Some(it.next().context("--limit needs a value")?.parse()?),
            "--concurrency" => {
                a.concurrency = it.next().context("--concurrency needs a value")?.parse()?;
            }
            "--out" => a.out = it.next().context("--out needs a value")?,
            "--min-gain" => {
                a.min_gain = it.next().context("--min-gain needs a value")?.parse()?;
            }
            "--uid" => a.uid = Some(it.next().context("--uid needs a value")?),
            "-h" | "--help" => {
                eprintln!(
                    "base-regression [--limit N] [--concurrency N] [--out FILE] [--min-gain F] [--uid UID]"
                );
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument {other}"),
        }
    }
    Ok(a)
}

/// One room where the player's crew scores above the plan's crew for the
/// same slot.
#[derive(Serialize, Clone)]
struct RoomLead {
    slot_id: String,
    room_type: String,
    level: i32,
    formula: Option<String>,
    player_efficiency: f64,
    plan_efficiency: f64,
    player_crew: Vec<String>,
    plan_crew: Vec<String>,
}

#[derive(Serialize, Clone)]
struct Row {
    uid: String,
    nickname: String,
    operators: usize,
    rooms: usize,
    /// Sustained LMD-equivalent per day as the player stations the base.
    player_value: f64,
    /// The same for the Optimizer tab's plan on that roster and rooms.
    plan_value: f64,
    /// `player_value / plan_value - 1`: positive means the player beats us.
    gain: f64,
    preset_shifts: usize,
    leads: Vec<RoomLead>,
    seconds: f64,
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "base_regression=info,backend=warn".into()),
        )
        .init();
    let args = parse_args()?;

    let base = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());
    let server = default_bin_server_from_env();
    let data_dir = derive_game_data_dir(&base, server);
    let assets_dir = derive_assets_dir(&base, server);
    tracing::info!("loading game data...");
    let (game_data, _assets) = init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
        .context("failed to load game data")?;
    let game_data = Arc::new(game_data);

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(std::cmp::max(
            8,
            u32::try_from(args.concurrency).unwrap_or(4) * 2,
        ))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    // Every user with a synced base, newest sync first.
    let users: Vec<(Uuid, String, String)> = if let Some(uid) = &args.uid {
        sqlx::query_as("SELECT u.id, u.uid, u.nickname FROM users u WHERE u.uid = $1")
            .bind(uid)
            .fetch_all(&pool)
            .await?
    } else {
        sqlx::query_as(
            "SELECT u.id, u.uid, u.nickname FROM users u \
             JOIN user_building b ON b.user_id = u.id \
             ORDER BY u.updated_at DESC LIMIT $1",
        )
        .bind(args.limit.unwrap_or(i64::MAX))
        .fetch_all(&pool)
        .await?
    };
    tracing::info!(count = users.len(), "synced bases to check");

    let sem = Arc::new(Semaphore::new(args.concurrency));
    let done = Arc::new(AtomicU64::new(0));
    let total = users.len() as u64;
    let start = Instant::now();
    let mut tasks = Vec::new();
    for (user_id, uid, nickname) in users {
        let permit = sem.clone().acquire_owned().await?;
        let pool = pool.clone();
        let gd = Arc::clone(&game_data);
        let done = done.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            let result = check_one(&pool, user_id, &uid, &nickname, gd).await;
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_multiple_of(25) || n == total {
                tracing::info!(
                    done = n,
                    total,
                    elapsed_s = format!("{:.0}", start.elapsed().as_secs_f64()),
                    "progress"
                );
            }
            match result {
                Ok(row) => row,
                Err(e) => {
                    tracing::warn!(uid, error = %e, "skipped");
                    None
                }
            }
        }));
    }
    let mut rows: Vec<Row> = Vec::new();
    for t in tasks {
        if let Some(row) = t.await? {
            rows.push(row);
        }
    }
    rows.sort_by(|a, b| {
        b.gain
            .partial_cmp(&a.gain)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    std::fs::write(&args.out, serde_json::to_string_pretty(&rows)?)?;
    let ahead: Vec<&Row> = rows
        .iter()
        .filter(|r| r.gain >= args.min_gain.max(1e-9))
        .collect();
    println!(
        "{} bases checked, {} where the player beats the plan (>= {:.1}%), written to {}",
        rows.len(),
        ahead.len(),
        args.min_gain * 100.0,
        args.out
    );
    println!(
        "{:<10} {:<18} {:>4} {:>8} {:>8} {:>7}  leads",
        "uid", "nickname", "ops", "player", "plan", "gain"
    );
    for r in ahead.iter().take(60) {
        let leads: Vec<String> = r
            .leads
            .iter()
            .take(3)
            .map(|l| {
                format!(
                    "{} {:.0}>{:.0} {:?}",
                    l.room_type.to_lowercase(),
                    l.player_efficiency,
                    l.plan_efficiency,
                    l.player_crew
                )
            })
            .collect();
        println!(
            "{:<10} {:<18} {:>4} {:>8.0} {:>8.0} {:>+6.1}%  {}",
            r.uid,
            r.nickname.chars().take(18).collect::<String>(),
            r.operators,
            r.player_value,
            r.plan_value,
            r.gain * 100.0,
            leads.join(" | ")
        );
    }
    // Which crews recur among the leads: the squads the model under-prices.
    let mut crews: HashMap<String, usize> = HashMap::new();
    for r in &ahead {
        for l in &r.leads {
            let mut crew = l.player_crew.clone();
            crew.sort();
            *crews
                .entry(format!("{} {}", l.room_type, crew.join(" + ")))
                .or_insert(0) += 1;
        }
    }
    let mut crews: Vec<(String, usize)> = crews.into_iter().collect();
    crews.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    if !crews.is_empty() {
        println!("\nrecurring leading crews:");
        for (crew, n) in crews.iter().take(25) {
            println!("{n:>4}  {crew}");
        }
    }
    Ok(())
}

async fn check_one(
    pool: &PgPool,
    user_id: Uuid,
    uid: &str,
    nickname: &str,
    gd: Arc<GameData>,
) -> Result<Option<Row>> {
    let Some(json) = get_building(pool, user_id).await? else {
        return Ok(None);
    };
    let roster = get_roster(pool, user_id).await?;
    let uid = uid.to_string();
    let nickname = nickname.to_string();
    let row = tokio::task::spawn_blocking(move || compare(&uid, &nickname, &roster, &gd, &json))
        .await??;
    Ok(row)
}

#[allow(clippy::too_many_lines, clippy::unnecessary_wraps)]
fn compare(
    uid: &str,
    nickname: &str,
    roster: &[backend::database::models::roster::RosterEntry],
    gd: &GameData,
    json: &serde_json::Value,
) -> Result<Option<Row>> {
    let t = Instant::now();
    let building = UserBuilding::from_json(json);
    if building.is_empty()
        || !building
            .rooms
            .iter()
            .any(|r| is_production_room(&r.room_type))
    {
        return Ok(None);
    }
    let BaseContext {
        profiles,
        registry,
        morale_drains,
    } = BaseContext::build(roster, gd, false);
    let value = |a: &BaseAssignment| {
        sustained_assignment_value(
            a,
            &profiles,
            &building,
            &gd.building,
            &registry,
            &morale_drains,
        )
    };
    let live = synced_live_morale(json);
    let stationed = |shift: Option<usize>| {
        compute_live_assignment(
            &profiles,
            &building,
            &gd.building,
            &registry,
            &morale_drains,
            shift,
            &live,
        )
    };
    // The grade's rule: preset shifts count where every production room
    // with presets has a crew at that index.
    let with_presets: Vec<_> = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type) && !r.preset_shifts.is_empty())
        .collect();
    let most = with_presets
        .iter()
        .map(|r| r.preset_shifts.len())
        .max()
        .unwrap_or(0);
    let usable: Vec<usize> = (0..most)
        .filter(|&i| {
            with_presets
                .iter()
                .all(|r| r.preset_shifts.get(i).is_some_and(|c| !c.is_empty()))
        })
        .collect();
    let player_assignments: Vec<BaseAssignment> = if usable.len() >= 2 {
        usable.iter().map(|&i| stationed(Some(i))).collect()
    } else {
        vec![stationed(None)]
    };
    #[allow(clippy::cast_precision_loss)]
    let player_value =
        player_assignments.iter().map(value).sum::<f64>() / player_assignments.len() as f64;
    if player_value <= 0.0 {
        return Ok(None);
    }

    let economy = search_economy(&profiles, &building, &gd.building, &registry);
    let mut plan = optimal_with_bundles(
        &profiles,
        &building,
        &gd.building,
        &registry,
        &economy.registry,
        &morale_drains,
        &economy.pins,
    )
    .optimal;
    align_rooms_to_current(&building, &mut plan);
    let plan_value = value(&plan);
    if plan_value <= 0.0 {
        return Ok(None);
    }

    let name = |id: &str| {
        gd.operators
            .get(id)
            .map_or_else(|| id.to_string(), |o| o.name.clone())
    };
    // Rooms where the player's best shift crew out-scores the plan's crew.
    let mut leads = Vec::new();
    for pr in plan
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let best_player = player_assignments
            .iter()
            .filter_map(|a| a.rooms.iter().find(|r| r.slot_id == pr.slot_id))
            .max_by(|a, b| {
                a.total_efficiency
                    .partial_cmp(&b.total_efficiency)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        let Some(p) = best_player else { continue };
        // A lead is measured in the room's LMD or EXP per day, not raw
        // efficiency: a Proviso post at 82 out-sells a plain post at 112
        // wherever the factories make the gold she draws (56505800).
        let per_day = |r: &backend::core::grade::base::types::RoomAssignment| {
            let y = room_yield(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_value,
                r.operators.len(),
                r.order_limit,
            );
            y.lmd_equivalent()
        };
        if per_day(p) > per_day(pr) * 1.02 && !p.operators.is_empty() {
            leads.push(RoomLead {
                slot_id: pr.slot_id.clone(),
                room_type: pr.room_type.clone(),
                level: pr.level,
                formula: p.formula_type.clone(),
                player_efficiency: p.total_efficiency,
                plan_efficiency: pr.total_efficiency,
                player_crew: p.operators.iter().map(|o| name(o)).collect(),
                plan_crew: pr.operators.iter().map(|o| name(o)).collect(),
            });
        }
    }
    leads.sort_by(|a, b| {
        (b.player_efficiency - b.plan_efficiency)
            .partial_cmp(&(a.player_efficiency - a.plan_efficiency))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(Some(Row {
        uid: uid.to_string(),
        nickname: nickname.to_string(),
        operators: profiles.len(),
        rooms: building.rooms.len(),
        player_value,
        plan_value,
        gain: player_value / plan_value - 1.0,
        preset_shifts: usable.len(),
        leads,
        seconds: t.elapsed().as_secs_f64(),
    }))
}
