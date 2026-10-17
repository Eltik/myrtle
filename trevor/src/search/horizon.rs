//! The spoiler horizon: which chunks a reader who has read up to one point of the EN release order may see.
//!
//! Since 2026-10-16 (design/trevor-contract-v1.md section 1) a horizon is an integer `horizonPos` on the **gatePos**
//! scale: every story of `artifacts/spoiler.jsonl` has one 1-based rank in Trevor's EN release order, and a chunk is
//! visible when its story's rank (or, for a dated source that is not a story, its insertion rank) is at or below the
//! horizon. The panels, the backend's `trevor_panels.gate_pos` / `trevor_answers.gate_pos` and this file all read the
//! one order written to [`GATE_ORDER_PATH`]; no component computes its own.
//!
//! The order is the panel prototype's (`~/overnight-2026-10-15/prototypes/trevor-panels/build_panels.py`, `Order`),
//! ported exactly. Each story gets a key, compared field by field:
//! - a story of a group the reading guide places (`artifacts/chrono/reading_guide.json`, 85 groups):
//!   `(guide position, 1, storySort, "", 0, file line)`;
//! - a story or other source the guide does not place but `artifacts/chrono/source_dates.json` dates (an upper bound on
//!   its EN release, scripts/source_dates.py): `(p, 0, date, id before "_story_", storySort, file line)`, where `p` is
//!   the first guide position whose latest guide date is at or after the source's date (10^6 when none is), so the
//!   source sits just before that group's stories (non-story sources take storySort 0 and line 0);
//! - anything else: NEVER, after every other key, and hidden at every horizon.
//!
//! The file line of `spoiler.jsonl` breaks the storySort ties the old story-id horizon let through: main_6 sort 20 (2
//! stories) and act33side sorts 7 (2) and 17 (4), so 8 placed stories that saw each other now see only the ones before
//! them in file order. Placed groups are compared by guide position first, which is how the old horizon read them, so
//! no other placed chunk moves (checked by `ask --horizon-check`), and a dated source's key reproduces the old date
//! rule exactly: visible iff its date is at or before the latest guide date at or before the horizon's group.
//!
//! # `artifacts/chrono/gate_order.json` (schema 1)
//!
//! Written by `ask --gate-order` (atomically), read here; the panel builder reads the same file.
//! ```text
//! {
//!   "schema": 1,
//!   "inputs": {"readingGuide": sha16, "spoiler": sha16, "sourceDates": sha16 | null},   // sha256 of each file, 16 hex
//!   "storyOrder": [storyId, ...],         // every story of spoiler.jsonl; gatePos = index + 1
//!   "never": [storyId, ...],              // stories with no position and no date: hidden at every horizon
//!   "dated": [storyId, ...],              // stories outside the guide placed by their source date (records)
//!   "sourcePos": {sourceId: int, ...},    // dated sources that are not stories (operator files, voice lines,
//!                                         // modules, outfits, game text): visible iff sourcePos <= horizonPos;
//!                                         // sourcePos = 1 + the number of stories whose key is below the source's
//!   "eras": [{"id": groupId, "name": str, "position": gatePos, "groupPosition": int}, ...]
//! }
//! ```
//! `eras` follows the prototype's provisional rule: every main episode, plus an event whenever 6 or more guide groups
//! pass without a boundary (22 eras on 2026-10-16). An era's `position` is the gatePos of its group's last story (the
//! prototype's group key), so `eraPos = max(era.position <= pos)` is the latest era a reader at `pos` has completed.
//! The file records the sha of its three inputs; a missing or stale file is rebuilt in memory from the inputs (with a
//! warning), so serving never runs on an order that disagrees with the story table.
//!
//! Kill switches: `TREVOR_HORIZON_GATE=0` serves the old story-id horizon (group position, then storySort, ties
//! mutually visible; a gatePos is turned back into the story at that rank), byte for byte the 2026-10-07 rule.
//! `TREVOR_HORIZON_DATES=0` ignores the source dates, so every chunk the guide does not place is hidden, as before
//! 2026-10-07 (the ranks stay those of the file).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Where the gatePos order lives, relative to the crate root.
pub const GATE_ORDER_PATH: &str = "artifacts/chrono/gate_order.json";

