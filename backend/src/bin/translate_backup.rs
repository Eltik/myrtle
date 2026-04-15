//! Translate an old-myrtle backup directory into a v3 import bundle.
//!
//! Usage:
//!   cargo run --release --bin translate-backup -- --in <backup_dir> --out <out_dir>
//!
//! Then feed the result into v3:
//!   cargo run --release --bin import-database -- --in <out_dir> --truncate
//!
//! The old backend stored each user as a row with a giant `data` JSONB blob
//! holding the raw Arknights syncData response (status, troop, inventory,
//! skin, medal, dungeon, building, rlv2, deepSea, checkIn, ...). v3 normalizes
//! that blob into ~14 child tables. This binary reads the backup JSON arrays
//! (streaming users.json record-by-record because it can be multiple GB) and
//! writes the v3-shaped JSONL files plus a manifest that import-database
//! understands.
//!
//! What we preserve:
//!   - users + per-user UUIDs (so foreign keys survive)
//!   - roster (operators, skills, modules), inventory, skins
//!   - status / currencies, building, medals, stages, roguelike, sandbox, checkin
//!   - precomputed score breakdown (totalScore + per-pillar)
//!   - tier_lists, tiers, tier_placements, operator_notes, operator_notes_audit_log
//!   - gacha_records (renaming columns where they drift from v3)
//!   - user_settings (folded from old `settings` blob + user_gacha_settings)
//!
//! What we drop:
//!   - tier_change_log (no equivalent table in v3)
//!   - tier_list_reports (no equivalent in v3)
//!   - the giant raw `data` blob beyond what we extract (v3 reconstructs from
//!     normalized tables on demand)

use anyhow::{Context, Result, bail};
use backend::db_export::{FORMAT_VERSION, MANIFEST_FILE, TABLES};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

// ─── argv ────────────────────────────────────────────────────────────────────

struct Args {
    in_dir: PathBuf,
    out_dir: PathBuf,
}

