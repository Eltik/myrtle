// Clippy: pedantic/nursery noise intentional for this codegen tool (string-built
// Rust source, byte/bit casts, long emit functions). See ../lib.rs for rationale.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::implicit_hasher,
    clippy::option_if_let_else,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::items_after_statements,
    clippy::needless_pass_by_value,
    clippy::branches_sharing_code,
    clippy::or_fun_call,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::format_push_string,
    clippy::literal_string_with_formatting_args
)]

use anyhow::bail;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use unpacker::schema::store::normalize_fbs;

#[allow(dead_code)]
struct Field {
    name: String,
    return_type: String,
    is_option: bool,
    is_vector: bool,
    is_enum: bool,
    is_nested: bool,
    element_type: Option<String>, // "string", "nested", "enum", or None (scalar)
}

struct ParsedStruct {
    name: String,
    fields: Vec<Field>,
    is_dict: bool,
    is_root: bool,
    root_fn_name: Option<String>, // e.g. "root_as_clz_torappu_stage_table_unchecked"
}

/// Which tables need a Yostar (EN/Global) schema variant.
///
/// A table needs one exactly when `assets/schemas/yostar/<table>.fbs` exists
/// AND differs from `cn/<table>.fbs` after normalisation. `schema-sync` only
/// writes a Yostar file whose bytes differ from the CN choice, so in practice
/// the two conditions coincide; the comparison stays because two identical
/// schemas produce identical flatc output and cannot decode a buffer
/// differently, and a hand-copied file must not buy a dead variant (a second,
/// pointless verifier pass at runtime). Measured 2026-09-04 against the old
/// submodules: 31 of 57 differed, and every one of the 26 identical ones
/// produced a flatc-generated `.rs` byte-identical to its CN counterpart.
///
/// The set is COMPUTED from the store, never curated: a curated list goes
/// stale silently, and a table whose EN layout has just diverged keeps
/// decoding to garbage because nobody added it.
fn compute_yostar_schemas(cn_fbs_dir: &Path, yostar_fbs_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut differ = Vec::new();
    let mut identical = Vec::new();
    let mut orphan = Vec::new();

    let mut entries: Vec<PathBuf> = fs::read_dir(yostar_fbs_dir)?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "fbs"))
        .collect();
    entries.sort();

    for yostar in &entries {
        let Some(stem) = yostar.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let cn = cn_fbs_dir.join(format!("{stem}.fbs"));
        if !cn.exists() {
            orphan.push(stem.to_string());
            continue;
        }
        let (Ok(cn_src), Ok(yostar_src)) = (fs::read_to_string(&cn), fs::read_to_string(yostar))
        else {
            continue;
        };
        if normalize_fbs(&cn_src) == normalize_fbs(&yostar_src) {
            identical.push(stem.to_string());
        } else {
            differ.push(stem.to_string());
        }
    }

    println!(
        "Yostar variants: {} of {} schemas differ from CN and get a variant",
        differ.len(),
        entries.len()
    );
    println!("  variant:   {}", differ.join(" "));
    println!("  identical: {}", identical.join(" "));
    if !orphan.is_empty() {
        println!("  no CN counterpart (skipped): {}", orphan.join(" "));
    }
    Ok(differ)
}

/// Removes stale generated modules so a table that no longer needs a Yostar
/// variant stops being compiled in (and stops being listed by
/// `has_yostar_schema`), together with its flatc `.log`.
fn clear_generated_dir(dir: &Path) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|ext| ext == "rs" || ext == "log")
        {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn run_flatc(fbs_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    let name = fbs_path.file_stem().unwrap().to_str().unwrap();
    println!("  flatc: {name}");

    let log_file_stdout = std::fs::File::create(output_dir.join(format!("{name}.log")))?;
    let log_file_stderr = log_file_stdout.try_clone()?; // .stdout/.stderr take ownership

    let status = Command::new("flatc")
        .args([
            "--rust",
            "--gen-object-api",
            "--rust-serialize",
            "-o",
            output_dir.to_str().unwrap(),
            fbs_path.to_str().unwrap(),
        ])
        .stdout(Stdio::from(log_file_stdout))
        .stderr(Stdio::from(log_file_stderr))
        .status()?;

    if !status.success() {
        eprintln!("  warning: flatc failed for {name}");
    }

    Ok(())
}

fn run_flatc_all(fbs_dir: &Path, output_dir: &Path) -> anyhow::Result<()> {
    println!("Running flatc on all schemas...");

    let mut entries: Vec<_> = fs::read_dir(fbs_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fbs"))
        .collect();
    entries.sort_by_key(std::fs::DirEntry::path);

    println!("Running flatc on {} schemas...", entries.len());
    entries
        .par_iter()
        .try_for_each(|entry| run_flatc(&entry.path(), output_dir))?;
    Ok(())
}

fn strip_serialize_impls(dir: &Path) -> anyhow::Result<()> {
    let serde_import_re = Regex::new(r"^use (?:self::)?serde(?:::ser)?::\{.*?\};$")?;
    let entries = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "mod.rs") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let mut output = String::with_capacity(content.len());
        let mut skip = false;
        let mut brace_depth = 0i32;
        let mut changed = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if skip {
                // Inside a Serialize impl block — count braces
                for ch in line.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }
                if brace_depth <= 0 {
                    skip = false;
                }
            } else {
                // Detect start of impl Serialize block
                if trimmed.starts_with("impl Serialize for ")
                    || trimmed.starts_with("impl<'a> Serialize for ")
                {
                    skip = true;
                    brace_depth = 0;
                    changed = true;
                    // Count braces on this line
                    for ch in line.chars() {
                        match ch {
                            '{' => brace_depth += 1,
                            '}' => brace_depth -= 1,
                            _ => {}
                        }
                    }
                    if brace_depth <= 0 {
                        skip = false; // Single-line impl (unlikely but safe)
                    }
                    continue;
                }

                // Strip serde imports
                if serde_import_re.is_match(trimmed) {
                    changed = true;
                    continue;
                }

                // Strip trailing whitespace flatc emits (e.g. `#[must_use] `),
                // which rustfmt on stable rejects ("left behind trailing whitespace").
                let stripped = line.trim_end();
                if stripped.len() != line.len() {
                    changed = true;
                }
                output.push_str(stripped);
                output.push('\n');
            }
        }

        if changed {
            let name = path.file_name().unwrap().to_str().unwrap();
            println!("  stripped serde from {name}");
            fs::write(&path, &output)?;
        }
    }

    Ok(())
}

