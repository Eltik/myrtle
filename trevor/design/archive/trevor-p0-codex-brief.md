# Codex brief: `trevor/src/corpus/chunk.rs`

> **SUPERSEDED 2026-09-24. Do not hand this to codex.** `chunk.rs` now exists in the
> scaffold, implemented directly and measured over the real corpus (implementation notes
> §8). This brief also carries two rules the measurement refuted: it tells the implementer
> that `[Image]` is a hard scene boundary (it is a CG overlay; that rule mislabeled 986
> chunks) and that breaking on every `[Background]` over-segments by roughly 2x (only 5.5%
> are fades). It is kept only as a record of the original design.

P0 does not fit in one brief. It decomposes into four fenced files, and this is the first
and hardest of them:

| file | why it is its own brief |
|---|---|
| **`trevor/src/corpus/chunk.rs`** | **pure, testable, has an exact precedent — this brief** |
| `trevor/src/corpus/ingest.rs` | HTTP + serde against a live backend; different failure modes |
| `trevor/src/corpus/embed.rs` | ONNX session lifecycle; needs the model on disk first |
| `trevor/src/store/ledger.rs` | SQLite schema; trivial, write by hand |

The crate scaffold (`Cargo.toml`, `src/lib.rs`, empty `src/corpus/mod.rs` with
`pub mod chunk;`) is written by hand before this runs. The brief below is self-contained
against that scaffold: `chunk.rs` defines its own input mirror types and depends on nothing
else in the crate.

---

## The brief

Copy everything below the line into `codex exec`.

---

Repo: /Users/eltik/Documents/Coding/myrtle. Rust crate: trevor. Build with `cargo build --release` and test with `cargo test --lib chunk` from trevor. Do NOT run git commit/checkout/reset/restore/stash/clean. Do not touch any file outside trevor/src/corpus/chunk.rs.

TASK: the Arknights story corpus must be split into retrieval chunks, and the two obvious approaches are both measurably wrong on this corpus. A fixed-size token window splits speaker turns, which strands a line of dialogue from the name that owns it and makes the chunk unattributable. Breaking on every `[Background]` command over-segments by roughly 2x, because a `Background` command with no `image` argument is a fade, not a scene change: a sample of 1,200 files shows 9,887 `Background` commands carrying `image` and the remainder carrying only `fadetime`/`block`. Breaking on `[Blocker]` or `[Delay]` is worse still: they occur 89,386 and 89,473 times across the corpus against `Background`'s 19,977, and they are intra-scene pacing. Implement scene-aware packing instead.

BACKGROUND — the working precedent, in backend/src/core/story/parser.rs:

- `pub struct StoryCommand` (line 52): `kind: String` (lowercased and typo-normalised), `args: BTreeMap<String,String>` (keys lowercased, values JSON-decoded), `text: Option<String>`, `line: u32` (1-based, in the source file).
- `pub const PROSE_KINDS: &[&str] = &["name", "text", "multiline", "narration"];` (line 63) — these carry prose in `c.text`.
- `pub const PROSE_ARG_KINDS: &[&str] = &["subtitle", "sticker"];` (line 66) — these carry prose in `c.args["text"]`.
- `pub fn word_count(commands: &[StoryCommand]) -> u32` (line 427) is the exact prose-extraction walk to mirror: `PROSE_KINDS` reads `c.text.as_deref()`, `PROSE_ARG_KINDS` reads `c.args.get("text")`, everything else yields `None`. Mirror that selection exactly. Do not invent a different prose predicate.
- Speaker attribution comes free from the grammar: `[name="Guard"] line` parses to `kind: "name"`, `args["name"] = "Guard"`, `text = "line"`. A bare text line parses to `kind: "text"` with the whole line in `text` and no `name` arg.

