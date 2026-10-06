#!/usr/bin/env bash
# Bring Trevor up to date after a game asset update, redoing only what changed (design/trevor-updates.md).
#
# Every model stage skips a unit whose id, input sha and prompt sha all match (scripts/incr.py; deaths.py
# keys the same way), so an unchanged story costs nothing and a changed story carries through: its
# summary changes, so its group's input changes, and so on, with no dependency graph to keep. Vectors
# are reused for every chunk whose embedded text is unchanged (embed-corpus), so an update re-embeds
# only new or edited chunks instead of 28 min for all 13,837.
#
# Needs the myrtle backend on :3060 (Ian starts it; this script never does). One model is loaded at a
# time; the server and the thermal logger stop on any exit. Every step resumes where it stopped.
#   PLAN=1               print what each model stage would redo, load no model, write only this run's log
#   SKIP_FETCH=1         skip build-corpus (use the current chunks.jsonl), e.g. after a gamedata-only update
#   SKIP_BANK_ANSWERS=1  skip `ask` over new bank questions (18.6 s each)
#   SKIP_P4_MODELS=1     skip step 4b (topic summaries and IS ending canon over P4), the script before 2026-10-01
#   REFRESH_REFERENCE=1  refresh the wiki EVAL REFERENCES under eval/reference (network; off). Never served or built
#                        from (Ian, 2026-10-04: the wiki is a reference Trevor's own deductions are scored against)
#   DEEP_TOP=N           deep topic entries (scripts/topic_deep.py, 2026-10-03 night 9) for the N most asked topics (40)
# Since 2026-10-01 (evening) step 4b mines topic candidates from the game tables and the corpus and classifies new ones on
# Gemma (scripts/topics.py mine, classify; TOPIC_SOURCE=v1 = the 46 topics alone); p3b/p4 are built from the 200 v1
# dossiers and 46 topics, and ask --lore v2 adds the others at answer time.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/llama.sh  # stop_pid, await_health
BASE="${BASE:-http://127.0.0.1:3060}"
PORT="${PORT:-8081}"
GEN=models/llm/gemma-4-12b-it-qat-q4_0.gguf
JUDGE=models/llm/Qwen3.5-9B-Q4_K_M.gguf
TOK=models/gte-modernbert-base/tokenizer.json
LOG=artifacts/update/$(date +%Y%m%d-%H%M%S)
mkdir -p "$LOG"
SERVER_PID=""; THERM_PID=""
cleanup() {
  stop_pid "$SERVER_PID"; stop_pid "$THERM_PID"
  SERVER_PID=""; THERM_PID=""
}
trap cleanup EXIT INT TERM
T0=$(date +%s)
step() { echo "== $(date +%H:%M:%S) [$(( $(date +%s) - T0 ))s] $*" | tee -a "$LOG/steps.log"; }

# 1. Corpus: refetch every story, replace only changed ones (changes.json lists them).
if [[ -z "${SKIP_FETCH:-}" && -z "${PLAN:-}" ]]; then
  code=$(curl -s -o /dev/null -w '%{http_code}' "$BASE/" || true)
  if [[ "$code" == "000" ]]; then
    echo "The backend does not answer at $BASE. Start it (cd ../backend && ./target/debug/backend), then rerun." >&2
    exit 1
  fi
  step "build-corpus --refresh"
  target/release/build-corpus --tokenizer "$TOK" --refresh
  python3 -c "import json; c=json.load(open('artifacts/changes.json')); print({k: len(v) if isinstance(v, list) else v for k, v in c.items()})" | tee -a "$LOG/steps.log"
fi

# 2. Deterministic corpus dirs and indexes; vectors reused where the text is unchanged.
if [[ -z "${PLAN:-}" ]]; then
  step "embed + index P0";   target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts
  target/release/build-index --corpus artifacts
  step "archives (p3a)";     target/release/build-archives
  target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p3a --reuse-from artifacts
  target/release/build-index --corpus artifacts/p3a
fi

