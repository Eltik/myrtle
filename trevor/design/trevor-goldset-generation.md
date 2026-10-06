# Trevor: gold-set generation, prompts and grammar

Implements §2 of `claude/trevor-eval-spec.md`. Everything here runs on the Mac against a
local `llama-server`. Output is `eval/goldset_v1.jsonl`, frozen and reviewed in PRs.

**Why generate rather than hand-write.** Synthetic questions are measured valid for exactly
this use: Kendall τ between synthetic-benchmark and human-benchmark rankings is **0.44 to
0.75 for retriever configurations**. The same study found τ ranging **-1.00 to +1.00** when
synthetic sets were used to compare generators, so nothing generated here may be used to
judge answer quality. Retrieval only.

Budget: 150 target items at 1.5x oversample is 225 generations at ~80 output tokens each,
about 18,000 tokens. Minutes, not hours. The cost is the human review pass, which is 2 to 3
hours.

---

## 1. Request shape

```
POST http://127.0.0.1:8080/completion
{
  "prompt":        "<system>\n\n<user>",
  "grammar":       "<contents of prompts/goldset.gbnf>",
  "temperature":   0.7,
  "top_p":         0.95,
  "seed":          <deterministic per item: fnv1a(chunk_id)>,
  "n_predict":     200,
  "cache_prompt":  true,
  "stop":          []
}
```

Temperature 0.7 here, not 0. Diversity is the point for question generation, and LIMA's
result is that diversity beats volume (scaling quality-filtered data 2K -> 16K plateaued).
The seed is derived from `chunk_id` so a rerun reproduces the same set. Everything
downstream of generation is deterministic.

**The system prompt must be byte-identical across all 225 calls.** Hash it into
`prompt_sha` in the ledger and assert it never varies mid-phase; prefix caching depends on
the bytes matching, and this is a 5:1 input-heavy job.

---

## 2. System prompt

Store as `trevor/prompts/goldset.system.txt`. Do not edit it without bumping the gold set
version, because a changed prompt is a changed dataset.

```
You write evaluation questions for a search system over the story scripts of the
video game Arknights.

You will be given one passage of dialogue from one story. Write one question that a
player might genuinely ask, which can be answered using that passage.

Rules, all of which matter:

1. The question must be SELF-CONTAINED. A reader who has never seen the passage must
   be able to understand what is being asked. Name the people, places and events
   explicitly. Never write "this passage", "the above", "here", "this scene", "the
   speaker", "he" or "she" without an antecedent inside the question itself.

2. Do NOT reuse the passage's wording. Ask the question in your own words, using
   different phrasing from the text. If the passage says "the Originium arts backfired",
   do not ask "what happened when the Originium arts backfired"; ask about the
   consequence in different terms.

3. The question must have a definite answer that the passage supports. Do not ask for
   opinion, speculation, or anything the passage only hints at.

4. Do not ask about formatting, line numbers, who is on screen, or anything about the
   script as a document. Ask about the story.

5. Quote, verbatim and exactly as it appears, the shortest span of the passage that
   answers the question.

Classify the question into exactly one stratum:
  factoid     - a single fact stated in one line
  entity      - about a named character, faction, place or object
  causal      - why something happened, or what motivated someone
  temporal    - ordering, before/after, or the first or last time something occurs
  aggregation - something that requires collecting several statements in the passage

Answer with a single JSON object and nothing else.
```

---

## 3. User turn

```
STORY: {story_name}  ({story_code}, {group_name})

PASSAGE:
{chunk_text}
```

Deliberately minimal. Do not include the story synopsis: the question must be answerable
from the passage alone, and a synopsis in context invites questions the passage cannot
support, which is exactly the class the leakage filter then has to throw away.

---

## 4. Grammar

`trevor/prompts/goldset.gbnf`. Flat object, enum, bounded strings, no `$ref`, no `oneOf`,
no regex `pattern` — JSONSchemaBench measured llama.cpp GBNF at 39% coverage on hard
schemas against Outlines' 3%, and the coverage cliff is entirely about schema complexity.
Constrained decoding also measured as *improving* downstream accuracy by up to 4 points
with no observed declines, so this is not a quality tax.

```gbnf
# One evaluation question derived from one passage.
# Whitespace is fixed, not free, so the output shape is byte-stable and parses
# with serde_json without a tolerant reader.

root ::=
  "{\n" 
  "  \"question\": "                 qstring ",\n"
  "  \"stratum\": "                  stratum ",\n"
  "  \"answerable_from_passage\": "  bool    ",\n"
  "  \"evidence_quote\": "           estring "\n"
  "}"

stratum ::= "\"factoid\"" | "\"entity\"" | "\"causal\"" | "\"temporal\"" | "\"aggregation\""

# 20-180 chars keeps questions from rambling; 10-300 bounds the evidence span.
qstring ::= "\"" qchar{20,180} "\""
estring ::= "\"" qchar{10,300} "\""

qchar   ::= [^"\\\x00-\x1F] | "\\" (["\\bfnrt/] | "u" hex hex hex hex)
hex     ::= [0-9a-fA-F]
bool    ::= "true" | "false"
```

If the `{m,n}` repetition form is unsupported on your llama.cpp build, drop the bounds to
`qchar*` and enforce the lengths in the filter instead. Check once with
`llama-server --grammar-file` on a throwaway prompt before running the batch.

---

## 5. Filter pipeline

Run in this order; each stage is cheap and each removes work from the next.