Command frequencies across all 4,860 EN script files, counted 2026-09-23 — these are the numbers the rules below are derived from:

    name 448650   charslot 222222   character 176564   delay 89473   blocker 89386
    dialog 80777  playsound 48144   background 19977   stopmusic 12426
    playmusic 10419  subtitle 9691  sticker 9053  predicate 7474  image 6692
    decision 6237  header 3356  multiline 1807  curtain 1308

Verified argument keys: `background` -> `image`, `screenadapt`, `block`, `fadetime`; `image` -> `image`; `charslot` -> `slot`, `name`, `focus`; `character` -> `name`, `name2`, `focus`; `decision` -> `options`, `values`; `header` -> `key`. IMPORTANT: `charslot`/`character` `name` is a sprite asset id such as `avg_npc_032`, NOT a display name. Do not conflate it with the speaker string from `[name="..."]`.

Corpus scale, for the assertions in step 6: 1,887 stories, 4,108,615 prose words. Roughly 20,000 background-delimited scenes, so the mean scene is about 205 words (~290 tokens), which is SMALLER than the chunk target. Scenes are packed, not split.

WHAT TO IMPLEMENT in trevor/src/corpus/chunk.rs:

1. Define the input mirror types at the top of the file. They deserialize the `commands` array of `GET /api/story/{story_id}`, which is camelCase over the wire:

   ```rust
   /// Mirror of `backend::core::story::parser::StoryCommand` as it arrives over
   /// `GET /api/story/{id}`. Duplicated rather than shared because trevor is a
   /// sibling crate with no path dependency on backend, exactly as discord/ is.
   #[derive(Debug, Clone, serde::Deserialize)]
   #[serde(rename_all = "camelCase")]
   pub struct StoryCommand {
       pub kind: String,
       #[serde(default)]
       pub args: std::collections::BTreeMap<String, String>,
       #[serde(default)]
       pub text: Option<String>,
       pub line: u32,
   }
   ```

2. Define the output type:

   ```rust
   /// One retrieval chunk. `line_start`/`line_end` are the 1-based source-file
   /// lines of its first and last PROSE commands, and they are the anchor the
   /// evaluation gold set references. Chunk ids churn whenever chunking is
   /// retuned; line spans do not, so the gold set survives retuning and the
   /// eval keeps measuring the thing being changed.
   #[derive(Debug, Clone, serde::Serialize)]
   #[serde(rename_all = "camelCase")]
   pub struct Chunk {
       pub chunk_id: String,        // "{story_id}#{ordinal:04}"
       pub story_id: String,
       pub group_id: String,
       pub ordinal: u32,
       pub scene_ordinal: u32,      // index of the scene this chunk STARTS in
       pub line_start: u32,
       pub line_end: u32,
       pub text: String,
       pub speakers: Vec<String>,   // display names, deduped, in first-appearance order
       pub on_screen: Vec<String>,  // sprite asset ids from charslot/character
       pub background: Option<String>,
       pub token_count: u32,
       pub content_sha: String,     // sha256 of `text`, first 16 hex chars
   }

   #[derive(Debug, Clone, Copy)]
   pub struct ChunkConfig {
       pub target_min_tokens: u32,   // 400
       pub target_max_tokens: u32,   // 600
       pub max_scenes_per_chunk: u32,// 3
       pub overlap_turns: u32,       // 0
   }
   impl Default for ChunkConfig { /* the values above */ }
   ```

3. Scene segmentation. Walk the command stream once and assign every command a
   `scene_ordinal` starting at 0, incrementing on a HARD boundary:

   - `kind == "background"` AND `args` contains `image` AND that value differs from the
     current background. CRITICAL: a `background` command with NO `image` key is a fade and
     is NOT a boundary; it must not increment the ordinal and must not clear the current
     background.
   - `kind == "image"` AND `args` contains `image` (full-screen CG).
   - `kind == "header"`.

   Soft boundaries — record them as preferred split points for step 5 but do NOT increment
   `scene_ordinal`: `kind == "decision"`, `kind == "curtain"`.

   `blocker`, `delay`, `dialog`, `playsound`, `playmusic`, `stopmusic`, `charslot`,
   `character`, `predicate` are NEVER boundaries of either kind.

