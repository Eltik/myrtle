//! Where the 2026-09-23 refresh stalled, and what it was NOT.
//!
//! After `sync: inferred clears folded into stage records` the process served
//! nothing further and ignored SIGTERM until it was killed. The database says
//! exactly how far the refresh got: `users.updated_at` for the account moved to
//! 21:30:12.168817Z, the same second as that log line, so `sp_sync_user_data`
//! committed; `user_scores.calculated_at` is still 2026-09-20 07:40:51Z, so
//! `update_score` never ran; `user_game_story_read` holds 0 rows, so
//! `store_game_read` never ran either. The window between those two is
//! `find_by_uid` plus `calculate_user_grade`, and only one of them computes.
//!
//! These tests hold that shape in place. The first two REFUTE the pool
//! hypotheses by reproducing them: a refresh-shaped sequence against a pool of
//! ONE completes, because nothing in the sequence holds a connection across a
//! call that takes another. The third reproduces the shape that does stall a
//! runtime, and shows the bounded form that does not.

#![allow(clippy::unreadable_literal)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Two tasks, a pool of one connection, the refresh's own ordering: write,
/// then a transaction. Neither holds a connection while asking for another, so
/// they serialize and both finish. A nested acquire would hang here instead.
///
/// Ruled out by measurement: hypothesis (a), "the game-read store opened its
/// own connection while the refresh already held one".
#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn a_refresh_shaped_sequence_does_not_deadlock_on_a_pool_of_one() {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await
        .expect("connect to local postgres");

    let started = Instant::now();
    let a = {
        let pool = pool.clone();
        tokio::spawn(async move {
            // Stands in for `sync_user_data`: a statement on the pool, fully
            // released, then a transaction, exactly as `refresh` orders them.
            sqlx::query("SELECT 1").execute(&pool).await?;
            let mut tx = pool.begin().await?;
            sqlx::query("SELECT 1").execute(&mut *tx).await?;
            tx.commit().await
        })
    };
    let b = {
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut tx = pool.begin().await?;
            sqlx::query("SELECT 1").execute(&mut *tx).await?;
            tx.commit().await
        })
    };
    a.await.unwrap().expect("task a");
    b.await.unwrap().expect("task b");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "a pool of one serialized two refresh-shaped tasks in {:?}; it should not queue",
        started.elapsed()
    );
}

/// The shape that WOULD hang: a connection held across an acquire of another
/// from a pool that has none left. It does not hang forever either, because
/// `acquire_timeout` sheds it; the refresh path contains no such nesting.
///
/// Ruled out by measurement: hypothesis (b) as a cause of an UNBOUNDED hang.
/// A pool starvation on this pool surfaces as an error after the acquire
/// timeout, not as a process that stops serving.
#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn a_nested_acquire_sheds_on_the_timeout_rather_than_hanging() {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&url)
        .await
        .expect("connect to local postgres");

    let mut held = pool.begin().await.expect("hold the only connection");
    sqlx::query("SELECT 1")
        .execute(&mut *held)
        .await
        .expect("statement on the held connection");

    let started = Instant::now();
    let nested = pool.begin().await;
    let waited = started.elapsed();
    assert!(nested.is_err(), "the nested acquire should have been shed");
    assert!(
        waited < Duration::from_secs(3),
        "the nested acquire waited {waited:?}; the pool's own timeout should bound it"
    );
    held.rollback().await.expect("rollback");
}

