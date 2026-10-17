//! The command line: every flag of `ask`, and the gates derived from more than one flag (which router
//! runs, which corpus and source rules apply).

use std::path::PathBuf;

use clap::Parser;
use trevor::router;
use trevor::search::runtime::RuntimeArgs;
use trevor::tools::{Route, Tools};

use crate::detect::source_kind;

#[derive(Parser, Clone, Debug)]
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
    /// Turn off question routing (table tools, timeline notes for time questions, synthesis for comparisons).
    #[arg(long)]
    pub(crate) no_route: bool,
    /// Which router picks a table tool before retrieval: `model` (one grammar-bound call to the answer model),
    /// `keywords` (the keyword rules, the kill switch), `off` (no table tools; the policy lines and timeline notes
    /// stay, unlike --no-route).
    #[arg(long, value_enum, default_value_t = RouterKind::Model)]
    pub(crate) router: RouterKind,
    /// Do not lead the reading guide with the game's Storylines links.
    #[arg(long)]
    pub(crate) no_storylines: bool,
    /// Ignore the topic tool: no topic summary is added to retrieval.
    #[arg(long)]
    pub(crate) no_topics: bool,
    /// With `--lore v2`, do not add the game-data passage and inference rule to a question about a named character's
    /// body, race features, abilities or age.
    #[arg(long)]
    pub(crate) no_game_data: bool,
    /// v1: the 46 topics and the P3b and P4 built with the 200 dossiers; v2 (the default): plus the topics and dossiers
    /// chosen from data, a new dossier and a game-data passage at answer time. `--lore v1` is the kill switch.
    #[arg(long, value_enum, default_value = "v2")]
    pub(crate) lore: LoreSet,
    /// Recap stories by the old substring test on story ids (Episode 1 then also takes Episodes 10 to 16); the
    /// keyword router always uses it, since the exact test changes its "ending of Episode 1" answer.
    #[arg(long)]
    pub(crate) recap_substring: bool,
    /// Do not let the model router name the source of a retrieval route (story, operator_file, module,
    /// voice, skin, is, enemy, item, any), which picks P4 and the preferred kind instead of `source_kind`'s keyword
    /// list. With this flag the model router names no source.
    #[arg(long)]
    pub(crate) no_router_source: bool,
    /// Number Integrated Strategies runs as the data's ids do (rogue_1 = IS 1); the game counts Ceobe's Fungimist as
    /// IS #1, so rogue_1 Phantom & Crimson Solitaire is IS #2.
    #[arg(long)]
    pub(crate) is_numbering_v1: bool,
    /// Do not add lore to a question about an outfit, item, module or enemy. By default the story
    /// and operator-file passages its best unit's text retrieves follow the unit, other operators' units are dropped,
    /// and the answer has two parts, the second opening "Lore context:" (see `LORE_KINDS` for why operator files and
    /// voice lines are left out).
    #[arg(long)]
    pub(crate) no_source_lore: bool,
    /// Do not add the full name after a short name the question uses ("kal" -> "kal (Kal'tsit)"), for retrieval
    /// and the answer.
    #[arg(long)]
    pub(crate) no_name_expansion: bool,
    /// The model router has no reading_compare tool ("can I read X after Y", "does X spoil Y").
    #[arg(long)]
    pub(crate) no_reading_compare: bool,
    /// No question-form call. By default one more grammar-bound Gemma call sorts a retrieval
    /// question into fact, yes_no or opinion with an evidence query; a yes/no question interleaves the evidence query's
    /// hits and gets YES_NO_RULE, a judgment question gathers candidates from operator files and dossiers (named
    /// characters first, found by codename, real name or identity link) and gets OPINION_FORM_RULE.
    #[arg(long)]
    pub(crate) no_question_form: bool,
    /// No second look at an opinion form: the first call's opinion stands even for a question about one character's
    /// trait or feelings (see FORM_SYSTEM_V2).
    #[arg(long)]
    pub(crate) form_v1: bool,
    /// Add OPINION_RULE on the keyword test ("favorite", "strongest", ...) even when the form call ran and chose fact or
    /// yes_no. By default the form call's choice decides, since a fact question that only uses an opinion word ("What
    /// is Insider's favorite food?") has a factual answer that OPINION_RULE makes the model hedge away.
    #[arg(long)]
    pub(crate) keyword_opinion: bool,
    /// Print the question form of each question of --batch (or the question) as `qid<TAB>form<TAB>evidence`, and exit.
    #[arg(long)]
    pub(crate) form_only: bool,
    /// Print each question of --batch (or the question) with its short names expanded, and exit (no model).
    #[arg(long)]
    pub(crate) show_expansion: bool,
    /// Answer canon questions from reference counts only, without the per-run verdicts.
    #[arg(long)]
    pub(crate) canon_v1: bool,
    /// Set for the single question, which `main` routes before loading the index.
    #[arg(skip)]
    pub(crate) routed: bool,
    /// Print the table answer when a table route answers, else the line RETRIEVAL, and exit without loading a
    /// model or the index (scripts/route-check.py runs the routing regression set with it).
    #[arg(long)]
    pub(crate) route_only: bool,
    /// Do not start a llama-server when none answers at --server (fail instead).
    #[arg(long)]
    pub(crate) no_spawn: bool,
    /// The shared model lock (`<pid> <label>`): while another job holds it, `ask` waits for it instead of sharing that
    /// job's server, which may run with other settings (reasoning on, for one). A lock held by an ancestor of this
    /// process (the script that started `ask`) counts as this process's own. `ask` writes the lock itself for a server
    /// it starts.
    #[arg(long, default_value = "/private/tmp/claude-501/trevor-llm.lock")]
    pub(crate) lock: PathBuf,
    /// Fail at once when another job holds the model lock, instead of waiting.
    #[arg(long)]
    pub(crate) no_wait: bool,
    /// Share the server even when another job holds the model lock.
    #[arg(long)]
    pub(crate) ignore_lock: bool,
    /// The llama-server at --server is shared by several processes and run by someone else (contract v1.1 section 1d:
    /// N `ask --worker` processes on one host, one llama-server with `-np N`): connect to it without waiting for the
    /// model lock, never start one (as --no-spawn) and never stop it. Off: each process waits for the lock and starts
    /// its own server when none answers.
    #[arg(long)]
    pub(crate) share_server: bool,
    /// Append every model request's rendered prompt, `prompt_n`, `cache_n` and output to this file (JSON lines), to
    /// check that two runs send the same prompts. Debug only; the answers are unchanged.
    #[arg(long)]
    pub(crate) dump_prompt: Option<PathBuf>,
    /// Send every request with `cache_prompt: false`: no prompt reuses KV cache from an earlier one, so the same
    /// prompts decode to the same answers whatever ran before (slower by the reused prefix). Off by default.
    #[arg(long)]
    pub(crate) no_cache_prompt: bool,
    /// Serve questions over a local HTTP JSON API instead of answering one (`POST /v1/ask`, `GET /health`; see
    /// `serve.rs`): the indexes, tools and llama-server load once and every job is a library call. Binds 127.0.0.1.
    #[arg(long)]
    pub(crate) serve: bool,
    /// The port `--serve` listens on.
    #[arg(long, default_value_t = 8090)]
    pub(crate) port: u16,
    /// Pull jobs from a myrtle backend instead of listening (design/trevor-contract-v1.md section 3, v1.1 section 2.1):
    /// long-polls `{URL}/api/trevor/worker/next` with the `x-trevor-worker-key` header from `TREVOR_WORKER_KEY`; see
    /// serve.rs for the heartbeat, the power guard and running several workers on one llama-server (--share-server).
    #[arg(long, value_name = "URL")]
    pub(crate) worker: Option<String>,
    /// The pipeline profile (design/trevor-contract-v1.md section 2; also per worker job): `full` is the default
    /// pipeline (the kill switch: no flag changes); `slim` is contract v1's CPU pipeline: the keyword router, k 5, a
    /// 2,000-token passage budget, 250 output tokens, no typed fallback, no answer extension and no second-look retries
    /// (answer check, operator list check); `mid` (contract v1.1 section 4, the
    /// cpu rung's default) is the keyword router, k 8, a 3,600-token budget, 250 output tokens and the same three
    /// follow-ups off; `mid6k` is `mid` with a 6,000-token budget.
    #[arg(long, value_enum, default_value_t = Profile::Full)]
    pub(crate) profile: Profile,
    /// Retrieval for serving: `hybrid` (dense + BM25, the default) or `bm25`, which never loads the dense index or the
    /// query embedder and runs every dense or hybrid retrieval as BM25 (for a host with no INT8-safe query embedder).
    /// `TREVOR_RETRIEVAL=bm25` sets it for a worker.
    #[arg(long, value_enum, default_value_t = RetrievalKind::Hybrid)]
    pub(crate) retrieval: RetrievalKind,
    /// The model backend: `llama` (the local llama-server, the default) or `openai`, an OpenAI-compatible chat
    /// endpoint: Cloudflare Workers AI when TREVOR_CF_MODELS and TREVOR_CF_ACCOUNT_ID (or TREVOR_CF_BASE_URL) are set,
    /// else the local llama-server's own /v1/chat/completions (started as usual), for tests. Grammar-bound calls become
    /// validated JSON prompts there, with the keyword router and the fact form as fallbacks.
    #[arg(long, value_enum, default_value_t = BackendKind::Llama)]
    pub(crate) backend: BackendKind,
    /// `--worker`: do not stream answer deltas to the backend (send the result alone).
    #[arg(long)]
    pub(crate) no_stream: bool,
    /// Run the 10 canary questions against each Cloudflare model id in TREVOR_CF_MODELS until one passes, and write
    /// the result to artifacts/canary/cf.json (TREVOR_CANARY_FILE), which the worker reads for its `cf` rung.
    #[arg(long)]
    pub(crate) canary: bool,
    /// Write the gatePos order (artifacts/chrono/gate_order.json, see src/search/horizon.rs) from the reading guide,
    /// the story table and the source dates, and exit.
    #[arg(long)]
    pub(crate) gate_order: bool,
    /// With `--batch`: compare the gatePos horizon with the old story-id horizon on every chunk and on the questions'
    /// retrieved candidates, print every difference, and exit (no model).
    #[arg(long, hide = true)]
    pub(crate) horizon_check: bool,
    /// With `--batch`: run only the router and question-form calls of each question and print the route, the form and
    /// the usage totals (on `--backend openai`, how often the JSON prompts fail validation).
    #[arg(long, hide = true)]
    pub(crate) json_check: bool,
    /// Passages in a `passages` rung answer (no model).
    #[arg(long, default_value_t = 5)]
    pub(crate) passages_n: usize,
    /// Leave out the explicit-estimate rule a question asking an age gets with the game-data passage.
    #[arg(long)]
    pub(crate) no_age_estimate: bool,
    /// Turn off the origin chain: a why/when/how-it-began question gets ORIGIN_RULE (trace the causes back to the
    /// earliest one the passages show) and the earlier scenes of the story its search keeps hitting, placed first.
    #[arg(long)]
    pub(crate) no_origin_chain: bool,
    /// RELATION_RULE for this question: `!no_relation_rule`, set by `main` and per job; `answer_one_inner` clears it for
    /// a question naming a group topic, and for a relation question naming fewer than two people (not an admiration one).
    #[arg(skip)]
    pub(crate) relation_rule: bool,
    /// Kill switch for RELATION_RULE (relationship and admiration questions; see `relation_rule`).
    #[arg(long)]
    pub(crate) no_relation_rule: bool,
    /// Kill switch for the scoped scenes: a question naming a main episode or event and a
    /// character who speaks or is named in it gets, first, that event's scenes with the character ranked by the question.
    #[arg(long)]
    pub(crate) no_scoped_scenes: bool,
    /// Kill switch for the character-scoped scenes (`character_scenes`): a question naming no event and one
    /// character as the subject of a said or done act ("During which event Mon3tr suggested assassinating a child") gets
    /// excerpts of the scenes where that character speaks, their lines ranked by the reranker.
    #[arg(long)]
    pub(crate) no_character_scenes: bool,
    /// Kill switch for the answer extension: an answer that stopped at `--n-predict` is asked again with twice the
    /// limit, so it does not end mid-sentence.
    #[arg(long)]
    pub(crate) no_answer_extend: bool,
    /// Measurement: with --batch, print the scoped scenes each question would get (chunk ids), without a model.
    #[arg(long, hide = true)]
    pub(crate) scoped_only: bool,
    /// Measurement: with --batch of {qid, question, chunk, quote}, the rank of the evidence line
    /// among every line of the corpus (each with the line before it) by line dense and by the reranker, estimated on a
    /// stride sample (TREVOR_LINE_STRIDE, default 30; the reranker on every TREVOR_LINE_RR_STRIDE-th, default 300), next to
    /// the chunk's dense, BM25 and fused ranks. No model server.
    #[arg(long, hide = true)]
    pub(crate) line_study: bool,
    /// Kill switch for the opinion gate (`opinion_needs_candidates`): an opinion question the keyword test also calls an
    /// opinion ("favourite", "do you think", ...) that names no character or group, asks no who/which and names no word
    /// the corpus writes as a proper name (`names_proper_word`: "Fort barron") gets no candidates from operator files and
    /// dossiers and OPINION_RULE ("the story does not settle it") instead of OPINION_FORM_RULE.
    #[arg(long)]
    pub(crate) no_opinion_gate: bool,
    /// Kill switch for group evidence: an opinion question naming two or more
    /// group topics (organizations, nations, places, races; `Tools::named_group_topics`) gets each group's best 3 chunks
    /// naming it, from the topic name plus the evidence query, ahead of the candidates.
    #[arg(long)]
    pub(crate) no_group_evidence: bool,
    /// Kill switch for the operator list check (`Tools::asks_operators`,
    /// `Tools::non_operator_items`): a question asking for operators whose answer lists someone who is not a playable
    /// operator is asked once more with a note naming them and whose operator file the cited passage is.
    #[arg(long)]
    pub(crate) no_operator_list_check: bool,
    /// Turn off the race-trait rule: race topics serve the summary after `topics.py traitrule`, where a body-feature
    /// sentence stays only when a cited line gives the feature to a speaker or subject of that race in
    /// operator_attributes.jsonl (drops Durin "thick tails" and Savra "a horn").
    #[arg(long)]
    pub(crate) no_race_trait_rule: bool,
    /// Turn off the typed fallback: a declined answer to a question naming one operator, not routed to a typed source, is
    /// tried again with that operator's voice lines, module stories or art units first, and kept only when it cites one
    /// of that operator's own units.
    #[arg(long)]
    pub(crate) no_typed_fallback: bool,
    /// Turn off the game text: P4 routes read artifacts/p4x, P4 plus the game text the coverage audit found unindexed
    /// (scripts/gametext.py).
    #[arg(long)]
    pub(crate) no_game_text: bool,
    /// Turn off the appearance index: "What chapters does Elysium appear in?" is answered, before
    /// the router, by the `appearances` tool (scripts/appearances.py: the story groups where a character speaks, and apart
    /// those where they are only named).
    #[arg(long)]
    pub(crate) no_appearances: bool,
    /// Turn off the cross-reference trigger: a question with "boss" and a joining word goes to
    /// the `cross_ref` tool before the router, with no router line.
    #[arg(long)]
    pub(crate) no_cross_ref_trigger: bool,
    /// Turn off the design-inspiration tool: a question with a design word ("based off an animal",
    /// "inspired by", "motif", "design inspiration") whose content word is an inferred subject or an operator name in
    /// artifacts/entities/design_infer.jsonl (scripts/design_infer.py: deduced from the game data, checked by a second
    /// model) is answered, before the router, by the `design_basis` tool, both readings, labelled as Trevor's inference.
    #[arg(long)]
    pub(crate) no_design_basis: bool,
    /// Turn off IS-ending passages: a question naming an Integrated Strategies run and one of its
    /// endings (number, ordinal or name) or its endbooks is answered from that ending's scene and endbook stories, with
    /// the game's ending order, without the router.
    #[arg(long)]
    pub(crate) no_is_ending: bool,
    /// Turn off the term rule: a "what is/are X" question gets TERM_RULE (plain words, a short
    /// definition of each in-world term used).
    #[arg(long)]
    pub(crate) no_term_rule: bool,
    /// Turn off chronology lines for the subject: a when-question gets the dated lines of
    /// artifacts/chrono/terra_history.md that name its subject, and DATE_RULE (a labelled estimate).
    #[arg(long)]
    pub(crate) no_chrono_subject: bool,
    /// Turn off the own-file fallback: an answer to a one-operator question that cites only
    /// that operator's file is tried again like a decline (typed fallback), kept only when it cites the operator's own
    /// voice line, module or art unit.
    #[arg(long)]
    pub(crate) no_own_file_fallback: bool,
    /// Turn off time evidence: when chronology lines name a when-question's subject, they come as a numbered
    /// TIME EVIDENCE block right before the question, and the question is followed by the estimate as the required
    /// final line ("Estimate: ..."); off, they are the CHRONOLOGY LINES block.
    #[arg(long)]
    pub(crate) no_time_evidence: bool,
    /// Turn off deep topic entries (lore v2 only): the topic tool serves the judged map-reduce
    /// entry of scripts/topic_deep.py (artifacts/topics/deep.jsonl: the story groups, operator files, game records and
    /// dossiers naming the topic, summarized per source, then into What it is, Traits, History, Society and politics,
    /// Notable people, Key events, Open questions; the judge's unsupported sentences dropped) instead of the summary, for
    /// a question about the topic as a whole (`Tools::broad_topic_question`: at most one content word besides the
    /// topic's name).
    #[arg(long)]
    pub(crate) no_deep_topics: bool,
    /// Turn off term lookup: a short what-is question whose term occurs in P4x as one
    /// to three words written together or apart ("khagan quest" is "Khaganquest") searches with the corpus's spelling
    /// and gets, before the passages, the 2 P4x chunks naming the term most (records and files first) and a glossary
    /// of the served topics those chunks name (the first two sentences of each summary, at most 3).
    #[arg(long)]
    pub(crate) no_term_lookup: bool,
    /// Turn off the answer category check: a who/which question that constrains its answer's category ("not playable",
    /// "NPC", "playable operator", "from <nation>") has the names its answer gives checked against the game data
    /// (operator_attributes names, real names, identity links); when every name it gives fails, it is asked once more
    /// with a passage saying which names fail and why, and a second failure gets that note appended.
    #[arg(long)]
    pub(crate) no_answer_check: bool,
    /// The note of the answer check's second ask, set only inside `answer_one`.
    #[arg(skip)]
    pub(crate) check_note: Option<String>,
    /// Turn off the rarity order of the term-lookup glossary: the served topics the term's chunks name are ordered by
    /// first mention instead of rarest first (fewest P4x chunks naming them).
    #[arg(long)]
    pub(crate) no_glossary_rarity: bool,
    /// With `--batch`: print, per question, which question-only detectors fire, without a model (the prefilter of the
    /// A/B runs).
    #[arg(long, hide = true)]
    pub(crate) detect_only: bool,
    /// Model file for the server `ask` starts when none is running.
    #[arg(long, default_value = "models/llm/gemma-4-12b-it-qat-q4_0.gguf")]
    pub(crate) model: PathBuf,
    /// Answer every item of a gold set (JSONL with `qid` and `question`).
    #[arg(long)]
    pub(crate) batch: Option<PathBuf>,
    #[arg(long, default_value = "artifacts/answers.jsonl")]
    pub(crate) out: PathBuf,
}