const NEVER_POS: u64 = 1_000_000_000;
const PAST_LAST_GROUP: u64 = 1_000_000;

/// One release key, compared field by field (the prototype's tuple).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key(u64, u8, i64, String, i64, usize);

impl Key {
    fn never() -> Self {
        Key(NEVER_POS, 0, 0, String::new(), 0, 0)
    }
    fn is_never(&self) -> bool {
        self.0 == NEVER_POS
    }
}

/// An era boundary: the panel builder's, on the gatePos scale.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Era {
    pub id: String,
    pub name: String,
    /// The gatePos of the era group's last story.
    pub position: u32,
    /// The reading guide's position of the group.
    pub group_position: u32,
}

/// The sha16 of each input the order was built from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GateInputs {
    pub reading_guide: String,
    pub spoiler: String,
    pub source_dates: Option<String>,
}

/// `artifacts/chrono/gate_order.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GateOrderFile {
    pub schema: u32,
    pub inputs: Option<GateInputs>,
    pub story_order: Vec<String>,
    pub never: Vec<String>,
    pub dated: Vec<String>,
    pub source_pos: BTreeMap<String, u32>,
    pub eras: Vec<Era>,
}

/// The EN release order: the gatePos of every story and dated source, and (for the `TREVOR_HORIZON_GATE=0` kill switch)
/// the old group position, storySort and date rule.
#[derive(Debug, Default)]
pub struct ReleaseOrder {
    group_pos: HashMap<String, u32>,
    /// story id -> (group id, storySort)
    story: HashMap<String, (String, i64)>,
    /// guide position -> the latest guide release date (unix s) at or before it; absent before the first dated group.
    date_at: HashMap<u32, i64>,
    /// unplaced source story id -> its EN release date (unix s), an upper bound.
    source_date: HashMap<String, i64>,
    /// The gatePos order.
    gate: GateOrderFile,
    /// story id -> gatePos.
    gate_pos: HashMap<String, u32>,
    never: HashSet<String>,
    dated: HashSet<String>,
    /// group id -> the gatePos of its last story.
    group_last: HashMap<String, u32>,
    /// `TREVOR_HORIZON_DATES` was not 0 when loaded.
    use_dates: bool,
    /// `TREVOR_HORIZON_GATE=0`: the old story-id rule.
    legacy: bool,
}

fn sha16_file(path: &Path) -> Result<String> {
    let b = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(crate::util::sha_hex(&b)[..16].to_owned())
}

impl ReleaseOrder {
    /// Read the reading guide, the story table and the source dates under `root`, and the gatePos order from
    /// [`GATE_ORDER_PATH`] (rebuilt in memory, with a warning, when that file is missing or its inputs changed).
    ///
    /// # Errors
    /// The guide or the story table missing or unreadable.
    pub fn load(root: &Path) -> Result<Self> {
        let (guide, rows, dates, inputs) = read_inputs(root)?;
        let use_dates = crate::env::on_unless_0("TREVOR_HORIZON_DATES");
        let legacy = crate::env::is_0("TREVOR_HORIZON_GATE");
        let file: Option<GateOrderFile> = crate::util::read_json_opt(&root.join(GATE_ORDER_PATH));
        let gate = match file {
            Some(f) if f.schema == 1 && f.inputs.as_ref() == Some(&inputs) => f,
            other => {
                let why = if other.is_some() { "is stale (its inputs changed)" } else { "is missing" };
                eprintln!("horizon: {GATE_ORDER_PATH} {why}; using the order rebuilt from its inputs (run `ask --gate-order`)");
                build_gate_order(&guide, &rows, dates.as_ref(), Some(inputs))
            }
        };
        let mut order = Self::from_parts(&guide, &rows);
        if use_dates {
            if let Some(d) = &dates {
                order.set_source_dates(&d["dates"]);
            }
        }
        order.set_gate(gate);
        order.use_dates = use_dates;
        order.legacy = legacy;
        Ok(order)
    }

