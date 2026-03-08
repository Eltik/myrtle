use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloaderError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("invalid server: {0}")]
    InvalidServer(String),

    #[error("zip extraction failed: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
