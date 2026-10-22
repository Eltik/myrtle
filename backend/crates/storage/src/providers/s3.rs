//! Cloudflare R2, Amazon S3 and generic S3-compatible servers, through the
//! AWS SDK.
//!
//! Per kind:
//! * `cloudflare`: region `auto`, checksums only when an operation requires
//!   them (R2 rejects the SDK's default aws-chunked CRC32 trailers), never an
//!   ACL (R2 has none).
//! * `aws`: SDK defaults; static credentials, or the default credential chain
//!   when none are configured.
//! * `s3`: path-style by default, checksums only when required.
//!
//! Every single-part put carries `Content-MD5`.

use std::path::Path;

use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_credential_types::provider::{
    ProvideCredentials, SharedCredentialsProvider, future as cred_future,
};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{
    BehaviorVersion, Region, RequestChecksumCalculation, ResponseChecksumValidation,
};
use aws_sdk_s3::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_s3::primitives::{ByteStream, Length};
use aws_sdk_s3::types::{
    CompletedMultipartUpload, CompletedPart, CorsConfiguration, CorsRule, Delete, ObjectIdentifier,
};
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::{self, BoxStream};

use crate::config::{StorageConfig, StorageKind};
use crate::store::{
    GetOptions, GetResult, ObjectMeta, ObjectStore, PutBody, PutObject, StorageError, unquote_etag,
};

/// `DeleteObjects` takes at most 1000 keys per request.
const DELETE_BATCH: usize = 1000;
/// How long browsers may cache a CORS preflight: 86400 s, one day.
const CORS_MAX_AGE_SECONDS: i32 = 86_400;

pub struct S3Store {
    client: Client,
    bucket: String,
    kind: StorageKind,
}

/// The AWS default credential chain, built on the first request so the store
/// itself can be constructed outside an async context.
#[derive(Debug, Default)]
struct LazyDefaultChain {
    chain:
        tokio::sync::OnceCell<aws_config::default_provider::credentials::DefaultCredentialsChain>,
}

impl ProvideCredentials for LazyDefaultChain {
    fn provide_credentials<'a>(&'a self) -> cred_future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        cred_future::ProvideCredentials::new(async move {
            let chain = self
                .chain
                .get_or_init(|| async {
                    aws_config::default_provider::credentials::DefaultCredentialsChain::builder()
                        .build()
                        .await
                })
                .await;
            chain.provide_credentials().await
        })
    }
}

fn request_error<E, R>(err: &SdkError<E, R>) -> StorageError
where
    E: ProvideErrorMetadata + std::error::Error + 'static,
    R: std::fmt::Debug,
{
    let detail = match err {
        SdkError::ServiceError(se) => {
            let e = se.err();
            format!(
                "{}: {}",
                e.code().unwrap_or("service error"),
                e.message().unwrap_or("no message")
            )
        }
        other => format!("{other}: {:?}", std::error::Error::source(other)),
    };
    StorageError::Request(detail)
}

/// The HTTP status of a service error, when the request got that far.
fn status_of<E>(err: &SdkError<E, aws_sdk_s3::config::http::HttpResponse>) -> Option<u16> {
    err.raw_response().map(|r| r.status().as_u16())
}

fn header_of<E>(
    err: &SdkError<E, aws_sdk_s3::config::http::HttpResponse>,
    name: &str,
) -> Option<String> {
    err.raw_response()
        .and_then(|r| r.headers().get(name))
        .map(ToOwned::to_owned)
}

fn byte_stream_body(body: ByteStream) -> BoxStream<'static, Result<Bytes, std::io::Error>> {
    stream::unfold(body, |mut body| async move {
        match body.next().await {
            Some(Ok(chunk)) => Some((Ok(chunk), body)),
            Some(Err(e)) => Some((Err(std::io::Error::other(e)), body)),
            None => None,
        }
    })
    .boxed()
}