    /// The gatePos order from the inputs under `root`, as [`GATE_ORDER_PATH`] holds it.
    ///
    /// # Errors
    /// An input missing or unreadable.
    pub fn build_file(root: &Path) -> Result<GateOrderFile> {
        let (guide, rows, dates, inputs) = read_inputs(root)?;
        Ok(build_gate_order(&guide, &rows, dates.as_ref(), Some(inputs)))
    }

    /// Write the gatePos order to [`GATE_ORDER_PATH`] under `root` (through a temporary file renamed into place).
    ///
    /// # Errors
    /// An input missing, or the write failing.
    pub fn write_file(root: &Path) -> Result<GateOrderFile> {
        let f = Self::build_file(root)?;
        let path = root.join(GATE_ORDER_PATH);
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(&f)?)?;
        std::fs::rename(&tmp, &path)?;
        Ok(f)
    }

    /// Date the unplaced sources from `{storyId: unix seconds}` (the old rule's dates).
    pub fn set_source_dates(&mut self, dates: &serde_json::Value) {
        self.source_date = dates.as_object().into_iter().flatten()
            .filter_map(|(k, v)| Some((k.clone(), v.as_i64()?))).collect();
    }

    fn set_gate(&mut self, gate: GateOrderFile) {
        self.gate_pos = gate.story_order.iter().enumerate()
            .map(|(i, s)| (s.clone(), u32::try_from(i + 1).unwrap_or(u32::MAX))).collect();
        self.never = gate.never.iter().cloned().collect();
        self.dated = gate.dated.iter().cloned().collect();
        self.group_last.clear();
        for (sid, (g, _)) in &self.story {
            if let Some(&p) = self.gate_pos.get(sid).filter(|_| !self.never.contains(sid)) {
                let e = self.group_last.entry(g.clone()).or_insert(0);
                *e = (*e).max(p);
            }
        }
        self.gate = gate;
    }

    /// How many unplaced sources carry a date.
    #[must_use]
    pub fn dated_sources(&self) -> usize {
        self.source_date.len()
    }

    /// The gatePos order this horizon reads.
    #[must_use]
    pub fn gate(&self) -> &GateOrderFile {
        &self.gate
    }

    /// The order from the parsed guide (`{"items": [{"groupId", "position"}]}`) and story rows
    /// (`{"storyId", "groupId", "storySort"}`), with the gatePos order built from them (no source dates).
    #[must_use]
    pub fn from_parts(guide: &serde_json::Value, stories: &[serde_json::Value]) -> Self {
        let group_pos = guide["items"].as_array().into_iter().flatten()
            .filter_map(|i| Some((i["groupId"].as_str()?.to_owned(), u32::try_from(i["position"].as_u64()?).ok()?)))
            .collect();
        let story = stories.iter()
            .filter_map(|s| Some((s["storyId"].as_str()?.to_owned(), (s["groupId"].as_str()?.to_owned(), s["storySort"].as_i64()?))))
            .collect();
        let mut order = Self { group_pos, story, date_at: latest_dates(guide), use_dates: true, ..Self::default() };
        order.set_gate(build_gate_order(guide, stories, None, None));
        order
    }

    /// The gatePos of a story id, or of a group id (its last story: the whole group).
    #[must_use]
    pub fn gate_pos_of(&self, id: &str) -> Option<u32> {
        self.gate_pos.get(id).copied().filter(|_| !self.never.contains(id)).or_else(|| self.group_last.get(id).copied())
    }

    /// The horizon at a story id or a group id the reading guide places: its gatePos (`TREVOR_HORIZON_GATE=0`: the old
    /// story-id rule).
    ///
    /// # Errors
    /// The id names no story or group the reading guide places.
    pub fn horizon(self: &Arc<Self>, id: &str) -> Result<Horizon> {
        let group = self.story.get(id).map_or(id, |(g, _)| g.as_str());
        if !self.group_pos.contains_key(group) {
            bail!("horizon {id:?} is not a story or group in the EN release order");
        }
        if self.legacy {
            return self.legacy_horizon(id);
        }
        let pos = self.gate_pos_of(id).with_context(|| format!("horizon {id:?} has no gatePos"))?;
        Ok(Horizon { id: id.to_owned(), pos, legacy: None, order: Arc::clone(self) })
    }

    /// The horizon at gatePos `pos` (contract section 1: a job's `horizonPos`). 0 hides every story.
    ///
    /// # Errors
    /// Only under `TREVOR_HORIZON_GATE=0`, when the story at `pos` is not placed by the reading guide.
    pub fn horizon_at(self: &Arc<Self>, pos: u32) -> Result<Horizon> {
        if self.legacy {
            let sid = pos.checked_sub(1).and_then(|i| self.gate.story_order.get(i as usize)).cloned()
                .with_context(|| format!("horizonPos {pos} names no story"))?;
            return self.legacy_horizon(&sid);
        }
        Ok(Horizon { id: format!("gate:{pos}"), pos, legacy: None, order: Arc::clone(self) })
    }

    /// The old (2026-10-07) horizon of a story or group id: group position, then storySort with ties visible.
    ///
    /// # Errors
    /// The id names no story or group the reading guide places.
    pub fn legacy_horizon(self: &Arc<Self>, id: &str) -> Result<Horizon> {
        let (group, sort) = match self.story.get(id) {
            Some((g, s)) => (g.as_str(), Some(*s)),
            None => (id, None),
        };
        let pos = *self.group_pos.get(group)
            .with_context(|| format!("horizon {id:?} is not a story or group in the EN release order"))?;
        let legacy = Legacy { pos, sort, date: self.date_at.get(&pos).copied() };
        Ok(Horizon { id: id.to_owned(), pos: self.gate_pos_of(id).unwrap_or(0), legacy: Some(legacy), order: Arc::clone(self) })
    }

    /// The era a reader at gatePos `pos` has completed: `max(era.position <= pos)`, or None (eraPos 0).
    #[must_use]
    pub fn era_at(&self, pos: u32) -> Option<&Era> {
        self.gate.eras.iter().filter(|e| e.position <= pos).max_by_key(|e| e.position)
    }
}

