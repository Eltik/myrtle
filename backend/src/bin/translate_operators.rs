//! Translates Python operator classes from ArknightsDpsCompare to Rust
//!
//! Usage: cargo run --bin translate-operators -- <input_file> <output_dir>
//!
//! This tool parses the damage_formulas.py file and generates individual Rust files
//! for each operator, organized into alphabetical folders (a/, b/, c/, etc.)
//!
//! The skill_dps() methods are automatically translated from Python to Rust.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// Represents a parsed Python operator class
#[derive(Debug, Clone)]
struct OperatorClass {
    /// The Rust struct name (e.g., "Blaze", "ExusiaiAlter")
    class_name: String,
    /// The JSON name used in super().__init__ (e.g., "Blaze", "ExusiaiAlter")
    #[allow(dead_code)]
    json_name: String,
    /// Available skills list (e.g., [1, 2, 3])
    available_skills: Vec<i32>,
    /// Available modules list (e.g., [1, 2])
    available_modules: Vec<i32>,
    /// Default skill index
    default_skill: i32,
    /// Default potential
    default_pot: i32,
    /// Default module
    default_mod: i32,
    /// The __init__ method body (for conditional name logic)
    #[allow(dead_code)]
    init_body: String,
    /// The skill_dps method body
    skill_dps_body: String,
    /// Whether the class overrides total_dmg
    has_total_dmg_override: bool,
    /// The total_dmg method body (if overridden)
    total_dmg_body: String,
    /// Init-time field modifications (e.g., self.ammo = 4 + 4 * self.skill)
    init_modifications: Vec<InitModification>,
}

/// Represents an init-time field modification
#[derive(Debug, Clone)]
struct InitModification {
    /// The field being modified (e.g., "ammo", "drone_atk")
    field: String,
    /// The operation: "=" for assignment, "+=" for add-assign, etc.
    operation: String,
    /// The expression to apply
    expression: String,
    /// Optional condition for the modification
    condition: Option<String>,
}

/// Pre-compiled regex patterns for performance
/// These are compiled once and reused across all translations
struct CompiledPatterns {
    // Variable parsing
    identifier: Regex,
    compound_assign: Regex,

    // Condition translation
    and_op: Regex,
    or_op: Regex,
    not_op: Regex,

    // Integer to float conversions
    eq_ne_int: Regex,
    ge_le_int: Regex,
    gt_lt_int: Regex,
    mul_int_compare: Regex,
    mul_space_int_compare: Regex,
    div_int_compare: Regex,
    start_int_mul_float: Regex,
    mul_int_end: Regex,
    div_int_end: Regex,
    bare_int: Regex,
    start_int_mul: Regex,
    start_int_div: Regex,
    start_int_add: Regex,
    start_int_sub: Regex,
    eq_int_semi: Regex,
    brace_int_brace: Regex,
    add_int_space: Regex,
    add_int_end: Regex,
    sub_int_end: Regex,
    mul_int_end2: Regex,
    div_int_end2: Regex,
    sub_int_space: Regex,
    mul_int_space: Regex,
    mul_int_compare2: Regex,
    mul_nospace_compare: Regex,
    mul_int_add_sub: Regex,
    mul_nospace_add_sub: Regex,
    mul_int_lt: Regex,
    mod_int: Regex,
    brace_int_mul: Regex,
    brace_int_div: Regex,
    eq_int_mul: Regex,
    eq_int_div: Regex,
    int_mul_float: Regex,
    if_int_mul_float: Regex,
    add_eq_int_mul: Regex,
    sub_eq_int_mul: Regex,
    mul_eq_int_semi: Regex,
    div_eq_int_semi: Regex,
    add_eq_int_semi: Regex,
    sub_eq_int_semi: Regex,
    paren_int_add: Regex,
    paren_int_sub: Regex,
    paren_int_mul: Regex,
    paren_int_div: Regex,

    // Expression translation
    targets_idx: Regex,
    targets_sub1: Regex,
    neg_idx: Regex,
    bare_int_re: Regex,
    int_mul_div_re: Regex,
    int_mul_div_replace: Regex,

    // Self references
    self_skill: Regex,
    self_module: Regex,
    self_elite: Regex,
    skill_index: Regex,
    elite_ref: Regex,
    module_index: Regex,
    module_level: Regex,
    atk_interval: Regex,
    skill_cost: Regex,
    targets_ref: Regex,
    sp_boost: Regex,
    drone_atk_interval: Regex,
    crate_kw: Regex,
    // Specific cloned_op patterns for Ela defaults (must run before generic patterns)
    cloned_op_atk: Regex,
    cloned_op_atk_interval: Regex,
    cloned_op_ranged: Regex,
    cloned_op_physical: Regex,
    // Generic cloned_op patterns (fallback)
    cloned_op_bool: Regex,
    cloned_op_num: Regex,
    self_hits: Regex,
    self_pot: Regex,
    self_freeze_rate: Regex,
    self_count: Regex,
    self_below50: Regex,
    self_ammo: Regex,
    self_shadows: Regex,
    self_params: Regex,
    self_params2: Regex,
    self_no_kill: Regex,
    min_slice_from: Regex,
    max_slice_to: Regex,
    min_slice_to: Regex,
    max_slice_from: Regex,
    max_single: Regex,
    min_single: Regex,

    // In operator
    in_list: Regex,

    // Float division
    div_bare_int: Regex,

    // Min/max functions
    min_func: Regex,
    max_func: Regex,

    // Array indexing with bounds check
    talent1_idx: Regex,
    talent2_idx: Regex,
    skill_params_idx: Regex,
    shreds_idx: Regex,

    // CamelCase variables
    camel_case_vars: Vec<(Regex, &'static str)>,

    // Simple self mappings (stored as Vec for iteration)
    self_simple_mappings: Vec<(Regex, String)>,
}

impl CompiledPatterns {
    fn new() -> Self {
        let simple_mappings = [
            ("skill_params", "skill_parameters"),
            ("skill_duration", "skill_duration"),
            ("talent1_params", "talent1_parameters"),
            ("talent2_params", "talent2_parameters"),
            ("trait_dmg", "trait_damage"),
            ("talent_dmg", "talent_damage"),
            ("talent2_dmg", "talent2_damage"),
            ("skill_dmg", "skill_damage"),
            ("module_dmg", "module_damage"),
            ("module_lvl", "module_level"),
            ("buff_atk", "buff_atk"),
            ("buff_atk_flat", "buff_atk_flat"),
            ("buff_fragile", "buff_fragile"),
            ("atk", "atk"),
            ("attack_speed", "attack_speed"),
            ("physical", "is_physical"),
            ("ranged", "is_ranged"),
            ("shreds", "shreds"),
            ("drone_atk", "drone_atk"),
        ];

        let self_simple_mappings: Vec<(Regex, String)> = simple_mappings
            .iter()
            .map(|(py, rs)| {
                (
                    Regex::new(&format!(r"\bself\.{py}\b")).unwrap(),
                    format!("self.unit.{rs}"),
                )
            })
            .collect();

        let camel_case_vars = [
            ("timeToFallout", "time_to_fallout"),
            ("dpsFallout", "dps_fallout"),
            ("dpsNorm", "dps_norm"),
            ("hitdmgTW", "hitdmg_tw"),
            ("eleApplicationTarget", "ele_application_target"),
            ("eleApplicationBase", "ele_application_base"),
            ("targetEledps", "target_eledps"),
            ("ambientEledps", "ambient_eledps"),
        ]
        .iter()
        .map(|(camel, snake)| (Regex::new(&format!(r"\b{camel}\b")).unwrap(), *snake))
        .collect();

        Self {
            identifier: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap(),
            compound_assign: Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*(\+=|-=|\*=|/=)\s*(.+)$")
                .unwrap(),

            and_op: Regex::new(r"\s+and\s+").unwrap(),
            or_op: Regex::new(r"\s+or\s+").unwrap(),
            not_op: Regex::new(r"\bnot\s+").unwrap(),

            eq_ne_int: Regex::new(r"(==|!=)\s*(\d+)(\s|;|,|\)|$|\{|\}|&|\|)").unwrap(),
            ge_le_int: Regex::new(r"(>=|<=)\s*(\d+)(\s|;|,|\)|$|\{|\}|&|\|)").unwrap(),
            gt_lt_int: Regex::new(r"([^><])(>|<)\s*(\d+)(\s|;|,|\)|$|\{|\}|&|\|)").unwrap(),
            mul_int_compare: Regex::new(r"\*(\d+)(\s*[><=!])").unwrap(),
            mul_space_int_compare: Regex::new(r"(\*\s+)(\d+)(\s*[><=!])").unwrap(),
            div_int_compare: Regex::new(r"/(\d+)(\s*[><=!])").unwrap(),
            start_int_mul_float: Regex::new(r"^(\d+)\s*\*\s*(\d+\.\d+)").unwrap(),

            mul_int_end: Regex::new(r"\*\s*(\d+)$").unwrap(),
            div_int_end: Regex::new(r"/\s*(\d+)$").unwrap(),
            bare_int: Regex::new(r"^\d+$").unwrap(),
            start_int_mul: Regex::new(r"^(\d+)\s*\*").unwrap(),
            start_int_div: Regex::new(r"^(\d+)\s*/").unwrap(),
            start_int_add: Regex::new(r"^(\d+)\s*\+").unwrap(),
            start_int_sub: Regex::new(r"^(\d+)\s*-").unwrap(),
            eq_int_semi: Regex::new(r"=\s*(\d+)\s*;").unwrap(),
            brace_int_brace: Regex::new(r"\{\s*(\d+)\s*\}").unwrap(),
            add_int_space: Regex::new(r"(\+\s*)(\d+)(\s*[^\.\d\[])").unwrap(),
            add_int_end: Regex::new(r"(\+\s*)(\d+)$").unwrap(),
            sub_int_end: Regex::new(r"(-\s*)(\d+)$").unwrap(),
            mul_int_end2: Regex::new(r"(\*\s*)(\d+)$").unwrap(),
            div_int_end2: Regex::new(r"(/\s*)(\d+)$").unwrap(),
            sub_int_space: Regex::new(r"(-\s*)(\d+)(\s*[^\.\d\[])").unwrap(),
            mul_int_space: Regex::new(r"(\*\s*)(\d+)(\s*[^\.\d\[])").unwrap(),
            mul_int_compare2: Regex::new(r"(\*\s*)(\d+)(\s*[><=!])").unwrap(),
            mul_nospace_compare: Regex::new(r"\*(\d+)(\s*[><=!])").unwrap(),
            mul_int_add_sub: Regex::new(r"(\*\s*)(\d+)(\s*[\+\-])").unwrap(),
            mul_nospace_add_sub: Regex::new(r"\*(\d+)(\s*[\+\-])").unwrap(),
            mul_int_lt: Regex::new(r"\*\s*(\d+)\s*<").unwrap(),
            mod_int: Regex::new(r"(%\s*)(\d+)").unwrap(),
            brace_int_mul: Regex::new(r"\{\s*(\d+)\s*\*").unwrap(),
            brace_int_div: Regex::new(r"\{\s*(\d+)\s*/").unwrap(),
            eq_int_mul: Regex::new(r"=\s*(\d+)\s*\*").unwrap(),
            eq_int_div: Regex::new(r"=\s*(\d+)\s*/").unwrap(),
            int_mul_float: Regex::new(r"([\s({=+\-*/,;:])(\d+)\s*\*\s*(\d+\.\d+)").unwrap(),
            if_int_mul_float: Regex::new(r"\bif\s+(\d+)\s*\*\s*(\d+\.\d+)").unwrap(),
            add_eq_int_mul: Regex::new(r"\+=\s*(\d+)\s*\*").unwrap(),
            sub_eq_int_mul: Regex::new(r"-=\s*(\d+)\s*\*").unwrap(),
            mul_eq_int_semi: Regex::new(r"(\*=\s*)(\d+)(\s*;)").unwrap(),
            div_eq_int_semi: Regex::new(r"(/=\s*)(\d+)(\s*;)").unwrap(),
            add_eq_int_semi: Regex::new(r"(\+=\s*)(\d+)(\s*;)").unwrap(),
            sub_eq_int_semi: Regex::new(r"(-=\s*)(\d+)(\s*;)").unwrap(),
            paren_int_add: Regex::new(r"\((\d+)\s*\+").unwrap(),
            paren_int_sub: Regex::new(r"\((\d+)\s*-").unwrap(),
            paren_int_mul: Regex::new(r"\((\d+)\s*\*").unwrap(),
            paren_int_div: Regex::new(r"\((\d+)\s*/").unwrap(),

            targets_idx: Regex::new(r"\[targets\]").unwrap(),
            targets_sub1: Regex::new(r"\[targets\s*-\s*1\.0\]").unwrap(),
            neg_idx: Regex::new(r"\[-([\d.]+)\]").unwrap(),
            bare_int_re: Regex::new(r"^\d+$").unwrap(),
            int_mul_div_re: Regex::new(r"^\d+\s*[*/]").unwrap(),
            int_mul_div_replace: Regex::new(r"^(\d+)(\s*[*/])").unwrap(),

            self_skill: Regex::new(r"\bself\.skill\b").unwrap(),
            self_module: Regex::new(r"\bself\.module\b").unwrap(),
            self_elite: Regex::new(r"\bself\.elite\b").unwrap(),
            skill_index: Regex::new(r"\bself\.unit\.skill_index\b").unwrap(),
            elite_ref: Regex::new(r"\bself\.unit\.elite\b").unwrap(),
            module_index: Regex::new(r"\bself\.unit\.module_index\b").unwrap(),
            module_level: Regex::new(r"\bself\.unit\.module_level\b").unwrap(),
            atk_interval: Regex::new(r"\bself\.atk_interval\b").unwrap(),
            skill_cost: Regex::new(r"\bself\.skill_cost\b").unwrap(),
            targets_ref: Regex::new(r"\bself\.targets\b").unwrap(),
            sp_boost: Regex::new(r"\bself\.sp_boost\b").unwrap(),
            drone_atk_interval: Regex::new(r"\bself\.drone_atk_interval\b").unwrap(),
            crate_kw: Regex::new(r"\bcrate\b").unwrap(),
            // Specific cloned_op patterns for Ela defaults (Muelsyse's default clone)
            // Ela E2 90 stats: base ATK=588, trust=+80, pot3=+27 = 695 total
            cloned_op_atk: Regex::new(r"\bself\.cloned_op\.atk\b").unwrap(),
            cloned_op_atk_interval: Regex::new(r"\bself\.cloned_op\.atk_interval\b").unwrap(),
            cloned_op_ranged: Regex::new(r"\bself\.cloned_op\.ranged\b").unwrap(),
            cloned_op_physical: Regex::new(r"\bself\.cloned_op\.physical\b").unwrap(),
            // Generic cloned_op patterns (fallback for other properties)
            cloned_op_bool: Regex::new(
                r"self\.cloned_op\.(melee|trait_damage|talent_damage|skill_damage|module_damage)",
            )
            .unwrap(),
            cloned_op_num: Regex::new(r"self\.cloned_op\.(\w+)").unwrap(),
            self_hits: Regex::new(r"\bself\.hits\b").unwrap(),
            self_pot: Regex::new(r"\bself\.pot\b").unwrap(),
            self_freeze_rate: Regex::new(r"\bself\.freezeRate\b").unwrap(),
            self_count: Regex::new(r"\bself\.count\b").unwrap(),
            self_below50: Regex::new(r"\bself\.below50\b").unwrap(),
            self_ammo: Regex::new(r"\bself\.ammo\b").unwrap(),
            self_shadows: Regex::new(r"\bself\.shadows\b").unwrap(),
            self_params: Regex::new(r"\bself\.params\b").unwrap(),
            self_params2: Regex::new(r"\bself\.params2\b").unwrap(),
            self_no_kill: Regex::new(r"\bself\.no_kill\b").unwrap(),
            min_slice_from: Regex::new(r"\bmin\(self\.unit\.(\w+)\[(\d+):\]\)").unwrap(),
            max_slice_to: Regex::new(r"\bmax\(self\.unit\.(\w+)\[:(\d+)\]\)").unwrap(),
            min_slice_to: Regex::new(r"\bmin\(self\.unit\.(\w+)\[:(\d+)\]\)").unwrap(),
            max_slice_from: Regex::new(r"\bmax\(self\.unit\.(\w+)\[(\d+):\]\)").unwrap(),
            max_single: Regex::new(r"\bmax\(self\.unit\.(\w+)\)").unwrap(),
            min_single: Regex::new(r"\bmin\(self\.unit\.(\w+)\)").unwrap(),

            in_list: Regex::new(r"(\S+)\s+in\s+\[([^\]]+)\]").unwrap(),
            div_bare_int: Regex::new(r"/\s*(\d+)(\s|$|\)|,|\])").unwrap(),

            min_func: Regex::new(r"(?:^|[^.])min\(").unwrap(),
            max_func: Regex::new(r"(?:^|[^.])max\(").unwrap(),

            // Bounds-checked array access patterns
            talent1_idx: Regex::new(r"self\.unit\.talent1_parameters\[(\d+)\]").unwrap(),
            talent2_idx: Regex::new(r"self\.unit\.talent2_parameters\[(\d+)\]").unwrap(),
            skill_params_idx: Regex::new(r"self\.unit\.skill_parameters\[(\d+)\]").unwrap(),
            shreds_idx: Regex::new(r"self\.unit\.shreds\[(\d+)\]").unwrap(),

            camel_case_vars,
            self_simple_mappings,
        }
    }
}

/// Get the global compiled patterns (compiled once, reused forever)
fn patterns() -> &'static CompiledPatterns {
    static PATTERNS: OnceLock<CompiledPatterns> = OnceLock::new();
    PATTERNS.get_or_init(CompiledPatterns::new)
}