impl S3Store {
    /// # Errors
    /// None today; kept fallible so a provider check can refuse a config.
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let credentials = match (&config.access_key_id, &config.secret_access_key) {
            (Some(id), Some(secret)) => SharedCredentialsProvider::new(Credentials::new(
                id.clone(),
                secret.clone(),
                None,
                None,
                "myrtle-s3-env",
            )),
            _ => SharedCredentialsProvider::new(LazyDefaultChain::default()),
        };
        let mut builder = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .force_path_style(config.force_path_style);
        if let Some(endpoint) = &config.endpoint {
            builder = builder.endpoint_url(endpoint);
        }
        if matches!(config.kind, StorageKind::Cloudflare | StorageKind::S3) {
            builder = builder
                .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
                .response_checksum_validation(ResponseChecksumValidation::WhenRequired);
        }
        Ok(Self {
            client: Client::from_conf(builder.build()),
            bucket: config.bucket.clone(),
            kind: config.kind,
        })
    }

    async fn upload_parts(
        &self,
        key: &str,
        upload_id: &str,
        path: &Path,
        size: u64,
        part_size: u64,
    ) -> Result<Vec<CompletedPart>, StorageError> {
        let part_size = part_size.max(5 * 1024 * 1024);
        let mut parts = Vec::new();
        let mut offset = 0u64;
        let mut number = 1i32;
        while offset < size {
            let len = part_size.min(size - offset);
            let body = ByteStream::read_from()
                .path(path)
                .offset(offset)
                .length(Length::Exact(len))
                .build()
                .await
                .map_err(|e| StorageError::Request(e.to_string()))?;
            let out = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(key)
                .upload_id(upload_id)
                .part_number(number)
                .content_length(i64::try_from(len).unwrap_or(i64::MAX))
                .body(body)
                .send()
                .await
                .map_err(|e| request_error(&e))?;
            parts.push(
                CompletedPart::builder()
                    .part_number(number)
                    .set_e_tag(out.e_tag)
                    .build(),
            );
            offset += len;
            number += 1;
        }
        Ok(parts)
    }
}

#[async_trait]
impl ObjectStore for S3Store {
    fn kind(&self) -> StorageKind {
        self.kind
    }