/// Which lore Trevor serves: v1 is the 46 topics and the 200 dossiers, v2 adds the topics and dossiers
/// chosen from data (scripts/topics.py mine, scripts/dossiers.py characters_v2), in their own corpus directories.
#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub(crate) enum LoreSet {
    V1,
    V2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub(crate) enum RouterKind {
    Model,
    Keywords,
    Off,
}

/// Whether the form call runs: the model and off routers, not the keyword kill switch.
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
    router::RouterPrompt { source: router_source(a), compare: !a.no_reading_compare,
                           wide_topics: a.lore == LoreSet::V2 }
}

/// Whether the model router names the source of a retrieval route (default; `--no-router-source` turns it off).
pub(crate) fn router_source(a: &Args) -> bool {
    a.router == RouterKind::Model && !a.no_router_source && !a.no_route
}

/// Whether a new dossier may join this question's passages: not when it is routed to a typed P4 source, which asks
/// about that text, not about the character.
pub(crate) fn dossier_allowed(q: &str, a: &Args, route: Option<&Route>) -> bool {
    !retrieval_plan(q, a, route).1.is_some_and(|k| k != "archive")
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
        Some(k) => (Some(true), ["module", "voice", "skin", "is", "enemy", "item"].into_iter().find(|x| *x == k)),
    }
}