type Inputs = (serde_json::Value, Vec<serde_json::Value>, Option<serde_json::Value>, GateInputs);

fn read_inputs(root: &Path) -> Result<Inputs> {
    let guide_path = root.join("artifacts/chrono/reading_guide.json");
    let spoiler_path = root.join("artifacts/spoiler.jsonl");
    let dates_path = root.join("artifacts/chrono/source_dates.json");
    let guide: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&guide_path)
        .context("reading artifacts/chrono/reading_guide.json")?)?;
    let stories = std::fs::read_to_string(&spoiler_path).context("reading artifacts/spoiler.jsonl")?;
    let rows: Vec<serde_json::Value> = stories.lines().filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
    let dates: Option<serde_json::Value> = if dates_path.exists() {
        Some(serde_json::from_str(&std::fs::read_to_string(&dates_path).context("reading artifacts/chrono/source_dates.json")?)?)
    } else { None };
    let inputs = GateInputs { reading_guide: sha16_file(&guide_path)?, spoiler: sha16_file(&spoiler_path)?,
                              source_dates: if dates.is_some() { Some(sha16_file(&dates_path)?) } else { None } };
    Ok((guide, rows, dates, inputs))
}

/// guide position -> the latest guide date at or before it. The guide's "released" is "2020-01-22", "by 2020-01-22",
/// "about 2020-07-01" or undated text; a position takes the latest date at or before it, so an undated group never
/// admits a source dated after its successors.
fn latest_dates(guide: &serde_json::Value) -> HashMap<u32, i64> {
    let mut dated: Vec<(u32, Option<i64>)> = guide["items"].as_array().into_iter().flatten()
        .filter_map(|i| Some((u32::try_from(i["position"].as_u64()?).ok()?, i["released"].as_str().and_then(parse_day))))
        .collect();
    dated.sort_by_key(|(p, _)| *p);
    let mut date_at = HashMap::new();
    let mut latest: Option<i64> = None;
    for (p, d) in dated {
        latest = match (latest, d) { (Some(a), Some(b)) => Some(a.max(b)), (a, b) => a.or(b) };
        if let Some(l) = latest {
            date_at.insert(p, l);
        }
    }
    date_at
}

