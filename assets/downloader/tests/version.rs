use downloader::version::fetch_version;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn fetch_version_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resVersion": "24-12-01-10-00-00-abcdef",
            "clientVersion": "2.3.81"
        })))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/version", server.uri());
    let ver = fetch_version(&client, &url).await.unwrap();

    assert_eq!(ver.res_version, "24-12-01-10-00-00-abcdef");
    assert_eq!(ver.client_version, "2.3.81");
}

#[tokio::test]
async fn fetch_version_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/version", server.uri());
    let result = fetch_version(&client, &url).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn fetch_version_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = reqwest::Client::new();
    let url = format!("{}/version", server.uri());
    let result = fetch_version(&client, &url).await;

    assert!(result.is_err());
}
