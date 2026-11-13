use std::io::{Cursor, Read};
use std::time::Duration;

use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::core::hypergryph::{
    constants::{AuthSession, Server},
    fetch::{
        FetchError, FetchRequest, REQUEST_TIMEOUT, UpstreamError, auth_request, fetch, parse_json,
        read_body, upstream_code,
    },
};
use crate::utils::redact::redacted_body;

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
            timeout: REQUEST_TIMEOUT,
        },
    )
    .await?;

    let reply: serde_json::Value = parse_json(response, "yostar::send_code").await?;
    yostar_outcome(&reply, "yostar::send_code")?;
    Ok(reply)
}

/// Whether a Yostar SDK endpoint did what it was asked.
///
/// These endpoints answer HTTP 200 whatever happened and put the outcome in the
/// body: `{"Code":200,"Data":{},"Msg":"OK"}`, or another `Code` with the reason
/// in `Msg` (too many requests, a captcha is wanted). `read_body` only looks at
/// the HTTP status, so a refusal used to pass for success: `send_code` returned
/// `Ok` for any reply that was JSON, and the site told the player a code was on
/// its way when Yostar had said it would not send one.
///
/// A refusal becomes [`FetchError::Upstream`], which reaches the client as
/// `<Msg> (<Code>)`. A reply with no `Code` at all is not a confirmed send
/// either, and is reported as an invalid upstream response.
fn yostar_outcome(reply: &serde_json::Value, context: &str) -> Result<(), FetchError> {
    match reply.get("Code").and_then(serde_json::Value::as_i64) {
        Some(200) => Ok(()),
        Some(code) => {
            let msg = reply
                .get("Msg")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            Err(yostar_refusal(code, msg, context))
        }
        None => {
            tracing::warn!(context, body = %redacted_body(&reply.to_string()), "yostar reply carries no Code");
            Err(FetchError::ParseError(format!(
                "{context}: invalid upstream response"
            )))
        }
    }
}

/// Yostar turning a login code down: wrong, expired, or already spent. Its own
/// `Msg` for this is a Chinese format string with the verb left in
/// (`获取授权信息失败,错误代码:%!d(string=3)`), so the client gets ours.
const CODE_REJECTED: i64 = 100_303;

/// The error for a Yostar reply whose `Code` is not 200, logged as it is made.
///
/// It reaches the client as `<message> (<code>)`. The message is Yostar's `Msg`,
/// except where that says nothing a player can act on.
fn yostar_refusal(code: i64, msg: &str, context: &str) -> FetchError {
    tracing::warn!(context, code, msg, "yostar refused the request");
    let shown = match code {
        CODE_REJECTED => "The code is wrong, has expired, or was already used",
        _ if msg.is_empty() => "Yostar refused the request",
        _ => msg,
    };
    FetchError::Upstream(UpstreamError {
        status_code: 200,
        code,
        error: String::new(),
        message: String::new(),
        msg: shown.to_owned(),
        info: String::new(),
    })
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
    #[serde(default)]
    msg: String,
}

/// `Token` exists ONLY on success. A refusal is `{"Code":100303,"Data":{},...}`:
/// `Data` is there and empty. Requiring `Token` made serde fail on that first,
/// so the `Code` check below never ran and every wrong code surfaced as
/// `yostar::submit_auth: invalid upstream response` (314 times between
/// 2026-10-15 and 2026-11-13) instead of what Yostar had answered.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AuthFetchData {
    #[serde(default)]
    token: Option<String>,
}

