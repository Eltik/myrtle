use std::{collections::HashMap, fs};

use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct OperatorFormula {
    name: String,
    class_name: String,
    available_skills: Vec<i32>,
    available_modules: Vec<i32>,
    default_skill: i32,
    default_potential: i32,
    default_module: i32,
    skills: HashMap<String, SkillFormula>,
    conditionals: Vec<Conditional>,
}

#[derive(Debug, Serialize)]
struct SkillFormula {
    #[serde(rename = "type")]
    formula_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    atk_scale_idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hits_idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hits: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    targets: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspd_talent_idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    def_ignore_idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    res_ignore_idx: Option<usize>,
}

impl Default for SkillFormula {
    fn default() -> Self {
        Self {
            formula_type: "custom".to_string(),
            atk_scale_idx: None,
            hits_idx: None,
            hits: None,
            targets: None,
            aspd_talent_idx: None,
            def_ignore_idx: None,
            res_ignore_idx: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct Conditional {
    #[serde(rename = "type")]
    cond_type: String,
    name: String,
    default: bool,
    skills: Vec<i32>,
    modules: Vec<i32>,
}

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "external/ArknightsDpsCompare".to_string());

    let id_dict = parse_id_dict(&repo_path);
    println!("Parsed {} char_id mappings", id_dict.len());

    let formulas_src = fs::read_to_string(format!("{repo_path}/damagecalc/damage_formulas.py"))
        .expect("Failed to read damage_formulas.py");

    let operators = parse_operators(&formulas_src, &id_dict);
    println!("Parsed {} operators", operators.len());

    let json = serde_json::to_string_pretty(&operators).unwrap();
    fs::write("src/dps/config/operator_formulas.json", &json).unwrap();
    println!("Written to src/dps/config/operator_formulas.json");

    // Stats
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for op in operators.values() {
        for skill in op.skills.values() {
            *type_counts.entry(skill.formula_type.clone()).or_default() += 1;
        }
    }
    println!("\nFormula type distribution:");
    let mut sorted: Vec<_> = type_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (t, c) in &sorted {
        println!("  {t}: {c}");
    }
}

fn parse_id_dict(repo_path: &str) -> HashMap<String, String> {
    let src = fs::read_to_string(format!("{repo_path}/Database/JsonReader.py"))
        .expect("Failed to read JsonReader.py");

    let re = Regex::new(r"'(\w+)'\s*:\s*'(char_\d+_\w+)'").unwrap();
    let mut map = HashMap::new();

    for cap in re.captures_iter(&src) {
        map.insert(cap[1].to_string(), cap[2].to_string());
    }

    map
}

fn parse_operators(
    src: &str,
    id_dict: &HashMap<String, String>,
) -> HashMap<String, OperatorFormula> {
    let class_re = Regex::new(r"class\s+(\w+)\s*\(\s*Operator\s*\)\s*:").unwrap();

    // Find all class positions
    let class_positions: Vec<(usize, String)> = class_re
        .captures_iter(src)
        .map(|cap| (cap.get(0).unwrap().start(), cap[1].to_string()))
        .collect();

    let mut result = HashMap::new();

    for (i, (start, class_name)) in class_positions.iter().enumerate() {
        // Extract the class body (from this class to the next class or EOF)
        let end = class_positions
            .get(i + 1)
            .map(|(pos, _)| *pos)
            .unwrap_or(src.len());
        let class_body = &src[*start..end];

        // Extract __init__ body and skill_dps body
        let init_body = extract_method(class_body, "__init__");
        let skill_dps_body = extract_method(class_body, "skill_dps");

        // Parse super().__init__ call
        let (name, skills, modules, def_skill, def_pot, def_mod) = parse_super_init(&init_body);

        // Look up char_id
        let char_id = id_dict
            .get(class_name)
            .or_else(|| id_dict.get(&name))
            .cloned()
            .unwrap_or_else(|| format!("unknown_{}", class_name.to_lowercase()));

        // Extract conditionals from __init__
        let conditionals = extract_conditionals(&init_body);

        // Classify formula for each skill
        let mut skill_formulas = HashMap::new();
        for &skill_idx in &skills {
            let formula = classify_formula(&skill_dps_body, skill_idx);
            skill_formulas.insert(skill_idx.to_string(), formula);
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
                skills: skill_formulas,
                conditionals,
            },
        );
    }

