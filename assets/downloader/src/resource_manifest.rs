//! Minimal reader for the Arknights resource manifest (`.idx`) and transitive
//! bundle-dependency resolution.
//!
//! The download profiles ([`crate::profile`]) pick bundles by **name prefix**
//! only. Unity asset bundles, however, reference shared materials/textures/
//! shaders that live in *other* bundles via `allDependencies`. When such a
//! dependency's name matches no kept prefix (e.g. a dynchar rain material in
//! `arts/maps/effect.ab`, or a shared shader in `shaders/special.ab`), it is
//! silently skipped and the referencing asset renders with missing pieces.
//!
//! This module parses just enough of the FlatBuffer (`ResourceManifest`
//! -> `bundles[]` with each `BundleMeta { name, allDependencies }`) to expand a
//! kept name-set into its full dependency closure, so those shared bundles are
//! downloaded too.
//!
//! We hand-roll the tiny bit of FlatBuffer decoding rather than pull the full
//! `flatbuffers` crate + generated schema (which live in the `unpacker` crate)
//! into the downloader — only two fields on one table type are needed.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result, bail};

/// vtable offsets, mirrored from the generated schema
/// (`resource_manifest_generated.rs`).
const RM_VT_BUNDLES: u16 = 6; // ResourceManifest.bundles
const BM_VT_NAME: u16 = 4; // BundleMeta.name
const BM_VT_ALLDEPENDENCIES: u16 = 10; // BundleMeta.allDependencies

/// The resource manifest is prefixed with a fixed-size RSA signature before the
/// FlatBuffer root.
const SIGNATURE_LEN: usize = 128;

/// Parsed bundle dependency graph from a resource manifest.
pub struct ResourceManifest {
    /// bundle index -> bundle name
    names: Vec<String>,
    /// bundle index -> direct dependency bundle indices (`allDependencies`)
    deps: Vec<Vec<usize>>,
    /// bundle name -> bundle index
    index_of: HashMap<String, usize>,
}

impl ResourceManifest {
    /// Construct directly from parts (used by tests and the parser).
    #[must_use]
    pub fn from_parts(names: Vec<String>, deps: Vec<Vec<usize>>) -> Self {
        let index_of = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        Self {
            names,
            deps,
            index_of,
        }
    }

    /// Load and parse a resource manifest `.idx` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the FlatBuffer is
    /// malformed / truncated.
    pub fn load(idx_path: &Path) -> Result<Self> {
        let data = std::fs::read(idx_path)
            .with_context(|| format!("read manifest {}", idx_path.display()))?;
        Self::parse(&data)
    }

    /// Locate the resource manifest `.idx` inside `dir` (a 32-hex-char md5 name,
    /// e.g. `e5b768244d4f6a988d12d67ca6e02cca.idx`) and parse it. Returns
    /// `Ok(None)` when no such file is present so callers can fall back to
    /// prefix-only selection.
    ///
    /// # Errors
    ///
    /// Returns an error only if a candidate file is found but fails to parse.
    pub fn find_and_load(dir: &Path) -> Result<Option<Self>> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(None);
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if is_manifest_idx_name(&name) {
                return Ok(Some(Self::load(&entry.path())?));
            }
        }
        Ok(None)
    }

    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < SIGNATURE_LEN + 4 {
            bail!("resource manifest too small ({} bytes)", data.len());
        }
        let fb = &data[SIGNATURE_LEN..];

        // Root table offset lives at the start of the FlatBuffer.
        let root = read_uoffset(fb, 0)?;
        let bundles_vec = table_field_offset(fb, root, RM_VT_BUNDLES)?
            .context("resource manifest has no bundles vector")?;
        let bundles_start = read_uoffset(fb, bundles_vec)?;
        let count = read_u32(fb, bundles_start)? as usize;

        let mut names = Vec::with_capacity(count);
        let mut deps = Vec::with_capacity(count);
        for i in 0..count {
            // Each element of a vector-of-tables is a uoffset to the table.
            let elem_ptr = bundles_start + 4 + i * 4;
            let bundle = read_uoffset(fb, elem_ptr)?;

            let name = match table_field_offset(fb, bundle, BM_VT_NAME)? {
                Some(off) => read_string(fb, off)?,
                None => String::new(),
            };
            let bundle_deps = match table_field_offset(fb, bundle, BM_VT_ALLDEPENDENCIES)? {
                Some(off) => read_i32_vector(fb, off)?,
                None => Vec::new(),
            };
            names.push(name);
            deps.push(bundle_deps);
        }

        Ok(Self::from_parts(names, deps))
    }

    /// Expand `seed_names` to the full set of bundle names reachable through
    /// transitive `allDependencies`. Names not present in the manifest are kept
    /// as-is (they simply contribute no edges). The returned set always includes
    /// every seed name that exists in the manifest plus all names reachable from
    /// them.
    #[must_use]
    pub fn dependency_closure<'a, I>(&self, seed_names: I) -> HashSet<String>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut visited: HashSet<usize> = HashSet::new();
        let mut stack: Vec<usize> = Vec::new();
        for name in seed_names {
            if let Some(&idx) = self.index_of.get(name)
                && visited.insert(idx)
            {
                stack.push(idx);
            }
        }
        while let Some(idx) = stack.pop() {
            for &dep in &self.deps[idx] {
                if dep < self.names.len() && visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }
        visited.into_iter().map(|i| self.names[i].clone()).collect()
    }
}