| # | filter | threshold | rationale |
|---|---|---|---|
| 1 | grammar | — | shape guaranteed at decode time |
| 2 | `answerable_from_passage == false` | reject | the model's own abstention |
| 3 | `evidence_quote` not a substring of the passage after whitespace normalization | reject | free, deterministic grounding check; catches fabricated evidence |
| 4 | deictic terms in question | reject | `this passage`, `the above`, `here`, `this scene`, `the speaker`, and a leading pronoun with no antecedent |
| 5 | **character-trigram Jaccard(question, passage) > 0.30** | reject | the chunk-wording leak |
| 6 | **BM25 over the full index ranks the source chunk at 1** | reject | too extractive to distinguish retrievers |
| 7 | duplicate question (cosine > 0.95 against accepted set) | reject | oversampling produces near-duplicates |
| 8 | human review | accept / rewrite / reject | ~15 min per 50 |

Trigram definition, so the threshold is reproducible: lowercase, strip everything outside
`[a-z0-9 ]`, collapse runs of whitespace, take the set of character 3-grams of each string,
and compute `|A ∩ B| / |A ∪ B|`.

Stage 6 is the important one and the one people skip. If BM25 alone already ranks the gold
chunk first for more than **60%** of the surviving set, the set cannot tell a good retriever
from a bad one and must be regenerated with stronger stage-5 pressure.

---

## 6. The strata the generator cannot produce

Three strata are built differently, because generating them from a single passage produces
the wrong thing.

**Multi-hop (n=30).** Sample two chunks from *different stories* sharing a gazetteer entity,
and require a question answerable only from both. Same grammar; the user turn carries both
passages labeled `PASSAGE A` / `PASSAGE B`, and the system prompt gains one rule:

```
6. The question must require BOTH passages. A reader with only one of them must not be
   able to answer it. Do not write two questions joined by "and".
```

Anchors are the union of both chunks' line spans. Add a filter: reject if either passage
alone entails the evidence quote.

**Unanswerable (n=15). Hand-write these.** They are questions about things the corpus
genuinely does not say, and a model asked to invent them will produce either nonsense or
something the corpus quietly does answer. This stratum is where models collapse — measured
performance gaps of **13.6 to 68.4 pp**, with Phi-3-medium falling from 75.8% to 7.4% — so
it is worth the hour it takes to write fifteen by hand.

**Spoiler boundary (n=5 here, expanded later).** Also hand-written, one per declared
progress boundary, each built around a specific post-boundary fact that is recorded
alongside the question as a spoiler claim for the entailment check in eval spec §8.3.

---

## 7. The closed-book control

Run after filtering, before freezing. Separate prompt, no passage, no retrieval:

```
Answer this question about the video game Arknights in one sentence. If you do not
know, reply exactly: I don't know.

QUESTION: {question}
```

Grade it binary against `evidence_quote` with the local entailment model
(`premise = model answer`, `hypothesis = evidence_quote`), not with a generative judge. Small
generative judges reach **89.55%** on binary verdicts but collapse to **Spearman ρ = 0.21**
on graded quality, so keep every judgment in this harness binary.

Record `closed_book.correct` per item and the pooled rate in the gold set header.

**The gate: if pooled closed-book accuracy exceeds 30%, stop.** The eval is measuring the
model's pretraining knowledge of Arknights rather than your retrieval. On standard
benchmarks GPT-5 scored **HotpotQA 50%, 2WikiMultihopQA 62%, QASC 75%** with no context at
all, and removing that leakage widened the spread between competing RAG methods from **3% to
10-18%** — before removal the benchmark could barely rank them.

Arknights has a large wiki footprint in pretraining data, so treat this number as a **model
selection criterion**, not only an eval control. There is a real tension here worth stating
rather than resolving silently: a model that knows Arknights writes more fluent answers,
*and* invalidates your eval, *and* makes spoiler gating impossible because it can confabulate
post-boundary facts from weights. Measure it per candidate model before choosing one.

Remediation when the gate trips, in order of preference: drop items whose answers are
wiki-famous and keep the obscure ones; shift the stratum mix toward `causal` and
`aggregation`, which are far less likely to be memorized than `factoid` and `entity`; and
only as a last resort re-scope to stories with the thinnest wiki coverage.

---

## 8. Output

```json
{"qid":"g0042",
 "question":"Why does Kal'tsit refuse to explain the Doctor's condition?",
 "stratum":"causal",
 "anchors":[{"story_id":"main_08-14_end","line_start":412,"line_end":455}],
 "evidence_quote":"You aren't ready to hear it, and I am not ready to say it.",
 "source_chunk_id":"main_08-14_end#0007",
 "generated_by":"gemma-4-26b-a4b-q4@<sha>","prompt_sha":"<sha>","seed":2847361092,
 "filters":{"trigram_jaccard":0.11,"bm25_rank":4,"deictic":false},
 "closed_book":{"model":"gemma-4-26b-a4b-q4@<sha>","correct":false,"checked_at":"2026-09-24"},
 "reviewed_by":"ian","review":"accepted"}
```

Header line carries `{"goldset_version":"v1","built_at":…,"n":150,"closed_book_pooled":0.14,
"strata":{…},"system_prompt_sha":…,"grammar_sha":…}`.

Freeze it. Review it in PRs like source. **Never regenerate it in CI** — a gold set that
rebuilds itself is optimized into uselessness within a month.
