//! S3 Configuration

/// Configuration for S3-compatible storage connection
#[derive(Debug, Clone)]
pub struct S3Config {
    /// S3 endpoint URL (e.g., "https://s3.example.com" or "http://localhost:9000" for MinIO)
    pub endpoint: String,
    /// Bucket name
    pub bucket: String,
    /// AWS region or region name for S3-compatible service
    pub region: String,
    /// Access key ID
    pub access_key: String,
    /// Secret access key
    pub secret_key: String,
    /// Use path-style URLs (required for MinIO and some S3-compatible services)
    pub path_style: bool,
    /// Optional prefix for all keys (e.g., "assets/en")
    pub prefix: Option<String>,
}

impl S3Config {
    /// Create a new S3 configuration from environment variables
    ///
    /// Expected environment variables:
    /// - S3_ENDPOINT: The S3 endpoint URL
    /// - S3_BUCKET: Bucket name
    /// - S3_REGION: Region name (default: "us-east-1")
    /// - S3_ACCESS_KEY: Access key ID
    /// - S3_SECRET_KEY: Secret access key
    /// - S3_PATH_STYLE: Use path-style URLs ("true" or "false", default: "true")
    /// - S3_PREFIX: Optional prefix for all keys
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            endpoint: std::env::var("S3_ENDPOINT")
                .map_err(|_| "S3_ENDPOINT environment variable not set")?,
            bucket: std::env::var("S3_BUCKET")
                .map_err(|_| "S3_BUCKET environment variable not set")?,
            region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY")
                .map_err(|_| "S3_ACCESS_KEY environment variable not set")?,
            secret_key: std::env::var("S3_SECRET_KEY")
                .map_err(|_| "S3_SECRET_KEY environment variable not set")?,
            path_style: std::env::var("S3_PATH_STYLE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            prefix: std::env::var("S3_PREFIX").ok().filter(|s| !s.is_empty()),
        })
    }

    /// Create a new S3 configuration with explicit values
    pub fn new(
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        path_style: bool,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            region: region.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            path_style,
            prefix: None,
        }
    }

    /// Set the key prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}
