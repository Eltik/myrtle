use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn fetch_cn_schemas(script_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let fbs_dir = script_dir
        .parent()
        .ok_or("no parent dir")?
        .join("OpenArknightsFBS/FBS");

    if !fbs_dir.exists() {
        // Try cloning it
        let parent = fbs_dir.parent().unwrap().parent().unwrap();
        println!("Cloning OpenArknightsFBS...");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/MooncellWiki/OpenArknightsFBS.git",
                fbs_dir.parent().unwrap().to_str().unwrap(),
            ])
            .current_dir(parent)
            .status()?;
        if !status.success() {
            return Err("Failed to clone OpenArknightsFBS".into());
        }
    } else {
        println!("Updating OpenArknightsFBS...");
        let _ = Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(fbs_dir.parent().unwrap())
            .status();
    }

    if !fbs_dir.exists() {
        return Err(format!("FBS directory not found: {}", fbs_dir.display()).into());
    }

    patch_schemas(&fbs_dir);

    println!("CN schemas: {}", fbs_dir.display());
    Ok(fbs_dir)
}

/// Apply known fixes to upstream FBS schemas before running flatc.
///
/// The OpenArknightsFBS repo is community-maintained and occasionally has
/// field ordering issues that cause VTable misalignment. FlatBuffers assigns
/// VTable slots by declaration order, so a field inserted in the middle
/// (rather than appended) breaks all subsequent offsets.
///
/// Each patch is documented with the symptom and root cause.
fn patch_schemas(fbs_dir: &Path) {
    let patches: &[(&str, &str, &str)] = &[
        // roguelike_topic_table.fbs: `relicTipsData` was inserted before `activity`
        // in clz_Torappu_RoguelikeTopicDetail, but the binary has `activity` at
        // slot 51 (matching CN-gamedata field order). Moving `relicTipsData` after
        // `activity` fixes the VTable alignment and allows Details to decode.
        (
            "roguelike_topic_table.fbs",
            // old: relicTipsData before activity
            "    rollNodeData: [dict__string__clz_Torappu_RoguelikeRollNodeData]; \n\
             \x20   relicTipsData: [dict__string__clz_Torappu_RoguelikeRelicTipsData]; \n\
             \x20   activity: clz_Torappu_RoguelikeActivityData; \n\
             }",
            // new: activity before relicTipsData
            "    rollNodeData: [dict__string__clz_Torappu_RoguelikeRollNodeData];\n\
             \x20   activity: clz_Torappu_RoguelikeActivityData;\n\
             \x20   relicTipsData: [dict__string__clz_Torappu_RoguelikeRelicTipsData];\n\
             }",
        ),
        // skin_table.fbs: upstream commit 4975a03 "Update 2.7.21" (Apr 7 2026)
        // inserted `spAvatarId` and `spPortraitId` into the middle of
        // clz_Torappu_CharSkinData (between avatarId/portraitId and
        // portraitId/dynPortraitId). The actual CN binary was serialized with
        // the pre-4975a03 schema and does NOT contain these fields. The inserted
        // fields shift all subsequent VTable slots by +4, so every CharSkin
        // entry panics when the decoder follows garbage offsets, and the
        // filter_map safety net drops all ~5669 skins → CharSkins: [].
        //
        // Fix: remove the two inserted fields from the schema. The backend
        // never reads `spAvatarId` / `spPortraitId` (see backend/src/core/
        // gamedata/types/skin.rs Skin struct — only skin_id, char_id, avatar_id,
        // portrait_id, battle_skin, display_skin are consumed), so this is a
        // pure alignment fix with zero backend impact. When the binary later
        // ships these fields, upstream's schema can be re-enabled.
        (
            "skin_table.fbs",
            // old: spAvatarId and spPortraitId inserted mid-struct
            "    avatarId: string; \n\
             \x20   spAvatarId: string; \n\
             \x20   portraitId: string; \n\
             \x20   spPortraitId: string; \n\
             \x20   dynPortraitId: string; ",
            // new: inserted fields removed, original order restored
            "    avatarId: string;\n\
             \x20   portraitId: string;\n\
             \x20   dynPortraitId: string; ",
        ),
    ];

    for (filename, old, new) in patches {
        let path = fbs_dir.join(filename);
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        // Normalize CRLF → LF so patches match on Windows (git autocrlf)
        let content = content.replace("\r\n", "\n");
        if content.contains(old) {
            let patched = content.replace(old, new);
            if fs::write(&path, patched).is_ok() {
                println!("  patched {filename}: reordered fields for VTable alignment");
            }
        }
    }
}

