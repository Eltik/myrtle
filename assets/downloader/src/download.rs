use crate::server::Server;
use crate::types::DownloadTask;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct Downloader {
    client: Client,
    server: Server,
    version: String,
    semaphore: Arc<Semaphore>,
}

impl Downloader {
    pub fn new(server: Server, version: String, max_concurrent: usize) -> Self {
        let client = Client::builder()
            .user_agent("BestHTTP")
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            server,
            version,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Transform filename to CDN URL
    fn build_url(&self, filename: &str) -> String {
        // Replace last extension with .dat
        let dat_name = replace_last_ext(filename, "dat");
        let encoded = dat_name.replace('/', "_").replace('#', "__");
        format!(
            "{}/{}/{}",
            self.server.cdn_base_url(),
            self.version,
            encoded
        )
    }

    pub async fn download(&self, task: &DownloadTask) -> anyhow::Result<Vec<u8>> {
        let _permit = self.semaphore.acquire().await?;
        let url = self.build_url(&task.filename);
        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }
}

fn replace_last_ext(name: &str, new_ext: &str) -> String {
    // Find last '.' that isn't part of directory separators
    match name.rfind('.') {
        Some(pos) => format!("{}.{}", &name[..pos], new_ext),
        None => format!("{}.{}", name, new_ext),
    }
}
