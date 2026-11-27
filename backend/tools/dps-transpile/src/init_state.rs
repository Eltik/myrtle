#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::Context;
use regex::Regex;

use crate::hand::apply_reference_corrections;
use crate::paths::Mode;
use crate::transpile::{extract_method, normalize_heal_variables};

#[derive(Debug, Clone)]
pub struct LocalSelfCollision {
    pub class_name: String,
    pub name: String,
    pub self_assigned: bool,
}

pub fn collision_candidates(init: &InitState) -> Vec<(String, String)> {
    let mut out = vec![("atk_interval".to_string(), "atk_interval".to_string())];
    for (py, rust, _) in INIT_STATE_FIELDS {
        if init.modified_fields.contains(*rust) && !rust.ends_with("_damage") {
            out.push(((*py).to_string(), (*rust).to_string()));
        }
    }
    out
}

pub fn find_local_self_collisions(
    class_name: &str,
    body: &str,
    candidates: &[(String, String)],
) -> Vec<LocalSelfCollision> {
    let mut out = Vec::new();
    for (py, rust) in candidates {
        let local_write = Regex::new(&format!(
            r"(?m)(?:^|[:\s])(?:{rust})\s*(?:\[[^\]]*\])?\s*[-+*/]?=[^=]"
        ))
        .unwrap();
        let has_local = body
            .lines()
            .any(|l| local_write.is_match(&l.replace(&format!("self.{rust}"), "")));
        let reads_self = body.contains(&format!("self.{py}"));
        if has_local && reads_self {
            let self_write = Regex::new(&format!(r"self\.{py}\s*[-+*/]?=[^=]")).unwrap();
            out.push(LocalSelfCollision {
                class_name: class_name.to_string(),
                name: (*rust).clone(),
                self_assigned: self_write.is_match(body),
            });
        }
    }
    out
}

/// Renames each colliding local `N` to `_N_w`
pub fn split_local_self_collisions(body: &str, collisions: &[LocalSelfCollision]) -> String {
    let mut s = body.to_string();
    for c in collisions.iter().filter(|c| !c.self_assigned) {
        s = rename_word(&s, &c.name, &format!("_{}_w", c.name));
    }
    s
}

/// Python field, its Rust shadow local, and that local's declaration
pub const INIT_STATE_FIELDS: &[(&str, &str, &str)] = &[
    (
        "drone_atk",
        "drone_atk",
        "let mut drone_atk: f64 = unit.drone_atk;",
    ),
    ("atk", "atk", "let mut atk: f64 = unit.atk;"),
    (
        "attack_speed",
        "attack_speed",
        "let mut attack_speed: f64 = unit.attack_speed;",
    ),
    ("ammo", "ammo", "let mut ammo: f64 = unit.ammo;"),
    (
        "skill_duration",
        "skill_duration",
        "let mut skill_duration: f64 = unit.skill_duration;",
    ),
    (
        "target_hp",
        "target_hp",
        "let mut target_hp: f64 = unit.target_hp;",
    ),
    ("shadows", "shadows", "let mut shadows: f64 = unit.shadows;"),
    (
        "trait_dmg",
        "trait_damage",
        "let mut trait_damage: bool = unit.trait_damage;",
    ),
    (
        "talent_dmg",
        "talent_damage",
        "let mut talent_damage: bool = unit.talent_damage;",
    ),
    (
        "talent2_dmg",
        "talent2_damage",
        "let mut talent2_damage: bool = unit.talent2_damage;",
    ),
    (
        "skill_dmg",
        "skill_damage",
        "let mut skill_damage: bool = unit.skill_damage;",
    ),
    (
        "module_dmg",
        "module_damage",
        "let mut module_damage: bool = unit.module_damage;",
    ),
];

