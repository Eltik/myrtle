# Trevor: `build-corpus` implementation spec

> **Superseded in places.** `claude/trevor-implementation-notes.md` (2026-09-23) corrects
> ten claims in this document against verified APIs, including the llama-server flag set
> below and the embedding pooling mode. Read it first. The corrections are inlined here
> where they are short enough; the notes carry the evidence.

Companion to `claude/trevor-research.md`. This specifies the offline Mac-side binary that
produces every artifact the VPS serves. Nothing in here runs in a request path.

**Sequencing note that overrides the research doc's ordering.** The research says build the
retrieval eval first. You cannot evaluate retrieval without an index, so the real order is
**P0 of this spec (about one hour), then the eval harness, then P1 onward with every phase
measured against the eval**. P1 through P4 are each a change whose value should be a number,
not an assumption.

---

## 0. Measured facts this spec is built on

Command census over all 4,860 EN script files, counted 2026-09-23:

| command | count | role here |
|---|---|---|
| `name` | 448,650 | the prose carrier |
| `charslot` | 222,222 | who is on screen |
| `character` | 176,564 | sprite placement |
| `delay` | 89,473 | pacing, ignore |
| `blocker` | 89,386 | fade, too frequent to be a scene break |
| `dialog` | 80,777 | text-box clear |
| `playsound` | 48,144 | ignore |
| **`background`** | **19,977** | **the scene boundary** |
| `stopmusic` / `playmusic` | 12,426 / 10,419 | weak scene signal |
| `subtitle` / `sticker` | 9,691 / 9,053 | prose in `args["text"]` |
| `predicate` | 7,474 | branch condition |
| `image` | 6,692 | full-screen CG |
| `decision` | 6,237 | **branching choice, a natural boundary** |
| `header` | 3,356 | title card |
| `multiline` | 1,807 | prose |
| `curtain` | 1,308 | transition |

> **Corrected 2026-09-24 by measurement** (implementation notes §8). The table above is a raw
> regex census from before the unpacker fix and double-counted some stories; the backend's
> own parser over the 1,862 distinct stories gives `background` 15,039, `blocker` 65,619,
> `delay` 65,758, `image` 5,082, `header` 1,862. The paragraph that stood here claimed
> scenes average ~290 tokens and are therefore packed rather than split. **That was wrong.**
> Measured: 13,700 scene boundaries against 13,396 chunks, so scenes are roughly chunk-sized
> and the packer does both jobs, merging short scenes and cutting long ones at turn
> boundaries.

From `backend/src/core/story/parser.rs`, verified:

```rust
pub const PROSE_KINDS: &[&str] = &["name", "text", "multiline", "narration"];
pub const PROSE_ARG_KINDS: &[&str] = &["subtitle", "sticker"];   // prose in args["text"]
pub struct StoryCommand { pub kind: String, pub args: BTreeMap<String,String>,
                          pub text: Option<String>, pub line: u32 }
pub struct StoryScript  { pub id: String, pub name: String, pub group_id: String,
                          pub commands: Vec<StoryCommand>, pub assets: StoryAssets,
                          pub word_count: u32 }
```

`line` is 1-based in the source file. That field is load-bearing for section 3.

---

## 1. Crate placement

Sibling crate `trevor/`, following the `discord/` precedent exactly: own `Cargo.toml` and
lock, own `Dockerfile`, own `.env`, own `ecosystem.config.cjs`, no workspace membership. Add
a `DO_TREVOR` branch to `vps-update.sh` and a case to `pm2_ecosystem_for()` (line 363), and
a port to `vps-start.sh`'s health list.

```
trevor/
  Cargo.toml
  src/
    lib.rs
    corpus/    ingest.rs  chunk.rs  scene.rs  context.rs  summarize.rs  entity.rs  embed.rs
    store/     artifacts.rs  ledger.rs  manifest.rs
    llm/       client.rs  grammar.rs  prompts/
    eval/      (see the eval spec)
  src/bin/
    build-corpus.rs
    eval.rs
  artifacts/   (gitignored; the build output)
  prompts/     (versioned, hashed into job ids)
```

`build-corpus` is a **Mac-side tool**. It must not be in the VPS binary's dependency graph:
no `llama-cpp-2`, no model loading, just `reqwest` to a local `llama-server`.

---

## 2. Ingestion