/// A resource manifest `.idx` is named `<32-hex-md5>.idx`, distinguishing it
/// from `hot_update_list.idx`.
fn is_manifest_idx_name(name: &str) -> bool {
    match name.strip_suffix(".idx") {
        Some(stem) => stem.len() == 32 && stem.bytes().all(|b| b.is_ascii_hexdigit()),
        None => false,
    }
}

// --- minimal little-endian FlatBuffer primitives ---

fn read_u16(buf: &[u8], pos: usize) -> Result<u16> {
    let bytes = buf.get(pos..pos + 2).context("u16 out of bounds")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_i32(buf: &[u8], pos: usize) -> Result<i32> {
    let bytes = buf.get(pos..pos + 4).context("i32 out of bounds")?;
    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u32(buf: &[u8], pos: usize) -> Result<u32> {
    let bytes = buf.get(pos..pos + 4).context("u32 out of bounds")?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Follow a forward uoffset stored at `pos` to the position it points to.
fn read_uoffset(buf: &[u8], pos: usize) -> Result<usize> {
    Ok(pos + read_u32(buf, pos)? as usize)
}

/// Resolve a table field to its absolute byte position, or `None` when the field
/// is absent (default). `table` is the absolute position of the table.
fn table_field_offset(buf: &[u8], table: usize, vt_offset: u16) -> Result<Option<usize>> {
    // A table starts with a signed soffset to its vtable.
    let soffset = read_i32(buf, table)?;
    let vtable =
        usize::try_from(table as i64 - i64::from(soffset)).context("vtable position underflow")?;
    let vt_size = read_u16(buf, vtable)?;
    if vt_offset >= vt_size {
        return Ok(None);
    }
    let field_rel = read_u16(buf, vtable + vt_offset as usize)?;
    if field_rel == 0 {
        return Ok(None);
    }
    Ok(Some(table + field_rel as usize))
}

fn read_string(buf: &[u8], field_pos: usize) -> Result<String> {
    let str_pos = read_uoffset(buf, field_pos)?;
    let len = read_u32(buf, str_pos)? as usize;
    let bytes = buf
        .get(str_pos + 4..str_pos + 4 + len)
        .context("string out of bounds")?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn read_i32_vector(buf: &[u8], field_pos: usize) -> Result<Vec<usize>> {
    let vec_pos = read_uoffset(buf, field_pos)?;
    let len = read_u32(buf, vec_pos)? as usize;
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let v = read_i32(buf, vec_pos + 4 + i * 4)?;
        if v >= 0 {
            out.push(v as usize);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::keep_for_operators;

    /// A dynchar bundle that depends on a shared bundle whose name matches no
    /// operators prefix must pull that shared bundle in via the dependency
    /// closure, even though prefix-only selection would drop it.
    #[test]
    fn closure_pulls_in_non_prefix_dependency() {
        // 0: dynchar (kept by prefix), depends on 1 and 2
        // 1: shared shader bundle — NOT matched by any operators prefix
        // 2: refs/fx texture (kept by prefix), depends on 3
        // 3: shared map effect bundle — NOT matched by any operators prefix
        // 4: unrelated bundle, must NOT appear
        let names = vec![
            "arts/dynchars/char_437_mizuki_sale#7.ab".to_string(),
            "shaders/special.ab".to_string(),
            "refs/fx/texture/star.ab".to_string(),
            "arts/maps/effect.ab".to_string(),
            "scenes/obt/main/level_main_00-01/level_main_00-01.ab".to_string(),
        ];
        let deps = vec![vec![1, 2], vec![], vec![3], vec![], vec![]];
        let rm = ResourceManifest::from_parts(names, deps);

        // Sanity: the shared bundles are genuinely missed by prefix selection.
        assert!(!keep_for_operators("shaders/special.ab"));
        assert!(!keep_for_operators("arts/maps/effect.ab"));

        let closure = rm.dependency_closure(["arts/dynchars/char_437_mizuki_sale#7.ab"]);

        assert!(
            closure.contains("shaders/special.ab"),
            "direct non-prefix dep missing"
        );
        assert!(
            closure.contains("arts/maps/effect.ab"),
            "transitive non-prefix dep missing"
        );
        assert!(closure.contains("refs/fx/texture/star.ab"));
        assert!(closure.contains("arts/dynchars/char_437_mizuki_sale#7.ab"));
        assert!(!closure.contains("scenes/obt/main/level_main_00-01/level_main_00-01.ab"));
    }

    #[test]
    fn unknown_seed_names_are_ignored() {
        let rm = ResourceManifest::from_parts(vec!["a.ab".to_string()], vec![vec![]]);
        let closure = rm.dependency_closure(["does/not/exist.ab"]);
        assert!(closure.is_empty());
    }

    #[test]
    fn recognizes_manifest_idx_name() {
        assert!(is_manifest_idx_name("e5b768244d4f6a988d12d67ca6e02cca.idx"));
        assert!(!is_manifest_idx_name("hot_update_list.idx"));
        assert!(!is_manifest_idx_name("e5b768244d4f6a988d12d67ca6e02cca.ab"));
    }
}