4. Turn grouping. A turn is a maximal run of prose commands owned by one speaker:
   - A `kind == "name"` command OPENS a turn; its speaker is `args["name"]` (which may be
     the empty string).
   - A following prose command that is not `kind == "name"` CONTINUES the open turn.
   - A prose command with no open turn (the file starts with bare text) opens an
     unattributed turn with speaker `None`.

   Render a turn as `format!("{speaker}: {joined}")` when the speaker is a non-empty
   string, and as `joined` alone when the speaker is `None` or empty. `joined` is the turn's
   prose lines joined with `" "`. IMPORTANT: the speaker string goes in the TEXT, not only
   in metadata — lexical retrieval has to be able to match it.

5. Packing. Within a story, accumulate turns into a chunk in order:
   - Never split a turn across chunks.
   - Close the chunk when adding the next turn would exceed `target_max_tokens`.
   - When the chunk is at or above `target_min_tokens` AND the next turn begins a new scene
     or follows a soft boundary, close it there in preference to running on.
   - Close the chunk unconditionally when it already spans `max_scenes_per_chunk` scenes and
     the next turn begins another.
   - Never span two stories.
   - A story whose total prose is below `target_min_tokens` yields exactly one chunk.
   - A SINGLE turn longer than `target_max_tokens` becomes its own oversized chunk. Do not
     split it; record it and let the caller see the count.
   - `overlap_turns` repeats that many trailing turns at the head of the next chunk. Default
     0, and with 0 the output must contain no repeated turn text at all.

6. Metadata per chunk. `speakers` is the deduped display names of its turns in
   first-appearance order, with the empty string excluded. `on_screen` is the set of
   `args["name"]` values from `charslot`/`character` commands whose `line` falls inside
   `[line_start, line_end]`. `background` is whatever background image was current when the
   chunk opened. `token_count` uses the tokenizer passed in by the caller.

7. Signature. Keep the file pure — no I/O, no HTTP, no ONNX:

   ```rust
   pub fn chunk_story(
       story_id: &str,
       group_id: &str,
       commands: &[StoryCommand],
       cfg: &ChunkConfig,
       count_tokens: &dyn Fn(&str) -> u32,
   ) -> Vec<Chunk>
   ```

   The tokenizer arrives as a closure so this file never links `tokenizers`. Tests pass a
   whitespace-word counter.

INVARIANCE GUARANTEE. `chunk_story` must be a pure deterministic function of
`(story_id, group_id, commands, cfg)`: the same input produces byte-identical output,
including `content_sha`, across runs and machines. It must hold, and be asserted in tests,
that: concatenating every chunk's `text` in ordinal order reproduces the story's full prose
in order with no loss and (at `overlap_turns == 0`) no duplication; no chunk's line span
overlaps another's; `line_start <= line_end` on every chunk; and every chunk's `ordinal`
equals its index. A command that is neither prose nor a boundary contributes metadata only
and can never change the chunk boundaries — so adding, removing or reordering `delay`,
`playsound` or `blocker` commands leaves the chunk text byte-identical.

CONSTRAINTS. No new files. Unit tests in a `#[cfg(test)] mod tests` at the bottom of the
same file, covering at minimum: a `Background` with no `image` does not open a scene; a
repeated identical `background` image does not open a scene; a turn is never split; an
oversized single turn survives whole; a sub-minimum story yields one chunk; the
concatenation invariant. Comments explain WHY, not the diff, and no comment may reference
"the old code" or "previously". `cargo build --release`, `cargo fmt --check` and
`cargo clippy -- -D warnings` must be clean for what you introduce; pre-existing warnings
elsewhere are fine.

WHEN DONE, print the final diff scoped to trevor/src/corpus/chunk.rs, the test output, and
confirmation that build, fmt and clippy are clean. Print the diff, not a summary of it.
