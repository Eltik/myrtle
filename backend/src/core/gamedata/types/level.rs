//! Stage **level** data — the tile map, enemy routes, and wave/spawn schedule
//! that drive the Stage Viewer's pathing simulator.
//!
//! The raw `level_*.json` files (`PascalCase`, ~50-300 KB each) live under
//! `gamedata/levels/` and are *not* loaded into [`GameData`] at boot (there are
//! 2600+ of them). Instead a single file is read + normalized on demand by
//! [`crate::app::services::level`]. This module owns both the raw source shape
//! and the clean camelCase [`StageMap`] the frontend consumes.
//!
//! ## Coordinate convention
//! The game stores tile rows bottom-up (route `Row` 0 = bottom) while the tile
//! `Matrix_data` is laid out top-down. We normalize **everything** to screen
//! space: `y = 0` is the top row, growing downward; `x` is the column. Callers
//! never have to flip again.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::enemy::{DamageType, Enemy, EnemyHandbook, EnemyLevel};
use super::stage::Stage;

// ============================================================================
// Raw source shape (level_*.json)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawLevel {
    map_data: RawMapData,
    #[serde(default)]
    routes: Vec<RawRoute>,
    #[serde(default)]
    waves: Vec<RawWave>,
    #[serde(default)]
    enemy_db_refs: Vec<RawDbRef>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawMapData {
    map: RawMap,
    #[serde(default)]
    tiles: Vec<RawTile>,
}

