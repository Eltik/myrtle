use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub route: String,
    pub param: String,
    pub kind: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdSet {
    #[serde(default)]
    pub kinds: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<Binding>,
}

pub const SERVERS: &[&str] = &["en", "jp", "kr", "cn", "tw"];
pub const STATIC_RESOURCES: &[&str] = &[
    "operators",
    "skills",
    "modules",
    "skins",
    "materials",
    "stages",
    "zones",
    "activities",
    "retro_acts",
    "enemies",
    "enemy-stages",
    "enemy-stage-table",
    "stage-index",
    "gacha",
    "banners",
    "voices",
    "handbook",
    "chibis",
    "enemy-chibis",
    "trust",
    "ranges",
];

pub const IMAGE_SIZES: &[&str] = &["thumb", "header"];
pub const MANUAL_KINDS: &[&str] = &[
    "announcement",
    "asset_path",
    "grid_slug",
    "i18n_hash",
    "i18n_key",
    "i18n_namespace",
    "locale",
    "lookback_snapshot",
    "music_track",
    "search_query",
    "siren_album",
    "siren_song",
    "sprite",
    "sprite_base",
    "sprite_variant",
    "story_art",
    "story_gallery",
    "tier_list_slug",
    "uid",
];

impl IdSet {
    pub fn values(&self, kind: &str) -> &[String] {
        self.kinds.get(kind).map_or(&[], Vec::as_slice)
    }

    pub fn kind_for(&self, route: &str, param: &str) -> Option<String> {
        self.bindings
            .iter()
            .find(|b| b.route == route && b.param == param)
            .map(|b| b.kind.clone())
            .or_else(|| builtin_kind(route, param).map(str::to_owned))
    }

    pub fn merge(&mut self, other: Self) {
        for (kind, values) in other.kinds {
            let slot = self.kinds.entry(kind).or_default();
            slot.extend(values);
        }
        for b in other.bindings {
            self.bindings
                .retain(|x| !(x.route == b.route && x.param == b.param));
            self.bindings.push(b);
        }
        self.normalize();
    }

    pub fn normalize(&mut self) {
        for values in self.kinds.values_mut() {
            let set: BTreeSet<String> = values.drain(..).filter(|v| !v.is_empty()).collect();
            values.extend(set);
        }
        self.bindings
            .sort_by(|a, b| (&a.route, &a.param).cmp(&(&b.route, &b.param)));
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading ids file {}", path.display()))?;
        let mut ids: Self = serde_json::from_str(&text)
            .with_context(|| format!("parsing ids file {}", path.display()))?;
        ids.normalize();
        Ok(ids)
    }
}

fn literal_before(route: &str, param: &str) -> Option<&'static str> {
    let segs: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
    let pos = segs
        .iter()
        .position(|s| *s == format!("{{{param}}}") || *s == format!("{{*{param}}}"))?;
    let lit = segs[..pos].iter().rev().find(|s| !s.starts_with('{'))?;
    KNOWN_LITERALS.iter().copied().find(|k| k == lit)
}

const KNOWN_LITERALS: &[&str] = &[
    "operators",
    "skins",
    "voices",
    "portrait",
    "charart",
    "avatar",
    "chibis",
    "enemies",
    "enemy-icon",
    "skill-icon",
    "module-icon",
    "module-big",
    "item-icon",
    "medal-icon",
    "skin-portrait",
    "announcements",
    "story-sprite-thumb",
    "gallery",
    "art-gallery",
    "albums",
    "songs",
    "tier-lists",
    "grids",
    "snapshots",
];