fn generate_mod_rs(dir: &Path) -> anyhow::Result<()> {
    let mut modules: Vec<String> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_str().unwrap().to_string();
        if name.ends_with("_generated.rs") {
            let mod_name = name.strip_suffix(".rs").unwrap().to_string();
            modules.push(mod_name);
        }
    }
    modules.sort();

    let mut out = String::new();
    out.push_str("// Auto-generated - do not edit\n\n");
    out.push_str("#![allow(dead_code, unused_imports, non_snake_case, non_camel_case_types, unreachable_patterns, clippy::all)]\n\n");

    for m in &modules {
        out.push_str(&format!("pub mod {m};\n"));
    }

    fs::write(dir.join("mod.rs"), &out)?;
    println!(
        "Generated mod.rs with {} modules in {}",
        modules.len(),
        dir.display()
    );
    Ok(())
}

fn parse_all_generated(dir: &Path) -> anyhow::Result<HashMap<String, Vec<ParsedStruct>>> {
    let mut result = HashMap::new();
    let method_re = Regex::new(r"pub fn (\w+)\(&self\)\s*->\s*([^{]+?)\s*\{").unwrap();
    let inner_re = Regex::new(r"ForwardsUOffset<([^>]+)>").unwrap();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if !name.ends_with("_generated.rs") {
            continue;
        }

        let module_name = name.strip_suffix(".rs").unwrap().to_string();
        print!("  parsing {module_name}...");
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        let content = fs::read_to_string(&path)?;
        let structs = parse_structs(&content, &method_re, &inner_re);
        println!(" {} structs", structs.len());

        if !structs.is_empty() {
            println!("  {module_name}: {} structs", structs.len());
            result.insert(module_name, structs);
        }
    }
    Ok(result)
}

fn parse_all_enums(dir: &Path) -> anyhow::Result<HashMap<String, Vec<String>>> {
    let mut result = HashMap::new();

    // Matches: pub struct enum__SomeName(pub i32) or (pub u8)
    let re = Regex::new(r"pub struct (enum__\w+)\(pub (?:i32|u8)\)")?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if !name.ends_with("_generated.rs") {
            continue;
        }

        let module_name = name.strip_suffix(".rs").unwrap().to_string();
        let content = fs::read_to_string(&path)?;

        let enums: Vec<String> = re
            .captures_iter(&content)
            .map(|c| c[1].to_string())
            .collect();

        if !enums.is_empty() {
            println!("  {module_name}: {} enums", enums.len());
            result.insert(module_name, enums);
        }
    }

    Ok(result)
}

fn parse_structs(content: &str, method_re: &Regex, inner_re: &Regex) -> Vec<ParsedStruct> {
    let mut structs = Vec::new();

    let prefix_re = Regex::new(r"^pub struct ((?:clz_|dict__|list_|kvp__|hg__)\w+)<").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(cap) = prefix_re.captures(trimmed) {
            // Verify this is a FlatBuffer table struct (has _tab field)
            // Check if "pub _tab:" appears nearby
            let struct_name = cap[1].to_string();

            // Quick check: find "pub _tab:" after this struct definition
            if let Some(pos) = content.find(&format!("pub struct {struct_name}<")) {
                let after = &content[pos..std::cmp::min(pos + 200, content.len())];
                if !after.contains("pub _tab:") {
                    continue;
                }
            }

            let fields = parse_struct_fields(content, &struct_name, method_re, inner_re);

            let field_names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
            let is_dict = field_names.contains(&"key") && field_names.contains(&"value");

            let root_fn_name = find_root_fn(content, &struct_name);
            let is_root = root_fn_name.is_some();

            structs.push(ParsedStruct {
                name: struct_name,
                fields,
                is_dict,
                is_root,
                root_fn_name,
            });
        }
    }

    structs
}

fn find_root_fn(content: &str, struct_name: &str) -> Option<String> {
    // Look for: pub unsafe fn root_as_xxx_unchecked(buf: &[u8]) -> StructName
    // Return type may or may not have lifetime: -> StructName<'a> or -> StructName {
    let search = "pub unsafe fn root_as_";
    let with_lt = format!("-> {struct_name}<");
    let with_space = format!("-> {struct_name} ");
    let with_brace = format!("-> {struct_name}{{");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(search)
            && (trimmed.contains(&with_lt)
                || trimmed.contains(&with_space)
                || trimmed.contains(&with_brace))
        {
            let after = trimmed.strip_prefix("pub unsafe fn ").unwrap();
            if let Some(paren) = after.find('(') {
                return Some(after[..paren].to_string());
            }
        }
    }
    None
}

fn parse_struct_fields(
    content: &str,
    struct_name: &str,
    method_re: &Regex,
    inner_re: &Regex,
) -> Vec<Field> {
    let mut fields = Vec::new();

    // Find impl block using plain string search
    let needle = format!("impl<'a> {struct_name}<'a>");
    let impl_start = match content.find(&needle) {
        Some(pos) => pos,
        None => return fields,
    };

    // Find opening brace
    let brace_start = match content[impl_start..].find('{') {
        Some(pos) => impl_start + pos + 1,
        None => return fields,
    };

    // Find matching close brace
    let mut brace_count = 1i32;
    let mut end = brace_start;
    let bytes = content.as_bytes();
    while brace_count > 0 && end < bytes.len() {
        match bytes[end] {
            b'{' => brace_count += 1,
            b'}' => brace_count -= 1,
            _ => {}
        }
        end += 1;
    }

    let impl_content = &content[brace_start..end];

    for cap in method_re.captures_iter(impl_content) {
        let method_name = cap[1].to_string();
        let return_type = cap[2].trim().to_string();

        if method_name.contains("VT_")
            || method_name.contains("init_from_table")
            || method_name.contains("create")
            || method_name.contains("key_compare")
            || method_name.contains("unpack")
            || method_name.starts_with('_')
            || method_name.ends_with("Length")
        {
            continue;
        }

        let is_option = return_type.starts_with("Option<");
        let is_vector = return_type.contains("Vector<");
        let has_nested_prefix = return_type.contains("clz_")
            || return_type.contains("dict__")
            || return_type.contains("list_")
            || return_type.contains("kvp__")
            || return_type.contains("hg__");
        // Only mark as enum if it contains enum__ and is NOT a nested type
        // (e.g. list_dict__enum__X__Y is nested, not an enum)
        let is_enum = return_type.contains("enum__") && !has_nested_prefix;
        let is_nested = has_nested_prefix;

        let element_type = if is_vector {
            let from_inner = inner_re.captures(&return_type).map(|c| {
                let inner = c[1].trim();
                if inner.contains("&'a str") || inner == "&str" {
                    "string".to_string()
                } else if inner.contains("clz_")
                    || inner.contains("dict__")
                    || inner.contains("list_")
                    || inner.contains("kvp__")
                    || inner.contains("hg__")
                {
                    "nested".to_string()
                } else if inner.contains("enum__") {
                    "enum".to_string()
                } else {
                    "scalar".to_string()
                }
            });
            // Fallback: if no ForwardsUOffset match, check raw return type
            // Handles Vector<'a, enum__Type> (no ForwardsUOffset wrapper)
            from_inner.or_else(|| {
                if return_type.contains("enum__") {
                    Some("enum".to_string())
                } else if return_type.contains("clz_")
                    || return_type.contains("dict__")
                    || return_type.contains("list_")
                    || return_type.contains("kvp__")
                    || return_type.contains("hg__")
                {
                    Some("nested".to_string())
                } else {
                    None // true scalar vector (i32, f32, etc.)
                }
            })
        } else {
            None
        };

        fields.push(Field {
            name: method_name,
            return_type,
            is_option,
            is_vector,
            is_enum,
            is_nested: is_nested && !is_vector,
            element_type,
        });
    }

    fields
}