Consume `GET /api/story/index` and `GET /api/story/{story_id}` from a locally running
backend. Do not re-read `.txt.txt` files. `core::story` already handles the swapped-path
probe. *(Corrected 2026-09-24: the swap was fixed on disk by the unpacker change of
2026-09-23; 0 of the 1,862 resolving stories now come from `[uc]info/`. The probe stays for
trees extracted before the fix.)*
the 16-entry typo table, backslash continuations, JSON-decoded argument values, and speaker
attribution. Re-implementing `parser.rs` is 670 lines of re-derived bugs.

Fallback path only when the backend is unreachable: read the files directly and reproduce
the probe (take whichever path's first non-whitespace byte is `[`). Log loudly when this
fires; a silent fallback that half-works is the worst outcome.

Record for every story: `story_id`, `group_id`, `StoryCode`, `StoryName`, `AvgTag`,
`StorySort`, `RequiredStages`, and the group's `EntryType` / `ActType`. **25 of 1,887
`StoryTxt` paths resolve to nothing (1.3%)**, and they are not scattered: they are two whole
events that were never extracted, `act24side` "A Flurry to the Flame" (19 of 19) and
`act13mini` "It's Been A While" (6 of 6), with zero matching files on disk. Emit them to
`artifacts/unresolved.jsonl` rather than failing the build, and assert the count has not
grown. Until the extraction gap is fixed the lore model will not know those stories exist.

---

## 3. Chunk identity, and the decision that saves the gold set

**Chunks are not stable across chunking changes. Gold sets must be.** If the gold set
references `chunk_id`, then every chunking experiment invalidates the eval that is supposed
to measure it, and you have built a ratchet that prevents the one thing you wanted to tune.

So: **the gold set anchors on `(story_id, line_start, line_end)`**, using `StoryCommand.line`
from the parser. Those line numbers are stable as long as the underlying script file is
unchanged, which is to say stable except when the game itself patches a story. At eval time
a chunk counts as gold if its line span overlaps the anchor span. Chunking can then change
freely and the gold set survives.

Chunk identity for everything else:

```
chunk_id     = "{story_id}#{ordinal:04}"     // ordinal = position within the story
content_sha  = sha256(prefix || text)[..16]  // change detection, not identity
line_start   = first prose command's line
line_end     = last prose command's line
```

Emit `artifacts/chunks.jsonl`:

```json
{"chunk_id":"main_08-14_end#0007","story_id":"...","group_id":"main_8","ordinal":7,
 "line_start":412,"line_end":455,"scene_ordinal":3,"text":"...","prefix":null,
 "speakers":["Amiya","Kal'tsit"],"on_screen":["char_002_amiya","char_003_kalts"],
 "background":"bg_rhodes_infirmary","token_count":487,"content_sha":"..."}
```

---

## 4. Scene segmentation and chunking

**Scene boundaries**, in priority order:

- **Hard**: any of the backend's five `BACKGROUND_KINDS` (`background`, `backgroundtween`,
  `largebg`, `gridbg`, `verticalbg`) naming a plate that differs from the current one. The
  plate is `image` on the first two and **`imagegroup` only** on the strip kinds (86, 60 and 17
  uses, never `image`). Also `header`, which occurs exactly once per story as its first
  command.
- **NOT `image`.** *(Corrected 2026-09-24.)* This spec originally listed `[Image]` as a hard
  boundary. It is a CG overlay that fades in over the scene, per `docs/story-reader.md`, and
  treating it as a scene change left a CG name in the `background` field of 986 of 13,439
  chunks (7.3%).
- **Soft**: `decision` and `curtain`. *(The `playmusic`/`stopmusic` pair rule that stood here
  was never implemented or measured, and is dropped.)*
- **Not a boundary**: `blocker` (89,386) and `delay` (89,473). They are far too frequent and
  mostly intra-scene pacing. Using `blocker` as a break is the obvious mistake here.

**Chunking**, given scenes averaging ~290 tokens:

1. Extract prose commands only: `PROSE_KINDS` reading `c.text`, `PROSE_ARG_KINDS` reading
   `c.args["text"]`. Everything else contributes metadata, not text.
2. Group consecutive prose into **speaker turns**: a `name` command starts a turn, following
   `text` / `multiline` commands with no intervening `name` continue it.
3. Render each turn as `Speaker: line` with the speaker string verbatim, so BM25 can match
   it. Unnamed narration renders bare.