/// Python to Rust translator for skill_dps methods
struct PythonToRustTranslator {
    indent_level: usize,
    /// True if the code modifies self.atk_interval (e.g., self.atk_interval = 0.5)
    modifies_self_atk_interval: bool,
}

impl PythonToRustTranslator {
    fn new() -> Self {
        Self {
            indent_level: 2,
            modifies_self_atk_interval: false,
        }
    }

    /// Translate a Python skill_dps method body to Rust
    fn translate(&mut self, python_code: &str) -> String {
        let mut rust_lines = Vec::new();
        let lines: Vec<&str> = python_code.lines().collect();

        // Check if the code modifies self.atk_interval (e.g., "self.atk_interval = 0.5")
        // This determines whether self.atk_interval reads should use the local variable
        self.modifies_self_atk_interval = python_code.contains("self.atk_interval =")
            || python_code.contains("self.atk_interval=");

        // Scan for variables that are used across if/else branches
        let mut shared_vars = self.find_shared_variables(&lines);

        // Always declare atk_interval as a local variable since it's commonly used
        // and may be modified by skills
        shared_vars.insert("atk_interval".to_string());

        // IMPORTANT: Remove function parameters from shared_vars to avoid shadowing
        // The function already has `defense` and `res` as parameters from enemy stats
        shared_vars.remove("defense");
        shared_vars.remove("res");

        // Pre-declare shared variables at function start
        for var in &shared_vars {
            // Special initialization for variables that shadow struct fields
            let init_value = match var.as_str() {
                "atk_interval" => "self.unit.attack_interval as f64".to_string(),
                "skill_duration" => "self.unit.skill_duration".to_string(),
                _ => "0.0".to_string(),
            };
            rust_lines.push(format!(
                "{}let mut {}: f64 = {};",
                self.indent(self.indent_level),
                var,
                init_value
            ));
        }
        if !shared_vars.is_empty() {
            rust_lines.push(String::new());
        }

        let mut declared_vars: HashSet<String> = shared_vars;
        let mut indent_stack: Vec<(usize, bool)> = Vec::new();
        let mut _last_skill_check_indent: Option<usize> = None;
        let mut skip_until_indent: Option<usize> = None;
        let mut lines_to_skip: HashSet<usize> = HashSet::new();

        for (i, line) in lines.iter().enumerate() {
            if lines_to_skip.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let line_indent = line.len() - line.trim_start().len();

            if let Some(skip_indent) = skip_until_indent {
                if line_indent > skip_indent {
                    continue;
                } else {
                    skip_until_indent = None;
                }
            }

            let (code_part, comment_part) = if let Some(hash_pos) = trimmed.find('#') {
                (&trimmed[..hash_pos], Some(&trimmed[hash_pos..]))
            } else {
                (trimmed, None)
            };
            let trimmed = code_part.trim();

            if trimmed.is_empty() {
                if let Some(comment) = comment_part {
                    rust_lines.push(format!(
                        "{}// {}",
                        self.indent(self.indent_level),
                        &comment[1..].trim()
                    ));
                }
                continue;
            }

            let is_skill_check = trimmed.starts_with("if ")
                && trimmed.ends_with(':')
                && (trimmed.contains("self.skill") || trimmed.contains("self.module"));

            while !indent_stack.is_empty() {
                let (last_indent, _was_skill_check) = *indent_stack.last().unwrap();

                if line_indent > last_indent {
                    break;
                }

                if (trimmed.starts_with("elif ") || trimmed == "else:")
                    && line_indent == last_indent
                {
                    break;
                }

                indent_stack.pop();
                rust_lines.push(format!("{}}}", self.indent(self.indent_level)));
                _last_skill_check_indent = None;
            }

            // Handle return statement
            if trimmed.starts_with("return ") || trimmed.starts_with("return(") {
                let expr = if let Some(stripped) = trimmed.strip_prefix("return ") {
                    stripped
                } else if let Some(inner) = trimmed.strip_prefix("return") {
                    if inner.starts_with('(') && inner.ends_with(')') {
                        &inner[1..inner.len() - 1]
                    } else {
                        inner
                    }
                } else {
                    trimmed
                };

                if expr.contains("super()")
                    || expr.contains("self.skill_dps")
                    || expr.contains("self.total_dmg")
                    || expr.contains("self.avg_dps")
                {
                    rust_lines.push(format!(
                        "{}// UNTRANSLATED: {} - method calls need manual implementation",
                        self.indent(self.indent_level),
                        trimmed
                    ));
                    rust_lines.push(format!(
                        "{}0.0 // placeholder",
                        self.indent(self.indent_level)
                    ));
                    continue;
                }

                let rust_expr = self.translate_expression(expr);
                rust_lines.push(format!(
                    "{}return {};",
                    self.indent(self.indent_level),
                    rust_expr
                ));
                continue;
            }

            // Handle try-except patterns
            if let Some(try_stmt) = trimmed.strip_prefix("try: ") {
                // First, check for assignment pattern
                if let Some((var, try_expr)) = self.parse_assignment(try_stmt) {
                    let mut found_except = false;
                    for (j, next_line) in lines[i + 1..].iter().enumerate() {
                        let next_trimmed = next_line.trim();
                        if next_trimmed.starts_with("except:") {
                            let except_stmt =
                                if let Some(stripped) = next_trimmed.strip_prefix("except: ") {
                                    stripped
                                } else {
                                    break;
                                };

                            if let Some((except_var, default_expr)) =
                                self.parse_assignment(except_stmt)
                                && except_var == var
                            {
                                let rust_default = self.translate_expression(default_expr);

                                // Check if try_expr is an array access like self.talent1_params[2]
                                // If so, generate unwrap_or pattern instead of just using default
                                let rust_expr = if let Some(array_access) =
                                    self.extract_array_access(try_expr)
                                {
                                    let (array_name, idx) = array_access;
                                    format!(
                                        "self.unit.{array_name}.get({idx}).copied().unwrap_or({rust_default})"
                                    )
                                } else {
                                    rust_default
                                };

                                if !declared_vars.contains(&var) {
                                    rust_lines.push(format!(
                                        "{}let mut {} = {}; // try-except fallback",
                                        self.indent(self.indent_level),
                                        var,
                                        rust_expr
                                    ));
                                    declared_vars.insert(var.clone());
                                } else {
                                    rust_lines.push(format!(
                                        "{}{} = {}; // try-except fallback",
                                        self.indent(self.indent_level),
                                        var,
                                        rust_expr
                                    ));
                                }
                                found_except = true;
                                lines_to_skip.insert(i + j + 1);
                                break;
                            }
                        }
                        if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                            break;
                        }
                    }
                    if found_except {
                        continue;
                    }
                }

                // Handle compound assignment with except: pass (e.g., try: extrascale *= self.skill_params[3])
                if let Some((var, op, expr)) = self.parse_compound_assignment(try_stmt) {
                    // Check if next line is except: pass
                    for (j, next_line) in lines[i + 1..].iter().enumerate() {
                        let next_trimmed = next_line.trim();
                        if next_trimmed == "except: pass" || next_trimmed == "except:pass" {
                            // Generate if-let pattern for safe array access
                            // try: var *= self.skill_params[3] -> if let Some(val) = self.unit.skill_parameters.get(3) { var *= val; }
                            if let Some(array_access) = self.extract_array_access(expr) {
                                let (array_name, idx) = array_access;
                                rust_lines.push(format!(
                                    "{}// Python: try: {} {} {} except: pass",
                                    self.indent(self.indent_level),
                                    var,
                                    op,
                                    expr
                                ));
                                rust_lines.push(format!(
                                    "{}if let Some(val) = self.unit.{}.get({}) {{",
                                    self.indent(self.indent_level),
                                    array_name,
                                    idx
                                ));
                                rust_lines.push(format!(
                                    "{}    {} {} val;",
                                    self.indent(self.indent_level),
                                    var,
                                    op
                                ));
                                rust_lines.push(format!("{}}}", self.indent(self.indent_level)));
                                lines_to_skip.insert(i + j + 1);
                                break;
                            }
                        }
                        if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                            break;
                        }
                    }
                    continue;
                }
            }

            if trimmed.starts_with("except:") {
                continue;
            }

