//! Dump the AVG story-scene prefab geometry: the RectTransform tree with anchors, sizes and
//! pivots, the Canvas/CanvasScaler settings, and every MonoBehaviour's serialized fields keyed by
//! its MonoScript class name.
//!
//! Motivation: the IL2CPP read (`docs/story-reader-il2cpp-characters.md`) proved that
//! `AVGCharacterslotPanel._GenPos` maps no slot name to a position and that the lit/dim tints are
//! serialized Colors on the panel MonoBehaviour. Both live in the prefab, so they can only be read
//! from the bundle.
//!
//! Usage:
//!   cargo run --release --example dump_avg_prefab -- <bundle.ab> [tree|mb|canvas] [filter]
//!     tree   : the RectTransform hierarchy with geometry (filter = root GameObject name substring)
//!     mb     : every MonoBehaviour whose script class name contains `filter`, full JSON
//!     canvas : Canvas + CanvasScaler objects
//!     scripts: the MonoScript inventory with instance counts
//!     hub    : the AVG character sprite hub as `hub.json` carries it, through
//!              the same `export::avg_hub` decoder the texture export writes,
//!              each sprite entry with its own `size {w,h}` in texture pixels
//!              (the `sprites` mode prints the same number as `rect=`)
#![allow(
    clippy::too_many_lines,
    clippy::case_sensitive_file_extension_comparisons
)]
use serde_json::Value;
use std::collections::HashMap;
use unpacker::export::avg_hub;
use unpacker::unity::{
    bundle::BundleFile, object_reader::read_object, serialized_file::SerializedFile,
};

fn pid(v: &Value) -> Option<i64> {
    v.get("m_PathID").and_then(Value::as_i64)
}

fn f(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(o) => {
            let x = o.get("x").and_then(Value::as_f64);
            let y = o.get("y").and_then(Value::as_f64);
            let z = o.get("z").and_then(Value::as_f64);
            match (x, y, z) {
                (Some(a), Some(b), Some(c)) => format!("({a},{b},{c})"),
                (Some(a), Some(b), None) => format!("({a},{b})"),
                _ => o.to_string(),
            }
        }
        None => "-".into(),
    }
}

struct Ctx {
    objs: HashMap<i64, (i32, Value)>,
    /// pathID -> the root type-tree name, which for a MonoBehaviour is the C# class.
    tname: HashMap<i64, String>,
}

impl Ctx {
    fn name(&self, p: i64) -> String {
        self.objs
            .get(&p)
            .and_then(|(_, v)| v.get("m_Name"))
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string()
    }
    /// The MonoScript class name for a MonoBehaviour value.
    fn script_name_of(&self, p: i64) -> String {
        let t = self.tname.get(&p).cloned().unwrap_or_default();
        if t.is_empty() || t == "MonoBehaviour" {
            let sp = self
                .objs
                .get(&p)
                .and_then(|(_, v)| v.get("m_Script"))
                .and_then(pid)
                .unwrap_or(0);
            return format!("MonoBehaviour(script#{sp})");
        }
        t
    }
    /// Component class labels attached to a GameObject.
    fn components(&self, go: i64) -> Vec<String> {
        let Some((_, gv)) = self.objs.get(&go) else {
            return vec![];
        };
        let mut out = vec![];
        if let Some(arr) = gv.get("m_Component").and_then(Value::as_array) {
            for c in arr {
                let p = c
                    .get("component")
                    .and_then(pid)
                    .or_else(|| pid(c))
                    .unwrap_or(0);
                if let Some((cid, cv)) = self.objs.get(&p) {
                    let label = match *cid {
                        114 => {
                            let _ = cv;
                            format!("MB:{}#{p}", self.script_name_of(p))
                        }
                        223 => format!("Canvas#{p}"),
                        224 => format!("RectTransform#{p}"),
                        4 => format!("Transform#{p}"),
                        222 => format!("CanvasRenderer#{p}"),
                        225 => format!("CanvasGroup#{p}"),
                        213 => format!("SpriteRenderer?#{p}"),
                        other => format!("cls{other}#{p}"),
                    };
                    out.push(label);
                }
            }
        }
        out
    }
}

struct Ent {
    path: String,
    data: Vec<u8>,
}