fn fetch_yostar_schemas() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = PathBuf::from("/tmp/ArknightsFlatbuffers");
    let fbs_dir = repo_dir.join("yostar");

    if !repo_dir.exists() {
        println!("Cloning ArknightsFlatbuffers...");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/ArknightsAssets/ArknightsFlatbuffers.git",
                repo_dir.to_str().unwrap(),
            ])
            .status()?;
        if !status.success() {
            return Err("Failed to clone ArknightsFlatbuffers".into());
        }
    } else {
        println!("Updating ArknightsFlatbuffers...");
        let _ = Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(&repo_dir)
            .status();
    }

    println!("Yostar schemas: {}", fbs_dir.display());
    Ok(fbs_dir)
}

fn run_flatc(fbs_path: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let name = fbs_path.file_stem().unwrap().to_str().unwrap();
    println!("  flatc: {name}");

    let status = Command::new("flatc")
        .args([
            "--rust",
            "--gen-object-api",
            "--rust-serialize",
            "-o",
            output_dir.to_str().unwrap(),
            fbs_path.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        eprintln!("  warning: flatc failed for {name}");
    }
    Ok(())
}

fn run_flatc_all(fbs_dir: &Path, output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: Vec<_> = fs::read_dir(fbs_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fbs"))
        .collect();
    entries.sort_by_key(|e| e.path());

    println!("Running flatc on {} schemas...", entries.len());
    for entry in &entries {
        run_flatc(&entry.path(), output_dir)?;
    }
    Ok(())
}

fn strip_serialize_impls(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let serde_import_re = Regex::new(r"^use (?:self::)?serde(?:::ser)?::\{.*?\};$")?;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
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

            if !skip {
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

                output.push_str(line);
                output.push('\n');
            } else {
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

fn generate_mod_rs(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

fn parse_all_generated(
    dir: &Path,
) -> Result<HashMap<String, Vec<ParsedStruct>>, Box<dyn std::error::Error>> {
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

fn parse_all_enums(dir: &Path) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
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
) -> Result<(), Box<dyn std::error::Error>> {
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
            emit_struct_impl(&mut out, s, module, s.is_root);
        }
    }

    fs::write(output_path, &out)?;
    let struct_count: usize = structs.values().map(|v| v.len()).sum();
    let enum_count: usize = enums.values().map(|v| v.len()).sum();
    println!(
        "Generated {} ({struct_count} structs, {enum_count} enums)",
        output_path.display()
    );
    Ok(())
}

fn emit_struct_impl(out: &mut String, s: &ParsedStruct, module: &str, is_root: bool) {
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
            if is_root {
                emit_field_safe(out, field, &pascal);
            } else {
                emit_field_direct(out, field, &pascal);
            }
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
            out.push_str("        if let Some(k) = self.key() {\n");
            out.push_str("            map.insert(\"key\".to_string(), json!(k));\n");
            out.push_str("        }\n");
        } else if kf.is_enum {
            out.push_str(
                "        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {\n",
            );
            out.push_str("            map.insert(\"key\".to_string(), k.to_json_value());\n");
            out.push_str("        }\n");
        } else {
            out.push_str(
                "        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {\n",
            );
            out.push_str("            map.insert(\"key\".to_string(), json!(k));\n");
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
                    out.push_str("        if let Some(vec) = self.value() {\n");
                    out.push_str(
                        "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                    );
                    out.push_str("            let arr: Vec<Value> = vec.iter().map(|v| json!(v)).collect();\n");
                    out.push_str("            map.insert(\"value\".to_string(), json!(arr));\n");
                    out.push_str("        }\n");
                }
            } else {
                out.push_str("        if let Some(v) = self.value() {\n");
                out.push_str("            map.insert(\"value\".to_string(), json!(v));\n");
                out.push_str("        }\n");
            }
        } else if vf.is_nested {
            out.push_str("        map.insert(\"value\".to_string(), self.value().to_json());\n");
        } else if vf.is_enum {
            out.push_str(
                "        map.insert(\"value\".to_string(), self.value().to_json_value());\n",
            );
        } else {
            out.push_str("        map.insert(\"value\".to_string(), json!(self.value()));\n");
        }
    }

    out.push_str("        Value::Object(map)\n");
}

