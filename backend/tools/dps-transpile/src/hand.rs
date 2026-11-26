use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::paths::{Mode, hand_dir};

#[derive(Deserialize)]
pub struct ReferenceCorrection {
    pub file: String,
    pub class: String,
    pub replace: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct ReferenceCorrections {
    corrections: Vec<ReferenceCorrection>,
}

pub fn load_reference_corrections() -> Vec<ReferenceCorrection> {
    let Ok(text) = fs::read_to_string(hand_dir().join("reference_corrections.json")) else {
        return Vec::new();
    };
    serde_json::from_str::<ReferenceCorrections>(&text)
        .expect("Invalid reference_corrections.json")
        .corrections
}

/// `class Name(` up to the next top-level `class `
fn class_span(src: &str, class: &str) -> Option<(usize, usize)> {
    let start = src.find(&format!("class {class}("))?;
    let end = src[start + 1..]
        .find("\nclass ")
        .map_or(src.len(), |i| start + 1 + i);
    Some((start, end))
}

/// Panics when a corrected class or text is gone or ambiguous upstream
pub fn apply_reference_corrections(file: &str, src: &str) -> String {
    let mut out = src.to_string();
    for c in load_reference_corrections()
        .iter()
        .filter(|c| c.file == file)
    {
        let (start, end) = class_span(&out, &c.class)
            .unwrap_or_else(|| panic!("reference correction: class {} not in {file}", c.class));
        let mut body = out[start..end].to_string();
        for (old, new) in &c.replace {
            assert_eq!(
                body.matches(old.as_str()).count(),
                1,
                "reference correction for {}: `{old}` must occur exactly once; upstream \
                 changed this code, review the correction",
                c.class
            );
            body = body.replacen(old.as_str(), new, 1);
        }
        out = format!("{}{body}{}", &out[..start], &out[end..]);
    }
    out
}

/// Python that loads the corrected upstream modules before the script imports them
#[allow(clippy::format_push_string)]
pub fn python_reference_corrections(repo_path: &str) -> String {
    let corrections = load_reference_corrections();
    let mut files: Vec<&str> = corrections.iter().map(|c| c.file.as_str()).collect();
    files.sort_unstable();
    files.dedup();
    let mut out = String::new();
    for file in files {
        let original = fs::read_to_string(format!("{repo_path}/{file}"))
            .unwrap_or_else(|e| panic!("read {repo_path}/{file}: {e}"));
        let patched = apply_reference_corrections(file, &original);
        let module = file.trim_end_matches(".py").replace('/', ".");
        let tmp = std::env::temp_dir().join(format!("myrtle_corrected_{}", file.replace('/', "_")));
        fs::write(&tmp, patched).expect("write corrected reference source");
        let tmp = tmp.display();
        out.push_str(&format!(
            "import types as _types, damagecalc as _pkg\n_m = _types.ModuleType({module:?})\n_m.__file__ = {file:?}\nsys.modules[{module:?}] = _m\nexec(compile(open({tmp:?}, encoding='utf8').read(), {file:?}, 'exec'), _m.__dict__)\n",
            tmp = tmp.to_string()
        ));
    }
    out
}

pub fn guard_output(path: &Path) -> anyhow::Result<()> {
    let hand = hand_dir();
    if resolved(path).starts_with(&hand) {
        bail!(
            "dps-transpile refuses to write {}: {} is hand-written",
            path.display(),
            hand.display()
        );
    }
    Ok(())
}

/// `..` and symlinks resolved
fn resolved(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    match (out.parent().map(fs::canonicalize), out.file_name()) {
        (Some(Ok(parent)), Some(name)) => parent.join(name),
        _ => out,
    }
}

pub fn write_output(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    guard_output(path)?;
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let name = path
        .file_name()
        .with_context(|| format!("{} has no file name", path.display()))?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(".tmp");
    let tmp: PathBuf = parent.join(tmp_name);
    let written = fs::write(&tmp, contents)
        .with_context(|| format!("write {}", tmp.display()))
        .and_then(|()| {
            fs::rename(&tmp, path)
                .with_context(|| format!("rename {} over {}", tmp.display(), path.display()))
        });
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    written
}

pub fn report_hand_overlap(mode: Mode, dispatch_arms: &[(String, String)]) {
    let path = hand_dir().join("formulas.json");
    let Ok(text) = fs::read_to_string(&path) else {
        eprintln!(
            "  NOTE: {} not found; hand overlap not checked",
            path.display()
        );
        return;
    };
    let tables: HashMap<String, HashMap<String, serde_json::Value>> =
        serde_json::from_str(&text).expect("Invalid hand formulas.json");
    let section = match mode {
        Mode::Damage => "dps",
        Mode::Healing => "hps",
    };
    let Some(hand) = tables.get(section) else {
        return;
    };
    for (char_id, fn_name) in dispatch_arms {
        if hand.contains_key(char_id) {
            println!(
                "  HAND OVERRIDE: {char_id} ({fn_name}) is in custom/hand/ and wins over the \
                 transpiled formula; review whether upstream now covers it"
            );
        }
    }
}
