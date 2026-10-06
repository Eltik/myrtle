//! Scene-aware chunking of Arknights story scripts.
//!
//! The two obvious approaches are both measurably wrong on this corpus. A
//! fixed-size token window splits speaker turns, stranding a line of dialogue
//! from the `[name="..."]` that owns it, which makes the chunk unattributable.
//! Breaking on every background command is wrong in a smaller way than first
//! believed: 822 of 15,039 `[Background]` commands name no plate and are
//! fades, 5.5%, not the roughly half an earlier regex sample suggested. The
//! larger errors were elsewhere, see [`hard_boundary`].
//!
//! Command frequencies from the backend's own parser over the 1,862 scripted
//! EN stories, measured 2026-09-24, which are the numbers these rules are
//! derived from:
//!
//! ```text
//! name 334594  charslot 165931  character 130277  delay 65758  blocker 65619
//! dialog 57856  playsound 36127  text 32292  background 15039
//! camerashake 13291  stopmusic 9215  playmusic 7801  subtitle 7367
//! sticker 5295  image 5082  predicate 4754  decision 3883  header 1862
//! multiline 1450
//! ```
//!
//! `blocker` and `delay` are the tempting wrong answer: they outnumber
//! `background` 4.4 to one and are intra-scene pacing. They are never
//! boundaries here, and the invariant test asserts that adding or removing
//! them cannot move a chunk edge. `header` occurs exactly once per story, as
//! its first command, so treating it as a boundary only opens scene 1.
//!
//! Scenes are roughly chunk-sized, not smaller than chunks as an earlier
//! estimate claimed: 13,700 scene boundaries over 1,862 stories against 13,837
//! chunks. The packer therefore does both jobs, merging short scenes up to the
//! floor and cutting long ones at turn boundaries under the ceiling.
//!
//! Measured on the real corpus with the real gte-modernbert tokenizer and the
//! calibrated overhead (2 per sequence, 1 per join, both measured, the join
//! cost identical across all 24,757 real joins): 13,837 chunks, p10 328, p50
//! 548, p90 595, max 1,355 tokens, 13.8% under the 400 floor and 0.43% (60)
//! over the 600 ceiling, every one of those 60 a single turn too long to split.
//! The stored `token_count` equals a whole-chunk encode for all 13,837. Exact
//! reconstruction of every story's prose from its chunks holds for all 1,862,
//! so nothing is lost and nothing is duplicated.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

/// The kinds whose prose sits in `text`, mirroring
/// `backend::core::story::parser::PROSE_KINDS` (parser.rs:63).
pub const PROSE_KINDS: &[&str] = &["name", "text", "multiline", "narration"];

/// The kinds whose prose sits in `args["text"]`, mirroring `PROSE_ARG_KINDS`
/// (parser.rs:66).
pub const PROSE_ARG_KINDS: &[&str] = &["subtitle", "sticker"];

/// Mirror of `backend::core::story::parser::StoryCommand` (parser.rs:52) as it
/// arrives over `GET /api/story/{id}`, which serializes camelCase. Duplicated
/// rather than shared because trevor is a sibling crate with no path
/// dependency on backend, exactly as `discord/` is.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryCommand {
    pub kind: String,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
    #[serde(default)]
    pub text: Option<String>,
    /// 1-based line number in the source script file.
    pub line: u32,
}

/// One retrieval chunk.
///
/// `line_start`/`line_end` are the source-file lines of the first and last
/// PROSE commands, and they are the anchor the evaluation gold set references.
/// Chunk ids churn whenever chunking is retuned; line spans do not, so the
/// gold set survives retuning and the eval keeps measuring the thing being
/// changed. Referencing `chunk_id` from the gold set would build a ratchet
/// that prevents the tuning it was meant to measure.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub chunk_id: String,
    pub story_id: String,
    pub group_id: String,
    pub ordinal: u32,
    /// Index of the scene this chunk STARTS in.
    pub scene_ordinal: u32,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
    /// Display names, deduped, in first-appearance order. Excludes the empty
    /// speaker and the masking conventions.
    pub speakers: Vec<String>,
    /// Sprite asset ids from `charslot`/`character`. NOT display names: the
    /// `name` argument there is an asset id such as `avg_npc_032`.
    pub on_screen: Vec<String>,
    pub background: Option<String>,
    pub token_count: u32,
    /// sha256 of `text`, first 16 hex chars. Change detection, not identity.
    pub content_sha: String,
    /// Contextual prefix (P1), prepended to `text` for embedding and BM25
    /// only; display, reranking and anchors use `text`. Absent in P0, and
    /// absent means exactly the P0 behavior: the key is not serialized and
    /// [`Chunk::indexed_text`] returns `text` unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

