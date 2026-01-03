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
}

/// Python to Rust translator for skill_dps methods
struct PythonToRustTranslator {
    indent_level: usize,
}

impl PythonToRustTranslator {
    fn new() -> Self {
        Self { indent_level: 2 }
    }

    /// Translate a Python skill_dps method body to Rust
    fn translate(&self, python_code: &str) -> String {
        let mut rust_lines = Vec::new();
        let lines: Vec<&str> = python_code.lines().collect();

        // Scan for variables that are used across if/else branches
        // These need to be declared at function level
        let shared_vars = self.find_shared_variables(&lines);

        // Pre-declare shared variables at function start
        for var in &shared_vars {
            rust_lines.push(format!(
                "{}let mut {}: f64 = 0.0;",
                self.indent(self.indent_level),
                var
            ));
        }
        if !shared_vars.is_empty() {
            rust_lines.push(String::new()); // blank line
        }

        // Track declared variables (including pre-declared ones)
        let mut declared_vars: HashSet<String> = shared_vars;

        // Track indentation levels of open blocks (stack)
        // Each entry is (Python indent level, whether it was a skill check)
        let mut indent_stack: Vec<(usize, bool)> = Vec::new();

        // Track if we're in a chain of skill checks at the same level
        let mut last_skill_check_indent: Option<usize> = None;

        // Track if we're inside an untranslated block (like a for loop)
        let mut skip_until_indent: Option<usize> = None;

        // Track lines to skip (e.g., except: after processing try:)
        let mut lines_to_skip: HashSet<usize> = HashSet::new();

        for (i, line) in lines.iter().enumerate() {
            // Check if this line should be skipped
            if lines_to_skip.contains(&i) {
                continue;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Get this line's indentation level
            let line_indent = line.len() - line.trim_start().len();

            // Check if we should skip this line because we're inside an untranslated block
            if let Some(skip_indent) = skip_until_indent {
                if line_indent > skip_indent {
                    // Still inside the untranslated block, skip this line
                    continue;
                } else {
                    // We've exited the untranslated block
                    skip_until_indent = None;
                }
            }

            // Strip Python comments but preserve them as Rust comments
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

            // Check if this is a skill check (if self.skill == X or similar)
            let is_skill_check = trimmed.starts_with("if ")
                && trimmed.ends_with(':')
                && (trimmed.contains("self.skill") || trimmed.contains("self.module"));

            // Close blocks if we've dedented
            while !indent_stack.is_empty() {
                let (last_indent, _was_skill_check) = *indent_stack.last().unwrap();

                if line_indent > last_indent {
                    break; // We're nested deeper, don't close
                }

                // If this is elif/else at the same level, don't close yet
                if (trimmed.starts_with("elif ") || trimmed == "else:")
                    && line_indent == last_indent
                {
                    break;
                }

                indent_stack.pop();
                rust_lines.push(format!("{}}}", self.indent(self.indent_level)));
                last_skill_check_indent = None;
            }

            // Handle return statement
            if trimmed.starts_with("return ") || trimmed.starts_with("return(") {
                // Extract expression after return
                let expr = if trimmed.starts_with("return ") {
                    &trimmed[7..]
                } else {
                    // return(expr) - remove the outer parens
                    let inner = &trimmed[7..]; // "(...)"
                    if inner.starts_with('(') && inner.ends_with(')') {
                        &inner[1..inner.len() - 1]
                    } else {
                        inner
                    }
                };

                // Check if it contains super() or recursive self method calls which we can't translate
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
                // Early returns need to be explicit in Rust
                rust_lines.push(format!(
                    "{}return {};",
                    self.indent(self.indent_level),
                    rust_expr
                ));
                continue;
            }

            // Handle try-except patterns: try: var = expr followed by except: var = default
            // Python uses this for safe array access. We use the except value as default.
            if let Some(try_stmt) = trimmed.strip_prefix("try: ") {
                if let Some((var, _try_expr)) = self.parse_assignment(try_stmt) {
                    // Look ahead for the except line
                    let mut found_except = false;
                    for (j, next_line) in lines[i + 1..].iter().enumerate() {
                        let next_trimmed = next_line.trim();
                        if next_trimmed.starts_with("except:") {
                            // Found except - get the default value
                            let except_stmt = if next_trimmed.starts_with("except: ") {
                                &next_trimmed[8..]
                            } else {
                                break; // Multi-line except block - too complex
                            };

                            if let Some((except_var, default_expr)) =
                                self.parse_assignment(except_stmt)
                            {
                                if except_var == var {
                                    // Pre-declare variable with except (default) value
                                    let rust_default = self.translate_expression(default_expr);
                                    if !declared_vars.contains(&var) {
                                        rust_lines.push(format!(
                                            "{}let mut {} = {}; // try-except fallback",
                                            self.indent(self.indent_level),
                                            var,
                                            rust_default
                                        ));
                                        declared_vars.insert(var.clone());
                                    } else {
                                        rust_lines.push(format!(
                                            "{}{} = {}; // try-except fallback",
                                            self.indent(self.indent_level),
                                            var,
                                            rust_default
                                        ));
                                    }
                                    found_except = true;
                                    // Mark the except line to be skipped
                                    lines_to_skip.insert(i + j + 1);
                                    break;
                                }
                            }
                        }
                        // If next line isn't empty/comment and isn't except, stop looking
                        if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                            break;
                        }
                    }
                    if found_except {
                        continue;
                    }
                }
            }

            // Handle except: lines (skip if we processed the try above)
            if trimmed.starts_with("except:") {
                // Already handled in try: processing above, or standalone except we can't handle
                continue;
            }

            // Handle single-line if statements: if cond: var = value
            // These get translated to Rust if blocks with assignment inside
            if trimmed.starts_with("if ") && trimmed.contains(": ") && !trimmed.ends_with(':') {
                // Parse pattern: if condition: var = value
                if let Some(colon_pos) = trimmed.find(": ") {
                    let condition = &trimmed[3..colon_pos];
                    let statement = &trimmed[colon_pos + 2..];

                    // Handle assignment in statement
                    if let Some((var, expr)) = self.parse_assignment(statement) {
                        let rust_condition = self.translate_condition(condition);
                        let rust_expr = self.translate_expression(expr);

                        // Pre-declare the variable if needed
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
                }
            }

            // Handle single-line elif statements: elif cond: var = value
            if trimmed.starts_with("elif ") && trimmed.contains(": ") && !trimmed.ends_with(':') {
                if let Some(colon_pos) = trimmed.find(": ") {
                    let condition = &trimmed[5..colon_pos];
                    let statement = &trimmed[colon_pos + 2..];

                    if let Some((var, expr)) = self.parse_assignment(statement) {
                        let rust_condition = self.translate_condition(condition);
                        let rust_expr = self.translate_expression(expr);

                        // Variable should already be declared by preceding if
                        rust_lines.push(format!(
                            "{}else if {} {{ {} = {}; }}",
                            self.indent(self.indent_level),
                            rust_condition,
                            var,
                            rust_expr
                        ));
                        continue;
                    }
                }
            }

            // Handle single-line else statements: else: var = value
            if let Some(statement) = trimmed.strip_prefix("else: ") {
                if let Some((var, expr)) = self.parse_assignment(statement) {
                    let rust_expr = self.translate_expression(expr);

                    // Variable should already be declared
                    rust_lines.push(format!(
                        "{}else {{ {} = {}; }}",
                        self.indent(self.indent_level),
                        var,
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
                    last_skill_check_indent = Some(line_indent);
                }
                continue;
            }

            // Handle elif statements
            if trimmed.starts_with("elif ") && trimmed.ends_with(':') {
                let condition = &trimmed[5..trimmed.len() - 1];
                let rust_condition = self.translate_condition(condition);

                // Check if we have a matching if/elif at this indent level to continue
                if let Some(&(last_indent, _)) = indent_stack.last() {
                    if last_indent == line_indent {
                        // Pop the previous if/elif block we're closing
                        indent_stack.pop();
                        rust_lines.push(format!(
                            "{}}} else if {} {{",
                            self.indent(self.indent_level),
                            rust_condition
                        ));
                        // Push the elif block we're opening
                        indent_stack.push((line_indent, false));
                    } else {
                        // No matching if at this level - treat as standalone if
                        rust_lines.push(format!(
                            "{}if {} {{",
                            self.indent(self.indent_level),
                            rust_condition
                        ));
                        indent_stack.push((line_indent, false));
                    }
                } else {
                    // Empty indent_stack - treat as standalone if
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
                // Check if we have a matching if/elif at this indent level to continue
                if let Some(&(last_indent, _)) = indent_stack.last() {
                    if last_indent == line_indent {
                        // Pop the if/elif block we're closing
                        indent_stack.pop();
                        rust_lines.push(format!("{}}} else {{", self.indent(self.indent_level)));
                        // Push the else block we're opening
                        indent_stack.push((line_indent, false));
                    } else {
                        // No matching if at this level - this else follows an untranslated single-line if
                        // Mark as untranslated
                        rust_lines.push(format!(
                            "{}// UNTRANSLATED ELSE (no matching if): else:",
                            self.indent(self.indent_level)
                        ));
                    }
                } else {
                    // Empty indent_stack - this else has no matching if
                    rust_lines.push(format!(
                        "{}// UNTRANSLATED ELSE (empty stack): else:",
                        self.indent(self.indent_level)
                    ));
                }
                continue;
            }

            // Handle for loops - mark as UNTRANSLATED and skip the loop body
            if trimmed.starts_with("for ") && trimmed.contains(" in ") && trimmed.ends_with(':') {
                rust_lines.push(format!(
                    "{}// UNTRANSLATED FOR LOOP: {}",
                    self.indent(self.indent_level),
                    trimmed
                ));
                rust_lines.push(format!(
                    "{}// TODO: Implement loop logic manually",
                    self.indent(self.indent_level)
                ));
                // Skip all lines inside this for loop
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
                    // Always use mut for new variable declarations since Python allows reassignment
                    rust_lines.push(format!(
                        "{}let mut {} = {};",
                        self.indent(self.indent_level),
                        var,
                        rust_expr
                    ));
                }
                continue;
            }

            // Handle compound assignments (+=, -=, *=, /=)
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

        // Convert camelCase variable names to snake_case as a final step
        self.convert_camel_case_variables(&result)
    }

    /// Convert camelCase user-defined variable names to snake_case
    /// This handles variables like dpsNorm -> dps_norm, timeToFallout -> time_to_fallout
    fn convert_camel_case_variables(&self, code: &str) -> String {
        let mut result = code.to_string();

        // List of known camelCase variable names that need conversion
        // These are extracted from the Python code and used in Rust
        let camel_case_vars = [
            ("timeToFallout", "time_to_fallout"),
            ("dpsFallout", "dps_fallout"),
            ("dpsNorm", "dps_norm"),
            ("hitdmgTW", "hitdmg_tw"),
            ("eleApplicationTarget", "ele_application_target"),
            ("eleApplicationBase", "ele_application_base"),
            ("targetEledps", "target_eledps"),
            ("ambientEledps", "ambient_eledps"),
            // Add more as needed - these are the ones causing warnings
        ];

        for (camel, snake) in camel_case_vars {
            // Replace as whole word using word boundaries
            let pattern = format!(r"\b{camel}\b");
            result = Regex::new(&pattern)
                .unwrap()
                .replace_all(&result, snake)
                .to_string();
        }

        result
    }

    fn indent(&self, level: usize) -> String {
        "    ".repeat(level)
    }

    /// Find variables that are assigned in multiple if branches or used after if blocks
    /// These need to be declared at function level
    fn find_shared_variables(&self, lines: &[&str]) -> HashSet<String> {
        let mut shared_vars: HashSet<String> = HashSet::new();
        let mut vars_in_branch: HashMap<String, usize> = HashMap::new(); // var -> branch count
        let mut current_branch = 0;
        let mut if_depth = 0;
        let base_indent = lines
            .iter()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let line_indent = line.len() - line.trim_start().len();

            // Detect ANY if statements at base level
            if line_indent == base_indent && trimmed.starts_with("if ") && trimmed.ends_with(':') {
                current_branch += 1;
                if_depth = 1;
            }

            // Track if depth for nested ifs
            if line_indent > base_indent {
                if trimmed.starts_with("if ") && trimmed.ends_with(':') {
                    if_depth += 1;
                } else if trimmed == "else:" || trimmed.starts_with("elif ") {
                    // Stay in same if structure
                }
            }

            // Detect end of if block when returning to base indent (not if/elif/else)
            if line_indent == base_indent
                && !trimmed.starts_with("if ")
                && !trimmed.starts_with("elif ")
                && trimmed != "else:"
            {
                if_depth = 0;
            }

            // Detect return at base level (end of branches)
            if line_indent == base_indent && trimmed.starts_with("return ") {
                // Variables used in return should be shared
                let var_re = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
                for cap in var_re.captures_iter(trimmed) {
                    let var = cap[1].to_string();
                    if vars_in_branch.contains_key(&var) {
                        shared_vars.insert(var);
                    }
                }
            }

            // Detect variable assignments inside ANY if blocks
            if if_depth > 0 && line_indent > base_indent {
                if let Some((var, _)) = self.parse_assignment(trimmed) {
                    let entry = vars_in_branch.entry(var.clone()).or_insert(0);
                    if *entry != current_branch {
                        // Variable assigned in a different branch
                        if *entry != 0 {
                            shared_vars.insert(var.clone());
                        }
                        *entry = current_branch;
                    }
                    // Also add variables assigned in nested if blocks since they need pre-declaration
                    if if_depth > 1 {
                        shared_vars.insert(var);
                    }
                }
            }
        }

        // Common variables that often need to be shared
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
            // Variables defined in if blocks but used elsewhere
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
        for var in common_vars {
            // Check if this var is used in the code
            let var_pattern = format!(r"\b{var}\b");
            let re = Regex::new(&var_pattern).unwrap();
            let code_text = lines.join("\n");
            if re.is_match(&code_text) && vars_in_branch.len() > 1 {
                // Variable exists and there are multiple branches
                shared_vars.insert(var.to_string());
            }
        }

        shared_vars
    }

    /// Parse a simple assignment like "var = expr"
    fn parse_assignment<'a>(&self, line: &'a str) -> Option<(String, &'a str)> {
        // Find the first '=' that's not part of ==, +=, -=, *=, /=, !=, <=, >=
        let mut i = 0;
        let chars: Vec<char> = line.chars().collect();

        while i < chars.len() {
            if chars[i] == '=' {
                // Check it's not a compound assignment or comparison
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

                    // Handle Python's double assignment pattern: "var = var = expr"
                    // Strip the repeated "var = " from the expression
                    let double_assign_prefix = format!("{var_part} = ");
                    if expr_part.starts_with(&double_assign_prefix) {
                        expr_part = &expr_part[double_assign_prefix.len()..];
                    }

                    // Validate var is a simple identifier
                    if Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$")
                        .unwrap()
                        .is_match(var_part)
                    {
                        // Rename reserved Rust keywords
                        let var_name = self.rename_reserved_keyword(var_part);
                        return Some((var_name, expr_part));
                    }
                }
            }
            i += 1;
        }
        None
    }

    /// Rename Python variables that conflict with Rust reserved keywords
    fn rename_reserved_keyword(&self, name: &str) -> String {
        match name {
            "crate" => "crit_rate".to_string(),
            "type" => "dmg_type".to_string(),
            "match" => "match_val".to_string(),
            "mod" => "module".to_string(),
            _ => name.to_string(),
        }
    }

    /// Parse compound assignments like "var += expr"
    fn parse_compound_assignment<'a>(&self, line: &'a str) -> Option<(String, &'a str, &'a str)> {
        let re = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*(\+=|-=|\*=|/=)\s*(.+)$").unwrap();

        if let Some(caps) = re.captures(line) {
            let var_raw = caps.get(1)?.as_str();
            let var = self.rename_reserved_keyword(var_raw);
            let op = caps.get(2)?.as_str();
            let expr = caps.get(3)?.as_str();
            return Some((var, op, expr));
        }
        None
    }