pub const INIT_STATE_ELSEWHERE: &[(&str, &str, &str)] = &[
    (
        "Amiya",
        "skill",
        "forces skill 3 when pp.skill is -1 (the bot's 'best skill'); the engine always \
         passes an explicit skill",
    ),
    (
        "Amiya",
        "params",
        "S3 multiplier by mastery from pp.mastery; the engine reads the same value from \
         skill_parameters at the requested skill level",
    ),
    (
        "Stainless",
        "params",
        "S3 multiplier by mastery from pp.mastery; the engine reads skill_parameters",
    ),
    (
        "Stainless",
        "params2",
        "S3 multiplier by mastery from pp.mastery; the engine reads skill_parameters",
    ),
    (
        "Nian",
        "atk",
        "the reference adds module 1's ATK by hand because its own data reader drops it \
         ('module lvl 1 does not come with an atk increase, breaking the automatic \
         system'); the engine reads module ATK from game data, so hoisting it would count \
         it twice (measured: hoisted, s1 m1 DEF 0 740.360 -> 795.693 against the \
         reference's 740.360)",
    ),
    (
        "Necrass",
        "no_kill",
        "keyword switch nokill / kill (try_kwargs, False without kwargs); the body maps \
         self.no_kill to !skill_damage, so the skill conditional is that switch and its \
         default (on) is the reference's False",
    ),
    (
        "AmiyaMedic",
        "res",
        "copies pp.res for a formula the transpiler cannot carry (HEAL_FALLBACKS stub)",
    ),
    (
        "ReedAlter",
        "res",
        "copies pp.res for a formula the transpiler cannot carry (HEAL_FALLBACKS stub)",
    ),
    (
        "Pozemka",
        "pot",
        "copies pp.pot; the formula's self.pot maps to unit.potential directly",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitStateKind {
    Hoisted,
    KwargsDefault,
    NotRead,
    Elsewhere(&'static str),
    Unhandled(String),
}

#[derive(Debug, Clone)]
pub struct InitStateFinding {
    pub class_name: String,
    pub field: String,
    pub line: String,
    pub kind: InitStateKind,
}

#[derive(Debug, Clone)]
struct PyNode {
    text: String,
    children: Vec<Self>,
}

static SELF_ASSIGN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^self\.(\w+)\s*([-+*/]?=)\s*([^=\s].*)$").unwrap());
static LOCAL_ASSIGN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Za-z_]\w*)\s*(?:\[[^\]]*\])?\s*[-+*/]?=[^=]").unwrap());
static IDENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:\b|^)([A-Za-z_]\w*)\b").unwrap());

const COMPOUND_KEYWORDS: &[&str] = &["if ", "elif ", "else", "try", "except", "for ", "while "];

fn split_compound(text: &str) -> Option<(String, Option<String>)> {
    if !COMPOUND_KEYWORDS.iter().any(|k| text.starts_with(k)) {
        return None;
    }
    let (mut depth, mut quote) = (0i32, None::<char>);
    for (i, c) in text.char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[' | '{') => depth += 1,
            (None, ')' | ']' | '}') => depth -= 1,
            (None, ':') if depth == 0 => {
                let head = format!("{}:", text[..i].trim_end());
                let rest = text[i + 1..].split('#').next().unwrap_or("").trim();
                return Some((head, (!rest.is_empty()).then(|| rest.to_string())));
            }
            _ => {}
        }
    }
    None
}

fn parse_py_block(src: &str) -> Vec<PyNode> {
    let mut items: Vec<(usize, String)> = Vec::new();
    for line in src.lines() {
        let code = line.split('#').next().unwrap_or("");
        if code.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|&c| c == '\t').count();
        items.push((indent, code.trim().to_string()));
    }
    fn build(items: &[(usize, String)], pos: &mut usize, indent: usize) -> Vec<PyNode> {
        let mut out = Vec::new();
        while *pos < items.len() && items[*pos].0 >= indent {
            let (ind, text) = items[*pos].clone();
            *pos += 1;
            let mut node = match split_compound(&text) {
                Some((head, Some(inline))) => PyNode {
                    text: head,
                    children: vec![PyNode {
                        text: inline,
                        children: Vec::new(),
                    }],
                },
                Some((head, None)) => PyNode {
                    text: head,
                    children: Vec::new(),
                },
                None => PyNode {
                    text,
                    children: Vec::new(),
                },
            };
            if *pos < items.len() && items[*pos].0 > ind {
                let child_indent = items[*pos].0;
                node.children.extend(build(items, pos, child_indent));
            }
            out.push(node);
        }
        out
    }
    let mut pos = 0;
    build(&items, &mut pos, 0)
}

pub struct InitState {
    pub prelude: Vec<String>,
    pub shadow_decls: Vec<String>,
    pub modified_fields: HashSet<String>,
    pub free_fields: Vec<String>,
    pub findings: Vec<InitStateFinding>,
}

