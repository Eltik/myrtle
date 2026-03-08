use serde::Deserialize;

/// Raw JSON response from version endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    pub res_version: String,
    pub client_version: String,
}

/// A single asset bundle file
#[derive(Debug, Clone)]
pub struct HotFile {
    pub name: String,
    pub total_size: u64,
    pub md5: String,
}

/// A group of files (pack)
#[derive(Debug)]
pub struct HotGroup {
    pub name: String,
    pub total_size: u64,
    pub files: Vec<HotFile>,
}

/// What we pass to the download pipeline
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub filename: String,
    pub md5: String,
    pub total_size: u64,
}

/// Stats returned after pipeline completes
#[derive(Debug, Default)]
pub struct PipelineStats {
    pub downloaded: usize,
    pub failed: usize,
    pub total_bytes: u64,
}