    /// Translate a Python condition to Rust
    fn translate_condition(&self, condition: &str) -> String {
        let mut result = condition.to_string();

        // Handle 'in' operator FIRST (before and/or translation)
        result = self.translate_in_operator(&result);

        // Handle 'and' -> '&&'
        result = Regex::new(r"\s+and\s+")
            .unwrap()
            .replace_all(&result, " && ")
            .to_string();

        // Handle 'or' -> '||'
        result = Regex::new(r"\s+or\s+")
            .unwrap()
            .replace_all(&result, " || ")
            .to_string();

        // Handle 'not ' -> '!'
        result = Regex::new(r"\bnot\s+")
            .unwrap()
            .replace_all(&result, "!")
            .to_string();

        // Translate self references
        result = self.translate_self_references(&result);

        // Convert comparison integers to floats (needed for casted integer fields)
        result = self.convert_comparison_integers(&result);

        // Also convert arithmetic integers in conditions (e.g., *100 in comparison expressions)
        result = self.convert_condition_arithmetic_integers(&result);

        result
    }

    /// Convert integers in arithmetic expressions within conditions
    fn convert_condition_arithmetic_integers(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Handle *N (no space) followed by comparison operators (e.g., *100 >= 2.0)
        result = Regex::new(r"\*(\d+)(\s*[><=!])")
            .unwrap()
            .replace_all(&result, "*$1.0$2")
            .to_string();

        // Handle * N (with space) followed by comparison operators
        result = Regex::new(r"(\*\s+)(\d+)(\s*[><=!])")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        // Handle /N or / N followed by comparison operators
        result = Regex::new(r"/(\d+)(\s*[><=!])")
            .unwrap()
            .replace_all(&result, "/$1.0$2")
            .to_string();

        // Handle integer * float at the start of conditions (e.g., "12 * 0.25 * ...")
        // This catches patterns like "12 * 0.25" where 12 should become 12.0
        result = Regex::new(r"^(\d+)\s*\*\s*(\d+\.\d+)")
            .unwrap()
            .replace_all(&result, "$1.0 * $2")
            .to_string();

        result
    }

