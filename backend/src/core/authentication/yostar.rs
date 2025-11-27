use crate::core::authentication::{FetchError, constants::Server, fetch_url};
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendCodeBody<'a> {
    account: &'a str,
    randstr: &'a str,
    ticket: &'a str,
}

pub async fn send_code(
    client: &Client,
    email: &str,
    server: Server,
) -> Result<serde_json::Value, FetchError> {
    let body = SendCodeBody {
        account: email,
        randstr: "",
        ticket: "",
    };

    let base_url = server.yostar_domain().expect("Server not supported");

    let response = fetch_url(
        client,
        base_url,
        Some("yostar/send-code"),
        Some(serde_json::to_value(&body).unwrap()),
        None,
    )
    .await?;

    let data: serde_json::Value = response.json().await.map_err(FetchError::RequestFailed)?;
    Ok(data)
}