fn init_state_field(py_field: &str) -> Option<(&'static str, &'static str)> {
    INIT_STATE_FIELDS
        .iter()
        .find(|(py, _, _)| *py == py_field)
        .map(|(_, rust, decl)| (*rust, *decl))
}

/// The statements after `super().__init__(...)`
fn init_statements(class_body: &str) -> Vec<PyNode> {
    let init = extract_method(class_body, "__init__").replace("pp.pot", "self.pot");
    let nodes = parse_py_block(&init);
    let start = nodes
        .iter()
        .position(|n| n.text.contains("super().__init__"))
        .map_or(0, |i| i + 1);
    nodes[start..].to_vec()
}

#[derive(Debug, Clone, Copy, Default)]
struct Ctx {
    in_try: bool,
}

const REQUEST_SUPPLIED: &[&str] = &["shreds", "cloned_op"];

static CTOR_ARG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b(pp|kwargs)\b").unwrap());

const INIT_LOCAL_PREFIX: &str = "init_";
pub const FREE_FIELD_PREFIX: &str = "st_";

struct InitWalk<'a> {
    class_name: &'a str,
    method_body: &'a str,
    needed_locals: BTreeSet<String>,
    local_deps: BTreeMap<String, BTreeSet<String>>,
    wanted: BTreeSet<String>,
    free_fields: BTreeMap<String, bool>,
    findings: Vec<InitStateFinding>,
}

fn idents(text: &str) -> BTreeSet<String> {
    IDENT_RE
        .captures_iter(text)
        .filter(|c| {
            let m = c.get(1).unwrap();
            !text[..m.start()].ends_with('.')
        })
        .map(|c| c[1].to_string())
        .collect()
}

pub fn rename_word(text: &str, from: &str, to: &str) -> String {
    Regex::new(&format!(r"(^|[^.\w]){from}\b"))
        .unwrap()
        .replace_all(text, |c: &regex::Captures<'_>| format!("{}{to}", &c[1]))
        .to_string()
}

impl InitWalk<'_> {
    fn walk(&mut self, nodes: &[PyNode], guards: &[String], ctx: Ctx) -> Vec<PyNode> {
        let mut out: Vec<PyNode> = Vec::new();
        let mut i = 0;
        while i < nodes.len() {
            let node = &nodes[i];
            if node.text.starts_with("if ") {
                let mut j = i + 1;
                while j < nodes.len()
                    && (nodes[j].text.starts_with("elif ") || nodes[j].text == "else:")
                {
                    j += 1;
                }
                let mut branches = Vec::new();
                let mut any = false;
                for branch in &nodes[i..j] {
                    let mut g = guards.to_vec();
                    g.push(branch.text.clone());
                    let kept = self.walk(&branch.children, &g, ctx);
                    any |= !kept.is_empty();
                    let children = if kept.is_empty() {
                        vec![PyNode {
                            text: "pass".into(),
                            children: Vec::new(),
                        }]
                    } else {
                        kept
                    };
                    branches.push(PyNode {
                        text: branch.text.clone(),
                        children,
                    });
                }
                if any {
                    out.extend(branches);
                }
                i = j;
                continue;
            }
            if node.text == "try:" {
                self.walk(&node.children, guards, Ctx { in_try: true });
                i += 1;
                continue;
            }
            if node.text.starts_with("except") {
                let effective = Ctx { in_try: false };
                out.extend(self.walk(&node.children, guards, effective));
                i += 1;
                continue;
            }
            if node.text.starts_with("for ") || node.text.starts_with("while ") {
                let mut g = guards.to_vec();
                g.push(node.text.clone());
                let kept = self.walk(&node.children, &g, ctx);
                if !kept.is_empty() {
                    out.push(PyNode {
                        text: node.text.clone(),
                        children: kept,
                    });
                }
                i += 1;
                continue;
            }
            if let Some(kept) = self.statement(node, guards, ctx) {
                out.push(kept);
            }
            i += 1;
        }
        out
    }

    fn note_wanted(&mut self, guards: &[String], value: &str) {
        for g in guards {
            self.wanted.extend(idents(g));
        }
        self.wanted.extend(idents(value));
    }

    fn statement(&mut self, node: &PyNode, guards: &[String], ctx: Ctx) -> Option<PyNode> {
        let text = node.text.as_str();
        let Some(cap) = SELF_ASSIGN_RE.captures(text) else {
            let local = LOCAL_ASSIGN_RE.captures(text)?;
            let name = local[1].to_string();
            let rhs = text.split_once('=').map_or("", |(_, r)| r);
            let mut deps = idents(rhs);
            for g in guards {
                deps.extend(idents(g));
            }
            deps.extend(idents(text.split_once('=').map_or("", |(l, _)| l)));
            self.local_deps
                .entry(name.clone())
                .or_default()
                .extend(deps);
            if !self.needed_locals.contains(&name) {
                return None;
            }
            self.note_wanted(guards, rhs);
            return Some(PyNode {
                text: text.to_string(),
                children: Vec::new(),
            });
        };
        let field = cap[1].to_string();
        if field == "name" || field == "base_name" {
            return None;
        }
        let read_by_formula = Regex::new(&format!(r"self\.{field}\b"))
            .unwrap()
            .is_match(self.method_body);
        let mut finding = InitStateFinding {
            class_name: self.class_name.to_string(),
            field: field.clone(),
            line: text.to_string(),
            kind: InitStateKind::Hoisted,
        };
        let value = cap[3].to_string();
        let mut keep = None;
        let elsewhere = INIT_STATE_ELSEWHERE
            .iter()
            .find(|(c, f, _)| *c == self.class_name && *f == field);
        if ctx.in_try || REQUEST_SUPPLIED.contains(&field.as_str()) {
            finding.kind = InitStateKind::KwargsDefault;
        } else if let Some((_, _, why)) = elsewhere {
            finding.kind = InitStateKind::Elsewhere(why);
        } else if init_state_field(&field).is_none() && !read_by_formula {
            finding.kind = InitStateKind::NotRead;
        } else if CTOR_ARG_RE.is_match(&value) {
            finding.kind = InitStateKind::Unhandled("reads a constructor argument".into());
        } else if guards.iter().any(|g| CTOR_ARG_RE.is_match(g)) {
            finding.kind = InitStateKind::Unhandled("guarded by a constructor argument".into());
        } else if guards.iter().any(|g| g.starts_with("while ")) {
            finding.kind = InitStateKind::Unhandled("inside a while loop".into());
        } else {
            let target = if let Some((rust, _)) = init_state_field(&field) {
                rust.to_string()
            } else {
                let is_bool = matches!(value.trim(), "True" | "False");
                *self.free_fields.entry(field.clone()).or_insert(false) |= is_bool;
                format!("{FREE_FIELD_PREFIX}{field}")
            };
            self.note_wanted(guards, &value);
            keep = Some(PyNode {
                text: format!("{target} {} {value}", &cap[2]),
                children: Vec::new(),
            });
        }
        self.findings.push(finding);
        keep
    }
}