/// The gatePos order, as the panel prototype's `Order` ranks it.
fn build_gate_order(guide: &serde_json::Value, rows: &[serde_json::Value], dates: Option<&serde_json::Value>,
                    inputs: Option<GateInputs>) -> GateOrderFile {
    let items: Vec<&serde_json::Value> = guide["items"].as_array().into_iter().flatten().collect();
    let gpos: HashMap<&str, u64> = items.iter().filter_map(|i| Some((i["groupId"].as_str()?, i["position"].as_u64()?))).collect();
    let date_at = latest_dates(guide);
    let mut pos_sorted: Vec<u32> = date_at.keys().copied().collect();
    pos_sorted.sort_unstable();
    let dates_sorted: Vec<i64> = pos_sorted.iter().map(|p| date_at[p]).collect();
    let source_date: HashMap<&str, i64> = dates.and_then(|d| d["dates"].as_object()).into_iter().flatten()
        .filter_map(|(k, v)| Some((k.as_str(), v.as_i64()?))).collect();
    let dated_key = |d: i64, sid: &str, sort: i64, line: usize| {
        // The first guide position whose latest date admits d (bisect_left); the source sits just before its stories.
        let i = dates_sorted.partition_point(|x| *x < d);
        let p = pos_sorted.get(i).map_or(PAST_LAST_GROUP, |p| u64::from(*p));
        let base = sid.rsplit_once("_story_").map_or(sid, |(b, _)| b).to_owned();
        Key(p, 0, d, base, sort, line)
    };
    // Stories in file order; a repeated id keeps its first place and its last line, as a Python dict does.
    let mut ids: Vec<&str> = Vec::new();
    let mut row_of: HashMap<&str, (usize, &serde_json::Value)> = HashMap::new();
    for (line, r) in rows.iter().enumerate() {
        let Some(sid) = r["storyId"].as_str() else { continue };
        if row_of.insert(sid, (line, r)).is_none() {
            ids.push(sid);
        }
    }
    let key_of = |sid: &str| -> Key {
        let (line, r) = row_of[sid];
        let sort = r["storySort"].as_i64().unwrap_or(0);
        match r["groupId"].as_str().and_then(|g| gpos.get(g)) {
            Some(&p) => Key(p, 1, sort, String::new(), 0, line),
            None => source_date.get(sid).map_or_else(Key::never, |&d| dated_key(d, sid, sort, line)),
        }
    };
    let mut keyed: Vec<(Key, &str)> = ids.iter().map(|s| (key_of(s), *s)).collect();
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    let story_order: Vec<String> = keyed.iter().map(|(_, s)| (*s).to_owned()).collect();
    let never = keyed.iter().filter(|(k, _)| k.is_never()).map(|(_, s)| (*s).to_owned()).collect();
    let dated = keyed.iter().filter(|(k, _)| !k.is_never() && k.1 == 0).map(|(_, s)| (*s).to_owned()).collect();
    let keys: Vec<&Key> = keyed.iter().map(|(k, _)| k).collect();
    let source_pos = source_date.iter().filter(|(s, _)| !row_of.contains_key(*s)).map(|(s, &d)| {
        let k = dated_key(d, s, 0, 0);
        ((*s).to_owned(), u32::try_from(keys.partition_point(|x| **x < k) + 1).unwrap_or(u32::MAX))
    }).collect();
    // Eras: every main episode, plus an event whenever 6 or more groups pass without a boundary; each at the gatePos
    // of its group's last story (the prototype's group key).
    let mut last_of: HashMap<&str, u32> = HashMap::new();
    for (i, (k, s)) in keyed.iter().enumerate() {
        if let Some(g) = row_of[s].1["groupId"].as_str().filter(|_| !k.is_never()) {
            last_of.insert(g, u32::try_from(i + 1).unwrap_or(u32::MAX));
        }
    }
    let mut sorted_items = items.clone();
    sorted_items.sort_by_key(|i| i["position"].as_u64().unwrap_or(u64::MAX));
    let mut eras = Vec::new();
    let mut last: i64 = -99;
    for g in sorted_items {
        let p = i64::try_from(g["position"].as_u64().unwrap_or(0)).unwrap_or(0);
        if g["kind"].as_str() == Some("main") || p - last >= 6 {
            last = p;
            let id = g["groupId"].as_str().unwrap_or_default();
            if let Some(&position) = last_of.get(id) {
                eras.push(Era { id: id.to_owned(), name: g["name"].as_str().unwrap_or_default().to_owned(), position,
                                group_position: u32::try_from(p).unwrap_or(0) });
            }
        }
    }
    GateOrderFile { schema: 1, inputs, story_order, never, dated, source_pos, eras }
}

