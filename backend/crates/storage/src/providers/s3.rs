//! Cloudflare R2 over its S3 API

use std::collections::HashMap;
use std::error;
use std::io::{self, SeekFrom};
use std::iter::{empty, once};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use futures::stream::{self, BoxStream, iter, try_unfold};
use futures::{StreamExt, TryStreamExt};
use jiff::Timestamp;
use reqwest::header::{
    CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, HeaderMap, HeaderName,
    IF_NONE_MATCH, RANGE,
};
use reqwest::{Body, Client, Error, RequestBuilder, Response, StatusCode};
use rustls::ClientConfig;
use rustls::crypto::ring::default_provider;
use rusty_s3::actions::{
    AbortMultipartUpload, CompleteMultipartUpload, CreateMultipartUpload, DeleteObjects,
    DeleteObjectsResponse, GetObject, HeadObject, ListObjectsV2, ObjectIdentifier,
    PutObject as PutObjectAction, UploadPart,
};
use rusty_s3::signing::sign;
use rusty_s3::{Bucket, Credentials, Method, S3Action, UrlStyle};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::time::sleep;
use tokio_util::io::ReaderStream;
use tracing::warn;

use crate::config::{StorageConfig, StorageKind};
use crate::store::{
    GetOptions, GetResult, ObjectMeta, ObjectStore, PutBody, PutObject, StorageError, unquote_etag,
};

/// `DeleteObjects` takes at most 1000 keys per request.
const DELETE_BATCH: usize = 1000;
/// How long browsers may cache a CORS preflight: 86400 s, one day.
const CORS_MAX_AGE_SECONDS: u32 = 86_400;
/// Lifetime of a presigned request
const SIGN_TTL: Duration = Duration::from_secs(3600);
/// R2's smallest part
const MIN_PART_SIZE: u64 = 5 * 1024 * 1024;
const CONTENT_MD5: HeaderName = HeaderName::from_static("content-md5");
/// Attempts for a bodiless read (HEAD, GET, a list page)
const READ_ATTEMPTS: u32 = 3;
/// First read retry waits 200 ms, doubling each time
const READ_RETRY_BASE: Duration = Duration::from_millis(200);

pub struct S3Store {
    http: Client,
    bucket: Bucket,
    credentials: Credentials,
    kind: StorageKind,
}