            // Handle single-line if statements
            if trimmed.starts_with("if ")
                && trimmed.contains(": ")
                && !trimmed.ends_with(':')
                && let Some(colon_pos) = trimmed.find(": ")
            {
                let condition = &trimmed[3..colon_pos];
                let statement = &trimmed[colon_pos + 2..];

                // Try simple assignment first
                if let Some((var, expr)) = self.parse_assignment(statement) {
                    let rust_condition = self.translate_condition(condition);
                    let rust_expr = self.translate_expression(expr);

                    if !declared_vars.contains(&var) {
                        rust_lines.push(format!(
                            "{}let mut {}: f64 = 0.0;",
                            self.indent(self.indent_level),
                            var
                        ));
                        declared_vars.insert(var.clone());
                    }

                    rust_lines.push(format!(
                        "{}if {} {{ {} = {}; }}",
                        self.indent(self.indent_level),
                        rust_condition,
                        var,
                        rust_expr
                    ));
                    continue;
                }

                // Try compound assignment (e.g., var *= expr)
                if let Some((var, op, expr)) = self.parse_compound_assignment(statement) {
                    let rust_condition = self.translate_condition(condition);
                    let rust_expr = self.translate_expression(expr);

                    rust_lines.push(format!(
                        "{}if {} {{ {} {} {}; }}",
                        self.indent(self.indent_level),
                        rust_condition,
                        var,
                        op,
                        rust_expr
                    ));
                    continue;
                }

                // Try return statement
                if let Some(return_expr) = statement.strip_prefix("return ")
                    && !return_expr.contains("self.skill_dps")
                    && !return_expr.contains("super()")
                    && !return_expr.contains("self.total_dmg")
                {
                    let rust_condition = self.translate_condition(condition);
                    let rust_expr = self.translate_expression(return_expr);
                    rust_lines.push(format!(
                        "{}if {} {{ return {}; }}",
                        self.indent(self.indent_level),
                        rust_condition,
                        rust_expr
                    ));
                    continue;
                }
            }

            // Handle single-line elif statements
            if trimmed.starts_with("elif ")
                && trimmed.contains(": ")
                && !trimmed.ends_with(':')
                && let Some(colon_pos) = trimmed.find(": ")
            {
                let condition = &trimmed[5..colon_pos];
                let statement = &trimmed[colon_pos + 2..];

                // Try simple assignment
                if let Some((var, expr)) = self.parse_assignment(statement) {
                    let rust_condition = self.translate_condition(condition);
                    let rust_expr = self.translate_expression(expr);

                    rust_lines.push(format!(
                        "{}else if {} {{ {} = {}; }}",
                        self.indent(self.indent_level),
                        rust_condition,
                        var,
                        rust_expr
                    ));
                    continue;
                }

                // Try compound assignment
                if let Some((var, op, expr)) = self.parse_compound_assignment(statement) {
                    let rust_condition = self.translate_condition(condition);
                    let rust_expr = self.translate_expression(expr);

                    rust_lines.push(format!(
                        "{}else if {} {{ {} {} {}; }}",
                        self.indent(self.indent_level),
                        rust_condition,
                        var,
                        op,
                        rust_expr
                    ));
                    continue;
                }

                // Try return statement
                if let Some(return_expr) = statement.strip_prefix("return ")
                    && !return_expr.contains("self.skill_dps")
                    && !return_expr.contains("super()")
                    && !return_expr.contains("self.total_dmg")
                {
                    let rust_condition = self.translate_condition(condition);
                    let rust_expr = self.translate_expression(return_expr);
                    rust_lines.push(format!(
                        "{}else if {} {{ return {}; }}",
                        self.indent(self.indent_level),
                        rust_condition,
                        rust_expr
                    ));
                    continue;
                }
            }

            // Handle single-line else statements
            if let Some(statement) = trimmed.strip_prefix("else: ") {
                // Try simple assignment first
                if let Some((var, expr)) = self.parse_assignment(statement) {
                    let rust_expr = self.translate_expression(expr);

                    rust_lines.push(format!(
                        "{}else {{ {} = {}; }}",
                        self.indent(self.indent_level),
                        var,
                        rust_expr
                    ));
                    continue;
                }

                // Try compound assignment
                if let Some((var, op, expr)) = self.parse_compound_assignment(statement) {
                    let rust_expr = self.translate_expression(expr);

                    rust_lines.push(format!(
                        "{}else {{ {} {} {}; }}",
                        self.indent(self.indent_level),
                        var,
                        op,
                        rust_expr
                    ));
                    continue;
                }

                // Try return statement
                if let Some(return_expr) = statement.strip_prefix("return ") {
                    let rust_expr = self.translate_expression(return_expr);
                    rust_lines.push(format!(
                        "{}else {{ return {}; }}",
                        self.indent(self.indent_level),
                        rust_expr
                    ));
                    continue;
                }
            }

            // Handle if statements
            if trimmed.starts_with("if ") && trimmed.ends_with(':') {
                let condition = &trimmed[3..trimmed.len() - 1];
                let rust_condition = self.translate_condition(condition);
                let comment_suffix = comment_part
                    .map(|c| format!(" // {}", &c[1..].trim()))
                    .unwrap_or_default();

                rust_lines.push(format!(
                    "{}if {} {{{}",
                    self.indent(self.indent_level),
                    rust_condition,
                    comment_suffix
                ));
                indent_stack.push((line_indent, is_skill_check));

                if is_skill_check {
                    _last_skill_check_indent = Some(line_indent);
                }
                continue;
            }

            // Handle elif statements
            if trimmed.starts_with("elif ") && trimmed.ends_with(':') {
                let condition = &trimmed[5..trimmed.len() - 1];
                let rust_condition = self.translate_condition(condition);

                if let Some(&(last_indent, _)) = indent_stack.last() {
                    if last_indent == line_indent {
                        indent_stack.pop();
                        rust_lines.push(format!(
                            "{}}} else if {} {{",
                            self.indent(self.indent_level),
                            rust_condition
                        ));
                        indent_stack.push((line_indent, false));
                    } else {
                        rust_lines.push(format!(
                            "{}if {} {{",
                            self.indent(self.indent_level),
                            rust_condition
                        ));
                        indent_stack.push((line_indent, false));
                    }
                } else {
                    rust_lines.push(format!(
                        "{}if {} {{",
                        self.indent(self.indent_level),
                        rust_condition
                    ));
                    indent_stack.push((line_indent, false));
                }
                continue;
            }

            // Handle else statements
            if trimmed == "else:" {
                if let Some(&(last_indent, _)) = indent_stack.last() {
                    if last_indent == line_indent {
                        indent_stack.pop();
                        rust_lines.push(format!("{}}} else {{", self.indent(self.indent_level)));
                        indent_stack.push((line_indent, false));
                    } else {
                        rust_lines.push(format!(
                            "{}// UNTRANSLATED ELSE (no matching if): else:",
                            self.indent(self.indent_level)
                        ));
                    }
                } else {
                    rust_lines.push(format!(
                        "{}// UNTRANSLATED ELSE (empty stack): else:",
                        self.indent(self.indent_level)
                    ));
                }
                continue;
            }

            // Handle for loops - translate for i in range(...):
            if trimmed.starts_with("for ")
                && trimmed.contains(" in range(")
                && trimmed.ends_with(':')
            {
                // Parse: for i in range(count): or for i in range(int(duration)):
                if let Some(for_match) = self.parse_for_range(trimmed) {
                    let (loop_var, range_expr) = for_match;
                    let rust_range = self.translate_expression(&range_expr);

                    rust_lines.push(format!(
                        "{}// Implement for loop: {}",
                        self.indent(self.indent_level),
                        trimmed
                    ));
                    // Use _loop_var for the integer iterator, and loop_var for the f64 version
                    // This way the Python code using the loop variable will use the f64 version
                    rust_lines.push(format!(
                        "{}for _{} in 0..({} as i32) {{",
                        self.indent(self.indent_level),
                        loop_var,
                        rust_range
                    ));

                    // Define the loop variable as f64 for use in calculations
                    rust_lines.push(format!(
                        "{}    let {} = _{} as f64;",
                        self.indent(self.indent_level),
                        loop_var,
                        loop_var
                    ));

                    indent_stack.push((line_indent, false));
                    continue;
                }
                // Fallback for unrecognized for loop patterns
                rust_lines.push(format!(
                    "{}// UNTRANSLATED FOR LOOP: {}",
                    self.indent(self.indent_level),
                    trimmed
                ));
                rust_lines.push(format!(
                    "{}// TODO: Implement loop logic manually",
                    self.indent(self.indent_level)
                ));
                skip_until_indent = Some(line_indent);
                continue;
            }

            // Handle variable assignments
            if let Some((var, expr)) = self.parse_assignment(trimmed) {
                let rust_expr = self.translate_expression(expr);

                if declared_vars.contains(&var) {
                    rust_lines.push(format!(
                        "{}{} = {};",
                        self.indent(self.indent_level),
                        var,
                        rust_expr
                    ));
                } else {
                    declared_vars.insert(var.clone());
                    rust_lines.push(format!(
                        "{}let mut {} = {};",
                        self.indent(self.indent_level),
                        var,
                        rust_expr
                    ));
                }
                continue;
            }

            // Handle compound assignments
            if let Some((var, op, expr)) = self.parse_compound_assignment(trimmed) {
                let rust_expr = self.translate_expression(expr);
                rust_lines.push(format!(
                    "{}{} {} {};",
                    self.indent(self.indent_level),
                    var,
                    op,
                    rust_expr
                ));
                continue;
            }