fn generate_fb_json_auto(
    output_path: &Path,
    structs: &HashMap<String, Vec<ParsedStruct>>,
    enums: &HashMap<String, Vec<String>>,
    module_prefix: &str,
) -> anyhow::Result<()> {
    let mut out = String::new();

    out.push_str("//! Auto-generated FlatBufferToJson implementations\n");
    out.push_str("//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs\n\n");
    out.push_str("#![allow(unused_imports, unused_variables)]\n\n");
    out.push_str("use crate::fb_json_macros::{FlatBufferToJson, EnumToJson};\n");
    out.push_str("use serde_json::{json, Map, Value};\n");
    out.push_str("use std::panic::{self, AssertUnwindSafe};\n\n");

    let mut all_modules: Vec<&String> = structs.keys().chain(enums.keys()).collect();
    all_modules.sort();
    all_modules.dedup();

    for module in &all_modules {
        out.push_str(&format!("use crate::{module_prefix}::{module};\n"));
    }
    out.push('\n');

    out.push_str("// ============ Enum Implementations ============\n\n");

    let mut enum_modules: Vec<&String> = enums.keys().collect();
    enum_modules.sort();

    for module in &enum_modules {
        let enum_names = &enums[*module];
        out.push_str(&format!("// From {module}\n"));
        for enum_name in enum_names {
            out.push_str(&format!(
                "impl EnumToJson for {module}::{enum_name} {{\n\
                \x20   fn to_json_value(&self) -> Value {{\n\
                \x20       match self.variant_name() {{\n\
                \x20           Some(name) => json!(name),\n\
                \x20           None => json!(format!(\"UNKNOWN_{{}}\", self.0)),\n\
                \x20       }}\n\
                \x20   }}\n\
                }}\n\n"
            ));
        }
    }

    out.push_str("\n// ============ Struct Implementations ============\n\n");

    let mut struct_modules: Vec<&String> = structs.keys().collect();
    struct_modules.sort();

    for module in &struct_modules {
        let parsed_structs = &structs[*module];
        out.push_str(&format!("// From {module}\n"));
        for s in parsed_structs {
            emit_struct_impl(&mut out, s, module);
        }
    }

    fs::write(output_path, &out)?;
    let struct_count: usize = structs.values().map(std::vec::Vec::len).sum();
    let enum_count: usize = enums.values().map(std::vec::Vec::len).sum();
    println!(
        "Generated {} ({struct_count} structs, {enum_count} enums)",
        output_path.display()
    );
    Ok(())
}

fn emit_struct_impl(out: &mut String, s: &ParsedStruct, module: &str) {
    out.push_str(&format!(
        "impl FlatBufferToJson for {module}::{}<'_> {{\n\
        \x20   fn to_json(&self) -> Value {{\n",
        s.name
    ));

    if s.is_dict {
        emit_dict_impl(out, s, module);
    } else {
        if s.fields.is_empty() {
            out.push_str("        let map = Map::new();\n");
        } else {
            out.push_str("        let mut map = Map::new();\n");
        }

        for field in &s.fields {
            let pascal = pascal_case(&field.name);
            emit_field_safe(out, field, &pascal);
        }

        out.push_str("        Value::Object(map)\n");
    }

    out.push_str("    }\n}\n\n");
}