fn emit_py(nodes: &[PyNode], depth: usize, out: &mut Vec<String>) {
    for n in nodes {
        out.push(format!("{}{}", "\t".repeat(depth), n.text));
        emit_py(&n.children, depth + 1, out);
    }
}

pub fn analyze_init_state(class_name: &str, class_body: &str, method_body: &str) -> InitState {
    let nodes = init_statements(class_body);
    let new_walk = |needed| InitWalk {
        class_name,
        method_body,
        needed_locals: needed,
        local_deps: BTreeMap::new(),
        wanted: BTreeSet::new(),
        free_fields: BTreeMap::new(),
        findings: Vec::new(),
    };
    let mut first = new_walk(BTreeSet::new());
    first.walk(&nodes, &[], Ctx::default());
    let mut needed: BTreeSet<String> = first
        .wanted
        .iter()
        .filter(|w| first.local_deps.contains_key(*w))
        .cloned()
        .collect();
    loop {
        let more: Vec<String> = needed
            .iter()
            .flat_map(|n| first.local_deps.get(n).into_iter().flatten())
            .filter(|d| first.local_deps.contains_key(*d) && !needed.contains(*d))
            .cloned()
            .collect();
        if more.is_empty() {
            break;
        }
        needed.extend(more);
    }
    let mut walk = new_walk(needed.clone());
    let kept = walk.walk(&nodes, &[], Ctx::default());
    let mut prelude = Vec::new();
    emit_py(&kept, 0, &mut prelude);
    for local in &needed {
        let to = format!("{INIT_LOCAL_PREFIX}{local}");
        for line in &mut prelude {
            *line = rename_word(line, local, &to);
        }
    }

    let mut modified_fields = HashSet::new();
    let mut shadow_decls = Vec::new();
    let hoisted: BTreeSet<&str> = walk
        .findings
        .iter()
        .filter(|f| f.kind == InitStateKind::Hoisted)
        .map(|f| f.field.as_str())
        .collect();
    let any = !hoisted.is_empty();
    for (py, rust, decl) in INIT_STATE_FIELDS {
        let is_flag = decl.contains(": bool");
        if hoisted.contains(py) || (any && is_flag) {
            modified_fields.insert((*rust).to_string());
            shadow_decls.push(format!("    {decl}"));
        }
    }
    let mut free_fields = Vec::new();
    for (field, is_bool) in &walk.free_fields {
        let name = format!("{FREE_FIELD_PREFIX}{field}");
        let decl = if *is_bool {
            format!("    let mut {name}: bool = false;")
        } else {
            format!("    let mut {name}: f64 = 0.0;")
        };
        shadow_decls.push(decl);
        modified_fields.insert(name);
        free_fields.push(field.clone());
    }
    InitState {
        prelude,
        shadow_decls,
        modified_fields,
        free_fields,
        findings: walk.findings,
    }
}

