//! Building the gate order from its inputs: the reading guide, the stories of `spoiler.jsonl` and the source dates.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use super::{Era, GateInputs, GateOrderFile, Key, PAST_LAST_GROUP, sha16_file};

type Inputs = (serde_json::Value, Vec<serde_json::Value>, Option<serde_json::Value>, GateInputs);

pub(super) fn read_inputs(root: &Path) -> Result<Inputs> {
    let guide_path = root.join("artifacts/chrono/reading_guide.json");
    let spoiler_path = root.join("artifacts/spoiler.jsonl");
    let dates_path = root.join("artifacts/chrono/source_dates.json");
    let guide: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&guide_path)
        .context("reading artifacts/chrono/reading_guide.json")?)?;
    let stories = std::fs::read_to_string(&spoiler_path).context("reading artifacts/spoiler.jsonl")?;
    let rows: Vec<serde_json::Value> = stories.lines().filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
    let dates: Option<serde_json::Value> = if dates_path.exists() {
        Some(serde_json::from_str(
            &std::fs::read_to_string(&dates_path).context("reading artifacts/chrono/source_dates.json")?
        )?)
    } else { None };
    let inputs = GateInputs { reading_guide: sha16_file(&guide_path)?, spoiler: sha16_file(&spoiler_path)?,
                              source_dates: if dates.is_some() { Some(sha16_file(&dates_path)?) } else { None } };
    Ok((guide, rows, dates, inputs))
}

/// guide position -> the latest guide date at or before it. The guide's "released" is "2020-01-22", "by 2020-01-22",
/// "about 2020-07-01" or undated text; a position takes the latest date at or before it, so an undated group never
/// admits a source dated after its successors.
pub(super) fn latest_dates(guide: &serde_json::Value) -> HashMap<u32, i64> {
    let mut dated: Vec<(u32, Option<i64>)> = guide["items"].as_array().into_iter().flatten()
        .filter_map(|i| Some((
            u32::try_from(i["position"].as_u64()?).ok()?,
            i["released"].as_str().and_then(parse_day)
        )))
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
pub(super) fn build_gate_order(guide: &serde_json::Value, rows: &[serde_json::Value], dates: Option<&serde_json::Value>,
                    inputs: Option<GateInputs>) -> GateOrderFile {
    let items: Vec<&serde_json::Value> = guide["items"].as_array().into_iter().flatten().collect();
    let gpos: HashMap<&str, u64> = items
        .iter()
        .filter_map(|i| Some((i["groupId"].as_str()?, i["position"].as_u64()?)))
        .collect();
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
pub(super) fn parse_day(s: &str) -> Option<i64> {
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