fn main() {
    let path = std::env::args().nth(1).expect("bundle");
    let mode = std::env::args().nth(2).unwrap_or_else(|| "tree".into());
    let filter = std::env::args().nth(3).unwrap_or_default();
    let data = std::fs::read(&path).expect("read");
    // A raw SerializedFile (level0, sharedassets0.assets, globalgamemanagers) is read directly;
    // anything else goes through the bundle reader.
    let files: Vec<(String, Vec<u8>)> = BundleFile::parse(data.clone()).map_or_else(
        |_| vec![(path.clone(), data.clone())],
        |b| {
            b.files
                .iter()
                .map(|e| (e.path.clone(), e.data.clone()))
                .collect()
        },
    );
    let mut objs: HashMap<i64, (i32, Value)> = HashMap::new();
    let mut tname: HashMap<i64, String> = HashMap::new();
    for (epath, edata) in &files {
        let e = Ent {
            path: epath.clone(),
            data: edata.clone(),
        };
        let lower = e.path.to_ascii_lowercase();
        if lower.ends_with(".ress") || lower.ends_with(".resource") {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(e.data.clone()) else {
            continue;
        };
        println!("# file {} typetree={}", e.path, sf.enable_type_tree);
        for o in &sf.objects {
            if let Some(t) = sf
                .types
                .get(o.type_index)
                .and_then(|t| t.type_tree.as_ref())
            {
                tname.insert(o.path_id, t.type_name.clone());
            }
            match read_object(&sf, o) {
                Ok(v) => {
                    objs.insert(o.path_id, (o.class_id, v));
                }
                Err(err) => println!("# UNREADABLE cls{} pid{} {err}", o.class_id, o.path_id),
            }
        }
    }
    let ctx = Ctx { objs, tname };

    match mode.as_str() {
        "hub" => match avg_hub::hub_from_objects(&ctx.objs) {
            Some(hub) => println!("{}", serde_json::to_string_pretty(&hub).unwrap_or_default()),
            None => println!("# no AVGCharacterSpriteHub(Group) MonoBehaviour in this bundle"),
        },
        "scripts" => {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for (p, (cid, _)) in &ctx.objs {
                if *cid == 114 {
                    *counts.entry(ctx.script_name_of(*p)).or_default() += 1;
                }
            }
            let mut rows: Vec<_> = counts.into_iter().collect();
            rows.sort_unstable();
            for (n, c) in rows {
                println!("{c:4}  {n}");
            }
        }
        "monoscripts" => {
            let mut rows: Vec<(String, i64)> = vec![];
            for (p, (cid, v)) in &ctx.objs {
                if *cid != 115 {
                    continue;
                }
                let n = v
                    .get("m_ClassName")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string();
                if filter.is_empty() || n.to_lowercase().contains(&filter.to_lowercase()) {
                    rows.push((n, *p));
                }
            }
            rows.sort();
            for (n, p) in rows {
                println!("{p}  {n}");
            }
        }
        "sprites" => {
            let mut rows: Vec<String> = vec![];
            for (p, (cid, v)) in &ctx.objs {
                match *cid {
                    28 => rows.push(format!(
                        "Texture2D#{p} '{}' {}x{} fmt={}",
                        v.get("m_Name").and_then(Value::as_str).unwrap_or(""),
                        v.get("m_Width").and_then(Value::as_i64).unwrap_or(-1),
                        v.get("m_Height").and_then(Value::as_i64).unwrap_or(-1),
                        v.get("m_TextureFormat")
                            .and_then(Value::as_i64)
                            .unwrap_or(-1)
                    )),
                    213 => rows.push(format!(
                        "Sprite#{p} '{}' rect={} offset={} ppu={} pivot={}",
                        v.get("m_Name").and_then(Value::as_str).unwrap_or(""),
                        v.get("m_Rect")
                            .map_or_else(String::new, ToString::to_string),
                        v.get("m_Offset")
                            .map_or_else(String::new, ToString::to_string),
                        v.get("m_PixelsToUnits")
                            .map_or_else(String::new, ToString::to_string),
                        v.get("m_Pivot")
                            .map_or_else(String::new, ToString::to_string)
                    )),
                    _ => {}
                }
            }
            rows.sort();
            for r in rows {
                println!("{r}");
            }
        }
        "canvas" => {
            for (p, (cid, v)) in &ctx.objs {
                if *cid == 114 && v.get("m_ReferenceResolution").is_some() {
                    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                    println!(
                        "CanvasScaler#{p} on '{}' = {}",
                        ctx.name(go),
                        serde_json::to_string(v).unwrap_or_default()
                    );
                }
                if *cid == 223 {
                    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                    println!("Canvas#{p} on '{}' = {v}", ctx.name(go));
                }
                if *cid == 114 {
                    let n = ctx.script_name_of(*p);
                    if n.contains("Scaler") {
                        let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                        println!("{n}#{p} on '{}' = {v}", ctx.name(go));
                    }
                }
            }
        }
        "tt" => {
            let want: i64 = filter.parse().expect("pathID");
            for (_, edata) in &files {
                let Ok(sf) = SerializedFile::parse(edata.clone()) else {
                    continue;
                };
                for o in &sf.objects {
                    if o.path_id != want {
                        continue;
                    }
                    if let Some(t) = sf
                        .types
                        .get(o.type_index)
                        .and_then(|t| t.type_tree.as_ref())
                    {
                        walk(t, 0);
                    }
                }
            }
        }
        "mb" => {
            let mut rows: Vec<(String, i64, String)> = vec![];
            for (p, (cid, v)) in &ctx.objs {
                if *cid != 114 {
                    continue;
                }
                let n = ctx.script_name_of(*p);
                let keep = if let Ok(want) = filter.parse::<i64>() {
                    *p == want || v.get("m_Script").and_then(pid) == Some(want)
                } else {
                    filter.is_empty() || n.to_lowercase().contains(&filter.to_lowercase())
                };
                if !keep {
                    continue;
                }
                let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
                rows.push((
                    n.clone(),
                    *p,
                    format!(
                        "== {n}#{p} on GameObject '{}'\n{}",
                        ctx.name(go),
                        serde_json::to_string_pretty(v).unwrap_or_default()
                    ),
                ));
            }
            rows.sort();
            for (_, _, s) in rows {
                println!("{s}");
            }
        }
        _ => {
            // tree
            let mut father: HashMap<i64, i64> = HashMap::new();
            let mut tfs: Vec<i64> = vec![];
            for (p, (cid, v)) in &ctx.objs {
                if *cid == 224 || *cid == 4 {
                    father.insert(*p, v.get("m_Father").and_then(pid).unwrap_or(0));
                    tfs.push(*p);
                }
            }
            tfs.sort_unstable();
            let roots: Vec<i64> = tfs
                .iter()
                .copied()
                .filter(|t| father.get(t).copied().unwrap_or(0) == 0)
                .collect();
            for r in roots {
                print_tree(&ctx, r, 0, &filter, filter.is_empty());
            }
        }
    }
}

fn walk(n: &unpacker::unity::type_tree::TypeTreeNode, d: usize) {
    println!(
        "{}{} {}  size={} flags={} meta={:#x}",
        "  ".repeat(d),
        n.type_name,
        n.name,
        n.byte_size,
        n.type_flags,
        n.meta_flag
    );
    for c in &n.children {
        walk(c, d + 1);
    }
}

fn print_tree(ctx: &Ctx, tf: i64, depth: usize, filter: &str, mut on: bool) {
    let Some((cid, v)) = ctx.objs.get(&tf) else {
        return;
    };
    let go = v.get("m_GameObject").and_then(pid).unwrap_or(0);
    let name = ctx.name(go);
    if !on && !filter.is_empty() && name.to_lowercase().contains(&filter.to_lowercase()) {
        on = true;
    }
    if on {
        let ind = "  ".repeat(depth);
        let active = ctx
            .objs
            .get(&go)
            .and_then(|(_, g)| g.get("m_IsActive"))
            .cloned()
            .unwrap_or(Value::Null);
        println!(
            "{ind}{name}  [{}#{tf} go#{go} active={active}]",
            if *cid == 224 { "RT" } else { "TF" }
        );
        if *cid == 224 {
            println!(
                "{ind}   anchorMin={} anchorMax={} anchoredPos={} sizeDelta={} pivot={} localPos={} localScale={} rot={}",
                f(v, "m_AnchorMin"),
                f(v, "m_AnchorMax"),
                f(v, "m_AnchoredPosition"),
                f(v, "m_SizeDelta"),
                f(v, "m_Pivot"),
                f(v, "m_LocalPosition"),
                f(v, "m_LocalScale"),
                f(v, "m_LocalRotation"),
            );
        } else {
            println!(
                "{ind}   localPos={} localScale={}",
                f(v, "m_LocalPosition"),
                f(v, "m_LocalScale")
            );
        }
        let comps = ctx.components(go);
        if !comps.is_empty() {
            println!("{ind}   comps: {}", comps.join(" "));
        }
    }
    if let Some(ch) = v.get("m_Children").and_then(Value::as_array) {
        for c in ch {
            if let Some(p) = pid(c) {
                print_tree(ctx, p, if on { depth + 1 } else { depth }, filter, on);
            }
        }
    }
}
