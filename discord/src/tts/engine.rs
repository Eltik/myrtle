//! Speech from Google Translate's `translate_tts` endpoint: one MP3 per chunk of text.
//!
//! The endpoint is unofficial and keyless. It takes at most [`MAX_UNITS`] UTF-16 code units of
//! text per request (measured 2026-10-08: 200 ASCII, 200 kana and 100 emoji pass; 201 of any, or
//! 100 emoji and one letter, answer HTTP 400 with an HTML page), so [`super::text::chunks`]
//! splits a message to fit. Requests share the bot's HTTP client, at most [`CONCURRENT`] at a
//! time across every guild.

use std::time::{Duration, Instant};

use reqwest::{StatusCode, Url};
use tokio::sync::Semaphore;

use super::voices::Voice;

/// Text per request, in UTF-16 code units, the unit the endpoint counts in.
pub const MAX_UNITS: usize = 200;

/// Requests to Google in flight at once, across every guild.
const CONCURRENT: usize = 2;

/// Per-request deadline.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Wait before the one retry of a 429, a 5xx, or a network error.
const RETRY_AFTER: Duration = Duration::from_millis(750);

const ENDPOINT: &str = "https://translate.google.com/translate_tts";

/// A desktop browser's agent: without one the endpoint is more likely to refuse.
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                          (KHTML, like Gecko) Chrome/126.0 Safari/537.36";

/// The request URL for `text` in `voice`.
#[must_use]
pub fn tts_url(voice: Voice, text: &str) -> Url {
    let mut url = Url::parse(ENDPOINT).expect("the endpoint is a valid URL");
    url.query_pairs_mut()
        .append_pair("ie", "UTF-8")
        .append_pair("client", "tw-ob")
        .append_pair("tl", voice.tl)
        .append_pair("q", text);
    url
}

/// Whether a response body is MP3: an ID3 tag or an MPEG frame sync. Google answers refusals
/// (a captcha, an over-long text) with HTML, sometimes under a 200.
#[must_use]
pub fn looks_like_mp3(body: &[u8]) -> bool {
    body.starts_with(b"ID3") || (body.len() > 1 && body[0] == 0xFF && body[1] & 0xE0 == 0xE0)
}

/// Fetches speech, at most [`CONCURRENT`] requests at a time.
pub struct Google {
    http: reqwest::Client,
    permits: Semaphore,
}

impl Google {
    #[must_use]
    pub const fn new(http: reqwest::Client) -> Self {
        Self {
            http,
            permits: Semaphore::const_new(CONCURRENT),
        }
    }

    /// MP3 bytes for `text` in `voice`, or `None` after a logged failure. Retries once on a 429,
    /// a 5xx or a network error; a 4xx or a non-audio body is final.
    pub async fn fetch(&self, voice: Voice, text: &str) -> Option<Vec<u8>> {
        let _permit = self.permits.acquire().await.ok()?;
        let url = tts_url(voice, text);
        for attempt in 1..=2 {
            let started = Instant::now();
            match self.once(&url).await {
                Ok(bytes) => {
                    tracing::debug!(
                        "TTS fetched {} B for {} chars in {} ms",
                        bytes.len(),
                        text.chars().count(),
                        started.elapsed().as_millis()
                    );
                    return Some(bytes);
                }
                Err(Failure::Retry(why)) if attempt == 1 => {
                    tracing::debug!("TTS request failed ({why}); retrying");
                    tokio::time::sleep(RETRY_AFTER).await;
                }
                Err(Failure::Retry(why) | Failure::Final(why)) => {
                    tracing::warn!(
                        "TTS skipped a chunk of {} chars in {}: {why}",
                        text.chars().count(),
                        voice.id
                    );
                    return None;
                }
            }
        }
        None
    }

    async fn once(&self, url: &Url) -> Result<Vec<u8>, Failure> {
        let response = self
            .http
            .get(url.clone())
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .timeout(TIMEOUT)
            .send()
            .await
            .map_err(|e| Failure::Retry(format!("request: {e}")))?;
        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
            return Err(Failure::Retry(format!("HTTP {status}")));
        }
        if !status.is_success() {
            return Err(Failure::Final(format!("HTTP {status}")));
        }
        let audio_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|t| t.starts_with("audio/"));
        let body = response
            .bytes()
            .await
            .map_err(|e| Failure::Retry(format!("body: {e}")))?;
        if !audio_type || !looks_like_mp3(&body) {
            return Err(Failure::Final(format!(
                "not audio ({} B, HTTP {status})",
                body.len()
            )));
        }
        Ok(body.to_vec())
    }
}

enum Failure {
    Retry(String),
    Final(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::voices::find;

    #[test]
    fn url_per_voice() {
        let url = tts_url(find("en-gb").unwrap(), "Hello there, Doctor.");
        assert_eq!(url.host_str(), Some("translate.google.com"));
        assert_eq!(
            url.as_str(),
            "https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en-GB\
             &q=Hello+there%2C+Doctor."
        );
        let ja = tts_url(find("ja").unwrap(), "こんにちは & ?");
        let q: Vec<_> = ja.query_pairs().collect();
        assert!(q.contains(&("tl".into(), "ja".into())));
        assert!(
            q.contains(&("q".into(), "こんにちは & ?".into())),
            "the text round-trips"
        );
        assert!(ja.as_str().contains("q=%E3%81%93"), "encoded as UTF-8");
    }

    #[test]
    fn mp3_sniffing() {
        assert!(looks_like_mp3(include_bytes!(
            "../../tests/fixtures/hello-en-us.mp3"
        )));
        assert!(looks_like_mp3(&[0xFF, 0xF3, 0x44]));
        assert!(!looks_like_mp3(b"<!DOCTYPE html><html>"));
        assert!(!looks_like_mp3(b""));
    }

    /// The real request path against Google: every voice's first chunk comes back as MP3.
    /// Needs the network, so it is ignored by default:
    /// `cargo test -- --ignored --nocapture every_voice_fetches_live`.
    #[tokio::test]
    #[ignore = "calls Google Translate"]
    async fn every_voice_fetches_live() {
        let google = Google::new(reqwest::Client::new());
        for voice in crate::tts::voices::VOICES {
            let started = Instant::now();
            let mp3 = google.fetch(*voice, "Doctor, 1, 2, 3.").await;
            let bytes = mp3.as_ref().map_or(0, Vec::len);
            println!(
                "{}: {bytes} B in {} ms",
                voice.id,
                started.elapsed().as_millis()
            );
            assert!(bytes > 0, "{} returned no audio", voice.id);
        }
        let over = "a ".repeat(101);
        assert!(
            google
                .fetch(crate::tts::voices::FALLBACK, &over)
                .await
                .is_none(),
            "201 units is refused, and the refusal is not mistaken for audio"
        );
    }
}