            // Fallback: skip unrecognized lines
            rust_lines.push(format!(
                "{}// UNTRANSLATED: {}",
                self.indent(self.indent_level),
                trimmed
            ));
        }

        // Close any remaining open braces
        for _ in 0..indent_stack.len() {
            rust_lines.push(format!("{}}}", self.indent(self.indent_level)));
        }

        let result = rust_lines.join("\n");
        self.convert_camel_case_variables(&result)
    }

    fn convert_camel_case_variables(&self, code: &str) -> String {
        let p = patterns();
        let mut result = code.to_string();
        for (re, snake) in &p.camel_case_vars {
            result = re.replace_all(&result, *snake).to_string();
        }
        result
    }

    fn indent(&self, level: usize) -> String {
        "    ".repeat(level)
    }

    fn find_shared_variables(&self, lines: &[&str]) -> HashSet<String> {
        let mut shared_vars: HashSet<String> = HashSet::new();
        let mut vars_in_branch: HashMap<String, usize> = HashMap::new();
        let mut current_branch = 0;
        let mut if_depth = 0;
        let base_indent = lines
            .iter()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);

        let var_re = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let line_indent = line.len() - line.trim_start().len();

            if line_indent == base_indent && trimmed.starts_with("if ") && trimmed.ends_with(':') {
                current_branch += 1;
                if_depth = 1;
            }

            if line_indent > base_indent && trimmed.starts_with("if ") && trimmed.ends_with(':') {
                if_depth += 1;
            }

            if line_indent == base_indent
                && !trimmed.starts_with("if ")
                && !trimmed.starts_with("elif ")
                && trimmed != "else:"
            {
                if_depth = 0;
            }

            if line_indent == base_indent && trimmed.starts_with("return ") {
                for cap in var_re.captures_iter(trimmed) {
                    let var = cap[1].to_string();
                    if vars_in_branch.contains_key(&var) {
                        shared_vars.insert(var);
                    }
                }
            }

            if if_depth > 0
                && line_indent > base_indent
                && let Some((var, _)) = self.parse_assignment(trimmed)
            {
                let entry = vars_in_branch.entry(var.clone()).or_insert(0);
                if *entry != current_branch {
                    if *entry != 0 {
                        shared_vars.insert(var.clone());
                    }
                    *entry = current_branch;
                }
                if if_depth > 1 {
                    shared_vars.insert(var);
                }
            }
        }

        let common_vars = [
            "dps",
            "final_atk",
            "hitdmg",
            "hitdmgarts",
            "skilldmg",
            "avgdmg",
            "atk_scale",
            "skill_scale",
            "atkbuff",
            "newres",
            "critdmg",
            "cdmg",
            "crit_rate",
            "aspd",
            "bonusdmg",
            "avgphys",
            "avgarts",
            "avghit",
            "defshred",
            "atk_interval",
            "sp_cost",
            "skill_duration",
            "talent1_overwrite",
            "critdefignore",
            "extra_dmg",
            "dmg_rate",
            "necrosis_scale",
            "eledmg",
            "ele_scale",
            "burst_scale",
            "modatkbuff",
        ];

        let code_text = lines.join("\n");
        for var in common_vars {
            let var_pattern = format!(r"\b{var}\b");
            if let Ok(re) = Regex::new(&var_pattern)
                && re.is_match(&code_text)
                && vars_in_branch.len() > 1
            {
                shared_vars.insert(var.to_string());
            }
        }

        shared_vars
    }

    fn parse_assignment<'a>(&self, line: &'a str) -> Option<(String, &'a str)> {
        let p = patterns();
        let mut i = 0;
        let chars: Vec<char> = line.chars().collect();

        while i < chars.len() {
            if chars[i] == '=' {
                let prev = if i > 0 { chars[i - 1] } else { ' ' };
                let next = if i + 1 < chars.len() {
                    chars[i + 1]
                } else {
                    ' '
                };

                if prev != '='
                    && prev != '!'
                    && prev != '<'
                    && prev != '>'
                    && prev != '+'
                    && prev != '-'
                    && prev != '*'
                    && prev != '/'
                    && next != '='
                {
                    let var_part = line[..i].trim();
                    let mut expr_part = line[i + 1..].trim();

                    let double_assign_prefix = format!("{var_part} = ");
                    if expr_part.starts_with(&double_assign_prefix) {
                        expr_part = &expr_part[double_assign_prefix.len()..];
                    }

                    if p.identifier.is_match(var_part) {
                        let var_name = self.rename_reserved_keyword(var_part);
                        return Some((var_name, expr_part));
                    }

                    // Handle self.atk_interval = X as atk_interval = X
                    if var_part == "self.atk_interval" {
                        return Some(("atk_interval".to_string(), expr_part));
                    }

                    // Handle self.skill_duration = X as skill_duration = X
                    if var_part == "self.skill_duration" {
                        return Some(("skill_duration".to_string(), expr_part));
                    }
                }
            }
            i += 1;
        }
        None
    }

    fn rename_reserved_keyword(&self, name: &str) -> String {
        match name {
            "crate" => "crit_rate".to_string(),
            "type" => "dmg_type".to_string(),
            "match" => "match_val".to_string(),
            "mod" => "module".to_string(),
            _ => name.to_string(),
        }
    }

    fn parse_compound_assignment<'a>(&self, line: &'a str) -> Option<(String, &'a str, &'a str)> {
        let p = patterns();
        if let Some(caps) = p.compound_assign.captures(line) {
            let var_raw = caps.get(1)?.as_str();
            let var = self.rename_reserved_keyword(var_raw);
            let op = caps.get(2)?.as_str();
            let expr = caps.get(3)?.as_str();
            return Some((var, op, expr));
        }
        None
    }

    /// Parse a for loop like "for i in range(count):" or "for i in range(int(duration)):"
    fn parse_for_range(&self, line: &str) -> Option<(String, String)> {
        // Match: for VAR in range(EXPR):
        let re = Regex::new(r"^for\s+(\w+)\s+in\s+range\((.+)\):$").ok()?;
        let caps = re.captures(line)?;

        let loop_var = caps.get(1)?.as_str().to_string();
        let range_expr = caps.get(2)?.as_str().to_string();

        Some((loop_var, range_expr))
    }

    /// Extract array access from an expression like "self.skill_params[3]"
    /// Returns (rust_array_name, index) like ("skill_parameters", "3")
    fn extract_array_access(&self, expr: &str) -> Option<(String, String)> {
        // Map Python names to Rust names
        let mappings = [
            ("self.skill_params", "skill_parameters"),
            ("self.talent1_params", "talent1_parameters"),
            ("self.talent2_params", "talent2_parameters"),
            ("self.shreds", "shreds"),
        ];

        for (py_name, rust_name) in &mappings {
            if expr.starts_with(py_name) {
                // Extract the index from [N]
                let re = Regex::new(&format!(r"^{}\[(\d+)\]$", regex::escape(py_name))).ok()?;
                if let Some(caps) = re.captures(expr) {
                    let idx = caps.get(1)?.as_str().to_string();
                    return Some((rust_name.to_string(), idx));
                }
            }
        }
        None
    }

    /// Translate Python function calls like `int(expr)`, `abs(expr)`, `round(expr)` to Rust
    /// Handles nested parentheses correctly
    fn translate_balanced_function_calls(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Handle int(), abs(), round() with balanced parentheses
        for (func_name, rust_suffix) in [
            ("int", ".trunc()"),
            ("abs", ".abs()"),
            ("round", ".round()"),
        ] {
            #[allow(clippy::while_let_loop)]
            loop {
                // Find the function call
                if let Some(start) = result.find(&format!("{func_name}(")) {
                    // Ensure it's not part of a larger word (e.g., "print" shouldn't match "int")
                    if start > 0 {
                        let prev_char = result.chars().nth(start - 1).unwrap_or(' ');
                        if prev_char.is_alphanumeric() || prev_char == '_' {
                            // Skip this occurrence - part of a larger word
                            // Need to prevent infinite loop - replace with placeholder temporarily
                            let remaining = &result[start + func_name.len()..];
                            result =
                                format!("{}__SKIP__{}{}", &result[..start], func_name, remaining);
                            continue;
                        }
                    }

                    let open_paren = start + func_name.len();

                    // Find matching close paren with balanced counting
                    let mut depth = 0;
                    let mut close_paren = None;
                    for (i, c) in result[open_paren..].char_indices() {
                        match c {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    close_paren = Some(open_paren + i);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Some(close) = close_paren {
                        // Extract the inner expression (without the opening/closing parens)
                        let inner = &result[open_paren + 1..close];
                        // Replace: func(inner) -> ((inner) as f64).suffix()
                        let replacement = format!("(({inner}) as f64){rust_suffix}");
                        result = format!(
                            "{}{}{}",
                            &result[..start],
                            replacement,
                            &result[close + 1..]
                        );
                    } else {
                        // No matching paren found, break to avoid infinite loop
                        break;
                    }
                } else {
                    break;
                }
            }
            // Restore any skipped occurrences
            result = result.replace(&format!("__SKIP__{func_name}"), func_name);
        }

        result
    }

    fn translate_condition(&self, condition: &str) -> String {
        let p = patterns();
        let mut result = condition.to_string();

        result = self.translate_in_operator(&result);
        result = p.and_op.replace_all(&result, " && ").to_string();
        result = p.or_op.replace_all(&result, " || ").to_string();
        result = p.not_op.replace_all(&result, "!").to_string();
        result = self.translate_self_references(&result);
        result = self.convert_comparison_integers(&result);
        result = self.convert_condition_arithmetic_integers(&result);

        result
    }

    fn convert_condition_arithmetic_integers(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.to_string();

        result = p
            .mul_int_compare
            .replace_all(&result, "*$1.0$2")
            .to_string();
        result = p
            .mul_space_int_compare
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p
            .div_int_compare
            .replace_all(&result, "/$1.0$2")
            .to_string();
        result = p
            .start_int_mul_float
            .replace_all(&result, "$1.0 * $2")
            .to_string();

        result
    }

    fn convert_comparison_integers(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.to_string();

        result = p
            .eq_ne_int
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        result = p
            .ge_le_int
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        result = p
            .gt_lt_int
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{} {}.0{}", &caps[1], &caps[2], &caps[3], &caps[4])
            })
            .to_string();

        result
    }

    fn translate_expression(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.trim().to_string();

        result = self.translate_ternary(&result);
        result = self.translate_np_fmax(&result);
        result = self.translate_np_fmin(&result);
        result = self.translate_min_max(&result);

        // Use balanced parentheses handling for int(), abs(), round()
        result = self.translate_balanced_function_calls(&result);

        result = self.translate_self_references(&result);

        result = p.and_op.replace_all(&result, " && ").to_string();
        result = p.or_op.replace_all(&result, " || ").to_string();

        result = self.translate_in_operator(&result);
        result = self.translate_power_operator(&result);
        result = result.replace("//", " / ");
        result = self.ensure_float_division(&result);
        result = self.convert_integers_to_floats(&result);

        result = p.crate_kw.replace_all(&result, "crit_rate").to_string();

        result = p
            .targets_idx
            .replace_all(&result, "[(targets) as usize]")
            .to_string();
        result = p
            .targets_sub1
            .replace_all(&result, "[((targets) as usize).saturating_sub(1)]")
            .to_string();
        result = p
            .neg_idx
            .replace_all(
                &result,
                "[(self.unit.talent1_parameters.len() as isize - $1 as isize) as usize]",
            )
            .to_string();

        result = self.convert_array_elements_to_floats(&result);

        // Add bounds checking for array accesses
        result = self.add_bounds_checking(&result);

        result
    }

    /// Add bounds checking for array accesses to prevent panics
    fn add_bounds_checking(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.to_string();

        // talent1_parameters[N] -> self.unit.talent1_parameters.get(N).copied().unwrap_or(0.0)
        result = p
            .talent1_idx
            .replace_all(&result, |caps: &regex::Captures| {
                format!(
                    "self.unit.talent1_parameters.get({}).copied().unwrap_or(0.0)",
                    &caps[1]
                )
            })
            .to_string();

        // talent2_parameters[N] -> self.unit.talent2_parameters.get(N).copied().unwrap_or(0.0)
        result = p
            .talent2_idx
            .replace_all(&result, |caps: &regex::Captures| {
                format!(
                    "self.unit.talent2_parameters.get({}).copied().unwrap_or(0.0)",
                    &caps[1]
                )
            })
            .to_string();

        // skill_parameters[N] -> self.unit.skill_parameters.get(N).copied().unwrap_or(0.0)
        result = p
            .skill_params_idx
            .replace_all(&result, |caps: &regex::Captures| {
                format!(
                    "self.unit.skill_parameters.get({}).copied().unwrap_or(0.0)",
                    &caps[1]
                )
            })
            .to_string();

        // shreds[N] -> self.unit.shreds.get(N).copied().unwrap_or(0.0)
        result = p
            .shreds_idx
            .replace_all(&result, |caps: &regex::Captures| {
                format!("self.unit.shreds.get({}).copied().unwrap_or(0.0)", &caps[1])
            })
            .to_string();

        result
    }

    fn convert_array_elements_to_floats(&self, expr: &str) -> String {
        let p = patterns();
        let chars: Vec<char> = expr.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '[' {
                let start = i;
                let mut depth = 1;
                let mut j = i + 1;
                while j < chars.len() && depth > 0 {
                    if chars[j] == '[' {
                        depth += 1;
                    }
                    if chars[j] == ']' {
                        depth -= 1;
                    }
                    j += 1;
                }

                if depth == 0 {
                    let array_content: String = chars[start + 1..j - 1].iter().collect();

                    if array_content.contains(',')
                        && (array_content
                            .trim()
                            .starts_with(|c: char| c.is_ascii_digit())
                            || array_content.trim().starts_with('-'))
                    {
                        let elements: Vec<&str> = array_content.split(',').collect();
                        let float_elements: Vec<String> = elements
                            .iter()
                            .map(|e| {
                                let trimmed = e.trim();
                                if p.bare_int_re.is_match(trimmed) {
                                    format!("{trimmed}.0")
                                } else if p.int_mul_div_re.is_match(trimmed) {
                                    p.int_mul_div_replace.replace(trimmed, "$1.0$2").to_string()
                                } else {
                                    trimmed.to_string()
                                }
                            })
                            .collect();
                        result.push('[');
                        result.push_str(&float_elements.join(", "));
                        result.push(']');
                        i = j;
                        continue;
                    }
                }
                result.push(chars[i]);
                i += 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    fn convert_integers_to_floats(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.to_string();

        result = p
            .mul_int_end
            .replace_all(&result, |caps: &regex::Captures| {
                format!("* {}.0", &caps[1])
            })
            .to_string();

        result = p
            .div_int_end
            .replace_all(&result, |caps: &regex::Captures| {
                format!("/ {}.0", &caps[1])
            })
            .to_string();

        if p.bare_int.is_match(&result) {
            result = format!("{result}.0");
        }

        result = p.start_int_mul.replace_all(&result, "$1.0 *").to_string();
        result = p.start_int_div.replace_all(&result, "$1.0 /").to_string();
        result = p.start_int_add.replace_all(&result, "$1.0 +").to_string();
        result = p.start_int_sub.replace_all(&result, "$1.0 -").to_string();

        result = p
            .eq_ne_int
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        result = p
            .ge_le_int
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        result = p
            .gt_lt_int
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{} {}.0{}", &caps[1], &caps[2], &caps[3], &caps[4])
            })
            .to_string();

        result = p.eq_int_semi.replace_all(&result, "= $1.0;").to_string();
        result = p
            .brace_int_brace
            .replace_all(&result, "{ $1.0 }")
            .to_string();
        result = p.add_int_space.replace_all(&result, "$1$2.0$3").to_string();
        result = p.add_int_end.replace_all(&result, "$1$2.0").to_string();
        result = p.sub_int_end.replace_all(&result, "$1$2.0").to_string();
        result = p.mul_int_end2.replace_all(&result, "$1$2.0").to_string();
        result = p.div_int_end2.replace_all(&result, "$1$2.0").to_string();
        result = p.sub_int_space.replace_all(&result, "$1$2.0$3").to_string();
        result = p.mul_int_space.replace_all(&result, "$1$2.0$3").to_string();
        result = p
            .mul_int_compare2
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p
            .mul_nospace_compare
            .replace_all(&result, "*$1.0$2")
            .to_string();
        result = p
            .mul_int_add_sub
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p
            .mul_nospace_add_sub
            .replace_all(&result, "*$1.0$2")
            .to_string();
        result = p.mul_int_lt.replace_all(&result, "* $1.0 <").to_string();
        result = p.mod_int.replace_all(&result, "$1$2.0").to_string();
        result = p.brace_int_mul.replace_all(&result, "{ $1.0 *").to_string();
        result = p.brace_int_div.replace_all(&result, "{ $1.0 /").to_string();
        result = p.eq_int_mul.replace_all(&result, "= $1.0 *").to_string();
        result = p.eq_int_div.replace_all(&result, "= $1.0 /").to_string();
        result = p
            .int_mul_float
            .replace_all(&result, "$1$2.0 * $3")
            .to_string();
        result = p
            .if_int_mul_float
            .replace_all(&result, "if $1.0 * $2")
            .to_string();
        result = p
            .add_eq_int_mul
            .replace_all(&result, "+= $1.0 *")
            .to_string();
        result = p
            .sub_eq_int_mul
            .replace_all(&result, "-= $1.0 *")
            .to_string();
        result = p
            .mul_eq_int_semi
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p
            .div_eq_int_semi
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p
            .add_eq_int_semi
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p
            .sub_eq_int_semi
            .replace_all(&result, "$1$2.0$3")
            .to_string();
        result = p.paren_int_add.replace_all(&result, "($1.0 +").to_string();
        result = p.paren_int_sub.replace_all(&result, "($1.0 -").to_string();
        result = p.paren_int_mul.replace_all(&result, "($1.0 *").to_string();
        result = p.paren_int_div.replace_all(&result, "($1.0 /").to_string();

        result
    }

    fn translate_ternary(&self, expr: &str) -> String {
        if !expr.contains(" if ") || !expr.contains(" else ") {
            return expr.to_string();
        }

        if let Some(else_pos) = expr.rfind(" else ") {
            let before_else = &expr[..else_pos];
            let after_else = &expr[else_pos + 6..];

            if let Some(if_pos) = before_else.rfind(" if ") {
                let true_val = &before_else[..if_pos];
                let condition = &before_else[if_pos + 4..];
                let false_val = after_else;

                let rust_cond = self.translate_condition(condition);
                let rust_true = self.translate_expression(true_val);
                let rust_false = self.translate_expression(false_val);

                return format!("if {rust_cond} {{ {rust_true} }} else {{ {rust_false} }}");
            }
        }

        expr.to_string()
    }

    fn translate_np_fmax(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        while let Some(start) = result.find("np.fmax(") {
            let args_start = start + 8;
            let mut depth = 1;
            let mut end = args_start;
            let chars: Vec<char> = result[args_start..].chars().collect();
            for (i, c) in chars.iter().enumerate() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = args_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let args_str = &result[args_start..end];
            if let Some((arg1, arg2)) = self.split_args(args_str) {
                let replacement =
                    format!("(({}) as f64).max(({}) as f64)", arg1.trim(), arg2.trim());
                result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
            } else {
                break;
            }
        }

        result
    }

    fn translate_np_fmin(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        while let Some(start) = result.find("np.fmin(") {
            let args_start = start + 8;
            let mut depth = 1;
            let mut end = args_start;
            let chars: Vec<char> = result[args_start..].chars().collect();
            for (i, c) in chars.iter().enumerate() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = args_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let args_str = &result[args_start..end];
            if let Some((arg1, arg2)) = self.split_args(args_str) {
                let replacement =
                    format!("(({}) as f64).min(({}) as f64)", arg1.trim(), arg2.trim());
                result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
            } else {
                break;
            }
        }

        result
    }

    fn split_args(&self, args: &str) -> Option<(String, String)> {
        let mut depth = 0;
        let chars: Vec<char> = args.chars().collect();

        for (i, c) in chars.iter().enumerate() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                ',' if depth == 0 => {
                    return Some((
                        args[..i].trim().to_string(),
                        args[i + 1..].trim().to_string(),
                    ));
                }
                _ => {}
            }
        }
        None
    }

    fn translate_min_max(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.to_string();

        while let Some(m) = p.min_func.find(&result) {
            let (start, args_start) = if result[m.start()..].starts_with("min(") {
                (m.start(), m.start() + 4)
            } else {
                (m.start() + 1, m.start() + 5)
            };

            let mut depth = 1;
            let mut end = args_start;
            let chars: Vec<char> = result[args_start..].chars().collect();
            for (i, c) in chars.iter().enumerate() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = args_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let args_str = &result[args_start..end];
            if let Some((arg1, arg2)) = self.split_args(args_str) {
                let replacement =
                    format!("(({}) as f64).min(({}) as f64)", arg1.trim(), arg2.trim());
                result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
            } else {
                break;
            }
        }

        while let Some(m) = p.max_func.find(&result) {
            let (start, args_start) = if result[m.start()..].starts_with("max(") {
                (m.start(), m.start() + 4)
            } else {
                (m.start() + 1, m.start() + 5)
            };

            let mut depth = 1;
            let mut end = args_start;
            let chars: Vec<char> = result[args_start..].chars().collect();
            for (i, c) in chars.iter().enumerate() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end = args_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let args_str = &result[args_start..end];
            if let Some((arg1, arg2)) = self.split_args(args_str) {
                let replacement =
                    format!("(({}) as f64).max(({}) as f64)", arg1.trim(), arg2.trim());
                result = format!("{}{}{}", &result[..start], replacement, &result[end + 1..]);
            } else {
                break;
            }
        }

        result
    }

    fn translate_self_references(&self, expr: &str) -> String {
        let p = patterns();
        let mut result = expr.to_string();

        result = p
            .self_skill
            .replace_all(&result, "self.unit.skill_index")
            .to_string();
        result = p
            .self_module
            .replace_all(&result, "self.unit.module_index")
            .to_string();
        result = p
            .self_elite
            .replace_all(&result, "self.unit.elite")
            .to_string();

        for (re, replacement) in &p.self_simple_mappings {
            result = re.replace_all(&result, replacement.as_str()).to_string();
        }

        result = p
            .skill_index
            .replace_all(&result, "(self.unit.skill_index as f64)")
            .to_string();
        result = p
            .elite_ref
            .replace_all(&result, "(self.unit.elite as f64)")
            .to_string();
        result = p
            .module_index
            .replace_all(&result, "(self.unit.module_index as f64)")
            .to_string();
        result = p
            .module_level
            .replace_all(&result, "(self.unit.module_level as f64)")
            .to_string();
        // Translate self.atk_interval based on whether the code modifies it
        // If modified (e.g., self.atk_interval = 0.5), use local variable for subsequent reads
        // If not modified, use the struct field directly
        if self.modifies_self_atk_interval {
            result = p
                .atk_interval
                .replace_all(&result, "atk_interval")
                .to_string();
        } else {
            result = p
                .atk_interval
                .replace_all(&result, "(self.unit.attack_interval as f64)")
                .to_string();
        }
        result = p
            .skill_cost
            .replace_all(&result, "(self.unit.skill_cost as f64)")
            .to_string();
        result = p
            .targets_ref
            .replace_all(&result, "(self.unit.targets as f64)")
            .to_string();
        result = p
            .sp_boost
            .replace_all(&result, "(self.unit.sp_boost as f64)")
            .to_string();
        result = p
            .drone_atk_interval
            .replace_all(&result, "(self.unit.drone_atk_interval as f64)")
            .to_string();
        result = p.crate_kw.replace_all(&result, "crit_rate").to_string();

        // Specific cloned_op replacements for Ela defaults (must run before generic patterns)
        // Ela E2 90 with trust 100 and pot 3+: ATK=588+80+27=695, interval=0.85, ranged, physical
        result = p
            .cloned_op_atk
            .replace_all(&result, "cloned_op_atk")
            .to_string();
        result = p
            .cloned_op_atk_interval
            .replace_all(&result, "cloned_op_atk_interval")
            .to_string();
        result = p
            .cloned_op_ranged
            .replace_all(&result, "cloned_op_ranged")
            .to_string();
        result = p
            .cloned_op_physical
            .replace_all(&result, "cloned_op_physical")
            .to_string();
        // Generic cloned_op fallbacks
        result = p
            .cloned_op_bool
            .replace_all(&result, "false /* cloned_op.$1 */ ")
            .to_string();
        result = p
            .cloned_op_num
            .replace_all(&result, "0.0 /* cloned_op.$1 */ ")
            .to_string();
        // hits defaults to 0 in Python (no additional hits unless specified)
        result = p
            .self_hits
            .replace_all(&result, "0.0 /* self.hits - defaults to 0 */")
            .to_string();
        result = p
            .self_pot
            .replace_all(&result, "(self.unit.potential as f64)")
            .to_string();
        result = p
            .self_freeze_rate
            .replace_all(
                &result,
                "0.0 /* self.freezeRate - needs manual implementation */",
            )
            .to_string();
        result = p
            .self_count
            .replace_all(
                &result,
                "1.0 /* self.count - needs manual implementation */",
            )
            .to_string();
        result = p
            .self_below50
            .replace_all(
                &result,
                "false /* self.below50 - needs manual implementation */",
            )
            .to_string();
        result = p
            .self_ammo
            .replace_all(&result, "1.0 /* self.ammo - needs manual implementation */")
            .to_string();
        result = p
            .self_shadows
            .replace_all(
                &result,
                "1.0 /* self.shadows - needs manual implementation */",
            )
            .to_string();
        result = p
            .self_params
            .replace_all(
                &result,
                "1.0 /* self.params - needs manual implementation */",
            )
            .to_string();
        result = p
            .self_params2
            .replace_all(
                &result,
                "1.0 /* self.params2 - needs manual implementation */",
            )
            .to_string();
        result = p
            .self_no_kill
            .replace_all(
                &result,
                "false /* self.no_kill - needs manual implementation */",
            )
            .to_string();

        result = p
            .min_slice_from
            .replace_all(
                &result,
                "self.unit.$1[$2..].iter().cloned().fold(f64::INFINITY, f64::min)",
            )
            .to_string();
        result = p
            .max_slice_to
            .replace_all(
                &result,
                "self.unit.$1[..$2].iter().cloned().fold(f64::NEG_INFINITY, f64::max)",
            )
            .to_string();
        result = p
            .min_slice_to
            .replace_all(
                &result,
                "self.unit.$1[..$2].iter().cloned().fold(f64::INFINITY, f64::min)",
            )
            .to_string();
        result = p
            .max_slice_from
            .replace_all(
                &result,
                "self.unit.$1[$2..].iter().cloned().fold(f64::NEG_INFINITY, f64::max)",
            )
            .to_string();
        result = p
            .max_single
            .replace_all(
                &result,
                "self.unit.$1.iter().cloned().fold(f64::NEG_INFINITY, f64::max)",
            )
            .to_string();
        result = p
            .min_single
            .replace_all(
                &result,
                "self.unit.$1.iter().cloned().fold(f64::INFINITY, f64::min)",
            )
            .to_string();

        result
    }

    fn translate_power_operator(&self, expr: &str) -> String {
        let mut result = expr.to_string();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > 100 {
                eprintln!("POWER OPERATOR INFINITE LOOP DETECTED! expr={expr}");
                break;
            }
            if let Some(pos) = result.find("**") {
                let before = &result[..pos];
                let after_raw = &result[pos + 2..];
                let before_trimmed = before.trim_end();
                let after = after_raw.trim_start();

                let base_start_in_trimmed;
                let base;
                let chars: Vec<char> = before_trimmed.chars().collect();

                if chars.last() == Some(&')') {
                    let mut depth = 0;
                    let mut start_idx = chars.len();
                    for (i, c) in chars.iter().enumerate().rev() {
                        match c {
                            ')' => depth += 1,
                            '(' => {
                                depth -= 1;
                                if depth == 0 {
                                    start_idx = i;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    base_start_in_trimmed = start_idx;
                    base = &before_trimmed[base_start_in_trimmed..];
                } else {
                    base_start_in_trimmed = before_trimmed
                        .rfind(|c: char| {
                            c == ' '
                                || c == '('
                                || c == '+'
                                || c == '-'
                                || c == '*'
                                || c == '/'
                                || c == ','
                        })
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    base = &before_trimmed[base_start_in_trimmed..];
                }

                let exponent;
                let exp_end;
                if after.starts_with('(') {
                    let mut depth = 0;
                    let mut end_idx = 0;
                    for (i, c) in after.chars().enumerate() {
                        match c {
                            '(' => depth += 1,
                            ')' => {
                                depth -= 1;
                                if depth == 0 {
                                    end_idx = i + 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    exp_end = end_idx;
                    exponent = &after[..exp_end];
                } else {
                    exp_end = after
                        .find(|c: char| {
                            c == ' '
                                || c == ')'
                                || c == '+'
                                || c == '-'
                                || c == '*'
                                || c == '/'
                                || c == ','
                                || c == ']'
                                || c == ';'
                        })
                        .unwrap_or(after.len());
                    exponent = &after[..exp_end];
                }

                if base.trim().is_empty() || exponent.trim().is_empty() {
                    break;
                }

                let replacement =
                    format!("({} as f64).powf({} as f64)", base.trim(), exponent.trim());
                let before_prefix = &before[..before.len()
                    - (before.len() - before_trimmed.len())
                    - (before_trimmed.len() - base_start_in_trimmed)];
                let after_suffix = &after[exp_end..];
                result = format!("{before_prefix}{replacement}{after_suffix}");
            } else {
                break;
            }
        }

        result
    }

    fn translate_in_operator(&self, expr: &str) -> String {
        let p = patterns();
        p.in_list
            .replace_all(expr, |caps: &regex::Captures| {
                let item = &caps[1];
                let list_items = &caps[2];
                let float_items: Vec<String> = list_items
                    .split(',')
                    .map(|e| {
                        let trimmed = e.trim();
                        if p.bare_int_re.is_match(trimmed) {
                            format!("{trimmed}.0")
                        } else {
                            trimmed.to_string()
                        }
                    })
                    .collect();
                format!(
                    "[{}].contains(&(({}) as f64))",
                    float_items.join(", "),
                    item
                )
            })
            .to_string()
    }

    fn ensure_float_division(&self, expr: &str) -> String {
        let p = patterns();
        p.div_bare_int
            .replace_all(expr, |caps: &regex::Captures| {
                format!("/ {}.0{}", &caps[1], &caps[2])
            })
            .to_string()
    }
}

/// Parse the Python file and extract all operator classes
fn parse_python_file(path: &Path) -> Result<Vec<OperatorClass>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

    let mut operators = Vec::new();
    let class_pattern = Regex::new(r"(?m)^class\s+(\w+)\(Operator\):").unwrap();
    let super_pattern = Regex::new(
        r#"super\(\)\.__init__\(\s*"([^"]+)"\s*,\s*pp\s*,\s*\[([^\]]*)\]\s*,\s*\[([^\]]*)\](?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?"#,
    )
    .unwrap();

    let class_positions: Vec<(usize, &str)> = class_pattern
        .captures_iter(&content)
        .filter_map(|caps| {
            let full_match = caps.get(0)?;
            let name = caps.get(1)?.as_str();
            Some((full_match.start(), name))
        })
        .collect();

    for (idx, (start_pos, class_name)) in class_positions.iter().enumerate() {
        if matches!(
            *class_name,
            "Operator" | "AttackSpeed" | "NewBlueprint" | "Guide" | "Defense" | "Res"
        ) {
            continue;
        }

        let end_pos = if idx + 1 < class_positions.len() {
            class_positions[idx + 1].0
        } else {
            content.len()
        };

        let class_body = &content[*start_pos..end_pos];

        let (
            json_name,
            available_skills,
            available_modules,
            default_skill,
            default_pot,
            default_mod,
        ) = if let Some(caps) = super_pattern.captures(class_body) {
            let json_name = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or(class_name)
                .to_string();
            let skills = parse_int_list(caps.get(2).map(|m| m.as_str()).unwrap_or(""));
            let modules = parse_int_list(caps.get(3).map(|m| m.as_str()).unwrap_or(""));
            let def_skill = caps
                .get(4)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(3);
            let def_pot = caps
                .get(5)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1);
            let def_mod = caps
                .get(6)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1);
            (json_name, skills, modules, def_skill, def_pot, def_mod)
        } else {
            (class_name.to_string(), vec![], vec![], 3, 1, 1)
        };

        let init_body = extract_method_body(class_body, "__init__");
        let skill_dps_body = extract_method_body(class_body, "skill_dps");
        let total_dmg_body = extract_method_body(class_body, "total_dmg");
        let has_total_dmg_override = !total_dmg_body.is_empty();
        let init_modifications = parse_init_modifications(&init_body);

        operators.push(OperatorClass {
            class_name: class_name.to_string(),
            json_name,
            available_skills,
            available_modules,
            default_skill,
            default_pot,
            default_mod,
            init_body,
            skill_dps_body,
            has_total_dmg_override,
            total_dmg_body,
            init_modifications,
        });
    }

    Ok(operators)
}

fn extract_method_body(class_body: &str, method_name: &str) -> String {
    let method_pattern =
        Regex::new(&format!(r"(?m)^[ \t]+def\s+{method_name}\s*\([^)]*\):")).unwrap();

    if let Some(method_match) = method_pattern.find(class_body) {
        let method_start = method_match.end();
        let remaining = &class_body[method_start..];
        let method_text = method_match.as_str();
        let method_indent = method_text
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();

        let mut body_lines = Vec::new();
        for line in remaining.lines() {
            if line.trim().is_empty() {
                body_lines.push(line.to_string());
                continue;
            }

            let line_indent = line.chars().take_while(|c| c.is_whitespace()).count();

            if line_indent <= method_indent && line.trim().starts_with("def ") {
                break;
            }

            if line_indent <= method_indent && !line.trim().is_empty() {
                break;
            }
            body_lines.push(line.to_string());
        }

        while body_lines
            .last()
            .map(|l| l.trim().is_empty())
            .unwrap_or(false)
        {
            body_lines.pop();
        }

        body_lines.join("\n")
    } else {
        String::new()
    }
}

fn parse_int_list(s: &str) -> Vec<i32> {
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

/// Parse init_body to extract field modifications like:
/// - self.ammo = 4 + 4 * self.skill
/// - self.drone_atk += 100
/// - if condition: self.field = value
fn parse_init_modifications(init_body: &str) -> Vec<InitModification> {
    let mut modifications = Vec::new();

    // Fields we care about modifying
    let tracked_fields = [
        "ammo",
        "drone_atk",
        "drone_atk_interval",
        "atk",
        "attack_speed",
        "below50",
        "shadows",
    ];

    // Pattern for simple assignment: self.field = expr
    let assign_pattern = Regex::new(r"self\.(\w+)\s*=\s*(.+)$").unwrap();
    // Pattern for compound assignment: self.field += expr
    let compound_pattern = Regex::new(r"self\.(\w+)\s*(\+=|-=|\*=)\s*(.+)$").unwrap();
    // Pattern for inline if: if condition: statement
    let inline_if_pattern = Regex::new(r"^if\s+(.+?):\s+(.+)$").unwrap();

    // Stack of (condition, indent) for nested conditions
    let mut condition_stack: Vec<(String, usize)> = Vec::new();

    /// Helper function to get combined condition from stack
    fn get_combined_condition(stack: &[(String, usize)]) -> Option<String> {
        if stack.is_empty() {
            None
        } else {
            Some(
                stack
                    .iter()
                    .map(|(c, _)| c.as_str())
                    .collect::<Vec<_>>()
                    .join(" and "),
            )
        }
    }

    /// Helper to process an assignment statement and add to modifications
    fn process_statement(
        statement: &str,
        condition: Option<String>,
        tracked_fields: &[&str],
        assign_pattern: &Regex,
        compound_pattern: &Regex,
        modifications: &mut Vec<InitModification>,
    ) {
        let statement = statement.trim();

        // Check for compound assignment (+=, -=, *=)
        if let Some(caps) = compound_pattern.captures(statement) {
            let field = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let op = caps.get(2).map(|m| m.as_str()).unwrap_or("+=");
            let expr = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            if tracked_fields.contains(&field) && !expr.is_empty() {
                modifications.push(InitModification {
                    field: field.to_string(),
                    operation: op.to_string(),
                    expression: expr.to_string(),
                    condition,
                });
            }
            return;
        }

        // Check for simple assignment (=)
        if let Some(caps) = assign_pattern.captures(statement) {
            let field = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let expr = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            if tracked_fields.contains(&field) && !expr.is_empty() {
                modifications.push(InitModification {
                    field: field.to_string(),
                    operation: "=".to_string(),
                    expression: expr.to_string(),
                    condition,
                });
            }
        }
    }

    for line in init_body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip lines that are part of name construction or other logic
        if trimmed.contains("self.name")
            || trimmed.starts_with("super()")
            || trimmed.starts_with("else:")
        {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        // Pop conditions from stack if we've outdented
        while let Some((_, cond_indent)) = condition_stack.last() {
            if line_indent <= *cond_indent {
                condition_stack.pop();
            } else {
                break;
            }
        }

        // Check for inline if: if condition: statement
        if let Some(caps) = inline_if_pattern.captures(trimmed) {
            let inline_cond = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let statement = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            // If "statement" is just a comment, treat this as a multi-line if with trailing comment
            if statement.trim().starts_with('#') {
                condition_stack.push((inline_cond.to_string(), line_indent));
                continue;
            }

            // Combine with parent conditions
            let full_condition = if condition_stack.is_empty() {
                Some(inline_cond.to_string())
            } else {
                let parent = get_combined_condition(&condition_stack).unwrap();
                Some(format!("{parent} and {inline_cond}"))
            };

            process_statement(
                statement,
                full_condition,
                &tracked_fields,
                &assign_pattern,
                &compound_pattern,
                &mut modifications,
            );
            continue;
        }

        // Check for multi-line if: if condition:
        if trimmed.starts_with("if ") && trimmed.ends_with(':') {
            // Extract condition (remove "if " prefix and ":" suffix)
            let cond = trimmed
                .strip_prefix("if ")
                .and_then(|s| s.strip_suffix(':'))
                .unwrap_or("")
                .to_string();

            condition_stack.push((cond, line_indent));
            continue;
        }

        // Regular statement (not an if)
        let current_condition = get_combined_condition(&condition_stack);

        process_statement(
            trimmed,
            current_condition,
            &tracked_fields,
            &assign_pattern,
            &compound_pattern,
            &mut modifications,
        );
    }

    modifications
}

/// Returns true if an operator should skip init ATK modifications
/// because OperatorUnit already handles module ATK from data
fn should_skip_init_atk_modification(class_name: &str) -> bool {
    matches!(
        class_name,
        // Nian: Python comment says "module lvl 1 does not come with an atk increase,
        // breaking the automatic system" but our data has correct values
        "Nian"
    )
}

/// Generate Rust code for init modifications
fn generate_init_modifications_code(
    class_name: &str,
    modifications: &[InitModification],
) -> String {
    if modifications.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("        // Apply init-time modifications from Python __init__".to_string());

    let skip_atk = should_skip_init_atk_modification(class_name);

    for modification in modifications {
        let rust_field = match modification.field.as_str() {
            "ammo" => "ammo",
            "drone_atk" => "drone_atk",
            "drone_atk_interval" => "drone_atk_interval",
            "atk" => {
                if skip_atk {
                    continue; // Skip ATK modifications for this operator
                }
                "atk"
            }
            "attack_speed" => "attack_speed",
            _ => continue,
        };

        // Translate the expression from Python to Rust
        let rust_expr = translate_init_expression(&modification.expression);

        let code = match modification.operation.as_str() {
            "=" => format!("unit.{rust_field} = {rust_expr};"),
            "+=" => format!("unit.{rust_field} += {rust_expr};"),
            "-=" => format!("unit.{rust_field} -= {rust_expr};"),
            "*=" => format!("unit.{rust_field} *= {rust_expr};"),
            _ => continue,
        };

        if let Some(condition) = &modification.condition {
            let rust_cond = translate_init_condition(condition);
            lines.push(format!("        if {rust_cond} {{ {code} }}"));
        } else {
            lines.push(format!("        {code}"));
        }
    }

    lines.join("\n")
}

/// Translate a Python expression from __init__ to Rust
fn translate_init_expression(expr: &str) -> String {
    // Strip Python comments
    let expr = if let Some(hash_pos) = expr.find('#') {
        expr[..hash_pos].trim()
    } else {
        expr.trim()
    };

    let mut result = expr.to_string();

    // Replace Python self.field references with unit.field
    result = result.replace("self.skill", "(unit.skill_index as f64)");
    result = result.replace("self.module_lvl", "(unit.module_level as f64)");
    result = result.replace("self.module", "(unit.module_index as f64)");
    result = result.replace("self.elite", "(unit.elite as f64)");

    // Handle integer literals for float fields
    let int_pattern = Regex::new(r"\b(\d+)\b").unwrap();
    result = int_pattern.replace_all(&result, "${1}.0").to_string();

    // Fix cases like "1.0.0" from double replacement
    result = result.replace(".0.0", ".0");

    result
}

/// Translate a Python condition from __init__ to Rust
fn translate_init_condition(condition: &str) -> String {
    let mut result = condition.to_string();

    // Handle "not " prefix (Python negation)
    result = result.replace("not ", "!");

    // Replace Python self.field references for conditionals
    // Order matters - replace longer patterns first
    result = result.replace("self.talent2_dmg", "unit.talent2_damage");
    result = result.replace("self.talent_dmg", "unit.talent_damage");
    result = result.replace("self.trait_dmg", "unit.trait_damage");
    result = result.replace("self.skill_dmg", "unit.skill_damage");
    result = result.replace("self.module_dmg", "unit.module_damage");

    // Replace other self.field references
    result = result.replace("self.skill", "unit.skill_index");
    result = result.replace("self.module_lvl", "unit.module_level");
    result = result.replace("self.module", "unit.module_index");
    result = result.replace("self.elite", "unit.elite");

    // Replace pp.pot (parameter potential) with unit.potential
    result = result.replace("pp.pot", "unit.potential");

    // Replace Python operators
    result = result.replace(" and ", " && ");
    result = result.replace(" or ", " || ");

    // Handle "in [x, y, z]" patterns
    let in_list_pattern = Regex::new(r"(\w+(?:\.\w+)*)\s+in\s+\[([^\]]+)\]").unwrap();
    result = in_list_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            let var = &caps[1];
            let items: Vec<&str> = caps[2].split(',').map(|s| s.trim()).collect();
            let conditions: Vec<String> = items.iter().map(|i| format!("{var} == {i}")).collect();
            format!("({})", conditions.join(" || "))
        })
        .to_string();

    result
}

/// Generate Rust code to compute `below50` from __init__ modifications
fn generate_below50_code(modifications: &[InitModification]) -> String {
    let below50_mods: Vec<_> = modifications
        .iter()
        .filter(|m| m.field == "below50")
        .collect();

    if below50_mods.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("        // Calculate below50 from __init__ logic".to_string());
    lines.push("        let mut below50 = false;".to_string());

    for modification in below50_mods {
        let rust_expr = translate_skill_dps_init_expr(&modification.expression);
        let code = format!("below50 = {rust_expr};");

        if let Some(condition) = &modification.condition {
            let rust_cond = translate_skill_dps_init_cond(condition);
            lines.push(format!("        if {rust_cond} {{ {code} }}"));
        } else {
            lines.push(format!("        {code}"));
        }
    }
    lines.push(String::new());

    lines.join("\n")
}

/// Generate Rust code to compute `shadows` from __init__ modifications
fn generate_shadows_code(modifications: &[InitModification]) -> String {
    let shadows_mods: Vec<_> = modifications
        .iter()
        .filter(|m| m.field == "shadows")
        .collect();

    if shadows_mods.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("        // Calculate shadows from __init__ logic".to_string());
    lines.push("        let mut shadows: f64 = 1.0;".to_string());

    for modification in shadows_mods {
        let rust_expr = translate_skill_dps_init_expr(&modification.expression);
        let code = match modification.operation.as_str() {
            "=" => format!("shadows = {rust_expr};"),
            "+=" => format!("shadows += {rust_expr};"),
            "-=" => format!("shadows -= {rust_expr};"),
            _ => continue,
        };

        if let Some(condition) = &modification.condition {
            let rust_cond = translate_skill_dps_init_cond(condition);
            lines.push(format!("        if {rust_cond} {{ {code} }}"));
        } else {
            lines.push(format!("        {code}"));
        }
    }
    lines.push(String::new());

    lines.join("\n")
}

/// Generate Rust code to compute `ammo` from __init__ modifications
fn generate_ammo_code(modifications: &[InitModification]) -> String {
    let ammo_mods: Vec<_> = modifications.iter().filter(|m| m.field == "ammo").collect();

    if ammo_mods.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("        // Calculate ammo from __init__ logic".to_string());
    lines.push("        let mut ammo: f64 = 1.0;".to_string());

    for modification in ammo_mods {
        let rust_expr = translate_skill_dps_init_expr(&modification.expression);
        let code = match modification.operation.as_str() {
            "=" => format!("ammo = {rust_expr};"),
            "+=" => format!("ammo += {rust_expr};"),
            "-=" => format!("ammo -= {rust_expr};"),
            _ => continue,
        };

        if let Some(condition) = &modification.condition {
            let rust_cond = translate_skill_dps_init_cond(condition);
            lines.push(format!("        if {rust_cond} {{ {code} }}"));
        } else {
            lines.push(format!("        {code}"));
        }
    }
    lines.push(String::new());

    lines.join("\n")
}

/// Translate Python expression from __init__ for use in skill_dps (uses self.unit instead of unit)
fn translate_skill_dps_init_expr(expr: &str) -> String {
    let expr = if let Some(hash_pos) = expr.find('#') {
        expr[..hash_pos].trim()
    } else {
        expr.trim()
    };

    let mut result = expr.to_string();

    // Handle Python ternary: `X if cond else Y` -> Rust `if cond { X } else { Y }`
    if result.contains(" if ")
        && result.contains(" else ")
        && let Some(else_pos) = result.rfind(" else ")
    {
        let before_else = &result[..else_pos];
        let after_else = &result[else_pos + 6..];

        if let Some(if_pos) = before_else.rfind(" if ") {
            let true_val = before_else[..if_pos].trim();
            let condition = before_else[if_pos + 4..].trim();
            let false_val = after_else.trim();

            let rust_cond = translate_skill_dps_init_cond(condition);
            let rust_true = translate_skill_dps_init_value(true_val);
            let rust_false = translate_skill_dps_init_value(false_val);

            return format!("if {rust_cond} {{ {rust_true} }} else {{ {rust_false} }}");
        }
    }

    // Handle min(a, b) -> ((a) as f64).min((b) as f64)
    // Note: Wrap each argument in parens to handle expressions like `skill+1` correctly
    let min_re = regex::Regex::new(r"min\(([^,]+),\s*([^)]+)\)").unwrap();
    let had_min = min_re.is_match(&result);
    result = min_re
        .replace_all(&result, |caps: &regex::Captures| {
            let arg1 = translate_skill_dps_init_value_no_float_convert(&caps[1]);
            let arg2 = translate_skill_dps_init_value_no_float_convert(&caps[2]);
            format!("(({arg1}) as f64).min(({arg2}) as f64)")
        })
        .to_string();

    // Apply value translation for remaining cases (only if min wasn't present to avoid double-processing)
    if !had_min {
        result = translate_skill_dps_init_value(&result);
    }

    result
}

/// Translate a single value (field reference, literal, etc.) for skill_dps context
fn translate_skill_dps_init_value(val: &str) -> String {
    let mut result = translate_skill_dps_init_value_no_float_convert(val);

    // Convert standalone integer literals to floats for f64 context
    // Only for whole numbers that aren't already floats (not preceded or followed by .)
    // Also skip if it looks like a type annotation (as fNN or as iNN)
    if !result.contains("get(") && !result.contains('[') {
        // Match integers not preceded by . or f/i (to avoid matching digits in floats or types)
        // and not followed by . (to avoid converting the integer part of floats)
        let int_re = regex::Regex::new(r"(^|[^.\dfiu])(\d+)([^.\d]|$)").unwrap();
        result = int_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();
    }

    result
}

/// Translate a single value without integer-to-float conversion (for use inside min/max where cast handles it)
fn translate_skill_dps_init_value_no_float_convert(val: &str) -> String {
    let val = val.trim();
    let mut result = val.to_string();

    // Replace Python self.field references with self.unit.field (order matters - more specific first)
    result = result.replace("self.skill_dmg", "self.unit.skill_damage");
    result = result.replace("self.talent2_dmg", "self.unit.talent2_damage");
    result = result.replace("self.talent_dmg", "self.unit.talent_damage");
    result = result.replace("self.trait_dmg", "self.unit.trait_damage");
    result = result.replace("self.module_dmg", "self.unit.module_damage");
    result = result.replace("self.module_lvl", "(self.unit.module_level as f64)");
    result = result.replace("self.module", "self.unit.module_index");
    result = result.replace("self.elite", "(self.unit.elite as f64)");

    // Handle self.skill_params[i] -> self.unit.skill_parameters.get(i).copied().unwrap_or(0.0)
    let skill_params_re = regex::Regex::new(r"self\.skill_params\[(\d+)\]").unwrap();
    result = skill_params_re
        .replace_all(&result, |caps: &regex::Captures| {
            format!(
                "self.unit.skill_parameters.get({}).copied().unwrap_or(0.0)",
                &caps[1]
            )
        })
        .to_string();

    // Handle self.skill separately (after skill_params to avoid conflicts) - cast to f64 for arithmetic
    result = result.replace("self.skill", "(self.unit.skill_index as f64)");

    // Replace Python boolean literals
    result = result.replace("True", "true");
    result = result.replace("False", "false");

    // Convert integer literals in arithmetic expressions to floats
    // Match integers after arithmetic operators: +N, -N, *N, /N
    // Also capture any following character to check if it's already a float
    let arith_int_re = regex::Regex::new(r"([+\-*/])\s*(\d+)(\.|$|[^.\d])").unwrap();
    result = arith_int_re
        .replace_all(&result, |caps: &regex::Captures| {
            let suffix = &caps[3];
            if suffix == "." {
                // Already a float, keep original
                format!("{}{}{}", &caps[1], &caps[2], suffix)
            } else {
                // Not a float, convert to float
                format!("{}{}.0{}", &caps[1], &caps[2], suffix)
            }
        })
        .to_string();

    result
}

/// Translate Python condition from __init__ for use in skill_dps
fn translate_skill_dps_init_cond(condition: &str) -> String {
    let mut result = condition.to_string();

    // Handle "in [x, y, z]" patterns FIRST (before other replacements)
    // Convert: self.skill in [0,3] -> (self.skill == 0 || self.skill == 3)
    let in_list_pattern = regex::Regex::new(r"(\S+)\s+in\s+\[([^\]]+)\]").unwrap();
    result = in_list_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            let var = &caps[1];
            let items: Vec<&str> = caps[2].split(',').map(|s| s.trim()).collect();
            let conditions: Vec<String> = items.iter().map(|i| format!("{var} == {i}")).collect();
            format!("({})", conditions.join(" || "))
        })
        .to_string();

    // Replace Python self.field references with self.unit.field (order matters - more specific first)
    result = result.replace("self.skill_dmg", "self.unit.skill_damage");
    result = result.replace("self.talent2_dmg", "self.unit.talent2_damage");
    result = result.replace("self.talent_dmg", "self.unit.talent_damage");
    result = result.replace("self.trait_dmg", "self.unit.trait_damage");
    result = result.replace("self.module_dmg", "self.unit.module_damage");
    result = result.replace("self.module_lvl", "self.unit.module_level");
    result = result.replace("self.module", "self.unit.module_index");
    result = result.replace("self.elite", "self.unit.elite");

    // Handle self.skill_params[i] -> self.unit.skill_parameters.get(i).copied().unwrap_or(0.0)
    let skill_params_re = regex::Regex::new(r"self\.skill_params\[(\d+)\]").unwrap();
    result = skill_params_re
        .replace_all(&result, |caps: &regex::Captures| {
            format!(
                "self.unit.skill_parameters.get({}).copied().unwrap_or(0.0)",
                &caps[1]
            )
        })
        .to_string();

    // Handle self.skill separately (after skill_params to avoid conflicts)
    result = result.replace("self.skill", "self.unit.skill_index");

    // Handle "not " prefix for negation
    result = result.replace("not ", "!");

    // Replace Python operators
    result = result.replace(" and ", " && ");
    result = result.replace(" or ", " || ");

    // Convert integer literals in comparisons to floats when comparing with f64
    // Pattern: `expr == N` or `expr != N` where expr contains skill_parameters (f64)
    // This handles cases like `skill_parameters.get(1)...unwrap_or(0.0) == 1`
    let cmp_re =
        regex::Regex::new(r"(skill_parameters[^=!<>]+(?:==|!=|<=|>=|<|>)\s*)(\d+)(\s|$|&&|\|\|)")
            .unwrap();
    result = cmp_re
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}{}.0{}", &caps[1], &caps[2], &caps[3])
        })
        .to_string();

    result
}

