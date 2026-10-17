# Trevor design docs

The design record of Trevor, newest and most authoritative first within each group. When an older doc disagrees with a newer one, the newer one records the correction and the measurement behind it. Start with the two contracts (the binding interfaces) and `trevor-questions.md` section 12 (the dated experiment log every flag cites).

## Current

| doc | what it is |
|---|---|
| `trevor-contract-v1.1.md` | Additions of 2026-10-17: capacity and the scaling model (answer once, admission control, fair claiming, scale-out), the worker power guard and Discord alerts, the master switch, pre-generated answers that follow game updates, the `mid` profile, the stories-page Ask tab, the real-worker dev preview. |
| `trevor-contract-v1.md` | The binding backend/worker/publish interface (2026-10-16): one gatePos spoiler scale and 22 eras, the horizon from the reader's profile, rungs and the Neuron ledger, worker auth, streaming, storage. |
| `trevor-vps-runbook.md` | Installing, updating, rolling back and monitoring Trevor on the VPS; what happens on each failure. |
| `trevor-reembed.md` | fp32 embedding everywhere and the guarded, resumable re-embed (threads, duty cycle, thermal/memory/battery pauses, checkpoint); measurements and the VPS command. |
| `trevor-integration-plan-v3.md` | The $0 build order (2026-10-16): what exists on the trevor branch, corrections to the overnight plans, phases A to E, Ian's open decisions. |
| `trevor-integration.md` | The first integration design (2026-10-07, a proposal when written), superseded by the plan v3 and the contracts; the backend module and migration v039 still cite it. |
| `trevor-questions.md` | What people will ask: 16 question categories, where each answer lives, a 10-question probe, the behaviors needed; section 12 is the dated experiment log. |
| `trevor-chronology.md` | In-world chronology (2026-09-26): what the text offers, the cue, extract, event and flashback stages, and their measurements. |
| `trevor-reading-order-research.md` | What the community recommends as a story reading order (2026-09-29); the source of `data/reading_community.json`. |
| `trevor-updates.md` | What each stage does with a new or changed story (audit with file:line), and the incremental updater after asset updates. |
| `trevor-goldset-automation.md` | The automated gold-set pipeline: filters, cross-family judge, calibration, anchor completion; what building it refuted. |
| `trevor-retrieval-baseline.md` | Search stack, eval harness, first dev-set numbers and what they can and cannot say. |
| `trevor-p0-status.md` | Phase 0 as built and as run on the Mac; the INT8-on-x86 finding; the Ettin head check. |
| `trevor-implementation-notes.md` | Verified API facts and corrections to the older specs (read section 1 before trusting them); real-corpus measurements in section 8. |
| `trevor-eval-spec.md` | The eval design: gold set, metrics, statistics, gating, tiers. |
| `trevor-corpus-spec.md` | The corpus build spec, phases P0 to P4, llama-server rules. Corrected in places. |

## Logs

| doc | what it is |
|---|---|
| `conversation-log.md` | What Ian asked for and what was reported back, verbatim. |

## Archive (`archive/`, superseded, kept for history)

| doc | what it is |
|---|---|
| `archive/HANDOFF.md` | The state at the 2026-09-25 handoff to Claude Code, what to check on the first gold-set run, the roadmap of then. |
| `archive/trevor-ian-questions-verified.md` | Ian's 32 questions verified against the wikis (2026-10-03): the source of the `verified` field in `eval/ian-questions.jsonl`. |
| `archive/review-sheet.md` | A one-off review form of the automatic judge, filled in by Ian (cited by the retrieval baseline). |
| `archive/trevor-improvement-research.md` | Research of 2026-09-28: which methods would raise answer quality on real questions, ranked by evidence. |
| `archive/trevor-goldset-generation.md` | The original gold-set generation spec. Its grammar does not parse and its trigram filter cannot fire; see the automation doc. |
| `archive/trevor-research.md` | The original research: why RAG and not fine-tuning, retrieval, hosting, budget. Partially superseded. |
| `archive/trevor-p0-codex-brief.md` | The phase 0 implementation brief, superseded by the code and `trevor-p0-status.md`. |
| `archive/trevor-ingest-codex-brief.md` | The ingest brief, implemented directly; `trevor-p0-status.md` section 4 lists where the code departs from it. |
