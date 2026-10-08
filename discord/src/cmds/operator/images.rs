//! Which of a view's image links the backend can't serve.
//!
//! Some images the data names don't exist in the game's files on any server (13 summon
//! avatars and 10 reserve operators' module icons, 2026-10-08), so the backend answers 404 and
//! Discord would draw a broken frame. Before a view is sent, every thumbnail and image link it
//! sets is checked with a HEAD request, and a link the backend answers 404 for is left out.
//!
//! Answers are remembered per URL for [`TTL`], both ways. A timeout or any other failure keeps
//! the link (fail open) and is not remembered, so the next view asks again.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures_util::future::join_all;

/// How long an answer is trusted. The game's files change on patch days.
pub const TTL: Duration = Duration::from_hours(24);
/// The most URLs remembered. A full view sets about ten; the whole census used 5,333.
const MEMO_MAX: usize = 8192;
/// A probe slower than this keeps its link.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// What a probe learned about one URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// The backend answered 404: the image doesn't exist.
    Missing,
    /// The backend answered with a success status.
    Present,
    /// A timeout, a network error, or any other status: nothing learned.
    Unknown,
}

/// Something that can ask whether a URL exists. The real one is [`HttpProber`]; tests use a
/// fake that counts calls.
pub trait Prober: Sync {
    fn probe(&self, url: &str) -> impl Future<Output = Probe> + Send;
}

/// A HEAD request with [`PROBE_TIMEOUT`].
pub struct HttpProber<'a>(pub &'a reqwest::Client);

impl Prober for HttpProber<'_> {
    fn probe(&self, url: &str) -> impl Future<Output = Probe> + Send {
        let request = self.0.head(url).timeout(PROBE_TIMEOUT).send();
        async move {
            match request.await {
                Ok(r) if r.status() == reqwest::StatusCode::NOT_FOUND => Probe::Missing,
                Ok(r) if r.status().is_success() => Probe::Present,
                _ => Probe::Unknown,
            }
        }
    }
}

/// The per-URL memo.
///
/// The lock is only ever held to read or write the map, never across a probe. Two views probing the same unknown URL at the same moment both probe it; that costs
/// one extra HEAD and was not worth a second map of in-flight requests.
#[derive(Default)]
pub struct ImageCheck {
    memo: Mutex<HashMap<String, (Instant, bool)>>,
}

impl ImageCheck {
    /// Of `urls`, the ones to leave out: those the backend answered 404 for, now or within
    /// [`TTL`]. Unknown URLs are probed in parallel.
    pub async fn missing<P: Prober>(&self, prober: &P, urls: &[String]) -> HashSet<String> {
        let mut missing = HashSet::new();
        let mut unknown: Vec<&str> = Vec::new();
        {
            let memo = self
                .memo
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for url in urls {
                match memo.get(url.as_str()) {
                    Some((at, exists)) if at.elapsed() < TTL => {
                        if !exists {
                            missing.insert(url.clone());
                        }
                    }
                    _ if !unknown.contains(&url.as_str()) => unknown.push(url),
                    _ => {}
                }
            }
        }
        if unknown.is_empty() {
            return missing;
        }
        let answers = join_all(unknown.iter().map(|url| prober.probe(url))).await;
        let mut memo = self
            .memo
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if memo.len() + unknown.len() > MEMO_MAX {
            memo.retain(|_, (at, _)| at.elapsed() < TTL);
            if memo.len() + unknown.len() > MEMO_MAX {
                memo.clear();
            }
        }
        let now = Instant::now();
        for (url, answer) in unknown.into_iter().zip(answers) {
            match answer {
                Probe::Missing => {
                    memo.insert(url.to_string(), (now, false));
                    missing.insert(url.to_string());
                }
                Probe::Present => {
                    memo.insert(url.to_string(), (now, true));
                }
                Probe::Unknown => {}
            }
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Answers by URL suffix and counts every probe.
    struct Fake {
        calls: AtomicUsize,
    }

    impl Prober for Fake {
        fn probe(&self, url: &str) -> impl Future<Output = Probe> + Send {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let answer = if url.ends_with("/404") {
                Probe::Missing
            } else if url.ends_with("/slow") {
                Probe::Unknown
            } else {
                Probe::Present
            };
            async move { answer }
        }
    }

    fn urls(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    #[tokio::test]
    async fn drops_404_keeps_the_rest_and_remembers() {
        let check = ImageCheck::default();
        let fake = Fake {
            calls: AtomicUsize::new(0),
        };
        let list = urls(&["a/200", "b/404", "c/slow", "b/404"]);
        let missing = check.missing(&fake, &list).await;
        assert_eq!(missing, HashSet::from(["b/404".to_string()]));
        // The repeated URL was probed once.
        assert_eq!(fake.calls.load(Ordering::SeqCst), 3);

        // Second view: 200 and 404 come from the memo; the timed-out one is asked again.
        let missing = check.missing(&fake, &list).await;
        assert_eq!(missing, HashSet::from(["b/404".to_string()]));
        assert_eq!(fake.calls.load(Ordering::SeqCst), 4);

        // An expired answer is probed again.
        check.memo.lock().unwrap().get_mut("a/200").unwrap().0 = Instant::now()
            .checked_sub(TTL + Duration::from_secs(1))
            .unwrap();
        check.missing(&fake, &urls(&["a/200"])).await;
        assert_eq!(fake.calls.load(Ordering::SeqCst), 5);

        // Nothing to check, nothing probed.
        assert!(check.missing(&fake, &[]).await.is_empty());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 5);
    }
}