fn generate_rust_file(op: &OperatorClass) -> String {
    let mut output = String::new();
    let struct_name = to_upper_camel_case(&op.class_name);
    let mut translator = PythonToRustTranslator::new();

    // Get operator-specific variable declarations and replacements
    let extra_vars = get_operator_extra_vars(&op.class_name).unwrap_or_default();
    let replacements = get_operator_replacements(&op.class_name);

    let translated_skill_dps = if !op.skill_dps_body.is_empty() {
        let mut translated = translator.translate(&op.skill_dps_body);
        // For operators with extra vars, replace placeholder comments with actual variable usage
        if !extra_vars.is_empty() {
            translated = translated.replace(
                "1.0 /* self.params - needs manual implementation */",
                "params",
            );
        }
        // Apply operator-specific replacements
        for (from, to) in &replacements {
            translated = translated.replace(from, to);
        }
        // Inject extra vars at the start
        if !extra_vars.is_empty() {
            translated = format!("{extra_vars}\n{translated}");
        }
        // Inject cloned_op defaults (Ela) if needed
        // For Muelsyse, the cloned_op stats depend on module status
        if translated.contains("cloned_op_atk") || translated.contains("cloned_op_ranged") {
            let cloned_op_vars = if op.class_name == "Muelsyse" {
                // Muelsyse: cloned_op (Ela) ATK depends on module status
                // - No module: Ela base ATK = 668 (E2 90 with trust 100, pot 3+)
                // - With module: Ela ATK = 728 (adds module ATK)
                r#"        // Cloned operator defaults to Ela (Trapmaster)
        // Ela's ATK depends on Muelsyse's module status (Python behavior)
        let cloned_op_atk: f64 = if self.unit.module_index > 0 { 728.0 } else { 668.0 };
        let cloned_op_atk_interval: f64 = 0.85;
        let cloned_op_ranged: bool = true;
        let cloned_op_physical: bool = true;
"#
            } else {
                // Generic operators: use base Ela stats without module
                r#"        // Cloned operator defaults to Ela (Trapmaster)
        // Ela E2 90 stats with trust 100, pot 3+: ATK=668 (no module)
        let cloned_op_atk: f64 = 668.0;
        let cloned_op_atk_interval: f64 = 0.85;
        let cloned_op_ranged: bool = true;
        let cloned_op_physical: bool = true;
"#
            };
            translated = format!("{cloned_op_vars}\n{translated}");
        }
        // Inject below50 computation if needed (from __init__ logic)
        // Check for both patterns since the translation may vary based on processing order
        if translated.contains("/* self.below50") || translated.contains("/* below50") {
            let below50_code = generate_below50_code(&op.init_modifications);
            if !below50_code.is_empty() {
                // Replace both possible patterns
                translated = translated.replace(
                    "false /* self.below50 - needs manual implementation */",
                    "below50",
                );
                translated = translated.replace(
                    "false /* below50 - needs manual implementation */",
                    "below50",
                );
                translated = format!("{below50_code}\n{translated}");
            }
        }
        // Inject shadows computation if needed (from __init__ logic)
        // Check for both patterns since the translation may vary based on processing order
        if translated.contains("/* self.shadows") || translated.contains("/* shadows") {
            // Check for operator-specific shadows code first (for complex if/elif/else cases)
            let shadows_code = get_operator_shadows_code(&op.class_name)
                .unwrap_or_else(|| generate_shadows_code(&op.init_modifications));
            if !shadows_code.is_empty() {
                // Replace both possible patterns
                translated = translated.replace(
                    "1.0 /* self.shadows - needs manual implementation */",
                    "shadows",
                );
                translated = translated
                    .replace("1.0 /* shadows - needs manual implementation */", "shadows");
                translated = format!("{shadows_code}\n{translated}");
            }
        }
        // Inject ammo computation if needed (from __init__ logic)
        // Check for both patterns since the translation may vary based on processing order
        if translated.contains("/* self.ammo") || translated.contains("/* ammo") {
            let ammo_code = generate_ammo_code(&op.init_modifications);
            if !ammo_code.is_empty() {
                // Replace both possible patterns
                translated =
                    translated.replace("1.0 /* self.ammo - needs manual implementation */", "ammo");
                translated =
                    translated.replace("1.0 /* ammo - needs manual implementation */", "ammo");
                translated = format!("{ammo_code}\n{translated}");
            }
        }
        translated
    } else {
        "        // No skill_dps implementation found\n        self.unit.normal_attack(enemy, None, None, None)".to_string()
    };

    let skill_dps_comment = op
        .skill_dps_body
        .lines()
        .map(|l| format!("    /// {}", l.trim().replace('\t', "    ")))
        .collect::<Vec<_>>()
        .join("\n");

    output.push_str(&format!(
        r#"//! DPS calculations for {}
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit}};

/// {} operator implementation
pub struct {} {{
    pub unit: OperatorUnit,
}}

impl {} {{
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[{}];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[{}];

    /// Creates a new {} operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {{
        let mut unit = OperatorUnit::new(
            operator_data,
            params,
            {}, // default_skill_index
            {}, // default_potential
            {}, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

{}

        Self {{ unit }}
    }}

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
{}
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {{
        let mut defense = enemy.defense;
        let mut res = enemy.res;

{}
    }}
"#,
        struct_name,
        struct_name,
        struct_name,
        struct_name,
        op.available_skills
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        op.available_modules
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        struct_name,
        op.default_skill,
        op.default_pot,
        op.default_mod,
        generate_init_modifications_code(&op.class_name, &op.init_modifications),
        skill_dps_comment,
        translated_skill_dps,
    ));

    if op.has_total_dmg_override {
        let translated_total_dmg = translator.translate(&op.total_dmg_body);
        let total_dmg_comment = op
            .total_dmg_body
            .lines()
            .map(|l| format!("    /// {}", l.trim().replace('\t', "    ")))
            .collect::<Vec<_>>()
            .join("\n");

        output.push_str(&format!(
            r#"
    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
{total_dmg_comment}
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {{
        let mut defense = enemy.defense;
        let mut res = enemy.res;

{translated_total_dmg}
    }}
"#,
        ));
    }

    output.push_str("}\n");

    output.push_str(&format!(
        r#"
impl std::ops::Deref for {struct_name} {{
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {{
        &self.unit
    }}
}}

impl std::ops::DerefMut for {struct_name} {{
    fn deref_mut(&mut self) -> &mut Self::Target {{
        &mut self.unit
    }}
}}

impl DpsCalculator for {struct_name} {{
    fn skill_dps(&self, enemy: &EnemyStats) -> f64 {{
        Self::skill_dps(self, enemy)
    }}

    fn unit(&self) -> &OperatorUnit {{
        &self.unit
    }}

    fn unit_mut(&mut self) -> &mut OperatorUnit {{
        &mut self.unit
    }}
}}
"#
    ));

    output
}

