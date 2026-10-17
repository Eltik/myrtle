//! Fetch: the backend client, with retries.

use std::time::Duration;

use anyhow::{Context, Result, anyhow};

use super::{StoryIndex, StoryScript};

pub struct Fetcher {
    client: reqwest::Client,
    base: reqwest::Url,
    service_key: Option<String>,
}

pub enum Fetched {
    Script(StoryScript),
    /// The index said `hasScript` but the fetch did not return one.
    Failed {
        status: Option<u16>,
        error: String,
    },
}

impl Fetcher {
    /// # Errors
    /// An unparseable base URL.
    pub fn new(base: &str, service_key: Option<String>) -> Result<Self> {
        let base = reqwest::Url::parse(base).with_context(|| format!("bad base url {base}"))?;
        if base.cannot_be_a_base() {
            return Err(anyhow!("base url cannot take a path: {base}"));
        }
        let client = reqwest::Client::builder()
            // The backend's own handler timeout is 30 s; anything past that is
            // not coming back.
            .timeout(Duration::from_secs(45))
            .build()?;
        Ok(Self {
            client,
            base,
            service_key,
        })
    }

    fn url(&self, segments: &[&str]) -> reqwest::Url {
        let mut u = self.base.clone();
        {
            let mut p = u.path_segments_mut().expect("checked in new");
            p.pop_if_empty();
            p.push("api");
            for s in segments {
                p.push(s);
            }
        }
        u
    }

    fn get(&self, url: reqwest::Url) -> reqwest::RequestBuilder {
        let r = self.client.get(url);
        match &self.service_key {
            // The backend exempts this header from its rate limiter.
            Some(k) => r.header("x-service-key", k),
            None => r,
        }
    }

    /// # Errors
    /// Transport failure or a non-success status on the index.
    pub async fn index(&self) -> Result<StoryIndex> {
        let resp = self.get(self.url(&["story", "index"])).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("GET /api/story/index -> {status}"));
        }
        Ok(resp.json().await?)
    }

    /// Fetch one script. Transport errors and 5xx retry three times with
    /// backoff; 4xx do not, because a 404 on a story the index calls scripted
    /// is a data problem that retrying cannot fix.
    pub async fn script(&self, id: &str) -> Fetched {
        let url = self.url(&["story", id]);
        let mut last = String::new();
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << (attempt - 1))).await;
            }
            match self.get(url.clone()).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        return match resp.json::<StoryScript>().await {
                            Ok(s) => Fetched::Script(s),
                            Err(e) => Fetched::Failed {
                                status: Some(status.as_u16()),
                                error: format!("decode: {e}"),
                            },
                        };
                    }
                    if status.is_client_error() {
                        return Fetched::Failed {
                            status: Some(status.as_u16()),
                            error: status.to_string(),
                        };
                    }
                    last = status.to_string();
                }
                Err(e) => last = e.to_string(),
            }
        }
        Fetched::Failed {
            status: None,
            error: format!("after 4 attempts: {last}"),
        }
    }
}