# 3. Model stages on Gemma. With PLAN=1 each prints what it would redo and exits without a server.
run_models() {
  step "P2 story summaries";   python3 scripts/p2-summaries.py story
  step "P2 group summaries";   python3 scripts/p2-summaries.py group
  # Cue files are deterministic and extract recomputes its cue lines itself; PLAN leaves them alone.
  [[ -n "${PLAN:-}" ]] || { step "chronology cues"; python3 scripts/chrono.py cues; }
  step "chronology extract";   python3 scripts/chrono.py extract
  step "chronology events";    python3 scripts/chrono.py events
  [[ -n "${PLAN:-}" ]] || { step "identity cues"; python3 scripts/entities.py cues; }
  step "identity extract";     python3 scripts/entities.py extract
  step "death extract";        python3 scripts/deaths.py extract
  step "death check";          python3 scripts/deaths.py check
  step "real names";           python3 scripts/real_names.py gen
  step "dossiers";             python3 scripts/dossiers.py gen
  step "primers";              python3 scripts/primers.py gen
  step "bank questions";       python3 scripts/answer_bank.py questions
}
# 4b's plans: topics (their own keys, as incr.py's) and canon (the stamp of its last run, scripts/canon_refs.py inputs).
plan_p4_models() {
  step "topics mine";  python3 scripts/topics.py mine
  step "topics classify"; python3 scripts/topics.py classify
  step "topics plan";  python3 scripts/topics.py plan
  step "topics gen";   python3 scripts/topics.py gen
  step "topics judge"; python3 scripts/topics.py judge
  step "topics deep";  python3 scripts/topic_deep.py gen --top "${DEEP_TOP:-40}"; python3 scripts/topic_deep.py judge
  step "design infer"; python3 scripts/design_infer.py gen; python3 scripts/design_infer.py check
  step "design tiles"; python3 scripts/art_captions.py design_tiles
  step "canon";        python3 scripts/canon_refs.py inputs | sed '$d'
}
if [[ -n "${PLAN:-}" ]]; then
  export PLAN
  run_models
  [[ -n "${SKIP_P4_MODELS:-}" ]] || plan_p4_models
  exit 0
fi
( while true; do
    echo "$(date +%H:%M:%S) therm: $(pmset -g therm | grep -i level | grep -v 'No ' | tr '\n' ' ')pressure $(sysctl -n kern.memorystatus_vm_pressure_level)"
    sleep 900
  done ) >> "$LOG/thermal.log" &
THERM_PID=$!
while pgrep -x llama-server >/dev/null; do echo "waiting for another llama-server to exit"; sleep 30; done
llama-server -m "$GEN" --host 127.0.0.1 --port "$PORT" -np 2 --kv-unified-per-slot 16384 \
  -fa on -cram 512 --no-webui --reasoning off -ngl all > "$LOG/server.log" 2>&1 &
SERVER_PID=$!
await_health "$SERVER_PID" continue
export SERVER="http://127.0.0.1:$PORT"
run_models
if [[ -z "${SKIP_BANK_ANSWERS:-}" ]]; then
  step "bank answers";       python3 scripts/answer_bank.py answer
fi
cleanup

# 4. Deterministic rebuilds over the updated model outputs.
step "chronology derive/flashbacks/build/history"
python3 scripts/chrono.py derive; python3 scripts/chrono.py flashbacks
python3 scripts/chrono.py build;  python3 scripts/chrono.py history
# Main episode EN release dates from the game data (2026-10-04, no network): deaths' release order and the guide read them.
step "main episode dates";  python3 scripts/reading_guide.py dates | tee -a "$LOG/steps.log"
step "identities, deaths";  python3 scripts/entities.py build; python3 scripts/deaths.py build
step "operator attributes"; python3 scripts/attributes.py
# Eval references only, off by default so an update stays offline and nothing served depends on the wiki (2026-10-04):
# REFRESH_REFERENCE=1 refreshes the wiki answer keys under eval/reference (trivia pages for design_infer.py score,
# the deceased list and operator statuses for deaths.py eval, the main episode dates for reading_guide.py score).
if [[ "${REFRESH_REFERENCE:-0}" == "1" && -z "${PLAN:-}" ]]; then
  step "wiki eval references (network)"
  python3 scripts/design_basis.py all; python3 scripts/deaths.py wiki; python3 scripts/wiki_status.py
  python3 scripts/reading_guide.py wiki