    async fn head(&self, key: &str) -> Result<Option<ObjectMeta>, StorageError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(out) => Ok(Some(ObjectMeta {
                key: key.to_owned(),
                size: out
                    .content_length
                    .and_then(|n| u64::try_from(n).ok())
                    .unwrap_or(0),
                etag: out.e_tag.as_deref().map(unquote_etag),
                content_type: out.content_type,
            })),
            Err(e) if status_of(&e) == Some(404) => Ok(None),
            Err(e) => Err(request_error(&e)),
        }
    }

    async fn get(&self, key: &str, opts: GetOptions) -> Result<GetResult, StorageError> {
        let sent = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .set_range(opts.range)
            .set_if_none_match(opts.if_none_match)
            .send()
            .await;
        match sent {
            Ok(out) => Ok(GetResult {
                status: if out.content_range.is_some() {
                    206
                } else {
                    200
                },
                etag: out.e_tag,
                content_type: out.content_type,
                content_length: out.content_length.and_then(|n| u64::try_from(n).ok()),
                content_range: out.content_range,
                body: byte_stream_body(out.body),
            }),
            // The SDK surfaces both 304 and 416 as errors; they are answers.
            Err(e) => match status_of(&e) {
                Some(404) => Err(StorageError::NotFound(key.to_owned())),
                Some(status @ (304 | 416)) => Ok(GetResult {
                    status,
                    etag: header_of(&e, "etag"),
                    content_type: None,
                    content_length: None,
                    content_range: header_of(&e, "content-range"),
                    body: stream::empty().boxed(),
                }),
                _ => Err(request_error(&e)),
            },
        }
    }

    async fn put(&self, key: &str, object: PutObject) -> Result<String, StorageError> {
        let (body, len) = match object.body {
            PutBody::Bytes(b) => {
                let len = b.len() as u64;
                (ByteStream::from(b), len)
            }
            PutBody::File { path, size } => (
                ByteStream::from_path(&path)
                    .await
                    .map_err(|e| StorageError::Request(e.to_string()))?,
                size,
            ),
        };
        let out = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(object.content_type)
            .set_cache_control(object.cache_control)
            .set_content_md5(object.content_md5)
            .content_length(i64::try_from(len).unwrap_or(i64::MAX))
            .body(body)
            .send()
            .await
            .map_err(|e| request_error(&e))?;
        Ok(out.e_tag.as_deref().map(unquote_etag).unwrap_or_default())
    }

    async fn put_multipart_file(
        &self,
        key: &str,
        path: &Path,
        size: u64,
        part_size: u64,
        content_type: &str,
        cache_control: Option<&str>,
    ) -> Result<String, StorageError> {
        let created = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .set_cache_control(cache_control.map(ToOwned::to_owned))
            .send()
            .await
            .map_err(|e| request_error(&e))?;
        let upload_id = created.upload_id.ok_or_else(|| {
            StorageError::Request("CreateMultipartUpload returned no upload id".into())
        })?;

        let completed = match self
            .upload_parts(key, &upload_id, path, size, part_size)
            .await
        {
            Ok(parts) => self
                .client
                .complete_multipart_upload()
                .bucket(&self.bucket)
                .key(key)
                .upload_id(&upload_id)
                .multipart_upload(
                    CompletedMultipartUpload::builder()
                        .set_parts(Some(parts))
                        .build(),
                )
                .send()
                .await
                .map_err(|e| request_error(&e)),
            Err(e) => Err(e),
        };
        match completed {
            Ok(out) => Ok(out.e_tag.as_deref().map(unquote_etag).unwrap_or_default()),
            Err(e) => {
                // Parts of an unfinished upload are stored and billed until aborted.
                if let Err(abort) = self
                    .client
                    .abort_multipart_upload()
                    .bucket(&self.bucket)
                    .key(key)
                    .upload_id(&upload_id)
                    .send()
                    .await
                {
                    tracing::warn!(key, error = %request_error(&abort), "abort of a failed multipart upload failed");
                }
                Err(e)
            }
        }
    }

    async fn delete_many(&self, keys: &[String]) -> Vec<(String, Result<(), StorageError>)> {
        let mut out = Vec::with_capacity(keys.len());
        for batch in keys.chunks(DELETE_BATCH) {
            let ids: Result<Vec<ObjectIdentifier>, _> = batch
                .iter()
                .map(|k| ObjectIdentifier::builder().key(k).build())
                .collect();
            let delete =
                ids.and_then(|ids| Delete::builder().set_objects(Some(ids)).quiet(true).build());
            let delete = match delete {
                Ok(d) => d,
                Err(e) => {
                    let msg = e.to_string();
                    out.extend(
                        batch
                            .iter()
                            .map(|k| (k.clone(), Err(StorageError::Request(msg.clone())))),
                    );
                    continue;
                }
            };
            match self
                .client
                .delete_objects()
                .bucket(&self.bucket)
                .delete(delete)
                .send()
                .await
            {
                Ok(resp) => {
                    // Quiet mode lists only the failures.
                    let failed: std::collections::HashMap<String, String> = resp
                        .errors
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|e| {
                            let msg = format!(
                                "{}: {}",
                                e.code.as_deref().unwrap_or("error"),
                                e.message.as_deref().unwrap_or("")
                            );
                            e.key.map(|k| (k, msg))
                        })
                        .collect();
                    out.extend(batch.iter().map(|k| {
                        let r = failed
                            .get(k)
                            .map_or(Ok(()), |m| Err(StorageError::Request(m.clone())));
                        (k.clone(), r)
                    }));
                }
                Err(e) => {
                    let msg = request_error(&e).to_string();
                    out.extend(
                        batch
                            .iter()
                            .map(|k| (k.clone(), Err(StorageError::Request(msg.clone())))),
                    );
                }
            }
        }
        out
    }

    fn list<'a>(&'a self, prefix: &'a str) -> BoxStream<'a, Result<ObjectMeta, StorageError>> {
        // State: the continuation token, or `None` once the last page is in.
        let pages = stream::try_unfold(Some(None::<String>), move |state| async move {
            let Some(token) = state else {
                return Ok(None);
            };
            let out = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .set_continuation_token(token)
                .send()
                .await
                .map_err(|e| request_error(&e))?;
            let items: Vec<ObjectMeta> = out
                .contents
                .unwrap_or_default()
                .into_iter()
                .filter_map(|o| {
                    Some(ObjectMeta {
                        key: o.key?,
                        size: o.size.and_then(|n| u64::try_from(n).ok()).unwrap_or(0),
                        etag: o.e_tag.as_deref().map(unquote_etag),
                        content_type: None,
                    })
                })
                .collect();
            let next = if out.is_truncated == Some(true) {
                out.next_continuation_token.map(Some)
            } else {
                None
            };
            Ok::<_, StorageError>(Some((items, next)))
        });
        pages
            .map(|page| match page {
                Ok(items) => stream::iter(items.into_iter().map(Ok)).boxed(),
                Err(e) => stream::iter([Err(e)]).boxed(),
            })
            .flatten()
            .boxed()
    }

    async fn put_bucket_cors(&self, origins: &[String]) -> Result<(), StorageError> {
        let rule = CorsRule::builder()
            .set_allowed_origins(Some(origins.to_vec()))
            .allowed_methods("GET")
            .allowed_methods("HEAD")
            .allowed_headers("*")
            .expose_headers("ETag")
            .expose_headers("Content-Range")
            .expose_headers("Content-Length")
            .expose_headers("Accept-Ranges")
            .max_age_seconds(CORS_MAX_AGE_SECONDS)
            .build()
            .map_err(|e| StorageError::Config(e.to_string()))?;
        let cors = CorsConfiguration::builder()
            .cors_rules(rule)
            .build()
            .map_err(|e| StorageError::Config(e.to_string()))?;
        self.client
            .put_bucket_cors()
            .bucket(&self.bucket)
            .cors_configuration(cors)
            .send()
            .await
            .map_err(|e| request_error(&e))?;
        Ok(())
    }
}
