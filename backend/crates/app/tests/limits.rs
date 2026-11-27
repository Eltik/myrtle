#![allow(clippy::unwrap_used)]

use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::config::CpuConfig;
use myrtle_app::limits::CpuLimiter;

fn limiter(task_permits: usize, task_wait_ms: u64, task_queue: usize) -> CpuLimiter {
    CpuLimiter::new(&CpuConfig {
        task_permits,
        light_permits: 0,
        task_wait_ms,
        task_queue,
    })
}

#[test]
fn zeroes_take_the_derived_defaults() {
    assert!(CpuLimiter::default().permits() >= 1);
    assert_eq!(limiter(3, 0, 0).permits(), 3);
}

#[tokio::test]
async fn work_runs_and_the_permit_comes_back() {
    let cpu = limiter(2, 1_000, 0);
    for i in 0..8 {
        assert_eq!(cpu.run("test", move || i * 2).await.unwrap(), i * 2);
    }
}

#[tokio::test]
async fn a_panicking_task_is_an_internal_error() {
    let cpu = limiter(1, 1_000, 0);
    let failed: Result<(), _> = cpu.run("test", || panic!("boom")).await;
    assert!(matches!(failed, Err(AppError::Internal(_))));
    assert_eq!(
        cpu.run("test", || 1).await.unwrap(),
        1,
        "the permit came back"
    );
}

#[tokio::test]
async fn a_waiter_is_served_when_a_permit_returns() {
    let cpu = limiter(1, 5_000, 0);
    let held = cpu.admit("test").await.unwrap();
    let queued = {
        let cpu = cpu.clone();
        tokio::spawn(async move { cpu.admit("test").await.map(drop) })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cpu.waiting(), 1, "queued, not refused");

    drop(held);
    assert!(queued.await.unwrap().is_ok());
    assert_eq!(cpu.waiting(), 0);
}

#[tokio::test]
async fn no_wait_refuses_at_once() {
    let cpu = limiter(1, 0, 0);
    let _held = cpu.admit("test").await.unwrap();
    assert!(matches!(
        cpu.admit("test").await,
        Err(AppError::Unavailable)
    ));
    assert_eq!(cpu.waiting(), 0);
}

#[tokio::test]
async fn a_full_queue_refuses_the_next_arrival() {
    let cpu = limiter(1, 5_000, 1);
    let _held = cpu.admit("test").await.unwrap();
    let queued = {
        let cpu = cpu.clone();
        tokio::spawn(async move { cpu.admit("test").await.map(drop) })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(matches!(
        cpu.admit("test").await,
        Err(AppError::Unavailable)
    ));
    assert_eq!(cpu.waiting(), 1, "the refused arrival left the queue");
    queued.abort();
}

#[tokio::test]
async fn a_wait_that_outlasts_the_budget_is_refused() {
    let cpu = limiter(1, 30, 0);
    let _held = cpu.admit("test").await.unwrap();
    assert!(matches!(
        cpu.admit("test").await,
        Err(AppError::Unavailable)
    ));
    assert_eq!(cpu.waiting(), 0);
}

#[tokio::test]
async fn a_cancelled_waiter_releases_its_slot() {
    let cpu = limiter(1, 5_000, 0);
    let held = cpu.admit("test").await.unwrap();
    let abandoned = {
        let cpu = cpu.clone();
        tokio::spawn(async move { cpu.admit("test").await.map(drop) })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cpu.waiting(), 1);

    abandoned.abort();
    let _ = abandoned.await;
    assert_eq!(cpu.waiting(), 0);
    drop(held);
    assert!(cpu.admit("test").await.is_ok());
}

#[tokio::test]
async fn the_light_pool_is_not_starved_by_the_heavy_one() {
    let cpu = limiter(1, 0, 0);
    let _heavy = cpu.admit("test").await.unwrap();
    assert!(cpu.admit_light("test").await.is_ok());
}

#[tokio::test]
async fn a_dropped_caller_does_not_free_the_permit_while_its_work_runs() {
    let cpu = limiter(1, 0, 0);
    let caller = {
        let cpu = cpu.clone();
        tokio::spawn(async move {
            cpu.run("test", || std::thread::sleep(Duration::from_millis(300)))
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(50)).await;
    caller.abort();
    let _ = caller.await;
    assert!(
        matches!(cpu.admit("test").await, Err(AppError::Unavailable)),
        "the blocking task still holds the permit"
    );
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(cpu.admit("test").await.is_ok());
}