4. Pack turns into a chunk, **never splitting a turn**, until `token_count` reaches
   **[400, 600]**. Prefer to close a chunk at a hard scene boundary when within ±25% of
   target. Allow a chunk to span up to 3 scenes; never span a story.
5. A story shorter than 400 tokens is one chunk.
6. **Overlap 0 by default.** Chroma measured that reducing overlap improves IoU, one 2026
   study found overlap gives no benefit at all to sparse retrieval, and OpenAI's 800/400
   default has "particularly poor recall-efficiency tradeoffs". Make it a config knob at
   0-10% and let the eval decide; do not assume.
7. Token counting uses `tokenizers = "0.23.2"` with **the same `tokenizer.json` the embedder
   uses**. A mismatched tokenizer silently truncates chunks. Ship it as a build artifact.

Expected output: **roughly 9,800 to 14,000 chunks** at 4.11M words and ~420 words per chunk.
**Measured 2026-09-24: 13,396 chunks** (chars/4 token proxy; p10 309, p50 541, p90 595; 15.4%
under 400, 0.5% over 600, largest 1,513). Record the actual number with the real tokenizer;
it is a golden-file assertion.

Masked-speaker conventions are **not entities**: `???` appears on 5,777 lines, plus `""` and
`?`. Keep them in the text, exclude them from the speaker index.

---

## 5. Embedding

**Model: `gte-modernbert-base`** (149M, 768-dim, 8,192-token context), INT8 dynamic-quantized
ONNX, **CLS pooling** (its `1_Pooling/config.json` sets `pooling_mode_cls_token: true`;
mean-pooling it would silently degrade retrieval). The official repo already ships
`onnx/model_int8.onnx` at 150,218,016 bytes plus all four tokenizer files, so **no Optimum
export or quantization step is needed** and the ModernBERT export traps never arise. Chosen over EmbeddingGemma-300m because the 30k WordPiece vocab costs ~25 MB RSS
against Gemma's 256k vocab at ~150 MB, which matters on a box that has already been
OOM-killed, and because it is the same ModernBERT lineage as the Ettin reranker, so one
tokenizer family and one export path. EmbeddingGemma remains the alternative if you want
Matryoshka truncation to 256 dims (matrix 43.0 MB -> 14.3 MB); take it only if RSS
measurements say you need to.

Output `artifacts/vectors.bin`: row-major `f32`, L2-normalized **at build time** so cosine
collapses to a bare dot product at query time. Plus `artifacts/vectors.meta.json` carrying
`{model, model_sha, dim, count, chunk_ids[], built_at, tokenizer_sha}`. 14,000 x 768 f32 is
**43.0 MB**; an exact SIMD scan over it is **~1.5 ms** at recall 1.0, so there is no index,
no ANN crate and no pgvector.

Embedding the whole corpus on the M5 Pro GPU is **15 to 30 minutes**. This is the cheapest
phase and the one you will re-run most often.

---

## 6. Phases, each resumable

Every phase writes artifacts plus `artifacts/manifest.{phase}.json`, and every unit of work
gets a ledger row. Ledger is SQLite `artifacts/build.db`:

```sql
CREATE TABLE jobs (
  job_id      TEXT PRIMARY KEY,   -- sha256(phase || input_sha || prompt_sha || model_sha)
  phase       TEXT NOT NULL,
  unit_id     TEXT NOT NULL,      -- story_id, chunk_id, group_id, question_id
  status      TEXT NOT NULL,      -- pending | running | done | failed
  output_path TEXT,
  tokens_in   INTEGER, tokens_out INTEGER,
  engine_build TEXT, model_sha TEXT, prompt_sha TEXT,
  started_at  INTEGER, finished_at INTEGER, error TEXT
);
```

`job_id` is a pure function of its inputs, so a re-run skips completed work and a changed
prompt template re-runs exactly what it invalidates. **A crash costs one unit, not the run.**

| phase | output | gen tokens | hours |
|---|---|---|---|
| **P0** ingest + chunk + embed | `chunks.jsonl`, `vectors.bin`, `spoiler.jsonl` | 0 | **~1** |
| **P1** contextual prefixes, re-embed | `chunks.jsonl` with `prefix`, new vectors | 1.4M | ~3.5 |
| **P2** summaries | `scenes.jsonl`, `stories.jsonl`, `groups.jsonl` | 1.2M | ~3.0 |
| **P3** entity index + dossiers | `entities.jsonl`, `dossiers.jsonl` | 0.8M | ~2.0 |
| **P4** answer bank seed (5,000) | `answers.jsonl` | 0.8M | ~2.0 |
| prefill across all passes | — | — | ~8.3 |