    /// Convert integers in comparison expressions to floats
    fn convert_comparison_integers(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // == 3 -> == 3.0, != 2 -> != 2.0
        result = Regex::new(r"(==|!=)\s*(\d+)(\s|;|,|\)|$|\{|\}|&|\|)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // >= and <=
        result = Regex::new(r"(>=|<=)\s*(\d+)(\s|;|,|\)|$|\{|\}|&|\|)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // > and < (but not >> or <<)
        result = Regex::new(r"([^><])(>|<)\s*(\d+)(\s|;|,|\)|$|\{|\}|&|\|)")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{} {}.0{}", &caps[1], &caps[2], &caps[3], &caps[4])
            })
            .to_string();

        result
    }

    /// Translate a Python expression to Rust
    fn translate_expression(&self, expr: &str) -> String {
        let mut result = expr.trim().to_string();

        // Handle Python ternary: "a if cond else b" -> "if cond { a } else { b }"
        result = self.translate_ternary(&result);

        // Handle np.fmax(a, b) -> (a).max(b)
        result = self.translate_np_fmax(&result);

        // Handle np.fmin(a, b) -> (a).min(b)
        result = self.translate_np_fmin(&result);

        // Handle min(a, b) and max(a, b)
        result = self.translate_min_max(&result);

        // Handle Python int() function -> truncate but keep as f64 for arithmetic
        // Using .trunc() instead of as i32 to avoid type issues in float arithmetic
        result = Regex::new(r"\bint\(([^)]+)\)")
            .unwrap()
            .replace_all(&result, "(($1) as f64).trunc()")
            .to_string();

        // Handle Python abs() function
        result = Regex::new(r"\babs\(([^)]+)\)")
            .unwrap()
            .replace_all(&result, "(($1) as f64).abs()")
            .to_string();

        // Handle Python round() function
        result = Regex::new(r"\bround\(([^)]+)\)")
            .unwrap()
            .replace_all(&result, "(($1) as f64).round()")
            .to_string();

        // Handle self.X translations
        result = self.translate_self_references(&result);

        // Handle 'and' and 'or' in expressions
        result = Regex::new(r"\s+and\s+")
            .unwrap()
            .replace_all(&result, " && ")
            .to_string();
        result = Regex::new(r"\s+or\s+")
            .unwrap()
            .replace_all(&result, " || ")
            .to_string();

        // Handle Python 'in' operator for skill checks
        result = self.translate_in_operator(&result);

        // Handle Python ** exponentiation operator -> .powf()
        // Must be done BEFORE integer division to avoid ** being partially matched
        result = self.translate_power_operator(&result);

        // Handle integer division //
        result = result.replace("//", " / ");

        // Ensure proper float division
        result = self.ensure_float_division(&result);

        // Convert bare integers to floats in arithmetic context
        result = self.convert_integers_to_floats(&result);

        // Rename Python variables that are Rust reserved keywords
        result = Regex::new(r"\bcrate\b")
            .unwrap()
            .replace_all(&result, "crit_rate")
            .to_string();

        // Handle array indexing with float expressions - convert to usize
        // Pattern: array[expr] where expr is not an integer literal
        // [targets] -> [(targets) as usize]
        result = Regex::new(r"\[targets\]")
            .unwrap()
            .replace_all(&result, "[(targets) as usize]")
            .to_string();

        // [targets-1.0] -> [((targets) as usize).saturating_sub(1)]
        result = Regex::new(r"\[targets\s*-\s*1\.0\]")
            .unwrap()
            .replace_all(&result, "[((targets) as usize).saturating_sub(1)]")
            .to_string();

        // Handle Python negative array indexing [-N] -> use len()-N pattern
        // For now, just use get with wrapping or mark as needs manual implementation
        result = Regex::new(r"\[-([\d.]+)\]")
            .unwrap()
            .replace_all(
                &result,
                "[(self.unit.talent1_parameters.len() as isize - $1 as isize) as usize]",
            )
            .to_string();

        // Ensure array literals have consistent float types
        // Only match array literals (after = or { or ,) not subscript operations
        // [0,1,2,3,4] -> [0.0,1.0,2.0,3.0,4.0]
        result = self.convert_array_elements_to_floats(&result);

        result
    }

