//! The spoiler horizon: which chunks a reader who has read up to one point of the EN release order may see.
//!
//! A horizon (design/trevor-contract-v1.md section 1) is an integer `horizonPos` on the **gatePos**
//! scale: every story of `artifacts/spoiler.jsonl` has one 1-based rank in Trevor's EN release order, and a chunk is
//! visible when its story's rank (or, for a dated source that is not a story, its insertion rank) is at or below the
//! horizon. The panels, the backend's `trevor_panels.gate_pos` / `trevor_answers.gate_pos` and this file all read the
//! one order written to [`GATE_ORDER_PATH`]; no component computes its own.
//!
//! The order is the panel prototype's (`build_panels.py`, `Order`), ported exactly. Each story gets a key, compared
//! field by field:
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
//! pass without a boundary. An era's `position` is the gatePos of its group's last story (the
//! prototype's group key), so `eraPos = max(era.position <= pos)` is the latest era a reader at `pos` has completed.
//! The file records the sha of its three inputs; a missing or stale file is rebuilt in memory from the inputs (with a
//! warning), so serving never runs on an order that disagrees with the story table.
//!
//! Kill switches: `TREVOR_HORIZON_GATE=0` serves the old story-id horizon (group position, then storySort, ties
//! mutually visible; a gatePos is turned back into the story at that rank), byte for byte the story-id rule.
//! `TREVOR_HORIZON_DATES=0` ignores the source dates, so every chunk the guide does not place is hidden (the ranks
//! stay those of the file).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

mod build;
#[cfg(test)]
mod tests;

use build::{build_gate_order, latest_dates, read_inputs};

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
                let why = if other.is_some() {
                    "is stale (its inputs changed)"
                } else {
                    "is missing"
                };
                eprintln!(
                    "horizon: {GATE_ORDER_PATH} {why}; \
                           using the order rebuilt from its inputs (run `ask --gate-order`)"
                );
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
        self.source_date = dates
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(k, v)| Some((k.clone(), v.as_i64()?)))
            .collect();
    }

    fn set_gate(&mut self, gate: GateOrderFile) {
        self.gate_pos = gate
            .story_order
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), u32::try_from(i + 1).unwrap_or(u32::MAX)))
            .collect();
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
        let group_pos = guide["items"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|i| {
                Some((
                    i["groupId"].as_str()?.to_owned(),
                    u32::try_from(i["position"].as_u64()?).ok()?,
                ))
            })
            .collect();
        let story = stories
            .iter()
            .filter_map(|s| {
                Some((
                    s["storyId"].as_str()?.to_owned(),
                    (s["groupId"].as_str()?.to_owned(), s["storySort"].as_i64()?),
                ))
            })
            .collect();
        let mut order = Self {
            group_pos,
            story,
            date_at: latest_dates(guide),
            use_dates: true,
            ..Self::default()
        };
        order.set_gate(build_gate_order(guide, stories, None, None));
        order
    }

    /// The gatePos of a story id, or of a group id (its last story: the whole group).
    #[must_use]
    pub fn gate_pos_of(&self, id: &str) -> Option<u32> {
        self.gate_pos
            .get(id)
            .copied()
            .filter(|_| !self.never.contains(id))
            .or_else(|| self.group_last.get(id).copied())
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
        let pos = self
            .gate_pos_of(id)
            .with_context(|| format!("horizon {id:?} has no gatePos"))?;
        Ok(Horizon {
            id: id.to_owned(),
            pos,
            legacy: None,
            order: Arc::clone(self),
        })
    }

    /// The horizon at gatePos `pos` (contract section 1: a job's `horizonPos`). 0 hides every story.
    ///
    /// # Errors
    /// Only under `TREVOR_HORIZON_GATE=0`, when the story at `pos` is not placed by the reading guide.
    pub fn horizon_at(self: &Arc<Self>, pos: u32) -> Result<Horizon> {
        if self.legacy {
            let sid = pos
                .checked_sub(1)
                .and_then(|i| self.gate.story_order.get(i as usize))
                .cloned()
                .with_context(|| format!("horizonPos {pos} names no story"))?;
            return self.legacy_horizon(&sid);
        }
        Ok(Horizon {
            id: format!("gate:{pos}"),
            pos,
            legacy: None,
            order: Arc::clone(self),
        })
    }

    /// The old story-id horizon of a story or group id: group position, then storySort with ties visible.
    ///
    /// # Errors
    /// The id names no story or group the reading guide places.
    pub fn legacy_horizon(self: &Arc<Self>, id: &str) -> Result<Horizon> {
        let (group, sort) = match self.story.get(id) {
            Some((g, s)) => (g.as_str(), Some(*s)),
            None => (id, None),
        };
        let pos = *self
            .group_pos
            .get(group)
            .with_context(|| format!("horizon {id:?} is not a story or group in the EN release order"))?;
        let legacy = Legacy {
            pos,
            sort,
            date: self.date_at.get(&pos).copied(),
        };
        Ok(Horizon {
            id: id.to_owned(),
            pos: self.gate_pos_of(id).unwrap_or(0),
            legacy: Some(legacy),
            order: Arc::clone(self),
        })
    }

    /// The era a reader at gatePos `pos` has completed: `max(era.position <= pos)`, or None (eraPos 0).
    #[must_use]
    pub fn era_at(&self, pos: u32) -> Option<&Era> {
        self.gate
            .eras
            .iter()
            .filter(|e| e.position <= pos)
            .max_by_key(|e| e.position)
    }

    /// The gatePos of a group's first and last placed story (stories hidden at every horizon left out), or None when
    /// it has none.
    #[must_use]
    pub fn group_span(&self, group_id: &str) -> Option<(u32, u32)> {
        let pos = self
            .story
            .iter()
            .filter(|(sid, (g, _))| g == group_id && !self.never.contains(*sid))
            .filter_map(|(sid, _)| self.gate_pos.get(sid).copied());
        pos.fold(None, |acc, p| {
            Some(acc.map_or((p, p), |(a, b): (u32, u32)| (a.min(p), b.max(p))))
        })
    }
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
    /// The release order this horizon reads.
    #[must_use]
    pub fn order(&self) -> &ReleaseOrder {
        &self.order
    }

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
