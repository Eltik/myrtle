use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use chrono::TimeZone;
use myrtle_types::Server;
use serde_json::{Value, json};

use super::{DumpError, FORMAT_VERSION, Manifest, TABLES, TableEntry, file_name, write_manifest};

const DATABASE_VERSION: &str = "translated-from-old-myrtle";

/// Translate an old-myrtle JSON dump to import
pub fn translate(in_dir: &Path, out_dir: &Path) -> Result<Manifest, DumpError> {
    fs::create_dir_all(out_dir).map_err(DumpError::io(out_dir))?;
    let mut out = Out::open(out_dir)?;

    let tier_lists = read_array(&in_dir.join("tier_lists.json"))?;
    let mut known_lists: HashSet<String> = HashSet::new();
    for r in &tier_lists {
        if bool_at(r, "is_deleted").unwrap_or(false) {
            continue;
        }
        if let Some(id) = str_at(r, "id") {
            known_lists.insert(id.to_owned());
        }
        out.write(
            "tier_lists",
            &json!({
                "id": r.get("id"),
                "name": r.get("name"),
                "slug": r.get("slug"),
                "description": r.get("description"),
                "list_type": str_at(r, "tier_list_type").unwrap_or("community"),
                "created_by": r.get("created_by"),
                "is_active": bool_at(r, "is_active").unwrap_or(true),
                "created_at": r.get("created_at"),
                "updated_at": r.get("updated_at"),
            }),
        )?;
    }

    let mut known_tiers: HashSet<String> = HashSet::new();
    for r in &read_array(&in_dir.join("tiers.json"))? {
        if !known_lists.contains(str_at(r, "tier_list_id").unwrap_or("")) {
            continue;
        }
        if let Some(id) = str_at(r, "id") {
            known_tiers.insert(id.to_owned());
        }
        out.write(
            "tiers",
            &json!({
                "id": r.get("id"),
                "tier_list_id": r.get("tier_list_id"),
                "name": r.get("name"),
                "display_order": r.get("display_order"),
                "color": r.get("color"),
                "description": r.get("description"),
            }),
        )?;
    }

    let mut placed: HashSet<(String, String)> = HashSet::new();
    for r in &read_array(&in_dir.join("tier_placements.json"))? {
        let tier_id = str_at(r, "tier_id").unwrap_or("");
        let op_id = str_at(r, "operator_id").unwrap_or("");
        if !known_tiers.contains(tier_id) || !placed.insert((tier_id.to_owned(), op_id.to_owned()))
        {
            continue;
        }
        out.write(
            "tier_placements",
            &json!({
                "tier_id": tier_id,
                "entity_kind": "operator",
                "entity_id": op_id,
                "sub_order": int_at(r, "sub_order").unwrap_or(0),
                "description": r.get("notes"),
                "updated_at": r.get("updated_at"),
            }),
        )?;
    }

    let mut note_by_operator: HashMap<String, String> = HashMap::new();
    for r in &read_array(&in_dir.join("operator_notes.json"))? {
        if let (Some(id), Some(op)) = (str_at(r, "id"), str_at(r, "operator_id"))
            && !id.is_empty()
            && !op.is_empty()
        {
            note_by_operator.insert(op.to_owned(), id.to_owned());
        }
        out.write(
            "operator_notes",
            &json!({
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
            }),
        )?;
    }

    let mut next_id: i64 = 1;
    for r in &read_array(&in_dir.join("operator_notes_audit_log.json"))? {
        let Some(note_id) = note_by_operator.get(str_at(r, "operator_id").unwrap_or("")) else {
            continue;
        };
        out.write(
            "operator_notes_audit_log",
            &json!({
                "id": next_id,
                "note_id": note_id,
                "field_name": r.get("field_name"),
                "old_value": r.get("old_value"),
                "new_value": r.get("new_value"),
                "changed_by": r.get("changed_by"),
                "changed_at": r.get("changed_at"),
            }),
        )?;
        next_id += 1;
    }

    let gacha_settings: HashMap<String, (bool, bool)> =
        read_array(&in_dir.join("user_gacha_settings.json"))?
            .iter()
            .filter_map(|r| {
                Some((
                    str_at(r, "user_id")?.to_owned(),
                    (
                        bool_at(r, "store_records").unwrap_or(true),
                        bool_at(r, "share_anonymous_stats").unwrap_or(true),
                    ),
                ))
            })
            .collect();

    tracing::info!("streaming users.json");
    let mut user_ids: HashSet<String> = HashSet::new();
    let mut users: u64 = 0;
    stream_array(&in_dir.join("users.json"), |user| {
        if let Some(id) = write_user(&mut out, &user, &gacha_settings)? {
            user_ids.insert(id);
            users += 1;
            if users.is_multiple_of(1000) {
                tracing::info!(users, "users translated");
            }
        }
        Ok(())
    })?;
    tracing::info!(users, "users translated");

    let mut next_id: i64 = 1;
    let mut pulls: BTreeSet<(String, i64, String, String)> = BTreeSet::new();
    for r in &read_array(&in_dir.join("gacha_records.json"))? {
        let user_id = str_at(r, "user_id").unwrap_or("");
        if !user_ids.contains(user_id) {
            continue; // orphan
        }
        let char_id = str_at(r, "char_id").unwrap_or("");
        let pool_id = str_at(r, "pool_id").unwrap_or("");
        let pull_ts = int_at(r, "pull_timestamp").unwrap_or(0);
        if !pulls.insert((
            user_id.to_owned(),
            pull_ts,
            char_id.to_owned(),
            pool_id.to_owned(),
        )) {
            continue;
        }
        out.write(
            "gacha_records",
            &json!({
                "id": next_id,
                "user_id": user_id,
                "char_id": char_id,
                "pool_id": pool_id,
                "rarity": int_at(r, "rarity").unwrap_or(0),
                "pull_timestamp": pull_ts,
                "pool_name": r.get("pool_name"),
                "gacha_type": r.get("gacha_type"),
                "created_at": r.get("created_at"),
            }),
        )?;
        next_id += 1;
    }

    let manifest = Manifest {
        format_version: FORMAT_VERSION,
        exported_at: chrono::Utc::now().to_rfc3339(),
        database_version: DATABASE_VERSION.to_owned(),
        tables: out.finish()?,
    };
    write_manifest(out_dir, &manifest)?;
    Ok(manifest)
}

