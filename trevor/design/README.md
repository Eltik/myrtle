# Trevor design docs

Copies of the claude.ai project "Trevor (Myrtle)" docs as of 2026-09-25, plus the handoff. Newest and most authoritative first. When an older doc disagrees with a newer one, the newer one records the correction and the measurement behind it.

| doc | what it is |
|---|---|
| `HANDOFF.md` | State at handoff, what to check on the first gold-set run, the roadmap. Start here. |
| `conversation-log.md` | What Ian asked for and what was reported back, verbatim. |
| `trevor-goldset-automation.md` | The automated gold-set pipeline: filters, cross-family judge, calibration, anchor completion; what building it refuted. |
| `trevor-questions.md` | What people will ask: 16 question categories, where each answer lives (1.2M words of lore outside the index), a 10-question probe, the behaviors needed. |
| `trevor-updates.md` | What each stage does with a new or changed story (audit with file:line), and the plan for an incremental updater after asset updates. |
| `trevor-contract-v1.md` | The binding backend/worker/publish interface (2026-10-16): one gatePos spoiler scale and 22 eras, the horizon from the reader's profile, rungs and the Neuron ledger, worker auth, streaming, storage. |
| `trevor-reembed.md` | fp32 embedding everywhere and the guarded, resumable re-embed (threads, duty cycle, thermal/memory/battery pauses, checkpoint); measurements and the VPS command. |
| `trevor-vps-runbook.md` | Installing, updating, rolling back and monitoring Trevor on the VPS; what happens on each failure. |
| `trevor-integration-plan-v3.md` | The $0 build order (2026-10-16): what exists on the trevor branch, corrections to the overnight plans, phases A to E, Ian's open decisions. |
| `trevor-retrieval-baseline.md` | Search stack, eval harness, first dev-set numbers and what they can and cannot say. |
| `trevor-p0-status.md` | Phase 0 as built and as run on the Mac; the INT8-on-x86 finding; the Ettin head check. |
| `trevor-implementation-notes.md` | Verified API facts and corrections to the older specs (read section 1 before trusting them); real-corpus measurements in section 8. |
| `trevor-eval-spec.md` | The eval design: gold set, metrics, statistics, gating, tiers. |
| `trevor-goldset-generation.md` | The original gold-set generation spec. Its grammar does not parse and its trigram filter cannot fire; see the automation doc. |
| `trevor-corpus-spec.md` | The corpus build spec, phases P0 to P4, llama-server rules. Corrected in places. |
| `trevor-research.md` | The original research: why RAG and not fine-tuning, retrieval, hosting, budget. Partially superseded. |
| `trevor-p0-codex-brief.md`, `trevor-ingest-codex-brief.md` | Superseded briefs, kept for history. |