/// Unix seconds (UTC midnight) of the first `YYYY-MM-DD` in `s`.
fn parse_day(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    (0..b.len().saturating_sub(9)).find_map(|i| {
        let w = s.get(i..i + 10)?;
        let ok = w.bytes().enumerate().all(|(j, c)| if j == 4 || j == 7 { c == b'-' } else { c.is_ascii_digit() });
        if !ok {
            return None;
        }
        let (y, m, d): (i64, i64, i64) = (w[..4].parse().ok()?, w[5..7].parse().ok()?, w[8..].parse().ok()?);
        // Days from civil (Howard Hinnant's algorithm).
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Some((era * 146_097 + doe - 719_468) * 86_400)
    })
}

/// The old rule's horizon: guide position, the horizon story's storySort (None for a whole group) and date.
#[derive(Debug, Clone)]
struct Legacy {
    pos: u32,
    sort: Option<i64>,
    date: Option<i64>,
}

/// A reader's position: what they may see.
#[derive(Debug, Clone)]
pub struct Horizon {
    /// What it was built from: a story or group id, or "gate:N".
    pub id: String,
    /// The gatePos: chunks of stories ranked at or below it are visible.
    pub pos: u32,
    legacy: Option<Legacy>,
    order: Arc<ReleaseOrder>,
}

impl Horizon {
    /// Whether a chunk of `story_id` in `group_id` was released at or before the horizon.
    #[must_use]
    pub fn allows(&self, story_id: &str, group_id: &str) -> bool {
        let o = &self.order;
        if let Some(l) = &self.legacy {
            return match o.group_pos.get(group_id) {
                None => match (l.date, o.source_date.get(story_id)) {
                    (Some(h), Some(&d)) => d <= h,
                    _ => false,
                },
                Some(&p) if p < l.pos => true,
                Some(&p) if p > l.pos => false,
                Some(_) => match l.sort {
                    None => true,
                    Some(h) => o.story.get(story_id).is_some_and(|(_, s)| *s <= h),
                },
            };
        }
        match o.gate_pos.get(story_id) {
            Some(&p) => p <= self.pos && !o.never.contains(story_id) && (o.use_dates || !o.dated.contains(story_id)),
            None => o.use_dates && o.gate.source_pos.get(story_id).is_some_and(|&p| p <= self.pos),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn guide() -> serde_json::Value {
        json!({"items": [{"groupId": "main_0", "position": 1, "released": "by 2020-01-22", "kind": "main", "name": "Ep 0"},
                         {"groupId": "act1", "position": 2, "released": "not dated in the game data", "kind": "event", "name": "A1"},
                         {"groupId": "main_1", "position": 3, "released": "about 2020-07-01", "kind": "main", "name": "Ep 1"}]})
    }

    fn order() -> Arc<ReleaseOrder> {
        let stories = vec![
            json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
            json!({"storyId": "a1_a", "groupId": "act1", "storySort": 1}),
            json!({"storyId": "a1_b", "groupId": "act1", "storySort": 2}),
            json!({"storyId": "a1_c", "groupId": "act1", "storySort": 3}),
            json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1}),
        ];
        Arc::new(ReleaseOrder::from_parts(&guide(), &stories))
    }

