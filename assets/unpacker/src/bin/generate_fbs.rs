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

    if fbs_dir.exists() {
        println!("Updating OpenArknightsFBS...");
        let _ = Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(fbs_dir.parent().unwrap())
            .status();
    } else {
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
/// The `OpenArknightsFBS` repo is community-maintained and occasionally has
/// field ordering issues that cause `VTable` misalignment. `FlatBuffers` assigns
/// `VTable` slots by declaration order, so a field inserted in the middle
/// (rather than appended) breaks all subsequent offsets.
///
/// AS OF CN 2.7.71 (upstream `b24069f`) THE ARRAY IS EMPTY: every patch this
/// function ever carried was REFUTED by running the `FlatBuffers` verifier over
/// the live binary. In each case pristine upstream verifies and the patched
/// schema does not — the patches were written against older binaries and each
/// one outlived the mismatch it was correcting, then started corrupting the
/// decode itself. The per-patch records below keep the measurement that
/// retired each one.
///
/// BEFORE ADDING A NEW PATCH, two measurements are required. First, the CN
/// `root_as_*_with_opts` verifier must FAIL on the live binary with pristine
/// upstream. Second, it must PASS with the patch applied. A patch that cannot
/// show both is not fixing a misalignment — see `select_schema_by_verification`
/// in the generated decoder for the check.
///
/// Note that a verifier failure alone does not prove the schema is wrong: the
/// verifier is stricter about alignment than the game's writer, and CN
/// `activity_table` and `roguelike_topic_table` both fail "unaligned" on schemas
/// whose decode is known-good.
fn patch_schemas(fbs_dir: &Path) {
    // roguelike_topic_table.fbs: NO patch as of CN 2.7.71 (upstream commit
    // b24069f). Upstream declares the tail of clz_Torappu_RoguelikeTopicDetail
    // as `rollNodeData, relicTipsData, legacyItems, activity`, and the live
    // binary agrees: decoded against that order every one of the four fields
    // holds data of its own shape (Activity populated for all 6 topics,
    // LegacyItems = real LegacyItemData, rogue_6 CharBuffData 6.7 KB with 49/49
    // readable blackboard keys, whole table 18.9 MB).
    //
    // A prior patch here swapped `relicTipsData` and `activity`. That was
    // right for the 2.7.41 binary, which lacked `legacyItems`; against 2.7.71
    // the same swap reads the legacy-item vector through the RelicTips schema
    // and leaves Activity empty, and the pinned pre-rebase schema decodes
    // rogue_6 CharBuffData into 1.6 GB of vtable garbage (a 1.8 GB table that
    // took the backend to a 5.3 GiB load peak). The rebased submodule carries
    // no roguelike change, so there is nothing for a patch to match.
    //
    // skin_table.fbs: NO patch needed as of CN 2.7.41 (upstream commit
    // 6121fa9). Upstream correctly ships spAvatarId (after avatarId) and
    // spPortraitId (after portraitId) in clz_Torappu_CharSkinData — 20
    // vtable fields, matching the live binary.
    //
    // A prior patch here REMOVED those two fields. That was correct at
    // 2.7.21, when the binary lacked them but upstream's schema had them;
    // it became WRONG once the binary started shipping them (as that patch's
    // own comment predicted). Dropping to 18 fields shifted every slot from
    // idx 14 on, so battleSkin / voiceId / displaySkin misread — a string
    // field landed on table bytes → invalid UTF-8 → the whole
    // skin_table.json was unparseable and the backend got nothing. Deleting
    // the patch (letting pristine upstream through) fixes it. Verified by
    // decoding the live 2.7.41 binary with flatc: char_355_ethan@epoque#7's
    // DisplaySkin.SkinName decodes as 渗透 and BattleSkin resolves.
    //
    // ---- The five June 2026 patches, all DROPPED at 2.7.71 ----
    // Each was measured on CN 26-09-03-04-06-00_ed95a2 by generating twice, once
    // from the patched fork and once from pristine b24069f, and diffing the
    // extraction. Pristine verifies in every row; four of the five patched
    // schemas fail verification outright.
    //
    // activity_table.fbs: the patch removed `defaultEnemyTag` from
    // clz_Torappu_ActivityEnemyDuelConstData. The 2.7.71 binary HAS the field
    // (3 occurrences under pristine, 0 under the patch). Patched output carries
    // 167,669 escaped NUL bytes inside strings and is not valid UTF-8
    // ("MilestoneName": "\u0000\u0000\u0003..."); pristine has zero and parses.
    // Patched CN verify FAILS (`u32 @2354602 unaligned`), pristine PASSES.
    //
    // open_server_table.fbs: the patch removed `compensateEndDay` from
    // clz_Torappu_NewbieCheckInPackageData. Patched CheckInRewardDict starts
    // with a valueless `{"key": 0}` head — the slot-shift artifact — and the
    // file is not valid UTF-8; pristine starts at key 1 with its reward list
    // intact. Patched CN verify FAILS (`u32 @56739 unaligned`), pristine PASSES.
    //
    // display_meta_table.fbs: the patch reverted three upstream changes
    // (`limitId`, `keyCodeType`, and the dict-wrapped `timeLimitInfoList`).
    // It loses data: "KeyCodes" 4 occurrences vs 95 pristine, "LimitId" 0 vs 40,
    // "KeyCodeType" 0 vs 95. A pristine KeyItem reads
    // {"KeyCodeType": "KEYBOARD", "KeyCodes": [50, 84], "KeyId": "num0", ...};
    // the patched one has neither field. Patched CN verify FAILS
    // (`i64 @121364 unaligned`), pristine PASSES.
    //
    // ep_breakbuff_table.fbs: the patch removed `enemyElementBreakDuration`
    // from clz_Torappu_EPBreakBuffData. The live binary has it — pristine, which
    // keeps it, is the layout that verifies. Patched CN verify FAILS with an
    // out-of-bounds read (`Range [1092616552, 1092616556)`) because a float is
    // read as a vector offset; pristine PASSES. Output is byte-identical either
    // way only because the Yostar schema (same 3-field layout) already rescued
    // the patched path via verification.
    //
    // stage_table.fbs: the patch truncated clz_Torappu_CGGalleryGroupData to
    // its first two fields to dodge a garbage `locationId` offset. That symptom
    // does not reproduce on 2.7.71: pristine CgGalleryGroups decode cleanly
    // ({"Displays": ["cgId_mainline_0_1_1", ...], "LocationId": "mainline_0_1",
    // ...}) and the file is valid UTF-8. Both schemas verify here, so this one
    // was merely discarding two real fields rather than corrupting the table.
    let patches: &[(&str, &str, &str)] = &[];

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

/// A `.fbs` reduced to the part that can change how flatc generates code:
/// comments gone, every run of whitespace collapsed to one space.
///
/// The Yostar repo pins each table to the CN commit it was forked from and
/// records that as a leading `// https://github.com/MooncellWiki/...` comment,
/// so a comment-sensitive comparison calls all 57 files "different" when most
/// are byte-identical CN copies.
fn normalize_fbs(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            out.push(' ');
        } else if bytes[i].is_ascii_whitespace() {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            i += 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out.trim().to_string()
}

/// Which tables need a Yostar (EN/Global) schema variant.
///
/// A table needs one exactly when its Yostar `.fbs` differs from the CN `.fbs`
/// after normalisation. Two identical schemas produce identical flatc output
/// and therefore cannot decode a buffer differently: a variant for such a table
/// is dead weight in the binary and a second, pointless verifier pass at
/// runtime. Measured on `OpenArknightsFBS` b24069f vs `ArknightsFlatbuffers`
/// 2026-09-04: 31 of 57 differ, and every one of the 26 identical ones produced
/// a flatc-generated `.rs` byte-identical to its CN counterpart.
///
/// This replaces a hand-curated list. A curated list has to be revisited every
/// time either upstream moves, and the failure mode is silent — a table whose
/// EN layout has just diverged keeps decoding to garbage because nobody added
/// it. The computed set cannot go stale.
fn compute_yostar_schemas(
    cn_fbs_dir: &Path,
    yostar_fbs_dir: &Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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
/// `has_yostar_schema`).
fn clear_generated_dir(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn fetch_yostar_schemas() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = PathBuf::from("/tmp/ArknightsFlatbuffers");
    let fbs_dir = repo_dir.join("yostar");

    if repo_dir.exists() {
        println!("Updating ArknightsFlatbuffers...");
        let _ = Command::new("git")
            .args(["pull", "--rebase"])
            .current_dir(&repo_dir)
            .status();
    } else {
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
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fbs"))
        .collect();
    entries.sort_by_key(std::fs::DirEntry::path);

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

/// Emits `SchemaChoice`, `verifier_opts` and `select_schema_by_verification`:
/// the CN-vs-Yostar schema pick that runs *before* any decode.
fn emit_schema_verification(
    out: &mut String,
    cn_structs: &HashMap<String, Vec<ParsedStruct>>,
    yostar_structs: &HashMap<String, Vec<ParsedStruct>>,
    schema_to_module: &[(&'static str, &'static str)],
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
             \x20               use crate::generated_fbs::{module}::*;\n\
             \x20               {cn_fn}(&opts, data).err().map(|e| first_line(&e))\n\
             \x20           }};\n"
        ));
        match yostar_fn {
            None => arms.push_str("            (cn_err, None)\n        }\n"),
            Some(yostar_fn) => arms.push_str(&format!(
                "            if cn_err.is_none() && !full {{\n\
                 \x20               // The CN schema verified and the caller only wants\n\
                 \x20               // the routing decision: skip the second verify.\n\
                 \x20               return (None, None);\n\
                 \x20           }}\n\
                 \x20           let yostar_err = {{\n\
                 \x20               use crate::generated_fbs_yostar::{module}::*;\n\
                 \x20               {yostar_fn}(&opts, data).err().map(|e| first_line(&e))\n\
                 \x20           }};\n\
                 \x20           (cn_err, Some(yostar_err))\n\
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

 /// One table's verification verdict, for the `unpacker verify` report.
 pub struct TableVerdict {
     /// The schema type `guess_root_type` resolved the filename to.
     pub table: &'static str,
     /// `None` when the CN schema verified, else the first line of the error.
     pub cn: Option<String>,
     /// Outer `None` when the table has no Yostar variant; inner `None` when
     /// the Yostar schema verified.
     pub yostar: Option<Option<String>>,
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

 /// Verifier options for schema selection.
 ///
 /// Deliberately generous. These buffers are real game data, not adversarial
 /// input, and the only question being asked is "do the offsets in this buffer
 /// make sense under this schema". A false *negative* would push a perfectly
 /// good CN table onto the legacy path, so every limit sits far above what any
 /// real table needs; `max_apparent_size` is the flatbuffers default (2 GiB).
 ///
 /// `max_alignment` is not an upstream flatbuffers option; it comes from the
 /// vendored copy in `vendor/flatbuffers` (see MYRTLE-PATCH.md). Hypergryph's
 /// serializer aligns 8-byte scalars to 4 bytes, so upstream's `is_aligned`
 /// rejects a buffer the crate then reads back perfectly well through its
 /// unaligned scalar reads: `roguelike_topic_table` failed verification under
 /// BOTH schemas on BOTH servers with `Type f64 at position N is unaligned`,
 /// N % 8 == 4 (CN 7819508, EN 7285868). Capping the demand at 4 accepts that
 /// layout and leaves every other check untouched.
 ///
 /// `ignore_utf8_errors` is the second vendored option. Upstream rejects a
 /// buffer when a string's BYTES do not decode as UTF-8, which is right for a
 /// reader that hands the `&str` to code assuming valid UTF-8. Ours does not:
 /// every string reaching the emitter goes through `fb_json_macros::json_str`,
 /// i.e. `String::from_utf8_lossy`, so the bad byte becomes U+FFFD and nothing
 /// downstream can observe it. Now that an unverified table is SKIPPED rather
 /// than decoded, a content check the emitter already handles must not be
 /// allowed to cost a whole table.
 ///
 /// `ignore_missing_null_terminator` stays FALSE, deliberately. It is tempting
 /// for the same reason — this reader is length-prefixed and never scans for a
 /// NUL — but it is measurably load-bearing for schema SELECTION. The
 /// `battle/level_script_table` buffer verifies under its own schema and fails
 /// `level_data` on exactly that check; with the option on it verifies under
 /// BOTH and the wrong one wins. EN `roguelike_topic_table` and `building_data`
 /// likewise fail the CN schema only on a missing null terminator, and routing
 /// them back to CN is the multi-GB garbage this whole mechanism exists to
 /// prevent. A missing terminator is a good discriminator; a bad UTF-8 byte is
 /// not.
 fn verifier_opts() -> ::flatbuffers::VerifierOptions {
     ::flatbuffers::VerifierOptions {
         max_depth: 256,
         max_tables: usize::MAX >> 1,
         max_apparent_size: 1 << 31,
         ignore_missing_null_terminator: false,
         max_alignment: 4,
         ignore_utf8_errors: true,
     }
 }

 /// Verify `data` against the schemas available for `schema_type`.
 ///
 /// Returns `(cn_err, yostar_err)`, where a `None` error means that schema
 /// verified, and the outer `None` on `yostar_err` means "no Yostar variant, or
 /// not run". `full = false` short-circuits: once the CN schema verifies the
 /// routing decision is already made, so the Yostar verifier is not run.
 /// `full = true` (the `verify` subcommand) always runs both.
 ///
 /// The whole function returns `None` if a verifier panics — it is not supposed
 /// to, but neither was the decoder, and this is the one place that can still
 /// contain it.
 fn verify_schemas(
     data: &[u8],
     schema_type: &str,
     full: bool,
 ) -> Option<(Option<String>, Option<Option<String>>)> {
     let opts = verifier_opts();
     panic::catch_unwind(AssertUnwindSafe(|| match schema_type {
"#,
    );
    if arms.is_empty() {
        out.push_str("        _ => (None, None),\n");
    } else {
        out.push_str(&arms);
        out.push_str("        _ => (None, None),\n");
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
 /// CN's own activity_table is 15,867,159. Not empty, so nothing detected it,
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
         Some((Some(_), Some(None))) => SchemaChoice::Yostar,
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
         .unwrap_or_else(|| (Some("verifier panicked".to_string()), None));
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = String::new();

    out.push_str("//! Auto-generated FlatBuffer decode dispatch\n");
    out.push_str("//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs\n\n");
    out.push_str("use serde_json::{json, Value};\n");
    out.push_str("use std::panic::{self, AssertUnwindSafe};\n\n");

    out.push_str(
        r"/// Check if data is likely a FlatBuffer
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
    // This is the hand-curated mapping. We embed it directly.
    out.push_str(
        r#"/// Guess the root type from filename
 fn guess_root_type(filename: &str) -> &'static str {
     let lower = filename.to_lowercase();

     // `level_script_table` MUST be tested before the `level_` prefix: it is a
     // battle/ table with its own schema, not a level. Under `level_data`
     // (prts___levels) its 312-byte buffer decoded to a 541-byte nonsense
     // record — `MapId` holding raw bytes, everything else empty — and once an
     // unverified table is skipped rather than decoded it would have been
     // dropped outright. Under its own schema it verifies and decodes to a
     // populated `LevelScriptDataLevelDict`.
     if lower.contains("level_script_table") {
         "level_script_table"
     } else if lower.starts_with("level_") {
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
                        schema_to_module.iter().find(|(_, m)| *m == *module)
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
        out.push_str("            if value.as_object().is_some_and(|o| o.is_empty()) {\n");
        out.push_str("                Err(\"Yostar decode returned empty\".to_string())\n");
        out.push_str("            } else { Ok(value) }\n");
        out.push_str("        }\n");
        out.push_str("        Ok(Err(e)) => Err(e),\n");
        out.push_str("        Err(_) => Err(\"Yostar decode panic\".to_string()),\n");
        out.push_str("    }\n");
    } else {
        out.push_str("    let _ = data;\n");
        out.push_str("    Err(format!(\"No Yostar schema for {}\", schema_type))\n");
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

/// Extract strings from FlatBuffer (fallback for unknown types)
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
        ("level_script_table", "level_script_table_generated"),
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

    // The Yostar variant set is COMPUTED, not curated: a table gets one exactly
    // when its Yostar .fbs differs from the CN one. See `compute_yostar_schemas`.
    // The output dir is cleared first so a table that stops differing stops
    // being compiled in.
    let yostar_schemas = compute_yostar_schemas(&cn_fbs_dir, &yostar_fbs_dir)?;
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
