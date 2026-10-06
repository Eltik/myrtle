//! The command line: every flag of `ask`, and the gates derived from more than one flag (which router
//! runs, which corpus and source rules apply).

use std::path::PathBuf;

use clap::Parser;
use trevor::router;
use trevor::search::runtime::RuntimeArgs;
use trevor::tools::{Route, Tools};

use crate::detect::source_kind;

#[derive(Parser, Clone)]
#[command(about = "Answer an Arknights lore question from retrieved passages")]
pub(crate) struct Args {
    /// The question (omit with --batch).
    pub(crate) question: Option<String>,
    #[command(flatten)]
    pub(crate) runtime: RuntimeArgs,
    #[arg(long, default_value = "http://127.0.0.1:8081")]
    pub(crate) server: String,
    /// Retrieved passages before neighbour expansion.
    #[arg(long, default_value_t = 8)]
    pub(crate) k: usize,
    /// Neighbouring chunks added on each side of every hit, within the same story.
    #[arg(long, default_value_t = 1)]
    pub(crate) neighbors: usize,
    /// Passage budget in model tokens; lower-ranked hits are dropped past it.
    #[arg(long, default_value_t = 9000)]
    pub(crate) max_context_tokens: u32,
    #[arg(long, default_value_t = 400)]
    pub(crate) n_predict: u32,
    /// Answer the covered part of a question instead of declining it whole.
    #[arg(long)]
    pub(crate) partial: bool,
    /// Turn off question routing (table tools, timeline notes for time questions, synthesis for comparisons).
    #[arg(long)]
    pub(crate) no_route: bool,
    /// Which router picks a table tool before retrieval: `model` (one grammar-bound call to the answer model),
    /// `keywords` (the keyword rules of 2026-09-29, the kill switch: identical output on all 1,266 real and
    /// route-check questions), `knn` (nearest labelled examples by embedding, no LLM), `off` (no table tools; the
    /// policy lines and timeline notes stay, unlike --no-route).
    #[arg(long, value_enum, default_value_t = RouterKind::Model)]
    pub(crate) router: RouterKind,
    /// Labelled example questions for `--router knn` (written by --label-intents).
    #[arg(long, default_value = "eval/intents.jsonl")]
    pub(crate) intents: PathBuf,
    /// Embedder of the kNN router (models/gte-trevor is the fine-tuned one). Separate from --model-dir, which must
    /// stay the model the corpus vectors were made with.
    #[arg(long, default_value = "models/gte-modernbert-base")]
    pub(crate) knn_model_dir: PathBuf,
    #[arg(long, default_value = "model_int8.onnx")]
    pub(crate) knn_onnx: String,
    /// Neighbours that vote.
    #[arg(long, default_value_t = 5)]
    pub(crate) knn_k: usize,
    /// A table tool needs its nearest example at this cosine or above. 0.80 with --knn-min-share 0.7 is the
    /// leave-one-out pick on the training 80% of eval/intents.jsonl (2026-09-29, gte-modernbert-base, k 5): test table
    /// precision 1.000 on 3 answers, recall 0.167; 0 and 0.6 gave precision 0.611 on the training sweep.
    #[arg(long, default_value_t = 0.80)]
    pub(crate) knn_min_sim: f32,
    /// A table tool needs this share of the similarity-weighted vote.
    #[arg(long, default_value_t = 0.7)]
    pub(crate) knn_min_share: f32,
    /// Route a question that is itself a labelled example by the other examples (the route check uses it, so the
    /// kNN router is not scored on its own labels).
    #[arg(long)]
    pub(crate) knn_holdout: bool,
    /// Label every question of the Reddit (without categories X and P), Discord and route-check sets with the model
    /// router into this file (resumable); route-check expectations override the model where they conflict.
    #[arg(long)]
    pub(crate) label_intents: Option<PathBuf>,
    /// Paraphrases of the training questions (`--paraphrase-intents`) added to the kNN examples; off by default.
    #[arg(long)]
    pub(crate) knn_synth: Option<PathBuf>,
    /// Write --paraphrases rewrites of every table-labelled training question, each checked by the model router.
    #[arg(long)]
    pub(crate) paraphrase_intents: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    pub(crate) paraphrases: usize,
    /// The hybrid router takes the kNN decision when its nearest example is at this cosine or above...
    #[arg(long, default_value_t = 0.92)]
    pub(crate) hybrid_min_sim: f32,
    /// ...and its vote share at least this; otherwise it asks the model router.
    #[arg(long, default_value_t = 0.9)]
    pub(crate) hybrid_min_share: f32,
    /// Evaluate the kNN router on a seeded 80/20 split of --intents and exit.
    #[arg(long)]
    pub(crate) knn_eval: bool,
    /// Keep the reading guide as it was before it led with the game's Storylines links (2026-09-29).
    #[arg(long)]
    pub(crate) no_storylines: bool,
    /// Ignore the topic tool: no topic summary is added to retrieval.
    #[arg(long)]
    pub(crate) no_topics: bool,
    /// With `--lore v2`, add a new dossier also to a question routed to a typed source (IS, voice, module, outfit, item,
    /// enemy), as on 2026-10-02 morning, when it lost r021 ("the same Tin Man from the random encounter in IS2?").
    #[arg(long)]
    pub(crate) dossier_on_typed: bool,
    /// With `--lore v2`, do not add the game-data passage and inference rule to a question about a named character's
    /// body, race features, abilities or age (2026-10-02).
    #[arg(long)]
    pub(crate) no_game_data: bool,
    /// v1: the 46 topics and the P3b and P4 built with the 200 dossiers, as before 2026-10-01; v2 (the default since
    /// 2026-10-03; 0 losses on real probe, held-out and gold): plus the topics and dossiers chosen from data, a new
    /// dossier and a game-data passage at answer time. `--lore v1` is the kill switch.
    #[arg(long, value_enum, default_value = "v2")]
    pub(crate) lore: LoreSet,
    /// Recap stories by the old substring test on story ids (Episode 1 then also takes Episodes 10 to 16); the
    /// keyword router always uses it, since the exact test changes its "ending of Episode 1" answer.
    #[arg(long)]
    pub(crate) recap_substring: bool,
    /// Do not let the model router name the source of a retrieval route (story, operator_file, module,
    /// voice, skin, is, enemy, item, any), which picks P4 and the preferred kind instead of `source_kind`'s keyword
    /// list. The source is the default since 2026-09-30: on the 32 real-probe questions it moves or that are
    /// source-specific, responds 7 to 14 (7 gained, 0 lost), faithfulness 0.712 to 0.762, gold set v2 1 gained of 4
    /// moved. This flag restores the router before it (`--route-only` byte-identical on 266 questions).
    #[arg(long)]
    pub(crate) no_router_source: bool,
    /// Number Integrated Strategies runs as the data's ids do (rogue_1 = IS 1), as before 2026-09-30; the game counts
    /// Ceobe's Fungimist as IS #1, so rogue_1 Phantom & Crimson Solitaire is IS #2.
    #[arg(long)]
    pub(crate) is_numbering_v1: bool,
    /// Do not add lore to a question about an outfit, item, module or enemy (before 2026-09-30). By default the story
    /// and operator-file passages its best unit's text retrieves follow the unit, other operators' units are dropped,
    /// and the answer has two parts, the second opening "Lore context:" (see `LORE_KINDS` for why operator files and
    /// voice lines are left out). Default since round 6c (2026-09-30): Ian's Angelina outfit question answers in two
    /// parts, r046 responds 0 to 1 (faithful 6/6), r102 unchanged, 0 losses, `--route-only` identical on 266 questions.
    #[arg(long)]
    pub(crate) no_source_lore: bool,
    /// Do not add the full name after a short name the question uses ("kal" -> "kal (Kal'tsit)"), for retrieval
    /// and the answer (before 2026-09-30).
    #[arg(long)]
    pub(crate) no_name_expansion: bool,
    /// The model router has no reading_compare tool ("can I read X after Y", "does X spoil Y"), as before 2026-09-30.
    #[arg(long)]
    pub(crate) no_reading_compare: bool,
    /// No question-form call (before 2026-09-30). By default one more grammar-bound Gemma call sorts a retrieval
    /// question into fact, yes_no or opinion with an evidence query; a yes/no question interleaves the evidence query's
    /// hits and gets YES_NO_RULE, a judgment question gathers candidates from operator files and dossiers (named
    /// characters first, found by codename, real name or identity link) and gets OPINION_FORM_RULE. Measured on 63
    /// real-probe questions: responds 42 to 52 (13 gained, 3 lost: r030, r046, r080), handles 26 to 23 of 27,
    /// faithfulness 0.903 to 0.895; 6 fact controls byte-identical. Made the default by Ian (2026-09-30, +13/-3).
    #[arg(long)]
    pub(crate) no_question_form: bool,
    /// No second look at an opinion form (before 2026-10-01): the first call's opinion stands even for a question about
    /// one character's trait or feelings (see FORM_SYSTEM_V2).
    #[arg(long)]
    pub(crate) form_v1: bool,
    /// Add OPINION_RULE on the keyword test ("favorite", "strongest", ...) even when the form call ran and chose fact or
    /// yes_no (before 2026-10-01). By default the form call's choice decides: "What is Insider's favorite food?" (form
    /// fact) opened "The story does not settle on a single favorite food", and trivia t064 "what's blaze's favorite
    /// yanese saying?" hedged its right answer away the same way.
    #[arg(long)]
    pub(crate) keyword_opinion: bool,
    /// Decide TIME_RULE with the timeline notes, SYNTH_RULE, CANON_RULE and SOURCE_RULE by one more grammar-bound Gemma
    /// call after the form call (prompts/flags.system.txt) instead of the English keyword tests. Opt-in (2026-10-01):
    /// on the questions whose flags differ it lost answers on real probe, gold and Ian's set (design/trevor-questions.md
    /// section 12); without it the keyword tests decide, exactly as before.
    #[arg(long)]
    pub(crate) model_flags: bool,
    /// Give the model router an overview tool (opt-in, 2026-10-01): a question about the whole story or the whole world
    /// ("Summarize the Arknights story in one word") is answered from the summary tree of `scripts/overview.py` (the
    /// story and world overviews and one summary per storyline) instead of retrieval. Opt-in because its router lines
    /// move one of Ian's 9 retrieval sources (ian3, any to story); a rule added the same day keeps two-event reading
    /// questions on reading_compare (route check 75/76 with it). Without it the router prompt and grammar are exactly
    /// the ones before.
    #[arg(long)]
    pub(crate) overview: bool,
    /// Print the question form of each question of --batch (or the question) as `qid<TAB>form<TAB>evidence`, then
    /// (with --model-flags) the answer-rule flags and the keyword tests' flags, and exit.
    #[arg(long)]
    pub(crate) form_only: bool,
    /// Print each question of --batch (or the question) with its short names expanded, and exit (no model).
    #[arg(long)]
    pub(crate) show_expansion: bool,
    /// Answer canon questions from reference counts only, as before the per-run verdicts (2026-09-30).
    #[arg(long)]
    pub(crate) canon_v1: bool,
    /// Set for the single question, which `main` routes before loading the index.
    #[arg(skip)]
    pub(crate) routed: bool,
    /// Retry a declined answer once with a rewritten query and wider retrieval. Off by default: on 2026-09-29 it
    /// answered 0 of 36 declines on 120 real questions (rewrite only) and 0 on gold set v2 (rewrite plus 16 passages),
    /// at about 6 s more per question.
    #[arg(long)]
    pub(crate) retry: bool,
    /// Print the table answer when a table route answers, else the line RETRIEVAL, and exit without loading a
    /// model or the index (scripts/route-check.py runs the routing regression set with it).
    #[arg(long)]
    pub(crate) route_only: bool,
    /// Do not start a llama-server when none answers at --server (fail instead, as before).
    #[arg(long)]
    pub(crate) no_spawn: bool,
    /// The shared model lock (`<pid> <label>`): while another job holds it, `ask` waits for it instead of sharing that
    /// job's server (2026-09-30: `ask` reused a canon job's reasoning-on Gemma and hung). A lock held by an ancestor of
    /// this process (the script that started `ask`) is ours. `ask` writes the lock itself for a server it starts.
    #[arg(long, default_value = "/private/tmp/claude-501/trevor-llm.lock")]
    pub(crate) lock: PathBuf,
    /// Fail at once when another job holds the model lock, instead of waiting.
    #[arg(long)]
    pub(crate) no_wait: bool,
    /// Share the server even when another job holds the model lock (before 2026-09-30).
    #[arg(long)]
    pub(crate) ignore_lock: bool,
    /// A second retrieval query in the words the story itself would use (one more Gemma call), its hits interleaved
    /// with the question's own: "What operator declared that they want to pee in public?" never retrieves Chiave's "see
    /// who can piss farther", which a query in the story's words ranks 2nd (2026-09-30). Opt-in until measured.
    #[arg(long)]
    pub(crate) story_query: bool,
    /// Append every model request's rendered prompt, `prompt_n`, `cache_n` and output to this file (JSON lines), to
    /// check that two runs send the same prompts (2026-10-02). Debug only; the answers are unchanged.
    #[arg(long)]
    pub(crate) dump_prompt: Option<PathBuf>,
    /// Send every request with `cache_prompt: false`: no prompt reuses KV cache from an earlier one, so the same
    /// prompts decode to the same answers whatever ran before (2026-10-02; slower by the reused prefix). Off by default.
    #[arg(long)]
    pub(crate) no_cache_prompt: bool,
    /// The router sends a question about one operator's everyday life (likes, downtime, opinions of the base) to
    /// their voice lines (2026-10-03; opt-in until measured, see design/trevor-questions.md section 12).
    #[arg(long)]
    pub(crate) voice_rule: bool,
    /// Leave out the explicit-estimate rule a question asking an age gets with the game-data passage (the rule is the
    /// default since 2026-10-03).
    #[arg(long)]
    pub(crate) no_age_estimate: bool,
    /// Race topics serve the summary after the speaker-aware trait check (`topics.py traitcheck2`, 2026-10-03), which
    /// drops a body-feature sentence resting on a line about someone of another race (Durin "thick tails"). Opt-in.
    #[arg(long)]
    pub(crate) speaker_trait_check: bool,
    /// Operator art: the router's "art" source retrieves the model-written art descriptions (artifacts/p4-art, built
    /// by scripts/art_captions.py units and build-units; 2026-10-03, opt-in until measured).
    #[arg(long)]
    pub(crate) art: bool,
    /// Replace the last N of the k fused hits with the Ettin reranker's best hits (of the fused top 40) not already
    /// among them. Reranking outright fails on gold v2 over P3b (recall@10 0.732 to 0.605, 22 lost and 8 gained,
    /// 2026-09-30) but the reranker's first pick is often a passage fusion ranks lower: fused top 7 plus its best new
    /// hit against fused top 8 is recall +0.063 [+0.023, +0.109], 10 gained and 2 lost. Opt-in (0) until the answers
    /// are measured.
    #[arg(long, default_value_t = 0)]
    pub(crate) rerank_add: usize,
    /// Turn off the origin chain (default since 2026-10-03, item 1): a why/when/how-it-began question gets ORIGIN_RULE
    /// (trace the causes back to the earliest one the passages show) and the earlier scenes of the story its search keeps
    /// hitting, placed first (ian15 now starts from Swire's kidnapping; 0 losses; see design/trevor-questions.md section 12).
    #[arg(long)]
    pub(crate) no_origin_chain: bool,
    /// Relationship and admiration questions (2026-10-03, item 2): RELATION_RULE keeps literal relations apart from what
    /// characters call each other and keeps distinct people with one title apart. The default since 2026-10-05 night, for a
    /// relation question naming two people (or an admiration question), see `relation_rule_any`; this flag is kept and
    /// changes nothing.
    #[arg(long)]
    pub(crate) relation_rule: bool,
    /// Kill switch for RELATION_RULE (default since 2026-10-05 night: 0 pairwise losses on the 14 relationship questions,
    /// 4 wins: g0099, ian19, ian23, relx299).
    #[arg(long)]
    pub(crate) no_relation_rule: bool,
    /// With `--relation-rule`, apply it also to a question naming a nation, place, race or organization topic (since
    /// 2026-10-05 the rule is for people only: "Are Petrams related to Aegirs?", real r108, was lost to it twice).
    #[arg(long)]
    pub(crate) relation_rule_all: bool,
    /// Kill switch for the scoped scenes (default since 2026-10-05 night): a question naming a main episode or event and a
    /// character who speaks or is named in it gets, first, that event's scenes with the character ranked by the question.
    #[arg(long)]
    pub(crate) no_scoped_scenes: bool,
    /// Kill switch for the character-scoped scenes (2026-10-06, `character_scenes`): a question naming no event and one
    /// character as the subject of a said or done act ("During which event Mon3tr suggested assassinating a child") gets
    /// excerpts of the scenes where that character speaks, their lines ranked by the reranker.
    #[arg(long)]
    pub(crate) no_character_scenes: bool,
    /// The line pool (opt-in, 2026-10-06, Ian's item 1): the lines of the hybrid top `LINE_POOL_CHUNKS` chunks, each with the
    /// line before it, ranked by the reranker; when the best line lies outside the retrieved passages, the best
    /// `LINE_POOL_LINES` such lines come as excerpts after the retrieved passages, within `LINE_POOL_TOKENS` on top.
    /// Measured 2026-10-06: gold correct +7 -1 on 30 (g0034 lost), but Ian's ian17 and ian30 lost pairwise, so opt-in.
    #[arg(long)]
    pub(crate) line_pool: bool,
    /// With `--line-pool`, add the excerpts even when the best pool line is already among the retrieved passages.
    #[arg(long)]
    pub(crate) line_pool_all: bool,
    /// With `--line-pool`, fire only when the best line outside the retrieved passages scores at least this (reranker
    /// logit). TREVOR_LINE_POOL_DEBUG=1 prints each question's best score; TREVOR_LINE_POOL_DETECT=1 (measurement) skips
    /// the answer call and returns "line-pool detect: <excerpts>".
    #[arg(long)]
    pub(crate) line_pool_min: Option<f32>,
    /// Kill switch for the answer extension (default since 2026-10-06): an answer that stopped at `--n-predict` is asked
    /// again with twice the limit (Ian's ian27 stopped mid-sentence at 400 tokens).
    #[arg(long)]
    pub(crate) no_answer_extend: bool,
    /// The verdict-first yes/no rule (opt-in, refuted 2026-10-06, Ian's item 2): YES_NO_VERDICT_RULE instead of YES_NO_RULE;
    /// on the 50 yes/no questions Qwen pairwise 5 wins, 7 losses, 38 ties, and ian26 still opens "The passages do not say".
    #[arg(long)]
    pub(crate) verdict_first: bool,
    /// With `--relation-rule`, apply it also to a relation question naming fewer than two people (since 2026-10-05
    /// night the rule needs two named people, or an admiration question: real-style relx26 "what their relationship is ...
    /// Drudge is the big brother" and relx810 "Is Mon3tr related to the Aggeloi?" were lost to it pairwise).
    #[arg(long)]
    pub(crate) relation_rule_any: bool,
    /// Measurement: with --batch, print the scoped scenes each question would get (chunk ids), without a model.
    #[arg(long, hide = true)]
    pub(crate) scoped_only: bool,
    /// Measurement (2026-10-06, Ian's item 1): with --batch of {qid, question, chunk, quote}, the rank of the evidence line
    /// among every line of the corpus (each with the line before it) by line dense and by the reranker, estimated on a
    /// stride sample (TREVOR_LINE_STRIDE, default 30; the reranker on every TREVOR_LINE_RR_STRIDE-th, default 300), next to
    /// the chunk's dense, BM25 and fused ranks. No model server.
    #[arg(long, hide = true)]
    pub(crate) line_study: bool,
    /// The opinion gate (opt-in, 2026-10-05 night; `opinion_needs_candidates`): an opinion question that names no character
    /// or group and asks no who/which gets no candidates from operator files and dossiers and OPINION_RULE ("the story does
    /// not settle it") instead of OPINION_FORM_RULE. Fixes real r030 ("In lore how strong are my favourite characters?") by
    /// the real-probe judge, but loses 3 of 19 touched questions pairwise (r039, r067, r068). Since 2026-10-06 the default
    /// is the narrow gate (`no_opinion_gate`); this flag is night 11's wider reach, measurement only.
    #[arg(long)]
    pub(crate) opinion_gate: bool,
    /// Kill switch for the narrow opinion gate (default since 2026-10-06): the gate of `--opinion-gate` only for a question
    /// the keyword test also calls an opinion ("favourite", "do you think", ...) and that names no word the corpus writes
    /// as a proper name (`names_proper_word`: "Fort barron"), so real r039, r067 and r068 keep their candidates. Fires on
    /// r030 and r116 of the 448 set questions: real-probe responds 1 to 2 of 2 and handles 1 to 2 (r030 regains both),
    /// faithfulness 16/18 to 12/14, Qwen pairwise 2 ties, 0 losses.
    #[arg(long)]
    pub(crate) no_opinion_gate: bool,
    /// When-questions (2026-10-03, item 3): DATE_RULE, end with a labelled year estimate from the timeline notes when
    /// no passage states the year (the age rule's counterpart for dates). Opt-in until measured.
    #[arg(long)]
    pub(crate) date_estimate: bool,
    /// The router's `cross_ref` tool (2026-10-03, item 4): cast-wide game-data cross-references, first "boss enemies
    /// who are playable operators" (artifacts/crossref/, scripts/crossref.py). Opt-in until measured.
    #[arg(long)]
    pub(crate) cross_ref: bool,
    /// Off-topic guard (2026-10-03, item 8): a question that names nothing Trevor's tables know gets LORE_ONLY_RULE
    /// (Trevor answers Arknights lore only). Opt-in until measured.
    #[arg(long)]
    pub(crate) lore_only: bool,
    /// Turn off the race-trait rule (default since 2026-10-03, item 11, no model): race topics serve the summary after
    /// `topics.py traitrule`, where a body-feature sentence stays only when a cited line gives the feature to a speaker or
    /// subject of that race in operator_attributes.jsonl (drops Durin "thick tails" and Savra "a horn"; 0 losses).
    #[arg(long)]
    pub(crate) no_race_trait_rule: bool,
    /// Turn off the typed fallback (default since 2026-10-03, item 10; no router change): a declined answer to a
    /// question naming one operator, not routed to a typed source, is tried again with that operator's voice lines,
    /// module stories or art units first, and kept only when it cites one of that operator's own units (trivia correct
    /// 0 to 3 of 3 changed, gold 0 losses, Ian's 25 unchanged; see design/trevor-questions.md section 12).
    #[arg(long)]
    pub(crate) no_typed_fallback: bool,
    /// Turn off the game text (default since 2026-10-03, item 6): P4 routes read artifacts/p4x, P4 plus the game text
    /// the coverage audit found unindexed (scripts/gametext.py; real probe responds 2 to 4 of 6 changed, 0 losses).
    #[arg(long)]
    pub(crate) no_game_text: bool,
    /// Turn off the appearance index (2026-10-03, item 7): "What chapters does Elysium appear in?" is answered, before
    /// the router, by the `appearances` tool (scripts/appearances.py: the story groups where a character speaks, and apart
    /// those where they are only named).
    #[arg(long)]
    pub(crate) no_appearances: bool,
    /// Turn off the cross-reference trigger (2026-10-03, leftover d): a question with "boss" and a joining word goes to
    /// the `cross_ref` tool before the router, with no router line (the `--cross-ref` line moved 34 routes).
    #[arg(long)]
    pub(crate) no_cross_ref_trigger: bool,
    /// Turn off the design-inspiration tool (2026-10-04, Ian's ian11): a question with a design word ("based off an animal",
    /// "inspired by", "motif", "design inspiration") whose content word is an inferred subject or an operator name in
    /// artifacts/entities/design_infer.jsonl (scripts/design_infer.py: deduced from the game data, checked by a second
    /// model) is answered, before the router, by the `design_basis` tool, both readings, labelled as Trevor's inference.
    #[arg(long)]
    pub(crate) no_design_basis: bool,
    /// Two-step design deduction (2026-10-05, opt-in): a design question the `design_basis` table cannot answer goes,
    /// before the router, to a deduction from the game's art and text (artifacts/entities/design_evidence.jsonl, from
    /// scripts/design_infer.py evidence). Forward ("which operator is based on an animal with phantom in its name"):
    /// Gemma lists real-world candidates with their visible features from general knowledge, a data shortlist (inverse-
    /// frequency feature words over the evidence rows) keeps 8 operators, and Gemma names the ones whose evidence shows the
    /// features. Reverse ("what animal is Kirara based on"): Gemma reads the named operator's evidence. Labelled "Trevor's
    /// deduction from the game's art and text"; says plainly when nothing in the game shows a match.
    #[arg(long)]
    pub(crate) design_deduce: bool,
    /// The wiki-based answers of the morning of 2026-10-04, for measurement only (2026-10-04, Ian: the wiki is a
    /// reference, never a source): the design tool reads the wiki trivia table (trevor::reference) and the reading-guide
    /// and death texts credit the wiki as before. Pair it with the wiki-built tables (MAIN_DATES=wiki reading_guide.py
    /// dates and build, DEATH_CONFIRM=wiki deaths.py build) to reproduce the old answers.
    #[arg(long)]
    pub(crate) wiki_legacy: bool,
    /// Turn off IS-ending passages (2026-10-03, item 2): a question naming an Integrated Strategies run and one of its
    /// endings (number, ordinal or name) or its endbooks is answered from that ending's scene and endbook stories, with
    /// the game's ending order, without the router.
    #[arg(long)]
    pub(crate) no_is_ending: bool,
    /// The named topic (2026-10-03, item 5; opt-in after losses): a retrieval question that names one served nation,
    /// place, race or organization topic and no operator gets its summary although the router chose no topic ("factions
    /// in dossoles"). Measured with --topic-events and --place-nation: held-out responds 10 to 7 of 10 changed (lost
    /// h004, h017, h033, near-identical declines), real 10 to 11 of 14, gold 16 to 17 of 24; ian30 answers the factions.
    #[arg(long)]
    pub(crate) named_topic: bool,
    /// With `--named-topic`, the night 7 reach: every question naming one such topic (since 2026-10-05 only a question about
    /// the topic as a whole, `Tools::broad_topic_question`).
    #[arg(long)]
    pub(crate) named_topic_all: bool,
    /// Topic events (2026-10-03, items 3 and 4; opt-in, measured with --named-topic): a topic summary comes with the P2
    /// event summaries of the 3 story groups that name the topic most (scripts/topic_groups.py).
    #[arg(long)]
    pub(crate) topic_events: bool,
    /// Turn off the term rule (2026-10-03, item 6): a "what is/are X" question gets TERM_RULE (plain words, a short
    /// definition of each in-world term used).
    #[arg(long)]
    pub(crate) no_term_rule: bool,
    /// The place's nation (2026-10-03, item 5; opt-in, measured with --named-topic): a place topic comes with the topic
    /// of the nation its summary names most (Dossoles: Bolívar).
    #[arg(long)]
    pub(crate) place_nation: bool,
    /// Turn off chronology lines for the subject (2026-10-03, leftover c): a when-question gets the dated lines of
    /// artifacts/chrono/terra_history.md that name its subject, and DATE_RULE (a labelled estimate).
    #[arg(long)]
    pub(crate) no_chrono_subject: bool,
    /// Turn off the own-file fallback (2026-10-03, leftover b): an answer to a one-operator question that cites only
    /// that operator's file is tried again like a decline (typed fallback), kept only when it cites the operator's own
    /// voice line, module or art unit.
    #[arg(long)]
    pub(crate) no_own_file_fallback: bool,
    /// Identity notes (2026-10-03, leftover a; opt-in): for a who/which/identity question, when a passage is the source
    /// of a stated identity link of identities.v2.json (Iris's file: Mabel was Bluishsilver), a note after the passages
    /// states the link and its quote; inferred links sharing a name follow, labelled. ian24 still declines with the note
    /// in its prompt, and the prefilter undercounted where it fires (ian1, ian3, ian7, ian18 changed outside it).
    #[arg(long)]
    pub(crate) identity_notes: bool,
    /// Turn off time evidence (2026-10-03 night 8; default, 0 losses on the 3 questions it fires on, ian17 now
    /// estimates "between 1086 and 1090"): when chronology lines name a when-question's subject, they come as a numbered
    /// TIME EVIDENCE block right before the question, and the question is followed by the estimate as the required
    /// final line ("Estimate: ..."); off, they are the CHRONOLOGY LINES block before the identity notes, as before.
    #[arg(long)]
    pub(crate) no_time_evidence: bool,
    /// Identity labels (2026-10-03 night 8; opt-in): for a who/which/identity question, a passage that is the source of a
    /// stated identity link of identities.v2.json carries the resolved link in its label ("Bluishsilver, real name
    /// Mabel, per ..."), both names proper names by the corpus (the lowercase form is under 1% of the capitalized
    /// count: "Operator" 1,168 of 6,260 is dropped).
    #[arg(long)]
    pub(crate) identity_labels: bool,
    /// Turn off deep topic entries (default since 2026-10-05, lore v2 only): the topic tool serves the judged map-reduce
    /// entry of scripts/topic_deep.py (artifacts/topics/deep.jsonl: the story groups, operator files, game records and
    /// dossiers naming the topic, summarized per source, then into What it is, Traits, History, Society and politics,
    /// Notable people, Key events, Open questions; the judge's unsupported sentences dropped) instead of the summary, for
    /// a question about the topic as a whole (`Tools::broad_topic_question`: at most one content word besides the
    /// topic's name). Night 9 served it to every topic question and lost held-out h033 and real r079 (both specific
    /// questions); gated, it changes ian28 alone of the 447 (Qwen pairwise: the entry wins) and the 14 topic-routed
    /// questions with an entry are byte-identical.
    #[arg(long)]
    pub(crate) no_deep_topics: bool,
    /// Turn off term lookup (default since 2026-10-03 night 9; fires on 4 of 447 questions, changes 2, 0 losses: ian31
    /// now defines the Khaganquest from its game-text record): a short what-is question whose term occurs in P4x as one
    /// to three words written together or apart ("khagan quest" is "Khaganquest") searches with the corpus's spelling
    /// and gets, before the passages, the 2 P4x chunks naming the term most (records and files first) and a glossary
    /// of the served topics those chunks name (the first two sentences of each summary, at most 3).
    #[arg(long)]
    pub(crate) no_term_lookup: bool,
    /// Identity expansion (2026-10-03 night 9; opt-in, no gains): a who/which question naming a character, whose
    /// passages include the source of a stated identity link (Iris's file: Mabel was Bluishsilver), also gets the 2
    /// passages where a linked name and the named character appear together (the interlude where Iris looks for Mabel
    /// Grimm), placed right after the link's source passage, each labelled with the resolved link. Measured on the 11
    /// questions it fires on: 0 judged losses, no gains; ian24 answers Sakiko Togawa alone and declines with
    /// --identity-labels.
    #[arg(long)]
    pub(crate) identity_expand: bool,
    /// Turn off the answer category check (default since 2026-10-05): a who/which question that constrains its answer's category ("not
    /// playable", "NPC", "playable operator", "from <nation>") has the names its answer gives checked against the
    /// game data (operator_attributes names, real names, identity links); when every name it gives fails, it is asked
    /// once more with a passage saying which names fail and why, and a second failure gets that note appended. Fires on
    /// ian24 alone of the 447 set questions and on 2 of 1,228 Reddit and Discord questions; changes no default answer
    /// (the default declines ian24); with --identity-expand ian24 goes from Sakiko Togawa (a playable operator) to a decline.
    #[arg(long)]
    pub(crate) no_answer_check: bool,
    /// The note of the answer check's second ask, set only inside `answer_one`.
    #[arg(skip)]
    pub(crate) check_note: Option<String>,
    /// The evidence composition (`compose_evidence`; opt-in, 2026-10-05 night: it fires on ian24 alone of the 448 set
    /// questions, which still declines with the composed evidence, so it changes no answer and costs one more call).
    #[arg(long)]
    pub(crate) compose_evidence: bool,
    /// The composed passages of the evidence composition's ask, set only inside `answer_one`.
    #[arg(skip)]
    pub(crate) compose: Vec<(String, String, String)>,
    /// Serve the deep entry to every topic question, as `--deep-topics` did on night 9 (measurement only; lost h033 and
    /// r079 then).
    #[arg(long)]
    pub(crate) deep_topics_all: bool,
    /// Turn off the rarity order of the term-lookup glossary (default since 2026-10-05): the served topics the term's
    /// chunks name are ordered by first mention, as on night 9, instead of rarest first (fewest P4x chunks naming them).
    #[arg(long)]
    pub(crate) no_glossary_rarity: bool,
    /// With `--place-nation`, the nation of a place from the summary count of night 7 (Chernobog: Yan) instead of
    /// artifacts/topics/place_nation.json (scripts/place_nation.py, 2026-10-03 night 8; Chernobog: Ursus).
    #[arg(long)]
    pub(crate) place_nation_summary: bool,
    /// With `--batch`: print, per question, which of the 2026-10-03 question-only detectors fire, without a model
    /// (the prefilter of the A/B runs).
    #[arg(long, hide = true)]
    pub(crate) detect_only: bool,
    /// With `--batch`: print the identity questions whose plain top k would get identity notes (needs the index).
    #[arg(long, hide = true)]
    pub(crate) detect_identity: bool,
    /// Model file for the server `ask` starts when none is running.
    #[arg(long, default_value = "models/llm/gemma-4-12b-it-qat-q4_0.gguf")]
    pub(crate) model: PathBuf,
    /// Answer every item of a gold set (JSONL with `qid` and `question`).
    #[arg(long)]
    pub(crate) batch: Option<PathBuf>,
    #[arg(long, default_value = "artifacts/answers.jsonl")]
    pub(crate) out: PathBuf,
}

