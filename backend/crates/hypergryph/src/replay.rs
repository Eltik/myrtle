use std::io::{Cursor, Read};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use myrtle_types::Server;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::error::{UpstreamError, envelope_code};
use crate::fetch::{auth_request, read_body, redacted_body};
use crate::login::yostar::sync_data;
use crate::session::AuthSession;
use crate::upstream::Upstream;

fn de_bool_loose<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    use serde::de::Error;
    match Value::deserialize(d)? {
        Value::Bool(b) => Ok(b),
        Value::Number(n) => Ok(n.as_i64().unwrap_or(0) != 0),
        v => Err(D::Error::custom(format!("expected bool or 0/1, got {v}"))),
    }
}

fn de_i64_loose<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    use serde::de::Error;
    match Value::deserialize(d)? {
        Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| D::Error::custom(format!("expected i64, got {n}"))),
        Value::String(s) => s
            .parse::<i64>()
            .map_err(|_| D::Error::custom(format!("expected i64 string, got {s:?}"))),
        v => Err(D::Error::custom(format!("expected i64 or string, got {v}"))),
    }
}

fn de_unix_secs<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
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
    pub rune_list: Value,
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

const DECODE: &str = "replay::decode";
const GET_BATTLE_REPLAY: &str = "replay::getBattleReplay";

pub fn decode_battle_replay(battle_replay_b64: &str) -> Result<BattleReplay, UpstreamError> {
    let zip_bytes = STANDARD
        .decode(battle_replay_b64)
        .map_err(|e| UpstreamError::parse(DECODE, format!("base64 decode: {e}")))?;
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| UpstreamError::parse(DECODE, format!("open zip: {e}")))?;
    let mut entry = archive
        .by_name("default_entry")
        .map_err(|e| UpstreamError::parse(DECODE, format!("missing default_entry in zip: {e}")))?;

    let mut json_bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut json_bytes)
        .map_err(|e| UpstreamError::parse(DECODE, format!("read default_entry: {e}")))?;

    serde_json::from_slice(&json_bytes).map_err(|e| {
        UpstreamError::parse(
            DECODE,
            format!(
                "parse default_entry: {e} (line {}, col {})",
                e.line(),
                e.column()
            ),
        )
    })
}

#[derive(Deserialize)]
struct GetBattleReplayResp {
    #[serde(rename = "battleReplay")]
    battle_replay: String,
}

pub async fn get_battle_replay(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
    battle_type: &str,
    stage_id: &str,
) -> Result<BattleReplay, UpstreamError> {
    let endpoint = format!("{battle_type}/getBattleReplay");
    let body = serde_json::json!({ "stageId": stage_id });

    let response = auth_request(upstream, &endpoint, Some(&body), session, server).await?;
    let text = read_body(response, GET_BATTLE_REPLAY).await?;

    let data: GetBattleReplayResp = serde_json::from_str(&text).map_err(|e| {
        UpstreamError::parse(
            GET_BATTLE_REPLAY,
            format!(
                "parse battleReplay JSON ({e}); body={}",
                redacted_body(&text)
            ),
        )
    })?;
    decode_battle_replay(&data.battle_replay)
}

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

impl ReplayHarvestOptions {
    /// A random wait in `min_interval..=max_interval`, `min_interval` when the range is empty
    #[must_use]
    pub fn next_delay(&self) -> Duration {
        if self.max_interval <= self.min_interval {
            return self.min_interval;
        }
        let lo = self.min_interval.as_millis() as u64;
        let hi = self.max_interval.as_millis() as u64;
        Duration::from_millis(rand::random_range(lo..=hi))
    }
}

#[derive(Debug)]
pub enum ReplayOutcome {
    Replay(BattleReplay),
    /// 5516: nothing saved for the stage
    NoSaved,
}

pub async fn harvest_replays<I, F>(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
    targets: I,
    opts: &ReplayHarvestOptions,
    mut on_progress: F,
) -> Result<(), UpstreamError>
where
    I: IntoIterator<Item = (String, String)>,
    F: FnMut(&str, &str, Result<ReplayOutcome, &UpstreamError>),
{
    let mut consecutive_errors = 0usize;
    let mut first = true;

    for (battle_type, stage_id) in targets {
        if !first {
            let delay = opts.next_delay();
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
        }
        first = false;

        match get_battle_replay(upstream, session, server, &battle_type, &stage_id).await {
            Ok(r) => {
                consecutive_errors = 0;
                on_progress(&battle_type, &stage_id, Ok(ReplayOutcome::Replay(r)));
            }
            Err(e) if e.is_envelope_code(envelope_code::STAGE_NO_REPLAY) => {
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

#[must_use]
pub fn saved_replay_targets(sync: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    collect_replay_flags(sync.pointer("/user/dungeon/stages"), "quest", &mut out);
    collect_replay_flags(
        sync.pointer("/user/campaignsV2/instances"),
        "campaignV2",
        &mut out,
    );
    out
}

fn collect_replay_flags(node: Option<&Value>, battle_type: &str, out: &mut Vec<(String, String)>) {
    let Some(Value::Object(stages)) = node else {
        return;
    };
    out.extend(stages.iter().filter_map(|(stage_id, entry)| {
        let flag = entry.get("hasBattleReplay")?;
        let on = flag.as_i64().is_some_and(|n| n != 0) || flag.as_bool().unwrap_or(false);
        on.then(|| (battle_type.to_owned(), stage_id.clone()))
    }));
}

pub async fn collect_player_replays<F>(
    upstream: &Upstream,
    session: &mut AuthSession,
    server: Server,
    opts: &ReplayHarvestOptions,
    on_progress: F,
) -> Result<(), UpstreamError>
where
    F: FnMut(&str, &str, Result<ReplayOutcome, &UpstreamError>),
{
    let sync = sync_data(upstream, session, server).await?;
    let targets = saved_replay_targets(&sync);
    tracing::info!(count = targets.len(), "discovered saved replays");
    harvest_replays(upstream, session, server, targets, opts, on_progress).await
}