/// P4 plus the game text the coverage audit found unindexed (`scripts/gametext.py`): story intros, unfetched story
/// scripts, record notes, event archives, mail, operator record intros, world tips and event text, as "gametext" units.
/// Read for P4 routes by default; `--no-game-text` reads P4 alone.
pub(crate) const GAME_TEXT_DIR: &str = "artifacts/p4x";

/// The P3b and P4 directories `ask` uses when `--corpus` is left at its default, for both lore sets: `--lore v2`
/// adds its dossiers and topics by name at answer time instead of building them into these corpora.
pub(crate) const P3B_DIR: &str = "artifacts/p3b";
pub(crate) const P4_DIR: &str = "artifacts/p4";

/// The P4 directory `a` reads: `GAME_TEXT_DIR` when it is built (unless `--no-game-text`), else `P4_DIR`.
pub(crate) fn p4_dir(a: &Args) -> &'static str {
    if !a.no_game_text && std::path::Path::new(GAME_TEXT_DIR).join("chunks.jsonl").exists() {
        GAME_TEXT_DIR
    } else {
        P4_DIR
    }
}

/// Whether the topic tool serves deep entries for this question (the default, lore v2 only; `--no-deep-topics`):
/// only for a question about the topic as a whole (`Tools::broad_topic_question`).
pub(crate) fn deep_for(a: &Args, tools: &Tools, q: &str) -> bool {
    !a.no_deep_topics && a.lore != LoreSet::V1 && tools.broad_topic_question(q)
}