pub fn builtin_kind(route: &str, param: &str) -> Option<&'static str> {
    let by_name = match param {
        "server" => Some("server"),
        "char_id" | "operator_id" => Some("operator"),
        "stage_id" => Some("stage"),
        "story_id" => Some("story"),
        "group_id" => Some("story_group"),
        "pool_id" => Some("gacha_pool"),
        "kv_id" => Some("brand_kv"),
        "brand_id" => Some("brand"),
        "act_id" => Some("activity"),
        "resource" => Some("static_resource"),
        "track_id" => Some("music_track"),
        "locale" => Some("locale"),
        "namespace" => Some("i18n_namespace"),
        "hash" => Some("i18n_hash"),
        "base" => Some("sprite_base"),
        "variant" => Some("sprite_variant"),
        "path" => Some("asset_path"),
        "size" => Some("image_size"),
        "uid" => Some("uid"),
        "item" => Some("item"),
        "key" => Some("i18n_key"),
        "q" => Some("search_query"),
        _ => None,
    };
    if by_name.is_some() {
        return by_name;
    }
    let lit = literal_before(route, param)?;
    match (param, lit) {
        ("id", "operators" | "skins" | "voices" | "portrait" | "charart" | "avatar" | "chibis") => {
            Some("operator")
        }
        ("id", "enemies" | "enemy-icon") => Some("enemy"),
        ("id", "skill-icon") => Some("skill_icon"),
        ("id", "module-icon" | "module-big") => Some("module_icon"),
        ("id", "item-icon") => Some("item"),
        ("id", "medal-icon") => Some("medal"),
        ("id", "skin-portrait") => Some("skin"),
        ("id", "announcements") => Some("announcement"),
        ("id", "story-sprite-thumb") => Some("sprite"),
        ("id", "gallery") => Some("story_gallery"),
        ("id", "art-gallery") => Some("story_art"),
        ("cid", "albums") => Some("siren_album"),
        ("cid", "songs") => Some("siren_song"),
        ("slug", "tier-lists") => Some("tier_list_slug"),
        ("slug", "grids") => Some("grid_slug"),
        ("id", "snapshots") => Some("lookback_snapshot"),
        _ => None,
    }
}

pub fn sanitize_lone_surrogates(text: &str) -> Cow<'_, str> {
    if !text.contains("\\u") {
        return Cow::Borrowed(text);
    }
    let bytes = text.as_bytes();
    let mut out: Option<String> = None;
    let mut copied = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' || i + 1 >= bytes.len() {
            i += 1;
            continue;
        }
        if bytes[i + 1] != b'u' {
            i += 2;
            continue;
        }
        let Some(unit) = hex4(bytes, i + 2) else {
            i += 2;
            continue;
        };
        let is_high = (0xD800..=0xDBFF).contains(&unit);
        let is_low = (0xDC00..=0xDFFF).contains(&unit);
        if is_high {
            let paired = bytes.get(i + 6) == Some(&b'\\')
                && bytes.get(i + 7) == Some(&b'u')
                && hex4(bytes, i + 8).is_some_and(|n| (0xDC00..=0xDFFF).contains(&n));
            if paired {
                i += 12;
                continue;
            }
        }
        if is_high || is_low {
            let buf = out.get_or_insert_with(|| String::with_capacity(text.len()));
            buf.push_str(&text[copied..i]);
            buf.push_str("\\uFFFD");
            copied = i + 6;
        }
        i += 6;
    }
    match out {
        None => Cow::Borrowed(text),
        Some(mut buf) => {
            buf.push_str(&text[copied..]);
            Cow::Owned(buf)
        }
    }
}

fn hex4(bytes: &[u8], at: usize) -> Option<u32> {
    let s = std::str::from_utf8(bytes.get(at..at + 4)?).ok()?;
    u32::from_str_radix(s, 16).ok()
}