    result
}

fn extract_method(class_body: &str, method_name: &str) -> String {
    let pattern = format!("def {method_name}(");
    let mut lines = class_body.lines();
    let mut body = String::new();
    let mut found = false;
    let mut base_indent = 0;

    for line in &mut lines {
        if line.trim_start().starts_with(&pattern) {
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
                break; // New method or class at same/lower indent
            }
            body.push_str(trimmed);
            body.push('\n');
        }
    }

    body
}

fn parse_super_init(init_body: &str) -> (String, Vec<i32>, Vec<i32>, i32, i32, i32) {
    // Match: super().__init__("Name", pp, [skills], [modules], ...)
    let re = Regex::new(
        r#"super\(\)\.__init__\(\s*"([^"]+)"\s*,\s*pp\s*,\s*\[([^\]]*)\]\s*,\s*\[([^\]]*)\](?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?"#
    ).unwrap();

    if let Some(cap) = re.captures(init_body) {
        let name = cap[1].to_string();
        let skills = parse_int_list(&cap[2]);
        let modules = parse_int_list(&cap[3]);
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

fn parse_int_list(s: &str) -> Vec<i32> {
    s.split(',').filter_map(|v| v.trim().parse().ok()).collect()
}

fn extract_conditionals(init_body: &str) -> Vec<Conditional> {
    let mut conditionals = Vec::new();

    // Pattern: if self.{type}_dmg ... self.name += " Label"
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
            if line.contains(py_field) {
                // Try to extract label from same line or next few lines
                if let Some(cap) = label_re.captures(line) {
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
    }

    // Deduplicate by name
    conditionals.sort_by(|a, b| a.name.cmp(&b.name));
    conditionals.dedup_by(|a, b| a.name == b.name);

    conditionals
}

fn classify_formula(skill_dps_body: &str, _skill_idx: i32) -> SkillFormula {
    let body = skill_dps_body;

    // Count skill branches (if self.skill == N)
    let branch_re = Regex::new(r"if self\.skill\s*==\s*\d+").unwrap();
    let branch_count = branch_re.find_iter(body).count();

    // Check for drone usage
    let uses_drone = body.contains("self.drone_atk");

    // Check for crit
    let uses_crit = body.contains("cdmg") || body.contains("crate");

    // Check for res ignore
    let uses_res_ignore = body.contains("resignore") || body.contains("newres");

    // Check for multi-target
    let uses_targets = body.contains("min(self.targets");

    // Check for multiple complex branches → custom
    if branch_count >= 3 || uses_drone || uses_crit {
        return SkillFormula {
            formula_type: "custom".to_string(),
            ..Default::default()
        };
    }

    // Simple cases: extract skill_params indices
    let param_re = Regex::new(r"self\.skill_params\[(\d+)\]").unwrap();
    let params_used: Vec<usize> = param_re
        .captures_iter(body)
        .filter_map(|c| c[1].parse().ok())
        .collect();

    let talent_re = Regex::new(r"self\.talent1_params\[(\d+)\]").unwrap();
    let talents_used: Vec<usize> = talent_re
        .captures_iter(body)
        .filter_map(|c| c[1].parse().ok())
        .collect();

    // Determine formula type
    if uses_res_ignore {
        SkillFormula {
            formula_type: "res_ignore".to_string(),
            atk_scale_idx: params_used.first().copied(),
            res_ignore_idx: talents_used.first().copied(),
            ..Default::default()
        }
    } else if uses_targets {
        SkillFormula {
            formula_type: "aoe".to_string(),
            atk_scale_idx: params_used.first().copied(),
            ..Default::default()
        }
    } else if !talents_used.is_empty() && body.contains("attack_speed") {
        SkillFormula {
            formula_type: "atk_buff_aspd".to_string(),
            atk_scale_idx: params_used.first().copied(),
            aspd_talent_idx: talents_used.first().copied(),
            ..Default::default()
        }
    } else if !params_used.is_empty() {
        SkillFormula {
            formula_type: "atk_buff".to_string(),
            atk_scale_idx: params_used.first().copied(),
            ..Default::default()
        }
    } else {
        SkillFormula {
            formula_type: "custom".to_string(),
            ..Default::default()
        }
    }
}
