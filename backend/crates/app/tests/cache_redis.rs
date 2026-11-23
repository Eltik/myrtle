#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::cache::{Cache, CachedJson, strong_etag};

const URL: &str = "redis://127.0.0.1:6379/15";
const TTL: Duration = Duration::from_mins(1);

async fn redis(test: &str) -> Option<(Cache, String)> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 6379));
    if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_err() {
        eprintln!("skipping {test}: no Redis at 127.0.0.1:6379");
        return None;
    }
    let cache = Cache::connect(Some(URL)).await.unwrap();
    assert!(cache.is_redis());
    Some((
        cache,
        format!("myrtle-test:{}:", uuid::Uuid::new_v4().simple()),
    ))
}

#[tokio::test]
async fn a_value_reads_back_until_it_expires_or_is_invalidated() {
    let Some((cache, p)) = redis("a_value_reads_back").await else {
        return;
    };
    let (a, gone, short) = (format!("{p}a"), format!("{p}gone"), format!("{p}short"));
    cache.set(&a, "1", TTL).await;
    cache.set(&gone, "2", TTL).await;
    cache.set(&gone, "3", Duration::ZERO).await;
    cache.set(&short, "4", Duration::from_millis(300)).await;
    assert_eq!(cache.get(&a).await.as_deref(), Some("1"));
    assert_eq!(
        cache.get(&gone).await,
        None,
        "a zero ttl replaces the value"
    );
    assert_eq!(
        cache.get(&short).await.as_deref(),
        Some("4"),
        "sub-second ttl"
    );
    assert_eq!(cache.get(&format!("{p}never")).await, None);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(cache.get(&short).await, None);

    cache.invalidate(&a).await;
    assert_eq!(cache.get(&a).await, None);
    assert!(cache.ping().await);
}

#[tokio::test]
async fn a_prefix_invalidation_leaves_other_prefixes() {
    let Some((cache, p)) = redis("a_prefix_invalidation").await else {
        return;
    };
    let keys: Vec<String> = (0..250).map(|n| format!("{p}dps:calc:{n}")).collect();
    for key in &keys {
        cache.set(key, "x", TTL).await;
    }
    let kept = format!("{p}static:en:ops");
    cache.set(&kept, "x", TTL).await;
    cache.invalidate_prefix(&format!("{p}dps:")).await;
    for key in &keys {
        assert_eq!(cache.get(key).await, None, "{key} survived the scan");
    }
    assert_eq!(cache.get(&kept).await.as_deref(), Some("x"));
    cache.invalidate_prefix(&p).await;
    assert_eq!(cache.get(&kept).await, None);
}

#[tokio::test]
async fn a_cached_float_reads_back_bit_for_bit() {
    let Some((cache, p)) = redis("a_cached_float").await else {
        return;
    };
    let values = vec![
        1.229_166_666_666_666_5_f64,
        0.1,
        1.0 / 3.0,
        1e-300,
        -2.5e300,
    ];
    cache.set_json(&format!("{p}floats"), &values, TTL).await;
    let back: Vec<f64> = cache.get_json(&format!("{p}floats")).await.unwrap();
    assert_eq!(back.len(), values.len());
    for (a, b) in values.iter().zip(&back) {
        assert_eq!(a.to_bits(), b.to_bits(), "{a} came back as {b}");
    }
    cache.invalidate_prefix(&p).await;
}

#[tokio::test]
async fn json_builds_once_and_serves_the_stored_etag() {
    let Some((cache, p)) = redis("json_builds_once").await else {
        return;
    };
    let key = format!("{p}k");
    let builds = AtomicUsize::new(0);
    let build = || async {
        builds.fetch_add(1, Ordering::Relaxed);
        Ok("{\"n\":1}".to_owned())
    };
    let first = cache.json(&key, TTL, build).await.unwrap();
    let second = cache.json(&key, TTL, build).await.unwrap();
    assert_eq!(builds.load(Ordering::Relaxed), 1);
    assert_eq!(first, second);
    assert_eq!(first.etag, strong_etag(b"{\"n\":1}"));

    let refused = cache
        .json(&format!("{p}refused"), TTL, || async {
            Err(AppError::Forbidden)
        })
        .await;
    assert!(matches!(refused, Err(AppError::Forbidden)));
    assert_eq!(cache.get(&format!("{p}refused")).await, None);
    cache.invalidate_prefix(&p).await;
}

#[tokio::test]
async fn concurrent_detached_callers_share_one_build_and_store_it_in_redis() {
    let Some((cache, p)) = redis("concurrent_detached_callers").await else {
        return;
    };
    let key: Arc<str> = format!("{p}k").into();
    let builds = Arc::new(AtomicUsize::new(0));
    let call = || {
        let (cache, builds, key) = (cache.clone(), Arc::clone(&builds), Arc::clone(&key));
        tokio::spawn(async move {
            cache
                .json_detached(&key, TTL, move || async move {
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

    let other = Cache::connect(Some(URL)).await.unwrap();
    let stored: CachedJson = other.get_json(&key).await.unwrap();
    assert_eq!(
        stored,
        CachedJson::new("body".into()),
        "a second process sees it"
    );
    assert_eq!(&*call().await.unwrap().unwrap().body, "body");
    assert_eq!(builds.load(Ordering::Relaxed), 1, "the stored body answers");
    cache.invalidate_prefix(&p).await;
}

#[tokio::test]
async fn a_detached_error_is_not_stored() {
    let Some((cache, p)) = redis("a_detached_error").await else {
        return;
    };
    let key = format!("{p}k");
    let failed = cache
        .json_detached(&key, TTL, || async {
            Err(AppError::Conflict("taken".into()))
        })
        .await;
    assert!(matches!(failed, Err(AppError::Conflict(m)) if m == "taken"));
    assert_eq!(cache.get(&key).await, None);
}

#[tokio::test]
async fn an_invalidation_reaches_another_process() {
    let Some((cache, p)) = redis("an_invalidation_reaches").await else {
        return;
    };
    let other = Cache::connect(Some(URL)).await.unwrap();
    let key = format!("{p}k");
    cache.set(&key, "v", TTL).await;
    assert_eq!(other.get(&key).await.as_deref(), Some("v"));
    other.invalidate(&key).await;
    assert_eq!(cache.get(&key).await, None);
}