**Do not enumerate 50,000 answer-bank questions.** At 150 tokens each that is 7.5M tokens and
**19 hours, 62% of the entire build**. Seed 5,000 and grow from real miss logs, which is both
cheaper and better targeted than any enumeration you would guess at. V1 is P0 through P2,
about **10 hours, one night**.

---

## 7. Driving `llama-server`

HTTP via `reqwest` to `127.0.0.1:8080`. No in-process bindings: `llama-cpp-2` is at 0.1.156
releasing every 1-3 weeks with an undocumented llama.cpp pin, and keeping it out of the build
graph buys process isolation and restart-without-recompile for free.

Rules that are not optional:

- **The system prompt must be byte-identical across every call in a phase.** Prefix caching
  is the single biggest lever on a 5:1 input-heavy job, and slot matching needs the bytes to
  match. Hash it into `prompt_sha` and assert it never varies mid-phase.
- `"cache_prompt": true` (it is already the default). **Not `-cram -1`:** the host prompt
  cache already defaults to 8192 MiB and that default is the documented top cause of
  "llama-server leaks RAM" reports; on 24 GB use `-cram 1024`, and `-cram 0` on the
  long-context instance. **Not `--cache-reuse 256`** either: it reuses KV across *mid-prompt*
  edits via KV shifting, and every divergence here is at the end of the prompt, which the
  plain longest-common-prefix already handles.
- **GBNF grammar on every structured output.** JSONSchemaBench measured constrained decoding
  *improving* downstream accuracy by up to 4 points with no observed declines (GSM8K 80.1%
  unconstrained -> 82.4% with llama.cpp). Keep schemas flat: objects, enums, bounded arrays,
  no `$ref` chains, no `oneOf`, no regex `pattern`.
- **Restart the server every ~2,000 requests.** There is a reported batch-mode memory leak,
  unverified on Metal, and mmap makes a reload cost under 2 seconds.
- Two tiers, two ports: bulk work on **Gemma 4 26B-A4B Q4** (port 8080, `-np 4 -c 131072
  -ctk q8_0 -ctv q8_0`), and the 135k-token passes plus anything shipped verbatim on
  **Qwen3.5-9B Q8_0** (port 8081, `-np 1 -c 139264`, fp16 KV). The second tier exists because
  4-bit quantization loses **up to 23% at 128k tokens** while 8-bit loses 0.2-0.8%.

---

## 8. P1: contextual prefixes

For each chunk, generate ~100 tokens prepended to the text **before both embedding and BM25
indexing**. Anthropic measured failure rate (1 - recall@20) falling **5.7% -> 3.7%** with
contextual embeddings and **-> 2.9%** adding contextual BM25.

The prefix must contain, in this order: group name and `StoryCode`; position in the arc
(story N of M); scene location derived from the `background` asset name; speakers present
from `charslot` / `character`; **resolution of who "he", "she", "they" and epithets like
"the Doctor" refer to in this chunk**; and one clause of what is happening.

The referent resolution is the point. NoLiMa measured questions with literal lexical overlap
scoring **98.5% at 32k context** against latent-association questions at **56.2%**.
Screenplay dialogue is the worst case for latent association, and the prefix is what converts
one into the other.

Prompt shape: system prompt carries the instruction and output grammar and never changes;
user turn carries the story synopsis (from P2 if available, else the story's own summary
line) plus the chunk. Cache the synopsis prefix across all chunks of a story by ordering the
work story-by-story, never chunk-by-chunk across stories.

**Measure P1 against P0 with the eval before keeping it.** It costs 3.5 hours and a full
re-embed; it should show up as a recall@10 improvement with a paired CI that excludes zero.

---

## 9. P2: summaries, flat not deep

Three levels, all embedded **into the same flat index as the leaf chunks**. RAPTOR found the
collapsed tree consistently beat tree traversal, and measured NarrativeQA **+7.3 ROUGE-L**
and QuALITY **+20.3 points**. Do not build a deep recursive tree; the corpus has no such
depth.

