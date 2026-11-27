//! `LocalStore` as an `ObjectStore`: round trips, refusals and multipart ETags.

use bytes::Bytes;
use futures::StreamExt;
use futures::stream::BoxStream;
use myrtle_storage::providers::local::LocalStore;
use myrtle_storage::store::{GetOptions, ObjectStore, PutBody, PutObject, StorageError};

async fn collect(body: BoxStream<'static, Result<Bytes, std::io::Error>>) -> Vec<u8> {
    let chunks: Vec<_> = body.collect().await;
    chunks
        .into_iter()
        .flat_map(|c| c.unwrap().to_vec())
        .collect()
}

fn put_bytes(b: &'static [u8]) -> PutObject {
    PutObject {
        body: PutBody::Bytes(Bytes::from_static(b)),
        content_type: "text/plain".into(),
        cache_control: Some("public, max-age=1".into()),
        content_md5: None,
    }
}

#[tokio::test]
async fn put_get_head_list_delete_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = LocalStore::new(dir.path().to_path_buf());
    let etag = store
        .put("en/textures/a.txt", put_bytes(b"hello world"))
        .await
        .unwrap();
    assert_eq!(etag, "5eb63bbbe01eeed093cb22bb8f5acdc3");

    let head = store.head("en/textures/a.txt").await.unwrap().unwrap();
    assert_eq!(head.size, 11);
    assert_eq!(head.etag.as_deref(), Some(etag.as_str()));
    assert_eq!(head.content_type.as_deref(), Some("text/plain"));
    assert_eq!(store.head("en/textures/b.txt").await.unwrap(), None);

    let full = store
        .get("en/textures/a.txt", GetOptions::default())
        .await
        .unwrap();
    assert_eq!(full.status, 200);
    assert_eq!(full.content_length, Some(11));
    assert_eq!(collect(full.body).await, b"hello world");

    let part = store
        .get(
            "en/textures/a.txt",
            GetOptions {
                range: Some("bytes=6-".into()),
                ..GetOptions::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(part.status, 206);
    assert_eq!(part.content_range.as_deref(), Some("bytes 6-10/11"));
    assert_eq!(collect(part.body).await, b"world");

    let suffix = store
        .get(
            "en/textures/a.txt",
            GetOptions {
                range: Some("bytes=-5".into()),
                ..GetOptions::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(collect(suffix.body).await, b"world");

    let past = store
        .get(
            "en/textures/a.txt",
            GetOptions {
                range: Some("bytes=-0".into()),
                ..GetOptions::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(past.status, 416);
    assert_eq!(past.content_range.as_deref(), Some("bytes */11"));

    let cached = store
        .get(
            "en/textures/a.txt",
            GetOptions {
                if_none_match: Some(format!("\"{etag}\"")),
                ..GetOptions::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(cached.status, 304);

    assert!(matches!(
        store.get("en/nope.txt", GetOptions::default()).await,
        Err(StorageError::NotFound(_))
    ));

    let listed: Vec<_> = store.list("en/").collect().await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].as_ref().unwrap().key, "en/textures/a.txt");

    let deleted = store.delete_many(&["en/textures/a.txt".into()]).await;
    assert!(deleted[0].1.is_ok());
    assert_eq!(store.head("en/textures/a.txt").await.unwrap(), None);
    assert!(store.list("").collect::<Vec<_>>().await.is_empty());
}

#[tokio::test]
async fn bad_keys_and_wrong_md5_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let store = LocalStore::new(dir.path().to_path_buf());
    assert!(matches!(
        store.put("en/../x.txt", put_bytes(b"x")).await,
        Err(StorageError::InvalidKey(_))
    ));
    assert!(matches!(
        store.put(".meta/x.txt", put_bytes(b"x")).await,
        Err(StorageError::InvalidKey(_))
    ));
    let mut obj = put_bytes(b"x");
    obj.content_md5 = Some("AAAAAAAAAAAAAAAAAAAAAA==".into());
    assert!(store.put("en/x.txt", obj).await.is_err());
    assert_eq!(store.head("en/x.txt").await.unwrap(), None);
}

#[tokio::test]
async fn multipart_etag_has_the_part_count() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.bin");
    std::fs::write(&src, vec![7u8; 2500]).unwrap();
    let store = LocalStore::new(dir.path().join("bucket"));
    let etag = store
        .put_multipart_file("en/video/a.mp4", &src, 2500, 1000, "video/mp4", None)
        .await
        .unwrap();
    assert!(etag.ends_with("-3"), "{etag}");
    let head = store.head("en/video/a.mp4").await.unwrap().unwrap();
    assert_eq!(head.etag.as_deref(), Some(etag.as_str()));
}