    /// Convert integer elements in array literals to floats
    /// Handle arrays with mixed types like [0, 1, 1.85, ...] -> [0.0, 1.0, 1.85, ...]
    fn convert_array_elements_to_floats(&self, expr: &str) -> String {
        // Match array literals that have at least 2 elements (to distinguish from subscripts)
        // Look for patterns like [0, 1, ...] or [0.0, 1.0, ...]
        // We need to handle arrays with complex expressions like [0, 1, 1.85+(0.85 as f64).powf(2)]

        // Find array-like structures with commas
        let mut i = 0;
        let chars: Vec<char> = expr.chars().collect();
        let mut result = String::new();

        while i < chars.len() {
            if chars[i] == '[' {
                // Find matching bracket
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

                    // Check if this looks like an array literal (has commas and starts with number)
                    if array_content.contains(',')
                        && (array_content
                            .trim()
                            .starts_with(|c: char| c.is_ascii_digit())
                            || array_content.trim().starts_with('-'))
                    {
                        // Process elements, converting integers to floats
                        let elements: Vec<&str> = array_content.split(',').collect();
                        let float_elements: Vec<String> = elements
                            .iter()
                            .map(|e| {
                                let trimmed = e.trim();
                                // Convert bare integers (like 0, 1)
                                if Regex::new(r"^\d+$").unwrap().is_match(trimmed) {
                                    format!("{trimmed}.0")
                                }
                                // Convert integers at start of expression followed by * or /
                                // e.g., "2 * 1.85" -> "2.0 * 1.85", "2*(..." -> "2.0*(...
                                else if Regex::new(r"^\d+\s*[*/]").unwrap().is_match(trimmed) {
                                    Regex::new(r"^(\d+)(\s*[*/])")
                                        .unwrap()
                                        .replace(trimmed, "$1.0$2")
                                        .to_string()
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
                // Not an array literal or couldn't find matching bracket
                result.push(chars[i]);
                i += 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// Convert bare integers to floats where needed for arithmetic
    fn convert_integers_to_floats(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Convert bare integer at end of expression (e.g., "* 2" at end without semicolon)
        result = Regex::new(r"\*\s*(\d+)$")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("* {}.0", &caps[1])
            })
            .to_string();

        result = Regex::new(r"/\s*(\d+)$")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("/ {}.0", &caps[1])
            })
            .to_string();

        // Convert bare integer assignment (entire expression is just a number)
        if Regex::new(r"^\d+$").unwrap().is_match(&result) {
            result = format!("{result}.0");
        }

        // Convert integer at START of expression followed by arithmetic: "2 * x" -> "2.0 * x"
        result = Regex::new(r"^(\d+)\s*\*")
            .unwrap()
            .replace_all(&result, "$1.0 *")
            .to_string();

        result = Regex::new(r"^(\d+)\s*/")
            .unwrap()
            .replace_all(&result, "$1.0 /")
            .to_string();

        result = Regex::new(r"^(\d+)\s*\+")
            .unwrap()
            .replace_all(&result, "$1.0 +")
            .to_string();

        result = Regex::new(r"^(\d+)\s*-")
            .unwrap()
            .replace_all(&result, "$1.0 -")
            .to_string();

        // Convert comparison integers to floats (for casted integer fields)
        // == 3 -> == 3.0, != 2 -> != 2.0, etc. (but not for array indices)
        result = Regex::new(r"(==|!=)\s*(\d+)(\s|;|,|\)|$|\{|\})")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // Also handle >= and <= comparisons
        result = Regex::new(r"(>=|<=)\s*(\d+)(\s|;|,|\)|$|\{|\})")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{} {}.0{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string();

        // Handle > and < comparisons (but not >> or << bit shifts)
        result = Regex::new(r"([^><])(>|<)\s*(\d+)(\s|;|,|\)|$|\{|\})")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{} {}.0{}", &caps[1], &caps[2], &caps[3], &caps[4])
            })
            .to_string();

        // Convert "= 0" or "= 1" at end of expressions or statements to floats
        result = Regex::new(r"=\s*(\d+)\s*;")
            .unwrap()
            .replace_all(&result, "= $1.0;")
            .to_string();

        // Convert standalone integers in if/else blocks (but not array indices)
        result = Regex::new(r"\{\s*(\d+)\s*\}")
            .unwrap()
            .replace_all(&result, "{ $1.0 }")
            .to_string();

        // Convert integers in arithmetic: + 1, - 1, * 2, / 2, etc. (but not indices like [0])
        result = Regex::new(r"(\+\s*)(\d+)(\s*[^\.\d\[])")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        // Also handle + N, - N, * N, / N at end of expression
        result = Regex::new(r"(\+\s*)(\d+)$")
            .unwrap()
            .replace_all(&result, "$1$2.0")
            .to_string();

        result = Regex::new(r"(-\s*)(\d+)$")
            .unwrap()
            .replace_all(&result, "$1$2.0")
            .to_string();

        result = Regex::new(r"(\*\s*)(\d+)$")
            .unwrap()
            .replace_all(&result, "$1$2.0")
            .to_string();

        result = Regex::new(r"(/\s*)(\d+)$")
            .unwrap()
            .replace_all(&result, "$1$2.0")
            .to_string();

        result = Regex::new(r"(-\s*)(\d+)(\s*[^\.\d\[])")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        result = Regex::new(r"(\*\s*)(\d+)(\s*[^\.\d\[])")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        // Handle * N followed by comparison operators (>= <= == != > <)
        result = Regex::new(r"(\*\s*)(\d+)(\s*[><=!])")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        // Handle *N (no space) followed by comparison operators
        result = Regex::new(r"\*(\d+)(\s*[><=!])")
            .unwrap()
            .replace_all(&result, "*$1.0$2")
            .to_string();

        // Handle * N followed by + or -
        result = Regex::new(r"(\*\s*)(\d+)(\s*[\+\-])")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        // Handle *N (no space) followed by + or -
        result = Regex::new(r"\*(\d+)(\s*[\+\-])")
            .unwrap()
            .replace_all(&result, "*$1.0$2")
            .to_string();

        // Handle * N where N is before a non-float context like < or comparison
        result = Regex::new(r"\*\s*(\d+)\s*<")
            .unwrap()
            .replace_all(&result, "* $1.0 <")
            .to_string();

        // Handle modulo operator % N -> % N.0
        result = Regex::new(r"(%\s*)(\d+)")
            .unwrap()
            .replace_all(&result, "$1$2.0")
            .to_string();

        // Convert integers BEFORE arithmetic operators: 4 * x -> 4.0 * x
        // Handle { 4 * ... } style
        result = Regex::new(r"\{\s*(\d+)\s*\*")
            .unwrap()
            .replace_all(&result, "{ $1.0 *")
            .to_string();

        result = Regex::new(r"\{\s*(\d+)\s*/")
            .unwrap()
            .replace_all(&result, "{ $1.0 /")
            .to_string();

        // Handle = 2 * ... (assignment with integer at start)
        result = Regex::new(r"=\s*(\d+)\s*\*")
            .unwrap()
            .replace_all(&result, "= $1.0 *")
            .to_string();

        result = Regex::new(r"=\s*(\d+)\s*/")
            .unwrap()
            .replace_all(&result, "= $1.0 /")
            .to_string();

        // Handle integer * float patterns like "12 * 0.25" -> "12.0 * 0.25"
        // Only match integers that are preceded by spaces, operators, or punctuation (not variable names)
        // This prevents matching "2" in "hitdmg2 * 2.0"
        result = Regex::new(r"([\s({=+\-*/,;:])(\d+)\s*\*\s*(\d+\.\d+)")
            .unwrap()
            .replace_all(&result, "$1$2.0 * $3")
            .to_string();

        // Handle "if 12 * 0.25" pattern (if followed by integer * float)
        result = Regex::new(r"\bif\s+(\d+)\s*\*\s*(\d+\.\d+)")
            .unwrap()
            .replace_all(&result, "if $1.0 * $2")
            .to_string();

        // Handle += 3 * ... (compound assignment with integer at start)
        result = Regex::new(r"\+=\s*(\d+)\s*\*")
            .unwrap()
            .replace_all(&result, "+= $1.0 *")
            .to_string();

        result = Regex::new(r"-=\s*(\d+)\s*\*")
            .unwrap()
            .replace_all(&result, "-= $1.0 *")
            .to_string();

        // Handle compound assignments: *= 2 -> *= 2.0
        result = Regex::new(r"(\*=\s*)(\d+)(\s*;)")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        result = Regex::new(r"(/=\s*)(\d+)(\s*;)")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        result = Regex::new(r"(\+=\s*)(\d+)(\s*;)")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        result = Regex::new(r"(-=\s*)(\d+)(\s*;)")
            .unwrap()
            .replace_all(&result, "$1$2.0$3")
            .to_string();

        // Convert (1 + ...) style expressions
        result = Regex::new(r"\((\d+)\s*\+")
            .unwrap()
            .replace_all(&result, "($1.0 +")
            .to_string();

        result = Regex::new(r"\((\d+)\s*-")
            .unwrap()
            .replace_all(&result, "($1.0 -")
            .to_string();

        result = Regex::new(r"\((\d+)\s*\*")
            .unwrap()
            .replace_all(&result, "($1.0 *")
            .to_string();

        result = Regex::new(r"\((\d+)\s*/")
            .unwrap()
            .replace_all(&result, "($1.0 /")
            .to_string();

        result
    }

    /// Translate Python ternary expressions
    fn translate_ternary(&self, expr: &str) -> String {
        // Match: "value_if_true if condition else value_if_false"
        // Use a more careful approach to handle nested ternaries

        if !expr.contains(" if ") || !expr.contains(" else ") {
            return expr.to_string();
        }

        // Find the rightmost "else" first, then work backwards
        if let Some(else_pos) = expr.rfind(" else ") {
            let before_else = &expr[..else_pos];
            let after_else = &expr[else_pos + 6..];

            // Now find the matching "if" in the before_else part
            if let Some(if_pos) = before_else.rfind(" if ") {
                let true_val = &before_else[..if_pos];
                let condition = &before_else[if_pos + 4..];
                let false_val = after_else;

                let rust_cond = self.translate_condition(condition);
                let rust_true = self.translate_expression(true_val);
                let rust_false = self.translate_expression(false_val);

                return format!(
                    "if {rust_cond} {{ {rust_true} }} else {{ {rust_false} }}"
                );
            }
        }

        expr.to_string()
    }

    /// Translate np.fmax with proper nesting support
    fn translate_np_fmax(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        loop {
            if let Some(start) = result.find("np.fmax(") {
                let args_start = start + 8;

                // Find matching closing paren
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
            } else {
                break;
            }
        }

        result
    }

    /// Translate np.fmin
    fn translate_np_fmin(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        loop {
            if let Some(start) = result.find("np.fmin(") {
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
            } else {
                break;
            }
        }

        result
    }

    /// Split function arguments at the top level comma
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

    /// Translate min/max functions
    fn translate_min_max(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Pre-compile regex patterns outside the loop for performance
        let min_re = Regex::new(r"(?:^|[^.])min\(").unwrap();
        let max_re = Regex::new(r"(?:^|[^.])max\(").unwrap();

        // Handle min(a, b) - but not .min( which is already-translated method calls
        loop {
            // Match min( only when not preceded by .
            if let Some(m) = min_re.find(&result) {
                // Adjust start position if we matched a character before 'min'
                let (start, args_start) = if result[m.start()..].starts_with("min(") {
                    (m.start(), m.start() + 4) // "min(" is 4 chars
                } else {
                    (m.start() + 1, m.start() + 5) // skip the matched char, "min(" is 4 chars
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
            } else {
                break;
            }
        }

        // Handle max(a, b) - but not .max( which is already-translated method calls
        loop {
            // Match max( only when not preceded by .
            if let Some(m) = max_re.find(&result) {
                // Adjust start position if we matched a character before 'max'
                let (start, args_start) = if result[m.start()..].starts_with("max(") {
                    (m.start(), m.start() + 4) // "max(" is 4 chars
                } else {
                    (m.start() + 1, m.start() + 5) // skip the matched char, "max(" is 4 chars
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
            } else {
                break;
            }
        }

        result
    }

    /// Translate self.X references to Rust equivalents
    fn translate_self_references(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // self.skill -> self.unit.skill_index (special case)
        result = Regex::new(r"\bself\.skill\b")
            .unwrap()
            .replace_all(&result, "self.unit.skill_index")
            .to_string();

        // self.module -> self.unit.module_index (special case)
        result = Regex::new(r"\bself\.module\b")
            .unwrap()
            .replace_all(&result, "self.unit.module_index")
            .to_string();

        // self.elite -> self.unit.elite
        result = Regex::new(r"\bself\.elite\b")
            .unwrap()
            .replace_all(&result, "self.unit.elite")
            .to_string();

        // Map Python names to Rust field names (simple mappings first)
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

        for (py_name, rs_name) in simple_mappings {
            let pattern = format!(r"\bself\.{py_name}\b");
            let replacement = format!("self.unit.{rs_name}");
            result = Regex::new(&pattern)
                .unwrap()
                .replace_all(&result, replacement.as_str())
                .to_string();
        }

        // Integer fields that need type casting when used in arithmetic
        // Note: Parentheses ARE needed for `as` casts in comparisons because
        // `x as f64 < y` is parsed as `x as f64<y>` (generic type), not comparison
        result = Regex::new(r"\bself\.unit\.skill_index\b")
            .unwrap()
            .replace_all(&result, "(self.unit.skill_index as f64)")
            .to_string();

        result = Regex::new(r"\bself\.unit\.elite\b")
            .unwrap()
            .replace_all(&result, "(self.unit.elite as f64)")
            .to_string();

        result = Regex::new(r"\bself\.unit\.module_index\b")
            .unwrap()
            .replace_all(&result, "(self.unit.module_index as f64)")
            .to_string();

        result = Regex::new(r"\bself\.unit\.module_level\b")
            .unwrap()
            .replace_all(&result, "(self.unit.module_level as f64)")
            .to_string();

        // Fields that need type casting (use full replacement syntax)
        // Note: Parentheses needed to avoid parsing issues with comparison operators
        result = Regex::new(r"\bself\.atk_interval\b")
            .unwrap()
            .replace_all(&result, "(self.unit.attack_interval as f64)")
            .to_string();

        result = Regex::new(r"\bself\.skill_cost\b")
            .unwrap()
            .replace_all(&result, "(self.unit.skill_cost as f64)")
            .to_string();

        result = Regex::new(r"\bself\.targets\b")
            .unwrap()
            .replace_all(&result, "(self.unit.targets as f64)")
            .to_string();

        result = Regex::new(r"\bself\.sp_boost\b")
            .unwrap()
            .replace_all(&result, "(self.unit.sp_boost as f64)")
            .to_string();

        // drone_atk_interval is f32, needs casting
        result = Regex::new(r"\bself\.drone_atk_interval\b")
            .unwrap()
            .replace_all(&result, "(self.unit.drone_atk_interval as f64)")
            .to_string();

        // Rename reserved keywords in expressions (not just assignments)
        result = Regex::new(r"\bcrate\b")
            .unwrap()
            .replace_all(&result, "crit_rate")
            .to_string();

        // Handle cloned_op references (complex Muelsyse-specific feature)
        // Mark as placeholder that needs manual implementation
        // Use trailing space to prevent */ followed by / from becoming *// which gets replaced
        // Use false for boolean fields, 0.0 for numeric fields
        result = Regex::new(r"self\.cloned_op\.(ranged|melee|physical|trait_damage|talent_damage|skill_damage|module_damage)")
            .unwrap()
            .replace_all(&result, "false /* cloned_op.$1 */ ")
            .to_string();
        result = Regex::new(r"self\.cloned_op\.(\w+)")
            .unwrap()
            .replace_all(&result, "0.0 /* cloned_op.$1 */ ")
            .to_string();

        // Handle operator-specific Python attributes that don't exist in Rust struct
        // These are custom parameters used in Python DPS calculations
        // Use appropriate placeholder values based on typical usage

        // self.hits - typically a float representing hit count
        result = Regex::new(r"\bself\.hits\b")
            .unwrap()
            .replace_all(&result, "1.0 /* self.hits - needs manual implementation */")
            .to_string();

        // self.pot - potential-related value (typically 1-6)
        result = Regex::new(r"\bself\.pot\b")
            .unwrap()
            .replace_all(&result, "(self.unit.potential as f64)")
            .to_string();

        // self.freezeRate - Kjera-specific freeze rate parameter
        result = Regex::new(r"\bself\.freezeRate\b")
            .unwrap()
            .replace_all(
                &result,
                "0.0 /* self.freezeRate - needs manual implementation */",
            )
            .to_string();

        // self.count - counter variable (Vina-specific)
        result = Regex::new(r"\bself\.count\b")
            .unwrap()
            .replace_all(
                &result,
                "1.0 /* self.count - needs manual implementation */",
            )
            .to_string();

        // self.below50 - boolean for HP below 50% condition (LuoXiaohei)
        result = Regex::new(r"\bself\.below50\b")
            .unwrap()
            .replace_all(
                &result,
                "false /* self.below50 - needs manual implementation */",
            )
            .to_string();

        // self.ammo - ammunition count (ExecutorAlter)
        result = Regex::new(r"\bself\.ammo\b")
            .unwrap()
            .replace_all(&result, "1.0 /* self.ammo - needs manual implementation */")
            .to_string();

        // self.shadows - shadow count (Walter)
        result = Regex::new(r"\bself\.shadows\b")
            .unwrap()
            .replace_all(
                &result,
                "1.0 /* self.shadows - needs manual implementation */",
            )
            .to_string();

        // self.params - operator-specific custom parameter (Stainless etc)
        result = Regex::new(r"\bself\.params\b")
            .unwrap()
            .replace_all(
                &result,
                "1.0 /* self.params - needs manual implementation */",
            )
            .to_string();

        // self.params2 - secondary custom parameter (Stainless)
        result = Regex::new(r"\bself\.params2\b")
            .unwrap()
            .replace_all(
                &result,
                "1.0 /* self.params2 - needs manual implementation */",
            )
            .to_string();

        // self.no_kill - no-kill condition (Necrass)
        result = Regex::new(r"\bself\.no_kill\b")
            .unwrap()
            .replace_all(
                &result,
                "false /* self.no_kill - needs manual implementation */",
            )
            .to_string();

        // Handle Python slice syntax with min/max
        // min(array[1:]) -> array[1..].iter()...
        result = Regex::new(r"\bmin\(self\.unit\.(\w+)\[(\d+):\]\)")
            .unwrap()
            .replace_all(
                &result,
                "self.unit.$1[$2..].iter().cloned().fold(f64::INFINITY, f64::min)",
            )
            .to_string();

        // max(array[:N]) -> array[..N].iter()...
        result = Regex::new(r"\bmax\(self\.unit\.(\w+)\[:(\d+)\]\)")
            .unwrap()
            .replace_all(
                &result,
                "self.unit.$1[..$2].iter().cloned().fold(f64::NEG_INFINITY, f64::max)",
            )
            .to_string();

        // min(array[:N]) -> array[..N].iter()...
        result = Regex::new(r"\bmin\(self\.unit\.(\w+)\[:(\d+)\]\)")
            .unwrap()
            .replace_all(
                &result,
                "self.unit.$1[..$2].iter().cloned().fold(f64::INFINITY, f64::min)",
            )
            .to_string();

        // max(array[N:]) -> array[N..].iter()...
        result = Regex::new(r"\bmax\(self\.unit\.(\w+)\[(\d+):\]\)")
            .unwrap()
            .replace_all(
                &result,
                "self.unit.$1[$2..].iter().cloned().fold(f64::NEG_INFINITY, f64::max)",
            )
            .to_string();

        // Handle single-argument max/min (e.g., max(talent_params) -> take first element)
        result = Regex::new(r"\bmax\(self\.unit\.(\w+)\)")
            .unwrap()
            .replace_all(
                &result,
                "self.unit.$1.iter().cloned().fold(f64::NEG_INFINITY, f64::max)",
            )
            .to_string();

        result = Regex::new(r"\bmin\(self\.unit\.(\w+)\)")
            .unwrap()
            .replace_all(
                &result,
                "self.unit.$1.iter().cloned().fold(f64::INFINITY, f64::min)",
            )
            .to_string();

        result
    }

    /// Translate Python ** power operator to Rust .powf()
    fn translate_power_operator(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Match patterns like "base**exponent" and convert to "(base as f64).powf(exponent as f64)"
        // Handle parenthesized expressions like (1-x)**y and also "base ** exp" with spaces
        let mut iterations = 0;
        loop {
            iterations += 1;
            if iterations > 100 {
                eprintln!("POWER OPERATOR INFINITE LOOP DETECTED! expr={expr}");
                eprintln!("CURRENT result={result}");
                break;
            }
            if let Some(pos) = result.find("**") {
                let before = &result[..pos];
                let after_raw = &result[pos + 2..];

                // Trim whitespace from both sides of ** to handle "base ** exponent"
                let before_trimmed = before.trim_end();
                let after = after_raw.trim_start();
                let whitespace_after_len = after_raw.len() - after.len();

                // Find the base - handle parenthesized expressions
                let base_start_in_trimmed;
                let base;
                let chars: Vec<char> = before_trimmed.chars().collect();

                if chars.last() == Some(&')') {
                    // Base is a parenthesized expression - find matching open paren
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
                    // Simple base - find the start (looking for delimiters)
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

                // Find the exponent - go forwards to find the end
                // Handle parenthesized exponents too
                let exponent;
                let exp_end;
                if after.starts_with('(') {
                    // Find matching close paren
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

                // Skip if base or exponent is empty
                if base.trim().is_empty() || exponent.trim().is_empty() {
                    break;
                }

                // Build replacement
                let replacement =
                    format!("({} as f64).powf({} as f64)", base.trim(), exponent.trim());
                // Use the position in the original string for before
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

    /// Translate Python 'in' operator
    fn translate_in_operator(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Handle "X in [a, b, c]" -> "[a.0, b.0, c.0].contains(&((X) as f64))"
        // Convert array elements to floats and compare as f64 for consistency
        let in_list_re = Regex::new(r"(\S+)\s+in\s+\[([^\]]+)\]").unwrap();
        result = in_list_re
            .replace_all(&result, |caps: &regex::Captures| {
                let item = &caps[1];
                let list_items = &caps[2];
                // Convert array elements to floats if they're integers
                let float_items: Vec<String> = list_items
                    .split(',')
                    .map(|e| {
                        let trimmed = e.trim();
                        if Regex::new(r"^\d+$").unwrap().is_match(trimmed) {
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
            .to_string();

        result
    }

    /// Ensure proper float division
    fn ensure_float_division(&self, expr: &str) -> String {
        let mut result = expr.to_string();

        // Replace / followed by bare integer with / integer.0
        let div_re = Regex::new(r"/\s*(\d+)(\s|$|\)|,|\])").unwrap();
        result = div_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("/ {}.0{}", &caps[1], &caps[2])
            })
            .to_string();

        result
    }
}

/// Parse the Python file and extract all operator classes
fn parse_python_file(path: &Path) -> Result<Vec<OperatorClass>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;

    let mut operators = Vec::new();

    // Find all class definitions
    let class_pattern = Regex::new(r"(?m)^class\s+(\w+)\(Operator\):").unwrap();

    // Collect all class positions
    let class_positions: Vec<(usize, &str)> = class_pattern
        .captures_iter(&content)
        .filter_map(|caps| {
            let full_match = caps.get(0)?;
            let name = caps.get(1)?.as_str();
            Some((full_match.start(), name))
        })
        .collect();

    for (idx, (start_pos, class_name)) in class_positions.iter().enumerate() {
        // Skip utility classes
        if matches!(
            *class_name,
            "Operator" | "AttackSpeed" | "NewBlueprint" | "Guide" | "Defense" | "Res"
        ) {
            continue;
        }

        // Determine class body end (next class or end of file)
        let end_pos = if idx + 1 < class_positions.len() {
            class_positions[idx + 1].0
        } else {
            content.len()
        };

        let class_body = &content[*start_pos..end_pos];

        // Parse super().__init__
        let super_pattern = Regex::new(
            r#"super\(\)\.__init__\(\s*"([^"]+)"\s*,\s*pp\s*,\s*\[([^\]]*)\]\s*,\s*\[([^\]]*)\](?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?"#,
        )
        .unwrap();

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
            // Couldn't parse super().__init__, use defaults
            (class_name.to_string(), vec![], vec![], 3, 1, 1)
        };

        // Extract method bodies
        let init_body = extract_method_body(class_body, "__init__");
        let skill_dps_body = extract_method_body(class_body, "skill_dps");
        let total_dmg_body = extract_method_body(class_body, "total_dmg");
        let has_total_dmg_override = !total_dmg_body.is_empty();

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
        });
    }

    Ok(operators)
}

/// Extract a method body from class content
fn extract_method_body(class_body: &str, method_name: &str) -> String {
    // Use [ \t]+ instead of \s+ to avoid matching newlines
    let method_pattern =
        Regex::new(&format!(r"(?m)^[ \t]+def\s+{method_name}\s*\([^)]*\):")).unwrap();

    if let Some(method_match) = method_pattern.find(class_body) {
        let method_start = method_match.end();
        let remaining = &class_body[method_start..];

        // Find method indent level - count leading whitespace in the matched line
        let method_text = method_match.as_str();
        let method_indent = method_text
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();

        // Find end of method (next line with same or less indent that's not empty,
        // or a new method definition)
        let mut body_lines = Vec::new();
        for line in remaining.lines() {
            if line.trim().is_empty() {
                body_lines.push(line.to_string());
                continue;
            }

            let line_indent = line.chars().take_while(|c| c.is_whitespace()).count();

            // Stop at new method definition at the same level
            if line_indent <= method_indent && line.trim().starts_with("def ") {
                break;
            }

            if line_indent <= method_indent && !line.trim().is_empty() {
                break;
            }
            body_lines.push(line.to_string());
        }

        // Remove trailing empty lines
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

/// Parse a Python list of integers like "1,2,3" or "1, 2"
fn parse_int_list(s: &str) -> Vec<i32> {
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

/// Generate a Rust file for an operator
fn generate_rust_file(op: &OperatorClass) -> String {
    let mut output = String::new();

    // Ensure struct name starts with uppercase (e.g., "twelveF" -> "TwelveF")
    let struct_name = to_upper_camel_case(&op.class_name);

    // Translate skill_dps body to Rust
    let translator = PythonToRustTranslator::new();
    let translated_skill_dps = if !op.skill_dps_body.is_empty() {
        translator.translate(&op.skill_dps_body)
    } else {
        "        // No skill_dps implementation found\n        self.unit.normal_attack(enemy, None, None, None)".to_string()
    };

    // Clean up skill_dps body for display in comments
    let skill_dps_comment = op
        .skill_dps_body
        .lines()
        .map(|l| format!("    /// {}", l.trim()))
        .collect::<Vec<_>>()
        .join("\n");

    // File header
    output.push_str(&format!(
        r#"//! DPS calculations for {}
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{{EnemyStats, OperatorParams, OperatorUnit}};
use super::super::super::operator_data::OperatorData;

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
        let unit = OperatorUnit::new(
            operator_data,
            params,
            {}, // default_skill_index
            {}, // default_potential
            {}, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self {{ unit }}
    }}

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
{}
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {{
        let defense = enemy.defense;
        let res = enemy.res;

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
        skill_dps_comment,
        translated_skill_dps,
    ));

    // Add total_dmg override if present
    if op.has_total_dmg_override {
        let translated_total_dmg = translator.translate(&op.total_dmg_body);
        let total_dmg_comment = op
            .total_dmg_body
            .lines()
            .map(|l| format!("    /// {}", l.trim()))
            .collect::<Vec<_>>()
            .join("\n");

        output.push_str(&format!(
            r#"
    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
{total_dmg_comment}
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens)]
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {{
        let defense = enemy.defense;
        let res = enemy.res;

{translated_total_dmg}
    }}
"#,
        ));
    }

    // Close impl block
    output.push_str("}\n");

    // Add Deref impl for easy access to unit fields
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
"#
    ));

    output
}

/// Generate mod.rs for a letter folder
fn generate_mod_file(operators: &[&OperatorClass]) -> String {
    let mut output = String::new();

    // Sort operators alphabetically
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

/// Generate the main operators mod.rs
fn generate_main_mod_file(letter_folders: &HashSet<char>) -> String {
    let mut output = String::new();

    let mut letters: Vec<char> = letter_folders.iter().copied().collect();
    letters.sort();

    output.push_str("//! Auto-generated operator implementations\n");
    output.push_str("//!\n");
    output.push_str("//! Each operator is organized into alphabetical subfolders.\n\n");

    for letter in &letters {
        output.push_str(&format!("pub mod {letter};\n"));
    }

    output.push('\n');

    for letter in &letters {
        output.push_str(&format!("pub use {letter}::*;\n"));
    }

    output
}

/// Ensure struct name starts with uppercase (for cases like "twelveF" -> "TwelveF")
fn to_upper_camel_case(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if !chars.is_empty() && chars[0].is_lowercase() {
        chars[0] = chars[0].to_ascii_uppercase();
    }
    chars.into_iter().collect()
}

/// Convert camelCase variable names to snake_case (e.g., "dpsNorm" -> "dps_norm")
fn camel_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            // Add underscore before uppercase letters (except at start)
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert PascalCase to snake_case
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

/// Get the first letter (lowercase) for folder organization
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

    let operators = match parse_python_file(input_path) {
        Ok(ops) => ops,
        Err(e) => {
            eprintln!("Error parsing Python file: {e}");
            std::process::exit(1);
        }
    };

    println!("Found {} operator classes", operators.len());
    println!("Translating Python skill_dps methods to Rust...");

    // Group operators by first letter
    let mut by_letter: HashMap<char, Vec<&OperatorClass>> = HashMap::new();
    for op in &operators {
        let letter = get_folder_letter(&op.class_name);
        by_letter.entry(letter).or_default().push(op);
    }

    // Remove old operators directory and create fresh
    if output_dir.exists() {
        if let Err(e) = fs::remove_dir_all(output_dir) {
            eprintln!("Warning: Failed to clean output directory: {e}");
        }
    }

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {e}");
        std::process::exit(1);
    }

    let mut letter_folders: HashSet<char> = HashSet::new();
    let mut generated_count = 0;

    // Generate files for each letter folder
    for (letter, ops) in &by_letter {
        letter_folders.insert(*letter);

        let letter_dir = output_dir.join(letter.to_string());
        if let Err(e) = fs::create_dir_all(&letter_dir) {
            eprintln!("Failed to create letter directory: {e}");
            continue;
        }

        // Generate individual operator files
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

        // Generate mod.rs for letter folder
        let mod_path = letter_dir.join("mod.rs");
        let mod_content = generate_mod_file(ops);
        if let Err(e) = fs::write(&mod_path, mod_content) {
            eprintln!("Failed to write {}: {}", mod_path.display(), e);
        }
    }

    // Generate main mod.rs
    let main_mod_path = output_dir.join("mod.rs");
    let main_mod_content = generate_main_mod_file(&letter_folders);
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

// Helper trait for sorting
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
