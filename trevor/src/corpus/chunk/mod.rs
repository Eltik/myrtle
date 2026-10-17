//! Scene-aware chunking of Arknights story scripts.
//!
//! A chunk never splits a speaker turn: a fixed-size token window would strand a line of dialogue from the
//! `[name="..."]` that owns it, which makes the chunk unattributable. Not every background command is a boundary: a
//! `[Background]` that names no plate is a fade (the rules are at [`hard_boundary`]). `blocker` and `delay` are
//! intra-scene pacing and never boundaries; the invariant test asserts that adding or removing them cannot move a
//! chunk edge. `header` occurs exactly once per story, as its first command, so treating it as a boundary only opens
//! scene 1.
//!
//! Scenes are roughly chunk-sized, so the packer does both jobs: it merges short scenes up to the floor and cuts long
//! ones at turn boundaries under the ceiling. A chunk's token count includes the calibrated overhead (2 per sequence,
//! 1 per join), so the stored `token_count` equals a whole-chunk encode, and a story's chunks reconstruct its prose
//! exactly: nothing is lost and nothing is duplicated. The command frequencies and chunk-size distribution these rules
//! were derived from are in design/trevor-corpus-spec.md.

use std::collections::BTreeMap;


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
pub use crate::util::sha16;

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
mod tests;
