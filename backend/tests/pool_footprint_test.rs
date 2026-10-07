//! How many Postgres connections the regrade's widest unit holds at once,
//! measured against a real database. Read-only (`calculate_user_grade` only
//! reads; nothing is written back).
//!
//! `#[ignore]`d: needs `POOL_FOOTPRINT_DATABASE_URL` and the EN tree.
//! ```text
//! POOL_FOOTPRINT_DATABASE_URL=postgres://... cargo test --test pool_footprint_test -- --ignored --nocapture
//! ```

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs POOL_FOOTPRINT_DATABASE_URL and the EN tree"]
async fn regrade_connection_footprint() {
    let Ok(url) = std::env::var("POOL_FOOTPRINT_DATABASE_URL") else {
        eprintln!("POOL_FOOTPRINT_DATABASE_URL unset; skipped");
        return;
    };
    let gd = common::shared_game_data();
    for concurrency in [1usize, 4] {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(64)
            .min_connections(0)
            .connect(&url)
            .await
            .expect("connect");
        let users: i64 = std::env::var("FOOTPRINT_USERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8);
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT $1")
            .bind(users)
            .fetch_all(&pool)
            .await
            .expect("user ids");
        let peak = Arc::new(AtomicU32::new(0));
        let sampler = {
            let pool = pool.clone();
            let peak = Arc::clone(&peak);
            tokio::spawn(async move {
                loop {
                    let busy = pool
                        .size()
                        .saturating_sub(u32::try_from(pool.num_idle()).unwrap_or(u32::MAX));
                    peak.fetch_max(busy, Ordering::Relaxed);
                    tokio::time::sleep(Duration::from_micros(200)).await;
                }
            })
        };
        let sem = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut set = tokio::task::JoinSet::new();
        for id in ids {
            let permit = Arc::clone(&sem).acquire_owned().await.expect("permit");
            let pool = pool.clone();
            let gd = Arc::clone(&gd);
            set.spawn(async move {
                let _p = permit;
                backend::core::grade::calculate::calculate_user_grade(&pool, id, &gd)
                    .await
                    .is_ok()
            });
        }
        let mut ok = 0;
        while let Some(r) = set.join_next().await {
            ok += usize::from(r.expect("join"));
        }
        sampler.abort();
        eprintln!(
            "concurrency {concurrency}: {ok} grades, peak busy connections {}, opened {}",
            peak.load(Ordering::Relaxed),
            pool.size()
        );
    }
}