fn http_client() -> Result<Client, StorageError> {
    use rustls_platform_verifier::BuilderVerifierExt;
    let tls_error = |e: rustls::Error| StorageError::Config(format!("TLS setup: {e}"));
    let tls = ClientConfig::builder_with_provider(Arc::new(default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(tls_error)?
        .with_platform_verifier()
        .map_err(tls_error)?
        .with_no_client_auth();
    Client::builder()
        .tls_backend_preconfigured(tls)
        .build()
        .map_err(|e| StorageError::Config(format!("HTTP client: {e}")))
}

fn transport_error(err: &Error) -> StorageError {
    StorageError::Request(format!("{err}: {:?}", error::Error::source(err)))
}

/// The text of the first `<tag>` element in `xml`
fn xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find(&format!("</{tag}>"))?;
    Some(
        xml[start..end]
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&"),
    )
}

/// `Code: Message` from an S3 error body, else HTTP status
fn error_detail(status: StatusCode, body: &str) -> StorageError {
    StorageError::Request(match xml_text(body, "Code") {
        Some(code) => format!("{code}: {}", xml_text(body, "Message").unwrap_or_default()),
        None => format!("HTTP {status}"),
    })
}

/// The response when it succeeded, else its error body as a [`StorageError`]
async fn ok_or_error(resp: Response) -> Result<Response, StorageError> {
    if resp.status().is_success() {
        return Ok(resp);
    }
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    Err(error_detail(status, &body))
}

async fn ok_text(resp: Response) -> Result<String, StorageError> {
    ok_or_error(resp)
        .await?
        .text()
        .await
        .map_err(|e| transport_error(&e))
}

/// Sends the request `build` makes, again after a transport error, a 5xx or
/// a 429, up to [`READ_ATTEMPTS`] times. Last answer is returned as is.
async fn send_read(build: impl Fn() -> RequestBuilder) -> Result<Response, StorageError> {
    let mut attempt = 1;
    loop {
        let sent = build().send().await;
        let retry = match &sent {
            Ok(resp) => {
                resp.status().is_server_error() || resp.status() == StatusCode::TOO_MANY_REQUESTS
            }
            Err(e) => !e.is_builder(),
        };
        if !retry || attempt == READ_ATTEMPTS {
            return sent.map_err(|e| transport_error(&e));
        }
        sleep(READ_RETRY_BASE * 2u32.pow(attempt - 1)).await;
        attempt += 1;
    }
}

fn header(headers: &HeaderMap, name: HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

fn content_length(headers: &HeaderMap) -> Option<u64> {
    header(headers, CONTENT_LENGTH).and_then(|v| v.parse().ok())
}

/// `len` bytes of the file at `path` from `offset`
async fn file_body(path: &Path, offset: u64, len: u64) -> Result<Body, StorageError> {
    let mut file = File::open(path).await?;
    if offset > 0 {
        file.seek(SeekFrom::Start(offset)).await?;
    }
    Ok(Body::wrap_stream(ReaderStream::new(file.take(len))))
}

/// The `PutBucketCors` body: browser GET and HEAD from `origins`
fn cors_xml(origins: &[String]) -> String {
    let origins: String = origins
        .iter()
        .map(|o| {
            let o = o
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            format!("<AllowedOrigin>{o}</AllowedOrigin>")
        })
        .collect();
    format!(
        "<CORSConfiguration><CORSRule>{origins}\
         <AllowedMethod>GET</AllowedMethod><AllowedMethod>HEAD</AllowedMethod>\
         <AllowedHeader>*</AllowedHeader>\
         <ExposeHeader>ETag</ExposeHeader><ExposeHeader>Content-Range</ExposeHeader>\
         <ExposeHeader>Content-Length</ExposeHeader><ExposeHeader>Accept-Ranges</ExposeHeader>\
         <MaxAgeSeconds>{CORS_MAX_AGE_SECONDS}</MaxAgeSeconds>\
         </CORSRule></CORSConfiguration>"
    )
}

/// Base64 of the body's raw MD5, for `Content-MD5`
fn md5_base64(body: &[u8]) -> String {
    use base64::Engine;
    use md5::{Digest, Md5};
    STANDARD.encode(Md5::digest(body))
}

impl S3Store {
    /// # Errors
    /// A config without an endpoint or both credentials
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let (Some(endpoint), Some(id), Some(secret)) = (
            &config.endpoint,
            &config.access_key_id,
            &config.secret_access_key,
        ) else {
            return Err(StorageError::Config(
                "R2 needs an endpoint, an access key id and a secret access key".into(),
            ));
        };
        let url = endpoint
            .parse()
            .map_err(|e| StorageError::Config(format!("CLOUDFLARE_S3_API {endpoint:?}: {e}")))?;
        let bucket = Bucket::new(url, UrlStyle::Path, config.bucket.clone(), "auto")
            .map_err(|e| StorageError::Config(format!("bucket {:?}: {e}", config.bucket)))?;
        Ok(Self {
            http: http_client()?,
            bucket,
            credentials: Credentials::new(id.clone(), secret.clone()),
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
    ) -> Result<Vec<String>, StorageError> {
        let part_size = part_size.max(MIN_PART_SIZE);
        let mut etags = Vec::new();
        let mut offset = 0u64;
        let mut number = 1u16;
        while offset < size {
            let len = part_size.min(size - offset);
            let url = UploadPart::new(
                &self.bucket,
                Some(&self.credentials),
                key,
                number,
                upload_id,
            )
            .sign(SIGN_TTL);
            let resp = self
                .http
                .put(url)
                .header(CONTENT_LENGTH, len)
                .body(file_body(path, offset, len).await?)
                .send()
                .await
                .map_err(|e| transport_error(&e))?;
            let resp = ok_or_error(resp).await?;
            etags.push(header(resp.headers(), ETAG).unwrap_or_default());
            offset += len;
            number = number
                .checked_add(1)
                .ok_or_else(|| StorageError::Request(format!("{key}: more than 65535 parts")))?;
        }
        Ok(etags)
    }

    /// Completes the upload
    async fn complete(
        &self,
        key: &str,
        upload_id: &str,
        etags: &[String],
    ) -> Result<String, StorageError> {
        let action = CompleteMultipartUpload::new(
            &self.bucket,
            Some(&self.credentials),
            key,
            upload_id,
            etags.iter().map(String::as_str),
        );
        let url = action.sign(SIGN_TTL);
        let resp = self
            .http
            .post(url)
            .body(action.body())
            .send()
            .await
            .map_err(|e| transport_error(&e))?;
        let status = resp.status();
        let text = ok_text(resp).await?;
        if text.contains("<Error>") {
            return Err(error_detail(status, &text));
        }
        Ok(xml_text(&text, "ETag")
            .as_deref()
            .map(unquote_etag)
            .unwrap_or_default())
    }

    async fn abort(&self, key: &str, upload_id: &str) -> Result<(), StorageError> {
        let url = AbortMultipartUpload::new(&self.bucket, Some(&self.credentials), key, upload_id)
            .sign(SIGN_TTL);
        let resp = self
            .http
            .delete(url)
            .send()
            .await
            .map_err(|e| transport_error(&e))?;
        ok_or_error(resp).await.map(drop)
    }

    /// Keys R2 refused
    async fn delete_batch(&self, batch: &[String]) -> Result<Vec<(String, String)>, StorageError> {
        let ids: Vec<ObjectIdentifier> = batch
            .iter()
            .map(|k| ObjectIdentifier::new(k.clone()))
            .collect();
        let mut action = DeleteObjects::new(&self.bucket, Some(&self.credentials), ids.iter());
        // Quiet mode lists only the failures.
        action.set_quiet(true);
        let url = action.sign(SIGN_TTL);
        let (body, md5) = action.body_with_md5();
        let resp = self
            .http
            .post(url)
            .header(CONTENT_MD5, md5)
            .body(body)
            .send()
            .await
            .map_err(|e| transport_error(&e))?;
        let text = ok_text(resp).await?;
        let parsed = DeleteObjectsResponse::parse(&text)
            .map_err(|e| StorageError::Request(format!("DeleteObjects response: {e}")))?;
        Ok(parsed
            .errors
            .into_iter()
            .map(|e| (e.key, format!("{}: {}", e.code, e.message)))
            .collect())
    }
}

#[async_trait]
impl ObjectStore for S3Store {
    fn kind(&self) -> StorageKind {
        self.kind
    }

    async fn head(&self, key: &str) -> Result<Option<ObjectMeta>, StorageError> {
        let url = HeadObject::new(&self.bucket, Some(&self.credentials), key).sign(SIGN_TTL);
        let resp = send_read(|| self.http.head(url.clone())).await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let resp = ok_or_error(resp).await?;
        let headers = resp.headers();
        Ok(Some(ObjectMeta {
            key: key.to_owned(),
            size: content_length(headers).unwrap_or(0),
            etag: header(headers, ETAG).as_deref().map(unquote_etag),
            content_type: header(headers, CONTENT_TYPE),
        }))
    }

    async fn get(&self, key: &str, opts: GetOptions) -> Result<GetResult, StorageError> {
        let url = GetObject::new(&self.bucket, Some(&self.credentials), key).sign(SIGN_TTL);
        let resp = send_read(|| {
            let mut req = self.http.get(url.clone());
            if let Some(range) = &opts.range {
                req = req.header(RANGE, range);
            }
            if let Some(tag) = &opts.if_none_match {
                req = req.header(IF_NONE_MATCH, tag);
            }
            req
        })
        .await?;
        let status = resp.status().as_u16();
        match status {
            404 => Err(StorageError::NotFound(key.to_owned())),
            200 | 206 | 304 | 416 => {
                let has_body = matches!(status, 200 | 206);
                let headers = resp.headers();
                let etag = header(headers, ETAG);
                let content_range = header(headers, CONTENT_RANGE);
                let (content_type, content_length) = if has_body {
                    (header(headers, CONTENT_TYPE), content_length(headers))
                } else {
                    (None, None)
                };
                let body = if has_body {
                    resp.bytes_stream().map_err(io::Error::other).boxed()
                } else {
                    stream::empty().boxed()
                };
                Ok(GetResult {
                    status,
                    etag,
                    content_type,
                    content_length,
                    content_range,
                    body,
                })
            }
            _ => match ok_or_error(resp).await {
                Err(e) => Err(e),
                Ok(_) => Err(StorageError::Request(format!(
                    "GET {key}: unexpected HTTP {status}"
                ))),
            },
        }
    }

    async fn put(&self, key: &str, object: PutObject) -> Result<String, StorageError> {
        let (body, len) = match object.body {
            PutBody::Bytes(b) => {
                let len = b.len() as u64;
                (Body::from(b), len)
            }
            PutBody::File { path, size } => (file_body(&path, 0, size).await?, size),
        };
        let url = PutObjectAction::new(&self.bucket, Some(&self.credentials), key).sign(SIGN_TTL);
        let mut req = self
            .http
            .put(url)
            .header(CONTENT_TYPE, object.content_type)
            .header(CONTENT_LENGTH, len)
            .body(body);
        if let Some(cache) = object.cache_control {
            req = req.header(CACHE_CONTROL, cache);
        }
        if let Some(md5) = object.content_md5 {
            req = req.header(CONTENT_MD5, md5);
        }
        let resp = req.send().await.map_err(|e| transport_error(&e))?;
        let resp = ok_or_error(resp).await?;
        Ok(header(resp.headers(), ETAG)
            .as_deref()
            .map(unquote_etag)
            .unwrap_or_default())
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
        let url =
            CreateMultipartUpload::new(&self.bucket, Some(&self.credentials), key).sign(SIGN_TTL);
        let mut req = self.http.post(url).header(CONTENT_TYPE, content_type);
        if let Some(cache) = cache_control {
            req = req.header(CACHE_CONTROL, cache);
        }
        let text = ok_text(req.send().await.map_err(|e| transport_error(&e))?).await?;
        let created = CreateMultipartUpload::parse_response(&text)
            .map_err(|e| StorageError::Request(format!("CreateMultipartUpload response: {e}")))?;
        let upload_id = created.upload_id();

        let completed = match self
            .upload_parts(key, upload_id, path, size, part_size)
            .await
        {
            Ok(etags) => self.complete(key, upload_id, &etags).await,
            Err(e) => Err(e),
        };
        if completed.is_err()
            && let Err(abort) = self.abort(key, upload_id).await
        {
            warn!(key, error = %abort, "abort of a failed multipart upload failed");
        }
        completed
    }

    async fn delete_many(&self, keys: &[String]) -> Vec<(String, Result<(), StorageError>)> {
        let mut out = Vec::with_capacity(keys.len());
        for batch in keys.chunks(DELETE_BATCH) {
            match self.delete_batch(batch).await {
                Ok(errors) => {
                    let failed: HashMap<String, String> = errors.into_iter().collect();
                    out.extend(batch.iter().map(|k| {
                        let r = failed
                            .get(k)
                            .map_or(Ok(()), |m| Err(StorageError::Request(m.clone())));
                        (k.clone(), r)
                    }));
                }
                Err(e) => {
                    let msg = e.to_string();
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
        let pages = try_unfold(Some(None::<String>), move |state| async move {
            let Some(token) = state else {
                return Ok(None);
            };
            let mut action = ListObjectsV2::new(&self.bucket, Some(&self.credentials));
            action.with_prefix(prefix);
            if let Some(token) = token {
                action.with_continuation_token(token);
            }
            let url = action.sign(SIGN_TTL);
            let resp = send_read(|| self.http.get(url.clone())).await?;
            let text = ok_text(resp).await?;
            let out = ListObjectsV2::parse_response(&text)
                .map_err(|e| StorageError::Request(format!("ListObjectsV2 response: {e}")))?;
            let items: Vec<ObjectMeta> = out
                .contents
                .into_iter()
                .map(|o| ObjectMeta {
                    etag: Some(unquote_etag(&o.etag)).filter(|e| !e.is_empty()),
                    key: o.key,
                    size: o.size,
                    content_type: None,
                })
                .collect();
            Ok::<_, StorageError>(Some((items, out.next_continuation_token.map(Some))))
        });
        pages
            .map(|page| match page {
                Ok(items) => iter(items.into_iter().map(Ok)).boxed(),
                Err(e) => iter([Err(e)]).boxed(),
            })
            .flatten()
            .boxed()
    }

    async fn put_bucket_cors(&self, origins: &[String]) -> Result<(), StorageError> {
        let url = sign(
            &Timestamp::now(),
            Method::Put,
            self.bucket.base_url().clone(),
            self.credentials.key(),
            self.credentials.secret(),
            self.credentials.token(),
            self.bucket.region(),
            SIGN_TTL.as_secs(),
            once(("cors", "")),
            empty(),
        );
        let body = cors_xml(origins);
        let resp = self
            .http
            .put(url)
            .header(CONTENT_MD5, md5_base64(body.as_bytes()))
            .header(CONTENT_TYPE, "application/xml")
            .body(body)
            .send()
            .await
            .map_err(|e| transport_error(&e))?;
        ok_or_error(resp).await.map(drop)
    }
}