fn parse_args() -> Result<Args> {
    let mut in_dir: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--in" | "-i" => in_dir = Some(PathBuf::from(it.next().context("--in needs path")?)),
            "--out" | "-o" => out_dir = Some(PathBuf::from(it.next().context("--out needs path")?)),
            "-h" | "--help" => {
                eprintln!(
                    "Usage: translate-backup --in <backup_dir> --out <out_dir>\n\
                     \n\
                     Reads the old myrtle PostgreSQL JSON dump in <backup_dir>\n\
                     and writes v3-shaped JSONL files + manifest.json to <out_dir>,\n\
                     ready to feed to `import-database --in <out_dir> --truncate`."
                );
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    Ok(Args {
        in_dir: in_dir.context("missing --in")?,
        out_dir: out_dir.context("missing --out")?,
    })
}

// ─── output writer ───────────────────────────────────────────────────────────

struct OutTable {
    writer: BufWriter<File>,
    rows: u64,
}

impl OutTable {
    fn new(path: &Path) -> Result<Self> {
        let f =
            File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
        Ok(Self {
            writer: BufWriter::with_capacity(1 << 20, f),
            rows: 0,
        })
    }

    fn write<S: Serialize>(&mut self, row: &S) -> Result<()> {
        serde_json::to_writer(&mut self.writer, row)?;
        self.writer.write_all(b"\n")?;
        self.rows += 1;
        Ok(())
    }

    fn finish(mut self) -> Result<u64> {
        self.writer.flush()?;
        Ok(self.rows)
    }
}

// ─── main ────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args = parse_args()?;
    fs::create_dir_all(&args.out_dir).context("failed to create out dir")?;

    let total_start = Instant::now();
    let mut counts: BTreeMap<&'static str, u64> = BTreeMap::new();

    // Open one writer per v3 table. Order matches db_export::TABLES.
    let mut tables: HashMap<&'static str, OutTable> = HashMap::new();
    for &t in TABLES {
        tables.insert(t, OutTable::new(&args.out_dir.join(format!("{t}.jsonl")))?);
    }

    // ── seed servers (v3 expects 0..5 to exist) ────────────────────────────
    {
        let w = tables.get_mut("servers").unwrap();
        for (id, code, name) in [
            (0i16, "EN", "Yostar"),
            (1, "JP", "Yostar JP"),
            (2, "KR", "Yostar KR"),
            (3, "CN", "Hypergryph"),
            (4, "Bili", "Bilibili"),
            (5, "TW", "Longcheng"),
        ] {
            w.write(&json!({"id": id, "code": code, "name": name}))?;
        }
    }

    // ── tier_lists ─────────────────────────────────────────────────────────
    let tier_lists: Vec<Value> = read_array(&args.in_dir.join("tier_lists.json"))?;
    let known_tier_list_ids: HashSet<String> = tier_lists
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();
    {
        let w = tables.get_mut("tier_lists").unwrap();
        for r in &tier_lists {
            // skip soft-deleted rows — v3 has no is_deleted column
            if r.get("is_deleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }
            let list_type = r
                .get("tier_list_type")
                .and_then(|v| v.as_str())
                .unwrap_or("community");
            w.write(&json!({
                "id": r.get("id"),
                "name": r.get("name"),
                "slug": r.get("slug"),
                "description": r.get("description"),
                "list_type": list_type,
                "created_by": r.get("created_by"),
                "is_active": r.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true),
                "created_at": r.get("created_at"),
                "updated_at": r.get("updated_at"),
            }))?;
        }
    }

    // ── tiers (collect FK universe) ────────────────────────────────────────
    let tiers: Vec<Value> = read_array(&args.in_dir.join("tiers.json"))?;
    let known_tier_ids: HashSet<String> = tiers
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();
    {
        let w = tables.get_mut("tiers").unwrap();
        for r in &tiers {
            // drop tiers whose parent list was soft-deleted out of our set
            let list_id = r.get("tier_list_id").and_then(|v| v.as_str()).unwrap_or("");
            if !known_tier_list_ids.contains(list_id) {
                continue;
            }
            w.write(&json!({
                "id": r.get("id"),
                "tier_list_id": r.get("tier_list_id"),
                "name": r.get("name"),
                "display_order": r.get("display_order"),
                "color": r.get("color"),
                "description": r.get("description"),
            }))?;
        }
    }

    // ── tier_placements (PK is (tier_id, operator_id) in v3, no `id`) ──────
    {
        let w = tables.get_mut("tier_placements").unwrap();
        let mut seen: HashSet<(String, String)> = HashSet::new();
        let placements: Vec<Value> = read_array(&args.in_dir.join("tier_placements.json"))?;
        for r in &placements {
            let tier_id = r.get("tier_id").and_then(|v| v.as_str()).unwrap_or("");
            let op_id = r.get("operator_id").and_then(|v| v.as_str()).unwrap_or("");
            if !known_tier_ids.contains(tier_id) {
                continue;
            }
            // dedupe — old schema's PK was an `id` UUID, v3 collapses to
            // (tier_id, operator_id). Take the first sighting to be deterministic.
            if !seen.insert((tier_id.to_string(), op_id.to_string())) {
                continue;
            }
            w.write(&json!({
                "tier_id": tier_id,
                "operator_id": op_id,
                "sub_order": r.get("sub_order").and_then(|v| v.as_i64()).unwrap_or(0),
                "notes": r.get("notes"),
                "updated_at": r.get("updated_at"),
            }))?;
        }
    }

    // tier_list_versions / tier_list_permissions: empty in source backups, so
    // their JSONL files stay empty (already created above).

    // ── operator_notes ─────────────────────────────────────────────────────
    let op_notes: Vec<Value> = read_array(&args.in_dir.join("operator_notes.json"))?;
    // old PK was operator_id-UUID per row; build operator_id→note_id map so
    // the audit log can resolve note_id from the old operator_id-keyed rows.
    let mut note_id_by_op: HashMap<String, String> = HashMap::new();
    {
        let w = tables.get_mut("operator_notes").unwrap();
        for r in &op_notes {
            let id = r
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default();
            let op = r
                .get("operator_id")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default();
            if !id.is_empty() && !op.is_empty() {
                note_id_by_op.insert(op.clone(), id.clone());
            }
            w.write(&json!({
                "id": r.get("id"),
                "operator_id": r.get("operator_id"),
                "pros": r.get("pros"),
                "cons": r.get("cons"),
                "notes": r.get("notes"),
                "trivia": r.get("trivia"),
                "summary": r.get("summary"),
                "tags": r.get("tags").cloned().unwrap_or_else(|| json!([])),
                "created_at": r.get("created_at"),
                "updated_at": r.get("updated_at"),
            }))?;
        }
    }

    // ── operator_notes_audit_log ───────────────────────────────────────────
    {
        let w = tables.get_mut("operator_notes_audit_log").unwrap();
        let audit: Vec<Value> = read_array(&args.in_dir.join("operator_notes_audit_log.json"))?;
        // v3 PK is BIGSERIAL — emit synthetic incrementing ids. Sequence is
        // reset post-import by import-database.
        let mut next_id: i64 = 1;
        for r in &audit {
            let op = r.get("operator_id").and_then(|v| v.as_str()).unwrap_or("");
            let Some(note_id) = note_id_by_op.get(op) else {
                continue; // orphan audit row — its parent note is gone
            };
            w.write(&json!({
                "id": next_id,
                "note_id": note_id,
                "field_name": r.get("field_name"),
                "old_value": r.get("old_value"),
                "new_value": r.get("new_value"),
                "changed_by": r.get("changed_by"),
                "changed_at": r.get("changed_at"),
            }))?;
            next_id += 1;
        }
    }

    // ── users + all child tables (streaming, multi-GB file) ────────────────
    println!("streaming users.json (this is the big one)...");
    let user_start = Instant::now();
    let mut user_ids: HashSet<String> = HashSet::new();
    let mut user_count: u64 = 0;
    stream_array(&args.in_dir.join("users.json"), |obj| -> Result<()> {
        let Some(uid) = obj.get("uid").and_then(|v| v.as_str()) else {
            return Ok(());
        };
        let Some(id) = obj.get("id").and_then(|v| v.as_str()) else {
            return Ok(());
        };
        user_ids.insert(id.to_string());

        let server_code = obj.get("server").and_then(|v| v.as_str()).unwrap_or("en");
        let server_id = server_code_to_id(server_code);
        let role = obj.get("role").and_then(|v| v.as_str()).unwrap_or("user");
        let data = obj.get("data");
        let status = data.and_then(|d| d.get("status"));

        let nickname = status
            .and_then(|s| s.get("nickName"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let level = status
            .and_then(|s| s.get("level"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let avatar_id = status
            .and_then(|s| s.get("avatar"))
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_str());
        let secretary = status
            .and_then(|s| s.get("secretary"))
            .and_then(|v| v.as_str());
        let secretary_skin_id = status
            .and_then(|s| s.get("secretarySkinId"))
            .and_then(|v| v.as_str());
        let resume_id = status
            .and_then(|s| s.get("resume"))
            .and_then(|v| v.as_str());

        // users
        tables.get_mut("users").unwrap().write(&json!({
            "id": id,
            "uid": uid,
            "server_id": server_id,
            "nickname": nickname,
            "level": level,
            "role": role,
            "avatar_id": avatar_id,
            "resume_id": resume_id,
            "secretary": secretary,
            "secretary_skin_id": secretary_skin_id,
            "created_at": obj.get("created_at"),
            "updated_at": obj.get("updated_at"),
        }))?;

        // user_settings — old `settings` blob was free-form; default to true
        let settings = obj.get("settings");
        tables.get_mut("user_settings").unwrap().write(&json!({
            "user_id": id,
            "public_profile": settings_bool(settings, "public_profile", true),
            "store_gacha": settings_bool(settings, "store_gacha", true),
            "share_stats": settings_bool(settings, "share_stats", true),
            "updated_at": obj.get("updated_at"),
        }))?;

        // user_status
        let status_row = build_status_row(id, status);
        tables.get_mut("user_status").unwrap().write(&status_row)?;

        // user_operators / skills / modules
        if let Some(troop) = data
            .and_then(|d| d.get("troop"))
            .and_then(|t| t.get("chars"))
            .and_then(|v| v.as_object())
        {
            let mut seen_ops: HashSet<String> = HashSet::new();
            for raw in troop.values() {
                let Some(char_id) = raw.get("charId").and_then(|v| v.as_str()) else {
                    continue;
                };
                if !seen_ops.insert(char_id.to_string()) {
                    continue; // dupe across instIds → keep first
                }
                tables.get_mut("user_operators").unwrap().write(&json!({
                    "user_id": id,
                    "operator_id": char_id,
                    "elite": raw.get("evolvePhase").and_then(|v| v.as_i64()).unwrap_or(0),
                    "level": raw.get("level").and_then(|v| v.as_i64()).unwrap_or(1),
                    "exp": raw.get("exp").and_then(|v| v.as_i64()).unwrap_or(0),
                    "potential": raw.get("potentialRank").and_then(|v| v.as_i64()).unwrap_or(0),
                    "skill_level": raw.get("mainSkillLvl").and_then(|v| v.as_i64()).unwrap_or(1),
                    "favor_point": raw.get("favorPoint").and_then(|v| v.as_i64()).unwrap_or(0),
                    "skin_id": raw.get("skin").and_then(|v| v.as_str()),
                    "default_skill": raw.get("defaultSkillIndex").and_then(|v| v.as_i64()).unwrap_or(0),
                    "voice_lan": raw.get("voiceLan").and_then(|v| v.as_str()),
                    "current_equip": raw.get("currentEquip").and_then(|v| v.as_str()),
                    "current_tmpl": raw.get("currentTmpl").and_then(|v| v.as_str()),
                    "obtained_at": raw.get("gainTime").and_then(|v| v.as_i64()).unwrap_or(0),
                }))?;

                if let Some(skills) = raw.get("skills").and_then(|v| v.as_array()) {
                    for (i, s) in skills.iter().enumerate() {
                        tables.get_mut("user_operator_skills").unwrap().write(&json!({
                            "user_id": id,
                            "operator_id": char_id,
                            "skill_index": i as i64,
                            "specialize_level": s.get("specializeLevel").and_then(|v| v.as_i64()).unwrap_or(0),
                        }))?;
                    }
                }

                if let Some(equip) = raw.get("equip").and_then(|v| v.as_object()) {
                    let mut seen_mods: HashSet<String> = HashSet::new();
                    for (mid, e) in equip {
                        if mid.starts_with("uniequip_001_") || !seen_mods.insert(mid.clone()) {
                            continue;
                        }
                        tables.get_mut("user_operator_modules").unwrap().write(&json!({
                            "user_id": id,
                            "operator_id": char_id,
                            "module_id": mid,
                            "module_level": e.get("level").and_then(|v| v.as_i64()).unwrap_or(0),
                            "locked": e.get("locked").and_then(|v| v.as_i64()).map(|n| n != 0).unwrap_or(false),
                        }))?;
                    }
                }
            }
        }

        // user_items — old myrtle stored inventory as {item_id: {amount, ...meta}}
        // (with the full ItemTable entry inlined). v3 only needs id+quantity.
        if let Some(inv) = data
            .and_then(|d| d.get("inventory"))
            .and_then(|v| v.as_object())
        {
            for (item_id, val) in inv {
                let q = val
                    .get("amount")
                    .and_then(|v| v.as_i64())
                    .or_else(|| val.as_i64())
                    .unwrap_or(0);
                if q <= 0 {
                    continue;
                }
                tables.get_mut("user_items").unwrap().write(&json!({
                    "user_id": id,
                    "item_id": item_id,
                    "quantity": q,
                }))?;
            }
        }

        // user_skins
        if let Some(skins) = data
            .and_then(|d| d.get("skin"))
            .and_then(|s| s.get("characterSkins"))
            .and_then(|v| v.as_object())
        {
            let mut seen_skins: HashSet<String> = HashSet::new();
            for (sid, e) in skins {
                if !seen_skins.insert(sid.clone()) {
                    continue;
                }
                tables.get_mut("user_skins").unwrap().write(&json!({
                    "user_id": id,
                    "skin_id": sid,
                    "obtained_at": e.get("obtainTime").and_then(|v| v.as_i64())
                        .or_else(|| e.get("obtainedAt").and_then(|v| v.as_i64()))
                        .unwrap_or(0),
                }))?;
            }
        }

        // user_stage_progress
        let stages = data
            .and_then(|d| d.get("dungeon"))
            .and_then(|d| d.get("stages"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        tables
            .get_mut("user_stage_progress")
            .unwrap()
            .write(&json!({
                "user_id": id,
                "stages": stages,
            }))?;

        // user_roguelike_progress (rlv2.outer keyed by theme_id)
        if let Some(outer) = data
            .and_then(|d| d.get("rlv2"))
            .and_then(|r| r.get("outer"))
            .and_then(|v| v.as_object())
        {
            let mut seen_themes: HashSet<String> = HashSet::new();
            for (theme_id, progress) in outer {
                if !seen_themes.insert(theme_id.clone()) {
                    continue;
                }
                tables
                    .get_mut("user_roguelike_progress")
                    .unwrap()
                    .write(&json!({
                        "user_id": id,
                        "theme_id": theme_id,
                        "progress": progress,
                    }))?;
            }
        }

        // user_sandbox_progress (prefer sandboxPerm, fall back to deepSea)
        let sandbox = data
            .and_then(|d| d.get("sandboxPerm"))
            .or_else(|| data.and_then(|d| d.get("deepSea")))
            .cloned()
            .unwrap_or_else(|| json!({}));
        tables
            .get_mut("user_sandbox_progress")
            .unwrap()
            .write(&json!({
                "user_id": id,
                "progress": sandbox,
            }))?;

        // user_medals
        if let Some(medals) = data
            .and_then(|d| d.get("medal"))
            .and_then(|m| m.get("medals"))
            .and_then(|v| v.as_object())
        {
            let mut seen_medals: HashSet<String> = HashSet::new();
            for (mid, e) in medals {
                if !seen_medals.insert(mid.clone()) {
                    continue;
                }
                tables.get_mut("user_medals").unwrap().write(&json!({
                    "user_id": id,
                    "medal_id": mid,
                    "val": e.get("val").cloned().unwrap_or(json!(null)),
                    "first_ts": e.get("fts").and_then(|v| v.as_i64()).unwrap_or(0),
                    "reach_ts": e.get("rts").and_then(|v| v.as_i64()).unwrap_or(0),
                }))?;
            }
        }

        // user_building (full blob)
        let building = data
            .and_then(|d| d.get("building"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        tables.get_mut("user_building").unwrap().write(&json!({
            "user_id": id,
            "data": building,
        }))?;

        // user_checkin (history array — try both shapes)
        let history = data
            .and_then(|d| d.get("checkIn"))
            .and_then(|c| c.get("history").or_else(|| c.get("checkInHistory")))
            .cloned()
            .unwrap_or_else(|| json!([]));
        tables.get_mut("user_checkin").unwrap().write(&json!({
            "user_id": id,
            "history": history,
        }))?;

        // user_scores (precomputed in old backend's score blob)
        if let Some(score) = obj.get("score").filter(|v| !v.is_null()) {
            let grade_str = score
                .get("grade")
                .and_then(|g| g.get("grade"))
                .and_then(|v| v.as_str())
                .map(String::from);
            let calculated_at = score
                .get("grade")
                .and_then(|g| g.get("calculatedAt"))
                .and_then(|v| v.as_i64())
                .map(epoch_to_iso)
                .unwrap_or_else(|| {
                    obj.get("updated_at")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
                });
            tables.get_mut("user_scores").unwrap().write(&json!({
                "user_id": id,
                "total_score": score.get("totalScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "operator_score": score.get("operatorScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "stage_score": score.get("stageScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "roguelike_score": score.get("roguelikeScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "sandbox_score": score.get("sandboxScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "medal_score": score.get("medalScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "base_score": score.get("baseScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "skin_score": score.get("skinScore").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "grade": grade_str,
                "calculated_at": calculated_at,
            }))?;
        }

        user_count += 1;
        if user_count.is_multiple_of(25) {
            println!("  …{user_count} users processed");
        }
        Ok(())
    })?;
    println!(
        "  finished {user_count} users in {:.2}s",
        user_start.elapsed().as_secs_f64()
    );

    // ── user_gacha_settings → fold into user_settings overrides ────────────
    // (already wrote one settings row per user above with defaults — overwrite
    //  by reading user_gacha_settings and rewriting only changed flags. Simpler
    //  to do this as a post-pass: read what we already wrote, patch, rewrite.)
    let gacha_settings: Vec<Value> = read_array(&args.in_dir.join("user_gacha_settings.json"))?;
    if !gacha_settings.is_empty() {
        // map user_id → (store_gacha, share_stats) from gacha settings
        let overrides: HashMap<String, (bool, bool)> = gacha_settings
            .iter()
            .filter_map(|r| {
                let u = r.get("user_id")?.as_str()?.to_string();
                let store = r
                    .get("store_records")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let share = r
                    .get("share_anonymous_stats")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                Some((u, (store, share)))
            })
            .collect();
        rewrite_user_settings(&args.out_dir, &overrides)?;
    }

    // ── gacha_records ──────────────────────────────────────────────────────
    {
        let w = tables.get_mut("gacha_records").unwrap();
        let mut next_id: i64 = 1;
        let mut seen: BTreeSet<(String, i64, String, String)> = BTreeSet::new();
        let records: Vec<Value> = read_array(&args.in_dir.join("gacha_records.json"))?;
        for r in &records {
            let user_id = r.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
            if !user_ids.contains(user_id) {
                continue; // orphan — FK would fail
            }
            let char_id = r
                .get("char_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pool_id = r
                .get("pool_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let pull_ts = r
                .get("pull_timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let key = (
                user_id.to_string(),
                pull_ts,
                char_id.clone(),
                pool_id.clone(),
            );
            if !seen.insert(key) {
                continue; // v3 has UNIQUE(user_id, pull_timestamp, char_id, pool_id)
            }
            w.write(&json!({
                "id": next_id,
                "user_id": user_id,
                "char_id": char_id,
                "pool_id": pool_id,
                "rarity": r.get("rarity").and_then(|v| v.as_i64()).unwrap_or(0),
                "pull_timestamp": pull_ts,
                "pool_name": r.get("pool_name"),
                "gacha_type": r.get("gacha_type"),
                "created_at": r.get("created_at"),
            }))?;
            next_id += 1;
        }
    }

    // audit_log stays empty (no equivalent rich source in old backup).

    // ── flush + manifest ───────────────────────────────────────────────────
    for &name in TABLES {
        let t = tables.remove(name).unwrap();
        let rows = t.finish()?;
        counts.insert(name, rows);
    }

    write_manifest(&args.out_dir, &counts)?;

    println!(
        "\nWrote {} tables, {} total rows in {:.2}s",
        counts.len(),
        counts.values().sum::<u64>(),
        total_start.elapsed().as_secs_f64()
    );
    for &t in TABLES {
        println!("  {t:<32} {:>10}", counts[t]);
    }
    Ok(())
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn server_code_to_id(code: &str) -> i16 {
    match code.to_ascii_lowercase().as_str() {
        "en" => 0,
        "jp" => 1,
        "kr" => 2,
        "cn" => 3,
        "bili" => 4,
        "tw" => 5,
        _ => 0,
    }
}

fn settings_bool(settings: Option<&Value>, key: &str, default: bool) -> bool {
    settings
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

fn build_status_row(user_id: &str, status: Option<&Value>) -> Value {
    let g = |k: &str| -> i64 {
        status
            .and_then(|s| s.get(k))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    };
    let gs = |k: &str| -> Option<String> {
        status
            .and_then(|s| s.get(k))
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    json!({
        "user_id": user_id,
        "exp": g("exp"),
        "orundum": g("diamondShard"),
        "orundum_shard": 0,
        "lmd": g("gold"),
        "sanity": g("ap"),
        "max_sanity": g("maxAp"),
        "gacha_tickets": g("gachaTicket"),
        "ten_pull_tickets": g("tenGachaTicket"),
        "classic_gacha_tickets": g("classicGachaTicket"),
        "classic_ten_pull_tickets": g("classicTenGachaTicket"),
        "recruit_permits": g("recruitLicense"),
        "social_point": g("socialPoint"),
        "hgg_shard": g("hggShard"),
        "lgg_shard": g("lggShard"),
        "practice_tickets": g("practiceTicket"),
        "gold": g("gold"),
        "monthly_sub_end": g("monthlySubscriptionEndTime"),
        "register_ts": g("registerTs"),
        "last_online_ts": g("lastOnlineTs"),
        "main_stage_progress": gs("mainStageProgress"),
        "resume": gs("resume"),
        "friend_num_limit": g("friendNumLimit"),
    })
}

fn epoch_to_iso(secs: i64) -> String {
    use chrono::TimeZone;
    chrono::Utc
        .timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Read a small/medium JSON-array file fully into memory. For multi-GB files
/// use `stream_array` instead.
fn read_array(path: &Path) -> Result<Vec<Value>> {
    let f = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut reader = BufReader::with_capacity(1 << 20, f);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let v: Value = serde_json::from_str(&buf)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    match v {
        Value::Array(a) => Ok(a),
        _ => bail!("{}: expected top-level JSON array", path.display()),
    }
}

/// Stream a JSON array of objects, yielding each object as `Value`. Memory
/// stays bounded to ~one object regardless of file size — necessary for
/// users.json which can be multiple GB.
fn stream_array<F>(path: &Path, mut on_item: F) -> Result<()>
where
    F: FnMut(Value) -> Result<()>,
{
    let f = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut reader = BufReader::with_capacity(8 << 20, f);
    let mut byte = [0u8; 1];

    // skip until '['
    loop {
        let n = reader.read(&mut byte)?;
        if n == 0 {
            return Ok(());
        }
        if byte[0] == b'[' {
            break;
        }
    }

    let mut buf: Vec<u8> = Vec::with_capacity(1 << 16);
    loop {
        // skip whitespace + commas to find next element start (or ']')
        let start_byte;
        loop {
            let n = reader.read(&mut byte)?;
            if n == 0 {
                return Ok(());
            }
            let b = byte[0];
            if b.is_ascii_whitespace() || b == b',' {
                continue;
            }
            if b == b']' {
                return Ok(());
            }
            start_byte = b;
            break;
        }

        buf.clear();
        buf.push(start_byte);
        // We expect each element to be an object or array. Track depth +
        // string state so we know when the element ends.
        let mut depth: i32 = if start_byte == b'{' || start_byte == b'[' {
            1
        } else {
            0
        };
        let mut in_string = start_byte == b'"';
        let mut escape = false;

        if depth == 0 && !in_string {
            // Scalar element: read until comma / whitespace / ']'.
            loop {
                let n = reader.read(&mut byte)?;
                if n == 0 {
                    break;
                }
                let b = byte[0];
                if b == b',' || b.is_ascii_whitespace() || b == b']' {
                    break;
                }
                buf.push(b);
            }
        } else {
            while depth > 0 || in_string {
                let n = reader.read(&mut byte)?;
                if n == 0 {
                    bail!("unexpected EOF inside JSON array element");
                }
                let b = byte[0];
                buf.push(b);
                if escape {
                    escape = false;
                    continue;
                }
                if in_string {
                    match b {
                        b'\\' => escape = true,
                        b'"' => in_string = false,
                        _ => {}
                    }
                } else {
                    match b {
                        b'"' => in_string = true,
                        b'{' | b'[' => depth += 1,
                        b'}' | b']' => depth -= 1,
                        _ => {}
                    }
                }
            }
        }

        let value: Value = serde_json::from_slice(&buf)
            .with_context(|| format!("failed to parse element in {}", path.display()))?;
        on_item(value)?;
    }
}

/// Patch user_settings.jsonl with overrides from user_gacha_settings.
fn rewrite_user_settings(out_dir: &Path, overrides: &HashMap<String, (bool, bool)>) -> Result<()> {
    let path = out_dir.join("user_settings.jsonl");
    let f = File::open(&path)?;
    let reader = BufReader::new(f);
    let mut rows: Vec<Value> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let mut v: Value = serde_json::from_str(&line)?;
        if let Some(uid) = v.get("user_id").and_then(|x| x.as_str())
            && let Some(&(store, share)) = overrides.get(uid)
            && let Some(map) = v.as_object_mut()
        {
            map.insert("store_gacha".into(), json!(store));
            map.insert("share_stats".into(), json!(share));
        }
        rows.push(v);
    }
    let mut w = BufWriter::new(File::create(&path)?);
    for r in rows {
        serde_json::to_writer(&mut w, &r)?;
        w.write_all(b"\n")?;
    }
    w.flush()?;
    Ok(())
}

fn write_manifest(out_dir: &Path, counts: &BTreeMap<&'static str, u64>) -> Result<()> {
    let exported_at = chrono::Utc::now().to_rfc3339();
    let table_entries: Vec<Value> = TABLES
        .iter()
        .map(|t| {
            json!({
                "name": t,
                "rows": counts.get(t).copied().unwrap_or(0),
                "file": format!("{t}.jsonl"),
            })
        })
        .collect();

    let manifest = json!({
        "format_version": FORMAT_VERSION,
        "exported_at": exported_at,
        "database_version": "translated-from-old-myrtle",
        "tables": table_entries,
    });

    let f = File::create(out_dir.join(MANIFEST_FILE))?;
    let mut w = BufWriter::new(f);
    serde_json::to_writer_pretty(&mut w, &manifest)?;
    w.flush()?;
    Ok(())
}