/// The shape that DOES stall a runtime: synchronous CPU on an async worker.
///
/// Two workers, three tasks that each compute inline for 300 ms, and a fourth
/// that only wants to be polled. The fourth waits behind them. Bounding the
/// computing tasks to ONE at a time leaves a worker free and the fourth is
/// polled immediately. This is the mechanism the refresh hit inside
/// `calculate_user_grade`, which scores operators, base, roguelike and medals
/// inline between its pool queries.
#[test]
fn inline_cpu_starves_a_worker_and_a_permit_does_not() {
    fn burn(ms: u64) {
        let until = Instant::now() + Duration::from_millis(ms);
        let mut acc: u64 = 0;
        while Instant::now() < until {
            acc = acc.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        }
        std::hint::black_box(acc);
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    let unbounded = runtime.block_on(async {
        let polled = Arc::new(AtomicUsize::new(0));
        let started = Instant::now();
        let mut tasks = Vec::new();
        for _ in 0..3 {
            tasks.push(tokio::spawn(async { burn(300) }));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        let p = Arc::clone(&polled);
        let probe = tokio::spawn(async move {
            p.store(1, Ordering::SeqCst);
            started.elapsed()
        });
        let latency = probe.await.unwrap();
        for t in tasks {
            t.await.unwrap();
        }
        assert_eq!(polled.load(Ordering::SeqCst), 1);
        latency
    });

    let bounded = runtime.block_on(async {
        let gate = Arc::new(tokio::sync::Semaphore::new(1));
        let started = Instant::now();
        let mut tasks = Vec::new();
        for _ in 0..3 {
            let gate = Arc::clone(&gate);
            tasks.push(tokio::spawn(async move {
                let _permit = gate.acquire_owned().await.unwrap();
                tokio::task::spawn_blocking(|| burn(300)).await.unwrap();
            }));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
        let probe = tokio::spawn(async move { started.elapsed() });
        let latency = probe.await.unwrap();
        for t in tasks {
            t.await.unwrap();
        }
        latency
    });

    assert!(
        bounded < unbounded,
        "inline {unbounded:?} vs bounded {bounded:?}: bounding should let an idle task be polled sooner"
    );
}

/// What one account's grade actually costs, which is the number that decides
/// whether the stall above was a stall or a wait. Prints; asserts only that it
/// completed, because the value is the deliverable.
#[tokio::test]
#[ignore = "reads the local Postgres roster"]
async fn grade_wall_clock_for_one_account() {
    use backend::core::grade::calculate::calculate_user_grade;

    let uid = std::env::var("GRADE_UID").unwrap_or_else(|_| "09525371".into());
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let game_data = common::load_game_data();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await
        .expect("connect to local postgres");
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE uid = $1")
        .bind(&uid)
        .fetch_one(&pool)
        .await
        .expect("the account is in the local database");

    let started = Instant::now();
    let grade = calculate_user_grade(&pool, user_id, game_data)
        .await
        .expect("grade");
    let elapsed = started.elapsed();
    println!(
        "uid {uid}: calculate_user_grade {} ms, overall {} ({:.6})",
        elapsed.as_millis(),
        grade.overall,
        grade.total_score
    );
}

/// The four census numbers, computed by the shipped parser against the real
/// dump instead of by a script beside it. Reads the payload from
/// `MYRTLE_SYNC_DUMP`, never from the repository: it is the user's whole
/// account and nothing of it belongs in a fixture.
///
/// Measured 2026-09-24 on `en_09525371`: flag_hits 1,084, review_hits 1,020,
/// union 1,365, flag_misses 197.
#[test]
#[ignore = "needs MYRTLE_SYNC_DUMP pointing at a real syncData payload"]
fn the_census_numbers_on_a_real_payload() {
    use backend::app::services::story::build_index;
    use backend::app::services::story_progress::parse_game_story_read;
    use backend::core::gamedata::assets::AssetIndex;

    let Ok(path) = std::env::var("MYRTLE_SYNC_DUMP") else {
        panic!("set MYRTLE_SYNC_DUMP to a syncData payload outside the repository");
    };
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read the dump"))
            .expect("parse the dump");

    let dir = std::path::PathBuf::from(
        std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into()),
    );
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let cache = build_index(common::load_game_data(), &asset_index, &dir);

    let set = parse_game_story_read(&raw, &cache.by_txt, &cache.gates);
    println!(
        "index {} txt pairs | flags {} | flag_hits {} | flag_misses {} | review_hits {} | stage_records {} | stage_hits {} | union {} | read {} | archived {} | archive_only_unread {} | first misses {:?}",
        cache.by_txt.len(),
        set.flags,
        set.flag_hits,
        set.flag_misses,
        set.review_hits,
        set.stage_records,
        set.stage_hits,
        set.read.len(),
        set.read_count(),
        set.archived(),
        set.archive_only_unread,
        set.flag_miss_sample,
    );
    assert!(
        set.stage_hits > 0 && set.read_count() < set.read.len(),
        "the stage rule reached nothing or refused nothing: stage_hits {} read {} of {}",
        set.stage_hits,
        set.read_count(),
        set.read.len()
    );
    assert!(
        set.read.len() > 500,
        "the union is {}; the old `rc > 0` rule yielded 2 and this must be in the hundreds",
        set.read.len()
    );
    assert_eq!(set.flags, set.flag_hits + set.flag_misses);
    assert!(set.flag_hits > 0, "the flags half mapped nothing");
}

/// WHY the store wrote nothing and the request answered 5xx, as a shape.
///
/// `middleware::observe` wraps every handler in a 30 s `tokio::time::timeout`.
/// A timeout can only be OBSERVED when its task is polled, and a task that is
/// inside inline compute is not polled: `calculate_user_grade` scores
/// operators, base, roguelike and medals on the async worker for 39,861 ms in a
/// debug build. So the deadline passed unnoticed, the handler kept the thread,
/// and the moment the grade yielded the timeout fired and DROPPED the rest of
/// the future. That is the 2026-09-24 refresh: `/api/refresh` recorded at
/// 75.967 s with a 5xx though the budget is 30 s, `user_scores` written,
/// `user_game_story_read` at 0 rows, and the store never entered.
///
/// The first half reproduces it. The second half is the shipped order: the
/// store runs BEFORE the grade and the grade runs on its own task, so the tail
/// is done before any deadline can cancel anything.
#[test]
fn an_inline_block_past_the_deadline_drops_the_tail_unrun() {
    fn burn(ms: u64) {
        let until = Instant::now() + Duration::from_millis(ms);
        let mut acc: u64 = 0;
        while Instant::now() < until {
            acc = acc.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        }
        std::hint::black_box(acc);
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    // The shape that shipped the defect: grade inline, store after it.
    let (broken_ran, broken_elapsed, timed_out) = runtime.block_on(async {
        let stored = Arc::new(AtomicUsize::new(0));
        let flag = Arc::clone(&stored);
        let started = Instant::now();
        let outcome = tokio::time::timeout(Duration::from_millis(100), async move {
            burn(600); // calculate_user_grade, inline on this task
            // update_score: the first await after the grade. `Timeout` polls
            // its inner future FIRST, so the deadline is only observed at an
            // await that is not instantly ready, which a database round trip
            // never is. The real one had already SENT its INSERT, which is why
            // `user_scores` was written by a request that answered 5xx.
            tokio::time::sleep(Duration::from_millis(50)).await;
            flag.store(1, Ordering::SeqCst); // store_game_read: never reached
        })
        .await;
        (
            stored.load(Ordering::SeqCst),
            started.elapsed(),
            outcome.is_err(),
        )
    });
    assert!(timed_out, "the handler timeout must fire");
    assert_eq!(
        broken_ran, 0,
        "the store after an over-budget inline grade must be dropped unrun; that is the 0 rows"
    );
    assert!(
        broken_elapsed >= Duration::from_millis(500),
        "the request is recorded LONG past its own deadline ({broken_elapsed:?}); that is 75.967 s against a 30 s budget"
    );

    // The shipped shape: store first, grade on its own task, bounded.
    let (fixed_ran, fixed_elapsed) = runtime.block_on(async {
        let stored = Arc::new(AtomicUsize::new(0));
        let flag = Arc::clone(&stored);
        let started = Instant::now();
        let _ = tokio::time::timeout(Duration::from_millis(1_000), async move {
            flag.store(1, Ordering::SeqCst); // the store, ahead of the grade
            let grading = tokio::spawn(async { burn(600) });
            // Bounded: the handle is dropped on a timeout, which DETACHES the
            // grade rather than cancelling it.
            let _ = tokio::time::timeout(Duration::from_millis(100), grading).await;
        })
        .await;
        (stored.load(Ordering::SeqCst), started.elapsed())
    });
    assert_eq!(fixed_ran, 1, "the store must run whatever the grade costs");
    assert!(
        fixed_elapsed < Duration::from_millis(500),
        "the refresh answered in {fixed_elapsed:?}; it must not wait out the grade"
    );
}

/// The post-refresh store path on the real payload, stage by stage, inside a
/// transaction that is ROLLED BACK.
///
/// Runs what the server runs: the same `build_index` the request reaches for
/// `by_txt`, the same `parse_game_story_read`, and the same DELETE and UNNEST
/// INSERT, driven through `store_game_read_on` so the statements can be timed
/// and then undone. Nothing is written: the transaction is rolled back and the
/// row count is read back INSIDE it.
///
/// The payload comes from `MYRTLE_SYNC_DUMP`, never from the repository.
#[tokio::test]
#[ignore = "needs MYRTLE_SYNC_DUMP and the local Postgres"]
async fn the_store_path_on_a_real_payload() {
    use backend::app::services::story::build_index;
    use backend::app::services::story_progress::{parse_game_story_read, store_game_read_on};
    use backend::core::gamedata::assets::AssetIndex;

    let Ok(path) = std::env::var("MYRTLE_SYNC_DUMP") else {
        panic!("set MYRTLE_SYNC_DUMP to a syncData payload outside the repository");
    };
    let uid = std::env::var("GRADE_UID").unwrap_or_else(|_| "09525371".into());
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());

    let t = Instant::now();
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("read the dump"))
            .expect("parse the dump");
    let parse_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let game_data = common::load_game_data();
    let game_data_ms = t.elapsed().as_millis();

    let dir = std::path::PathBuf::from(
        std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into()),
    );
    let t = Instant::now();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let asset_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let cache = build_index(game_data, &asset_index, &dir);
    let index_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let set = parse_game_story_read(&raw, &cache.by_txt, &cache.gates);
    let census_ms = t.elapsed().as_millis();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect to local postgres");
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE uid = $1")
        .bind(&uid)
        .fetch_one(&pool)
        .await
        .expect("the account is in the local database");

    let t = Instant::now();
    let mut tx = pool.begin().await.expect("begin");
    let begin_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let outcome = store_game_read_on(&mut tx, user_id, &set).await;
    let store_ms = t.elapsed().as_millis();

    let rows_in_tx: i64 =
        sqlx::query_scalar("SELECT count(*) FROM user_game_story_read WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .expect("count inside the transaction");
    tx.rollback().await.expect("roll back; nothing is written");

    println!(
        "dump parse {parse_ms} ms | game data {game_data_ms} ms | asset index {asset_ms} ms | story index {index_ms} ms | census {census_ms} ms | begin {begin_ms} ms | store {store_ms} ms"
    );
    match &outcome {
        Ok(o) => println!(
            "store ok: rows {} | delete {} ms | insert {} ms | rows visible in the transaction {rows_in_tx}",
            o.rows, o.delete_ms, o.insert_ms
        ),
        Err(e) => println!("store FAILED: {e}"),
    }
    let o = outcome.expect("the store must not fail");
    assert_eq!(
        rows_in_tx as usize, o.rows,
        "every row the store reported must be visible inside the transaction"
    );
    assert!(
        o.rows > 1_000,
        "the union is {} rows; the measured account is 1,365",
        o.rows
    );
}

/// WHERE the 2026-09-24 refresh's 12 s budget actually went, measured on both
/// placements of the same grade.
///
/// The shape under test is the refresh's, verbatim: the grade on its own task,
/// a `tokio::time::timeout(12 s)` awaiting that task's `JoinHandle`, and a
/// probe running the two queries `/auth/verify` runs (the role read and
/// `has_any_permission`) every 100 ms for the whole window. `worst` is the
/// slowest of those probes, which is what the header's session fetch feels.
///
/// The INLINE arm is what shipped: `calculate_user_grade` awaited straight on
/// the spawned task, so its scoring runs on a runtime worker. The OFFLOADED
/// arm is the fix: the same call driven by `Handle::block_on` inside
/// `cpu::run`, so the queries still use this runtime's IO driver but nothing
/// occupies an async worker.
///
/// Ruled out by reading, before this was written: there is NO per-user lock to
/// find. `AppState` holds no mutex over user data (`app/state.rs` is atomics
/// and `ArcSwap` only), `AuthUser::from_request_parts`
/// (`app/extractors/auth.rs:33`) takes no lock and touches no database, the
/// rate limiter's `DashMap` (`app/middleware.rs:350`) is held per lookup, the
/// cache's single-flight registry (`app/cache/mod.rs:38`) is a
/// `tokio::sync::Mutex` dropped before its build task is spawned, and
/// `core/grade/**` holds nothing but `LazyLock` regexes. What the refresh held
/// across the grade was a runtime WORKER, and the timer that owed the budget
/// its wakeup.
#[test]
#[ignore = "reads the local Postgres roster"]
fn the_grade_budget_is_observed_on_time_only_off_the_runtime() {
    use backend::core::gamedata::types::GameData;
    use backend::core::grade::calculate::calculate_user_grade;
    use sqlx::PgPool;
    use std::sync::atomic::AtomicBool;

    /// The refresh's own `GRADE_BUDGET`.
    const BUDGET: Duration = Duration::from_secs(12);
    /// A session probe that takes longer than this is recorded at the ceiling
    /// and the run moves on; hanging the suite would tell us nothing.
    const PROBE_CEILING: Duration = Duration::from_secs(5);
    /// The longest an arm waits for its detached grade before giving up.
    const GRADE_CEILING: Duration = Duration::from_secs(120);

    let uid = std::env::var("GRADE_UID").unwrap_or_else(|_| "09525371".into());
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let game_data: &'static GameData = common::load_game_data();

    // Built exactly as `main` builds the server's: multi-threaded, default
    // worker count, and the test thread drives the outer future.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (pool, user_id) = runtime.block_on(async {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await
            .expect("connect to local postgres");
        let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE uid = $1")
            .bind(&uid)
            .fetch_one(&pool)
            .await
            .expect("the account is in the local database");
        (pool, user_id)
    });

    /// One arm: run the grade the given way, time when the budget is OBSERVED,
    /// and record the worst session-route latency while it runs. Returns
    /// `(observed, worst_session, grade_wall)`.
    fn arm(
        runtime: &tokio::runtime::Runtime,
        pool: &PgPool,
        user_id: uuid::Uuid,
        game_data: &'static GameData,
        offloaded: bool,
    ) -> (Duration, Duration, Duration) {
        runtime.block_on(async move {
            let stop = Arc::new(AtomicBool::new(false));
            let worst = Arc::new(AtomicUsize::new(0));
            let grade_ms = Arc::new(AtomicUsize::new(0));

            let probe = tokio::spawn({
                let pool = pool.clone();
                let stop = Arc::clone(&stop);
                let worst = Arc::clone(&worst);
                async move {
                    while !stop.load(Ordering::Relaxed) {
                        let at = Instant::now();
                        // Bounded on purpose: a probe that never answers must
                        // record its ceiling and let the test FAIL on the
                        // assertion, not hang the run.
                        let _ = tokio::time::timeout(PROBE_CEILING, async {
                            let _role: Option<String> =
                                sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
                                    .bind(user_id)
                                    .fetch_optional(&pool)
                                    .await
                                    .unwrap_or(None);
                            backend::database::queries::i18n::has_any_permission(&pool, user_id)
                                .await
                                .unwrap_or(false)
                        })
                        .await;
                        worst.fetch_max(at.elapsed().as_millis() as usize, Ordering::Relaxed);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            });

            let started = Instant::now();
            let grading = {
                let pool = pool.clone();
                let grade_ms = Arc::clone(&grade_ms);
                if offloaded {
                    let handle = tokio::runtime::Handle::current();
                    tokio::spawn(async move {
                        let done = backend::app::cpu::run("user_grade", move || {
                            handle.block_on(calculate_user_grade(&pool, user_id, game_data))
                        })
                        .await;
                        assert!(done.is_ok_and(|inner| inner.is_ok()), "the grade must land");
                        grade_ms.store(started.elapsed().as_millis() as usize, Ordering::Relaxed);
                    })
                } else {
                    tokio::spawn(async move {
                        calculate_user_grade(&pool, user_id, game_data)
                            .await
                            .expect("the grade must land");
                        grade_ms.store(started.elapsed().as_millis() as usize, Ordering::Relaxed);
                    })
                }
            };

            let _ = tokio::time::timeout(BUDGET, grading).await;
            let observed = started.elapsed();

            // The handle was dropped on a timeout, which DETACHES rather than
            // cancels: wait for the grade to land before the next arm starts.
            while grade_ms.load(Ordering::Relaxed) == 0 && started.elapsed() < GRADE_CEILING {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            stop.store(true, Ordering::Relaxed);
            let _ = probe.await;

            (
                observed,
                Duration::from_millis(worst.load(Ordering::Relaxed) as u64),
                Duration::from_millis(grade_ms.load(Ordering::Relaxed) as u64),
            )
        })
    }

    let (inline_observed, inline_worst, inline_grade) =
        arm(&runtime, &pool, user_id, game_data, false);
    let (off_observed, off_worst, off_grade) = arm(&runtime, &pool, user_id, game_data, true);

    println!(
        "inline:    budget observed {:.3} s | worst session {:.3} s | grade {:.3} s",
        inline_observed.as_secs_f64(),
        inline_worst.as_secs_f64(),
        inline_grade.as_secs_f64()
    );
    println!(
        "offloaded: budget observed {:.3} s | worst session {:.3} s | grade {:.3} s",
        off_observed.as_secs_f64(),
        off_worst.as_secs_f64(),
        off_grade.as_secs_f64()
    );

    assert!(
        inline_grade > BUDGET,
        "the grade is {:.3} s, inside the {} s budget; this account no longer measures anything",
        inline_grade.as_secs_f64(),
        BUDGET.as_secs()
    );
    assert!(
        off_observed.as_secs_f64() < BUDGET.as_secs_f64() + 0.5,
        "off the runtime the 12 s budget was observed at {:.3} s; it must fire at 12.0 +- 0.5",
        off_observed.as_secs_f64()
    );
    assert!(
        off_worst < Duration::from_secs(1),
        "a session read took {:.3} s while the grade ran off the runtime; it must answer inside 1 s",
        off_worst.as_secs_f64()
    );
    assert!(
        off_observed <= inline_observed,
        "the budget must be observed no later off the runtime ({:.3} s) than on it ({:.3} s)",
        off_observed.as_secs_f64(),
        inline_observed.as_secs_f64()
    );
}