/// The email token out of a `yostar/get-auth` reply, or why there is none.
fn auth_token(reply: AuthFetchResponse) -> Result<String, FetchError> {
    if reply.code != 200 {
        return Err(yostar_refusal(
            reply.code,
            &reply.msg,
            "yostar::submit_auth",
        ));
    }
    reply
        .data
        .and_then(|data| data.token)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| FetchError::ParseError("yostar::submit_auth: no token in the reply".into()))
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
            timeout: REQUEST_TIMEOUT,
        },
    )
    .await?;

    let data: AuthFetchResponse = parse_json(response, "yostar::submit_auth").await?;

    Ok(AuthResponse {
        token: auth_token(data)?,
    })
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
            timeout: REQUEST_TIMEOUT,
        },
    )
    .await?;

    // Parsed loosely first for the same reason as `AuthFetchData`: a refusal has
    // no `UserInfo`, and reading it as the success shape would hide its `Code`.
    // Only a `Code` that is present and not 200 counts as one. This endpoint's
    // success reply has not been seen without its fields, so a reply with no
    // `Code` still goes to the typed parse below, as it always has.
    let reply: serde_json::Value = parse_json(response, "yostar::request_token").await?;
    if let Some(code) = reply.get("Code").and_then(serde_json::Value::as_i64)
        && code != 200
    {
        let msg = reply
            .get("Msg")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        return Err(yostar_refusal(code, msg, "yostar::request_token"));
    }
    let data: TokenFetchResponse = serde_json::from_value(reply).map_err(|e| {
        tracing::warn!(context = "yostar::request_token", error = %e, "failed to parse response body");
        FetchError::ParseError("yostar::request_token: invalid upstream response".into())
    })?;

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

// The server is inconsistent: bools arrive as `true`/`false` or `0`/`1`, and
// integers sometimes as decimal strings.
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
            0 => Ok(Self::Deploy),
            1 => Ok(Self::Retreat),
            2 => Ok(Self::Skill),
            _ => Err(format!("unknown op code: {v}")),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Deploy => "deploy",
            Self::Retreat => "retreat",
            Self::Skill => "skill",
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
            0 => Ok(Self::Up),
            1 => Ok(Self::Right),
            2 => Ok(Self::Down),
            3 => Ok(Self::Left),
            _ => Err(format!("unknown direction code: {v}")),
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Up => "up",
            Self::Right => "right",
            Self::Down => "down",
            Self::Left => "left",
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
    battle_type: &str, // "quest" (normal stages) or "campaignV2" (Annihilation), see saved_replay_targets
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

/// Pacing for [`harvest_replays`].
///
/// One session must call `getBattleReplay` sequentially: `seqnum` is monotonic,
/// so concurrent calls on one `AuthSession` race and invalidate it. Sessions are
/// separate rate-limit buckets, so run one task per player.
///
/// The delay is a random 80..180 ms (`min_interval`..`max_interval`). The server
/// tolerates back-to-back requests; the pause mostly keeps a runaway loop from
/// starving tokio. Set both to `Duration::ZERO` for max throughput.
///
/// Build targets from `account/syncData`, not every gamedata stage. The live
/// client only asks for stages with a saved replay; a stream of 5516 misses is
/// traffic anti-abuse can fingerprint.
#[derive(Debug, Clone)]
pub struct ReplayHarvestOptions {
    pub min_interval: Duration,
    pub max_interval: Duration,
    pub max_consecutive_errors: usize,
}

impl Default for ReplayHarvestOptions {
    fn default() -> Self {
        Self {
            min_interval: Duration::from_millis(80),
            max_interval: Duration::from_millis(180),
            max_consecutive_errors: 3,
        }
    }
}

fn jitter_delay(min: Duration, max: Duration) -> Duration {
    if max <= min {
        return min;
    }
    let lo = min.as_millis() as u64;
    let hi = max.as_millis() as u64;
    Duration::from_millis(rand::random_range(lo..=hi))
}

#[derive(Debug)]
pub enum ReplayOutcome {
    Replay(BattleReplay),
    /// Upstream 5516: no saved auto-deploy script for the stage.
    NoSaved,
}