- **Scene summaries**, ~20,000 units, 1-2 sentences each. Feeds the contextual prefix.
- **Story synopses**, 1,887 units, ~150 words. This is the reader-enrichment deliverable.
- **Group summaries**, 451 units, ~400 words, plus a separate "what you need to know before
  starting this" variant. The median group is 3,042 words so most fit in context whole;
  `main_14` at 93,221 words (~135k tokens) is the only one needing the Tier-2 model and a
  two-pass map-reduce over its 41 stories.

Add a **coverage counter** to every summary: faithfulness metrics are one-sided, and a
summary that says nothing scores 100% faithful. Sample salient source claims by entity
frequency and record what fraction the summary covers. Faithfulness plus coverage, or the
metric is gameable.

---

## 10. P3: entity index and dossiers

**The speaker index is free and exact.** 5,808 distinct speaker strings are structured
metadata, not extraction output, so the speaker-to-chunk inverted index has zero extraction
error and zero build cost. This is why you do not build a knowledge graph: on NovelQA's
detail subset plain RAG scored **55.28% against Community-GraphRAG's 46.88%**, and on STAGE's
screenplay QA **Hybrid RAG 70.2% Pass@5 against GraphRAG 52.0%**.

For entities *mentioned but not speaking*, use a **gazetteer, not an NER model**. Build it
from `character_table` (names and appellations), `handbook_info_table`, `enemy_handbook_table`
and `zone_table`, match case-insensitively with the same regex tokenizer the lexical index
uses. That keeps the whole pipeline in Rust with no Python dependency, and for a closed
corpus with a known cast it will beat general-purpose NER on exactly the invented proper
nouns that matter.

Dossiers: one per major entity, generated from its chunk set ordered chronologically, with a
hard instruction to cite `story_id` for every claim. These answer "who is Kal'tsit" without
live retrieval.

`spoiler.jsonl` is built in P0 and carries `{story_id, group_id, story_sort, required_stages,
global_ordinal}`. The join key for progress gating is `user.storyreview.groups[gid].stories[]
.id`, which is the same `StoryId`, so the filter is `WHERE story_id = ANY($read_ids)` and not
an inference from stage clears.

---

## 11. Determinism and golden files

Assert on every build, and fail loudly on drift:

- chunk count, token-count histogram deciles, and the count of stories yielding zero chunks
- `unresolved.jsonl` length is still 25
- distinct speaker count is still 5,808
- the SHA of `chunks.jsonl` with the `prefix` field stripped, so chunking changes are visible
  independently of generation changes
- every dossier and summary citation `story_id` exists in the index

Run the whole pipeline at `temperature 0, top_p 1, top_k 1, batch 1`. Note that temperature 0
is **not** deterministic across batch sizes: 1,000 completions at temperature 0 from
Qwen3-235B produced **80 unique outputs**, first diverging at token 103, because of
batch-size-dependent floating-point reduction order. Batch 1 removes the dominant source for
free; the generated text is still not bit-reproducible across engine builds, which is why the
golden files above are all on the deterministic layer.

---

## 12. Pre-flight, before any multi-hour run

1. **The open M5 Metal correctness bug.** llama.cpp PR #28748 is still open as of 2026-09-18:
   when a tensor slice offset reaches 2 GB the Metal Tensor API returns an address off by
   -4 GB from int32 wrap, producing **wrong matmuls and invalid embeddings**, reproduced on
   M5. A 16-17 GB model reaches that boundary. Run a fixed 200-token prompt at temp 0 with
   and without `GGML_METAL_TENSOR_DISABLE=1` and diff. Require build **b10734 or newer**
   (2026-09-01), which fixed M5 startup failures on macOS 26.2-26.4.
2. **Does `-np` scale here?** `llama-batched-bench` at 1, 2, 4, 8. The 2.9x aggregate figure
   measured on an M5 Pro is contested by a five-backend comparison that found everything flat.
   Ten minutes, and it decides whether the budget is 10 hours or 17.
3. **Quality gate.** Run 30 representative stories through Tier-1 Q4 and Tier-2 Q8 and read
   both outputs yourself. No public benchmark isolates abstractive summarization faithfulness
   for these models; that gap is real and only your own reading closes it.

Session setup: `sudo pmset -a powermode 2 disablesleep 1` (measured **+52% to +89%** on dense
models on M5 Pro), `sudo sysctl iogpu.wired_limit_mb=19456`, `caffeinate -dimsu`. Expect
**8-15% sustained thermal derate** on the 14" chassis. Nothing here leaves a daemon running
after the build exits.