impl Chunk {
    /// What the retrievers index: `prefix`, a blank line, then `text`, or
    /// `text` alone when there is no prefix.
    #[must_use]
    pub fn indexed_text(&self) -> std::borrow::Cow<'_, str> {
        indexed_text(self.prefix.as_deref(), &self.text)
    }
}

/// The one place the prefix is joined to the text, shared with the embedder,
/// which reads `chunks.jsonl` rows without building a [`Chunk`].
#[must_use]
pub fn indexed_text<'a>(prefix: Option<&str>, text: &'a str) -> std::borrow::Cow<'a, str> {
    match prefix {
        Some(p) => std::borrow::Cow::Owned(format!("{p}\n\n{text}")),
        None => std::borrow::Cow::Borrowed(text),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkConfig {
    pub target_min_tokens: u32,
    pub target_max_tokens: u32,
    pub max_scenes_per_chunk: u32,
    pub overlap_turns: u32,
    /// Tokens the model adds once per sequence (`[CLS]` and `[SEP]` for
    /// gte-modernbert, so 2). The chunker packs by summing per-turn costs, and
    /// without this the ceiling it enforces is not the ceiling the embedder
    /// sees. Measured with the real tokenizer at 0/0: the whole-chunk encode
    /// exceeded the per-turn sum by 2 to 47 tokens (median 15), and 26.1% of
    /// chunks crossed the 600 ceiling. Default 0, which reproduces the old
    /// packing exactly; the corpus build sets it from the tokenizer.
    pub sequence_overhead_tokens: u32,
    /// Tokens each `\n` between two rendered turns costs, charged once per
    /// join. Default 0, as above.
    pub joiner_tokens: u32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            target_min_tokens: 400,
            target_max_tokens: 600,
            max_scenes_per_chunk: 3,
            overlap_turns: 0,
            sequence_overhead_tokens: 0,
            joiner_tokens: 0,
        }
    }
}

/// Speaker strings that mask identity rather than naming a character. `???`
/// alone accounts for 5,777 prose lines. They stay in the rendered text but
/// never enter the speaker index.
const MASKED_SPEAKERS: &[&str] = &["", "?", "??", "???"];

fn is_masked(speaker: &str) -> bool {
    MASKED_SPEAKERS.contains(&speaker.trim())
}

/// Prose carried by a command, following `parser::word_count` (parser.rs:427)
/// exactly. Mirroring that selection is deliberate: a different prose
/// predicate here would silently disagree with the `wordCount` the API
/// reports.
fn prose_of(c: &StoryCommand) -> Option<&str> {
    if PROSE_KINDS.contains(&c.kind.as_str()) {
        c.text.as_deref()
    } else if PROSE_ARG_KINDS.contains(&c.kind.as_str()) {
        c.args.get("text").map(String::as_str)
    } else {
        None
    }
}

/// The commands that set a background, exactly the backend's own
/// `BACKGROUND_KINDS` (backend/src/core/story/mod.rs). Measured over the 1,862
/// scripted EN stories: `background` names its plate with `image` (14,217 of
/// 15,039; the other 822 are fades), `backgroundtween` with `image` (49 of
/// 491), and `gridbg`, `largebg` and `verticalbg` with `imagegroup` ONLY (86,
/// 60 and 17 uses, never `image`). Reading `image` alone would treat every
/// strip background as a fade.
const BACKGROUND_KINDS: &[&str] = &[
    "background",
    "backgroundtween",
    "largebg",
    "gridbg",
    "verticalbg",
];

/// The outcome of testing one command for a hard scene boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Boundary {
    /// Not a boundary. A background command naming no plate is a fade or a
    /// tween of the current one: it must neither open a scene nor clear the
    /// current background.
    None,
    /// Opens a scene, and sets the background to this plate.
    OpensWith(Option<String>),
}

