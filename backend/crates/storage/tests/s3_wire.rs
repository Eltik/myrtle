//! `S3Store` against a canned HTTP server on localhost: what goes on the
//! wire for an R2 put, and how GET statuses map back.

use std::sync::{Arc, Mutex};

use futures::StreamExt;
use myrtle_storage::{GetOptions, PutBody, PutObject, StorageConfig, StorageError, open_store};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Serves `response` to every request and records each request's head.
async fn canned(response: &'static str) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let log = Arc::clone(&log);
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let head_end = loop {
                    let n = sock.read(&mut chunk).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        break i + 4;
                    }
                };
                let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                let len = head
                    .lines()
                    .find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        k.eq_ignore_ascii_case("content-length")
                            .then(|| v.trim().parse::<usize>().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                while buf.len() < head_end + len {
                    let n = sock.read(&mut chunk).await.unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                log.lock().unwrap().push(head);
                let _ = sock.write_all(response.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    (format!("http://{addr}"), seen)
}

fn r2_config(endpoint: &str) -> StorageConfig {
    let vars = [
        ("S3_TYPE", "cloudflare"),
        ("S3_BUCKET", "myrtle"),
        ("S3_ENDPOINT", endpoint),
        ("S3_ACCESS_KEY_ID", "id"),
        ("S3_SECRET_ACCESS_KEY", "secret"),
        // Path-style keeps the bucket out of the host name, so the canned
        // server's 127.0.0.1 address resolves.
        ("S3_FORCE_PATH_STYLE", "1"),
    ];
    StorageConfig::from_lookup(&|k| {
        vars.iter()
            .find(|(name, _)| *name == k)
            .map(|(_, v)| (*v).to_owned())
    })
    .unwrap()
    .unwrap()
}

#[tokio::test]
async fn r2_put_sends_content_md5_and_no_checksum_trailer() {
    let (endpoint, seen) = canned(
        "HTTP/1.1 200 OK\r\nETag: \"5d41402abc4b2a76b9719d911017c592\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    )
    .await;
    let store = open_store(&r2_config(&endpoint)).unwrap();
    let etag = store
        .put(
            "assets/en/textures/[a].png",
            PutObject {
                body: PutBody::Bytes(bytes::Bytes::from_static(b"hello")),
                content_type: "image/png".into(),
                cache_control: Some("public, max-age=604800".into()),
                content_md5: Some("XUFAKrxLKna5cZ2REBfFkg==".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(etag, "5d41402abc4b2a76b9719d911017c592");

    let head = seen.lock().unwrap()[0].to_ascii_lowercase();
    assert!(
        head.starts_with("put /myrtle/assets/en/textures/%5ba%5d.png"),
        "{head}"
    );
    assert!(
        head.contains("content-md5: xufakrxlkna5cz2rebffkg=="),
        "{head}"
    );
    assert!(head.contains("content-type: image/png"), "{head}");
    assert!(
        head.contains("cache-control: public, max-age=604800"),
        "{head}"
    );
    assert!(!head.contains("x-amz-checksum"), "{head}");
    assert!(!head.contains("aws-chunked"), "{head}");
    assert!(!head.contains("x-amz-acl"), "{head}");
}

#[tokio::test]
async fn get_maps_200_with_body() {
    let (endpoint, seen) = canned(
        "HTTP/1.1 200 OK\r\nETag: \"abc\"\r\nContent-Type: image/png\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello",
    )
    .await;
    let store = open_store(&r2_config(&endpoint)).unwrap();
    let got = store
        .get(
            "assets/en/textures/a.png",
            GetOptions {
                range: None,
                if_none_match: Some("\"old\"".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(got.status, 200);
    assert_eq!(got.etag.as_deref(), Some("\"abc\""));
    assert_eq!(got.content_type.as_deref(), Some("image/png"));
    assert_eq!(got.content_length, Some(5));
    let body: Vec<u8> = got.body.map(|c| c.unwrap().to_vec()).concat().await;
    assert_eq!(body, b"hello");
    let head = seen.lock().unwrap()[0].to_ascii_lowercase();
    assert!(head.contains("if-none-match: \"old\""), "{head}");
}

#[tokio::test]
async fn get_maps_206_with_content_range() {
    let (endpoint, seen) = canned(
        "HTTP/1.1 206 Partial Content\r\nETag: \"abc\"\r\nContent-Range: bytes 1-2/5\r\nContent-Length: 2\r\nConnection: close\r\n\r\nel",
    )
    .await;
    let store = open_store(&r2_config(&endpoint)).unwrap();
    let got = store
        .get(
            "k.png",
            GetOptions {
                range: Some("bytes=1-2".into()),
                if_none_match: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(got.status, 206);
    assert_eq!(got.content_range.as_deref(), Some("bytes 1-2/5"));
    assert!(
        seen.lock().unwrap()[0]
            .to_ascii_lowercase()
            .contains("range: bytes=1-2")
    );
}

#[tokio::test]
async fn get_maps_304_as_an_answer() {
    let (endpoint, _) =
        canned("HTTP/1.1 304 Not Modified\r\nETag: \"abc\"\r\nConnection: close\r\n\r\n").await;
    let store = open_store(&r2_config(&endpoint)).unwrap();
    let got = store
        .get(
            "k.png",
            GetOptions {
                range: None,
                if_none_match: Some("\"abc\"".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(got.status, 304);
    assert_eq!(got.etag.as_deref(), Some("\"abc\""));
}

#[tokio::test]
async fn get_maps_404_to_not_found() {
    let (endpoint, _) = canned(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/xml\r\nContent-Length: 87\r\nConnection: close\r\n\r\n<?xml version=\"1.0\"?><Error><Code>NoSuchKey</Code><Message>gone</Message></Error>      ",
    )
    .await;
    let store = open_store(&r2_config(&endpoint)).unwrap();
    let got = store.get("k.png", GetOptions::default()).await;
    assert!(matches!(got, Err(StorageError::NotFound(_))), "{got:?}");
    assert_eq!(store.head("k.png").await.unwrap(), None);
}

#[tokio::test]
async fn an_unreachable_endpoint_is_a_request_error() {
    let store = open_store(&r2_config("http://127.0.0.1:1")).unwrap();
    let got = store.get("k.png", GetOptions::default()).await;
    assert!(matches!(got, Err(StorageError::Request(_))), "{got:?}");
}
