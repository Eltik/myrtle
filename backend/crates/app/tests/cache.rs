#![allow(
    clippy::unwrap_used,
    clippy::manual_assert,
    clippy::tuple_array_conversions
)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::cache::{Cache, CachedJson, key_hash, strong_etag};
use myrtle_hypergryph::UpstreamError;

const TTL: Duration = Duration::from_mins(1);

#[tokio::test]
async fn a_value_reads_back_until_it_expires_or_is_invalidated() {
    let cache = Cache::memory();
    cache.set("a", "1", TTL).await;
    cache.set("gone", "2", Duration::ZERO).await;
    assert_eq!(cache.get("a").await.as_deref(), Some("1"));
    assert_eq!(cache.get("gone").await, None);
    assert_eq!(cache.get("never").await, None);

    cache.invalidate("a").await;
    assert_eq!(cache.get("a").await, None);
    assert!(cache.ping().await);
    assert!(!cache.is_redis());
}

#[tokio::test]
async fn a_prefix_invalidation_leaves_other_prefixes() {
    let cache = Cache::memory();
    for key in ["dps:calc:1", "dps:rank:2", "static:en:ops"] {
        cache.set(key, "x", TTL).await;
    }
    cache.invalidate_prefix("dps:").await;
    assert_eq!(cache.get("dps:calc:1").await, None);
    assert_eq!(cache.get("dps:rank:2").await, None);
    assert_eq!(cache.get("static:en:ops").await.as_deref(), Some("x"));
}

#[tokio::test]
async fn a_cached_float_reads_back_bit_for_bit() {
    let values = vec![
        1.229_166_666_666_666_5_f64,
        0.1,
        1.0 / 3.0,
        1e-300,
        -2.5e300,
    ];
    let cache = Cache::memory();
    cache.set_json("floats", &values, TTL).await;
    let back: Vec<f64> = cache.get_json("floats").await.unwrap();
    assert_eq!(back.len(), values.len());
    for (a, b) in values.iter().zip(&back) {
        assert_eq!(a.to_bits(), b.to_bits(), "{a} came back as {b}");
    }
}

#[tokio::test]
async fn json_builds_once_and_serves_the_stored_etag() {
    let cache = Cache::memory();
    let builds = AtomicUsize::new(0);
    let build = || async {
        builds.fetch_add(1, Ordering::Relaxed);
        Ok("{\"n\":1}".to_owned())
    };
    let first = cache.json("k", TTL, build).await.unwrap();
    let second = cache.json("k", TTL, build).await.unwrap();
    assert_eq!(builds.load(Ordering::Relaxed), 1);
    assert_eq!(first, second);
    assert_eq!(first, CachedJson::new("{\"n\":1}".into()));
    assert_eq!(first.etag, strong_etag(b"{\"n\":1}"));
}

#[tokio::test]
async fn a_failed_build_is_not_cached() {
    let cache = Cache::memory();
    let refused = cache
        .json("k", TTL, || async { Err(AppError::Forbidden) })
        .await;
    assert!(matches!(refused, Err(AppError::Forbidden)));
    let built = cache.json("k", TTL, || async { Ok("1".to_owned()) }).await;
    assert_eq!(&*built.unwrap().body, "1");
}

#[tokio::test]
async fn concurrent_detached_callers_share_one_build() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let call = || {
        let (cache, builds) = (cache.clone(), Arc::clone(&builds));
        tokio::spawn(async move {
            cache
                .json_detached("k", TTL, move || async move {
                    builds.fetch_add(1, Ordering::Relaxed);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Ok("body".to_owned())
                })
                .await
        })
    };
    let callers: Vec<_> = (0..8).map(|_| call()).collect();
    for caller in callers {
        assert_eq!(&*caller.await.unwrap().unwrap().body, "body");
    }
    assert_eq!(builds.load(Ordering::Relaxed), 1);
    assert_eq!(&*call().await.unwrap().unwrap().body, "body");
    assert_eq!(builds.load(Ordering::Relaxed), 1, "the stored body answers");
}