/// Returns operator-specific variable declarations that should be added to skill_dps
/// These handle special parameters like Amiya's self.params that are set in __init__
fn get_operator_extra_vars(class_name: &str) -> Option<String> {
    match class_name {
        // Amiya S3 uses a mastery-based ATK buff: self.params = [0,1,1.1,1.2,1.3,1.4,1.5,1.6,1.8,2,2.3][pp.mastery]
        // skill_level in Rust: 1-7 = SL1-7, 8 = M1, 9 = M2, 10 = M3
        // Python mastery: 0 = unset, 1-7 = SL1-7, 8 = M1, 9 = M2, 10 = M3
        // We map skill_level to the Python array index
        "Amiya" => Some(
            r#"        // Amiya S3 ATK buff based on skill level (maps to Python mastery index)
        // skill_level 1-7 -> indices 1-7, skill_level 8-10 -> indices 8-10
        let params: f64 = [0.0, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.8, 2.0, 2.3]
            .get(self.unit.skill_level as usize)
            .copied()
            .unwrap_or(2.3);
"#
            .to_string(),
        ),
        // Arene: if skill == 1 and talent_dmg, then trait_dmg = false
        // This affects the atk_scale *= 0.8 condition
        "Arene" => Some(
            r#"        // Arene: trait_dmg is false when skill == 1 and talent_dmg (Python __init__ logic)
        let trait_damage = if self.unit.skill_index == 1 && self.unit.talent_damage {
            false
        } else {
            self.unit.trait_damage
        };
"#
            .to_string(),
        ),
        // Muelsyse: copy_factor depends on module_level, but module matching may fail
        // causing both module_level and operator_module_level to be 0.
        // Use effective_module_level that defaults to 3 when a module is selected.
        "Muelsyse" => Some(
            r#"        // Muelsyse: Get effective module level for copy_factor calculation
        // When module_index > 0 but module_level is 0 (matching failed), default to 3
        let effective_module_level: i32 = if self.unit.module_index > 0 {
            if self.unit.operator_module_level > 0 {
                self.unit.operator_module_level
            } else if self.unit.module_level > 0 {
                self.unit.module_level
            } else {
                3 // Default to max level when module is selected
            }
        } else {
            0
        };
"#
            .to_string(),
        ),
        _ => None,
    }
}

