# SUPERSEDED: Codex brief for `ingest.rs`

This brief was implemented directly on 2026-09-24 and is no longer a task. The code is at `myrtle/trevor/src/corpus/ingest.rs` and `src/bin/build-corpus.rs`. See `trevor-p0-status.md`, section 4, for where the implementation departs from the brief, including one instruction here that was wrong: counting special tokens on every per-turn encode put 26.1% of chunks over the 600-token ceiling.