/// `[Image]` is deliberately NOT a boundary. It is a CG overlay: per
/// `docs/story-reader.md`, a new `[Image]` fades in over the still-opaque
/// scene and a bare one fades it out, and the resolver classes an
/// `[Image]`-written plate as a CG even when it is named `bg_*`. Treating it as
/// a scene change, as this function once did, opened 1,973 extra scenes and
/// left a CG name in the `background` field of 986 of 13,439 chunks (7.3%),
/// which the contextual-prefix pass would then have read as the location.
/// After the change that is 64, and every one of them is a real background:
/// the corpus uses exactly three plate names both ways (`ac5_2_on`,
/// `bg_0_ori`, `bg_towerinside`), and 0 chunks carry a background that no
/// background command set.
fn hard_boundary(c: &StoryCommand, current_bg: Option<&str>) -> Boundary {
    let kind = c.kind.as_str();
    if BACKGROUND_KINDS.contains(&kind) {
        let plate = c.args.get("image").or_else(|| c.args.get("imagegroup"));
        return match plate {
            Some(p) if !p.is_empty() && Some(p.as_str()) != current_bg => {
                Boundary::OpensWith(Some(p.clone()))
            }
            _ => Boundary::None,
        };
    }
    if kind == "header" {
        return Boundary::OpensWith(current_bg.map(str::to_owned));
    }
    Boundary::None
}

/// A soft boundary is a preferred place to close a chunk but does not open a
/// scene.
fn soft_boundary(c: &StoryCommand) -> bool {
    matches!(c.kind.as_str(), "decision" | "curtain")
}

#[derive(Debug, Clone)]
struct Turn {
    speaker: Option<String>,
    lines: Vec<String>,
    line_start: u32,
    line_end: u32,
    scene: u32,
    /// True when a soft boundary was seen immediately before this turn.
    after_soft: bool,
    /// Sprite asset ids on screen when this turn opened.
    on_screen: Vec<String>,
}

impl Turn {
    fn render(&self) -> String {
        let body = self.lines.join(" ");
        match self.speaker.as_deref() {
            Some(s) if !s.trim().is_empty() => format!("{s}: {body}"),
            _ => body,
        }
    }
}

/// Walk the command stream once, assigning scenes and grouping prose into
/// speaker turns.
///
/// Consecutive `name` commands carrying the SAME speaker merge into one turn.
/// That keeps a monologue from repeating its own attribution every line, which
/// reads better and gives lexical retrieval one strong speaker match per turn
/// rather than several weak ones.
fn turns_of(commands: &[StoryCommand]) -> (Vec<Turn>, Vec<(u32, Option<String>)>) {
    let mut turns: Vec<Turn> = Vec::new();
    let mut scene: u32 = 0;
    let mut current_bg: Option<String> = None;
    let mut pending_soft = false;
    // (scene ordinal, background image at the time that scene opened)
    let mut scene_bg: Vec<(u32, Option<String>)> = vec![(0, None)];
    // Who is on screen is RUNNING STATE, not a property of a line range: the
    // `charslot`/`character` commands that place a sprite come BEFORE the
    // dialogue it speaks. Scanning a chunk's own line span for them finds
    // nothing, which is a silent empty-metadata bug rather than a loud one.
    let mut slots: BTreeMap<String, String> = BTreeMap::new();
    let mut chars: Vec<String> = Vec::new();
    let mut cutin: Vec<String> = Vec::new();

    for c in commands {
        if let Boundary::OpensWith(new_bg) = hard_boundary(c, current_bg.as_deref()) {
            scene += 1;
            current_bg = new_bg;
            scene_bg.push((scene, current_bg.clone()));
            pending_soft = false;
            continue;
        }
        if soft_boundary(c) {
            pending_soft = true;
            continue;
        }
        match c.kind.as_str() {
            "charslot" => {
                let slot = c
                    .args
                    .get("slot")
                    .cloned()
                    .unwrap_or_else(|| "m".to_owned());
                match c.args.get("name") {
                    Some(n) if !n.is_empty() => {
                        slots.insert(slot, n.clone());
                    }
                    // A charslot with no name moves or clears that slot.
                    _ => {
                        slots.remove(&slot);
                    }
                }
                continue;
            }
            "character" | "charactercutin" => {
                let named: Vec<String> = ["name", "name2"]
                    .iter()
                    .filter_map(|k| c.args.get(*k))
                    .filter(|v| !v.is_empty())
                    .cloned()
                    .collect();
                // A bare `[Character]` clears the stage. A cut-in is a close-up
                // insert over the stage, so it holds its own state rather than
                // replacing who is standing there.
                if c.kind == "character" {
                    chars = named;
                } else {
                    cutin = named;
                }
                continue;
            }
            // `interlude` is in the backend's CHARACTER_KINDS but is left out
            // here: it names a sprite in only 258 of its 377 named uses and a
            // background or CG in the rest, so tracking it would put scenery
            // on stage a third of the time.
            _ => {}
        }

        let Some(prose) = prose_of(c) else { continue };
        if prose.trim().is_empty() {
            continue;
        }

        let speaker = if c.kind == "name" {
            Some(c.args.get("name").cloned().unwrap_or_default())
        } else {
            None
        };

        let continues = match (&speaker, turns.last()) {
            // A non-name prose line continues whatever turn is open.
            (None, Some(t)) => t.scene == scene,
            // A name command continues only a same-speaker turn in the same scene.
            (Some(s), Some(t)) => t.scene == scene && t.speaker.as_deref() == Some(s.as_str()),
            _ => false,
        };

        if continues && !pending_soft {
            let t = turns.last_mut().expect("continues implies a last turn");
            t.lines.push(prose.trim().to_owned());
            t.line_end = c.line;
        } else {
            turns.push(Turn {
                speaker: speaker.clone(),
                lines: vec![prose.trim().to_owned()],
                line_start: c.line,
                line_end: c.line,
                scene,
                after_soft: pending_soft,
                on_screen: {
                    let mut set: std::collections::BTreeSet<String> =
                        chars.iter().cloned().collect();
                    set.extend(slots.values().cloned());
                    set.extend(cutin.iter().cloned());
                    set.into_iter().collect()
                },
            });
            pending_soft = false;
        }
    }

    (turns, scene_bg)
}