/// Returns operator-specific translation replacements
/// These handle cases where trait_damage or similar flags need operator-specific logic
fn get_operator_replacements(class_name: &str) -> Vec<(&'static str, &'static str)> {
    match class_name {
        // Arene uses a local trait_damage variable instead of self.unit.trait_damage
        "Arene" => vec![("self.unit.trait_damage", "trait_damage")],
        // Muelsyse: Use effective_module_level variable (defined in extra_vars)
        // This handles cases where module data matching fails but we still want full module benefits
        "Muelsyse" => vec![
            (
                "(self.unit.module_level as f64)",
                "(effective_module_level as f64)",
            ),
            ("self.unit.module_level", "effective_module_level"),
        ],
        _ => vec![],
    }
}

/// Returns operator-specific shadows code for operators with complex if/elif/else logic
/// that can't be correctly translated by the generic generate_shadows_code()
fn get_operator_shadows_code(class_name: &str) -> Option<String> {
    match class_name {
        // Walter has nested if/elif/else that the generic parser flattens incorrectly
        // Original Python:
        // self.shadows = 0
        // if self.elite == 2:
        //     if self.skill in [0,3]:
        //         if self.talent2_dmg:
        //             self.shadows = 3
        //         else:
        //             self.shadows = min(self.skill +1,2)
        //         if self.skill_params[1] == 1 and self.skill == 3:
        //             self.shadows -= 1
        //     else:
        //         self.shadows = 1 if self.talent2_dmg else 0
        "Walter" => Some(
            r#"        // Calculate shadows from __init__ logic (Walter-specific)
        let mut shadows: f64 = 0.0;
        if self.unit.elite == 2 {
            if self.unit.skill_index == 0 || self.unit.skill_index == 3 {
                if self.unit.talent2_damage {
                    shadows = 3.0;
                } else {
                    shadows = ((self.unit.skill_index as f64) + 1.0).min(2.0);
                }
                if self.unit.skill_parameters.get(1).copied().unwrap_or(0.0) == 1.0
                    && self.unit.skill_index == 3
                {
                    shadows -= 1.0;
                }
            } else {
                shadows = if self.unit.talent2_damage { 1.0 } else { 0.0 };
            }
        }
"#
            .to_string(),
        ),
        _ => None,
    }
}

