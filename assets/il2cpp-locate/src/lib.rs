//! `il2cpp-locate` — parse Il2CppDumper `dump.cs` output and pinpoint the
//! native-client method(s) most likely to implement the dynchar "entrance
//! camera-follow" behaviour (compiled `Torappu.*` C#).
//!
//! The `dump.cs` format we target:
//! ```text
//! // Namespace: Torappu.Common
//! public class CharWordController : MonoBehaviour
//! {
//!     private Camera _mainCamera; // 0x18
//!
//!     // RVA: 0x1A2B3C Offset: 0x1A1B3C VA: 0x71A2B3C Slot: 12
//!     private void LateUpdate() { }
//! }
//! ```
//! Each method is preceded by a `// RVA: .. Offset: .. VA: ..` comment; types
//! are grouped under `// Namespace:` comments and declared with a normal C#
//! `class`/`struct` line. See `docs/DYNCHAR_ENTRANCE_INVESTIGATION.md` §5.

use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

/// One parsed method plus its address triple.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Method {
    pub namespace: String,
    pub type_name: String,
    pub signature: String,
    pub method_name: String,
    pub rva: Option<u64>,
    pub offset: Option<u64>,
    pub va: Option<u64>,
    pub slot: Option<u32>,
}

/// A parsed type block: its identity plus the raw text of its body (used for
/// anchor string-searches during scoring).
#[derive(Debug, Clone)]
struct TypeBlock {
    namespace: String,
    name: String,
    body: String,
    methods: Vec<Method>,
}

static NAMESPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*//\s*Namespace:\s*(.*?)\s*$").unwrap());

