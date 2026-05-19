use std::io::{Cursor, Read};

use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::core::hypergryph::{
    constants::{AuthSession, Server},
    fetch::{FetchError, FetchRequest, auth_request, fetch, parse_json, read_body},
};

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
    let base_url = server.yostar_domain().ok_or(FetchError::ParseError(
        "Server not supported for Yostar".into(),
    ))?;

    let body = serde_json::to_value(SendCodeBody {
        account: email,
        randstr: "",
        ticket: "",
    })
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let response = fetch(
        client,
        base_url,
        FetchRequest {
            endpoint: Some("yostar/send-code"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
        },
    )
    .await?;

    parse_json(response, "yostar::send_code").await
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AuthBody<'a> {
    account: &'a str,
    code: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchResponse {
    code: i64,
    data: Option<AuthFetchData>,
    msg: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchData {
    token: String,
}

pub struct AuthResponse {
    pub token: String,
}

pub async fn submit_auth(
    client: &Client,
    email: &str,
    code: &str,
    server: Server,
) -> Result<AuthResponse, FetchError> {
    let base_url = server.yostar_domain().ok_or(FetchError::ParseError(
        "Server not supported for Yostar".into(),
    ))?;

    let body = serde_json::to_value(AuthBody {
        account: email,
        code,
    })
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let response = fetch(
        client,
        base_url,
        FetchRequest {
            endpoint: Some("yostar/get-auth"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
        },
    )
    .await?;

    let data: AuthFetchResponse = parse_json(response, "yostar::submit_auth").await?;

    if data.code != 200 {
        return Err(FetchError::ParseError(format!(
            "Auth failed: {} (code: {})",
            data.msg, data.code
        )));
    }

    let inner = data
        .data
        .ok_or(FetchError::ParseError("Missing auth data".into()))?;

    Ok(AuthResponse { token: inner.token })
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct TokenBody<'a> {
    check_account: i32,
    geetest: Geetest,
    #[serde(rename = "OpenID")]
    open_id: &'a str,
    secret: &'a str,
    token: &'a str,
    #[serde(rename = "Type")]
    type_field: &'a str,
    user_name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Default)]
struct Geetest {
    captcha_id: Option<String>,
    captcha_output: Option<String>,
    gen_time: Option<String>,
    lot_number: Option<String>,
    pass_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenFetchResponse {
    data: TokenData,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenData {
    user_info: TokenUserInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TokenUserInfo {
    #[serde(rename = "ID")]
    id: String,
    token: String,
}

pub struct TokenResponse {
    pub uid: String,
    pub token: String,
}

pub async fn request_token(
    client: &Client,
    email: &str,
    email_token: &str,
    server: Server,
) -> Result<TokenResponse, FetchError> {
    let base_url = server.yostar_domain().ok_or(FetchError::ParseError(
        "Server not supported for Yostar".into(),
    ))?;

    let body = serde_json::to_value(TokenBody {
        check_account: 0,
        geetest: Geetest::default(),
        open_id: email,
        secret: "",
        token: email_token,
        type_field: "yostar",
        user_name: email,
    })
    .map_err(|e| FetchError::ParseError(e.to_string()))?;

    let response = fetch(
        client,
        base_url,
        FetchRequest {
            endpoint: Some("user/login"),
            body: Some(&body),
            session: None,
            server,
            sign: true,
        },
    )
    .await?;

    let data: TokenFetchResponse = parse_json(response, "yostar::request_token").await?;

    Ok(TokenResponse {
        uid: data.data.user_info.id,
        token: data.data.user_info.token,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortalLoginBody<'a> {
    channel: &'a str,
    token: &'a str,
    #[serde(rename = "openId")]
    open_id: &'a str,
    account: &'a str,
    check_account: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountPortalSession {
    pub yssid: String,
    pub yssid_sig: String,
}

pub async fn account_portal_login(
    client: &Client,
    email: &str,
    auth_token: &str,
) -> Result<AccountPortalSession, FetchError> {
    let body = PortalLoginBody {
        channel: "yostar",
        token: auth_token,
        open_id: email,
        account: email,
        check_account: false,
    };

    let response = client
        .post("https://account.yo-star.com/api/user/login")
        .header("Content-Type", "application/json")
        .header("Lang", "en")
        .header("Accept", "application/json, text/plain, */*")
        .header("Origin", "https://account.yo-star.com")
        .header("Referer", "https://account.yo-star.com/login")
        .json(&body)
        .send()
        .await
        .map_err(FetchError::RequestFailed)?;

    let mut yssid = None;
    let mut yssid_sig = None;

    for value in response.headers().get_all("set-cookie") {
        let cookie = value.to_str().unwrap_or("");
        if let Some(val) = cookie.strip_prefix("YSSID=") {
            if !cookie.starts_with("YSSID.sig=") {
                yssid = val.split(';').next().map(str::to_owned);
            }
        } else if let Some(val) = cookie.strip_prefix("YSSID.sig=") {
            yssid_sig = val.split(';').next().map(str::to_owned);
        }
    }

    let _ = response.text().await;

    match (yssid, yssid_sig) {
        (Some(yssid), Some(yssid_sig)) => Ok(AccountPortalSession { yssid, yssid_sig }),
        _ => Err(FetchError::ParseError("Missing YSSID cookies".into())),
    }
}

// The Arknights server is inconsistent: bools come through as `true`/`false` or `0`/`1`,
// and integers sometimes ride on the wire as decimal strings.
fn de_bool_loose<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    use serde::de::Error;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::Number(n) => Ok(n.as_i64().unwrap_or(0) != 0),
        v => Err(D::Error::custom(format!("expected bool or 0/1, got {v}"))),
    }
}

fn de_i64_loose<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    use serde::de::Error;
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| D::Error::custom(format!("expected i64, got {n}"))),
        serde_json::Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| D::Error::custom(format!("expected i64 string, got {s:?}"))),
        v => Err(D::Error::custom(format!("expected i64 or string, got {v}"))),
    }
}

fn de_unix_secs<'de, D: serde::Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
    use serde::de::Error;
    let secs = de_i64_loose(d)?;
    DateTime::from_timestamp(secs, 0)
        .ok_or_else(|| D::Error::custom(format!("invalid unix timestamp: {secs}")))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleReplay {
    #[serde(deserialize_with = "de_bool_loose")]
    pub campaign_only_version: bool,
    #[serde(deserialize_with = "de_unix_secs")]
    pub timestamp: DateTime<Utc>,
    pub journal: Journal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Journal {
    pub metadata: Metadata,
    pub squad: Vec<Character>,
    pub logs: Vec<LogEntry>,
    pub random_seed: i64,
    pub rune_list: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub standard_play_time: f64,
    #[serde(deserialize_with = "de_bool_loose")]
    pub game_result: bool,
    pub save_time: DateTime<Utc>,
    pub remaining_cost: i32,
    pub remaining_life_point: i32,
    pub killed_enemies_cnt: i32,
    pub missed_enemies_cnt: i32,
    pub level_id: String,
    pub stage_id: String,
    pub valid_killed_enemies_cnt: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub char_inst_id: i64,
    pub skin_id: String,
    pub tmpl_id: Option<String>,
    pub skill_id: Option<String>,
    pub skill_index: i32,
    pub skill_lvl: i32,
    pub level: i32,
    pub phase: i32,
    pub potential_rank: i32,
    pub favor_battle_phase: i32,
    #[serde(deserialize_with = "de_bool_loose")]
    pub is_assist_char: bool,
    pub uniequip_id: String,
    pub uniequip_level: i32,
}

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub timestamp: f64,
    #[serde(rename = "signiture")]
    pub signature: Signature,
    pub op: Op,
    pub direction: Direction,
    pub pos: Pos,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub unique_id: i64,
    pub char_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Pos {
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u8")]
pub enum Op {
    Deploy,
    Retreat,
    Skill,
}

impl TryFrom<u8> for Op {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Op::Deploy),
            1 => Ok(Op::Retreat),
            2 => Ok(Op::Skill),
            _ => Err(format!("unknown op code: {v}")),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Op::Deploy => "deploy",
            Op::Retreat => "retreat",
            Op::Skill => "skill",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u8")]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl TryFrom<u8> for Direction {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Direction::Up),
            1 => Ok(Direction::Right),
            2 => Ok(Direction::Down),
            3 => Ok(Direction::Left),
            _ => Err(format!("unknown direction code: {v}")),
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Direction::Up => "up",
            Direction::Right => "right",
            Direction::Down => "down",
            Direction::Left => "left",
        })
    }
}

