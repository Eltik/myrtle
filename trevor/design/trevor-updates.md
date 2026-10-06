# Trevor after an asset update

Ian (2026-09-28): "I want to be able to do this scalably, so eventually having a script that retrains Trevor whenever there is an asset update quickly would be nice." Trevor is not trained; an update re-ingests, re-embeds, re-indexes and re-extracts. This doc records what each stage does today when a story is new or changed, and the plan for an updater that redoes only what changed. **Built 2026-09-28:** `scripts/update.sh` (the plan below), `build-corpus --refresh`, vector reuse in `embed-corpus`, and content-hash keys on every model stage (`scripts/incr.py`). The audit table records the state before that build.

## The rule

Every per-unit stage skips a unit only when its key matches `(unit id, sha of the unit's input text, sha of the prompt and grammar)`, the key `scripts/deaths.py` uses (`deaths.py` extract and check). A changed story then changes its chunks' text sha and is redone; an edited prompt redoes everything it touches; an unchanged story costs nothing. Whole-corpus stages that take seconds (BM25, chronology build, bank merge) simply rerun.

## Audit, 2026-09-28 (read from the code)

| stage | skips on | changed story | new story | full rerun |
|---|---|---|---|---|
| build-corpus (`ingest.rs` recover, lines 256 to 297, filter 604 to 608) | storyId | kept stale | picked up | minutes, needs the backend on :3060 |
| embed-corpus | nothing, rewrites all | redone | embedded | about 28 min |
| build-index (BM25) | nothing; readers refuse a stale `chunks_sha` | redone | redone | 2.4 s |
| build-prefixes, build-archives, build-profiles | nothing; record `source_chunks_sha`, never checked | redone | redone | seconds |
| p2-summaries story, group | storyId; (groupId, method); `promptSha` stored, not compared | stale | picked up | about 6 h stories, 75 min groups |
| chrono extract, place, exceptions, events | storyId, groupId, (source, line) | stale | picked up | model-timed, hours |
| chrono cues, build, derive, flashbacks, history | full recompute | inherits upstream | inherits upstream | seconds |
| entities extract | (chunkId, line) | stale | picked up | model-timed |
| dossiers gen | character name | stale | picked up | about 54 min for 200 |
| primers gen | groupId | stale | picked up | not timed |
| deaths extract, check | (chunkId, textSha, promptSha) | **redone** | picked up | 1.1 s per passage in the pilot, so about 1.5 h for all 4,700 death-word passages |
| answer_bank questions | chunkId | stale | picked up | 1.7 s per question; answers 18.6 s each |

Readers are safe: `bm25.rs:79` and `dense.rs:44` refuse an index whose `chunks_sha` does not match the corpus, so a stale index errors rather than answering. Nothing rebuilds it automatically.

## Gaps an updater must close, in order

1. **build-corpus resumes on storyId**, so an update that only edits existing text is missed at the first stage. It needs the backend's per-story content hash (or a refetch of every story's text, then a diff) instead of "id already present".
2. **embed-corpus re-embeds all 13,837 chunks** (28 min). Each chunk already carries `content_sha`; reusing the vector of any chunk whose sha is unchanged makes an update of one event cost seconds. Safe because one text is embedded per ONNX call, so a vector does not depend on its batch-mates.
3. **The model stages skip on ids** (P2, chrono extract, entities, dossiers, primers, bank questions). Each needs the deaths key: add the unit's text sha to the skip key. Most already store `promptSha`; they never compare it.
4. **Dependencies.** A changed story invalidates its group summary, the primers whose prior context includes its group, the dossiers of characters who speak in it, and the bank entries gated on it. The updater computes the set of changed story ids first and derives these from the existing files (group membership, `dossier.stories`, primer `prior`, bank `gateStories`).

## The updater, as planned

`scripts/update.sh`, one model loaded at a time, every step resumable:
1. Refuse to start unless the backend answers on :3060 (Ian starts it; Claude never does).
2. build-corpus with content-hash change detection; write the changed and new story ids.
3. embed-corpus reusing unchanged vectors; build-index; build-archives and build-profiles.
4. Gemma: P2 story summaries for changed stories, then group summaries for touched groups; chrono extract for changed stories; entities extract for changed chunks; deaths extract; dossiers for touched characters; primers for touched targets.
5. Qwen: deaths check; spot judges.
6. Deterministic: chrono build, deaths build, bank seed and merge.
7. Report: stories added and changed, time per step, and the gold-set retrieval score against the last run (a regression check, not a gate).

A typical monthly event (one new group, 10 to 20 stories, about 150 chunks) should then cost minutes of embedding and well under an hour of generation, against about 10 hours for a rebuild from scratch.

## As built (2026-09-28)

- `build-corpus --refresh` refetches every story, replaces the chunks of stories whose content changed, adds new ones, drops ones the backend no longer lists, and writes `artifacts/changes.json` (added, changed, removed, unchanged, failed) plus `chunks.prev.jsonl`. With nothing changed, `chunks.jsonl` is not rewritten. It refuses `--limit`. A test against a fake backend shows resume missing an edited story and refresh producing a file byte-identical to a fresh build. Not yet run against the real backend.
- `embed-corpus` reuses the vector of every chunk whose embedded text is unchanged (key: sha of the text embedded, equal to `contentSha` when no prefix is embedded, checked on every row of P0, p3a, p3b), writes `contentShas` into the meta, and `--reuse-from DIR` borrows another dir's vectors (p3a from P0). `--no-reuse` is the old full embed. On 30 real chunks every reuse case was byte-identical to a full embed (edited corpus: 26 reused, 5 embedded, 1.0 s against 4.1 s). The three live dirs were rewritten through reuse at 100% (13,837, 15,762, 16,050 reused, under 1 s each) and tier A golden stayed identical on P0 and p3a.
- `scripts/incr.py`: each model stage keys a unit on (id, sha of the exact input it sends, prompt sha), replaces a redone unit's row in place, and drops duplicates and vanished units. `PLAN=1` prints what would be redone and why without a server; `KEY_IDS_ONLY=1` restores id-only skipping. Existing rows were backfilled with their current input sha (row counts unchanged; the backups were deleted on 2026-09-29 after the check). A one-word edit to one chunk flagged exactly: that story's summary, its group's B and C summaries (A reads the blurbs only), one exception check, one dossier, the one primer whose prior context holds it, and that chunk's bank question.
- Bank questions keep their sample after the first build (a redraw would move most of the 900 picks when a group is added); chunks of stories new since `question_pool.json` are drawn at 900/13,954 by a hash of the chunk id. `BANK_RESAMPLE=1` restores the redraw.
- State now, from `PLAN=1 scripts/update.sh`: every stage 0 to do except dossiers, 50, because identity links and the timeline were rebuilt after the dossiers were written (Kal'tsit's still lists "Raidian"); the first real update regenerates them (about 14 min). `chrono place` (81 stale) is outside the updater; only `chrono eval` reads it.
- Not run end to end: the real refresh needs the backend, which only Ian starts.