fn generate_mod_file(operators: &[&OperatorClass]) -> String {
    let mut output = String::new();
    output.push_str("#![allow(clippy::module_inception)]\n\n");

    let mut sorted_ops: Vec<_> = operators.iter().collect();
    sorted_ops.sort_by_key(|op| &op.class_name);

    for op in &sorted_ops {
        let snake_name = to_snake_case(&op.class_name);
        output.push_str(&format!("mod {snake_name};\n"));
    }

    output.push('\n');

    for op in &sorted_ops {
        let snake_name = to_snake_case(&op.class_name);
        let struct_name = to_upper_camel_case(&op.class_name);
        output.push_str(&format!("pub use {snake_name}::{struct_name};\n"));
    }

    output
}

fn generate_main_mod_file(letter_folders: &HashSet<char>, operators: &[OperatorClass]) -> String {
    let mut output = String::new();
    let mut letters: Vec<char> = letter_folders.iter().copied().collect();
    letters.sort();

    output.push_str("//! Auto-generated operator implementations\n");
    output.push_str("//!\n");
    output.push_str("//! Each operator is organized into alphabetical subfolders.\n\n");

    output.push_str("use super::operator_data::OperatorData;\n");
    output.push_str("use super::operator_unit::{DpsCalculator, OperatorParams};\n\n");

    for letter in &letters {
        output.push_str(&format!("pub mod {letter};\n"));
    }

    output.push('\n');

    for letter in &letters {
        output.push_str(&format!("pub use {letter}::*;\n"));
    }

    output.push('\n');

    // Generate registry function
    output.push_str(
        "/// Creates an operator by name, returning a boxed DpsCalculator trait object\n",
    );
    output.push_str("/// \n");
    output.push_str("/// # Arguments\n");
    output.push_str("/// * `name` - The operator name (e.g., \"Blaze\", \"ExusiaiAlter\")\n");
    output.push_str("/// * `operator_data` - The operator's base data from JSON\n");
    output.push_str("/// * `params` - The operator configuration parameters\n");
    output.push_str("/// \n");
    output.push_str("/// # Returns\n");
    output.push_str("/// Some(Box<dyn DpsCalculator>) if the operator is found, None otherwise\n");
    output.push_str("pub fn create_operator(\n");
    output.push_str("    name: &str,\n");
    output.push_str("    operator_data: OperatorData,\n");
    output.push_str("    params: OperatorParams,\n");
    output.push_str(") -> Option<Box<dyn DpsCalculator + Send + Sync>> {\n");
    output.push_str("    match name {\n");

    // Sort operators alphabetically for consistent output
    let mut sorted_ops: Vec<_> = operators.iter().collect();
    sorted_ops.sort_by_key(|op| &op.class_name);

    for op in &sorted_ops {
        let struct_name = to_upper_camel_case(&op.class_name);
        output.push_str(&format!(
            "        \"{struct_name}\" => Some(Box::new({struct_name}::new(operator_data, params))),\n"
        ));
    }

    output.push_str("        _ => None,\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    output
}

fn to_upper_camel_case(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if !chars.is_empty() && chars[0].is_lowercase() {
        chars[0] = chars[0].to_ascii_uppercase();
    }
    chars.into_iter().collect()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                if !prev.is_uppercase() && prev != '_' {
                    result.push('_');
                }
            }
            result.push(c.to_ascii_lowercase());
        } else if c.is_numeric() {
            if i > 0 && chars[i - 1].is_alphabetic() {
                result.push('_');
            }
            result.push(c);
        } else {
            result.push(c);
        }
    }
    result
}

fn get_folder_letter(name: &str) -> char {
    let first = name.chars().next().unwrap_or('_');
    if first.is_numeric() {
        '_'
    } else {
        first.to_ascii_lowercase()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input_python_file> <output_directory>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!(
            "  cargo run --bin translate-operators -- ../ArknightsDpsCompare/damagecalc/damage_formulas.py src/core/dps_calculator/operators"
        );
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_dir = Path::new(&args[2]);

    println!("Parsing Python file: {}", input_path.display());

    // Pre-initialize patterns for timing
    let _ = patterns();
    println!("Compiled regex patterns");

    let operators = match parse_python_file(input_path) {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("Error parsing Python file: {e}");
            std::process::exit(1);
        }
    };

    println!("Found {} operator classes", operators.len());
    println!("Translating Python skill_dps methods to Rust...");

    let mut by_letter: HashMap<char, Vec<&OperatorClass>> = HashMap::new();
    for op in &operators {
        let letter = get_folder_letter(&op.class_name);
        by_letter.entry(letter).or_default().push(op);
    }

    if output_dir.exists()
        && let Err(e) = fs::remove_dir_all(output_dir)
    {
        eprintln!("Warning: Failed to clean output directory: {e}");
    }

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {e}");
        std::process::exit(1);
    }

    let mut letter_folders: HashSet<char> = HashSet::new();
    let mut generated_count = 0;

    for (letter, ops) in &by_letter {
        letter_folders.insert(*letter);

        let letter_dir = output_dir.join(letter.to_string());
        if let Err(e) = fs::create_dir_all(&letter_dir) {
            eprintln!("Failed to create letter directory: {e}");
            continue;
        }

        for op in ops {
            let snake_name = to_snake_case(&op.class_name);
            let file_path = letter_dir.join(format!("{snake_name}.rs"));

            eprint!("  Processing {}...", &op.class_name);
            let content = generate_rust_file(op);
            if let Err(e) = fs::write(&file_path, content) {
                eprintln!("Failed to write {}: {}", file_path.display(), e);
            } else {
                generated_count += 1;
                eprintln!(" done");
            }
        }

        let mod_path = letter_dir.join("mod.rs");
        let mod_content = generate_mod_file(ops);
        if let Err(e) = fs::write(&mod_path, mod_content) {
            eprintln!("Failed to write {}: {}", mod_path.display(), e);
        }
    }

    let main_mod_path = output_dir.join("mod.rs");
    let main_mod_content = generate_main_mod_file(&letter_folders, &operators);
    if let Err(e) = fs::write(&main_mod_path, main_mod_content) {
        eprintln!("Failed to write {}: {}", main_mod_path.display(), e);
    }

    println!();
    println!("Generation complete!");
    println!("  Total operators: {generated_count}");
    println!("  Letter folders: {}", letter_folders.len());
    println!();
    println!("Generated folder structure:");
    for letter in letter_folders.iter().collect::<Vec<_>>().iter().sorted() {
        let count = by_letter.get(letter).map(|v| v.len()).unwrap_or(0);
        println!("  {letter}/  ({count} operators)");
    }
    println!();
    println!("Next steps:");
    println!("  1. Run 'cargo build' to check for compilation errors");
    println!("  2. Fix any translation issues manually");
    println!("  3. Run tests to verify DPS calculations");
}

trait Sorted: Iterator {
    fn sorted(self) -> Vec<Self::Item>
    where
        Self: Sized,
        Self::Item: Ord,
    {
        let mut v: Vec<_> = self.collect();
        v.sort();
        v
    }
}

impl<T: Iterator> Sorted for T {}