/// sha256 of `s`, first 16 hex chars: the `content_sha` rule, shared with the
/// embedder's reuse key so the two agree byte for byte.
#[must_use]
pub fn sha16(s: &str) -> String {
    use std::fmt::Write as _;
    let digest = Sha256::digest(s.as_bytes());
    digest
        .iter()
        .take(8)
        .fold(String::with_capacity(16), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

/// Chunk one story.
///
/// Pure and deterministic in `(story_id, group_id, commands, cfg)`: the same
/// input yields byte-identical output, `content_sha` included. The tokenizer
/// arrives as a closure so this module never links `tokenizers`.
///
/// `count_tokens` must count WITHOUT the model's special tokens. They belong
/// to the sequence, once, and are charged through
/// `cfg.sequence_overhead_tokens`; counting them per turn would bill `[CLS]`
/// and `[SEP]` a dozen times per chunk.
#[must_use]
pub fn chunk_story(
    story_id: &str,
    group_id: &str,
    commands: &[StoryCommand],
    cfg: &ChunkConfig,
    count_tokens: &dyn Fn(&str) -> u32,
) -> Vec<Chunk> {
    let (turns, scene_bg) = turns_of(commands);
    if turns.is_empty() {
        return Vec::new();
    }

    let bg_for = |scene: u32| -> Option<String> {
        scene_bg
            .iter()
            .rev()
            .find(|(s, _)| *s <= scene)
            .and_then(|(_, b)| b.clone())
    };

    let rendered: Vec<String> = turns.iter().map(Turn::render).collect();
    let costs: Vec<u32> = rendered.iter().map(|t| count_tokens(t)).collect();

    let mut out: Vec<Chunk> = Vec::new();
    let mut start = 0usize;

    while start < turns.len() {
        let mut end = start; // exclusive once incremented
        let mut total = cfg.sequence_overhead_tokens;
        let mut scenes_seen: Vec<u32> = Vec::new();

        while end < turns.len() {
            let t = &turns[end];
            let join = if end > start { cfg.joiner_tokens } else { 0 };
            let next_total = total.saturating_add(join).saturating_add(costs[end]);

            // Would exceed the ceiling, and we already have something: stop.
            if end > start && next_total > cfg.target_max_tokens {
                break;
            }

            let new_scene = !scenes_seen.contains(&t.scene);
            if end > start
                && new_scene
                && u32::try_from(scenes_seen.len()).unwrap_or(u32::MAX) >= cfg.max_scenes_per_chunk
            {
                break;
            }

            // Past the floor and a natural seam opens here: prefer to stop.
            if end > start && total >= cfg.target_min_tokens && (new_scene || t.after_soft) {
                break;
            }

            if new_scene {
                scenes_seen.push(t.scene);
            }
            total = next_total;
            end += 1;
        }

        // The loop above always admits the turn at `start`, because every
        // break is guarded by `end > start`. So a single turn wider than the
        // ceiling is never split and never dropped: it closes as its own
        // oversized chunk through the ceiling branch on the next iteration.
        // Splitting it would strand dialogue from its speaker. 60 of 13,837
        // real chunks take this path, the largest at 1,355 tokens.
        debug_assert!(end > start);

        let slice = &turns[start..end];
        let text = slice
            .iter()
            .map(Turn::render)
            .collect::<Vec<_>>()
            .join("\n");
        let line_start = slice.first().expect("non-empty slice").line_start;
        let line_end = slice.last().expect("non-empty slice").line_end;
        let scene_ordinal = slice.first().expect("non-empty slice").scene;

        let mut speakers: Vec<String> = Vec::new();
        for t in slice {
            if let Some(s) = &t.speaker {
                if !is_masked(s) && !speakers.contains(s) {
                    speakers.push(s.clone());
                }
            }
        }

        let ordinal = u32::try_from(out.len()).unwrap_or(u32::MAX);
        out.push(Chunk {
            chunk_id: format!("{story_id}#{ordinal:04}"),
            story_id: story_id.to_owned(),
            group_id: group_id.to_owned(),
            ordinal,
            scene_ordinal,
            line_start,
            line_end,
            content_sha: sha16(&text),
            // Stored as a whole-text count, not the packing estimate: this is
            // the number the embedder sees, joins and merges included. At
            // default config it can differ from the packing sum, because the
            // old stored value omitted exactly what the new fields model.
            token_count: count_tokens(&text).saturating_add(cfg.sequence_overhead_tokens),
            speakers,
            on_screen: {
                let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
                for t in slice {
                    set.extend(t.on_screen.iter().cloned());
                }
                set.into_iter().collect()
            },
            background: bg_for(scene_ordinal),
            text,
            prefix: None,
        });

        // Advance, honouring overlap. Overlap must not stall: always move at
        // least one turn forward.
        let step = (end - start)
            .saturating_sub(cfg.overlap_turns as usize)
            .max(1);
        start += step;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(s: &str) -> u32 {
        u32::try_from(s.split_whitespace().count()).unwrap_or(u32::MAX)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn _unused() {}

    fn cmd(kind: &str, args: &[(&str, &str)], text: Option<&str>, line: u32) -> StoryCommand {
        StoryCommand {
            kind: kind.to_owned(),
            args: args
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            text: text.map(str::to_owned),
            line,
        }
    }

    fn say(speaker: &str, text: &str, line: u32) -> StoryCommand {
        cmd("name", &[("name", speaker)], Some(text), line)
    }

    fn small() -> ChunkConfig {
        ChunkConfig {
            target_min_tokens: 6,
            target_max_tokens: 12,
            ..ChunkConfig::default()
        }
    }

    #[test]
    fn background_without_image_is_not_a_scene_break() {
        let cmds = vec![
            cmd("background", &[("image", "bg_a")], None, 1),
            say("Amiya", "one two three", 2),
            // a fade: fadetime only, no image
            cmd(
                "background",
                &[("fadetime", "1"), ("block", "false")],
                None,
                3,
            ),
            say("Amiya", "four five six", 4),
        ];
        let (turns, _) = turns_of(&cmds);
        let scenes: Vec<u32> = turns.iter().map(|t| t.scene).collect();
        assert_eq!(
            scenes,
            vec![1],
            "a fade must not open a scene, so both Amiya lines stay one turn"
        );
    }

    #[test]
    fn repeated_identical_background_is_not_a_scene_break() {
        let cmds = vec![
            cmd("background", &[("image", "bg_a")], None, 1),
            say("W", "one two three", 2),
            cmd("background", &[("image", "bg_a")], None, 3),
            say("W", "four five six", 4),
            cmd("background", &[("image", "bg_b")], None, 5),
            say("W", "seven eight nine", 6),
        ];
        let (turns, _) = turns_of(&cmds);
        let scenes: Vec<u32> = turns.iter().map(|t| t.scene).collect();
        // The first two W lines merge (same speaker, same scene); bg_b opens
        // scene 2 and therefore a new turn.
        assert_eq!(scenes, vec![1, 2]);
    }

    #[test]
    fn pacing_commands_are_inert() {
        let base = vec![
            say("Amiya", "alpha bravo charlie", 1),
            say("Kal'tsit", "delta echo foxtrot", 2),
        ];
        let mut noisy = vec![
            cmd("delay", &[("time", "1.3")], None, 1),
            say("Amiya", "alpha bravo charlie", 1),
            cmd("blocker", &[("a", "1")], None, 2),
            cmd("playsound", &[("key", "x")], None, 2),
            cmd("dialog", &[], None, 2),
            say("Kal'tsit", "delta echo foxtrot", 2),
        ];
        noisy.push(cmd("stopmusic", &[], None, 3));

        let a = chunk_story("s", "g", &base, &small(), &words);
        let b = chunk_story("s", "g", &noisy, &small(), &words);
        assert_eq!(
            a.iter().map(|c| &c.text).collect::<Vec<_>>(),
            b.iter().map(|c| &c.text).collect::<Vec<_>>(),
            "delay/blocker/dialog/playsound must never move a chunk edge"
        );
        assert_eq!(a, b, "and the whole chunk must be byte-identical");
    }

    #[test]
    fn a_turn_is_never_split() {
        // One turn far wider than the ceiling.
        let long = "lorem ".repeat(200);
        let cmds = vec![say("Amiya", long.trim(), 1)];
        let out = chunk_story("s", "g", &cmds, &small(), &words);
        assert_eq!(out.len(), 1, "an oversized turn survives whole");
        assert!(out[0].token_count > small().target_max_tokens);
        assert!(out[0].text.starts_with("Amiya: lorem"));
    }

    #[test]
    fn sub_minimum_story_is_one_chunk() {
        let cmds = vec![say("Dobermann", "short line", 1)];
        let out = chunk_story("s", "g", &cmds, &small(), &words);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].ordinal, 0);
        assert_eq!(out[0].chunk_id, "s#0000");
    }

    #[test]
    fn concatenation_is_lossless_and_unduplicated() {
        let mut cmds = Vec::new();
        for i in 0..40u32 {
            if i % 7 == 0 {
                cmds.push(cmd(
                    "background",
                    &[("image", &format!("bg_{i}"))],
                    None,
                    i * 2,
                ));
            }
            cmds.push(say(
                if i % 2 == 0 { "Amiya" } else { "Kal'tsit" },
                &format!("line {i} alpha bravo charlie delta"),
                i * 2 + 1,
            ));
        }
        let out = chunk_story("s", "g", &cmds, &small(), &words);
        assert!(out.len() > 4, "expected several chunks, got {}", out.len());

        let (turns, _) = turns_of(&cmds);
        let expected: Vec<String> = turns.iter().map(Turn::render).collect();
        let got: Vec<String> = out
            .iter()
            .flat_map(|c| c.text.split('\n').map(str::to_owned))
            .collect();
        assert_eq!(got, expected, "no loss and no duplication at overlap 0");

        for w in out.windows(2) {
            assert!(
                w[0].line_end < w[1].line_start,
                "line spans must not overlap: {:?} then {:?}",
                w[0].line_end,
                w[1].line_start
            );
        }
        for (i, c) in out.iter().enumerate() {
            assert_eq!(c.ordinal as usize, i);
            assert!(c.line_start <= c.line_end);
        }
    }

    #[test]
    fn same_speaker_lines_merge_and_masked_speakers_are_excluded() {
        let cmds = vec![
            say("Amiya", "one two", 1),
            say("Amiya", "three four", 2),
            say("???", "five six", 3),
            say("", "seven eight", 4),
        ];
        let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert_eq!(out.len(), 1);
        assert!(
            out[0].text.starts_with("Amiya: one two three four"),
            "same-speaker lines merge: {:?}",
            out[0].text
        );
        assert_eq!(out[0].speakers, vec!["Amiya".to_owned()]);
    }

    #[test]
    fn prose_arg_kinds_are_read_from_args() {
        let cmds = vec![
            cmd("subtitle", &[("text", "a narrated caption")], None, 1),
            cmd("sticker", &[("text", "a sticker line")], None, 2),
        ];
        let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains("a narrated caption"));
        assert!(out[0].text.contains("a sticker line"));
    }

    #[test]
    fn metadata_is_populated() {
        let cmds = vec![
            cmd("background", &[("image", "bg_rhodes")], None, 1),
            cmd(
                "charslot",
                &[("slot", "l"), ("name", "avg_npc_032")],
                None,
                2,
            ),
            say("Amiya", "one two three four", 3),
        ];
        let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert_eq!(out[0].background.as_deref(), Some("bg_rhodes"));
        assert_eq!(out[0].on_screen, vec!["avg_npc_032".to_owned()]);
        assert_eq!(out[0].scene_ordinal, 1);
        assert_eq!(out[0].content_sha.len(), 16);
    }

    #[test]
    fn strip_backgrounds_named_by_imagegroup_open_scenes() {
        // gridbg / largebg / verticalbg never carry `image`, only `imagegroup`.
        let cmds = vec![
            cmd("background", &[("image", "bg_a")], None, 1),
            say("Amiya", "one two", 2),
            cmd(
                "gridbg",
                &[("imagegroup", "bg_beach_1/bg_beach_2")],
                None,
                3,
            ),
            say("W", "three four", 4),
            // same strip again: not a new scene
            cmd(
                "largebg",
                &[("imagegroup", "bg_beach_1/bg_beach_2")],
                None,
                5,
            ),
            say("W", "five six", 6),
        ];
        let (turns, scene_bg) = turns_of(&cmds);
        let scenes: Vec<u32> = turns.iter().map(|t| t.scene).collect();
        assert_eq!(scenes, vec![1, 2]);
        assert_eq!(
            scene_bg.last().and_then(|(_, b)| b.clone()).as_deref(),
            Some("bg_beach_1/bg_beach_2")
        );
    }

    #[test]
    fn a_cg_is_an_overlay_not_a_scene_change() {
        let cmds = vec![
            cmd("background", &[("image", "bg_rhodes")], None, 1),
            say("Amiya", "before the cg", 2),
            cmd("image", &[("image", "avg_2_2")], None, 3),
            say("Amiya", "during the cg", 4),
            cmd("image", &[], None, 5),
            say("Kal'tsit", "after the cg", 6),
        ];
        let (turns, _) = turns_of(&cmds);
        assert!(
            turns.iter().all(|t| t.scene == 1),
            "a CG must not open a scene"
        );
        let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].background.as_deref(),
            Some("bg_rhodes"),
            "the background field must name the location, never the CG"
        );
    }

    #[test]
    fn a_cutin_joins_the_stage_without_clearing_it() {
        let cmds = vec![
            cmd("character", &[("name", "char_002_amiya")], None, 1),
            cmd("charactercutin", &[("name", "char_003_kalts")], None, 2),
            say("Kal'tsit", "one two", 3),
        ];
        let out = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert_eq!(
            out[0].on_screen,
            vec!["char_002_amiya".to_owned(), "char_003_kalts".to_owned()]
        );
    }

    #[test]
    fn packing_charges_sequence_overhead_and_joins() {
        // Four 3-word turns. With a 12-token ceiling and no overhead, all four
        // fit (12). Charging 2 per sequence and 1 per join, the real cost of
        // n turns is 2 + 3n + (n - 1), so only two fit (2 + 6 + 1 = 9; three
        // would be 13).
        let cmds = vec![
            say("Amiya", "a b", 1),
            say("W", "c d", 2),
            say("Amiya", "e f", 3),
            say("W", "g h", 4),
        ];
        let loose = ChunkConfig {
            target_min_tokens: 100,
            target_max_tokens: 12,
            ..ChunkConfig::default()
        };
        let exact = ChunkConfig {
            sequence_overhead_tokens: 2,
            joiner_tokens: 1,
            ..loose
        };
        assert_eq!(chunk_story("s", "g", &cmds, &loose, &words).len(), 1);
        let out = chunk_story("s", "g", &cmds, &exact, &words);
        assert_eq!(out.len(), 2);
        for c in &out {
            // stored count is the whole-text count plus the sequence overhead
            assert_eq!(c.token_count, words(&c.text) + 2);
            assert!(c.token_count <= 12);
        }
    }

    #[test]
    fn deterministic_across_runs() {
        let cmds = vec![say("W", "alpha bravo", 1), say("Hoederer", "charlie", 2)];
        let a = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        let b = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert_eq!(a, b);
    }
}