    #[test]
    fn story_horizon_keeps_earlier_groups_and_earlier_stories_of_its_own() {
        let h = order().horizon("a1_b").unwrap();
        assert_eq!(h.pos, 3);
        assert!(h.allows("m0_a", "main_0"));
        assert!(h.allows("a1_a", "act1"));
        assert!(h.allows("a1_b", "act1"));
        assert!(!h.allows("a1_c", "act1"));
        assert!(!h.allows("m1_a", "main_1"));
        // Unplaced groups (operator files, voice lines, generated summaries) are hidden, never let through.
        assert!(!h.allows("archive_char_002_amiya", "archive"));
        assert!(!h.allows("unknown_story", "act1"));
        // The same horizon by gatePos; gatePos 0 hides everything.
        let o = order();
        let g = o.horizon_at(3).unwrap();
        assert!(g.allows("a1_b", "act1") && !g.allows("a1_c", "act1"));
        assert!(!o.horizon_at(0).unwrap().allows("m0_a", "main_0"));
    }

    #[test]
    fn dated_sources_follow_the_horizon_date() {
        assert_eq!(parse_day("by 2020-01-22"), Some(1_579_651_200));
        assert_eq!(parse_day("not dated"), None);
        let rows = [json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
                    json!({"storyId": "a1_a", "groupId": "act1", "storySort": 1}),
                    json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1})];
        let mut o = ReleaseOrder::from_parts(&guide(), &rows);
        let dates = json!({"dates": {"archive_char_002_amiya": 1_579_651_200, "voice_char_1": 1_579_651_201}});
        o.set_source_dates(&dates["dates"]);
        o.set_gate(build_gate_order(&guide(), &rows, Some(&dates), None));
        let o = Arc::new(o);
        // Amiya's file sits before main_0's stories (same day); the voice line before main_1's.
        assert_eq!(o.gate().source_pos.get("archive_char_002_amiya"), Some(&1));
        assert_eq!(o.gate().source_pos.get("voice_char_1"), Some(&3));
        for (h, amiya, voice) in [("main_0", true, false), ("act1", true, false), ("main_1", true, true)] {
            for hz in [o.horizon(h).unwrap(), o.legacy_horizon(h).unwrap()] {
                assert_eq!(hz.allows("archive_char_002_amiya", "archive"), amiya, "{h} {:?}", hz.legacy);
                assert_eq!(hz.allows("voice_char_1", "voice"), voice, "{h}");
                // Undated sources stay hidden at any horizon.
                assert!(!hz.allows("enemy_enemy_1007_slime", "enemy"));
            }
        }
    }

    #[test]
    fn group_horizon_keeps_the_whole_group() {
        let h = order().horizon("act1").unwrap();
        assert!(h.allows("a1_c", "act1"));
        assert!(!h.allows("m1_a", "main_1"));
        assert!(order().horizon("nope").is_err());
    }

    #[test]
    fn story_sort_ties_break_by_file_order_and_eras_sit_at_the_last_story() {
        let stories = vec![
            json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
            json!({"storyId": "a1_x", "groupId": "act1", "storySort": 7}),
            json!({"storyId": "a1_y", "groupId": "act1", "storySort": 7}),
            json!({"storyId": "a1_w", "groupId": "act1", "storySort": 2}),
            json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1}),
            json!({"storyId": "rec_1", "groupId": "story_x_set_1", "storySort": 0}),
        ];
        let o = Arc::new(ReleaseOrder::from_parts(&guide(), &stories));
        assert_eq!(o.gate().story_order, ["m0_a", "a1_w", "a1_x", "a1_y", "m1_a", "rec_1"]);
        assert_eq!(o.gate().never, ["rec_1"]);
        // The tie: the old rule let a1_x see a1_y; the gatePos rule does not, and a1_y still sees a1_x.
        assert!(o.legacy_horizon("a1_x").unwrap().allows("a1_y", "act1"));
        assert!(!o.horizon("a1_x").unwrap().allows("a1_y", "act1"));
        assert!(o.horizon("a1_y").unwrap().allows("a1_x", "act1"));
        // Never-visible stories stay hidden even past their rank.
        assert!(!o.horizon_at(99).unwrap().allows("rec_1", "story_x_set_1"));
        let eras: Vec<(&str, u32)> = o.gate().eras.iter().map(|e| (e.id.as_str(), e.position)).collect();
        assert_eq!(eras, [("main_0", 1), ("main_1", 5)]);
        assert_eq!(o.era_at(4).map(|e| e.id.as_str()), Some("main_0"));
        assert_eq!(o.era_at(0), None);
    }
}