/// Which lore Trevor serves (2026-10-01): v1 is the 46 topics and the 200 dossiers, v2 adds the topics and dossiers
/// chosen from data (scripts/topics.py mine, scripts/dossiers.py characters_v2), in their own corpus directories.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum LoreSet {
    V1,
    V2,
}

impl LoreSet {
    /// The P3b and P4 directories `ask` uses when `--corpus` is left at its default.
    pub(crate) fn dirs(self) -> (&'static str, &'static str) {
        match self {
            Self::V1 => ("artifacts/p3b", "artifacts/p4"),
            // Since 2026-10-02 v2 retrieves from the v1 corpora: built into P3b, the 336 new dossiers changed the passages
            // of 50 of 120 real-probe questions and reordered others (5 lost), so they are added by name instead.
            Self::V2 => ("artifacts/p3b", "artifacts/p4"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum RouterKind {
    Model,
    Keywords,
    Knn,
    /// kNN when it is sure (--hybrid-min-sim, --hybrid-min-share), else the model router.
    Hybrid,
    Off,
}

/// Whether the form call runs: model, kNN and hybrid routers, not the keyword kill switch.
pub(crate) fn form_on(a: &Args) -> bool {
    a.router != RouterKind::Keywords && !a.no_route && !a.no_question_form
}

/// Whether a typed source's own text also retrieves story passages about what it names (not for the keyword
/// router, which keeps retrieval as it was; `--no-source-lore` turns it off).
pub(crate) fn lore_on(a: &Args) -> bool {
    a.router != RouterKind::Keywords && !a.no_source_lore && !a.no_route
}

/// The parts of the model router's prompt that `a` turns on.
pub(crate) fn router_prompt(a: &Args) -> router::RouterPrompt {
    router::RouterPrompt { source: router_source(a), compare: !a.no_reading_compare, overview: a.overview,
                           wide_topics: a.lore == LoreSet::V2, voice_rule: a.voice_rule, art: a.art, cross_ref: a.cross_ref }
}

/// Whether the model router names the source of a retrieval route (default; `--no-router-source` turns it off).
pub(crate) fn router_source(a: &Args) -> bool {
    // Not the hybrid router: its kNN decisions carry no source, so it keeps the keyword list for all of its routes.
    a.router == RouterKind::Model && !a.no_router_source && !a.no_route
}

/// Whether a new dossier may join this question's passages: not when it is routed to a typed P4 source, which asks
/// about that text, not about the character (`--dossier-on-typed` allows it).
pub(crate) fn dossier_allowed(q: &str, a: &Args, route: Option<&Route>) -> bool {
    a.dossier_on_typed || !retrieval_plan(q, a, route).1.is_some_and(|k| k != "archive")
}

/// Where retrieval looks: Some(true) for P4 (typed sources), Some(false) for P3b, None to keep the loaded corpus; and
/// the P4 unit kind whose best 3 chunks go first. The model router's source decides when it names one; otherwise
/// the keyword list `source_kind`, as before.
pub(crate) fn retrieval_plan(q: &str, a: &Args, route: Option<&Route>) -> (Option<bool>, Option<&'static str>) {
    if a.no_route {
        return (None, None);
    }
    if !router_source(a) {
        return (None, source_kind(q));
    }
    match route.and_then(|r| r.args.get("source")).map(String::as_str) {
        None | Some("story") => (Some(false), None),
        Some("any") => (Some(true), None),
        Some("operator_file") => (Some(true), Some("archive")),
        Some(k) => (Some(true), ["module", "voice", "skin", "is", "enemy", "item", "art"].into_iter().find(|x| *x == k)),
    }
}

/// P4 plus the game text the coverage audit found unindexed (`scripts/gametext.py`, 2026-10-03, item 6): story intros,
/// unfetched story scripts, record notes, event archives, mail, operator record intros, world tips and event text, as
/// "gametext" units. Read for P4 routes by default since 2026-10-03; `--no-game-text` reads P4 as before.
pub(crate) const GAME_TEXT_DIR: &str = "artifacts/p4x";

/// The P4 directory `a` reads: `GAME_TEXT_DIR` when it is built (unless `--no-game-text`), else the lore's P4.
pub(crate) fn p4_dir(a: &Args) -> &'static str {
    if !a.no_game_text && std::path::Path::new(GAME_TEXT_DIR).join("chunks.jsonl").exists() { GAME_TEXT_DIR } else { a.lore.dirs().1 }
}

/// Whether the topic tool serves deep entries for this question (default since 2026-10-05, lore v2 only; `--no-deep-topics`):
/// only for a question about the topic as a whole (`Tools::broad_topic_question`); `--deep-topics-all` is night 9's reach.
pub(crate) fn deep_for(a: &Args, tools: &Tools, q: &str) -> bool {
    !a.no_deep_topics && a.lore != LoreSet::V1 && (a.deep_topics_all || tools.broad_topic_question(q))
}
