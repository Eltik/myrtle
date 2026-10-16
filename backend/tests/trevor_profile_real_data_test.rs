//! Trevor's profile horizon over every local account, READ ONLY: for each
//! `user_stage_progress` row, the game's read verdict as `reverdict` would
//! re-derive it (`stored_payload` + `parse_game_story_read`, the pure part,
//! nothing stored), mapped to gatePos through the panel builder's
//! `gate_order.json`, then bucketed to its era.
//!
//! What it prints tells how well the era-bucketed answer cache can hit: how
//! many accounts get a profilePos above 0, how they spread over the eras,
//! the median and p90 gatePos, and the verdict's CPU time per account.
//!
//! The connection is opened with `default_transaction_read_only = on`, so a
//! write would fail rather than land. Needs the EN tree and a database:
//!
//! ```sh
//! DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/postgres \
//! ASSETS_DIR=/path/to/assets/output/en GAME_DATA_DIR=/path/to/assets/output/en/gamedata/excel \
//! cargo test --test trevor_profile_real_data_test -- --ignored --nocapture
//! ```
//!
//! `TREVOR_GATE_ORDER` overrides `../trevor/artifacts/chrono/gate_order.json`.

mod common;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use backend::app::services::story::build_index;
use backend::app::services::story_progress::{StoredRead, parse_game_story_read, stored_payload};
use backend::core::gamedata::assets::AssetIndex;
use futures_util::TryStreamExt;
use serde_json::Value;
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

fn under_manifest(p: PathBuf) -> PathBuf {
    if p.is_absolute() {
        p
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
    }
}

fn assets_dir() -> PathBuf {
    under_manifest(
        std::env::var_os("ASSETS_DIR").map_or_else(|| "../assets/output/en".into(), PathBuf::from),
    )
}

fn gate_order_path() -> PathBuf {
    under_manifest(std::env::var_os("TREVOR_GATE_ORDER").map_or_else(
        || "../trevor/artifacts/chrono/gate_order.json".into(),
        PathBuf::from,
    ))
}

/// `user_game_story_read` as `reverdict` reads it, with its account.
type StoredRow = (
    Uuid,
    String,
    i32,
    Option<chrono::DateTime<chrono::Utc>>,
    bool,
    bool,
);

struct Era {
    id: String,
    name: String,
    position: i64,
}

/// Story id -> gatePos (1-based `storyOrder` index) and the eras.
fn gate_order(path: &Path) -> (HashMap<String, i64>, Vec<Era>) {
    let doc: Value = serde_json::from_slice(&std::fs::read(path).expect("gate_order.json"))
        .expect("gate_order.json parses");
    let order: HashMap<String, i64> = doc["storyOrder"]
        .as_array()
        .expect("storyOrder")
        .iter()
        .zip(1..)
        .filter_map(|(id, pos)| id.as_str().map(|id| (id.to_owned(), pos)))
        .collect();
    let eras = doc["eras"]
        .as_array()
        .expect("eras")
        .iter()
        .map(|e| Era {
            id: e["id"].as_str().unwrap_or_default().to_owned(),
            name: e["name"].as_str().unwrap_or_default().to_owned(),
            position: e["position"].as_i64().unwrap_or(0),
        })
        .collect();
    (order, eras)
}

/// The contract's bucket: the era with the greatest position at or below `pos`.
fn era_at(eras: &[Era], pos: i64) -> Option<usize> {
    eras.iter()
        .enumerate()
        .filter(|(_, e)| e.position > 0 && e.position <= pos)
        .max_by_key(|(_, e)| e.position)
        .map(|(i, _)| i)
}