pub async fn harvest_replays<I, F>(
    client: &Client,
    session: &mut AuthSession,
    server: Server,
    targets: I,
    opts: &ReplayHarvestOptions,
    mut on_progress: F,
) -> Result<(), FetchError>
where
    I: IntoIterator<Item = (String, String)>,
    F: FnMut(&str, &str, Result<ReplayOutcome, &FetchError>),
{
    let mut consecutive_errors = 0usize;
    let mut first = true;

    for (battle_type, stage_id) in targets {
        if !first {
            let delay = jitter_delay(opts.min_interval, opts.max_interval);
            if !delay.is_zero() {
                sleep(delay).await;
            }
        }
        first = false;

        match get_battle_replay(client, session, server, &battle_type, &stage_id).await {
            Ok(r) => {
                consecutive_errors = 0;
                on_progress(&battle_type, &stage_id, Ok(ReplayOutcome::Replay(r)));
            }
            Err(FetchError::Upstream(ref err)) if err.code == upstream_code::STAGE_NO_REPLAY => {
                consecutive_errors = 0;
                on_progress(&battle_type, &stage_id, Ok(ReplayOutcome::NoSaved));
            }
            Err(e) => {
                consecutive_errors += 1;
                on_progress(&battle_type, &stage_id, Err(&e));
                if consecutive_errors >= opts.max_consecutive_errors {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

/// The player's full state as raw JSON. Body `{"platform": 1}` matches `ArkPRTS`
/// and the roster refresh. Often megabytes for endgame accounts; sessions are
/// independent, so fan out one task per player.
pub async fn sync_data_raw(
    client: &Client,
    session: &mut AuthSession,
    server: Server,
) -> Result<String, FetchError> {
    let body = serde_json::json!({ "platform": 1 });
    let response = auth_request(client, "account/syncData", Some(&body), session, server).await?;
    read_body(response, "yostar::sync_data").await
}

/// [`sync_data_raw`] parsed to a `Value`; [`saved_replay_targets`] reads this.
pub async fn sync_data(
    client: &Client,
    session: &mut AuthSession,
    server: Server,
) -> Result<serde_json::Value, FetchError> {
    let text = sync_data_raw(client, session, server).await?;
    serde_json::from_str(&text).map_err(|e| FetchError::ParseError(format!("parse syncData ({e})")))
}

/// Every `(battle_type, stage_id)` with a saved auto-deploy replay. Only these
/// return a real replay from [`get_battle_replay`]; the rest return 5516.
///
/// Per the `OpenBachelorS` / `DoctoratePy` server reimpls:
/// - `user.dungeon.stages[*].hasBattleReplay == 1`          -> `battle_type = "quest"`
/// - `user.campaignsV2.instances[*].hasBattleReplay == 1`   -> `battle_type = "campaignV2"`
///
/// `campaignsV2` is top-level under `user`, not under `dungeon`.
pub fn saved_replay_targets(sync: &serde_json::Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    collect_replay_flags(sync.pointer("/user/dungeon/stages"), "quest", &mut out);
    collect_replay_flags(
        sync.pointer("/user/campaignsV2/instances"),
        "campaignV2",
        &mut out,
    );
    out
}

fn collect_replay_flags(
    node: Option<&serde_json::Value>,
    battle_type: &str,
    out: &mut Vec<(String, String)>,
) {
    let Some(serde_json::Value::Object(stages)) = node else {
        return;
    };
    out.extend(stages.iter().filter_map(|(stage_id, entry)| {
        let flag = entry.get("hasBattleReplay")?;
        let on = flag.as_i64().is_some_and(|n| n != 0) || flag.as_bool().unwrap_or(false);
        on.then(|| (battle_type.to_string(), stage_id.clone()))
    }));
}

/// Discover via syncData, then harvest only saved replays: no 5516 misses, and
/// the traffic matches the live client. One task per session.
pub async fn collect_player_replays<F>(
    client: &Client,
    session: &mut AuthSession,
    server: Server,
    opts: &ReplayHarvestOptions,
    on_progress: F,
) -> Result<(), FetchError>
where
    F: FnMut(&str, &str, Result<ReplayOutcome, &FetchError>),
{
    let sync = sync_data(client, session, server).await?;
    let targets = saved_replay_targets(&sync);
    tracing::info!(count = targets.len(), "discovered saved replays");
    harvest_replays(client, session, server, targets, opts, on_progress).await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AuthFetchResponse, auth_token, yostar_outcome};
    use crate::app::error::ApiError;
    use crate::core::hypergryph::fetch::FetchError;

    fn auth_reply(body: &str) -> AuthFetchResponse {
        serde_json::from_str(body).expect("the reply parses")
    }

    #[test]
    fn a_rejected_login_code_says_so() {
        // Yostar's reply to a wrong code, verbatim from the backend log.
        let reply = auth_reply(
            r#"{"Code":100303,"Data":{},"Msg":"获取授权信息失败,错误代码:%!d(string=3)"}"#,
        );
        let err = auth_token(reply).expect_err("rejected");
        let ApiError::BadRequest(shown) = ApiError::from(err) else {
            panic!("a rejected code is a 400");
        };
        assert_eq!(
            shown,
            "The code is wrong, has expired, or was already used (100303)"
        );
    }

    #[test]
    fn an_accepted_login_code_yields_its_token() {
        let reply = auth_reply(r#"{"Code":200,"Data":{"Token":"abc123"},"Msg":"OK"}"#);
        assert_eq!(auth_token(reply).expect("accepted"), "abc123");
    }

    #[test]
    fn other_refusals_keep_yostars_own_message() {
        let reply = auth_reply(r#"{"Code":100123,"Data":null,"Msg":"Account is banned"}"#);
        let ApiError::BadRequest(shown) = ApiError::from(auth_token(reply).expect_err("refused"))
        else {
            panic!("expected a 400");
        };
        assert_eq!(shown, "Account is banned (100123)");
    }

    #[test]
    fn a_200_without_a_token_is_not_a_login() {
        for body in [
            r#"{"Code":200,"Data":{},"Msg":"OK"}"#,
            r#"{"Code":200,"Msg":"OK"}"#,
            r#"{"Code":200,"Data":{"Token":""}}"#,
        ] {
            let err = auth_token(auth_reply(body)).expect_err("no token");
            assert!(matches!(err, FetchError::ParseError(_)), "{body}: {err:?}");
        }
    }

    #[test]
    fn code_200_is_the_only_success() {
        // The reply to a real send-code request, 2026-11-13.
        let sent = json!({ "Code": 200, "Data": {}, "Msg": "OK" });
        assert!(yostar_outcome(&sent, "yostar::send_code").is_ok());
    }

    #[test]
    fn a_refusal_carries_yostar_code_and_message_to_the_client() {
        // The shape Yostar answers a rejected request with (HTTP 200).
        let refused = json!({ "Code": 100_302, "Data": {}, "Msg": "Requests are too frequent" });
        let err = yostar_outcome(&refused, "yostar::send_code").expect_err("refused");
        let FetchError::Upstream(upstream) = &err else {
            panic!("expected Upstream, got {err:?}");
        };
        assert_eq!(upstream.code, 100_302);
        assert_eq!(upstream.msg, "Requests are too frequent");

        let ApiError::BadRequest(shown) = ApiError::from(err) else {
            panic!("a refusal is the caller's to see, as a 400");
        };
        assert_eq!(shown, "Requests are too frequent (100302)");
    }

    #[test]
    fn a_refusal_without_a_message_still_says_something() {
        for refused in [
            json!({ "Code": 100_999 }),
            json!({ "Code": 100_999, "Msg": "" }),
        ] {
            let err = yostar_outcome(&refused, "yostar::send_code").expect_err("refused");
            let ApiError::BadRequest(shown) = ApiError::from(err) else {
                panic!("expected a 400");
            };
            assert_eq!(shown, "Yostar refused the request (100999)");
        }
    }

    #[test]
    fn a_reply_without_a_code_is_not_a_confirmed_send() {
        for reply in [
            json!({}),
            json!({ "Msg": "OK" }),
            json!({ "Code": "200" }),
            json!([]),
        ] {
            let err = yostar_outcome(&reply, "yostar::send_code").expect_err("unconfirmed");
            assert!(matches!(err, FetchError::ParseError(_)), "{reply}: {err:?}");
        }
    }
}