fn emit_dict_impl(out: &mut String, s: &ParsedStruct, _module: &str) {
    out.push_str("        let mut map = Map::new();\n");

    let key_field = s.fields.iter().find(|f| f.name == "key");
    let value_field = s.fields.iter().find(|f| f.name == "value");

    // Key
    if let Some(kf) = key_field {
        if kf.is_option && kf.is_enum {
            out.push_str("        if let Some(k) = self.key() {\n");
            out.push_str("            map.insert(\"key\".to_string(), k.to_json_value());\n");
            out.push_str("        }\n");
        } else if kf.is_option {
            let value = json_expr(kf, "k");
            out.push_str("        if let Some(k) = self.key() {\n");
            out.push_str(&format!(
                "            map.insert(\"key\".to_string(), {value});\n"
            ));
            out.push_str("        }\n");
        } else if kf.is_enum {
            out.push_str(
                "        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {\n",
            );
            out.push_str("            map.insert(\"key\".to_string(), k.to_json_value());\n");
            out.push_str("        }\n");
        } else {
            let value = json_expr(kf, "k");
            out.push_str(
                "        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {\n",
            );
            out.push_str(&format!(
                "            map.insert(\"key\".to_string(), {value});\n"
            ));
            out.push_str("        }\n");
        }
    }

    // Value
    if let Some(vf) = value_field {
        if vf.is_option {
            if vf.is_nested {
                out.push_str("        if let Some(v) = self.value() {\n");
                out.push_str("            map.insert(\"value\".to_string(), v.to_json());\n");
                out.push_str("        }\n");
            } else if vf.is_enum {
                out.push_str("        if let Some(v) = self.value() {\n");
                out.push_str("            map.insert(\"value\".to_string(), v.to_json_value());\n");
                out.push_str("        }\n");
            } else if vf.is_vector {
                if vf.element_type.as_deref() == Some("nested") {
                    out.push_str("        if let Some(vec) = self.value() {\n");
                    out.push_str(
                        "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                    );
                    out.push_str("            let arr: Vec<Value> = (0..vec.len()).map(|i| vec.get(i).to_json()).collect();\n");
                    out.push_str("            map.insert(\"value\".to_string(), json!(arr));\n");
                    out.push_str("        }\n");
                } else {
                    let elem = if vf.element_type.as_deref() == Some("string") {
                        "crate::fb_json_macros::json_str(v)"
                    } else {
                        "json!(v)"
                    };
                    out.push_str("        if let Some(vec) = self.value() {\n");
                    out.push_str(
                        "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                    );
                    out.push_str(&format!(
                        "            let arr: Vec<Value> = vec.iter().map(|v| {elem}).collect();\n"
                    ));
                    out.push_str("            map.insert(\"value\".to_string(), json!(arr));\n");
                    out.push_str("        }\n");
                }
            } else {
                let value = json_expr(vf, "v");
                out.push_str("        if let Some(v) = self.value() {\n");
                out.push_str(&format!(
                    "            map.insert(\"value\".to_string(), {value});\n"
                ));
                out.push_str("        }\n");
            }
        } else if vf.is_nested {
            out.push_str("        map.insert(\"value\".to_string(), self.value().to_json());\n");
        } else if vf.is_enum {
            out.push_str(
                "        map.insert(\"value\".to_string(), self.value().to_json_value());\n",
            );
        } else {
            let value = json_expr(vf, "self.value()");
            out.push_str(&format!(
                "        map.insert(\"value\".to_string(), {value});\n"
            ));
        }
    }

    out.push_str("        Value::Object(map)\n");
}

fn emit_field_safe(out: &mut String, field: &Field, pascal_name: &str) {
    // Wrap each field in catch_unwind so a single corrupted field doesn't kill the struct
    out.push_str("        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {\n");

    if field.is_vector {
        match field.element_type.as_deref() {
            Some("string") => {
                out.push_str(&format!(
                    "            if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "                assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("                let arr: Vec<Value> = (0..vec.len()).map(|i| crate::fb_json_macros::json_str(vec.get(i))).collect();\n");
                out.push_str(&format!(
                    "                return Some((\"{pascal_name}\".to_string(), json!(arr)));\n"
                ));
                out.push_str("            }\n");
            }
            Some("nested") => {
                out.push_str(&format!(
                    "            if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "                assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("                let arr: Vec<Value> = (0..vec.len()).filter_map(|i| panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()).collect();\n");
                out.push_str(&format!(
                    "                return Some((\"{pascal_name}\".to_string(), json!(arr)));\n"
                ));
                out.push_str("            }\n");
            }
            Some("enum") => {
                out.push_str(&format!(
                    "            if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "                assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();\n");
                out.push_str(&format!(
                    "                return Some((\"{pascal_name}\".to_string(), json!(arr)));\n"
                ));
                out.push_str("            }\n");
            }
            _ => {
                out.push_str(&format!(
                    "            if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "                assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("                let arr: Vec<Value> = vec.iter().map(|v| json!(v)).collect();\n");
                out.push_str(&format!(
                    "                return Some((\"{pascal_name}\".to_string(), json!(arr)));\n"
                ));
                out.push_str("            }\n");
            }
        }
    } else if field.is_enum {
        if field.is_option {
            out.push_str(&format!(
                "            if let Some(e) = self.{}() {{\n",
                field.name
            ));
            out.push_str(&format!(
                "                return Some((\"{pascal_name}\".to_string(), e.to_json_value()));\n"
            ));
            out.push_str("            }\n");
        } else {
            out.push_str(&format!(
                "            return Some((\"{pascal_name}\".to_string(), self.{}().to_json_value()));\n",
                field.name
            ));
        }
    } else if field.is_nested {
        out.push_str(&format!(
            "            if let Some(nested) = self.{}() {{\n",
            field.name
        ));
        out.push_str(&format!(
            "                return Some((\"{pascal_name}\".to_string(), nested.to_json()));\n"
        ));
        out.push_str("            }\n");
    } else if field.is_option {
        let value = json_expr(field, "v");
        out.push_str(&format!(
            "            if let Some(v) = self.{}() {{\n",
            field.name
        ));
        out.push_str(&format!(
            "                return Some((\"{pascal_name}\".to_string(), {value}));\n"
        ));
        out.push_str("            }\n");
    } else {
        let value = json_expr(field, &format!("self.{}()", field.name));
        out.push_str(&format!(
            "            return Some((\"{pascal_name}\".to_string(), {value}));\n"
        ));
    }

    out.push_str("            #[allow(unreachable_code)]\n");
    out.push_str("            None\n");
    out.push_str("        })) {\n");
    out.push_str("            map.insert(k, v);\n");
    out.push_str("        }\n");
}

/// Does this accessor return a `FlatBuffer` string (`&'a str`), directly or
/// wrapped in `Option`?
///
/// Vectors are asked about through `Field::element_type` instead, so a
/// `Vector<'a, ForwardsUOffset<&'a str>>` must never reach here.
fn is_string_type(return_type: &str) -> bool {
    return_type.contains("&'a str") || return_type.contains("&str")
}

/// `json!(expr)` for a scalar, `json_str(expr)` for a string. The generated
/// accessors build their `&str` with `from_utf8_unchecked`, so a string field
/// can hand back bytes that are not valid UTF-8 and `json!` would write them
/// straight into the `.json` file, making the whole file unparseable. See
/// `fb_json_macros::json_str`.
fn json_expr(field: &Field, expr: &str) -> String {
    if is_string_type(&field.return_type) {
        format!("crate::fb_json_macros::json_str({expr})")
    } else {
        format!("json!({expr})")
    }
}

fn pascal_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// `root_as_x_unchecked` → `root_as_x_with_opts`, the verified root flatc emits
/// next to the unchecked one.
fn with_opts_fn(root_fn: &str) -> Option<String> {
    root_fn
        .strip_suffix("_unchecked")
        .map(|base| format!("{base}_with_opts"))
}

/// The first root accessor a generated module exposes (one root per module).
fn root_fn_for(structs: &HashMap<String, Vec<ParsedStruct>>, module: &str) -> Option<String> {
    structs
        .get(module)?
        .iter()
        .find_map(|s| s.root_fn_name.clone())
}

/// Emits `SchemaChoice`, `verify_schemas` and `select_schema_by_verification`
/// (the verifier options come from `crate::schema::opts`, never a copy):
/// the CN-vs-Yostar schema pick that runs *before* any decode.
fn emit_schema_verification(
    out: &mut String,
    cn_structs: &HashMap<String, Vec<ParsedStruct>>,
    yostar_structs: &HashMap<String, Vec<ParsedStruct>>,
    schema_to_module: &[(String, String)],
) {
    let mut arms = String::new();
    for (schema_type, module) in schema_to_module {
        let Some(cn_fn) = root_fn_for(cn_structs, module)
            .as_deref()
            .and_then(with_opts_fn)
        else {
            continue;
        };
        let yostar_fn = root_fn_for(yostar_structs, module)
            .as_deref()
            .and_then(with_opts_fn);

        arms.push_str(&format!(
            "        \"{schema_type}\" => {{\n\
             \x20           let cn_err = {{\n\
             \x20               use crate::generated_fbs::{module}::{cn_fn};\n\
             \x20               {cn_fn}(&opts, data).err().map(|e| first_line(&e))\n\
             \x20           }};\n"
        ));
        match yostar_fn {
            None => arms.push_str("            (cn_err, YostarVerdict::NotRun)\n        }\n"),
            Some(yostar_fn) => arms.push_str(&format!(
                "            if cn_err.is_none() && !full {{\n\
                 \x20               // The CN schema verified and the caller only wants\n\
                 \x20               // the routing decision: skip the second verify.\n\
                 \x20               return (None, YostarVerdict::NotRun);\n\
                 \x20           }}\n\
                 \x20           let yostar_err = {{\n\
                 \x20               use crate::generated_fbs_yostar::{module}::{yostar_fn};\n\
                 \x20               {yostar_fn}(&opts, data).err().map(|e| first_line(&e))\n\
                 \x20           }};\n\
                 \x20           (cn_err, yostar_err.into())\n\
                 \x20       }}\n"
            )),
        }
    }

    out.push_str(
        r#"/// Which schema a buffer actually verifies against.
 #[derive(Clone, Copy, PartialEq, Eq)]
 enum SchemaChoice {
     /// The CN schema verified. Decode exactly as before.
     Cn,
     /// The CN schema did not verify but the Yostar (EN/Global) one did.
     Yostar,
     /// Neither verified — fall back to the legacy decode-then-inspect path.
     Neither,
 }

 /// The Yostar half of a schema-verification verdict.
 ///
 /// A bare `Option<Option<String>>` collapses "no Yostar variant" and "Yostar
 /// variant not run" into one `None`, which `clippy::option_option` flags —
 /// and which reads ambiguously at every call site anyway.
 pub enum YostarVerdict {
     /// The table has no Yostar variant, or the caller short-circuited before
     /// running it (`full = false` and the CN schema already verified).
     NotRun,
     /// The Yostar schema verified.
     Verified,
     /// The Yostar schema did not verify; the first line of the error.
     Failed(String),
 }

 impl From<Option<String>> for YostarVerdict {
     fn from(err: Option<String>) -> Self {
         match err {
             None => Self::Verified,
             Some(e) => Self::Failed(e),
         }
     }
 }

 /// One table's verification verdict, for the `unpacker verify` report.
 pub struct TableVerdict {
     /// The schema type `guess_root_type` resolved the filename to.
     pub table: &'static str,
     /// `None` when the CN schema verified, else the first line of the error.
     pub cn: Option<String>,
     /// The Yostar schema's verdict, if it was run.
     pub yostar: YostarVerdict,
     /// What `decode_flatbuffer` will actually do with this buffer:
     /// `"CN"` (CN schema verified), `"Yostar"` (routed to the Yostar schema),
     /// or `"none"` (nothing verified — the table is skipped, no file written).
     pub chosen: &'static str,
 }

 /// `InvalidFlatbuffer` Displays as a multi-line trace; one line per table is
 /// what every caller here wants.
 fn first_line(e: &::flatbuffers::InvalidFlatbuffer) -> String {
     e.to_string().lines().next().unwrap_or_default().to_string()
 }

 /// Verify `data` against the schemas available for `schema_type`.
 ///
 /// Returns `(cn_err, yostar_verdict)`, where a `None` `cn_err` means the CN
 /// schema verified. `full = false` short-circuits: once the CN schema
 /// verifies the routing decision is already made, so the Yostar verifier is
 /// not run and `yostar_verdict` comes back `NotRun`. `full = true` (the
 /// `verify` subcommand) always runs both.
 ///
 /// The whole function returns `None` if a verifier panics — it is not supposed
 /// to, but neither was the decoder, and this is the one place that can still
 /// contain it.
 fn verify_schemas(
     data: &[u8],
     schema_type: &str,
     full: bool,
 ) -> Option<(Option<String>, YostarVerdict)> {
     let opts = verifier_opts();
     panic::catch_unwind(AssertUnwindSafe(|| match schema_type {
"#,
    );
    if arms.is_empty() {
        out.push_str("        _ => (None, YostarVerdict::NotRun),\n");
    } else {
        out.push_str(&arms);
        out.push_str("        _ => (None, YostarVerdict::NotRun),\n");
    }
    out.push_str("    }))\n    .ok()\n}\n\n");

    out.push_str(
        r#"/// Pick CN or Yostar for `schema_type` by VERIFYING the buffer, before any
 /// decode runs.
 ///
 /// WHY: only a handful of tables have a Yostar (EN/Global) schema variant, and
 /// the CN→Yostar fallback used to fire only when the CN decode came out EMPTY.
 /// That catches a mismatch that nulls everything out. It does not catch a
 /// mismatch that produces plausible-looking garbage. Measured on EN
 /// 26-08-28-10-20-08_ea3678: `activity_table` under the CN schema writes
 /// 5,211,117,589 bytes of JSON where the Yostar schema writes 13,143,425, and
 /// CN's own `activity_table` is 15,867,159. Not empty, so nothing detected it,
 /// and it takes peak RSS for one EN extraction from 431 MB to 8.56 GB — which
 /// is the 10 GB OOM kill the unpacker took on the VPS on 2026-08-04.
 ///
 /// Verification is the only signal that fires BEFORE the garbage is
 /// materialised. Every emptiness heuristic runs on a fully built
 /// `serde_json::Value`, i.e. after the memory has already been spent, whereas
 /// `root_as_*_with_opts` only walks offsets and vtables: it allocates no
 /// `Value` and serialises nothing.
 ///
 /// A verifier panic (it is not supposed to, but a decode panic was not supposed
 /// to happen either) is treated as `Neither`, which is the pre-existing path.
 ///
 /// Silent on purpose: several callers ask the same question about the same
 /// buffer (the pre-decode skip check, the decode itself, `verify`), so the
 /// routing note is printed once by `decode_flatbuffer`, not here.
 fn select_schema_by_verification(data: &[u8], schema_type: &str) -> SchemaChoice {
     match verify_schemas(data, schema_type, false) {
         None => SchemaChoice::Neither,
         Some((None, _)) => SchemaChoice::Cn,
         Some((Some(_), YostarVerdict::Verified)) => SchemaChoice::Yostar,
         Some((Some(_), _)) => SchemaChoice::Neither,
     }
 }

 /// The verification verdict for one gamedata buffer, for `unpacker verify`.
 ///
 /// `chosen` comes from `select_schema_by_verification` itself, so the report
 /// cannot drift from what `extract` does; the error strings come from the same
 /// emitted match arms, run once more with `full = true` so the Yostar column is
 /// filled in even when the CN schema verified.
 #[must_use]
 pub fn verify_table(data: &[u8], filename: &str) -> TableVerdict {
     let table = guess_root_type(filename);
     let (cn, yostar) = verify_schemas(data, table, true)
         .unwrap_or_else(|| (Some("verifier panicked".to_string()), YostarVerdict::NotRun));
     let routed_yostar = has_yostar_schema(table)
         && select_schema_by_verification(data, table) == SchemaChoice::Yostar;
     let chosen = if routed_yostar {
         "Yostar"
     } else if cn.is_none() {
         "CN"
     } else {
         "none"
     };
     TableVerdict {
         table,
         cn,
         yostar,
         chosen,
     }
 }

 /// `Some(table)` when NO schema verifies this buffer, i.e. the caller must SKIP
 /// it: no decode, no file written. `None` when it will decode (CN or Yostar),
 /// and also when `guess_root_type` has no schema for the filename at all —
 /// there is nothing to verify against there, and that path only ever produces
 /// the harmless `extract_strings` listing.
 ///
 /// Callers use this to skip BEFORE `export_text_asset` runs, because the
 /// fall-through inside it would otherwise write the raw payload as `.bytes`
 /// over a perfectly good `.json` from the previous extraction.
 #[must_use]
 pub fn unverified_table(data: &[u8], filename: &str) -> Option<&'static str> {
     let schema_type = guess_root_type(filename);
     if schema_type == "unknown" {
         return None;
     }
     match select_schema_by_verification(data, schema_type) {
         SchemaChoice::Cn | SchemaChoice::Yostar => None,
         SchemaChoice::Neither => Some(schema_type),
     }
 }

"#,
    );
}

fn generate_decode_dispatch(
    output_path: &Path,
    cn_structs: &HashMap<String, Vec<ParsedStruct>>,
    yostar_structs: &HashMap<String, Vec<ParsedStruct>>,
) -> anyhow::Result<()> {
    let mut out = String::new();

    out.push_str("//! Auto-generated `FlatBuffer` decode dispatch\n");
    out.push_str("//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs\n\n");
    out.push_str("use crate::schema::opts::verifier_opts;\n");
    out.push_str("use serde_json::{Value, json};\n");
    out.push_str("use std::panic::{self, AssertUnwindSafe};\n\n");

    out.push_str(
        r"/// Check if data is likely a `FlatBuffer`
 #[must_use]
 pub fn is_flatbuffer(data: &[u8]) -> bool {
     if data.len() < 8 {
         return false;
     }
     let root_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
     if root_offset >= data.len() || root_offset < 4 {
         return false;
     }
     if root_offset + 4 > data.len() {
         return false;
     }
     let vtable_offset = i32::from_le_bytes([
         data[root_offset], data[root_offset + 1],
         data[root_offset + 2], data[root_offset + 3],
     ]);
     let vtable_pos = (root_offset as i32 - vtable_offset) as usize;
     if vtable_pos >= data.len() || vtable_pos < 4 {
         return false;
     }
     let vtable_size = u16::from_le_bytes([data[vtable_pos], data[vtable_pos + 1]]) as usize;
     (4..1000).contains(&vtable_size) && vtable_pos + vtable_size <= data.len()
 }

 ",
    );

    // ---- guess_root_type ----
    // Derived from the store: a new upstream table needs no code edit here.
    let schema_to_module = build_schema_to_module_map(cn_structs);
    emit_guess_root_type(&mut out, &schema_to_module);

    // Sorted so the emitted file is byte-stable across runs (HashMap iteration
    // order is not).
    let mut yostar_modules: Vec<&String> = yostar_structs.keys().collect();
    yostar_modules.sort();
    let yostar_types: Vec<&str> = yostar_modules
        .iter()
        .copied()
        .filter_map(|module| {
            let structs = &yostar_structs[module];
            if structs.iter().any(|s| s.is_root) {
                schema_to_module
                    .iter()
                    .find(|(_, m)| m == module)
                    .map(|(st, _)| st.as_str())
            } else {
                None
            }
        })
        .collect();

    out.push_str("/// Check if a schema type has a Yostar variant\n");
    out.push_str("fn has_yostar_schema(schema_type: &str) -> bool {\n");
    if yostar_types.is_empty() {
        out.push_str("    let _ = schema_type;\n");
        out.push_str("    false\n");
    } else {
        out.push_str("    matches!(schema_type,\n        ");
        let yostar_arms: Vec<String> = yostar_types.iter().map(|t| format!("\"{t}\"")).collect();
        out.push_str(&yostar_arms.join(" | "));
        out.push_str("\n    )\n");
    }
    out.push_str("}\n\n");

    emit_schema_verification(&mut out, cn_structs, yostar_structs, &schema_to_module);

    out.push_str("/// Try decoding with Yostar-specific schemas\n");
    out.push_str(
        "fn decode_flatbuffer_yostar(data: &[u8], schema_type: &str) -> Result<Value, String> {\n",
    );

    // Check if there are any yostar root types
    let has_yostar_roots = yostar_structs
        .iter()
        .any(|(_, structs)| structs.iter().any(|s| s.root_fn_name.is_some()));

    if has_yostar_roots {
        out.push_str("    use crate::fb_json_macros::FlatBufferToJson;\n");
        out.push_str("    let data_clone = data.to_vec();\n");
        out.push_str("    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {\n");
        out.push_str("        let data = &data_clone;\n");
        out.push_str("        match schema_type {\n");

        for module in &yostar_modules {
            let structs = &yostar_structs[*module];
            for s in structs {
                if let Some(ref root_fn) = s.root_fn_name
                    && let Some((schema_type, _)) =
                        schema_to_module.iter().find(|(_, m)| m == *module)
                {
                    out.push_str(&format!(
                        "            \"{schema_type}\" => {{\n\
                             \x20               use crate::generated_fbs_yostar::{module}::{root_fn};\n\
                             \x20               let root = unsafe {{ {root_fn}(data) }};\n\
                             \x20               Ok(root.to_json())\n\
                             \x20           }}\n"
                    ));
                }
            }
        }

        out.push_str("            _ => Err(format!(\"No Yostar schema for {schema_type}\")),\n");
        out.push_str("        }\n");
        out.push_str("    }));\n");
        out.push_str("    match decode_result {\n");
        out.push_str("        Ok(Ok(value)) => {\n");
        out.push_str("            if value.as_object().is_some_and(serde_json::Map::is_empty) {\n");
        out.push_str("                Err(\"Yostar decode returned empty\".to_string())\n");
        out.push_str("            } else { Ok(value) }\n");
        out.push_str("        }\n");
        out.push_str("        Ok(Err(e)) => Err(e),\n");
        out.push_str("        Err(_) => Err(\"Yostar decode panic\".to_string()),\n");
        out.push_str("    }\n");
    } else {
        out.push_str("    let _ = data;\n");
        out.push_str("    Err(format!(\"No Yostar schema for {schema_type}\"))\n");
    }
    out.push_str("}\n\n");

    out.push_str("/// Decode `FlatBuffer` data to JSON using schema-based decoding\n");
    out.push_str(
        "pub fn decode_flatbuffer(data: &[u8], filename: &str) -> Result<Value, String> {\n",
    );
    out.push_str("    use crate::fb_json_macros::FlatBufferToJson;\n\n");
    out.push_str("    if !is_flatbuffer(data) {\n");
    out.push_str("        return Err(\"Data is not a valid FlatBuffer\".to_string());\n");
    out.push_str("    }\n\n");
    out.push_str("    let schema_type = guess_root_type(filename);\n\n");
    out.push_str(
        r#"    // DECODE ONLY WHAT VERIFIES. `guess_root_type` names a schema; the
     // buffer is verified against it (and against the Yostar variant, if the
     // table has one) BEFORE anything is decoded, and a buffer that verifies
     // under neither is skipped rather than decoded.
     //
     // WHY there is no unchecked fallback any more: `root_as_*_unchecked` on a
     // buffer the schema does not match has no termination guarantee. It reads
     // a scalar as a vector length and walks a multi-million-element phantom
     // vector, one caught panic per element. Measured twice: EN stage_table
     // 26-08-28 under the 2.7.71 CN schema never finished (>45 min on one
     // 5,022,624-byte buffer), and the committed March-2026 CN fixture under
     // the same schema burned 10 min at 97% CPU writing 3.7 GB of garbage
     // before it was killed (sampled: 129 of 129 frames in
     // stage_table_generated). Verification rejects both in microseconds.
     //
     // WHY skipping beats decoding anyway: a missing table is explicit and a
     // garbage table is not. The caller (`export_gamedata`) writes no file, so
     // the previous extraction's JSON stays on disk — the backend tolerates a
     // missing non-critical table and degrades a non-default server, where a
     // multi-GB garbage table takes the process out (a 10 GB RSS OOM kill on
     // the VPS on 2026-08-04) and a plausible-looking one is worse still.
     //
     // Verification also costs nothing to be wrong about in the safe direction:
     // it allocates no `Value` and serialises nothing, it only walks offsets
     // and vtables.
     if schema_type != "unknown" {
         match select_schema_by_verification(data, schema_type) {
             SchemaChoice::Yostar => {
                 if let Some((Some(e), _)) = verify_schemas(data, schema_type, false) {
                     eprintln!("schema: {schema_type} CN verify failed ({e}), Yostar verified");
                 }
                 return decode_flatbuffer_yostar(data, schema_type);
             }
             SchemaChoice::Neither => {
                 return Err(format!("No schema verifies {schema_type}"));
             }
             SchemaChoice::Cn => {}
         }
     }

 "#,
    );
    out.push_str("    let data_clone = data.to_vec();\n\n");
    out.push_str("    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {\n");
    out.push_str("        let data = &data_clone;\n");
    out.push_str("        match schema_type {\n");
    for (schema_type, module) in &schema_to_module {
        if let Some(structs) = cn_structs.get(module) {
            for s in structs {
                if let Some(ref root_fn) = s.root_fn_name {
                    out.push_str(&format!(
                        "            \"{schema_type}\" => {{\n\
                        \x20               use crate::generated_fbs::{module}::{root_fn};\n\
                        \x20               let root = unsafe {{ {root_fn}(data) }};\n\
                        \x20               Ok(root.to_json())\n\
                        \x20           }}\n"
                    ));
                    break; // Only one root per module
                }
            }
        }
    }

    out.push_str("            _ => Err(format!(\"Unknown schema type: {schema_type}\")),\n");
    out.push_str("        }\n");
    out.push_str("    }));\n\n");

    out.push_str(
        r#"    match decode_result {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => {
            // `guess_root_type` found no schema for this filename, so nothing
            // was verified and nothing was decoded. The string scavenger is the
            // only thing left, and it is bounds-checked and UTF-8-checked.
            if schema_type == "unknown" {
                let strings = extract_strings(data);
                if !strings.is_empty() {
                    return Ok(json!({ "type": "unknown", "strings": strings }));
                }
            }
            Err(format!("Decode failed for {schema_type}: {e}"))
        }
        // The buffer verified under this exact schema, so a panic here is a bug
        // in the emitted `to_json`, not a schema mismatch: report it, do not
        // silently retry under a schema that just failed verification.
        Err(_) => Err(format!("Decode panic for {schema_type}")),
    }
}

/// Extract strings from `FlatBuffer` (fallback for unknown types)
#[must_use]
pub fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut i = 0;
    while i + 4 < data.len() {
        let len = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        if len > 0
            && len < 1000
            && i + 4 + len <= data.len()
            && let Ok(s) = std::str::from_utf8(&data[i + 4..i + 4 + len])
            && s.len() >= 2
            && s.chars()
                .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace() || !c.is_ascii())
        {
            strings.push(s.to_string());
        }
        i += 1;
    }
    strings
}
"#,
    );

    fs::write(output_path, &out)?;
    println!("Generated {}", output_path.display());
    Ok(())
}

/// `(schema type, generated module)` for every CN schema that has a root
/// type, derived from the store: the schema type IS the store stem and the
/// module is `<stem>_generated`. Sorted, so the emitted file is byte-stable.
///
/// This replaced 57 hand-written pairs, which meant a new upstream table needed
/// a code edit before it could decode at all (`arkvent_table` on CN fell to the
/// unknown-strings path), and which named two types differently from their
/// schema (`level_data` for `prts___levels`, `language_data` for `init_text`).
fn build_schema_to_module_map(
    cn_structs: &HashMap<String, Vec<ParsedStruct>>,
) -> Vec<(String, String)> {
    let mut map: Vec<(String, String)> = cn_structs
        .keys()
        .filter(|module| root_fn_for(cn_structs, module).is_some())
        .filter_map(|module| {
            module
                .strip_suffix("_generated")
                .map(|stem| (stem.to_string(), module.clone()))
        })
        .collect();
    map.sort();
    map
}

/// Filename substrings that name a table whose stem they do not contain,
/// checked after the stems and in this order. Carried over from the hand map.
const ROUTING_ALIASES: &[(&str, &str)] = &[
    ("char_table", "character_table"),
    ("language_data", "init_text"),
];

/// Emits `guess_root_type`: the LONGEST store stem contained in the filename
/// wins, then `ROUTING_ALIASES`, then the `level_` prefix as the last fallback
/// for `prts___levels`.
///
/// Longest-first is what the hand-ordered chain got wrong for
/// `legion_mode_buff_table`: its `buff_table` arm came first, so the table was
/// verified and decoded under the `buff_table` schema. `level_script_table` is
/// a stem, so it wins over the `level_` prefix without an ordering rule.
fn emit_guess_root_type(out: &mut String, schema_to_module: &[(String, String)]) {
    let mut stems: Vec<&str> = schema_to_module.iter().map(|(s, _)| s.as_str()).collect();
    stems.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    out.push_str(
        "/// Guess the root type (the store stem) from a gamedata filename.\n\
         ///\n\
         /// Generated from the schema store: the longest stem contained in the\n\
         /// filename wins, then a short alias table, then the `level_` prefix.\n\
         fn guess_root_type(filename: &str) -> &'static str {\n\
         \x20   const STEMS: &[&str] = &[\n",
    );
    for stem in &stems {
        out.push_str(&format!("        \"{stem}\",\n"));
    }
    out.push_str("    ];\n    const ALIASES: &[(&str, &str)] = &[\n");
    for (needle, stem) in ROUTING_ALIASES {
        if stems.contains(stem) {
            out.push_str(&format!("        (\"{needle}\", \"{stem}\"),\n"));
        }
    }
    out.push_str("    ];\n");
    out.push_str(
        "    let lower = filename.to_lowercase();\n\
         \x20   if let Some(stem) = STEMS.iter().find(|s| lower.contains(*s)) {\n\
         \x20       return stem;\n\
         \x20   }\n\
         \x20   if let Some((_, stem)) = ALIASES.iter().find(|(n, _)| lower.contains(n)) {\n\
         \x20       return stem;\n\
         \x20   }\n",
    );
    if stems.contains(&"prts___levels") {
        out.push_str(
            "    // Level files carry no table name; this must stay the LAST rule\n\
             \x20   // (`level_script_table` is matched above as a stem).\n\
             \x20   if lower.starts_with(\"level_\") {\n\
             \x20       return \"prts___levels\";\n\
             \x20   }\n",
        );
    }
    out.push_str("    \"unknown\"\n}\n\n");
}

fn main() -> anyhow::Result<()> {
    let script_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = script_dir.join("src");

    // The schema store is the only input. `unpacker schema-sync` fills it;
    // this generator never fetches anything.
    let store_dir = unpacker::schema::store::default_store_dir();
    let cn_fbs_dir = store_dir.join("cn");
    let yostar_fbs_dir = store_dir.join("yostar");
    if !cn_fbs_dir.is_dir() {
        bail!(
            "schema store not found: {} (run `unpacker schema-sync`)",
            cn_fbs_dir.display()
        );
    }
    println!("CN schemas:     {}", cn_fbs_dir.display());
    println!("Yostar schemas: {}", yostar_fbs_dir.display());

    let cn_output = src_dir.join("generated_fbs");
    let yostar_output = src_dir.join("generated_fbs_yostar");
    fs::create_dir_all(&cn_output)?;
    fs::create_dir_all(&yostar_output)?;

    run_flatc_all(&cn_fbs_dir, &cn_output)?;

    // The Yostar variant set is COMPUTED, not curated: a table gets one exactly
    // when its Yostar .fbs differs from the CN one. See `compute_yostar_schemas`.
    // The output dir is cleared first so a table that stops differing stops
    // being compiled in.
    let yostar_schemas = if yostar_fbs_dir.is_dir() {
        compute_yostar_schemas(&cn_fbs_dir, &yostar_fbs_dir)?
    } else {
        Vec::new()
    };
    clear_generated_dir(&yostar_output)?;
    for name in &yostar_schemas {
        let fbs = yostar_fbs_dir.join(format!("{name}.fbs"));
        if fbs.exists() {
            run_flatc(&fbs, &yostar_output)?;
        }
    }

    println!("=== Stripping serde impls ===");
    strip_serialize_impls(&cn_output)?;
    strip_serialize_impls(&yostar_output)?;

    println!("=== Generating mod.rs ===");
    generate_mod_rs(&cn_output)?;
    generate_mod_rs(&yostar_output)?;

    println!("=== Parsing CN structs ===");
    let cn_structs = parse_all_generated(&cn_output)?;
    println!("=== Parsing CN enums ===");
    let cn_enums = parse_all_enums(&cn_output)?;
    println!("=== Generating fb_json_auto.rs ===");
    generate_fb_json_auto(
        &src_dir.join("fb_json_auto.rs"),
        &cn_structs,
        &cn_enums,
        "generated_fbs",
    )?;

    println!("=== Parsing Yostar structs ===");
    let yostar_structs = parse_all_generated(&yostar_output)?;
    println!("=== Parsing Yostar enums ===");
    let yostar_enums = parse_all_enums(&yostar_output)?;
    println!("=== Generating fb_json_auto_yostar.rs ===");
    generate_fb_json_auto(
        &src_dir.join("fb_json_auto_yostar.rs"),
        &yostar_structs,
        &yostar_enums,
        "generated_fbs_yostar",
    )?;

    println!("=== Generating flatbuffers_decode.rs ===");
    generate_decode_dispatch(
        &src_dir.join("flatbuffers_decode.rs"),
        &cn_structs,
        &yostar_structs,
    )?;

    println!("Done! Run `cargo build` to compile.");
    Ok(())
}
