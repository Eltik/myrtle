//! S3 Client for Asset Serving

use s3::creds::Credentials;
use s3::{Bucket, Region};

use super::S3Config;

/// S3 client wrapper for serving game assets
pub struct S3AssetClient {
    bucket: Box<Bucket>,
    config: S3Config,
}

impl S3AssetClient {
    /// Create a new S3 client from configuration
    pub fn new(config: S3Config) -> Result<Self, String> {
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| format!("Failed to create S3 credentials: {}", e))?;

        let region = Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.clone(),
        };

        let mut bucket = Bucket::new(&config.bucket, region, credentials)
            .map_err(|e| format!("Failed to create S3 bucket: {}", e))?;

        if config.path_style {
            bucket = bucket.with_path_style();
        }

        Ok(Self {
            bucket: Box::new(bucket),
            config,
        })
    }

    /// Create a new S3 client from environment variables
    pub fn from_env() -> Result<Self, String> {
        Self::new(S3Config::from_env()?)
    }

    /// Get the bucket reference
    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }

    /// Get the configuration
    pub fn config(&self) -> &S3Config {
        &self.config
    }

    /// Build the full key with optional prefix
    fn build_key(&self, key: &str) -> String {
        match &self.config.prefix {
            Some(prefix) => format!(
                "{}/{}",
                prefix.trim_end_matches('/'),
                key.trim_start_matches('/')
            ),
            None => key.to_string(),
        }
    }

    /// Check if an object exists in the bucket
    pub async fn object_exists(&self, key: &str) -> Result<bool, String> {
        let full_key = self.build_key(key);
        match self.bucket.head_object(&full_key).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("404")
                    || err_str.contains("NoSuchKey")
                    || err_str.contains("Not Found")
                {
                    Ok(false)
                } else {
                    Err(format!("Failed to check existence of {}: {}", full_key, e))
                }
            }
        }
    }

    /// Download an object to memory
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, String> {
        let full_key = self.build_key(key);
        let response = self
            .bucket
            .get_object(&full_key)
            .await
            .map_err(|e| format!("Failed to download {}: {}", full_key, e))?;

        Ok(response.bytes().to_vec())
    }

    /// Download an object with ETag and content type
    pub async fn get_object_with_etag(
        &self,
        key: &str,
    ) -> Result<(Vec<u8>, Option<String>, Option<String>), String> {
        let full_key = self.build_key(key);
        let response = self
            .bucket
            .get_object(&full_key)
            .await
            .map_err(|e| format!("Failed to download {}: {}", full_key, e))?;

        // Extract ETag from headers
        let etag = response.headers().get("etag").map(|s| s.to_string());

        // Extract content type from headers
        let content_type = response
            .headers()
            .get("content-type")
            .map(|s| s.to_string());

        Ok((response.bytes().to_vec(), etag, content_type))
    }

    /// List objects with optional prefix
    pub async fn list_objects(&self, prefix: Option<&str>) -> Result<Vec<String>, String> {
        let full_prefix = match (prefix, &self.config.prefix) {
            (Some(p), Some(base)) => format!(
                "{}/{}",
                base.trim_end_matches('/'),
                p.trim_start_matches('/')
            ),
            (Some(p), None) => p.to_string(),
            (None, Some(base)) => base.clone(),
            (None, None) => String::new(),
        };

        let result = self
            .bucket
            .list(full_prefix, None)
            .await
            .map_err(|e| format!("Failed to list S3 objects: {}", e))?;

        let mut keys = Vec::new();
        for list_result in &result {
            for object in &list_result.contents {
                // Strip the prefix from the returned keys
                let key = match &self.config.prefix {
                    Some(p) => object
                        .key
                        .strip_prefix(&format!("{}/", p.trim_end_matches('/')))
                        .unwrap_or(&object.key)
                        .to_string(),
                    None => object.key.clone(),
                };
                keys.push(key);
            }
        }

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_key_without_prefix() {
        let config = S3Config::new(
            "http://localhost:9000",
            "test-bucket",
            "us-east-1",
            "test",
            "test",
            true,
        );
        let client = S3AssetClient::new(config).unwrap();

        assert_eq!(client.build_key("path/to/file.png"), "path/to/file.png");
        assert_eq!(client.build_key("/path/to/file.png"), "/path/to/file.png");
    }

    #[test]
    fn test_build_key_with_prefix() {
        let config = S3Config::new(
            "http://localhost:9000",
            "test-bucket",
            "us-east-1",
            "test",
            "test",
            true,
        )
        .with_prefix("assets/en");
        let client = S3AssetClient::new(config).unwrap();

        assert_eq!(
            client.build_key("path/to/file.png"),
            "assets/en/path/to/file.png"
        );
        assert_eq!(
            client.build_key("/path/to/file.png"),
            "assets/en/path/to/file.png"
        );
    }
}
