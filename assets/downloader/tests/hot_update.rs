use downloader::hot_update::fetch_hot_update_list;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn hot_update_json(
    pack_infos: serde_json::Value,
    ab_infos: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "packInfos": pack_infos,
        "abInfos": ab_infos
    })
}

#[tokio::test]
async fn groups_by_pack_id() {
    let server = MockServer::start().await;

    let body = hot_update_json(
        serde_json::json!([
            { "name": "arts_chararts" }
        ]),
        serde_json::json!([
            { "name": "char_002_amiya.ab", "totalSize": 1024, "md5": "aaa", "pid": "arts_chararts" },
            { "name": "char_003_kalts.ab", "totalSize": 2048, "md5": "bbb", "pid": "arts_chararts" }
        ]),
    );

    Mock::given(method("GET"))
        .and(path("/v1/hot_update_list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let groups = fetch_hot_update_list(&client, &server.uri(), "v1")
        .await
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "arts/chararts");
    assert_eq!(groups[0].files.len(), 2);
    assert_eq!(groups[0].total_size, 3072);
}

#[tokio::test]
async fn orphan_files_go_to_other() {
    let server = MockServer::start().await;

    let body = hot_update_json(
        serde_json::json!([
            { "name": "arts_chararts" }
        ]),
        serde_json::json!([
            { "name": "grouped.ab", "totalSize": 100, "md5": "aaa", "pid": "arts_chararts" },
            { "name": "orphan.ab", "totalSize": 200, "md5": "bbb", "pid": null },
            { "name": "unknown_pack.ab", "totalSize": 300, "md5": "ccc", "pid": "nonexistent_pack" }
        ]),
    );

    Mock::given(method("GET"))
        .and(path("/v1/hot_update_list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let groups = fetch_hot_update_list(&client, &server.uri(), "v1")
        .await
        .unwrap();

    let other = groups.iter().find(|g| g.name == "other");
    assert!(other.is_some(), "should have 'other' group");
    let other = other.unwrap();
    assert_eq!(other.files.len(), 2);
    assert_eq!(other.total_size, 500);
}

#[tokio::test]
async fn sorted_by_total_size_desc() {
    let server = MockServer::start().await;

    let body = hot_update_json(
        serde_json::json!([
            { "name": "small_pack" },
            { "name": "large_pack" },
            { "name": "medium_pack" }
        ]),
        serde_json::json!([
            { "name": "s.ab", "totalSize": 100, "md5": "a", "pid": "small_pack" },
            { "name": "l.ab", "totalSize": 9000, "md5": "b", "pid": "large_pack" },
            { "name": "m.ab", "totalSize": 500, "md5": "c", "pid": "medium_pack" }
        ]),
    );

    Mock::given(method("GET"))
        .and(path("/v1/hot_update_list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let groups = fetch_hot_update_list(&client, &server.uri(), "v1")
        .await
        .unwrap();

    assert_eq!(groups.len(), 3);
    assert!(
        groups[0].total_size >= groups[1].total_size,
        "groups should be sorted descending"
    );
    assert!(
        groups[1].total_size >= groups[2].total_size,
        "groups should be sorted descending"
    );
    assert_eq!(groups[0].name, "large/pack");
}

#[tokio::test]
async fn underscore_to_slash_in_names() {
    let server = MockServer::start().await;

    let body = hot_update_json(
        serde_json::json!([
            { "name": "gamedata_excel" }
        ]),
        serde_json::json!([
            { "name": "table.ab", "totalSize": 100, "md5": "a", "pid": "gamedata_excel" }
        ]),
    );

    Mock::given(method("GET"))
        .and(path("/v1/hot_update_list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let groups = fetch_hot_update_list(&client, &server.uri(), "v1")
        .await
        .unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "gamedata/excel");
}

#[tokio::test]
async fn empty_response() {
    let server = MockServer::start().await;

    let body = hot_update_json(serde_json::json!([]), serde_json::json!([]));

    Mock::given(method("GET"))
        .and(path("/v1/hot_update_list.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let groups = fetch_hot_update_list(&client, &server.uri(), "v1")
        .await
        .unwrap();

    assert!(groups.is_empty());
}