/// Name, class source and transpiled method source of each class in a formula file
fn formula_classes(mode: Mode, py_src: &str) -> Vec<(String, String, String)> {
    let class_re = Regex::new(&format!(
        r"class\s+(\w+)\s*\(\s*{}\s*\)\s*:",
        mode.class_base_alt()
    ))
    .unwrap();
    let starts: Vec<(usize, String)> = class_re
        .captures_iter(py_src)
        .map(|c| (c.get(0).unwrap().start(), c[1].to_string()))
        .collect();
    let mut out = Vec::new();
    for (i, (start, name)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map_or(py_src.len(), |(p, _)| *p);
        let body = py_src[*start..end].to_string();
        let mut method = mode
            .method_names()
            .iter()
            .map(|m| extract_method(&body, m))
            .find(|b| !b.trim().is_empty())
            .unwrap_or_default();
        if mode == Mode::Healing {
            method = normalize_heal_variables(&method);
        }
        out.push((name.clone(), body, method));
    }
    out
}

pub fn formula_audit(mode: Mode, py_src: &str) -> (Vec<InitStateFinding>, Vec<LocalSelfCollision>) {
    let mut findings = Vec::new();
    let mut collisions = Vec::new();
    for (name, body, method) in formula_classes(mode, py_src) {
        let init = analyze_init_state(&name, &body, &method);
        collisions.extend(find_local_self_collisions(
            &name,
            &method,
            &collision_candidates(&init),
        ));
        findings.extend(init.findings);
    }
    (findings, collisions)
}

/// Upstream source with the reference corrections applied; panics when one no longer applies
pub fn corrected_source(mode: Mode, repo_path: &Path) -> anyhow::Result<String> {
    let path = repo_path.join(mode.py_source_rel());
    let src = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(apply_reference_corrections(mode.py_source_rel(), &src))
}

pub fn print_formula_audit(repo_path: &Path) -> anyhow::Result<()> {
    for mode in [Mode::Damage, Mode::Healing] {
        let src = corrected_source(mode, repo_path)?;
        let (findings, collisions) = formula_audit(mode, &src);
        println!(
            "=== {} __init__ state writes: {} ===",
            mode.human_label(),
            findings.len()
        );
        for f in &findings {
            let kind = match &f.kind {
                InitStateKind::Hoisted => "HOISTED".to_string(),
                InitStateKind::KwargsDefault => "KWARGS DEFAULT".to_string(),
                InitStateKind::NotRead => "NOT READ BY FORMULA".to_string(),
                InitStateKind::Elsewhere(why) => format!("NOT HOISTED, ON PURPOSE: {why}"),
                InitStateKind::Unhandled(why) => format!("UNHANDLED: {why}"),
            };
            println!("  {}.{}  [{kind}]  {}", f.class_name, f.field, f.line);
        }
        println!(
            "=== {} local / self name collisions: {} ===",
            mode.human_label(),
            collisions.len()
        );
        for c in &collisions {
            let state = if c.self_assigned {
                "NOT SPLIT (body rebinds self too)"
            } else {
                "split"
            };
            println!("  {}.{}  [{state}]", c.class_name, c.name);
        }
    }
    Ok(())
}