#[tokio::test]
async fn a_detached_build_outlives_the_caller_that_started_it() {
    let cache = Cache::memory();
    let started = {
        let cache = cache.clone();
        tokio::spawn(async move {
            cache
                .json_detached("k", TTL, || async {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Ok("body".to_owned())
                })
                .await
        })
    };
    tokio::time::sleep(Duration::from_millis(10)).await;
    started.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let cached: CachedJson = cache.get_json("k").await.unwrap();
    assert_eq!(&*cached.body, "body");
}

#[tokio::test]
async fn a_detached_error_reaches_every_waiter_and_is_not_cached() {
    let cache = Cache::memory();
    let failing = |cache: Cache| {
        tokio::spawn(async move {
            cache
                .json_detached("k", TTL, || async {
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    Err(AppError::Conflict("taken".into()))
                })
                .await
        })
    };
    let (a, b) = (failing(cache.clone()), failing(cache.clone()));
    for waiter in [a, b] {
        assert!(matches!(waiter.await.unwrap(), Err(AppError::Conflict(m)) if m == "taken"));
    }
    assert_eq!(cache.get("k").await, None);
}

#[tokio::test]
async fn a_panicking_build_does_not_poison_its_key() {
    let cache = Cache::memory();
    let panicked = cache
        .json_detached("k", TTL, || async {
            if true {
                panic!("build blew up");
            }
            Ok(String::new())
        })
        .await;
    assert!(matches!(panicked, Err(AppError::Unavailable)));
    let rebuilt = cache
        .json_detached("k", TTL, || async { Ok("ok".to_owned()) })
        .await;
    assert_eq!(&*rebuilt.unwrap().body, "ok");
}

#[test]
fn key_hashes_are_stable_and_distinct() {
    assert_eq!(key_hash(b"abc"), "ba7816bf8f01cfea");
    assert_ne!(key_hash(b"abc"), key_hash(b"abd"));
}

type Build = Pin<Box<dyn Future<Output = Result<String, AppError>> + Send>>;

fn counted(builds: &Arc<AtomicUsize>, delay: Duration) -> impl FnOnce() -> Build + Send + 'static {
    let builds = Arc::clone(builds);
    move || {
        Box::pin(async move {
            builds.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(delay).await;
            Ok("{\"big\":true}".to_owned())
        })
    }
}

#[tokio::test]
async fn a_shared_hit_returns_the_same_allocation_without_the_store() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let first = cache
        .shared("static:en:ops", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    let second = cache
        .shared("static:en:ops", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    assert!(Arc::ptr_eq(&first.body, &second.body));
    assert_eq!(first.etag, strong_etag(b"{\"big\":true}"));
    assert_eq!(builds.load(Ordering::SeqCst), 1);
    assert_eq!(cache.get("static:en:ops").await, None);
}

#[tokio::test]
async fn a_reload_flush_drops_a_shared_body_and_spares_other_prefixes() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let call = |key: &'static str| {
        let (cache, build) = (cache.clone(), counted(&builds, Duration::ZERO));
        async move { cache.shared(key, TTL, build).await.unwrap() }
    };
    let en = call("static:en:ops").await;
    let cn = call("static:cn:ops").await;
    cache.invalidate_prefix("static:en:").await;
    assert!(!Arc::ptr_eq(&en.body, &call("static:en:ops").await.body));
    assert!(Arc::ptr_eq(&cn.body, &call("static:cn:ops").await.body));
    assert_eq!(builds.load(Ordering::SeqCst), 3);
    cache.invalidate("static:cn:ops").await;
    assert!(!Arc::ptr_eq(&cn.body, &call("static:cn:ops").await.body));
}

#[tokio::test]
async fn a_shared_build_raced_by_a_flush_answers_its_waiters_but_is_not_kept() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let racing = {
        let (cache, build) = (cache.clone(), counted(&builds, Duration::from_millis(50)));
        tokio::spawn(async move { cache.shared("static:en:ops", TTL, build).await })
    };
    tokio::time::sleep(Duration::from_millis(10)).await;
    cache.invalidate_prefix("static:").await;
    assert!(racing.await.unwrap().is_ok());
    cache
        .shared("static:en:ops", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    assert_eq!(
        builds.load(Ordering::SeqCst),
        2,
        "the raced body was dropped"
    );
}

