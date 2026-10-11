//! The public Monster Siren API, as it answers on 2026-10-11.
//!
//! Four JSON endpoints, each wrapped in `{code, msg, data}` with `code` 0 on
//! success:
//!
//! - `GET /api/albums`: `[{cid, name, coverUrl, artistes: [..]}]`, newest first
//!   (298 releases).
//! - `GET /api/songs`: `{list: [{cid, name, albumCid, artists: [..]}], autoplay}`
//!   (911 tracks).
//! - `GET /api/album/{cid}/detail`: `{cid, name, intro, belong, coverUrl,
//!   coverDeUrl, songs: [{cid, name, artistes}]}`.
//! - `GET /api/song/{cid}`: `{cid, name, albumCid, sourceUrl, lyricUrl, mvUrl,
//!   mvCoverUrl, artists}`. `sourceUrl` is SIGNED: `res01.hycdn.cn/<sig>/
//!   <unix expiry, hex>/siren/audio/YYYYMMDD/<md5>.wav`, valid about three
//!   hours from the call (10,614 s measured).
//!
//! Artist names arrive with stray whitespace (` Emperor`, ` D.D.D.`), as do a
//! few album names (` Epilogue`); every string is trimmed on the way in.

use std::time::Duration;

use anyhow::{Context, anyhow};
use reqwest::{Client, RequestBuilder, Response};
use serde::Deserialize;
use serde::de::DeserializeOwned;

/// Overridable so a test or a mirror can point the sync elsewhere.
const DEFAULT_BASE: &str = "https://monster-siren.hypergryph.com/api";
const USER_AGENT: &str = "myrtle-music-sync (+https://myrtle.moe)";
const TIMEOUT: Duration = Duration::from_secs(20);

/// The envelope every endpoint answers with.
#[derive(Debug, Deserialize)]
struct Envelope<T> {
    code: i64,
    #[serde(default)]
    msg: String,
    data: Option<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAlbumSummary {
    pub cid: String,
    pub name: String,
    pub cover_url: String,
    #[serde(default)]
    pub artistes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSongSummary {
    pub cid: String,
    pub name: String,
    pub album_cid: String,
    #[serde(default)]
    pub artists: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSongList {
    pub list: Vec<ApiSongSummary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAlbumSong {
    pub cid: String,
    pub name: String,
    #[serde(default)]
    pub artistes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAlbumDetail {
    pub cid: String,
    pub name: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub belong: String,
    pub cover_url: String,
    pub cover_de_url: Option<String>,
    #[serde(default)]
    pub songs: Vec<ApiAlbumSong>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSongDetail {
    pub cid: String,
    pub name: String,
    pub album_cid: String,
    pub source_url: Option<String>,
    pub lyric_url: Option<String>,
    pub mv_url: Option<String>,
    pub mv_cover_url: Option<String>,
    #[serde(default)]
    pub artists: Vec<String>,
}

pub struct SirenClient<'a> {
    http: &'a Client,
    base: String,
}

impl<'a> SirenClient<'a> {
    pub fn new(http: &'a Client) -> Self {
        let base = std::env::var("SIREN_API_BASE")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_owned());
        Self {
            http,
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    /// A GET to `url` with the sync's timeout and user agent.
    fn request(&self, url: &str) -> RequestBuilder {
        self.http
            .get(url)
            .timeout(TIMEOUT)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{path}", self.base);
        let resp = send(
            &url,
            self.request(&url)
                .header(reqwest::header::ACCEPT, "application/json"),
        )
        .await?;
        let env: Envelope<T> = resp.json().await.with_context(|| format!("decode {url}"))?;
        if env.code != 0 {
            return Err(anyhow!("{url}: code {} {}", env.code, env.msg));
        }
        env.data.ok_or_else(|| anyhow!("{url}: no data"))
    }

    pub async fn albums(&self) -> anyhow::Result<Vec<ApiAlbumSummary>> {
        self.get("/albums").await
    }

    pub async fn songs(&self) -> anyhow::Result<Vec<ApiSongSummary>> {
        Ok(self.get::<ApiSongList>("/songs").await?.list)
    }

    pub async fn album_detail(&self, cid: &str) -> anyhow::Result<ApiAlbumDetail> {
        self.get(&format!("/album/{}/detail", urlencoding::encode(cid)))
            .await
    }

    pub async fn song_detail(&self, cid: &str) -> anyhow::Result<ApiSongDetail> {
        self.get(&format!("/song/{}", urlencoding::encode(cid)))
            .await
    }

    /// A file on the label's CDN (an LRC lyrics file), as text.
    pub async fn text(&self, url: &str) -> anyhow::Result<String> {
        let resp = send(url, self.request(url)).await?;
        let bytes = resp.bytes().await.with_context(|| format!("read {url}"))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// Send `req`, a non-2xx status an error naming `url`.
async fn send(url: &str, req: RequestBuilder) -> anyhow::Result<Response> {
    req.send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("GET {url}"))
}

/// The unix second a signed stream URL stops working, read from its second
/// path segment (`/<sig>/<HEX>/siren/...`). `None` when the URL is not shaped
/// like that, so the caller picks a short safety window instead.
pub fn signed_url_expiry(url: &str) -> Option<u64> {
    let rest = url.split("://").nth(1)?;
    let mut segs = rest.split('/');
    let _host = segs.next()?;
    let _sig = segs.next()?;
    let hex = segs.next()?;
    u64::from_str_radix(hex, 16).ok()
}

/// The upload day in a CDN path (`/siren/pic/20261010/…`, `/siren/lyric/…`).
pub fn cdn_day(url: &str) -> Option<chrono::NaiveDate> {
    url.split('/')
        .filter(|seg| seg.len() == 8 && seg.bytes().all(|b| b.is_ascii_digit()))
        .find_map(|seg| chrono::NaiveDate::parse_from_str(seg, "%Y%m%d").ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expiry_is_the_hex_segment() {
        let url = "https://res01.hycdn.cn/2034bfa9e33b9483d844dcb611b9f0b9/6ACB78D8/siren/audio/20240709/a7f6.wav";
        assert_eq!(signed_url_expiry(url), Some(0x6ACB_78D8));
        assert_eq!(signed_url_expiry("https://example.com/x.wav"), None);
    }

    #[test]
    fn cdn_day_reads_the_eight_digit_segment() {
        let url = "https://web.hycdn.cn/siren/pic/20261010/ab46323f8b0d4580e34b61c837851cc7.jpg";
        assert_eq!(cdn_day(url), chrono::NaiveDate::from_ymd_opt(2026, 10, 10));
        assert_eq!(cdn_day("https://web.hycdn.cn/siren/pic/x.jpg"), None);
    }
}
