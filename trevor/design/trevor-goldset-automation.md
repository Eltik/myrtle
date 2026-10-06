# Trevor: automated gold-set generation

Written 2026-09-25. Replaces the human review step of `trevor-goldset-generation.md` with checks that run unattended, and records what the build of it found. Code: `src/goldgen/`, `src/bin/goldset.rs`, `prompts/`, `scripts/goldset-run.sh` in `myrtle/trevor/`.

## 1. How review is automated

The human reviewer did three jobs: reject bad questions, confirm the gold passage answers the question, and add passages the question also matches. Each now has an automatic replacement.

- **Deterministic filters, no model.** Unparsed or truncated output; generator abstention; evidence quote not found verbatim in the passage (after folding curly quotes, dashes and whitespace); deictic phrasing ("the speaker", "this scene", a pronoun with no name anywhere in the question); copied wording (more than 30% of the question's word trigrams found in the passage); BM25 ranking the source first (single-passage items); near-duplicates (question embeddings with cosine above 0.95).
- **A judge from a different model family.** Gemma 4 12B writes; Qwen3.5 9B judges, so no model grades its own work. Every verdict is binary under a one-line grammar, because small judges hold 89.55% on binary checks but fall to Spearman 0.21 on graded ones. Checks: is the question self-contained; does the source passage answer it; for multi-hop, does each passage alone fail to answer it; is the generator's closed-book answer correct.
- **Anchor completion.** The top 5 from BM25, dense and hybrid are pooled for each selected question, and the judge's "does this passage answer it" verdict decides which become extra gold. This is the "missed anchors" work, done for every item instead of the 40 most contested.
- **The judge is measured, not trusted.** `goldset calibrate` asks the judge about 60 questions paired with a random passage from another story (every "yes" is a false positive, reported with a Wilson interval), about the same questions with their true source, and about 10 known-bad and 10 known-good questions. The result goes into the gold set's meta file.
- **Optional spot-check.** `eval/goldset_v1.spotcheck.md` lists 20 random items. Marking them takes about 10 minutes and bounds the automatic review's error rate (0 rejections in 20 means at most 16%, Wilson 95%). Nothing blocks on it.

Closed-book gate: if the generator answers more than 30% of selected questions correctly without context, selection re-runs preferring questions it could not answer (the spec's first remediation), and both rates are recorded.

Unanswerable items: 17 hand-written candidates; one is kept only if no pooled passage is judged to answer it, up to 15. Deferred: ambiguous-entity (10) and spoiler-boundary (5), which need hand construction.

## 2. Found while building it

- **The spec's grammar does not parse.** llama.cpp GBNF ends a rule at a newline unless the newline is inside parentheses, so the multi-line `root ::=` in `trevor-goldset-generation.md` section 4 is rejected ("expecting name"). Wrapped in parentheses in `prompts/*.gbnf`.
- **The spec's trigram filter cannot fire.** Character-trigram Jaccard divides by the union, and a 2,300-character passage dominates it: measured values sit near 0.05 to 0.16 whatever the question copies, so a 0.30 threshold never trips. Replaced with word-trigram containment (share of the question's trigrams found in the passage); the Jaccard value is still recorded per item for comparison.
- **Multi-hop pairs now need different groups**, not just different stories: two chapters of one event share context, so a question across them is barely two-hop.
- **Model choice was set by the Mac, not by quality.** Gemma 4 26B-A4B Q4_0 is 14.4 GB; the Mac has 24 GB with 7.6 GB of swap already in use and 58 GB free disk. Gemma 4 12B QAT Q4_0 (7.0 GB) and Qwen3.5 9B Q4_K_M (5.7 GB) load one at a time. Both are pinned by revision and sha256 in `fetch-model.sh` (`WITH_LLM=1`).
- llama.cpp is Homebrew 0.5.0 (build 11146); installing it also upgraded `openssl@3` to 3.6.4. `--reasoning off` renders Qwen's template with an empty think block, checked.

## 3. Status

The whole pipeline ran end to end in the cloud with a 0.8B stand-in model: sample, generate, filter, closed-book, judge, finish, and the output passes `eval validate`. That tested the plumbing only; the tiny model's questions are nonsense. Both real models are downloaded and verified on the Mac. Not yet run there.

Expected Mac time, unmeasured: about 280 generations, a closed-book pass, roughly 700 judge calls, and roughly 1,500 anchor-completion calls, likely around an hour.
