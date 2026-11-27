//! Monster Siren API

use std::time::Duration;

use chrono::NaiveDate;
use reqwest::{Client, RequestBuilder, Response, header};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::UpstreamError;
use crate::fetch::read_bytes;

pub const DEFAULT_SIREN_BASE: &str = "https://monster-siren.hypergryph.com/api";
const USER_AGENT: &str = "myrtle-music-sync (+https://myrtle.moe)";
const TIMEOUT: Duration = Duration::from_secs(20);

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
    /// Signed, valid about three hours from request
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
    /// `base` is [`DEFAULT_SIREN_BASE`]
    #[must_use]
    pub fn new(http: &'a Client, base: &str) -> Self {
        Self {
            http,
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    fn request(&self, url: &str) -> RequestBuilder {
        self.http
            .get(url)
            .timeout(TIMEOUT)
            .header(header::USER_AGENT, USER_AGENT)
    }

    async fn get<T: DeserializeOwned>(
        &self,
        context: &'static str,
        path: &str,
    ) -> Result<T, UpstreamError> {
        let url = format!("{}{path}", self.base);
        let response = send(
            &url,
            self.request(&url)
                .header(header::ACCEPT, "application/json"),
        )
        .await?;
        let body = read_bytes(response, context).await?;
        let envelope: Envelope<T> = serde_json::from_slice(&body)
            .map_err(|e| UpstreamError::parse(context, format!("decode {url}: {e}")))?;
        if envelope.code != 0 {
            return Err(UpstreamError::ApiCode {
                url,
                code: envelope.code,
                message: envelope.msg,
            });
        }
        envelope
            .data
            .ok_or_else(|| UpstreamError::parse(context, format!("{url}: no data")))
    }

    pub async fn albums(&self) -> Result<Vec<ApiAlbumSummary>, UpstreamError> {
        self.get("siren::albums", "/albums").await
    }

    pub async fn songs(&self) -> Result<Vec<ApiSongSummary>, UpstreamError> {
        Ok(self
            .get::<ApiSongList>("siren::songs", "/songs")
            .await?
            .list)
    }

    pub async fn album_detail(&self, cid: &str) -> Result<ApiAlbumDetail, UpstreamError> {
        self.get(
            "siren::album/detail",
            &format!("/album/{}/detail", urlencoding::encode(cid)),
        )
        .await
    }

    pub async fn song_detail(&self, cid: &str) -> Result<ApiSongDetail, UpstreamError> {
        self.get(
            "siren::song",
            &format!("/song/{}", urlencoding::encode(cid)),
        )
        .await
    }

    pub async fn text(&self, url: &str) -> Result<String, UpstreamError> {
        let bytes = send(url, self.request(url))
            .await?
            .bytes()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    UpstreamError::Timeout(e)
                } else {
                    UpstreamError::parse("siren::cdn", format!("{url}: read body: {e}"))
                }
            })?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

async fn send(url: &str, req: RequestBuilder) -> Result<Response, UpstreamError> {
    let response = req.send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(UpstreamError::HttpStatus {
            url: url.to_owned(),
            status: status.as_u16(),
        });
    }
    Ok(response)
}

#[must_use]
pub fn signed_url_expiry(url: &str) -> Option<u64> {
    let rest = url.split("://").nth(1)?;
    let mut segs = rest.split('/');
    let _host = segs.next()?;
    let _sig = segs.next()?;
    u64::from_str_radix(segs.next()?, 16).ok()
}

/// The upload day in a CDN path (`/siren/pic/20261010/...`)
#[must_use]
pub fn cdn_day(url: &str) -> Option<NaiveDate> {
    url.split('/')
        .filter(|seg| seg.len() == 8 && seg.bytes().all(|b| b.is_ascii_digit()))
        .find_map(|seg| NaiveDate::parse_from_str(seg, "%Y%m%d").ok())
}