fi
# Since 2026-10-02 both lores (ask --lore v1|v2) retrieve from these corpora, built from the 200 v1 dossiers and the 46
# v1 topics (dossiers.v1.jsonl, units.v1.jsonl: byte for byte the files before); v2 adds a new topic through the topic
# tool and a new dossier by the character's name, at answer time. Built into P3b (2026-10-01, artifacts/p3b-lore) the
# 336 new dossiers changed the passages of 50 of 120 real-probe questions and lost 5.
build_lores() {
  python3 scripts/dossiers.py sets
  target/release/build-profiles --dossiers artifacts/dossiers/dossiers.v1.jsonl
  target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p3b --reuse-from artifacts/p3a
  target/release/build-index --corpus artifacts/p3b
  # P4 also holds Trevor's topic units (since 2026-09-30); build-units alone would drop them.
  cat artifacts/sources/units.jsonl artifacts/topics/units.v1.jsonl > artifacts/sources/all_units.jsonl
  target/release/build-units --units artifacts/sources/all_units.jsonl
  target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p4 --reuse-from artifacts/p3b
  target/release/build-index --corpus artifacts/p4
  # P4 plus the game text the 2026-10-03 coverage audit found unindexed (scripts/gametext.py), which ask reads for P4
  # routes by default (--no-game-text reads P4). GAMETEXT=0 skips it, as before 2026-10-03.
  if [[ "${GAMETEXT:-1}" != "0" ]]; then
    python3 scripts/gametext.py
    target/release/build-units --corpus artifacts/p3b --units artifacts/gametext/p4x_units.jsonl --out artifacts/p4x
    target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p4x --reuse-from artifacts/p4
    target/release/build-index --corpus artifacts/p4x
  fi
}
step "typed sources (p4: modules, voice lines, IS, enemies, outfits, items)"
python3 scripts/sources.py
step "profiles and typed sources (p3b, p4)"
build_lores
step "reading guide"; python3 scripts/reading_guide.py build
step "answer bank"; PRIMERS=1 python3 scripts/answer_bank.py seed; python3 scripts/answer_bank.py merge

