#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::hand::write_output;
use crate::paths::Mode;

#[derive(Deserialize, Serialize, Clone)]
pub struct OperatorFormula {
    pub name: String,
    pub class_name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    pub default_skill: i32,
    pub default_potential: i32,
    pub default_module: i32,
    pub skills: BTreeMap<String, SkillFormula>,
    pub conditionals: Vec<Conditional>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SkillFormula {
    #[serde(rename = "type")]
    pub formula_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conditional {
    #[serde(rename = "type")]
    pub cond_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
}

pub fn load_formulas(mode: Mode) -> anyhow::Result<HashMap<String, OperatorFormula>> {
    let path = mode.formulas_json_path();
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

pub fn generate_formulas_json(mode: Mode, repo_path: &Path, py_src: &str) -> anyhow::Result<()> {
    let out_path = mode.formulas_json_path();
    println!(
        "=== Generating {} ({}) ===",
        out_path.display(),
        mode.human_label()
    );

    let id_dict = parse_id_dict(repo_path)?;
    println!("Parsed {} char_id mappings", id_dict.len());

    let operators = parse_operators(mode, py_src, &id_dict);
    println!("Parsed {} operators", operators.len());

    let sorted: BTreeMap<&String, &OperatorFormula> = operators.iter().collect(); // a regeneration diffs by content only
    let json = serde_json::to_string_pretty(&sorted)?;
    write_output(&out_path, json.as_bytes())?;
    println!("Written to {}", out_path.display());
    Ok(())
}

fn parse_id_dict(repo_path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let path = repo_path.join("Database/JsonReader.py");
    let src = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let re = Regex::new(r"'(\w+)'\s*:\s*'(char_\d+_\w+)'")?;
    Ok(re
        .captures_iter(&src)
        .map(|cap| (cap[1].to_string(), cap[2].to_string()))
        .collect())
}

fn parse_operators(
    mode: Mode,
    src: &str,
    id_dict: &HashMap<String, String>,
) -> HashMap<String, OperatorFormula> {
    let class_re = Regex::new(&format!(
        r"class\s+(\w+)\s*\(\s*{}\s*\)\s*:",
        mode.class_base_alt()
    ))
    .unwrap();
    let class_positions: Vec<(usize, String)> = class_re
        .captures_iter(src)
        .map(|cap| (cap.get(0).unwrap().start(), cap[1].to_string()))
        .collect();

    let mut result = HashMap::new();
    for (i, (start, class_name)) in class_positions.iter().enumerate() {
        if class_name == "Healer" {
            continue; // an inheritance shim, not an operator
        }

        let end = class_positions
            .get(i + 1)
            .map_or(src.len(), |(pos, _)| *pos);
        let class_body = &src[*start..end];

        let init_body = extract_init_method(class_body);
        let (name, skills, modules, def_skill, def_pot, def_mod) = parse_super_init(&init_body);
        let char_id = id_dict
            .get(&name)
            .or_else(|| id_dict.get(class_name)) // display name first: Hibiscus -> HibiscusAlter
            .cloned()
            .unwrap_or_else(|| format!("unknown_{}", class_name.to_lowercase()));
        let conditionals = extract_conditionals(&init_body);

        let mut skill_formulas = HashMap::new();
        for &skill_idx in &skills {
            skill_formulas.insert(
                skill_idx.to_string(),
                SkillFormula {
                    formula_type: "custom".to_string(),
                },
            );
        }

        result.insert(
            char_id,
            OperatorFormula {
                name,
                class_name: class_name.clone(),
                available_skills: skills,
                available_modules: modules,
                default_skill: def_skill,
                default_potential: def_pot,
                default_module: def_mod,
                skills: skill_formulas.into_iter().collect(),
                conditionals,
            },
        );
    }
    result
}

fn parse_super_init(init_body: &str) -> (String, Vec<i32>, Vec<i32>, i32, i32, i32) {
    let re = Regex::new(
        r#"super\(\)\.__init__\(\s*"([^"]+)"\s*,\s*pp\s*,\s*\[([^\]]*)\]\s*,\s*\[([^\]]*)\](?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?"#
    ).unwrap();
    if let Some(cap) = re.captures(init_body) {
        let name = cap[1].to_string();
        let skills: Vec<i32> = cap[2]
            .split(',')
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        let modules: Vec<i32> = cap[3]
            .split(',')
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        let def_skill = cap
            .get(4)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(3);
        let def_pot = cap
            .get(5)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        let def_mod = cap
            .get(6)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(1);
        (name, skills, modules, def_skill, def_pot, def_mod)
    } else {
        ("Unknown".to_string(), vec![], vec![], 3, 1, 0)
    }
}

fn extract_conditionals(init_body: &str) -> Vec<Conditional> {
    let mut conditionals = Vec::new();
    let cond_types = [
        ("trait_dmg", "trait"),
        ("talent_dmg", "talent"),
        ("talent2_dmg", "talent2"),
        ("skill_dmg", "skill"),
        ("module_dmg", "module"),
    ];
    let label_re = Regex::new(r#"self\.name\s*\+=\s*[f"]?\s*"?\s*(\w+)"#).unwrap();
    for line in init_body.lines() {
        for &(py_field, cond_type) in &cond_types {
            if line.contains(py_field)
                && let Some(cap) = label_re.captures(line)
            {
                conditionals.push(Conditional {
                    cond_type: cond_type.to_string(),
                    name: cap[1].to_string(),
                    default: false,
                    skills: vec![],
                    modules: vec![],
                });
            }
        }
    }
    conditionals.sort_by(|a, b| a.name.cmp(&b.name));
    conditionals.dedup_by(|a, b| a.name == b.name);
    conditionals
}

fn extract_init_method(class_body: &str) -> String {
    let pattern = "def __init__(";
    let mut body = String::new();
    let mut found = false;
    let mut base_indent = 0;
    for line in class_body.lines() {
        if line.trim_start().starts_with(pattern) {
            found = true;
            base_indent = line.len() - line.trim_start().len();
            continue;
        }
        if found {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                body.push('\n');
                continue;
            }
            let indent = line.len() - trimmed.len();
            if indent <= base_indent && !trimmed.is_empty() {
                break;
            }
            body.push_str(trimmed);
            body.push('\n');
        }
    }
    body
}