pub fn decode_battle_replay(battle_replay_b64: &str) -> Result<BattleReplay> {
    let zip_bytes = STANDARD
        .decode(battle_replay_b64)
        .context("base64 decode")?;
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes)).context("open zip")?;
    let mut entry = archive
        .by_name("default_entry")
        .context("missing default_entry in zip")?;

    let mut json_bytes = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut json_bytes)?;

    serde_json::from_slice::<BattleReplay>(&json_bytes).map_err(|e| {
        let dump = std::env::temp_dir().join("battle_replay_default_entry.json");
        let _ = std::fs::write(&dump, &json_bytes);
        anyhow::anyhow!(
            "parse default_entry: {e} (line {}, col {}); raw dumped to {}",
            e.line(),
            e.column(),
            dump.display(),
        )
    })
}

#[derive(Deserialize)]
struct GetBattleReplayResp {
    #[serde(rename = "battleReplay")]
    battle_replay: String,
}

pub async fn get_battle_replay(
    client: &Client,
    session: &mut AuthSession,
    server: Server,
    battle_type: &str, // quest or main, normally quest
    stage_id: &str,
) -> Result<BattleReplay, FetchError> {
    let endpoint = format!("{battle_type}/getBattleReplay");
    let body = serde_json::json!({ "stageId": stage_id });

    let response = auth_request(client, &endpoint, Some(&body), session, server).await?;
    let text = read_body(response, "yostar::get_battle_replay").await?;

    let data: GetBattleReplayResp = serde_json::from_str(&text).map_err(|e| {
        FetchError::ParseError(format!("parse battleReplay JSON ({e}); body={text}"))
    })?;

    decode_battle_replay(&data.battle_replay)
        .map_err(|e| FetchError::ParseError(format!("decode battle replay: {e}")))
}