# 4b. Model stages over P4 (2026-10-01): topic summaries, then the IS ending canon, each only when its inputs changed.
# One model at a time; IS6's reasoning-on verdict needs a 24,576-token slot (design/trevor-questions.md section 12).
serve() {  # serve MODEL [llama-server args]: start one server on $PORT and wait for it
  while pgrep -x llama-server >/dev/null; do echo "waiting for another llama-server to exit"; sleep 30; done
  llama-server -m "$1" --host 127.0.0.1 --port "$PORT" -np 1 --kv-unified-per-slot "${KV:-16384}" \
    -fa on -cram 512 --no-webui -ngl all "${@:2}" >> "$LOG/server.log" 2>&1 &
  SERVER_PID=$!
  await_health "$SERVER_PID" exit
}
unserve() { stop_pid "$SERVER_PID"; SERVER_PID=""; }
planned() { PLAN=1 python3 "$@" | grep -o 'plan [0-9]*' | head -1 | cut -d' ' -f2; }
if [[ -z "${SKIP_P4_MODELS:-}" ]]; then
  export SERVER="http://127.0.0.1:$PORT"
  # Candidates from the game tables and the corpus (deterministic), each classified once by Gemma (keyed), then the plan.
  step "topics mine"; python3 scripts/topics.py mine | tee -a "$LOG/steps.log"
  if [[ "$(planned scripts/topics.py classify)" != "0" ]]; then
    step "topics classify (Gemma)"; serve "$GEN" --reasoning off; python3 scripts/topics.py classify; unserve
  fi
  step "topics plan"; python3 scripts/topics.py plan
  if [[ "$(planned scripts/topics.py gen)" != "0" ]]; then
    step "topics gen (Gemma)"; serve "$GEN" --reasoning off; python3 scripts/topics.py gen; unserve
  fi
  if [[ "$(planned scripts/topics.py judge)" != "0" ]]; then
    step "topics judge (Qwen)"; serve "$JUDGE" --reasoning off; python3 scripts/topics.py judge | tee -a "$LOG/steps.log"; unserve
  fi
  if [[ "$(planned scripts/topics.py retrait)" != "0" ]]; then
    step "topics retrait (Gemma)"; serve "$GEN" --reasoning off; python3 scripts/topics.py retrait; unserve
  fi
  # The race-trait rule (2026-10-03, no model) is served by default (ask --no-race-trait-rule): refresh it from summaryV2.
  python3 scripts/topics.py traitrule | tail -1 | tee -a "$LOG/steps.log"
  python3 scripts/topics.py thin | tee -a "$LOG/steps.log"
  cp artifacts/topics/units.jsonl "$LOG/topic-units.before.jsonl" 2>/dev/null || true
  cp artifacts/topics/units.v1.jsonl "$LOG/topic-units.v1.before.jsonl" 2>/dev/null || true
  python3 scripts/topics.py units
  if ! cmp -s artifacts/topics/units.jsonl "$LOG/topic-units.before.jsonl" || ! cmp -s artifacts/topics/units.v1.jsonl "$LOG/topic-units.v1.before.jsonl"; then
    step "topic units changed: rebuild p3b, p4"
    build_lores
  fi
  # Deep topic entries (2026-10-03 night 9; served by ask since 2026-10-05 for whole-topic questions, --no-deep-topics): map-reduce over the evidence naming each of the DEEP_TOP
  # (40) most asked topics, read from p4x; topic_deep.py keys each evidence set and entry, so only changed ones are redone.
  if [[ "$(planned scripts/topic_deep.py gen --top "${DEEP_TOP:-40}")" != "0" ]]; then
    step "topics deep gen (Gemma)"; serve "$GEN" --reasoning off; python3 scripts/topic_deep.py gen --top "${DEEP_TOP:-40}"; unserve
  fi
  if [[ "$(planned scripts/topic_deep.py judge)" != "0" ]]; then
    step "topics deep judge (Qwen)"; serve "$JUDGE" --reasoning off; python3 scripts/topic_deep.py judge | tail -1 | tee -a "$LOG/steps.log"; unserve
  fi
  # Design inspirations inferred from the game data (2026-10-04, ask's design_basis tool): keyed on each operator's
  # evidence and the prompt, so only new or changed operators are redone; Gemma infers, Qwen checks the cited evidence.
  if [[ "$(planned scripts/design_infer.py gen)" != "0" ]]; then
    step "design infer gen (Gemma)"; serve "$GEN" --reasoning off; python3 scripts/design_infer.py gen; unserve
  fi
  if [[ "$(planned scripts/design_infer.py check)" != "0" ]]; then
    step "design infer check (Qwen)"; serve "$JUDGE" --reasoning off; python3 scripts/design_infer.py check; unserve
  fi
  # Tiled design probe (2026-10-05): per operator's E2 art (E0 without one) the 12B vision model reads the 2x2
  # full-resolution tiles for creatures, animals, plants and objects; rows keyed on the PNG's sha256 + prompt hash, so
  # only new or changed art is read (about 33 s an image). TILES=0 skips it. Then the deduction evidence of
  # ask --design-deduce is rebuilt (no model, under a second).
  if [[ "${TILES:-1}" == "1" ]] && [[ "$(planned scripts/art_captions.py design_tiles)" != "0" ]]; then
    step "design tiles (Gemma + mmproj)"
    KV=8192 serve "$GEN" --reasoning off --mmproj models/llm/mmproj-gemma-4-12b-it-qat-q4_0.gguf -b 2048 -ub 2048 \
      --image-min-tokens 1120 --image-max-tokens 1120
    ART_THREADS=1 python3 scripts/art_captions.py design_tiles; unserve
  fi
  python3 scripts/design_infer.py evidence | tee -a "$LOG/steps.log"
  # Not served (2026-10-04 evening): no version met the bar of held-out precision 0.600 against the wiki answer key
  # (v1 checked 17/35 = 0.486), so the table is built beside the served path; DESIGN_SERVE=1 builds the served one.
  if [[ "${DESIGN_SERVE:-0}" == "1" ]]; then
    python3 scripts/design_infer.py build | tee -a "$LOG/steps.log"
  else
    DESIGN_OUT=artifacts/entities/design_infer.unserved.jsonl python3 scripts/design_infer.py build | tee -a "$LOG/steps.log"
  fi
  canon=$(python3 scripts/canon_refs.py inputs | tee -a "$LOG/steps.log" | tail -1)
  if [[ "$canon" != "none" ]]; then
    step "canon: $canon run"
    if [[ "$canon" == "full" ]]; then
      # Every resume file is keyed by id, so a full run starts from empty files; the old ones are kept beside.
      mkdir -p "$LOG/canon-before"; find artifacts/canon -maxdepth 1 -name '*.json*' ! -name inputs.json -exec mv {} "$LOG/canon-before/" \;
      python3 scripts/canon_refs.py endings
    else
      python3 scripts/canon_refs.py prune
    fi
    serve "$GEN" --reasoning off
    for s in summarize judge wide undated discriminate "story gemma"; do python3 scripts/canon_refs.py $s; done
    unserve; serve "$JUDGE" --reasoning off
    for s in second "story qwen" "verdict qwen-off"; do python3 scripts/canon_refs.py $s; done
    unserve; KV=24576 serve "$GEN" --reasoning on --reasoning-budget 3000
    python3 scripts/canon_refs.py verdict gemma-on
    unserve
    python3 scripts/canon_refs.py build | tail -6 | tee -a "$LOG/steps.log"
    python3 scripts/canon_refs.py stamp
  fi
fi

step "routing regression set"; python3 scripts/route-check.py | tail -1 | tee -a "$LOG/steps.log"

# 5. Report: the retrieval score on gold set v2 against the last recorded run (a regression check, not a gate).
step "retrieval check (gold v2, hybrid, p3a)"
target/release/eval run --goldset eval/goldset_v2.jsonl --corpus artifacts/p3a --mode hybrid --label "update $(date +%F)" \
  --out eval 2>&1 | tail -4 | tee -a "$LOG/steps.log"
step "done; log in $LOG"