#[derive(Debug, Deserialize)]
struct RawMap {
    #[serde(rename = "Column_size")]
    column_size: usize,
    #[serde(rename = "Matrix_data", default)]
    matrix_data: Vec<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawTile {
    #[serde(default)]
    tile_key: String,
    #[serde(default)]
    height_type: String,
    #[serde(default)]
    buildable_type: String,
    #[serde(default)]
    passable_mask: String,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawPos {
    #[serde(default)]
    col: i32,
    #[serde(default)]
    row: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawRoute {
    #[serde(default)]
    motion_mode: String,
    #[serde(default)]
    start_position: RawPos,
    #[serde(default)]
    end_position: RawPos,
    #[serde(default)]
    checkpoints: Vec<RawCheckpoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawCheckpoint {
    #[serde(rename = "Type_", default)]
    type_: String,
    #[serde(default)]
    position: RawPos,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawWave {
    #[serde(default)]
    pre_delay: f64,
    #[serde(default)]
    post_delay: f64,
    #[serde(default)]
    max_time_waiting_for_next_wave: f64,
    #[serde(default)]
    fragments: Vec<RawFragment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawFragment {
    #[serde(default)]
    pre_delay: f64,
    #[serde(default)]
    actions: Vec<RawAction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawAction {
    #[serde(default)]
    action_type: String,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    count: i64,
    #[serde(default)]
    pre_delay: f64,
    #[serde(default)]
    interval: f64,
    #[serde(default)]
    route_index: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawDbRef {
    id: String,
}

// ============================================================================
// Normalized output shape (what the API returns)
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileCell {
    /// Normalized rendering kind (`road`, `high`, `start`, `hazard`, …).
    pub kind: String,
    pub tile_key: String,
    pub height_type: String,
    pub buildable: String,
    pub passable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteOut {
    /// `WALK`, `FLY`, or `OTHER`.
    pub motion: String,
    /// Polyline in screen space: spawn → checkpoints → goal.
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WaveMarker {
    pub at: f64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnOut {
    pub route: usize,
    pub enemy_id: String,
    /// Absolute spawn time in seconds from stage start.
    pub t0: f64,
    /// Movement speed in tiles/sec (drives the token animation).
    pub speed: f64,
    /// Threat level: `NORMAL`, `ELITE`, or `BOSS`.
    pub level: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterEntry {
    pub enemy_id: String,
    pub name: String,
    pub level: String,
    pub damage_type: Vec<String>,
    pub attack_type: Option<String>,
    pub motion: String,
    /// Total spawned across all waves (0 = declared/summon-only).
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageMap {
    pub stage_id: String,
    pub level_id: String,
    pub code: String,
    pub name: Option<String>,
    pub width: usize,
    pub height: usize,
    /// Row-major, top row first. `tiles[y][x]`.
    pub tiles: Vec<Vec<TileCell>>,
    pub routes: Vec<RouteOut>,
    pub waves: Vec<WaveMarker>,
    pub spawns: Vec<SpawnOut>,
    pub roster: Vec<RosterEntry>,
    /// Total run length in seconds (last enemy reaches the goal).
    pub duration: f64,
}

// ============================================================================
// Builder
// ============================================================================

const DEFAULT_SPEED: f64 = 0.72;

/// Parse a raw level file and normalize it against the stage metadata and the
/// enemy handbook (for roster names, threat levels, and movement speeds).
pub fn parse_stage_map(
    stage: &Stage,
    level_id: &str,
    bytes: &[u8],
    enemies: &EnemyHandbook,
) -> Result<StageMap, serde_json::Error> {
    let raw: RawLevel = serde_json::from_slice(bytes)?;

    let cols = raw.map_data.map.column_size.max(1); // guards the division below
    let matrix = &raw.map_data.map.matrix_data;
    let rows = matrix.len() / cols;

    // ── Tiles (row 0 = top, matching Matrix_data layout) ──────────────────
    let tiles: Vec<Vec<TileCell>> = (0..rows)
        .map(|y| {
            (0..cols)
                .map(|x| {
                    let idx = matrix.get(y * cols + x).copied().unwrap_or(usize::MAX);
                    raw.map_data
                        .tiles
                        .get(idx)
                        .map_or_else(empty_tile, tile_cell)
                })
                .collect()
        })
        .collect();

    // ── Routes → screen-space polylines ───────────────────────────────────
    let to_point = |p: RawPos| Point {
        x: f64::from(p.col),
        y: f64::from((rows as i32 - 1).max(0) - p.row),
    };
    let routes: Vec<RouteOut> = raw
        .routes
        .iter()
        .map(|r| {
            let mut pts: Vec<Point> = Vec::with_capacity(r.checkpoints.len() + 2);
            pts.push(to_point(r.start_position));
            for cp in &r.checkpoints {
                // Non-MOVE checkpoints (WAIT_*, APPEAR_AT_POS, …) don't add a
                // distinct waypoint; we still place a point and dedupe below.
                let _ = &cp.type_;
                pts.push(to_point(cp.position));
            }
            pts.push(to_point(r.end_position));
            dedupe_consecutive(&mut pts);
            RouteOut {
                motion: motion_label(&r.motion_mode),
                points: pts,
            }
        })
        .collect();

    let route_len: Vec<f64> = routes.iter().map(|r| polyline_length(&r.points)).collect();

    // ── Roster counts + per-enemy speed/level lookups ─────────────────────
    let speed_of = |id: &str| -> f64 {
        enemies
            .enemy_data
            .get(id)
            .and_then(|e| e.stats.as_ref())
            .and_then(|s| s.levels.first())
            .map(|l| l.attributes.move_speed)
            .filter(|&v| v > 0.0)
            .unwrap_or(DEFAULT_SPEED)
    };
    let level_of = |id: &str| -> String {
        enemies.enemy_data.get(id).map_or_else(
            || "NORMAL".to_owned(),
            |e| level_str(&e.enemy_level).to_owned(),
        )
    };

    let mut counts: HashMap<String, u32> = HashMap::new();
    for r in &raw.enemy_db_refs {
        counts.entry(r.id.clone()).or_insert(0);
    }

    // ── Wave/spawn timeline (visual approximation of the battle scheduler) ─
    // Fragment preDelay is relative to its wave start; action preDelay is
    // relative to its fragment; a count>1 action emits one token per interval.
    // Waves advance after the wave's last spawn + maxTimeWaitingForNextWave.
    let mut spawns: Vec<SpawnOut> = Vec::new();
    let mut wave_markers: Vec<WaveMarker> = Vec::new();
    let mut clock = 0.0_f64;

    for (wi, wave) in raw.waves.iter().enumerate() {
        clock += wave.pre_delay;
        let wave_start = clock;
        let mut wave_end = wave_start;
        let mut wave_has_spawn = false;

        for frag in &wave.fragments {
            let frag_start = wave_start + frag.pre_delay;
            for action in &frag.actions {
                let Some(key) = action.key.as_deref() else {
                    continue;
                };
                if action.action_type != "SPAWN" || !key.starts_with("enemy_") {
                    continue;
                }
                let count = action.count.max(1);
                *counts.entry(key.to_owned()).or_insert(0) += count as u32;
                let route = usize::try_from(action.route_index.max(0)).unwrap_or(0);
                let speed = speed_of(key);
                let level = level_of(key);
                let action_start = frag_start + action.pre_delay;
                for k in 0..count {
                    let t0 = action_start + (k as f64) * action.interval;
                    wave_has_spawn = true;
                    let arrival =
                        t0 + route_len.get(route).copied().unwrap_or(0.0) / speed.max(0.1);
                    wave_end = wave_end.max(t0);
                    spawns.push(SpawnOut {
                        route,
                        enemy_id: key.to_owned(),
                        t0,
                        speed,
                        level: level.clone(),
                    });
                    clock = clock.max(arrival); // track latest arrival for duration
                }
            }
        }

        if wave_has_spawn {
            wave_markers.push(WaveMarker {
                at: wave_start,
                label: format!("Wave {}", wi + 1),
            });
        }
        // Advance the clock to the next wave's earliest start.
        clock = clock.max(wave_end + wave.max_time_waiting_for_next_wave + wave.post_delay);
    }

    // Duration: ensure the last enemy can finish its route, plus a small tail.
    let last_arrival = spawns
        .iter()
        .map(|s| s.t0 + route_len.get(s.route).copied().unwrap_or(0.0) / s.speed.max(0.1))
        .fold(0.0_f64, f64::max);
    let duration = last_arrival.max(clock).max(10.0) + 2.0;

    // ── Roster, sorted by threat then count ───────────────────────────────
    let mut roster: Vec<RosterEntry> = counts
        .into_iter()
        .map(|(id, count)| {
            let e = enemies.enemy_data.get(&id);
            RosterEntry {
                name: e.map_or_else(|| id.clone(), |e| e.name.clone()),
                level: e.map_or_else(|| "NORMAL".into(), |e| level_str(&e.enemy_level).to_owned()),
                damage_type: e
                    .map(|e| e.damage_type.iter().map(damage_str).collect())
                    .unwrap_or_default(),
                attack_type: e.and_then(|e| e.attack_type.clone()),
                motion: e.and_then(enemy_motion).unwrap_or_else(|| "WALK".into()),
                count,
                enemy_id: id,
            }
        })
        .collect();
    roster.sort_by(|a, b| {
        level_rank(&b.level)
            .cmp(&level_rank(&a.level))
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(StageMap {
        stage_id: stage.stage_id.clone(),
        level_id: level_id.to_owned(),
        code: stage.code.clone(),
        name: stage.name.clone(),
        width: cols,
        height: rows,
        tiles,
        routes,
        waves: wave_markers,
        spawns,
        roster,
        duration,
    })
}

// ============================================================================
// Helpers
// ============================================================================

fn tile_cell(t: &RawTile) -> TileCell {
    TileCell {
        kind: tile_kind(t),
        tile_key: t.tile_key.clone(),
        height_type: t.height_type.clone(),
        buildable: t.buildable_type.clone(),
        passable: t.passable_mask == "ALL" || t.passable_mask == "WALK_ONLY",
    }
}

fn empty_tile() -> TileCell {
    TileCell {
        kind: "empty".to_owned(),
        tile_key: String::new(),
        height_type: String::new(),
        buildable: String::new(),
        passable: false,
    }
}

/// Map an Arknights tile key to a normalized rendering kind. Falls back to the
/// height/buildable/passability when an exotic key isn't explicitly listed.
fn tile_kind(t: &RawTile) -> String {
    let kind = match t.tile_key.as_str() {
        "tile_start" | "tile_flystart" => "start",
        "tile_end" | "tile_flyend" => "end",
        "tile_road" => "road",
        "tile_floor" => "floor",
        "tile_wall" => "forbidden",
        "tile_forbidden" => "forbidden",
        "tile_hole" => "hole",
        "tile_telin" => "telein",
        "tile_telout" => "teleout",
        "tile_deepwater" | "tile_deepsea" => "deepwater",
        "tile_water" | "tile_shallow" | "tile_puddle" => "water",
        "tile_grass" => "grass",
        "tile_healing" | "tile_heal" => "heal",
        "tile_volcano" | "tile_volspread" | "tile_defbreak" | "tile_corrosion" | "tile_poison"
        | "tile_burning" | "tile_hazard" | "tile_icetrap" | "tile_smog" => "hazard",
        "tile_gazebo" => "high",
        _ => "",
    };
    if !kind.is_empty() {
        return kind.to_owned();
    }
    // Fallback by tile physics.
    if t.height_type == "HIGHLAND" {
        "high"
    } else if t.buildable_type == "NONE" && t.passable_mask.contains("FLY") {
        "forbidden"
    } else {
        "floor"
    }
    .to_owned()
}

fn motion_label(m: &str) -> String {
    match m {
        "WALK" => "WALK",
        "FLY" => "FLY",
        _ => "OTHER",
    }
    .to_owned()
}

const fn level_str(l: &EnemyLevel) -> &'static str {
    match l {
        EnemyLevel::Normal => "NORMAL",
        EnemyLevel::Elite => "ELITE",
        EnemyLevel::Boss => "BOSS",
    }
}

fn level_rank(l: &str) -> u8 {
    match l {
        "BOSS" => 2,
        "ELITE" => 1,
        _ => 0,
    }
}

fn damage_str(d: &DamageType) -> String {
    match d {
        DamageType::Physic => "PHYSIC",
        DamageType::Magic => "MAGIC",
        DamageType::NoDamage => "NO_DAMAGE",
        DamageType::Heal => "HEAL",
    }
    .to_owned()
}

fn enemy_motion(e: &Enemy) -> Option<String> {
    e.stats
        .as_ref()
        .and_then(|s| s.levels.first())
        .and_then(|l| l.motion.clone())
}

fn dedupe_consecutive(pts: &mut Vec<Point>) {
    pts.dedup_by(|a, b| (a.x - b.x).abs() < f64::EPSILON && (a.y - b.y).abs() < f64::EPSILON);
}

fn polyline_length(pts: &[Point]) -> f64 {
    pts.windows(2)
        .map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::enemy::EnemyHandbook;

    #[test]
    fn parses_main_01_07() {
        let path = "../assets/output/en/gamedata/levels/obt/main/level_main_01-07.json";
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => return, // asset not present in this checkout
        };
        let mut stage = Stage::default();
        stage.stage_id = "main_01-07".into();
        stage.code = "1-7".into();
        let map = parse_stage_map(
            &stage,
            "Obt/Main/level_main_01-07",
            &bytes,
            &EnemyHandbook::default(),
        )
        .unwrap();
        assert!(map.width > 0 && map.height > 0, "grid sized");
        assert_eq!(map.tiles.len(), map.height);
        assert_eq!(map.tiles[0].len(), map.width);
        assert!(!map.routes.is_empty(), "has routes");
        assert!(!map.spawns.is_empty(), "has enemy spawns");
        assert!(map.duration > 0.0);
        // start/end tiles should be discoverable
        let kinds: std::collections::HashSet<_> = map
            .tiles
            .iter()
            .flatten()
            .map(|t| t.kind.as_str())
            .collect();
        assert!(
            kinds.contains("start") && kinds.contains("end"),
            "kinds: {kinds:?}"
        );
        eprintln!(
            "main_01-07: {}x{} tiles, {} routes, {} spawns, {} roster, dur={:.1}s",
            map.width,
            map.height,
            map.routes.len(),
            map.spawns.len(),
            map.roster.len(),
            map.duration
        );
    }
}
