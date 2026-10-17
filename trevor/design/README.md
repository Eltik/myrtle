# Trevor design docs

The design record of Trevor, the local lore answerer for myrtle.moe. Where an older doc disagrees with a newer one,
the newer one records the correction and the measurement behind it. Start with the two contracts (the binding
interfaces), then the experiment log for why each default is what it is.

Backend environment variables are documented once, in the [backend README's Trevor table](../../backend/README.md#trevor).
Building and running the crate is covered in [../README.md](../README.md); installing on the VPS in
[../deploy/README.md](../deploy/README.md).

## Current

Interfaces and operations:

| doc | what it is |
|---|---|
| [trevor-contract-v1.md](trevor-contract-v1.md) | The binding backend, worker and publish interface (2026-10-16): one gatePos spoiler scale and 22 eras, the horizon from the reader's profile, rungs and the Neuron ledger, worker auth, streaming, storage. |
| [trevor-contract-v1.1.md](trevor-contract-v1.1.md) | Additions of 2026-10-17: capacity and admission control, fair claiming, the worker power guard and Discord alerts, the master switch, pre-generated answers that follow game updates, the `mid` profile, the stories-page Ask tab, the real-worker dev preview. |
| [trevor-vps-runbook.md](trevor-vps-runbook.md) | Installing, updating, rolling back and monitoring Trevor on the VPS, and what happens on each failure. |
| [trevor-updates.md](trevor-updates.md) | What each pipeline stage does with a new or changed story, and the incremental updater after asset updates. |
| [trevor-reembed.md](trevor-reembed.md) | fp32 embedding everywhere and the guarded, resumable re-embed (threads, duty cycle, thermal, memory and battery pauses, checkpoint), with measurements and the VPS command. |

Design and specs:

| doc | what it is |
|---|---|
| [trevor-integration-plan-v3.md](trevor-integration-plan-v3.md) | The $0 build order (2026-10-16): what exists, corrections to the earlier plans, phases A to E, the open decisions of the time. |
| [trevor-integration.md](trevor-integration.md) | The first integration design (2026-10-07), superseded by the plan v3 and the contracts where they differ; the backend module, migration v039 and the serve module still cite it. |
| [trevor-questions.md](trevor-questions.md) | What people ask: 16 question categories, where each answer lives, measured question mixes from Reddit and Discord, the first scored runs (sections 1 to 11). |
| [trevor-chronology.md](trevor-chronology.md) | In-world chronology (2026-09-26): what the text offers, the cue, extract, event and flashback stages, and their measurements. |
| [trevor-corpus-spec.md](trevor-corpus-spec.md) | The corpus build spec, phases P0 to P4, llama-server rules. Corrected in places by the implementation notes. |
| [trevor-implementation-notes.md](trevor-implementation-notes.md) | Verified API facts and corrections to the older specs (read section 1 before trusting them); real-corpus measurements in section 8. |
| [trevor-eval-spec.md](trevor-eval-spec.md) | The eval design: gold set, metrics, statistics, gating, tiers. |
| [trevor-goldset-automation.md](trevor-goldset-automation.md) | The automated gold-set pipeline: filters, cross-family judge, calibration, anchor completion, and what building it refuted. |
| [trevor-reading-order-research.md](trevor-reading-order-research.md) | What the community recommends as a story reading order (2026-09-29); the source of `data/reading_community.json`. |

## Logs

| doc | what it is |
|---|---|
| [experiment-log.md](experiment-log.md) | Every experiment from 2026-09-29 on, by topic (retrieval, routing, answer rules, entities and topics, chronology, design inference, embeddings, eval, serving): what was tried, the measured result, the decision and its kill switch. |
| [trevor-retrieval-baseline.md](trevor-retrieval-baseline.md) | The search stack, the eval harness, the first dev-set and gold-set numbers and what they can and cannot say (2026-09-24 onward). |
| [trevor-p0-status.md](trevor-p0-status.md) | Phase 0 as built and as run: the INT8-on-x86 finding and the Ettin reranker head check. |

## Archive (`archive/`, superseded, kept for history)

| doc | what it is |
|---|---|
| [archive/project-origins.md](archive/project-origins.md) | Requirements, research conclusions, corpus and API facts, the Phase 0 build and the early decisions (2026-09-22 to 2026-09-25). |
| [archive/HANDOFF.md](archive/HANDOFF.md) | The project state on 2026-09-25, what to check on the first gold-set run, and the roadmap of then. |
| [archive/trevor-probe-questions-verified.md](archive/trevor-probe-questions-verified.md) | The 32-question probe set verified against the wikis (2026-10-03): the source of the `verified` field in `eval/ian-questions.jsonl`. |
| [archive/review-sheet.md](archive/review-sheet.md) | A one-off hand review of the automatic judge (cited by the retrieval baseline). |
| [archive/trevor-improvement-research.md](archive/trevor-improvement-research.md) | Research of 2026-09-28: which methods would raise answer quality on real questions, ranked by evidence. |
| [archive/trevor-goldset-generation.md](archive/trevor-goldset-generation.md) | The original gold-set generation spec. Its grammar does not parse and its trigram filter cannot fire; see the automation doc. |
| [archive/trevor-research.md](archive/trevor-research.md) | The original research: why RAG and not fine-tuning, retrieval, hosting, budget. Partially superseded. |
| [archive/trevor-p0-codex-brief.md](archive/trevor-p0-codex-brief.md) | The phase 0 implementation brief, superseded by the code and `trevor-p0-status.md`. |
| [archive/trevor-ingest-codex-brief.md](archive/trevor-ingest-codex-brief.md) | The ingest brief, implemented directly; `trevor-p0-status.md` section 4 lists where the code departs from it. |