#[tokio::test]
async fn an_unrelated_flush_keeps_a_shared_build() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let building = {
        let (cache, build) = (cache.clone(), counted(&builds, Duration::from_millis(50)));
        tokio::spawn(async move { cache.shared("static:en:ops", TTL, build).await })
    };
    tokio::time::sleep(Duration::from_millis(10)).await;
    cache.invalidate_prefix("user:").await;
    building.await.unwrap().unwrap();
    cache
        .shared("static:en:ops", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    assert_eq!(builds.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_failed_shared_build_is_not_kept() {
    let cache = Cache::memory();
    let refused = cache
        .shared("k", TTL, || async { Err(AppError::NotFound(None)) })
        .await;
    assert!(matches!(refused, Err(AppError::NotFound(None))));
    let built = cache
        .shared("k", TTL, || async { Ok("1".to_owned()) })
        .await;
    assert_eq!(&*built.unwrap().body, "1");
}

#[test]
fn a_shared_maintenance_error_keeps_its_shape() {
    let upstream = AppError::Upstream(UpstreamError::Maintenance(Box::default()));
    assert!(matches!(
        upstream.shared_copy(),
        AppError::GameMaintenance { ends_at: None }
    ));
    let known = AppError::GameMaintenance {
        ends_at: Some(1_700_000_000),
    };
    assert!(matches!(
        known.shared_copy(),
        AppError::GameMaintenance {
            ends_at: Some(1_700_000_000)
        }
    ));
}

#[tokio::test]
async fn a_detached_build_raced_by_an_invalidation_is_not_stored() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let racing = {
        let (cache, build) = (cache.clone(), counted(&builds, Duration::from_millis(50)));
        tokio::spawn(async move { cache.json_detached("static:en:x", TTL, build).await })
    };
    tokio::time::sleep(Duration::from_millis(10)).await;
    cache.invalidate("static:en:x").await;
    assert!(
        racing.await.unwrap().is_ok(),
        "its waiters still get the body"
    );
    assert_eq!(cache.get("static:en:x").await, None);
    cache
        .json_detached("static:en:x", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    assert!(cache.get("static:en:x").await.is_some());
    assert_eq!(builds.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn clearing_shared_drops_every_body_and_outdates_a_build_in_flight() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let kept = cache
        .shared("story:en:1", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    let racing = {
        let (cache, build) = (cache.clone(), counted(&builds, Duration::from_millis(50)));
        tokio::spawn(async move { cache.shared("static:kr:ops", TTL, build).await })
    };
    tokio::time::sleep(Duration::from_millis(10)).await;
    cache.clear_shared();
    assert!(racing.await.unwrap().is_ok());
    let again = cache
        .shared("story:en:1", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    assert!(!Arc::ptr_eq(&kept.body, &again.body));
    cache
        .shared("static:kr:ops", TTL, counted(&builds, Duration::ZERO))
        .await
        .unwrap();
    assert_eq!(builds.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn an_invalidation_discards_only_the_builds_it_covers() {
    let cache = Cache::memory();
    let builds = Arc::new(AtomicUsize::new(0));
    let start = |key: &'static str| {
        let (cache, build) = (cache.clone(), counted(&builds, Duration::from_millis(50)));
        tokio::spawn(async move { cache.json_detached(key, TTL, build).await })
    };
    let operators = start("static:en:operators");
    let improvements = start("improvements:7:a");
    let kept = start("improvements:8:a");
    tokio::time::sleep(Duration::from_millis(10)).await;
    cache.invalidate_prefix("improvements:7:").await;
    for build in [operators, improvements, kept] {
        build.await.unwrap().unwrap();
    }
    assert!(
        cache.get("static:en:operators").await.is_some(),
        "an unrelated build is kept"
    );
    assert!(cache.get("improvements:8:a").await.is_some());
    assert_eq!(
        cache.get("improvements:7:a").await,
        None,
        "the covered one is not"
    );
    assert_eq!(builds.load(Ordering::SeqCst), 3);
}