/// The pipeline profile (contract section 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub(crate) enum Profile {
    Full,
    Slim,
    /// Contract v1.1 section 4: the eval's middle arm `e-e2b-mid36` (the cpu rung's default).
    Mid,
    /// `mid` with a 6,000-token passage budget (arm `f-e2b-mid6k`); needs a 12,288-token slot.
    Mid6k,
}

impl Profile {
    pub(crate) const NAMES: &[&str] = &["full", "slim", "mid", "mid6k"];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Slim => "slim",
            Self::Mid => "mid",
            Self::Mid6k => "mid6k",
        }
    }

    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "full" => Some(Self::Full),
            "slim" => Some(Self::Slim),
            "mid" => Some(Self::Mid),
            "mid6k" => Some(Self::Mid6k),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub(crate) enum RetrievalKind {
    Hybrid,
    Bm25,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub(crate) enum BackendKind {
    Llama,
    Openai,
}

/// Set `a` to profile `p`: `full` changes nothing; `slim` is the contract v1 CPU pipeline; `mid` and `mid6k` are
/// exactly the flags of the eval arms `e-e2b-mid36` and `f-e2b-mid6k` (`--router keywords --k 8 --n-predict 250
/// --no-typed-fallback --no-answer-extend --no-answer-check --no-operator-list-check --max-context-tokens 3600|6000` on
/// top of `--profile full`): one answer call, no router call.
pub(crate) fn apply_profile(a: &mut Args, p: Profile) {
    a.profile = p;
    if matches!(p, Profile::Mid | Profile::Mid6k) {
        a.router = RouterKind::Keywords;
        a.k = 8;
        a.max_context_tokens = if p == Profile::Mid { 3600 } else { 6000 };
        a.n_predict = 250;
        a.no_typed_fallback = true;
        a.no_answer_extend = true;
        a.no_answer_check = true;
        a.no_operator_list_check = true;
    }
    if p == Profile::Slim {
        a.router = RouterKind::Keywords;
        a.k = 5;
        a.max_context_tokens = 2000;
        a.n_predict = 250;
        a.no_typed_fallback = true;
        a.no_answer_extend = true;
        a.no_answer_check = true;
        a.no_operator_list_check = true;
    }
}

/// The table tools' options for `a`'s router: the keyword router is the kill switch for all routing work, so it keeps
/// the tools as they were; the exact recap match changes the keyword router's "Explain the ending of Episode 1" (and
/// Episode 0), so it applies to the model and off routers only.
pub(crate) fn tool_options(a: &Args) -> trevor::tools::ToolOptions {
    use trevor::tools::ToolOptions;
    if a.router == RouterKind::Keywords { ToolOptions { canon_v1: a.canon_v1, ..ToolOptions::KEYWORDS } }
        else { ToolOptions { storylines: !a.no_storylines, exact_recap: !a.recap_substring,
                             is_offset: !a.is_numbering_v1, canon_v1: a.canon_v1 } }
}