fn quantile<T: Copy + Ord>(sorted: &[T], q: f64) -> Option<T> {
    if sorted.is_empty() {
        return None;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    Some(sorted[i])
}

fn us(d: Duration) -> String {
    format!("{:.2} ms", d.as_secs_f64() * 1000.0)
}

#[test]
#[ignore = "reads the local Postgres (read only) and the EN story tree"]
fn trevor_profile_horizons_over_every_local_account() {
    let dir = assets_dir();
    if !dir.join("gamedata/excel/story_review_table.json").is_file() {
        eprintln!(
            "trevor_profile_real_data_test: no EN tree at {}, skipping",
            dir.display()
        );
        return;
    }
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("trevor_profile_real_data_test: DATABASE_URL is unset, skipping");
        return;
    };
    let (order, eras) = gate_order(&gate_order_path());
    println!("gate order: {} stories, {} eras", order.len(), eras.len());

    let gd = common::load_game_data();
    let t = Instant::now();
    let index = build_index(gd, &Arc::new(AssetIndex::build(&dir)), &dir, &dir);
    println!(
        "story index: {} lookup, {} by_txt, {} gated; built in {} ms",
        index.lookup.len(),
        index.by_txt.len(),
        index.gates.values().filter(|g| !g.is_empty()).count(),
        t.elapsed().as_millis()
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async move {
        let mut db = PgConnection::connect(&url).await.expect("database");
        sqlx::raw_sql("SET default_transaction_read_only = on")
            .execute(&mut db)
            .await
            .expect("read only");
        let ro: String = sqlx::query_scalar("SHOW default_transaction_read_only")
            .fetch_one(&mut db)
            .await
            .expect("show");
        assert_eq!(ro, "on", "refusing to run on a writable session");

        let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
            .fetch_one(&mut db)
            .await
            .expect("users");
        let stored: Vec<StoredRow> =
            sqlx::query_as(
                "SELECT user_id, story_id, reread_count, unlocked_at, played, archived FROM user_game_story_read",
            )
            .fetch_all(&mut db)
            .await
            .expect("stored reads");
        let mut rows_of: HashMap<Uuid, Vec<StoredRead>> = HashMap::new();
        for (user, story_id, reread_count, unlocked_at, played, archived) in stored {
            rows_of.entry(user).or_default().push(StoredRead {
                story_id,
                reread_count,
                unlocked_at,
                played,
                archived,
            });
        }

        let stored_users = rows_of.len();
        let mut positions: Vec<i64> = Vec::new();
        let mut verdict_cpu: Vec<Duration> = Vec::new();
        let mut per_era: BTreeMap<Option<usize>, usize> = BTreeMap::new();
        let mut reads_total = 0usize;
        let mut read_off_order = 0usize;
        let mut slowest: (Duration, Uuid, usize) = (Duration::ZERO, Uuid::nil(), 0);
        let mut accounts = 0usize;

        let fetch_started = Instant::now();
        let mut stream = sqlx::query_as::<_, (Uuid, Value, i32)>(
            "SELECT user_id, stages, pg_column_size(stages) FROM user_stage_progress",
        )
        .fetch(&mut db);
        while let Some((user, stages, bytes)) = stream.try_next().await.expect("stage row") {
            accounts += 1;
            let rows = rows_of.remove(&user).unwrap_or_default();
            let t = Instant::now();
            let raw = stored_payload(&rows, Some(stages), &index.lookup);
            let set = parse_game_story_read(&raw, &index.by_txt, &index.gates);
            let cpu = t.elapsed();
            verdict_cpu.push(cpu);
            if cpu > slowest.0 {
                slowest = (cpu, user, usize::try_from(bytes).unwrap_or(0));
            }
            let mut pos = 0i64;
            for r in set.read.iter().filter(|r| r.read) {
                reads_total += 1;
                match order.get(&r.story_id) {
                    Some(&p) => pos = pos.max(p),
                    None => read_off_order += 1,
                }
            }
            positions.push(pos);
            *per_era.entry(era_at(&eras, pos)).or_default() += 1;
        }
        drop(stream);
        let wall = fetch_started.elapsed();

        let released = positions.iter().filter(|&&p| p > 0).count();
        positions.sort_unstable();
        let mut pos_released: Vec<i64> = positions.iter().copied().filter(|&p| p > 0).collect();
        pos_released.sort_unstable();
        verdict_cpu.sort_unstable();

        println!(
            "accounts: {users} users, {accounts} with stage progress, {stored_users} with stored game read rows"
        );
        println!(
            "profilePos > 0: {released} of {accounts} ({:.1}%); {reads_total} reads in all, {read_off_order} on stories outside storyOrder",
            released as f64 * 100.0 / accounts.max(1) as f64
        );
        println!(
            "gatePos over all {accounts}: median {:?}, p90 {:?}, max {:?}; over the {released} released: median {:?}, p90 {:?}",
            quantile(&positions, 0.5),
            quantile(&positions, 0.9),
            positions.last(),
            quantile(&pos_released, 0.5),
            quantile(&pos_released, 0.9)
        );
        println!("era distribution ({} eras; position = the era's gatePos):", eras.len());
        println!(
            "  {:>5}  {:<14} {:>6}  (before the first era: eraPos 0)",
            per_era.get(&None).copied().unwrap_or(0),
            "-",
            0
        );
        for (i, e) in eras.iter().enumerate() {
            println!(
                "  {:>5}  {:<14} {:>6}  {}",
                per_era.get(&Some(i)).copied().unwrap_or(0),
                e.id,
                e.position,
                e.name
            );
        }
        let distinct = per_era.len();
        println!("distinct eraPos buckets in use: {distinct}");
        println!(
            "verdict CPU per account (stored_payload + parse_game_story_read, this build): median {}, p90 {}, max {} (account {}, stages {} bytes); fetch + verdict wall {} ms for {accounts}",
            us(quantile(&verdict_cpu, 0.5).unwrap_or_default()),
            us(quantile(&verdict_cpu, 0.9).unwrap_or_default()),
            us(verdict_cpu.last().copied().unwrap_or_default()),
            slowest.1,
            slowest.2,
            wall.as_millis()
        );

        // The reverdict's two reads for the heaviest account, timed as the ask
        // path pays them (the store it skips here is the third step).
        let t = Instant::now();
        let rows: Vec<StoredRead> = sqlx::query_as(
            "SELECT story_id, reread_count, unlocked_at, played, archived FROM user_game_story_read WHERE user_id = $1",
        )
        .bind(slowest.1)
        .fetch_all(&mut db)
        .await
        .expect("rows");
        let stages: Option<(Value,)> =
            sqlx::query_as("SELECT stages FROM user_stage_progress WHERE user_id = $1")
                .bind(slowest.1)
                .fetch_optional(&mut db)
                .await
                .expect("stages");
        let read_ms = t.elapsed();
        let t = Instant::now();
        let raw = stored_payload(&rows, stages.map(|s| s.0), &index.lookup);
        let set = parse_game_story_read(&raw, &index.by_txt, &index.gates);
        println!(
            "heaviest account: reads {} + verdict {} ({} rows would be stored, {} read)",
            us(read_ms),
            us(t.elapsed()),
            set.read.len(),
            set.read_count()
        );
        assert!(accounts > 0, "no stage progress rows to measure");
    });
}