// A type declaration line, e.g. `public sealed class Foo : Bar {`.
static TYPE_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:[\w]+\s+)*?(?:class|struct|interface|enum)\s+([A-Za-z_][\w`]*)").unwrap()
});

// The address annotation above each method.
static RVA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"//\s*RVA:\s*(0x[0-9A-Fa-f]+|-1)\s+Offset:\s*(0x[0-9A-Fa-f]+|-1)\s+VA:\s*(0x[0-9A-Fa-f]+|-1)(?:\s+Slot:\s*(\d+))?",
    )
    .unwrap()
});

// The method name: last identifier immediately before an opening paren.
static METHOD_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([A-Za-z_][\w]*)\s*\(").unwrap());

fn parse_addr(s: &str) -> Option<u64> {
    let s = s.trim();
    if s == "-1" {
        return None;
    }
    let hex = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(hex, 16).ok()
}

/// Parse a full `dump.cs` string into flat method candidates.
#[must_use]
pub fn parse_dump(text: &str) -> Vec<Method> {
    parse_blocks(text)
        .into_iter()
        .flat_map(|b| b.methods)
        .collect()
}

fn parse_blocks(text: &str) -> Vec<TypeBlock> {
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut namespace = String::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        if let Some(caps) = NAMESPACE_RE.captures(line) {
            namespace = caps[1].to_string();
            i += 1;
            continue;
        }
        if let Some(caps) = TYPE_DECL_RE.captures(line) {
            let name = caps[1].to_string();
            // Find the opening brace (this line or a following one).
            let Some(open) = (i..lines.len()).find(|&k| lines[k].contains('{')) else {
                break; // malformed tail
            };
            // Brace-match to find the matching close.
            let (close, body) = match_braces(&lines, open);
            let block_lines = &lines[i..=close.min(lines.len() - 1)];
            let methods = parse_methods(block_lines, &namespace, &name);
            blocks.push(TypeBlock {
                namespace: namespace.clone(),
                name,
                body,
                methods,
            });
            // Skip the whole consumed span (nested types included).
            i = close + 1;
            continue;
        }
        i += 1;
    }
    blocks
}

/// From the line containing the first `{` at `open`, scan braces (ignoring those
/// inside `//` line comments) and return the line index of the matching `}` plus
/// the joined body text.
fn match_braces(lines: &[&str], open: usize) -> (usize, String) {
    let mut depth = 0i32;
    let mut body = String::new();
    for (k, raw) in lines.iter().enumerate().skip(open) {
        // Strip a trailing line comment so braces inside `// ...` don't count.
        let code = match raw.find("//") {
            Some(p) => &raw[..p],
            None => raw,
        };
        for ch in code.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        body.push_str(raw);
        body.push('\n');
        if depth <= 0 && k >= open {
            return (k, body);
        }
    }
    (lines.len() - 1, body)
}

fn parse_methods(block_lines: &[&str], namespace: &str, type_name: &str) -> Vec<Method> {
    let mut methods = Vec::new();
    let mut j = 0usize;
    while j < block_lines.len() {
        if let Some(caps) = RVA_RE.captures(block_lines[j]) {
            let rva = parse_addr(&caps[1]);
            let offset = parse_addr(&caps[2]);
            let va = parse_addr(&caps[3]);
            let slot = caps.get(4).and_then(|m| m.as_str().parse().ok());

            // Next meaningful line is the declaration (skip blanks / attributes /
            // other comment lines).
            if let Some(sig_line) = block_lines
                .iter()
                .skip(j + 1)
                .find(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with('[') && !t.starts_with("//")
                })
                .copied()
            {
                let mut signature = sig_line.trim().to_string();
                for suffix in ["{ }", "{}", "{", ";"] {
                    if let Some(s) = signature.strip_suffix(suffix) {
                        signature = s.trim().to_string();
                        break;
                    }
                }
                let method_name = METHOD_NAME_RE
                    .captures(&signature)
                    .map(|c| c[1].to_string())
                    .unwrap_or_default();
                // Only treat as a method if it looks like one (has a paren).
                if signature.contains('(') {
                    methods.push(Method {
                        namespace: namespace.to_string(),
                        type_name: type_name.to_string(),
                        signature,
                        method_name,
                        rva,
                        offset,
                        va,
                        slot,
                    });
                }
            }
        }
        j += 1;
    }
    methods
}

// --- Scoring ------------------------------------------------------------------

/// Domain keywords that flag dynchar / Live2D / illust code.
const DOMAIN_KEYWORDS: &[&str] = &[
    "Torappu",
    "CharWord",
    "CharIllust",
    "DynIllust",
    "DynamicIllust",
    "Live2D",
    "L2D",
    "Illust",
    "Entrance",
    "Adjust",
];

/// Words that, combined with "Camera" in a type name, signal a follow rig.
const CAMERA_FOLLOW_WORDS: &[&str] = &[
    "Follow", "Focus", "Track", "LookAt", "Fit", "Frame", "Bounds",
];

/// Method names that signal per-frame camera positioning.
const CAMERA_METHOD_SIGNALS: &[&str] = &[
    "UpdateCamera",
    "LateUpdate",
    "CalcCenter",
    "GetBounds",
    "OnCharReform",
    "Update",
    "Follow",
    "Focus",
    "Fit",
];

/// Rig-anchor strings whose presence in a class body strongly implicates it.
const RIG_ANCHORS: &[&str] = &[
    "Dummy002",
    "static_offset",
    "start_animation_02",
    "_mainCamera",
    "charVoiceOffset",
];

const W_DOMAIN: i32 = 3;
const W_CAMERA_TYPE: i32 = 5;
const W_METHOD_SIGNAL: i32 = 4;
const W_ANCHOR: i32 = 6;

/// A scored candidate with the reasons behind its score.
#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub score: i32,
    pub reasons: Vec<String>,
    #[serde(flatten)]
    pub method: Method,
}

impl Candidate {
    /// Ready-to-paste navigation lines for a disassembler.
    #[must_use]
    pub fn nav_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(off) = self.method.offset {
            out.push(format!("Ghidra: go to file offset {off:#x}"));
        }
        if let Some(va) = self.method.va {
            out.push(format!("IDA: jump to VA {va:#x}"));
        }
        out
    }
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

/// Score & rank every method against the dynchar entrance-camera heuristics.
/// Results are sorted by score descending (ties keep parse order).
#[must_use]
pub fn rank(text: &str) -> Vec<Candidate> {
    let blocks = parse_blocks(text);
    let mut candidates = Vec::new();

    for block in &blocks {
        // Anchor boost is per-type (computed once from the body).
        let mut anchor_hits: Vec<&str> = Vec::new();
        for anchor in RIG_ANCHORS {
            if block.body.contains(anchor) {
                anchor_hits.push(anchor);
            }
        }
        let type_ns = format!("{}.{}", block.namespace, block.name);

        for method in &block.methods {
            let mut score = 0;
            let mut reasons = Vec::new();

            // Domain keywords in namespace or type name.
            for kw in DOMAIN_KEYWORDS {
                if contains_ci(&type_ns, kw) {
                    score += W_DOMAIN;
                    reasons.push(format!("domain:{kw} (+{W_DOMAIN})"));
                }
            }

            // Camera-follow type pattern.
            if contains_ci(&block.name, "Camera")
                && let Some(word) = CAMERA_FOLLOW_WORDS
                    .iter()
                    .find(|w| contains_ci(&block.name, w))
            {
                score += W_CAMERA_TYPE;
                reasons.push(format!("camera-type:*Camera*{word}* (+{W_CAMERA_TYPE})"));
            }

            // Camera-follow method name.
            if let Some(sig) = CAMERA_METHOD_SIGNALS
                .iter()
                .find(|s| contains_ci(&method.method_name, s))
            {
                score += W_METHOD_SIGNAL;
                reasons.push(format!("method:{sig} (+{W_METHOD_SIGNAL})"));
            }

            // Rig-anchor boost (per distinct anchor referenced by the class).
            for anchor in &anchor_hits {
                score += W_ANCHOR;
                reasons.push(format!("anchor:{anchor} (+{W_ANCHOR})"));
            }

            if score > 0 {
                candidates.push(Candidate {
                    score,
                    reasons,
                    method: method.clone(),
                });
            }
        }
    }

    // Stable sort by score descending; ties keep parse order.
    candidates.sort_by_key(|c| std::cmp::Reverse(c.score));
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
// Namespace: UnityEngine
public class Transform : Component
{
    // RVA: 0x100 Offset: 0x90 VA: 0x7100 Slot: 1
    public Vector3 get_position() { }
}

// Namespace: Torappu.Common
public class CharWordCameraController : MonoBehaviour
{
    private Camera _mainCamera; // 0x18
    private Transform Dummy002; // 0x20

    // RVA: 0x1A2B3C Offset: 0x1A1B3C VA: 0x71A2B3C Slot: 12
    private void LateUpdate() { }

    // RVA: 0x1A2C00 Offset: 0x1A1C00 VA: 0x71A2C00 Slot: 13
    public void Init(int id) { }
}

// Namespace: Torappu.UI
public class SomeMenu : MonoBehaviour
{
    // RVA: 0x2000 Offset: 0x1F00 VA: 0x72000
    private void Start() { }
}

// Namespace: Assets
public class Unrelated
{
    // RVA: 0x3000 Offset: 0x2F00 VA: 0x73000
    public void DoThing() { }
}
"#;

    #[test]
    fn parses_all_annotated_methods() {
        let methods = parse_dump(FIXTURE);
        assert_eq!(methods.len(), 5, "expected 5 annotated methods");
        let late = methods
            .iter()
            .find(|m| m.method_name == "LateUpdate")
            .expect("LateUpdate parsed");
        assert_eq!(late.offset, Some(0x1A_1B3C));
        assert_eq!(late.va, Some(0x71A_2B3C));
        assert_eq!(late.slot, Some(12));
        assert_eq!(late.type_name, "CharWordCameraController");
        assert_eq!(late.namespace, "Torappu.Common");
    }

    #[test]
    fn camera_follow_match_ranks_first() {
        let ranked = rank(FIXTURE);
        let top = &ranked[0];
        assert_eq!(top.method.method_name, "LateUpdate");
        assert_eq!(top.method.type_name, "CharWordCameraController");
        assert_eq!(top.method.offset, Some(0x1A_1B3C));
        // Domain (Torappu, CharWord, Camera as substring? no) + camera-type
        // (Camera+? no follow word) ... ensure it beats the runner-up clearly.
        assert!(
            top.score > ranked[1].score,
            "top score {} should beat runner-up {}",
            top.score,
            ranked[1].score
        );
        // The winning method must cite the _mainCamera anchor.
        assert!(
            top.reasons.iter().any(|r| r.contains("_mainCamera")),
            "reasons should include the _mainCamera anchor: {:?}",
            top.reasons
        );
    }

    #[test]
    fn nav_lines_are_ready_to_paste() {
        let ranked = rank(FIXTURE);
        let nav = ranked[0].nav_lines();
        assert!(
            nav.iter()
                .any(|l| l == "Ghidra: go to file offset 0x1a1b3c")
        );
        assert!(nav.iter().any(|l| l == "IDA: jump to VA 0x71a2b3c"));
    }

    #[test]
    fn handles_negative_one_addresses() {
        let src = r#"
// Namespace: Torappu
public class Abstract
{
    // RVA: -1 Offset: -1 VA: 0x0
    public abstract void Foo() { }
}
"#;
        let methods = parse_dump(src);
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].rva, None);
        assert_eq!(methods[0].offset, None);
    }
}