fn emit_field_direct(out: &mut String, field: &Field, pascal_name: &str) {
    if field.is_vector {
        match field.element_type.as_deref() {
            Some("string") => {
                out.push_str(&format!(
                    "        if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("            let arr: Vec<Value> = (0..vec.len()).map(|i| json!(vec.get(i))).collect();\n");
                out.push_str(&format!(
                    "            map.insert(\"{pascal_name}\".to_string(), json!(arr));\n"
                ));
                out.push_str("        }\n");
            }
            Some("nested") => {
                out.push_str(&format!(
                    "        if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("            let arr: Vec<Value> = (0..vec.len()).map(|i| vec.get(i).to_json()).collect();\n");
                out.push_str(&format!(
                    "            map.insert(\"{pascal_name}\".to_string(), json!(arr));\n"
                ));
                out.push_str("        }\n");
            }
            Some("enum") => {
                out.push_str(&format!(
                    "        if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str("            let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();\n");
                out.push_str(&format!(
                    "            map.insert(\"{pascal_name}\".to_string(), json!(arr));\n"
                ));
                out.push_str("        }\n");
            }
            _ => {
                // scalar vector
                out.push_str(&format!(
                    "        if let Some(vec) = self.{}() {{\n",
                    field.name
                ));
                out.push_str(
                    "            assert!(vec.len() <= 10_000_000, \"FB vector too large\");\n",
                );
                out.push_str(
                    "            let arr: Vec<Value> = vec.iter().map(|v| json!(v)).collect();\n",
                );
                out.push_str(&format!(
                    "            map.insert(\"{pascal_name}\".to_string(), json!(arr));\n"
                ));
                out.push_str("        }\n");
            }
        }
    } else if field.is_enum {
        if field.is_option {
            out.push_str(&format!(
                "        if let Some(e) = self.{}() {{\n",
                field.name
            ));
            out.push_str(&format!(
                "            map.insert(\"{pascal_name}\".to_string(), e.to_json_value());\n"
            ));
            out.push_str("        }\n");
        } else {
            out.push_str(&format!(
                "        map.insert(\"{pascal_name}\".to_string(), self.{}().to_json_value());\n",
                field.name
            ));
        }
    } else if field.is_nested {
        out.push_str(&format!(
            "        if let Some(nested) = self.{}() {{\n",
            field.name
        ));
        out.push_str(&format!(
            "            map.insert(\"{pascal_name}\".to_string(), nested.to_json());\n"
        ));
        out.push_str("        }\n");
    } else if field.is_option {
        out.push_str(&format!(
            "        if let Some(v) = self.{}() {{\n",
            field.name
        ));
        out.push_str(&format!(
            "            map.insert(\"{pascal_name}\".to_string(), json!(v));\n"
        ));
        out.push_str("        }\n");
    } else {
        // Plain scalar
        out.push_str(&format!(
            "        map.insert(\"{pascal_name}\".to_string(), json!(self.{}()));\n",
            field.name
        ));
    }
}

fn emit_field_safe(out: &mut String, field: &Field, pascal_name: &str) {
    // Wrap each field in catch_unwind for root structs
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
                out.push_str("                let arr: Vec<Value> = (0..vec.len()).map(|i| json!(vec.get(i))).collect();\n");
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
        out.push_str(&format!(
            "            if let Some(v) = self.{}() {{\n",
            field.name
        ));
        out.push_str(&format!(
            "                return Some((\"{pascal_name}\".to_string(), json!(v)));\n"
        ));
        out.push_str("            }\n");
    } else {
        out.push_str(&format!(
            "            return Some((\"{pascal_name}\".to_string(), json!(self.{}())));\n",
            field.name
        ));
    }

    out.push_str("            #[allow(unreachable_code)]\n");
    out.push_str("            None\n");
    out.push_str("        })) {\n");
    out.push_str("            map.insert(k, v);\n");
    out.push_str("        }\n");
}

fn pascal_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn generate_decode_dispatch(
    output_path: &Path,
    cn_structs: &HashMap<String, Vec<ParsedStruct>>,
    yostar_structs: &HashMap<String, Vec<ParsedStruct>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = String::new();

    out.push_str("//! Auto-generated FlatBuffer decode dispatch\n");
    out.push_str("//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs\n\n");
    out.push_str("use serde_json::{json, Value};\n");
    out.push_str("use std::panic::{self, AssertUnwindSafe};\n\n");

    out.push_str(
        r#"/// Check if data is likely a FlatBuffer
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

 "#,
    );

    // ---- guess_root_type ----
    // This is the hand-curated mapping. We embed it directly.
    out.push_str(
        r#"/// Guess the root type from filename
 fn guess_root_type(filename: &str) -> &'static str {
     let lower = filename.to_lowercase();

     if lower.starts_with("level_") {
         "level_data"
     } else if lower.contains("enemy_database") {
         "enemy_database"
     } else if lower.contains("enemy_handbook") {
         "enemy_handbook_table"
     } else if lower.contains("character_table") || lower.contains("char_table") {
         "character_table"
     } else if lower.contains("char_master") {
         "char_master_table"
     } else if lower.contains("char_meta") {
         "char_meta_table"
     } else if lower.contains("char_patch") {
         "char_patch_table"
     } else if lower.contains("charword") {
         "charword_table"
     } else if lower.contains("skill_table") {
         "skill_table"
     } else if lower.contains("item_table") {
         "item_table"
     } else if lower.contains("gacha_table") {
         "gacha_table"
     } else if lower.contains("skin_table") {
         "skin_table"
     } else if lower.contains("handbook_info") {
         "handbook_info_table"
     } else if lower.contains("handbook_team") {
         "handbook_team_table"
     } else if lower.contains("uniequip_table") {
         "uniequip_table"
     } else if lower.contains("battle_equip") {
         "battle_equip_table"
     } else if lower.contains("stage_table") {
         "stage_table"
     } else if lower.contains("activity_table") {
         "activity_table"
     } else if lower.contains("audio_data") {
         "audio_data"
     } else if lower.contains("building_local") {
         "building_local_data"
     } else if lower.contains("building_data") {
         "building_data"
     } else if lower.contains("campaign_table") {
         "campaign_table"
     } else if lower.contains("chapter_table") {
         "chapter_table"
     } else if lower.contains("charm_table") {
         "charm_table"
     } else if lower.contains("checkin_table") {
         "checkin_table"
     } else if lower.contains("climb_tower") {
         "climb_tower_table"
     } else if lower.contains("clue_data") {
         "clue_data"
     } else if lower.contains("crisis_v2") {
         "crisis_v2_table"
     } else if lower.contains("crisis_table") {
         "crisis_table"
     } else if lower.contains("display_meta") {
         "display_meta_table"
     } else if lower.contains("favor_table") {
         "favor_table"
     } else if lower.contains("gamedata_const") {
         "gamedata_const"
     } else if lower.contains("hotupdate_meta") {
         "hotupdate_meta_table"
     } else if lower.contains("medal_table") {
         "medal_table"
     } else if lower.contains("meta_ui") {
         "meta_ui_table"
     } else if lower.contains("mission_table") {
         "mission_table"
     } else if lower.contains("open_server") {
         "open_server_table"
     } else if lower.contains("retro_table") {
         "retro_table"
     } else if lower.contains("roguelike") {
         "roguelike_topic_table"
     } else if lower.contains("sandbox_perm") {
         "sandbox_perm_table"
     } else if lower.contains("sandbox_table") {
         "sandbox_table"
     } else if lower.contains("shop_client") {
         "shop_client_table"
     } else if lower.contains("special_operator") {
         "special_operator_table"
     } else if lower.contains("story_review_meta") {
         "story_review_meta_table"
     } else if lower.contains("story_review") {
         "story_review_table"
     } else if lower.contains("story_table") {
         "story_table"
     } else if lower.contains("tip_table") {
         "tip_table"
     } else if lower.contains("zone_table") {
         "zone_table"
     } else if lower.contains("ep_breakbuff") {
         "ep_breakbuff_table"
     } else if lower.contains("buff_table") {
         "buff_table"
     } else if lower.contains("cooperate") {
         "cooperate_battle_table"
     } else if lower.contains("init_text") || lower.contains("main_text") {
         "language_data"
     } else if lower.contains("extra_battlelog") {
         "extra_battlelog_table"
     } else if lower.contains("replicate") {
         "replicate_table"
     } else if lower.contains("legion_mode") {
         "legion_mode_buff_table"
     } else if lower.contains("token_table") {
         "token_table"
     } else {
         "unknown"
     }
 }

 "#,
    );

    let schema_to_module = build_schema_to_module_map();

    let yostar_types: Vec<&str> = yostar_structs
        .keys()
        .filter_map(|module| {
            let structs = &yostar_structs[module];
            if structs.iter().any(|s| s.is_root) {
                schema_to_module
                    .iter()
                    .find(|(_, m)| *m == module)
                    .map(|(st, _)| *st)
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

    out.push_str("/// Try decoding with Yostar-specific schemas\n");
    out.push_str(
        "fn decode_flatbuffer_yostar(data: &[u8], schema_type: &str) -> Result<Value, String> {\n",
    );

    // Check if there are any yostar root types
    let has_yostar_roots = yostar_structs
        .iter()
        .any(|(_, structs)| structs.iter().any(|s| s.root_fn_name.is_some()));

    if !has_yostar_roots {
        out.push_str("    let _ = data;\n");
        out.push_str("    Err(format!(\"No Yostar schema for {}\", schema_type))\n");
    } else {
        out.push_str("    use crate::fb_json_macros::FlatBufferToJson;\n");
        out.push_str("    let data_clone = data.to_vec();\n");
        out.push_str("    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {\n");
        out.push_str("        let data = &data_clone;\n");
        out.push_str("        match schema_type {\n");

        for (module, structs) in yostar_structs {
            for s in structs {
                if let Some(ref root_fn) = s.root_fn_name
                    && let Some((schema_type, _)) =
                        schema_to_module.iter().find(|(_, m)| *m == module)
                {
                    out.push_str(&format!(
                        "            \"{schema_type}\" => {{\n\
                             \x20               use crate::generated_fbs_yostar::{module}::*;\n\
                             \x20               let root = unsafe {{ {root_fn}(data) }};\n\
                             \x20               Ok(root.to_json())\n\
                             \x20           }}\n"
                    ));
                }
            }
        }

        out.push_str("            _ => Err(format!(\"No Yostar schema for {}\", schema_type)),\n");
        out.push_str("        }\n");
        out.push_str("    }));\n");
        out.push_str("    match decode_result {\n");
        out.push_str("        Ok(Ok(value)) => {\n");
        out.push_str("            if value.as_object().map_or(false, |o| o.is_empty()) {\n");
        out.push_str("                Err(\"Yostar decode returned empty\".to_string())\n");
        out.push_str("            } else { Ok(value) }\n");
        out.push_str("        }\n");
        out.push_str("        Ok(Err(e)) => Err(e),\n");
        out.push_str("        Err(_) => Err(\"Yostar decode panic\".to_string()),\n");
        out.push_str("    }\n");
    }
    out.push_str("}\n\n");

    out.push_str("/// Decode FlatBuffer data to JSON using schema-based decoding\n");
    out.push_str(
        "pub fn decode_flatbuffer(data: &[u8], filename: &str) -> Result<Value, String> {\n",
    );
    out.push_str("    use crate::fb_json_macros::FlatBufferToJson;\n\n");
    out.push_str("    if !is_flatbuffer(data) {\n");
    out.push_str("        return Err(\"Data is not a valid FlatBuffer\".to_string());\n");
    out.push_str("    }\n\n");
    out.push_str("    let schema_type = guess_root_type(filename);\n");
    out.push_str("    let data_clone = data.to_vec();\n\n");
    out.push_str("    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {\n");
    out.push_str("        let data = &data_clone;\n");
    out.push_str("        match schema_type {\n");
    for (schema_type, module) in &schema_to_module {
        if let Some(structs) = cn_structs.get(*module) {
            for s in structs {
                if let Some(ref root_fn) = s.root_fn_name {
                    out.push_str(&format!(
                        "            \"{schema_type}\" => {{\n\
                        \x20               use crate::generated_fbs::{module}::*;\n\
                        \x20               let root = unsafe {{ {root_fn}(data) }};\n\
                        \x20               Ok(root.to_json())\n\
                        \x20           }}\n"
                    ));
                    break; // Only one root per module
                }
            }
        }
    }

    out.push_str("            _ => Err(format!(\"Unknown schema type: {}\", schema_type)),\n");
    out.push_str("        }\n");
    out.push_str("    }));\n\n");

    out.push_str(r#"    match decode_result {
                     Ok(Ok(value)) => {
                         // A result is "useless" if either:
                         //   (a) the top-level object has no keys at all, or
                         //   (b) the root collection is present but empty because every
                         //       element was dropped by the filter_map safety net.
                         // Case (b) happens when the CN schema has fields the binary
                         // doesn't (e.g., validModeIndices in EquipTalentData for the
                         // Apr 2026 CN binary) and every element panics on decode.
                         // Without this check the Yostar fallback never fires.
                         let is_content_empty = match schema_type {
                             "battle_equip_table" => value
                                 .get("Equips")
                                 .and_then(|v| v.as_array())
                                 .map_or(false, |a| a.is_empty()),
                             _ => false,
                         };
                         if value.as_object().map_or(false, |o| o.is_empty()) || is_content_empty {
                             if has_yostar_schema(schema_type) {
                                 if let Ok(v) = decode_flatbuffer_yostar(data, schema_type) {
                                     return Ok(v);
                                 }
                             }
                             Err(format!("Schema mismatch for {} (empty result)", schema_type))
                         } else {
                             Ok(value)
                         }
                     }
                     Ok(Err(e)) => {
                         if has_yostar_schema(schema_type) {
                             if let Ok(v) = decode_flatbuffer_yostar(data, schema_type) {
                                 return Ok(v);
                             }
                         }
                         if schema_type == "unknown" {
                             let strings = extract_strings(data);
                             if !strings.is_empty() {
                                 return Ok(json!({ "type": "unknown", "strings": strings }));
                             }
                         }
                         Err(format!("Decode failed for {}: {}", schema_type, e))
                     }
                     Err(_) => {
                         if has_yostar_schema(schema_type) {
                             if let Ok(v) = decode_flatbuffer_yostar(data, schema_type) {
                                 return Ok(v);
                             }
                         }
                         Err(format!("Decode panic for {}", schema_type))
                     }
                 }
             }

             /// Extract strings from FlatBuffer (fallback for unknown types)
             pub fn extract_strings(data: &[u8]) -> Vec<String> {
                 let mut strings = Vec::new();
                 let mut i = 0;
                 while i + 4 < data.len() {
                     let len = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]) as usize;
                     if len > 0 && len < 1000 && i + 4 + len <= data.len() {
                         if let Ok(s) = std::str::from_utf8(&data[i+4..i+4+len]) {
                             if s.len() >= 2 && s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace() || !c.is_ascii()) {
                                 strings.push(s.to_string());
                             }
                         }
                     }
                     i += 1;
                 }
                 strings
             }
             "#);

    fs::write(output_path, &out)?;
    println!("Generated {}", output_path.display());
    Ok(())
}

fn build_schema_to_module_map() -> Vec<(&'static str, &'static str)> {
    vec![
        ("character_table", "character_table_generated"),
        ("char_master_table", "char_master_table_generated"),
        ("char_meta_table", "char_meta_table_generated"),
        ("char_patch_table", "char_patch_table_generated"),
        ("charword_table", "charword_table_generated"),
        ("skill_table", "skill_table_generated"),
        ("enemy_database", "enemy_database_generated"),
        ("enemy_handbook_table", "enemy_handbook_table_generated"),
        ("item_table", "item_table_generated"),
        ("skin_table", "skin_table_generated"),
        ("uniequip_table", "uniequip_table_generated"),
        ("battle_equip_table", "battle_equip_table_generated"),
        ("handbook_info_table", "handbook_info_table_generated"),
        ("handbook_team_table", "handbook_team_table_generated"),
        ("gacha_table", "gacha_table_generated"),
        ("stage_table", "stage_table_generated"),
        ("activity_table", "activity_table_generated"),
        ("audio_data", "audio_data_generated"),
        ("building_data", "building_data_generated"),
        ("building_local_data", "building_local_data_generated"),
        ("campaign_table", "campaign_table_generated"),
        ("chapter_table", "chapter_table_generated"),
        ("charm_table", "charm_table_generated"),
        ("checkin_table", "checkin_table_generated"),
        ("climb_tower_table", "climb_tower_table_generated"),
        ("clue_data", "clue_data_generated"),
        ("crisis_table", "crisis_table_generated"),
        ("crisis_v2_table", "crisis_v2_table_generated"),
        ("display_meta_table", "display_meta_table_generated"),
        ("favor_table", "favor_table_generated"),
        ("gamedata_const", "gamedata_const_generated"),
        ("hotupdate_meta_table", "hotupdate_meta_table_generated"),
        ("medal_table", "medal_table_generated"),
        ("meta_ui_table", "meta_ui_table_generated"),
        ("mission_table", "mission_table_generated"),
        ("open_server_table", "open_server_table_generated"),
        ("retro_table", "retro_table_generated"),
        ("roguelike_topic_table", "roguelike_topic_table_generated"),
        ("sandbox_perm_table", "sandbox_perm_table_generated"),
        ("sandbox_table", "sandbox_table_generated"),
        ("shop_client_table", "shop_client_table_generated"),
        ("special_operator_table", "special_operator_table_generated"),
        (
            "story_review_meta_table",
            "story_review_meta_table_generated",
        ),
        ("story_review_table", "story_review_table_generated"),
        ("story_table", "story_table_generated"),
        ("tip_table", "tip_table_generated"),
        ("zone_table", "zone_table_generated"),
        ("buff_table", "buff_table_generated"),
        ("cooperate_battle_table", "cooperate_battle_table_generated"),
        ("language_data", "init_text_generated"),
        ("ep_breakbuff_table", "ep_breakbuff_table_generated"),
        ("extra_battlelog_table", "extra_battlelog_table_generated"),
        ("replicate_table", "replicate_table_generated"),
        ("legion_mode_buff_table", "legion_mode_buff_table_generated"),
        ("token_table", "token_table_generated"),
        ("level_data", "prts___levels_generated"),
    ]
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = script_dir.join("src");

    let cn_fbs_dir = fetch_cn_schemas(&script_dir)?;
    let yostar_fbs_dir = fetch_yostar_schemas()?;

    let cn_output = src_dir.join("generated_fbs");
    let yostar_output = src_dir.join("generated_fbs_yostar");
    fs::create_dir_all(&cn_output)?;
    fs::create_dir_all(&yostar_output)?;

    run_flatc_all(&cn_fbs_dir, &cn_output)?;
    let yostar_schemas = [
        "character_table",
        "battle_equip_table",
        "token_table",
        "ep_breakbuff_table",
    ];
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