fn norm_key(k: &str) -> String {
    k.chars()
        .filter(|c| *c != '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn field<'a>(v: &'a Value, name: &str) -> Option<&'a Value> {
    let want = norm_key(name);
    v.as_object()?
        .iter()
        .find(|(k, _)| norm_key(k) == want)
        .map(|(_, v)| v)
}

fn str_field<'a>(v: &'a Value, name: &str) -> Option<&'a str> {
    field(v, name)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn entries(v: &Value) -> Vec<(String, &Value)> {
    match v {
        Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v)).collect(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                let key = field(item, "key")?;
                let key = match key {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                Some((key, field(item, "value").unwrap_or(&Value::Null)))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn list(v: Option<&Value>) -> &[Value] {
    v.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

#[derive(Debug, Clone, Default)]
pub struct ExtractReport {
    pub tables_read: Vec<String>,
    pub tables_missing: Vec<String>,
}

fn read_table(dir: &Path, name: &str, report: &mut ExtractReport) -> Result<Option<Value>> {
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        report.tables_missing.push(name.to_owned());
        return Ok(None);
    }
    let raw = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let text = String::from_utf8_lossy(&raw);
    let text = sanitize_lone_surrogates(&text);
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    report.tables_read.push(name.to_owned());
    Ok(Some(value))
}

pub fn extract_from_excel(dir: &Path) -> Result<(IdSet, ExtractReport)> {
    let mut report = ExtractReport::default();
    let mut kinds: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut add = |kind: &str, value: String| kinds.entry(kind.to_owned()).or_default().push(value);

    if let Some(t) = read_table(dir, "character_table", &mut report)? {
        let map = field(&t, "characters").unwrap_or(&t);
        for (id, _) in entries(map) {
            if id.starts_with("char_") {
                add("operator", id);
            }
        }
    }
    if let Some(t) = read_table(dir, "char_patch_table", &mut report)?
        && let Some(map) = field(&t, "patchChars")
    {
        for (id, _) in entries(map) {
            add("operator", id);
        }
    }
    if let Some(t) = read_table(dir, "skin_table", &mut report)? {
        if let Some(map) = field(&t, "charSkins") {
            for (id, _) in entries(map) {
                add("skin", id);
            }
        }
        if let Some(map) = field(&t, "brandList") {
            for (id, brand) in entries(map) {
                for kv in list(field(brand, "kvImgIdList")) {
                    if let Some(kv_id) = str_field(kv, "kvImgId") {
                        add("brand_kv", kv_id.to_owned());
                    }
                }
                add(
                    "brand",
                    str_field(brand, "brandId").map_or(id, str::to_owned),
                );
            }
        }
    }
    if let Some(t) = read_table(dir, "skill_table", &mut report)? {
        let map = field(&t, "skills").unwrap_or(&t);
        for (id, skill) in entries(map) {
            add(
                "skill_icon",
                str_field(skill, "iconId").map_or(id, str::to_owned),
            );
        }
    }
    if let Some(t) = read_table(dir, "uniequip_table", &mut report)?
        && let Some(map) = field(&t, "equipDict")
    {
        for (_, module) in entries(map) {
            if let Some(icon) = str_field(module, "uniEquipIcon") {
                add("module_icon", icon.to_owned());
            }
        }
    }
    if let Some(t) = read_table(dir, "item_table", &mut report)?
        && let Some(map) = field(&t, "items")
    {
        for (id, _) in entries(map) {
            add("item", id);
        }
    }
    if let Some(t) = read_table(dir, "medal_table", &mut report)? {
        for medal in list(field(&t, "medalList")) {
            if let Some(id) = str_field(medal, "medalId") {
                add("medal", id.to_owned());
            }
        }
    }
    if let Some(t) = read_table(dir, "enemy_handbook_table", &mut report)? {
        let map = field(&t, "enemyData").unwrap_or(&t);
        for (id, _) in entries(map) {
            if id.starts_with("enemy_") {
                add("enemy", id);
            }
        }
    }
    if let Some(t) = read_table(dir, "stage_table", &mut report)?
        && let Some(map) = field(&t, "stages")
    {
        for (id, _) in entries(map) {
            add("stage", id);
        }
    }
    if let Some(t) = read_table(dir, "story_review_table", &mut report)? {
        let map = field(&t, "storyReviews")
            .or_else(|| field(&t, "storyreviewtable"))
            .unwrap_or(&t);
        for (id, group) in entries(map) {
            let entry_type = str_field(group, "entryType").unwrap_or_default();
            if matches!(entry_type, "ACTIVITY" | "MINI_ACTIVITY") {
                add("activity", id.clone());
            }
            for story in list(field(group, "infoUnlockDatas")) {
                if let Some(story_id) = str_field(story, "storyId") {
                    add("story", story_id.to_owned());
                }
            }
            add("story_group", id);
        }
    }
    if let Some(t) = read_table(dir, "gacha_table", &mut report)? {
        for pool in list(field(&t, "gachaPoolClient")) {
            if let Some(id) = str_field(pool, "gachaPoolId") {
                add("gacha_pool", id.to_owned());
            }
        }
    }

    let consts: [(&str, &[&str]); 3] = [
        ("server", SERVERS),
        ("static_resource", STATIC_RESOURCES),
        ("image_size", IMAGE_SIZES),
    ];
    for (kind, values) in consts {
        for v in values {
            add(kind, (*v).to_owned());
        }
    }
    for kind in MANUAL_KINDS {
        kinds.entry((*kind).to_owned()).or_default();
    }

    let mut ids = IdSet {
        kinds,
        bindings: Vec::new(),
    };
    ids.normalize();
    Ok((ids, report))
}