fn write_user(
    out: &mut Out,
    user: &Value,
    gacha_settings: &HashMap<String, (bool, bool)>,
) -> Result<Option<String>, DumpError> {
    let (Some(uid), Some(id)) = (str_at(user, "uid"), str_at(user, "id")) else {
        return Ok(None);
    };
    let data = user.get("data");
    let section = |key: &str| data.and_then(|d| d.get(key));
    let status = section("status");
    let status_str = |key: &str| status.and_then(|s| str_at(s, key));

    out.write(
        "users",
        &json!({
            "id": id,
            "uid": uid,
            "server_id": Server::parse(str_at(user, "server").unwrap_or("en")).map_or(0, Server::index),
            "nickname": status_str("nickName").unwrap_or(""),
            "nick_number": status_str("nickNumber"),
            "level": status.and_then(|s| int_at(s, "level")).unwrap_or(0),
            "role": str_at(user, "role").unwrap_or("user"),
            "avatar_id": status.and_then(|s| s.get("avatar")).and_then(|a| str_at(a, "id")),
            "resume_id": status_str("resume"),
            "secretary": status_str("secretary"),
            "secretary_skin_id": status_str("secretarySkinId"),
            "created_at": user.get("created_at"),
            "updated_at": user.get("updated_at"),
        }),
    )?;

    let settings = user.get("settings");
    let setting = |key: &str| settings.and_then(|s| bool_at(s, key)).unwrap_or(true);
    let (store_gacha, share_stats) = gacha_settings
        .get(id)
        .copied()
        .unwrap_or_else(|| (setting("store_gacha"), setting("share_stats")));
    out.write(
        "user_settings",
        &json!({
            "user_id": id,
            "public_profile": setting("public_profile"),
            "store_gacha": store_gacha,
            "share_stats": share_stats,
            "updated_at": user.get("updated_at"),
        }),
    )?;
    out.write("user_status", &status_row(id, status))?;

    if let Some(chars) = section("troop")
        .and_then(|t| t.get("chars"))
        .and_then(Value::as_object)
    {
        let mut seen: HashSet<&str> = HashSet::new();
        for raw in chars.values() {
            let Some(char_id) = str_at(raw, "charId") else {
                continue;
            };
            if !seen.insert(char_id) {
                continue;
            }
            out.write(
                "user_operators",
                &json!({
                    "user_id": id,
                    "operator_id": char_id,
                    "elite": int_at(raw, "evolvePhase").unwrap_or(0),
                    "level": int_at(raw, "level").unwrap_or(1),
                    "exp": int_at(raw, "exp").unwrap_or(0),
                    "potential": int_at(raw, "potentialRank").unwrap_or(0),
                    "skill_level": int_at(raw, "mainSkillLvl").unwrap_or(1),
                    "favor_point": int_at(raw, "favorPoint").unwrap_or(0),
                    "skin_id": str_at(raw, "skin"),
                    "default_skill": int_at(raw, "defaultSkillIndex").unwrap_or(0),
                    "voice_lan": str_at(raw, "voiceLan"),
                    "current_equip": str_at(raw, "currentEquip"),
                    "current_tmpl": str_at(raw, "currentTmpl"),
                    "obtained_at": int_at(raw, "gainTime").unwrap_or(0),
                }),
            )?;
            for (i, skill) in raw
                .get("skills")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
            {
                out.write(
                    "user_operator_skills",
                    &json!({
                        "user_id": id,
                        "operator_id": char_id,
                        "skill_index": i,
                        "specialize_level": int_at(skill, "specializeLevel").unwrap_or(0),
                    }),
                )?;
            }
            for (module_id, e) in raw
                .get("equip")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
            {
                if module_id.starts_with("uniequip_001_") {
                    continue;
                }
                out.write(
                    "user_operator_modules",
                    &json!({
                        "user_id": id,
                        "operator_id": char_id,
                        "module_id": module_id,
                        "module_level": int_at(e, "level").unwrap_or(0),
                        "locked": int_at(e, "locked").is_some_and(|n| n != 0),
                    }),
                )?;
            }
        }
    }

    for (item_id, v) in section("inventory")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let quantity = int_at(v, "amount").or_else(|| v.as_i64()).unwrap_or(0);
        if quantity > 0 {
            out.write(
                "user_items",
                &json!({"user_id": id, "item_id": item_id, "quantity": quantity}),
            )?;
        }
    }

    for (skin_id, e) in section("skin")
        .and_then(|s| s.get("characterSkins"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        out.write(
            "user_skins",
            &json!({
                "user_id": id,
                "skin_id": skin_id,
                "obtained_at": int_at(e, "obtainTime").or_else(|| int_at(e, "obtainedAt")).unwrap_or(0),
            }),
        )?;
    }

    let stages = section("dungeon")
        .and_then(|d| d.get("stages"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    out.write(
        "user_stage_progress",
        &json!({"user_id": id, "stages": stages}),
    )?;

    for (theme_id, progress) in section("rlv2")
        .and_then(|r| r.get("outer"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        out.write(
            "user_roguelike_progress",
            &json!({"user_id": id, "theme_id": theme_id, "progress": progress}),
        )?;
    }

    let sandbox = section("sandboxPerm")
        .or_else(|| section("deepSea"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    out.write(
        "user_sandbox_progress",
        &json!({"user_id": id, "progress": sandbox}),
    )?;

    for (medal_id, e) in section("medal")
        .and_then(|m| m.get("medals"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        out.write(
            "user_medals",
            &json!({
                "user_id": id,
                "medal_id": medal_id,
                "val": e.get("val").cloned().unwrap_or(Value::Null),
                "first_ts": int_at(e, "fts").unwrap_or(0),
                "reach_ts": int_at(e, "rts").unwrap_or(0),
            }),
        )?;
    }

    let building = section("building").cloned().unwrap_or_else(|| json!({}));
    out.write("user_building", &json!({"user_id": id, "data": building}))?;

    let history = section("checkIn")
        .and_then(|c| c.get("history").or_else(|| c.get("checkInHistory")))
        .cloned()
        .unwrap_or_else(|| json!([]));
    out.write("user_checkin", &json!({"user_id": id, "history": history}))?;

    if let Some(score) = user.get("score").filter(|v| !v.is_null()) {
        let grade = score.get("grade");
        let calculated_at = grade.and_then(|g| int_at(g, "calculatedAt")).map_or_else(
            || str_at(user, "updated_at").map_or_else(|| EPOCH.to_owned(), str::to_owned),
            epoch_to_iso,
        );
        let pillar = |key: &str| score.get(key).and_then(Value::as_f64).unwrap_or(0.0);
        out.write(
            "user_scores",
            &json!({
                "user_id": id,
                "total_score": pillar("totalScore"),
                "operator_score": pillar("operatorScore"),
                "stage_score": pillar("stageScore"),
                "roguelike_score": pillar("roguelikeScore"),
                "sandbox_score": pillar("sandboxScore"),
                "medal_score": pillar("medalScore"),
                "base_score": pillar("baseScore"),
                "skin_score": pillar("skinScore"),
                "grade": grade.and_then(|g| str_at(g, "grade")),
                "calculated_at": calculated_at,
            }),
        )?;
    }
    Ok(Some(id.to_owned()))
}

const EPOCH: &str = "1970-01-01T00:00:00Z";

fn status_row(user_id: &str, status: Option<&Value>) -> Value {
    let n = |k: &str| status.and_then(|s| int_at(s, k)).unwrap_or(0);
    let s = |k: &str| status.and_then(|s| str_at(s, k));
    json!({
        "user_id": user_id,
        "exp": n("exp"),
        "orundum": n("diamondShard"),
        "orundum_shard": 0,
        "lmd": n("gold"),
        "sanity": n("ap"),
        "max_sanity": n("maxAp"),
        "gacha_tickets": n("gachaTicket"),
        "ten_pull_tickets": n("tenGachaTicket"),
        "classic_gacha_tickets": n("classicGachaTicket"),
        "classic_ten_pull_tickets": n("classicTenGachaTicket"),
        "recruit_permits": n("recruitLicense"),
        "social_point": n("socialPoint"),
        "hgg_shard": n("hggShard"),
        "lgg_shard": n("lggShard"),
        "practice_tickets": n("practiceTicket"),
        "gold": n("gold"),
        "monthly_sub_end": n("monthlySubscriptionEndTime"),
        "register_ts": n("registerTs"),
        "last_online_ts": n("lastOnlineTs"),
        "main_stage_progress": s("mainStageProgress"),
        "resume": s("resume"),
        "friend_num_limit": n("friendNumLimit"),
    })
}

fn epoch_to_iso(secs: i64) -> String {
    chrono::Utc
        .timestamp_opt(secs, 0)
        .single()
        .map_or_else(|| EPOCH.to_owned(), |t| t.to_rfc3339())
}

fn str_at<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

fn int_at(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(Value::as_i64)
}

fn bool_at(v: &Value, key: &str) -> Option<bool> {
    v.get(key).and_then(Value::as_bool)
}

struct Out {
    dir: PathBuf,
    tables: HashMap<&'static str, (BufWriter<File>, u64)>,
}

impl Out {
    fn open(dir: &Path) -> Result<Self, DumpError> {
        let mut tables = HashMap::new();
        for &t in TABLES {
            let path = dir.join(file_name(t));
            let file = File::create(&path).map_err(DumpError::io(&path))?;
            tables.insert(t, (BufWriter::with_capacity(1 << 20, file), 0));
        }
        Ok(Self {
            dir: dir.to_owned(),
            tables,
        })
    }

    fn write(&mut self, table: &str, row: &Value) -> Result<(), DumpError> {
        let path = self.dir.join(file_name(table));
        let (w, rows) = self
            .tables
            .get_mut(table)
            .ok_or_else(|| DumpError::Invalid(format!("no table {table}")))?;
        serde_json::to_writer(&mut *w, row).map_err(DumpError::json(&path))?;
        w.write_all(b"\n").map_err(DumpError::io(&path))?;
        *rows += 1;
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<TableEntry>, DumpError> {
        let mut entries = Vec::with_capacity(TABLES.len());
        for &t in TABLES {
            if let Some((mut w, rows)) = self.tables.remove(t) {
                w.flush()
                    .map_err(DumpError::io(&self.dir.join(file_name(t))))?;
                entries.push(TableEntry::new(t, rows));
            }
        }
        Ok(entries)
    }
}

fn read_array(path: &Path) -> Result<Vec<Value>, DumpError> {
    let file = File::open(path).map_err(DumpError::io(path))?;
    match serde_json::from_reader(BufReader::with_capacity(1 << 20, file))
        .map_err(DumpError::json(path))?
    {
        Value::Array(rows) => Ok(rows),
        _ => Err(DumpError::Invalid(format!(
            "{}: expected a top-level JSON array",
            path.display()
        ))),
    }
}

pub fn stream_array(
    path: &Path,
    mut on_item: impl FnMut(Value) -> Result<(), DumpError>,
) -> Result<(), DumpError> {
    let file = File::open(path).map_err(DumpError::io(path))?;
    let mut bytes = BufReader::with_capacity(8 << 20, file).bytes();
    let mut next = || bytes.next().transpose().map_err(DumpError::io(path));
    let eof = || DumpError::Invalid(format!("{}: unexpected end of file", path.display()));

    loop {
        match next()? {
            None => return Err(eof()),
            Some(b'[') => break,
            Some(b) if b.is_ascii_whitespace() => {}
            Some(_) => {
                return Err(DumpError::Invalid(format!(
                    "{}: expected a top-level JSON array",
                    path.display()
                )));
            }
        }
    }

    let mut buf: Vec<u8> = Vec::with_capacity(1 << 16);
    loop {
        let first = loop {
            match next()? {
                None => return Err(eof()),
                Some(b']') => return Ok(()),
                Some(b) if b.is_ascii_whitespace() || b == b',' => {}
                Some(b) => break b,
            }
        };
        buf.clear();
        buf.push(first);
        let mut depth = u32::from(first == b'{' || first == b'[');
        let mut in_string = first == b'"';
        let mut escape = false;
        if depth == 0 && !in_string {
            let mut closed = false;
            while let Some(b) = next()? {
                if b == b']' {
                    closed = true;
                    break;
                }
                if b == b',' || b.is_ascii_whitespace() {
                    break;
                }
                buf.push(b);
            }
            on_item(serde_json::from_slice(&buf).map_err(DumpError::json(path))?)?;
            if closed {
                return Ok(());
            }
            continue;
        }
        while depth > 0 || in_string {
            let b = next()?.ok_or_else(eof)?;
            buf.push(b);
            if escape {
                escape = false;
            } else if in_string {
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
        on_item(serde_json::from_slice(&buf).map_err(DumpError::json(path))?)?;
    }
}
