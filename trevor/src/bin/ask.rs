//! Answer a lore question from retrieved passages with a local model.
//!
//! Hybrid retrieval over the default corpus (stories plus operator archives), each hit widened by
//! its neighbouring chunks in the same story (`--neighbors`, 0 turns it off), the passages numbered
//! and handed to the model, which must answer only from them, cite them as [n], and say so when
//! they do not hold the answer. Needs a llama-server (`--server`). `--batch <goldset.jsonl>`
//! answers every item of a gold set into `--out` (resumable), for the answer eval.

use std::collections::{BTreeSet, HashMap};
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use trevor::goldgen::llm::{Llm, Request};
use trevor::router::{self, Intent, Knn, KnnParams};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{Runtime, RuntimeArgs};
use trevor::tools::{AnswerConstraint, Route, ToolError, ToolOptions, Tools, TopicPassage};

const SYSTEM: &str = "You answer questions about the Arknights story using only the numbered passages given. \
Cite the passages you use as [n] after each sentence that relies on them. If the passages do not contain \
the answer, say that you could not find it in the story text, and do not guess or use outside knowledge. \
Answer in a few sentences, in plain prose. Never claim that a list is complete or that nothing else exists: \
you see only a few passages of a large story.";
/// Added for time questions (`--no-route` turns routing off).
const TIME_RULE: &str = " Timeline notes follow the passages. Use them for years and order; say when a year is an \
estimate or a bound, and never give a year that neither the notes nor the passages support.";
/// Added for comparison and overview questions, which the story rarely answers in one line.
const SYNTH_RULE: &str = " This question asks for a comparison or overview the story may not state in one place. Answer \
from what the passages show about each part, saying plainly what the story states and what you infer from it, with \
citations; decline only if the passages hold nothing relevant.";

/// Canon and translation questions (4 to 5% of real questions, 0.20 responded): the game text does not rule on canon.
const CANON_RULE: &str = " This question asks whether something is canon, official, a translation difference or a developer \
decision. The story text does not rule on canon: say so in one sentence, then give what the passages show or reference \
about it, with citations, and never state a canon ruling of your own.";
/// Opinion, prediction and ranking questions (16% of real questions): no fact answers them.
const OPINION_RULE: &str = " This question asks for an opinion, a prediction or a ranking. Say that the story does not \
settle it, then give the evidence from the passages on each side, with citations; never invent a ranking or a verdict.";
/// A question about one kind of source text (module story, voice line, skin, Integrated Strategies, enemy file): the
/// first probe answered "what does Kal'tsit's module say" from story passages as if they were the module.
const SOURCE_RULE: &str = " This question asks about one kind of source text (a module story, voice line, skin description, \
Integrated Strategies, enemy entry or operator record). Each passage is labelled with its source. If no passage comes from \
that kind of source, begin by saying that Trevor does not have that text, then give only what the passages do say, naming \
where it comes from.";

/// A judgment question (`--no-question-form` turns the form call off): Ian's "Who is the best cook?" and "Who would commit
/// marriage fraud or adultery more, Midnight or Matsukiri?" were declined outright (2026-09-30), since no passage states a
/// verdict and the keyword test for OPINION_RULE did not fire on either.
const OPINION_FORM_RULE: &str = " This question asks for a judgment the story does not state in one line (a best or most, \
a comparison, a prediction or a what-if). Do not decline it. Name the candidates the passages give evidence about and, for \
each, what the passages say that bears on the judgment, with citations. Then give a hedged conclusion in one sentence (\"On \
this evidence, X seems the likeliest, but the story does not rank them\"), or say the passages give no evidence either way. \
Never state the conclusion as a fact of the story.";
/// A yes-or-no question (Ian's "Has Harold married twice? Does he have children?" was declined whole although the passages
/// held his wife and his daughter, 2026-09-30).
const YES_NO_RULE: &str = " This question asks whether something is true. Answer each part it asks. For a part the passages \
show, answer it with citations. For a part they do not show, say that the passages do not mention it (not that it never \
happened), then give in one or two sentences what they do say about the same person or thing that bears on it, with \
citations.";

/// The verdict-first yes/no rule (`--verdict-first`, opt-in, 2026-10-06): Ian's "Is Deepcolor part of the Church of the
/// Deep?" (ian26) answered "The passages do not mention whether ..." with her file and the Church's scenes in the prompt,
/// and "Has Harold married twice?" (ian5) joined a child and a beast of one scene into one false fact. Refuted 2026-10-06
/// (design/trevor-questions.md section 12): 50 of 50 yes/no answers change, Qwen pairwise 5 wins and 7 losses (h022,
/// h028, r063, r101, r106, t061, t099); ian26 keeps its hedge and ian5 now calls Harold's daughter "Lily".
const YES_NO_VERDICT_RULE: &str = " This question asks whether something is true. Answer each part it asks, each part opening \
with its verdict. For a part the passages show, answer yes or no with citations. For a part they do not show, open with \
\"Not that the passages show\" when they describe that person or thing in some detail without it, or \"The passages do not \
say\" when they barely mention them, then give in one or two sentences what they do say about the same person or thing that \
bears on it, with citations. Keep each fact with the person and scene its passage gives it to: never join two passages, or two \
people of one scene, into one fact.";

/// A whole-story question answered from the summary tree (`scripts/overview.py`; opt-in with `--overview`):
/// "Summarize the Arknights story in one word" was declined, since no passage states the whole story.
const OVERVIEW_RULE: &str = " The passages are Trevor's own summaries, not story text: [1] the whole story, [2] the world \
of Terra, and from [3] on one per storyline, the main story first. The question asks about the story or the world as a \
whole: answer it from these summaries, with citations. If it asks for something the story does not state (one word, a \
theme, a verdict, a comparison), do not decline: say that this is an interpretation, give it, and give the reasons from \
the passages in two to four sentences.";

/// The question-form call: prompt and grammar.
const FORM_SYSTEM: &str = include_str!("../../prompts/form.system.txt");
/// The second look at an opinion form (2026-10-01; `--form-v1` turns it off): the same prompt with opinion kept for
/// judgments across people, and a question about one character's trait, feelings or likes sent to fact or yes_no. The
/// first prompt classed "Is Ulpianus talkative or untalkative?", "is nian actually a nice person?" and "how does
/// greythroat feel about other people now?" as opinion, which gathered other operators' files as candidates and the
/// hedge "the story does not rank them"; the Ulpianus answer quoted Nightblade's file as his, and 6 of the 14 trivia
/// misses with the source chunk in the passages were one-character questions in the opinion form. It runs only when
/// the first call says opinion, and an opinion it confirms keeps the first call's evidence query, so fact and yes/no
/// questions and confirmed opinion questions retrieve exactly as before.
const FORM_SYSTEM_V2: &str = include_str!("../../prompts/form.system.v2.txt");
const FORM_GRAMMAR: &str = include_str!("../../prompts/form.gbnf");
/// The answer-rule flags (2026-10-01; opt-in with `--model-flags`): one more grammar-bound call beside the form call
/// says whether a retrieval question asks about time (TIME_RULE and the timeline notes), canon (CANON_RULE), a comparison
/// or overview (SYNTH_RULE) or one kind of source text (SOURCE_RULE), in place of the English keyword tests. A separate
/// call, so the form and its evidence query, and with them the passages, stay exactly as before.
const FLAGS_SYSTEM: &str = include_str!("../../prompts/flags.system.txt");
const FLAGS_GRAMMAR: &str = include_str!("../../prompts/flags.gbnf");

/// Which answer rules a retrieval question needs (`question_form` with `--model-flags`).
#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Flags {
    asks_time: bool,
    asks_canon: bool,
    asks_synthesis: bool,
    asks_source_text: bool,
}

impl Flags {
    /// The English keyword tests, used unless `--model-flags` is given and the form call runs.
    fn keywords(q: &str) -> Flags {
        Flags { asks_time: is_time_question(q), asks_canon: is_canon_question(q), asks_synthesis: is_synthesis_question(q),
                asks_source_text: is_source_question(q) }
    }

    fn show(self) -> String {
        [(self.asks_time, "time"), (self.asks_canon, "canon"), (self.asks_synthesis, "synthesis"),
         (self.asks_source_text, "source")].iter().filter(|(on, _)| *on).map(|(_, n)| *n).collect::<Vec<_>>().join(",")
    }
}

/// What kind of answer a retrieval question needs (`question_form`).
#[derive(Deserialize, Clone, Debug, Default)]
struct Form {
    form: String,
    /// A search query: for an opinion question in the words a character's file would use for the quality judged, for a
    /// yes/no question in the words a passage that settles it would use.
    evidence: Option<String>,
    /// For an opinion question: the characters it names, as (file story id, the name the question used, the file's
    /// name), set by `answer_one` from the corpus it retrieves from (not part of the model's output).
    #[serde(skip)]
    named: Vec<(String, String, String)>,
    /// `--story-query`: the question in the words the story would use (not part of the form call's output).
    #[serde(skip)]
    story: Option<String>,
    /// The answer-rule flags, from their own call (not part of the form call's output); None = the keyword tests.
    #[serde(skip)]
    flags: Option<Flags>,
    /// The opinion gate held this opinion question back (`opinion_needs_candidates`): no candidates, OPINION_RULE.
    #[serde(skip)]
    gated: bool,
}

/// One grammar-bound call to the answer model (about 0.5 s): fact, or yes_no or opinion with an evidence query. Output
/// that does not parse is a fact question, which leaves the answer as before.
async fn question_form(llm: &Llm, q: &str, v1: bool, flags: bool) -> Result<Form> {
    let ask = |system: &'static str| async move {
        let c = llm.complete(&Request { system, user: &format!("Q: {q}"), grammar: Some(FORM_GRAMMAR), seed: 1,
                                        temperature: 0.0, n_predict: 80, stop: &[] }).await?;
        anyhow::Ok(serde_json::from_str::<Form>(c.content.trim()).unwrap_or_default())
    };
    let first = ask(FORM_SYSTEM).await?;
    let mut form = if v1 || first.form != "opinion" { first } else {
        let second = ask(FORM_SYSTEM_V2).await?;
        if second.form == "opinion" || second.form.is_empty() { first } else { second }
    };
    if flags {
        // Output that does not parse leaves the keyword tests in charge (None).
        let c = llm.complete(&Request { system: FLAGS_SYSTEM, user: &format!("Q: {q}"), grammar: Some(FLAGS_GRAMMAR),
                                        seed: 1, temperature: 0.0, n_predict: 60, stop: &[] }).await?;
        form.flags = serde_json::from_str::<Flags>(c.content.trim()).ok();
    }
    Ok(form)
}

/// `--story-query`: the question as the story's own lines would put it.
const STORY_WORDS: &str = "Rewrite this question about the Arknights story as a search query in the words the story's own \
lines would use: the names it gives, and for what it asks about, the plain, blunt, casual or slang words characters would \
actually say (not formal or clinical terms). Output the query only, one line.";

async fn story_words(llm: &Llm, q: &str) -> Result<Option<String>> {
    let c = llm.complete(&Request { system: STORY_WORDS, user: q, grammar: None, seed: 1, temperature: 0.0, n_predict: 60,
                                    stop: &["\n"] }).await?;
    let t = c.content.trim().trim_matches('"').trim().to_owned();
    Ok((!t.is_empty()).then_some(t))
}

/// Hits of `extra` and `hits` alternately, first seen wins.
fn interleave(extra: Vec<trevor::search::pipeline::Hit>, hits: Vec<trevor::search::pipeline::Hit>) -> Vec<trevor::search::pipeline::Hit> {
    let mut taken: BTreeSet<usize> = BTreeSet::new();
    let mut mixed = Vec::with_capacity(hits.len() + extra.len());
    let (mut x, mut y) = (extra.into_iter(), hits.into_iter());
    loop {
        let (p, q) = (x.next(), y.next());
        if p.is_none() && q.is_none() {
            break;
        }
        mixed.extend([p, q].into_iter().flatten().filter(|h| taken.insert(h.row)));
    }
    mixed
}

/// Whether the form call runs: model, kNN and hybrid routers, not the keyword kill switch.
fn form_on(a: &Args) -> bool {
    a.router != RouterKind::Keywords && !a.no_route && !a.no_question_form
}

/// Character sources: operator files and Trevor's character dossiers.
const CHARACTER_GROUPS: [&str; 2] = ["archive", "profile"];
/// Candidates an opinion question gathers from character sources, and at most this many per character.
const CANDIDATES: usize = 6;
const CANDIDATE_PER_CHARACTER: usize = 2;

/// The retry's query rewrite (see `answer_one`).
const REWRITE: &str = "Rewrite this question about the Arknights story as a short search query in the words the story \
itself would use: character, place and event names and the concrete thing asked about. If the question assumes something \
that sounds wrong or uses slang, use the plain words for what probably happened instead. Output the query only, one line.";

/// `--partial`: when the passages hold part of the answer, give that part and say what is missing,
/// instead of declining outright (off by default; off is the prompt above, unchanged).
const PARTIAL: &str = " If the passages answer only part of the question, give that part with its citations and say \
plainly which part the passages do not cover; decline outright only when they hold nothing relevant.";

#[derive(Parser, Clone)]
#[command(about = "Answer an Arknights lore question from retrieved passages")]
struct Args {
    /// The question (omit with --batch).
    question: Option<String>,
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long, default_value = "http://127.0.0.1:8081")]
    server: String,
    /// Retrieved passages before neighbour expansion.
    #[arg(long, default_value_t = 8)]
    k: usize,
    /// Neighbouring chunks added on each side of every hit, within the same story.
    #[arg(long, default_value_t = 1)]
    neighbors: usize,
    /// Passage budget in model tokens; lower-ranked hits are dropped past it.
    #[arg(long, default_value_t = 9000)]
    max_context_tokens: u32,
    #[arg(long, default_value_t = 400)]
    n_predict: u32,
    /// Answer the covered part of a question instead of declining it whole.
    #[arg(long)]
    partial: bool,
    /// Turn off question routing (table tools, timeline notes for time questions, synthesis for comparisons).
    #[arg(long)]
    no_route: bool,
    /// Which router picks a table tool before retrieval: `model` (one grammar-bound call to the answer model),
    /// `keywords` (the keyword rules of 2026-09-29, the kill switch: identical output on all 1,266 real and
    /// route-check questions), `knn` (nearest labelled examples by embedding, no LLM), `off` (no table tools; the
    /// policy lines and timeline notes stay, unlike --no-route).
    #[arg(long, value_enum, default_value_t = RouterKind::Model)]
    router: RouterKind,
    /// Labelled example questions for `--router knn` (written by --label-intents).
    #[arg(long, default_value = "eval/intents.jsonl")]
    intents: PathBuf,
    /// Embedder of the kNN router (models/gte-trevor is the fine-tuned one). Separate from --model-dir, which must
    /// stay the model the corpus vectors were made with.
    #[arg(long, default_value = "models/gte-modernbert-base")]
    knn_model_dir: PathBuf,
    #[arg(long, default_value = "model_int8.onnx")]
    knn_onnx: String,
    /// Neighbours that vote.
    #[arg(long, default_value_t = 5)]
    knn_k: usize,
    /// A table tool needs its nearest example at this cosine or above. 0.80 with --knn-min-share 0.7 is the
    /// leave-one-out pick on the training 80% of eval/intents.jsonl (2026-09-29, gte-modernbert-base, k 5): test table
    /// precision 1.000 on 3 answers, recall 0.167; 0 and 0.6 gave precision 0.611 on the training sweep.
    #[arg(long, default_value_t = 0.80)]
    knn_min_sim: f32,
    /// A table tool needs this share of the similarity-weighted vote.
    #[arg(long, default_value_t = 0.7)]
    knn_min_share: f32,
    /// Route a question that is itself a labelled example by the other examples (the route check uses it, so the
    /// kNN router is not scored on its own labels).
    #[arg(long)]
    knn_holdout: bool,
    /// Label every question of the Reddit (without categories X and P), Discord and route-check sets with the model
    /// router into this file (resumable); route-check expectations override the model where they conflict.
    #[arg(long)]
    label_intents: Option<PathBuf>,
    /// Paraphrases of the training questions (`--paraphrase-intents`) added to the kNN examples; off by default.
    #[arg(long)]
    knn_synth: Option<PathBuf>,
    /// Write --paraphrases rewrites of every table-labelled training question, each checked by the model router.
    #[arg(long)]
    paraphrase_intents: Option<PathBuf>,
    #[arg(long, default_value_t = 20)]
    paraphrases: usize,
    /// The hybrid router takes the kNN decision when its nearest example is at this cosine or above...
    #[arg(long, default_value_t = 0.92)]
    hybrid_min_sim: f32,
    /// ...and its vote share at least this; otherwise it asks the model router.
    #[arg(long, default_value_t = 0.9)]
    hybrid_min_share: f32,
    /// Evaluate the kNN router on a seeded 80/20 split of --intents and exit.
    #[arg(long)]
    knn_eval: bool,
    /// Keep the reading guide as it was before it led with the game's Storylines links (2026-09-29).
    #[arg(long)]
    no_storylines: bool,
    /// Ignore the topic tool: no topic summary is added to retrieval.
    #[arg(long)]
    no_topics: bool,
    /// With `--lore v2`, add a new dossier also to a question routed to a typed source (IS, voice, module, outfit, item,
    /// enemy), as on 2026-10-02 morning, when it lost r021 ("the same Tin Man from the random encounter in IS2?").
    #[arg(long)]
    dossier_on_typed: bool,
    /// With `--lore v2`, do not add the game-data passage and inference rule to a question about a named character's
    /// body, race features, abilities or age (2026-10-02).
    #[arg(long)]
    no_game_data: bool,
    /// v1: the 46 topics and the P3b and P4 built with the 200 dossiers, as before 2026-10-01; v2 (the default since
    /// 2026-10-03; 0 losses on real probe, held-out and gold): plus the topics and dossiers chosen from data, a new
    /// dossier and a game-data passage at answer time. `--lore v1` is the kill switch.
    #[arg(long, value_enum, default_value = "v2")]
    lore: LoreSet,
    /// Recap stories by the old substring test on story ids (Episode 1 then also takes Episodes 10 to 16); the
    /// keyword router always uses it, since the exact test changes its "ending of Episode 1" answer.
    #[arg(long)]
    recap_substring: bool,
    /// Do not let the model router name the source of a retrieval route (story, operator_file, module,
    /// voice, skin, is, enemy, item, any), which picks P4 and the preferred kind instead of `source_kind`'s keyword
    /// list. The source is the default since 2026-09-30: on the 32 real-probe questions it moves or that are
    /// source-specific, responds 7 to 14 (7 gained, 0 lost), faithfulness 0.712 to 0.762, gold set v2 1 gained of 4
    /// moved. This flag restores the router before it (`--route-only` byte-identical on 266 questions).
    #[arg(long)]
    no_router_source: bool,
    /// Number Integrated Strategies runs as the data's ids do (rogue_1 = IS 1), as before 2026-09-30; the game counts
    /// Ceobe's Fungimist as IS #1, so rogue_1 Phantom & Crimson Solitaire is IS #2.
    #[arg(long)]
    is_numbering_v1: bool,
    /// Do not add lore to a question about an outfit, item, module or enemy (before 2026-09-30). By default the story
    /// and operator-file passages its best unit's text retrieves follow the unit, other operators' units are dropped,
    /// and the answer has two parts, the second opening "Lore context:" (see `LORE_KINDS` for why operator files and
    /// voice lines are left out). Default since round 6c (2026-09-30): Ian's Angelina outfit question answers in two
    /// parts, r046 responds 0 to 1 (faithful 6/6), r102 unchanged, 0 losses, `--route-only` identical on 266 questions.
    #[arg(long)]
    no_source_lore: bool,
    /// Do not add the full name after a short name the question uses ("kal" -> "kal (Kal'tsit)"), for retrieval
    /// and the answer (before 2026-09-30).
    #[arg(long)]
    no_name_expansion: bool,
    /// The model router has no reading_compare tool ("can I read X after Y", "does X spoil Y"), as before 2026-09-30.
    #[arg(long)]
    no_reading_compare: bool,
    /// No question-form call (before 2026-09-30). By default one more grammar-bound Gemma call sorts a retrieval
    /// question into fact, yes_no or opinion with an evidence query; a yes/no question interleaves the evidence query's
    /// hits and gets YES_NO_RULE, a judgment question gathers candidates from operator files and dossiers (named
    /// characters first, found by codename, real name or identity link) and gets OPINION_FORM_RULE. Measured on 63
    /// real-probe questions: responds 42 to 52 (13 gained, 3 lost: r030, r046, r080), handles 26 to 23 of 27,
    /// faithfulness 0.903 to 0.895; 6 fact controls byte-identical. Made the default by Ian (2026-09-30, +13/-3).
    #[arg(long)]
    no_question_form: bool,
    /// No second look at an opinion form (before 2026-10-01): the first call's opinion stands even for a question about
    /// one character's trait or feelings (see FORM_SYSTEM_V2).
    #[arg(long)]
    form_v1: bool,
    /// Add OPINION_RULE on the keyword test ("favorite", "strongest", ...) even when the form call ran and chose fact or
    /// yes_no (before 2026-10-01). By default the form call's choice decides: "What is Insider's favorite food?" (form
    /// fact) opened "The story does not settle on a single favorite food", and trivia t064 "what's blaze's favorite
    /// yanese saying?" hedged its right answer away the same way.
    #[arg(long)]
    keyword_opinion: bool,
    /// Decide TIME_RULE with the timeline notes, SYNTH_RULE, CANON_RULE and SOURCE_RULE by one more grammar-bound Gemma
    /// call after the form call (prompts/flags.system.txt) instead of the English keyword tests. Opt-in (2026-10-01):
    /// on the questions whose flags differ it lost answers on real probe, gold and Ian's set (design/trevor-questions.md
    /// section 12); without it the keyword tests decide, exactly as before.
    #[arg(long)]
    model_flags: bool,
    /// Give the model router an overview tool (opt-in, 2026-10-01): a question about the whole story or the whole world
    /// ("Summarize the Arknights story in one word") is answered from the summary tree of `scripts/overview.py` (the
    /// story and world overviews and one summary per storyline) instead of retrieval. Opt-in because its router lines
    /// move one of Ian's 9 retrieval sources (ian3, any to story); a rule added the same day keeps two-event reading
    /// questions on reading_compare (route check 75/76 with it). Without it the router prompt and grammar are exactly
    /// the ones before.
    #[arg(long)]
    overview: bool,
    /// Print the question form of each question of --batch (or the question) as `qid<TAB>form<TAB>evidence`, then
    /// (with --model-flags) the answer-rule flags and the keyword tests' flags, and exit.
    #[arg(long)]
    form_only: bool,
    /// Print each question of --batch (or the question) with its short names expanded, and exit (no model).
    #[arg(long)]
    show_expansion: bool,
    /// Answer canon questions from reference counts only, as before the per-run verdicts (2026-09-30).
    #[arg(long)]
    canon_v1: bool,
    /// Set for the single question, which `main` routes before loading the index.
    #[arg(skip)]
    routed: bool,
    /// Retry a declined answer once with a rewritten query and wider retrieval. Off by default: on 2026-09-29 it
    /// answered 0 of 36 declines on 120 real questions (rewrite only) and 0 on gold set v2 (rewrite plus 16 passages),
    /// at about 6 s more per question.
    #[arg(long)]
    retry: bool,
    /// Print the table answer when a table route answers, else the line RETRIEVAL, and exit without loading a
    /// model or the index (scripts/route-check.py runs the routing regression set with it).
    #[arg(long)]
    route_only: bool,
    /// Do not start a llama-server when none answers at --server (fail instead, as before).
    #[arg(long)]
    no_spawn: bool,
    /// The shared model lock (`<pid> <label>`): while another job holds it, `ask` waits for it instead of sharing that
    /// job's server (2026-09-30: `ask` reused a canon job's reasoning-on Gemma and hung). A lock held by an ancestor of
    /// this process (the script that started `ask`) is ours. `ask` writes the lock itself for a server it starts.
    #[arg(long, default_value = "/private/tmp/claude-501/trevor-llm.lock")]
    lock: PathBuf,
    /// Fail at once when another job holds the model lock, instead of waiting.
    #[arg(long)]
    no_wait: bool,
    /// Share the server even when another job holds the model lock (before 2026-09-30).
    #[arg(long)]
    ignore_lock: bool,
    /// A second retrieval query in the words the story itself would use (one more Gemma call), its hits interleaved
    /// with the question's own: "What operator declared that they want to pee in public?" never retrieves Chiave's "see
    /// who can piss farther", which a query in the story's words ranks 2nd (2026-09-30). Opt-in until measured.
    #[arg(long)]
    story_query: bool,
    /// Append every model request's rendered prompt, `prompt_n`, `cache_n` and output to this file (JSON lines), to
    /// check that two runs send the same prompts (2026-10-02). Debug only; the answers are unchanged.
    #[arg(long)]
    dump_prompt: Option<PathBuf>,
    /// Send every request with `cache_prompt: false`: no prompt reuses KV cache from an earlier one, so the same
    /// prompts decode to the same answers whatever ran before (2026-10-02; slower by the reused prefix). Off by default.
    #[arg(long)]
    no_cache_prompt: bool,
    /// The router sends a question about one operator's everyday life (likes, downtime, opinions of the base) to
    /// their voice lines (2026-10-03; opt-in until measured, see design/trevor-questions.md section 12).
    #[arg(long)]
    voice_rule: bool,
    /// Leave out the explicit-estimate rule a question asking an age gets with the game-data passage (the rule is the
    /// default since 2026-10-03).
    #[arg(long)]
    no_age_estimate: bool,
    /// Race topics serve the summary after the speaker-aware trait check (`topics.py traitcheck2`, 2026-10-03), which
    /// drops a body-feature sentence resting on a line about someone of another race (Durin "thick tails"). Opt-in.
    #[arg(long)]
    speaker_trait_check: bool,
    /// Operator art: the router's "art" source retrieves the model-written art descriptions (artifacts/p4-art, built
    /// by scripts/art_captions.py units and build-units; 2026-10-03, opt-in until measured).
    #[arg(long)]
    art: bool,
    /// Replace the last N of the k fused hits with the Ettin reranker's best hits (of the fused top 40) not already
    /// among them. Reranking outright fails on gold v2 over P3b (recall@10 0.732 to 0.605, 22 lost and 8 gained,
    /// 2026-09-30) but the reranker's first pick is often a passage fusion ranks lower: fused top 7 plus its best new
    /// hit against fused top 8 is recall +0.063 [+0.023, +0.109], 10 gained and 2 lost. Opt-in (0) until the answers
    /// are measured.
    #[arg(long, default_value_t = 0)]
    rerank_add: usize,
    /// Turn off the origin chain (default since 2026-10-03, item 1): a why/when/how-it-began question gets ORIGIN_RULE
    /// (trace the causes back to the earliest one the passages show) and the earlier scenes of the story its search keeps
    /// hitting, placed first (ian15 now starts from Swire's kidnapping; 0 losses; see design/trevor-questions.md section 12).
    #[arg(long)]
    no_origin_chain: bool,
    /// Relationship and admiration questions (2026-10-03, item 2): RELATION_RULE keeps literal relations apart from what
    /// characters call each other and keeps distinct people with one title apart. The default since 2026-10-05 night, for a
    /// relation question naming two people (or an admiration question), see `relation_rule_any`; this flag is kept and
    /// changes nothing.
    #[arg(long)]
    relation_rule: bool,
    /// Kill switch for RELATION_RULE (default since 2026-10-05 night: 0 pairwise losses on the 14 relationship questions,
    /// 4 wins: g0099, ian19, ian23, relx299).
    #[arg(long)]
    no_relation_rule: bool,
    /// With `--relation-rule`, apply it also to a question naming a nation, place, race or organization topic (since
    /// 2026-10-05 the rule is for people only: "Are Petrams related to Aegirs?", real r108, was lost to it twice).
    #[arg(long)]
    relation_rule_all: bool,
    /// Kill switch for the scoped scenes (default since 2026-10-05 night): a question naming a main episode or event and a
    /// character who speaks or is named in it gets, first, that event's scenes with the character ranked by the question.
    #[arg(long)]
    no_scoped_scenes: bool,
    /// Kill switch for the character-scoped scenes (2026-10-06, `character_scenes`): a question naming no event and one
    /// character as the subject of a said or done act ("During which event Mon3tr suggested assassinating a child") gets
    /// excerpts of the scenes where that character speaks, their lines ranked by the reranker.
    #[arg(long)]
    no_character_scenes: bool,
    /// The line pool (opt-in, 2026-10-06, Ian's item 1): the lines of the hybrid top `LINE_POOL_CHUNKS` chunks, each with the
    /// line before it, ranked by the reranker; when the best line lies outside the retrieved passages, the best
    /// `LINE_POOL_LINES` such lines come as excerpts after the retrieved passages, within `LINE_POOL_TOKENS` on top.
    /// Measured 2026-10-06: gold correct +7 -1 on 30 (g0034 lost), but Ian's ian17 and ian30 lost pairwise, so opt-in.
    #[arg(long)]
    line_pool: bool,
    /// With `--line-pool`, add the excerpts even when the best pool line is already among the retrieved passages.
    #[arg(long)]
    line_pool_all: bool,
    /// With `--line-pool`, fire only when the best line outside the retrieved passages scores at least this (reranker
    /// logit). TREVOR_LINE_POOL_DEBUG=1 prints each question's best score; TREVOR_LINE_POOL_DETECT=1 (measurement) skips
    /// the answer call and returns "line-pool detect: <excerpts>".
    #[arg(long)]
    line_pool_min: Option<f32>,
    /// Kill switch for the answer extension (default since 2026-10-06): an answer that stopped at `--n-predict` is asked
    /// again with twice the limit (Ian's ian27 stopped mid-sentence at 400 tokens).
    #[arg(long)]
    no_answer_extend: bool,
    /// The verdict-first yes/no rule (opt-in, refuted 2026-10-06, Ian's item 2): YES_NO_VERDICT_RULE instead of YES_NO_RULE;
    /// on the 50 yes/no questions Qwen pairwise 5 wins, 7 losses, 38 ties, and ian26 still opens "The passages do not say".
    #[arg(long)]
    verdict_first: bool,
    /// With `--relation-rule`, apply it also to a relation question naming fewer than two people (since 2026-10-05
    /// night the rule needs two named people, or an admiration question: real-style relx26 "what their relationship is ...
    /// Drudge is the big brother" and relx810 "Is Mon3tr related to the Aggeloi?" were lost to it pairwise).
    #[arg(long)]
    relation_rule_any: bool,
    /// Measurement: with --batch, print the scoped scenes each question would get (chunk ids), without a model.
    #[arg(long, hide = true)]
    scoped_only: bool,
    /// Measurement (2026-10-06, Ian's item 1): with --batch of {qid, question, chunk, quote}, the rank of the evidence line
    /// among every line of the corpus (each with the line before it) by line dense and by the reranker, estimated on a
    /// stride sample (TREVOR_LINE_STRIDE, default 30; the reranker on every TREVOR_LINE_RR_STRIDE-th, default 300), next to
    /// the chunk's dense, BM25 and fused ranks. No model server.
    #[arg(long, hide = true)]
    line_study: bool,
    /// The opinion gate (opt-in, 2026-10-05 night; `opinion_needs_candidates`): an opinion question that names no character
    /// or group and asks no who/which gets no candidates from operator files and dossiers and OPINION_RULE ("the story does
    /// not settle it") instead of OPINION_FORM_RULE. Fixes real r030 ("In lore how strong are my favourite characters?") by
    /// the real-probe judge, but loses 3 of 19 touched questions pairwise (r039, r067, r068). Since 2026-10-06 the default
    /// is the narrow gate (`no_opinion_gate`); this flag is night 11's wider reach, measurement only.
    #[arg(long)]
    opinion_gate: bool,
    /// Kill switch for the narrow opinion gate (default since 2026-10-06): the gate of `--opinion-gate` only for a question
    /// the keyword test also calls an opinion ("favourite", "do you think", ...) and that names no word the corpus writes
    /// as a proper name (`names_proper_word`: "Fort barron"), so real r039, r067 and r068 keep their candidates. Fires on
    /// r030 and r116 of the 448 set questions: real-probe responds 1 to 2 of 2 and handles 1 to 2 (r030 regains both),
    /// faithfulness 16/18 to 12/14, Qwen pairwise 2 ties, 0 losses.
    #[arg(long)]
    no_opinion_gate: bool,
    /// When-questions (2026-10-03, item 3): DATE_RULE, end with a labelled year estimate from the timeline notes when
    /// no passage states the year (the age rule's counterpart for dates). Opt-in until measured.
    #[arg(long)]
    date_estimate: bool,
    /// The router's `cross_ref` tool (2026-10-03, item 4): cast-wide game-data cross-references, first "boss enemies
    /// who are playable operators" (artifacts/crossref/, scripts/crossref.py). Opt-in until measured.
    #[arg(long)]
    cross_ref: bool,
    /// Off-topic guard (2026-10-03, item 8): a question that names nothing Trevor's tables know gets LORE_ONLY_RULE
    /// (Trevor answers Arknights lore only). Opt-in until measured.
    #[arg(long)]
    lore_only: bool,
    /// Turn off the race-trait rule (default since 2026-10-03, item 11, no model): race topics serve the summary after
    /// `topics.py traitrule`, where a body-feature sentence stays only when a cited line gives the feature to a speaker or
    /// subject of that race in operator_attributes.jsonl (drops Durin "thick tails" and Savra "a horn"; 0 losses).
    #[arg(long)]
    no_race_trait_rule: bool,
    /// Turn off the typed fallback (default since 2026-10-03, item 10; no router change): a declined answer to a
    /// question naming one operator, not routed to a typed source, is tried again with that operator's voice lines,
    /// module stories or art units first, and kept only when it cites one of that operator's own units (trivia correct
    /// 0 to 3 of 3 changed, gold 0 losses, Ian's 25 unchanged; see design/trevor-questions.md section 12).
    #[arg(long)]
    no_typed_fallback: bool,
    /// Turn off the game text (default since 2026-10-03, item 6): P4 routes read artifacts/p4x, P4 plus the game text
    /// the coverage audit found unindexed (scripts/gametext.py; real probe responds 2 to 4 of 6 changed, 0 losses).
    #[arg(long)]
    no_game_text: bool,
    /// Turn off the appearance index (2026-10-03, item 7): "What chapters does Elysium appear in?" is answered, before
    /// the router, by the `appearances` tool (scripts/appearances.py: the story groups where a character speaks, and apart
    /// those where they are only named).
    #[arg(long)]
    no_appearances: bool,
    /// Turn off the cross-reference trigger (2026-10-03, leftover d): a question with "boss" and a joining word goes to
    /// the `cross_ref` tool before the router, with no router line (the `--cross-ref` line moved 34 routes).
    #[arg(long)]
    no_cross_ref_trigger: bool,
    /// Turn off the design-inspiration tool (2026-10-04, Ian's ian11): a question with a design word ("based off an animal",
    /// "inspired by", "motif", "design inspiration") whose content word is an inferred subject or an operator name in
    /// artifacts/entities/design_infer.jsonl (scripts/design_infer.py: deduced from the game data, checked by a second
    /// model) is answered, before the router, by the `design_basis` tool, both readings, labelled as Trevor's inference.
    #[arg(long)]
    no_design_basis: bool,
    /// Two-step design deduction (2026-10-05, opt-in): a design question the `design_basis` table cannot answer goes,
    /// before the router, to a deduction from the game's art and text (artifacts/entities/design_evidence.jsonl, from
    /// scripts/design_infer.py evidence). Forward ("which operator is based on an animal with phantom in its name"):
    /// Gemma lists real-world candidates with their visible features from general knowledge, a data shortlist (inverse-
    /// frequency feature words over the evidence rows) keeps 8 operators, and Gemma names the ones whose evidence shows the
    /// features. Reverse ("what animal is Kirara based on"): Gemma reads the named operator's evidence. Labelled "Trevor's
    /// deduction from the game's art and text"; says plainly when nothing in the game shows a match.
    #[arg(long)]
    design_deduce: bool,
    /// The wiki-based answers of the morning of 2026-10-04, for measurement only (2026-10-04, Ian: the wiki is a
    /// reference, never a source): the design tool reads the wiki trivia table (trevor::reference) and the reading-guide
    /// and death texts credit the wiki as before. Pair it with the wiki-built tables (MAIN_DATES=wiki reading_guide.py
    /// dates and build, DEATH_CONFIRM=wiki deaths.py build) to reproduce the old answers.
    #[arg(long)]
    wiki_legacy: bool,
    /// Turn off IS-ending passages (2026-10-03, item 2): a question naming an Integrated Strategies run and one of its
    /// endings (number, ordinal or name) or its endbooks is answered from that ending's scene and endbook stories, with
    /// the game's ending order, without the router.
    #[arg(long)]
    no_is_ending: bool,
    /// The named topic (2026-10-03, item 5; opt-in after losses): a retrieval question that names one served nation,
    /// place, race or organization topic and no operator gets its summary although the router chose no topic ("factions
    /// in dossoles"). Measured with --topic-events and --place-nation: held-out responds 10 to 7 of 10 changed (lost
    /// h004, h017, h033, near-identical declines), real 10 to 11 of 14, gold 16 to 17 of 24; ian30 answers the factions.
    #[arg(long)]
    named_topic: bool,
    /// With `--named-topic`, the night 7 reach: every question naming one such topic (since 2026-10-05 only a question about
    /// the topic as a whole, `Tools::broad_topic_question`).
    #[arg(long)]
    named_topic_all: bool,
    /// Topic events (2026-10-03, items 3 and 4; opt-in, measured with --named-topic): a topic summary comes with the P2
    /// event summaries of the 3 story groups that name the topic most (scripts/topic_groups.py).
    #[arg(long)]
    topic_events: bool,
    /// Turn off the term rule (2026-10-03, item 6): a "what is/are X" question gets TERM_RULE (plain words, a short
    /// definition of each in-world term used).
    #[arg(long)]
    no_term_rule: bool,
    /// The place's nation (2026-10-03, item 5; opt-in, measured with --named-topic): a place topic comes with the topic
    /// of the nation its summary names most (Dossoles: Bolívar).
    #[arg(long)]
    place_nation: bool,
    /// Turn off chronology lines for the subject (2026-10-03, leftover c): a when-question gets the dated lines of
    /// artifacts/chrono/terra_history.md that name its subject, and DATE_RULE (a labelled estimate).
    #[arg(long)]
    no_chrono_subject: bool,
    /// Turn off the own-file fallback (2026-10-03, leftover b): an answer to a one-operator question that cites only
    /// that operator's file is tried again like a decline (typed fallback), kept only when it cites the operator's own
    /// voice line, module or art unit.
    #[arg(long)]
    no_own_file_fallback: bool,
    /// Identity notes (2026-10-03, leftover a; opt-in): for a who/which/identity question, when a passage is the source
    /// of a stated identity link of identities.v2.json (Iris's file: Mabel was Bluishsilver), a note after the passages
    /// states the link and its quote; inferred links sharing a name follow, labelled. ian24 still declines with the note
    /// in its prompt, and the prefilter undercounted where it fires (ian1, ian3, ian7, ian18 changed outside it).
    #[arg(long)]
    identity_notes: bool,
    /// Turn off time evidence (2026-10-03 night 8; default, 0 losses on the 3 questions it fires on, ian17 now
    /// estimates "between 1086 and 1090"): when chronology lines name a when-question's subject, they come as a numbered
    /// TIME EVIDENCE block right before the question, and the question is followed by the estimate as the required
    /// final line ("Estimate: ..."); off, they are the CHRONOLOGY LINES block before the identity notes, as before.
    #[arg(long)]
    no_time_evidence: bool,
    /// Identity labels (2026-10-03 night 8; opt-in): for a who/which/identity question, a passage that is the source of a
    /// stated identity link of identities.v2.json carries the resolved link in its label ("Bluishsilver, real name
    /// Mabel, per ..."), both names proper names by the corpus (the lowercase form is under 1% of the capitalized
    /// count: "Operator" 1,168 of 6,260 is dropped).
    #[arg(long)]
    identity_labels: bool,
    /// Turn off deep topic entries (default since 2026-10-05, lore v2 only): the topic tool serves the judged map-reduce
    /// entry of scripts/topic_deep.py (artifacts/topics/deep.jsonl: the story groups, operator files, game records and
    /// dossiers naming the topic, summarized per source, then into What it is, Traits, History, Society and politics,
    /// Notable people, Key events, Open questions; the judge's unsupported sentences dropped) instead of the summary, for
    /// a question about the topic as a whole (`Tools::broad_topic_question`: at most one content word besides the
    /// topic's name). Night 9 served it to every topic question and lost held-out h033 and real r079 (both specific
    /// questions); gated, it changes ian28 alone of the 447 (Qwen pairwise: the entry wins) and the 14 topic-routed
    /// questions with an entry are byte-identical.
    #[arg(long)]
    no_deep_topics: bool,
    /// Turn off term lookup (default since 2026-10-03 night 9; fires on 4 of 447 questions, changes 2, 0 losses: ian31
    /// now defines the Khaganquest from its game-text record): a short what-is question whose term occurs in P4x as one
    /// to three words written together or apart ("khagan quest" is "Khaganquest") searches with the corpus's spelling
    /// and gets, before the passages, the 2 P4x chunks naming the term most (records and files first) and a glossary
    /// of the served topics those chunks name (the first two sentences of each summary, at most 3).
    #[arg(long)]
    no_term_lookup: bool,
    /// Identity expansion (2026-10-03 night 9; opt-in, no gains): a who/which question naming a character, whose
    /// passages include the source of a stated identity link (Iris's file: Mabel was Bluishsilver), also gets the 2
    /// passages where a linked name and the named character appear together (the interlude where Iris looks for Mabel
    /// Grimm), placed right after the link's source passage, each labelled with the resolved link. Measured on the 11
    /// questions it fires on: 0 judged losses, no gains; ian24 answers Sakiko Togawa alone and declines with
    /// --identity-labels.
    #[arg(long)]
    identity_expand: bool,
    /// Turn off the answer category check (default since 2026-10-05): a who/which question that constrains its answer's category ("not
    /// playable", "NPC", "playable operator", "from <nation>") has the names its answer gives checked against the
    /// game data (operator_attributes names, real names, identity links); when every name it gives fails, it is asked
    /// once more with a passage saying which names fail and why, and a second failure gets that note appended. Fires on
    /// ian24 alone of the 447 set questions and on 2 of 1,228 Reddit and Discord questions; changes no default answer
    /// (the default declines ian24); with --identity-expand ian24 goes from Sakiko Togawa (a playable operator) to a decline.
    #[arg(long)]
    no_answer_check: bool,
    /// The note of the answer check's second ask, set only inside `answer_one`.
    #[arg(skip)]
    check_note: Option<String>,
    /// The evidence composition (`compose_evidence`; opt-in, 2026-10-05 night: it fires on ian24 alone of the 448 set
    /// questions, which still declines with the composed evidence, so it changes no answer and costs one more call).
    #[arg(long)]
    compose_evidence: bool,
    /// The composed passages of the evidence composition's ask, set only inside `answer_one`.
    #[arg(skip)]
    compose: Vec<(String, String, String)>,
    /// Serve the deep entry to every topic question, as `--deep-topics` did on night 9 (measurement only; lost h033 and
    /// r079 then).
    #[arg(long)]
    deep_topics_all: bool,
    /// Turn off the rarity order of the term-lookup glossary (default since 2026-10-05): the served topics the term's
    /// chunks name are ordered by first mention, as on night 9, instead of rarest first (fewest P4x chunks naming them).
    #[arg(long)]
    no_glossary_rarity: bool,
    /// With `--place-nation`, the nation of a place from the summary count of night 7 (Chernobog: Yan) instead of
    /// artifacts/topics/place_nation.json (scripts/place_nation.py, 2026-10-03 night 8; Chernobog: Ursus).
    #[arg(long)]
    place_nation_summary: bool,
    /// With `--batch`: print, per question, which of the 2026-10-03 question-only detectors fire, without a model
    /// (the prefilter of the A/B runs).
    #[arg(long, hide = true)]
    detect_only: bool,
    /// With `--batch`: print the identity questions whose plain top k would get identity notes (needs the index).
    #[arg(long, hide = true)]
    detect_identity: bool,
    /// Model file for the server `ask` starts when none is running.
    #[arg(long, default_value = "models/llm/gemma-4-12b-it-qat-q4_0.gguf")]
    model: PathBuf,
    /// Answer every item of a gold set (JSONL with `qid` and `question`).
    #[arg(long)]
    batch: Option<PathBuf>,
    #[arg(long, default_value = "artifacts/answers.jsonl")]
    out: PathBuf,
}

/// Which lore Trevor serves (2026-10-01): v1 is the 46 topics and the 200 dossiers, v2 adds the topics and dossiers
/// chosen from data (scripts/topics.py mine, scripts/dossiers.py characters_v2), in their own corpus directories.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum LoreSet {
    V1,
    V2,
}

impl LoreSet {
    /// The P3b and P4 directories `ask` uses when `--corpus` is left at its default.
    fn dirs(self) -> (&'static str, &'static str) {
        match self {
            Self::V1 => ("artifacts/p3b", "artifacts/p4"),
            // Since 2026-10-02 v2 retrieves from the v1 corpora: built into P3b, the 336 new dossiers changed the passages
            // of 50 of 120 real-probe questions and reordered others (5 lost), so they are added by name instead.
            Self::V2 => ("artifacts/p3b", "artifacts/p4"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum RouterKind {
    Model,
    Keywords,
    Knn,
    /// kNN when it is sure (--hybrid-min-sim, --hybrid-min-share), else the model router.
    Hybrid,
    Off,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Answer {
    qid: Option<String>,
    question: String,
    answer: String,
    /// Chunk ids of the passages the answer cites, in citation order.
    cited: Vec<String>,
    /// Chunk ids given to the model, in passage order.
    passages: Vec<String>,
    invalid_citations: usize,
    prompt_tokens: u64,
    ms: f64,
    /// The search query of the retry, when the first answer declined and the retry was used.
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_query: Option<String>,
    /// The question form and evidence query, when the form call ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    form: Option<String>,
    /// The text of each generated passage only `--lore v2` adds (new dossier, game data), by passage id, for the judges.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    generated: std::collections::BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Names {
    story: String,
    group: String,
}

/// P4 unit kinds: the typed sources and Trevor's topic summaries, which a lore query leaves out.
const TYPED: [&str; 7] = ["module", "voice", "skin", "is", "enemy", "item", "topic"];

/// The operator a typed unit belongs to, from its heading line: "Outfit: Newsgirl (Amiya)", "Voice lines: Texas",
/// "Module story: Drone Control Module.P (Magallan's module, SUM-X)"; None for IS, enemy and item units.
fn unit_owner(heading: &str) -> Option<String> {
    if let Some(n) = heading.strip_prefix("Voice lines: ") {
        return Some(n.trim().to_owned());
    }
    let open = heading.rfind('(')?;
    let inner = heading[open + 1..].trim_end_matches(')');
    let name = inner.split(['\'', '\u{2019}', ',']).next()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// What `passages` put first for a typed source: how many source units, then how many lore passages after them.
#[derive(Default, Clone, Copy)]
struct Lore {
    units: usize,
    lore: usize,
}

/// Passages for `q`, and for a typed source with its lore (see below) how many units and lore hits lead them.
fn passages(rt: &mut Runtime, q: &str, a: &Args, kind: Option<&str>, form: Option<&Form>) -> Result<(Vec<usize>, Option<Lore>)> {
    let cfg = RetrievalConfig { mode: Mode::Hybrid, k: a.k, ..RetrievalConfig::default() };
    let mut hits = rt.retrieve(q, &cfg)?;
    if a.rerank_add > 0 {
        let keep = a.k.saturating_sub(a.rerank_add).min(hits.len());
        let rr = rt.retrieve(q, &RetrievalConfig { rerank_top: 40, k: 40, ..cfg })?;
        let mut head: Vec<_> = hits[..keep].to_vec();
        for h in rr {
            if head.len() >= a.k {
                break;
            }
            if !head.iter().any(|x| x.row == h.row) {
                head.push(h);
            }
        }
        let rest: Vec<_> = hits.into_iter().filter(|h| !head.iter().any(|x| x.row == h.row)).collect();
        hits = head.into_iter().chain(rest).collect();
    }
    let mut lore_added: Option<Lore> = None;
    let evidence = form.and_then(|f| f.evidence.as_deref()).filter(|e| !e.trim().is_empty());
    // A yes/no question interleaves the hits of its evidence query with its own: "Has Harold married twice? Does he have
    // children?" ranked the lines naming his wife and daughter 29th to 44th, and "Harold wife daughter children married"
    // ranks them 1st to 8th (2026-09-30).
    if let (Some(ev), Some("yes_no")) = (evidence, form.map(|f| f.form.as_str())) {
        let extra = rt.retrieve(ev, &cfg)?;
        hits = interleave(extra, hits);
    }
    // `--story-query`: the story-words query's hits interleaved with the question's (and the yes/no evidence's).
    if let Some(sq) = form.and_then(|f| f.story.as_deref()) {
        let extra = rt.retrieve(sq, &cfg)?;
        hits = interleave(extra, hits);
    }
    let evidence = evidence.filter(|_| form.is_some_and(|f| f.form == "opinion"));
    // A judgment question gathers candidates first: the evidence query ("skilled cook, culinary talent") over a wide list,
    // kept to operator files and dossiers, at most two chunks per character. "Who is the best cook?" retrieved the Dungeon
    // Meshi collaboration and no operator file in its top 100; the evidence query puts Matterhorn's, Hung's, Lee's and
    // Gummy's files in the top 52 (2026-09-30). They take no neighbours, so six candidates cost six chunks. Characters the
    // question names come first, each from its own files: "Midnight or Matsukiri?" otherwise filled four of six places
    // with other operators' files and dropped Midnight's.
    let mut bare: BTreeSet<usize> = BTreeSet::new();
    if let Some(ev) = evidence {
        let mut per: HashMap<String, usize> = HashMap::new();
        let mut cands = Vec::new();
        // At most two files per name the question uses ("Margaret" is Nearl's and Nearl the Radiant Knight's real name
        // and a dossier's title), six in all.
        let mut per_name: HashMap<String, usize> = HashMap::new();
        let named: Vec<_> = form.map(|f| f.named.clone()).unwrap_or_default().into_iter()
            .filter(|(_, asked, _)| { let n = per_name.entry(asked.to_lowercase()).or_default(); *n += 1; *n <= 2 }).take(CANDIDATES).collect();
        for (sid, asked, file) in named {
            let own = rt.retrieve(&format!("{file} {asked} {ev}"), &RetrievalConfig { k: 60, ..cfg })?;
            let mine: Vec<_> = own.into_iter().filter(|h| rt.store.chunks[h.row].story_id == sid).take(CANDIDATE_PER_CHARACTER).collect();
            // A named character always gets its file: the first chunk when the query ranks none of it in the top 60.
            if mine.is_empty() {
                if let Some(&row) = rt.store.story_rows(&sid).first() {
                    cands.push(trevor::search::pipeline::Hit { row, score: 0.0, dense_rank: None, bm25_rank: None, fused_rank: None });
                }
            }
            cands.extend(mine);
            per.insert(sid, CANDIDATE_PER_CHARACTER);
        }
        let room = CANDIDATES.saturating_sub(cands.len());
        let wide = rt.retrieve(ev, &RetrievalConfig { k: 60, ..cfg })?;
        cands.extend(wide.into_iter().filter(|h| {
            let c = &rt.store.chunks[h.row];
            CHARACTER_GROUPS.contains(&c.group_id.as_str()) && {
                let n = per.entry(c.story_id.clone()).or_default();
                *n += 1;
                *n <= CANDIDATE_PER_CHARACTER
            }
        }).take(room));
        bare = cands.iter().map(|h| h.row).collect();
        hits = cands.into_iter().chain(hits.into_iter().filter(|h| !bare.contains(&h.row))).collect();
    }
    // A question about one kind of source (P4 units: module, voice, is, enemy, skin, item) gets the best 3 chunks of
    // that kind first from a wider candidate list, so story passages cannot crowd them out.
    if let Some(kind) = kind {
        let wide = rt.retrieve(q, &RetrievalConfig { k: 60, ..cfg })?;
        let lore_here = lore_on(a) && LORE_KINDS.contains(&kind);
        let mut pref: Vec<_> = wide.into_iter().filter(|h| rt.store.chunks[h.row].group_id == kind)
            .take(if lore_here { 12 } else { 3 }).collect();
        // With lore on, units of another operator do not follow the best unit ("What does Angelina's Bloodline of
        // Combat outfit say?" put Swire's Bloodline of Combat outfit third, 2026-09-30).
        if lore_here {
            let owner_of = |row: usize| unit_owner(rt.store.chunks[row].text.lines().next().unwrap_or_default());
            if let Some(o) = pref.first().and_then(|h| owner_of(h.row)) {
                pref.retain(|h| owner_of(h.row).is_none_or(|x| x == o));
            }
            pref.truncate(3);
        }
        // Lore for a typed source (Ian, 2026-09-30: "What does Angelina's Bloodline of Combat outfit say?" answered with
        // the outfit's two sentences and no lore): the best unit's own text is a second query, and its best story and
        // operator-file passages follow the units, so the answer can explain who and what the unit names.
        let mut lore = Vec::new();
        if lore_here {
            if let Some(top) = pref.first() {
                let text: String = rt.store.chunks[top.row].text.chars().take(600).collect();
                let lq = rt.retrieve(&text, &RetrievalConfig { k: 40, ..cfg })?;
                lore = lq.into_iter().filter(|h| !TYPED.contains(&rt.store.chunks[h.row].group_id.as_str())).take(4).collect();
            }
        }
        if lore_here && !pref.is_empty() {
            lore_added = Some(Lore { units: pref.len(), lore: lore.len() });
        }
        let mut taken: BTreeSet<usize> = BTreeSet::new();
        let head: Vec<_> = pref.into_iter().chain(lore).filter(|h| taken.insert(h.row)).collect();
        hits = head.into_iter().chain(hits.into_iter().filter(|h| !taken.contains(&h.row))).collect();
    }
    // The origin chain (`--no-origin-chain` turns it off): a why/how-it-began question also gets the earlier scenes of the story its search keeps hitting.
    // "Why did swire join the LGD?" ranks her story's L.G.D. years (#0009) 6th and its opening chunks 7th and 8th, and
    // the 9,000-token budget keeps only the first 6 hits' clusters, so the kidnapping (#0003, 20th) and the badge (#0005)
    // never reach the answer. The story with the most chunks in the top 20 (at least 3) goes first: its chunks there that
    // come before its best-ranked hit, without neighbours, in story order (at most 5), then that hit with its neighbours,
    // then the other hits. Two earlier versions (2026-10-03) added nothing to this question: the earliest top-60 scene of
    // the top 3 stories (her story is the 6th distinct one), and the dominant story's earlier chunks placed after the
    // first 4 hits while "earlier" was measured from the lowest position among the hits (#0000, so nothing came first).
    if !a.no_origin_chain && !a.no_route && asks_origin(q) {
        let wide = rt.retrieve(q, &RetrievalConfig { k: 20, ..cfg })?;
        let mut count: HashMap<&str, usize> = HashMap::new();
        for h in &wide {
            *count.entry(rt.store.chunks[h.row].story_id.as_str()).or_default() += 1;
        }
        let top = count.iter().filter(|(_, n)| **n >= 3).max_by_key(|(sid, n)| (**n, std::cmp::Reverse(**sid))).map(|(s, _)| (*s).to_owned());
        if let Some(sid) = top {
            let rows = rt.store.story_rows(&sid);
            let pos = |row: usize| rows.iter().position(|&r| r == row).unwrap_or(usize::MAX);
            let lead = hits.iter().chain(wide.iter()).find(|h| rt.store.chunks[h.row].story_id == sid).cloned();
            if let Some(lead) = lead {
                let mut add: Vec<_> = wide.iter().filter(|h| rt.store.chunks[h.row].story_id == sid && pos(h.row) < pos(lead.row))
                    .cloned().collect();
                add.sort_by_key(|h| pos(h.row));
                add.truncate(5);
                bare.extend(add.iter().map(|h| h.row));
                let first: Vec<_> = add.into_iter().chain(std::iter::once(lead)).collect();
                let rest: Vec<_> = hits.into_iter().filter(|h| !first.iter().any(|x| x.row == h.row)).collect();
                hits = first.into_iter().chain(rest).collect();
            }
        }
    }
    let mut chosen: Vec<usize> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut budget = 0u32;
    for h in hits {
        let c = &rt.store.chunks[h.row];
        let rows = rt.store.story_rows(&c.story_id);
        let pos = rows.iter().position(|&r| r == h.row).unwrap_or(0);
        let nb = if bare.contains(&h.row) { 0 } else { a.neighbors };
        let lo = pos.saturating_sub(nb);
        let hi = (pos + nb).min(rows.len().saturating_sub(1));
        let cluster: Vec<usize> = rows[lo..=hi].iter().copied().filter(|r| !seen.contains(r)).collect();
        let cost: u32 = cluster.iter().map(|&r| rt.store.chunks[r].token_count).sum();
        if !chosen.is_empty() && budget + cost > a.max_context_tokens {
            continue;
        }
        budget += cost;
        for r in cluster {
            seen.insert(r);
            chosen.push(r);
        }
    }
    Ok((chosen, lore_added))
}

/// The characters whose operator file or dossier the question names, as (story id, the name the question used, the
/// file's own name), longest name first. A file's names are its heading's ("Operator archive: Midnight", "Character
/// profile: Kal'tsit (also known as AMa-10, ...)"), plus, for each of them, the operator's real name from
/// `real_names.jsonl` (and its first word when it has two or more: "Margaret Nearl" gives "Margaret") and the names of its
/// identity links. Matched as whole words, case folded, 3 letters or more. "Margaret vs Degenbrecher" found no file
/// for Margaret before the real names were added: Nearl's files carry only codenames (2026-09-30).
fn named_characters(rt: &Runtime, tools: &Tools, q: &str) -> Vec<(String, String, String)> {
    let ql = q.to_lowercase();
    let whole = |n: &str| {
        let n = n.to_lowercase();
        n.chars().count() >= 3 && ql.match_indices(&n).any(|(i, _)| {
            ql[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                && ql[i + n.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
        })
    };
    let mut real: HashMap<String, Vec<String>> = HashMap::new();
    for r in tools.real_names.iter().flatten() {
        if let (Some(n), Some(rn)) = (r["name"].as_str(), r["realName"].as_str()) {
            let e = real.entry(trevor::tools::norm(n)).or_default();
            e.push(rn.to_owned());
            let words: Vec<&str> = rn.split_whitespace().collect();
            if words.len() >= 2 {
                e.push(words[0].to_owned());
            }
        }
    }
    let mut out: Vec<(String, String, String)> = Vec::new();
    for c in &rt.store.chunks {
        if !CHARACTER_GROUPS.contains(&c.group_id.as_str()) || out.iter().any(|(s, _, _)| *s == c.story_id) {
            continue;
        }
        let head = c.text.lines().next().unwrap_or_default();
        let own: Vec<String> = if let Some(n) = head.strip_prefix("Operator archive: ") { vec![n.trim().to_owned()] }
            else if let Some(r) = head.strip_prefix("Character profile: ") {
                let (main, aka) = r.split_once(" (also known as ").unwrap_or((r, ""));
                std::iter::once(main.trim()).chain(aka.trim_end_matches(')').split(", ").map(str::trim))
                    .filter(|x| !x.is_empty()).map(str::to_owned).collect()
            } else { Vec::new() };
        let Some(file) = own.first().cloned() else { continue };
        let mut names = own.clone();
        for n in &own {
            let k = trevor::tools::norm(n);
            names.extend(real.get(&k).into_iter().flatten().cloned());
            for ident in tools.identities.iter().filter(|i| i.iter().any(|x| trevor::tools::norm(x) == k)) {
                names.extend(ident.iter().cloned());
            }
        }
        if let Some(n) = names.into_iter().filter(|n| whole(n)).max_by_key(String::len) {
            out.push((c.story_id.clone(), n, file));
        }
    }
    out.sort_by_key(|(_, n, _)| std::cmp::Reverse(n.len()));
    out
}

/// Trevor's chronology, for timeline notes: storyline position per event, dated and derived lines per story.
#[derive(Default)]
struct Chrono {
    groups: std::collections::BTreeMap<String, serde_json::Value>,
    lines: HashMap<String, Vec<String>>,
    /// The dated lines of artifacts/chrono/terra_history.md as (year, line), for a when-question's subject (2026-10-03).
    history: Vec<(f64, String)>,
    /// Identity links of identities.v2.json: (name, other, relation, quote, where story id, stated), for identity notes.
    links: Vec<(String, String, String, String, String, bool)>,
}

fn load_chrono() -> Chrono {
    let mut c = Chrono::default();
    if let Ok(b) = std::fs::read("artifacts/chrono/timeline_v1.json") {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b) {
            for g in v["groups"].as_array().into_iter().flatten() {
                if let Some(id) = g["groupId"].as_str() {
                    c.groups.insert(id.to_owned(), g.clone());
                }
            }
        }
    }
    let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
    for l in read("artifacts/chrono/events.jsonl").lines() {
        let Ok(e) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        if e["is_year"].as_bool() == Some(true) && e["kind"].as_str() == Some("story") {
            let tag = if e["present"].as_bool() == Some(true) { "scene dated" } else { "stated" };
            c.lines.entry(e["source"].as_str().unwrap_or_default().to_owned()).or_default().push(format!(
                "{} ({tag}): {}", e["year"], e["line"].as_str().unwrap_or_default().chars().take(140).collect::<String>()));
        }
    }
    for l in read("artifacts/chrono/terra_history.md").lines() {
        // "- **1086**: text" or "- **ca. 600** (derived: ...): text"
        let Some(rest) = l.strip_prefix("- **") else { continue };
        let Some((year, _)) = rest.split_once("**") else { continue };
        if let Ok(y) = year.trim_start_matches("ca. ").trim().parse::<f64>() {
            c.history.push((y, rest.replacen("**", "", 1).trim().to_owned()));
        }
    }
    c.links = Tools::load(std::path::Path::new(".")).identity_links;
    for l in read("artifacts/chrono/derived.jsonl").lines() {
        let Ok(d) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        let approx = if d["approximate"].as_bool() == Some(true) { "about " } else { "" };
        c.lines.entry(d["storyId"].as_str().unwrap_or_default().to_owned()).or_default().push(format!(
            "{approx}{} (derived, counted from {} via {}): \"{}\"",
            d["year"], d["from"].as_f64().map_or(String::new(), |f| format!("{f:.0}")),
            d["basis"].as_str().unwrap_or_default(), d["quote"].as_str().unwrap_or_default().chars().take(120).collect::<String>()));
    }
    c
}

/// The chronology, loaded once per process (the detector listing calls it per question).
fn load_chrono_cached() -> &'static Chrono {
    static C: std::sync::OnceLock<Chrono> = std::sync::OnceLock::new();
    C.get_or_init(load_chrono)
}

fn has(q: &str, words: &[&str]) -> bool {
    let q = q.to_lowercase();
    words.iter().any(|w| q.contains(w))
}

fn is_time_question(q: &str) -> bool {
    has(q, &["chronolog", "what year", "which year", " year", "when did", "when was", "when is", "how long ago",
             "before or after", "in order", "order the", "timeline", "date"])
}

fn is_canon_question(q: &str) -> bool {
    has(q, &["canon", "official", "retcon", "mistranslat", "translation", "translated", "cn version", "en version", "localiz",
             "developer", "the devs", "hypergryph said", "writer"])
}

fn is_opinion_question(q: &str) -> bool {
    has(q, &["do you think", "what do you think", "who would win", "would win", "strongest", "most powerful", "stronger than",
             "predict", "prediction", "will happen", "theory", "theories", "speculat", "your opinion", "overrated",
             "underrated", "favorite", "favourite"])
}

/// The P4 source kind a question names, if any.
fn source_kind(q: &str) -> Option<&'static str> {
    [("module", &["module"][..]),
     ("voice", &["voice line", "voiceline", "voice lines", "poke", "trust line", "interact line", "dorm line", "says when"][..]),
     ("skin", &["skin", "outfit"][..]),
     ("is", &["integrated strategies", "roguelike", " is1", " is2", " is3", " is4", " is5", " is6", "is#"][..]),
     ("enemy", &["enemy file", "enemy description", "enemy entry", "enemy handbook"][..]),
     ("item", &["item description", "item text", "flavor text", "flavour text"][..])]
        .into_iter().find(|(_, ws)| has(&format!(" {q}"), ws)).map(|(k, _)| k)
}

fn is_source_question(q: &str) -> bool {
    has(q, &["module", "voice line", "voiceline", "voice lines", "skin", "outfit", "integrated strategies", "roguelike",
             " is1", " is2", " is3", " is4", " is5", " is6", "is#", "enemy file", "enemy description", "enemy entry",
             "operator record", "trust line", "interact line", "poke", "dorm line", "furniture"])
}

fn is_synthesis_question(q: &str) -> bool {
    has(q, &["rank", "strongest", "most powerful", "weakest", "compare", "comparison", "stronger", "overview",
             "in general", "summari"])
}

fn timeline_notes(rt: &Runtime, chrono: &Chrono, rows: &[usize], off: usize) -> String {
    // One note per event, listed in in-world order (storyline year, then main-story episode), so the
    // model reads the order instead of having to infer it from bounds: Episodes 6 and 7 share the bound
    // "at most 1098", and without the order the model called their order undeterminable.
    let mut events: Vec<(f64, u64, i64, String, Vec<usize>)> = Vec::new();
    let mut idx: HashMap<String, usize> = HashMap::new();
    for (i, &r) in rows.iter().enumerate() {
        let c = &rt.store.chunks[r];
        let Some(g) = chrono.groups.get(&c.group_id) else { continue };
        let k = *idx.entry(c.group_id.clone()).or_insert_with(|| {
            events.push((g["storylineYear"].as_f64().unwrap_or(9999.0), g["chapter"].as_u64().unwrap_or(0),
                         g["releaseTime"].as_i64().unwrap_or(0), c.group_id.clone(), Vec::new()));
            events.len() - 1
        });
        events[k].4.push(i + 1 + off);
    }
    events.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut out = String::from("\nTIMELINE NOTES (Trevor's chronology, listed in in-world order; years are estimates unless a line states them)\n");
    for (n, (_, chapter, _, gid, ps)) in events.iter().enumerate() {
        let g = &chrono.groups[gid];
        let bound = g["yearBound"].as_str().map_or(String::new(), |b| format!("{b} "));
        let ep = if *chapter > 0 || gid == "main_0" { format!(", main story Episode {chapter}") } else { String::new() };
        let list = ps.iter().map(|p| format!("[{p}]")).collect::<Vec<_>>().join(" ");
        out.push_str(&format!("{}. {}{ep}, passages {list}: storyline {bound}about {:.0} ({})\n", n + 1,
            g["name"].as_str().unwrap_or(gid), g["storylineYear"].as_f64().unwrap_or(0.0), g["basis"].as_str().unwrap_or_default()));
        let mut seen = BTreeSet::new();
        for &p in ps {
            let sid = &rt.store.chunks[rows[p - 1 - off]].story_id;
            if seen.insert(sid.clone()) {
                for l in chrono.lines.get(sid).into_iter().flatten().take(3) {
                    out.push_str(&format!("   - [{p}] {l}\n"));
                }
            }
        }
    }
    out
}

/// The chronology lines of `chronology_for` as the numbered TIME EVIDENCE block of `--time-evidence` (2026-10-03 night 8:
/// ian17 had the 1086 and 1090 to 1094 lines in its prompt and still declined without an estimate).
fn time_evidence_block(lines: &str) -> String {
    let items: Vec<&str> = lines.lines().filter_map(|l| l.strip_prefix("- ")).collect();
    let mut out = String::from("\nTIME EVIDENCE (dated anchors from Trevor's chronology of the whole story, oldest first; use them for the estimate)\n");
    for (i, l) in items.iter().enumerate() {
        out.push_str(&format!("T{}. {l}\n", i + 1));
    }
    out
}

/// After the question under `--time-evidence`: the estimate as the required final line.
const TIME_EVIDENCE_TAIL: &str = "\nREQUIRED FINAL LINE: unless a passage states the exact year, the last line of your answer must be \
\"Estimate: <a year or a range of years>, based on <the TIME EVIDENCE anchors (T1, T2, ...) and passages it rests on>\", \
even when the passages give no date; it is an estimate, so say so.";

/// Question words that never name a when-question's subject.
const QUESTION_WORDS: &[&str] = &["When", "What", "Which", "Who", "Why", "How", "Where", "Did", "Does", "Do", "Is", "Was", "Were", "Are", "In", "The"];

/// The dated lines of Trevor's chronology that name a when-question's subject (2026-10-03, leftover c: "When was Rhodes
/// Island founded?" had timeline notes only for the events its passages came from, none about the founding). Subjects
/// are the question's runs of capitalized words (question words left out); a line counts when it holds one as whole
/// words, case folded. Lines that share more of the question's other words (first 4 letters) come first, then the
/// earliest; at most 8, listed by year. Empty when no line names a subject.
fn chronology_for(q: &str, chrono: &Chrono) -> String {
    let words: Vec<&str> = q.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-')).filter(|w| !w.is_empty()).collect();
    let mut subjects: Vec<String> = Vec::new();
    let mut run: Vec<&str> = Vec::new();
    for w in words.iter().chain(std::iter::once(&"")) {
        if w.chars().next().is_some_and(char::is_uppercase) && !QUESTION_WORDS.contains(w) {
            run.push(w);
        } else if !run.is_empty() {
            subjects.push(run.join(" "));
            run.clear();
        }
    }
    let subjects: Vec<String> = subjects.into_iter().map(|x| trevor::tools::norm(&x)).filter(|x| x.chars().count() >= 3).collect();
    if subjects.is_empty() {
        return String::new();
    }
    let stems: Vec<String> = words.iter().filter(|w| w.len() >= 4 && !w.chars().next().is_some_and(char::is_uppercase))
        .map(|w| w.to_lowercase().chars().take(4).collect()).collect();
    let mut hits: Vec<(usize, f64, &String)> = chrono.history.iter().filter_map(|(y, l)| {
        let nl = trevor::tools::norm(l);
        subjects.iter().any(|x| trevor::tools::contains_words(&nl, x)).then(|| {
            let shared = stems.iter().filter(|st| nl.split(' ').any(|w| w.starts_with(st.as_str()))).count();
            (shared, *y, l)
        })
    }).collect();
    if hits.is_empty() {
        return String::new();
    }
    hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.total_cmp(&b.1)));
    hits.truncate(8);
    hits.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut out = format!("\nCHRONOLOGY LINES THAT NAME {} (Trevor's dated lines from the whole story, not only the passages; a derived \
year says how it was counted)\n", subjects.join(", ").to_uppercase());
    for (_, _, l) in hits {
        out.push_str(&format!("- {l}\n"));
    }
    out
}

/// Identity links whose source line is among the passages (2026-10-03, leftover a): Iris's file states that Mabel was
/// the deceased Operator Bluishsilver, but no answer took the step. Stated links quote their line; inferred links that
/// share a name with a stated one follow, labelled as inferences. At most 4 lines; empty when no passage is a source.
fn identity_notes(rt: &Runtime, rows: &[usize], chrono: &Chrono) -> String {
    let stories: BTreeSet<&str> = rows.iter().map(|&r| rt.store.chunks[r].story_id.as_str()).collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut lines: Vec<String> = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (a, b, rel, quote, at, stated) in &chrono.links {
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        if !*stated || rel == "title" || !stories.contains(at.as_str()) || lines.len() >= 4 || !seen.insert(key) {
            continue;
        }
        names.insert(a.clone());
        names.insert(b.clone());
        lines.push(format!("- {a} and {b} are the same person ({}, stated: \"{quote}\")", rel.replace('_', " ")));
    }
    for (a, b, _, _, _, stated) in &chrono.links {
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        if *stated || lines.len() >= 4 || !(names.contains(a) || names.contains(b)) || !seen.insert(key) {
            continue;
        }
        lines.push(format!("- {a} may be {b} (an inference from the names, not stated)"));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("\nIDENTITY LINKS (from Trevor's identity table, for people the passages name)\n{}\n", lines.join("\n"))
}

/// Whether a name is a proper name by the corpus (`--identity-labels`, 2026-10-03 night 8): its lowercase form occurs
/// as a whole word under 1% as often as the name itself across the loaded chunks ("Operator" 1,168 of 6,260 and "Doctor"
/// 676 of 4,317 are not; "Iris" 3 of 363, "Bluishsilver" 0 of 12 are). A name with no lowercase letters to fold, or
/// never written, is not.
fn proper_name(rt: &Runtime, name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower == name {
        return false;
    }
    let count = |needle: &str| -> usize {
        rt.store.chunks.iter().map(|c| c.text.match_indices(needle).filter(|(i, _)| {
            let before = c.text[..*i].chars().next_back();
            let after = c.text[i + needle.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        }).count()).sum()
    };
    let exact = count(name);
    exact > 0 && count(&lower) * 100 < exact
}

/// Stated identity links (not titles) whose source story is among the passages and whose two names are proper names,
/// as (story id, name, other, relation, quote); one per pair, at most 4 (`--identity-labels`).
fn label_links(rt: &Runtime, rows: &[usize], chrono: &Chrono) -> Vec<(String, String, String, String, String)> {
    let stories: BTreeSet<&str> = rows.iter().map(|&r| rt.store.chunks[r].story_id.as_str()).collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proper: HashMap<String, bool> = HashMap::new();
    let mut out = Vec::new();
    for (a, b, rel, quote, at, stated) in &chrono.links {
        if !*stated || rel == "title" || !stories.contains(at.as_str()) || out.len() >= 4 {
            continue;
        }
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        if seen.contains(&key) {
            continue;
        }
        let ok = [a, b].iter().all(|n| *proper.entry((*n).clone()).or_insert_with(|| proper_name(rt, n)));
        if ok {
            seen.insert(key);
            out.push((at.clone(), a.clone(), b.clone(), rel.replace('_', " "), quote.clone()));
        }
    }
    out
}

/// A passage label with the identity links its story states, resolved: "...; this passage states that Bluishsilver's
/// former name is Mabel (\"Mabel, the owner of the radio, was ...\")".
fn identity_label(label: &str, story: &str, links: &[(String, String, String, String, String)]) -> String {
    let mut out = label.to_owned();
    for (_, a, b, rel, quote) in links.iter().filter(|l| l.0 == story) {
        out.push_str(&format!("; this passage states that {a} and {b} are the same person, {b} being the {rel} of {a} (\"{quote}\")"));
    }
    out
}

/// A question asking who someone is or which person did something, or naming a real name, codename or identity: the
/// questions identity notes are for (on every question they fired on 61 of 336 earlier answers, 41 of them trivia
/// questions about one operator's file; with this test, 5).
fn asks_identity(q: &str) -> bool {
    let n = trevor::tools::norm(q);
    ["who", "whom", "whose", "which", "real name", "codename", "identity", "same person", "known as"].iter()
        .any(|w| trevor::tools::contains_words(&n, w))
}

/// The answer rule for a question asking what something is (2026-10-03, item 6: "What is the khagan quest?" copied a
/// passage and left Kuranta, Nightzmora and Khaganquest unexplained).
const TERM_RULE: &str = " The question asks what something is. Explain it in plain words for a reader who does not know \
Arknights, and the first time you use an in-world term (a people, place, organization, power or custom), define it in a few \
words from the passages. Do not copy a passage's sentences without explaining them.";

/// With the scoped scenes first (2026-10-05 night): the question may name the act in other words than the scene uses.
const SCOPED_RULE_HEAD: &str = " Passages [1] to [";
const SCOPED_RULE_TAIL: &str = "] are the scenes of the event the question names in which the character it names speaks or is \
named, best match first. The question may describe what happens in other words than the scene uses (a threat, a plan or a \
feeling said indirectly, in an aside or in thought, or an idiom such as \"get rid of him\" for a killing): match by meaning, \
not by the words. When a line does what the question describes in other words, it answers the question: do not decline \
because the wording differs. Begin with the answer, quote the line with its citation, say in a few words how its wording \
relates to the question's, and name the people involved as the scene names them.";

/// The label and answer rule of the character-scoped excerpts (`character_scenes`, 2026-10-06).
const CHAR_LABEL: &str = "an excerpt of a scene in which";
const CHAR_RULE_TAIL: &str = "] are excerpts of the scenes in which the character the question names speaks, best match for \
the question first; each label names the event and the story it comes from. The question may describe what the character says \
or does in other words than the scene uses (a threat, a plan or a feeling said indirectly, in an aside or in thought, or an \
idiom such as \"get rid of him\" for a killing): match by meaning, not by the words. When a line does what the question \
describes in other words, it answers the question: do not decline because the wording differs. Begin with the answer (when the \
question asks in which event or story, name it from the passage label), quote the line with its citation, say in a few words \
how its wording relates to the question's, and name the people involved as the scene names them.";

/// Scenes of the event a question names where the character it names speaks or is named (the scoped scenes, default since
/// 2026-10-05 night; `--no-scoped-scenes`). Ian's "what is the name of the npc that mon3tr suggests assassinating in masses
/// travels" was declined although the line is in the corpus ("Mon3tr: (I can take her out. Nobody would notice.)",
/// act42side_08_beg#0000): the whole-corpus search never ranks it, because nothing in the scene says "assassinate". The
/// event comes from `Tools::event_in_question`, the characters from `Tools::event_characters` (appearance index and identity
/// links) kept when the corpus writes the name capitalized (`proper_name`); the candidates are the event's chunks whose
/// speakers or text hold one of their forms, ranked by the question with dense and BM25 fused by reciprocal rank (k 60),
/// and the best are kept within `SCOPED_TOKENS`, at most `SCOPED_K`. Returns (event name, the characters, rows).
fn scoped_scenes(rt: &mut Runtime, tools: &Tools, q: &str) -> Option<(String, Vec<String>, Vec<usize>)> {
    let g = tools.event_in_question(q)?;
    let gid = g["groupId"].as_str()?.to_owned();
    let ename = g["name"].as_str().unwrap_or(&gid).to_owned();
    let chars: Vec<(String, Vec<String>)> = tools.event_characters(q, &gid, &ename).into_iter()
        .filter(|(name, _)| proper_name(rt, name)).collect();
    if chars.is_empty() {
        return None;
    }
    let forms: Vec<String> = chars.iter().flat_map(|(_, f)| f.iter().map(|x| trevor::tools::norm(x)))
        .filter(|x| x.chars().count() >= 3).collect();
    let cands: Vec<usize> = rt.store.chunks.iter().enumerate().filter(|(_, c)| c.group_id == gid).filter(|(_, c)| {
        c.speakers.iter().any(|s| forms.contains(&trevor::tools::norm(s)))
            || { let t = trevor::tools::norm(&c.text); forms.iter().any(|f| trevor::tools::contains_words(&t, f)) }
    }).map(|(i, _)| i).collect();
    if cands.is_empty() {
        return None;
    }
    let ranked = rank_by_question(rt, q, &cands)?;
    if std::env::var("TREVOR_SCOPED_DEBUG").is_ok() {
        eprintln!("scoped {gid}: {} candidates, {} tokens", cands.len(), cands.iter().map(|&r| rt.store.chunks[r].token_count).sum::<u32>());
        for (i, &r) in ranked.iter().enumerate() {
            eprintln!("  {i} {}", rt.store.chunks[r].chunk_id);
        }
    }
    let mut rows = Vec::new();
    let mut used = 0u32;
    for r in ranked {
        let cost = rt.store.chunks[r].token_count;
        if rows.len() >= SCOPED_K || (!rows.is_empty() && used + cost > SCOPED_TOKENS) {
            break;
        }
        used += cost;
        rows.push(r);
    }
    Some((ename, chars.into_iter().map(|(n, _)| n).collect(), rows))
}

/// Words that make a named character the subject of a said or done act ("Mon3tr suggested", "did Necrass say").
const SAID_VERBS: [&str; 39] = ["say", "said", "says", "tell", "told", "tells", "suggest", "suggested", "suggests", "declare",
    "declared", "declares", "joke", "joked", "jokes", "admit", "admitted", "admits", "threaten", "threatened", "threatens",
    "propose", "proposed", "proposes", "offer", "offered", "offers", "mention", "mentioned", "mentions", "claim", "claimed",
    "claims", "promise", "promised", "promises", "confess", "confessed", "confesses"];

/// Whether the question makes the character (any of `forms`, folded) the subject of an act it asks about: a form followed,
/// with at most one word between, by a said verb (`SAID_VERBS`), or, when the question asks in which event, story,
/// chapter, episode or scene, by a said verb or a past-tense word ("During which event Mon3tr suggested ...").
fn asks_scene(q: &str, forms: &[String]) -> bool {
    let n = trevor::tools::norm(&q.replace('\u{2019}', "'"));
    let words: Vec<&str> = n.split_whitespace().collect();
    let l = format!(" {} ", words.join(" "));
    let place = ["event", "story", "chapter", "episode", "scene", "stage"];
    let asks_place = ["which", "what"].iter().any(|w| place.iter().any(|p| l.contains(&format!(" {w} {p} "))));
    forms.iter().any(|f| {
        let fw: Vec<&str> = f.split_whitespace().collect();
        (0..words.len()).filter(|&i| !fw.is_empty() && words[i..].starts_with(&fw)).any(|i| {
            words.iter().skip(i + fw.len()).take(2).any(|w| SAID_VERBS.contains(w) || (asks_place && w.len() > 4 && w.ends_with("ed")))
        })
    })
}

/// Every form of the character a `people_named` key names: the operator, speaker or identity-link name it folds from,
/// and the names its identity links give the same person.
fn person_forms(tools: &Tools, key: &str) -> Vec<String> {
    let mut forms: Vec<String> = tools.operator_names().into_iter().chain(tools.speaker_names())
        .chain(tools.identities.iter().flatten().map(String::as_str))
        .filter(|x| trevor::tools::norm(x) == key).map(str::to_owned).collect();
    for ident in tools.identities.iter().filter(|i| i.iter().any(|x| trevor::tools::norm(x) == key)) {
        forms.extend(ident.iter().cloned());
    }
    forms.sort();
    forms.dedup();
    forms
}

/// One excerpt per scene: (row, text, estimated tokens).
type Excerpts = Vec<(usize, String, u32)>;

/// The lines `idx` of chunk `r` with `CHAR_WINDOW` lines either side, merged, gaps marked "...", and its token estimate
/// (the chunk's token count scaled by the kept share of its text).
fn excerpt(rt: &Runtime, r: usize, idx: &[usize]) -> (String, u32) {
    let c = &rt.store.chunks[r];
    let lines: Vec<&str> = c.text.lines().collect();
    let mut keep = vec![false; lines.len()];
    for &i in idx {
        let hi = (i + CHAR_WINDOW).min(lines.len().saturating_sub(1));
        keep.iter_mut().take(hi + 1).skip(i.saturating_sub(CHAR_WINDOW)).for_each(|k| *k = true);
    }
    let mut parts: Vec<String> = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for (k, l) in lines.iter().enumerate() {
        if keep[k] {
            cur.push(l);
        } else if !cur.is_empty() {
            parts.push(cur.join("\n"));
            cur.clear();
        }
    }
    if !cur.is_empty() {
        parts.push(cur.join("\n"));
    }
    let text = parts.join("\n...\n");
    let tok = u32::try_from(u64::from(c.token_count) * text.len() as u64 / c.text.len().max(1) as u64).unwrap_or(u32::MAX);
    (text, tok)
}

/// Character-scoped scenes (default since 2026-10-06; `--no-character-scenes`): a question that names no event, names one
/// character (`people_named`) and makes them the subject of a said or done act (`asks_scene`) gets excerpts of the
/// scenes where that character speaks, placed first, on top of the retrieved passages' whole budget. Ian's "During which
/// event Mon3tr suggested assassinating a child" (ian12) was declined: nothing in the scene says "assassinate" ("Mon3tr:
/// (I can take her out. Nobody would notice.)", act42side_08_beg#0000), so the whole-corpus search ranks that chunk 6,619th
/// dense, 93rd BM25 and 195th fused of 16,050 in P3b. Among Mon3tr's 172 speaking scenes (421 lines) the chunk ranks 71st
/// by the scoped scenes' dense + BM25 fusion, 158th by chunk dense and 134th by line dense (the bi-encoder is not
/// paraphrase-tolerant here), but 1st when the reranker (a cross-encoder) reads the question with each of her lines and the
/// line before it, in 0.97 s. Each picked line keeps `CHAR_WINDOW` lines either side, at most `CHAR_LINES_PER_SCENE` lines
/// a scene and `CHAR_K` scenes, within `CHAR_TOKENS`. A first version took the scoped scenes' 6,000 tokens and cut the
/// retrieved passages to 3,000: it lost gold g0004 (the retrieved "I won't burn Rhodes Island, or my homework" fell out).
/// Returns (the character as the corpus writes them, excerpts best first).
fn character_scenes(rt: &mut Runtime, ra: &RuntimeArgs, tools: &Tools, q: &str) -> Option<(String, Excerpts)> {
    if tools.event_in_question(q).is_some() || tools.appearance_question(q).is_some() {
        return None;
    }
    let people = people_named(rt, tools, q);
    let [key] = people.as_slice() else { return None };
    let forms: Vec<String> = person_forms(tools, key).iter().map(|x| trevor::tools::norm(x)).filter(|x| x.chars().count() >= 3).collect();
    if forms.is_empty() || tools.names_group_topic(key) || !asks_scene(q, &forms) {
        return None;
    }
    let shown = tools.operator_names().into_iter().chain(tools.speaker_names()).find(|x| trevor::tools::norm(x) == *key)
        .map_or_else(|| key.clone(), str::to_owned);
    let skip = |g: &str| GENERATED_KINDS.contains(&g) || RECORD_KINDS.contains(&g) || g == "voice";
    let cands: Vec<usize> = rt.store.chunks.iter().enumerate()
        .filter(|(_, c)| !skip(&c.group_id) && c.speakers.iter().any(|s| forms.contains(&trevor::tools::norm(s))))
        .map(|(i, _)| i).collect();
    if cands.is_empty() {
        return None;
    }
    // The units: each line the character speaks, with the line before it as context, scored by the reranker (a
    // cross-encoder reads the question and the line together, so "take her out" can match "assassinating").
    let speaks = |line: &str| line.split_once(':').is_some_and(|(s, _)| s.len() < 60 && forms.contains(&trevor::tools::norm(s)));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for &r in &cands {
        let lines: Vec<&str> = rt.store.chunks[r].text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| speaks(l)) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    if rt.reranker.is_none() {
        rt.reranker = Some(trevor::search::rerank::Reranker::load(&ra.rerank_dir, &ra.rerank_onnx, ra.rerank_max_tokens, ra.threads).ok()?);
    }
    let t0 = std::time::Instant::now();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let sc = rt.reranker.as_ref()?.score(q, &refs).ok()?;
    let mut ord: Vec<usize> = (0..units.len()).collect();
    ord.sort_by(|&x, &y| sc[y].total_cmp(&sc[x]).then(units[x].cmp(&units[y])));
    let units: Vec<(usize, usize)> = ord.into_iter().map(|o| units[o]).collect();
    if let Some(target) = std::env::var("TREVOR_SCOPED_DEBUG").ok().as_deref() {
        let mut by_line: Vec<usize> = Vec::new();
        for (r, _) in &units { if !by_line.contains(r) { by_line.push(*r); } }
        let rrf = rank_by_question(rt, q, &cands).unwrap_or_default();
        let pos = |v: &[usize]| v.iter().position(|&r| rt.store.chunks[r].chunk_id == target).map_or(-1, |p| p as i64 + 1);
        eprintln!("charscope {shown}: {} scenes, {} lines, rerank {:.2} s; {target}: chunk rrf {}, reranked lines {}",
                  cands.len(), units.len(), t0.elapsed().as_secs_f64(), pos(&rrf), pos(&by_line));
    }
    // Excerpts: the best lines first, each with its window, merged per scene, within `CHAR_TOKENS`.
    let mut picked: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut used = 0u32;
    for (r, i) in units {
        let at = picked.iter().position(|(x, _)| *x == r);
        let mut idx = at.map_or_else(Vec::new, |p| picked[p].1.clone());
        if idx.len() >= CHAR_LINES_PER_SCENE || (at.is_none() && picked.len() >= CHAR_K) {
            continue;
        }
        let before = if idx.is_empty() { 0 } else { excerpt(rt, r, &idx).1 };
        idx.push(i);
        let grow = excerpt(rt, r, &idx).1.saturating_sub(before);
        if used + grow > CHAR_TOKENS && !picked.is_empty() {
            break;
        }
        used += grow;
        match at { Some(p) => picked[p].1 = idx, None => picked.push((r, idx)) }
    }
    let out = picked.into_iter().map(|(r, idx)| { let (t, n) = excerpt(rt, r, &idx); (r, t, n) }).collect();
    Some((shown, out))
}

/// The line study of Ian's item 1 (`--line-study`, measurement only): every line of the corpus outside the generated kinds
/// with the line before it is a unit; a stride sample of them is embedded (one text per call) and, per question, the
/// evidence line's rank among all units is estimated as the sample units scoring above it times the stride, by line dense
/// and by the reranker, next to the evidence chunk's exact dense, BM25 and fused (RRF k 60) ranks.
fn line_study(rt: &mut Runtime, ra: &RuntimeArgs, batch: &std::path::Path) -> Result<()> {
    let env = |k: &str, d: usize| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
    let (stride, rr_stride, pool) = (env("TREVOR_LINE_STRIDE", 30), env("TREVOR_LINE_RR_STRIDE", 300), env("TREVOR_LINE_POOL", 100));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (r, c) in rt.store.chunks.iter().enumerate().filter(|(_, c)| !GENERATED_KINDS.contains(&c.group_id.as_str())) {
        let lines: Vec<&str> = c.text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    let n = units.len();
    let sample: Vec<usize> = (0..n).step_by(stride).collect();
    let t0 = std::time::Instant::now();
    let mut sv: Vec<Vec<f32>> = Vec::with_capacity(sample.len());
    for &u in &sample {
        sv.push(rt.embed_query(&texts[u])?);
    }
    eprintln!("line study: {n} units, {} sampled, embedded in {:.1} s ({:.1} units/s)", sample.len(),
              t0.elapsed().as_secs_f64(), sample.len() as f64 / t0.elapsed().as_secs_f64());
    if rt.reranker.is_none() {
        rt.reranker = Some(trevor::search::rerank::Reranker::load(&ra.rerank_dir, &ra.rerank_onnx, ra.rerank_max_tokens, ra.threads)?);
    }
    let rr_sample: Vec<usize> = (0..n).step_by(rr_stride).collect();
    let fold = |x: &str| trevor::tools::norm(&x.replace(['\u{2019}', '\u{2018}'], "'").replace(['\u{201c}', '\u{201d}'], "\""));
    println!("qid\tline_dense_est\tline_rerank_est\tchunk_dense\tchunk_bm25\tchunk_fused\tunits={n}");
    for l in std::fs::read_to_string(batch)?.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(l)?;
        let s = |k: &str| v[k].as_str().unwrap_or_default().to_owned();
        let (qid, q, chunk, quote) = (s("qid"), s("question"), s("chunk"), fold(&s("quote")));
        let key: String = quote.chars().take(40).collect();
        let Some(ev) = units.iter().position(|&(r, i)| rt.store.chunks[r].chunk_id == chunk
            && fold(rt.store.chunks[r].text.lines().nth(i).unwrap_or_default()).contains(&key)) else {
            println!("{qid}\tnot-found");
            continue;
        };
        let qv = rt.embed_query(&q)?;
        let dot = |x: &[f32]| x.iter().zip(&qv).map(|(a, b)| a * b).sum::<f32>();
        let es = dot(&rt.embed_query(&texts[ev])?);
        let dense_est = sv.iter().filter(|x| dot(x) > es).count() * stride + 1;
        let mut rr_texts: Vec<&str> = vec![texts[ev].as_str()];
        rr_texts.extend(rr_sample.iter().map(|&u| texts[u].as_str()));
        let sc = rt.reranker.as_ref().context("reranker")?.score(&q, &rr_texts)?;
        let rr_est = sc[1..].iter().filter(|&&x| x > sc[0]).count() * rr_stride + 1;
        let row = units[ev].0;
        let d = rt.dense.as_ref().context("dense")?.search(&qv, rt.store.len())?;
        let b = rt.bm25.as_ref().context("bm25")?.search(&q, rt.store.len(), &rt.store)?;
        let pos = |h: &[(usize, f32)]| h.iter().position(|&(r, _)| r == row).map_or(0, |p| p + 1);
        let mut fused: HashMap<usize, f64> = HashMap::new();
        for h in [&d, &b] {
            for (k, (r, _)) in h.iter().enumerate() {
                *fused.entry(*r).or_default() += 1.0 / (61.0 + k as f64);
            }
        }
        let fr = fused.get(&row).copied().unwrap_or(0.0);
        let fused_rank = fused.values().filter(|&&x| x > fr).count() + 1;
        // The pool variant: the lines of the fused top `pool` chunks, reranked (no line index).
        let mut top: Vec<(usize, f64)> = fused.into_iter().filter(|(r, _)| !GENERATED_KINDS.contains(&rt.store.chunks[*r].group_id.as_str())).collect();
        top.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
        let pool_rows: Vec<usize> = top.into_iter().take(pool).map(|(r, _)| r).collect();
        let pu: Vec<usize> = (0..n).filter(|&u| pool_rows.contains(&units[u].0)).collect();
        let t1 = std::time::Instant::now();
        let pt: Vec<&str> = pu.iter().map(|&u| texts[u].as_str()).collect();
        let ps = rt.reranker.as_ref().context("reranker")?.score(&q, &pt)?;
        let pool_rank = pu.iter().position(|&u| u == ev).map_or_else(|| "out".to_owned(), |p| (ps.iter().filter(|&&x| x > ps[p]).count() + 1).to_string());
        println!("{qid}\t{dense_est}\t{rr_est}\t{}\t{}\t{fused_rank}\t{pool_rank}/{}\t{:.2}", pos(&d), pos(&b), pu.len(), t1.elapsed().as_secs_f64());
    }
    Ok(())
}

/// The line pool (`--line-pool`, opt-in since 2026-10-06): the hybrid (dense + BM25, RRF k 60) top `LINE_POOL_CHUNKS`
/// chunks for `search` outside the generated kinds, each of their lines with the line before it, scored by the reranker
/// with the question. On 84 single-line gold questions the gold line ranks in the reranked top 4 of about 1,430 pool
/// lines for 7 of the 8 that are answered wrong without the gold chunk among their passages (`--line-study`, design/
/// trevor-questions.md section 12), where a whole-corpus line dense index (221,517 lines, a 71-minute build, 170 MB as
/// int8) ranks the gold line in its top 200 for 68 of 87 against 83 of 87 for the chunk fusion, and Chiave's line (ian6)
/// about 19,400th. Answered 2026-10-06: gold correct 4 to 10 of the 14 changed of 30 (+7, -1: g0034), Qwen pairwise 6
/// wins, 0 losses; Ian's 33: fires on 8, pairwise 0 wins, 2 losses (ian17, ian30), so it is opt-in. Fires when the best line is outside `rows` (or always with `all`); then the best lines outside `rows`, at most
/// `CHAR_LINES_PER_SCENE` a chunk, as excerpts with `CHAR_WINDOW` lines either side within `LINE_POOL_TOKENS`.
#[allow(clippy::too_many_arguments)]
fn line_pool(rt: &mut Runtime, ra: &RuntimeArgs, names: &HashMap<String, Names>, q: &str, search: &str, rows: &[usize], all: bool,
             min: Option<f32>) -> Result<Vec<Pre>> {
    if rt.dense.is_none() || rt.bm25.is_none() {
        return Ok(Vec::new());
    }
    if rt.reranker.is_none() {
        rt.reranker = Some(trevor::search::rerank::Reranker::load(&ra.rerank_dir, &ra.rerank_onnx, ra.rerank_max_tokens, ra.threads)?);
    }
    let qv = rt.embed_query(search)?;
    let d = rt.dense.as_ref().context("dense")?.search(&qv, rt.store.len())?;
    let b = rt.bm25.as_ref().context("bm25")?.search(search, rt.store.len(), &rt.store)?;
    let mut fused: HashMap<usize, f64> = HashMap::new();
    for h in [&d, &b] {
        for (k, (r, _)) in h.iter().enumerate() {
            *fused.entry(*r).or_default() += 1.0 / (61.0 + k as f64);
        }
    }
    let mut top: Vec<(usize, f64)> = fused.into_iter().filter(|(r, _)| !GENERATED_KINDS.contains(&rt.store.chunks[*r].group_id.as_str())).collect();
    top.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (r, _) in top.into_iter().take(LINE_POOL_CHUNKS) {
        let lines: Vec<&str> = rt.store.chunks[r].text.lines().collect();
        for (i, line) in lines.iter().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
            units.push((r, i));
            texts.push(if i > 0 { format!("{}\n{line}", lines[i - 1]) } else { (*line).to_owned() });
        }
    }
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let sc = rt.reranker.as_ref().context("reranker")?.score(q, &refs)?;
    let mut ord: Vec<usize> = (0..units.len()).collect();
    ord.sort_by(|&x, &y| sc[y].total_cmp(&sc[x]).then(units[x].cmp(&units[y])));
    let gate = ord.first().is_some_and(|&o| !rows.contains(&units[o].0));
    let best_out = ord.iter().find(|&&o| !rows.contains(&units[o].0)).map_or(f32::NEG_INFINITY, |&o| sc[o]);
    if std::env::var("TREVOR_LINE_POOL_DEBUG").is_ok() {
        eprintln!("linepool\tgate={gate}\tbest_out={best_out:.4}\t{q}");
    }
    if (!gate && !all) || min.is_some_and(|m| best_out < m) {
        return Ok(Vec::new());
    }
    let mut picked: Vec<(usize, Vec<usize>)> = Vec::new();
    let mut used = 0u32;
    for (r, i) in ord.into_iter().map(|o| units[o]).filter(|(r, _)| !rows.contains(r)) {
        let at = picked.iter().position(|(x, _)| *x == r);
        let mut idx = at.map_or_else(Vec::new, |p| picked[p].1.clone());
        if idx.len() >= CHAR_LINES_PER_SCENE {
            continue;
        }
        let before = if idx.is_empty() { 0 } else { excerpt(rt, r, &idx).1 };
        idx.push(i);
        let grow = excerpt(rt, r, &idx).1.saturating_sub(before);
        if used + grow > LINE_POOL_TOKENS && !picked.is_empty() {
            break;
        }
        used += grow;
        match at { Some(p) => picked[p].1 = idx, None => picked.push((r, idx)) }
        if picked.iter().map(|(_, v)| v.len()).sum::<usize>() >= LINE_POOL_LINES {
            break;
        }
    }
    Ok(picked.into_iter().map(|(r, idx)| {
        let c = &rt.store.chunks[r];
        let label = match c.group_id.as_str() {
            "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
            _ => names.get(&c.story_id).map_or_else(|| format!("story: {}", c.story_id), |n| format!("story: {}, {}", n.group, n.story)),
        };
        Pre { id: c.chunk_id.clone(), label: format!("{label}; an excerpt around a line that matches the question"),
              text: excerpt(rt, r, &idx).0, last: true, rule: false, scoped: false }
    }).collect())
}

/// The line pool: chunks whose lines are reranked, lines added, and their token budget on top of the passages'.
const LINE_POOL_CHUNKS: usize = 100;
const LINE_POOL_LINES: usize = 4;
const LINE_POOL_TOKENS: u32 = 1500;

/// Character-scoped excerpts: lines kept either side of a picked line, picked lines per scene, scenes, and tokens (on top
/// of the retrieved passages' budget; at 3,000 the 8 questions it fires on get 2 to 14 scenes).
const CHAR_WINDOW: usize = 2;
const CHAR_LINES_PER_SCENE: usize = 2;
const CHAR_K: usize = 24;
const CHAR_TOKENS: u32 = 3000;

/// `cands` ranked by the question: dense (dot product with the query vector) and BM25 (its whole-corpus ranking, kept to
/// `cands`) fused by reciprocal rank (k 60), ties by row. Used by the scoped scenes and the evidence composition.
fn rank_by_question(rt: &mut Runtime, q: &str, cands: &[usize]) -> Option<Vec<usize>> {
    let mut score: HashMap<usize, f64> = cands.iter().map(|&r| (r, 0.0)).collect();
    if rt.dense.is_some() {
        let qv = rt.embed_query(q).ok()?;
        let d = rt.dense.as_ref()?;
        let mut by: Vec<(usize, f32)> = cands.iter().map(|&r| (r, d.row(r).iter().zip(&qv).map(|(x, y)| x * y).sum())).collect();
        by.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
        for (rank, (r, _)) in by.iter().enumerate() {
            *score.entry(*r).or_default() += 1.0 / (60.0 + rank as f64 + 1.0);
        }
    }
    if let Some(b) = rt.bm25.as_ref() {
        let hits = b.search(q, rt.store.len(), &rt.store).ok()?;
        let inside: Vec<usize> = hits.iter().map(|(r, _)| *r).filter(|r| score.contains_key(r)).collect();
        for (rank, r) in inside.into_iter().enumerate() {
            *score.entry(r).or_default() += 1.0 / (60.0 + rank as f64 + 1.0);
        }
    }
    let mut ranked: Vec<(usize, f64)> = score.into_iter().collect();
    ranked.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    Some(ranked.into_iter().map(|(r, _)| r).collect())
}

/// The evidence composition's answer rule (`--compose-evidence`, opt-in since 2026-10-05 night).
const COMPOSE_RULE: &str = " Passages labelled \"Trevor's identity note\" say which names belong to one person, and the \
passages after each note are scenes that name that person. The question describes someone: compare what it says (what they \
did, where they were, what they are) with what each person's scenes and notes show, joining the scenes (one may say whom a \
character met or looks for, another who that person became). When the scenes together fit the description although no single \
line states it, name that person as a labelled inference (\"Most likely X, also known as Y: ... [n], and ... [m]\"), giving \
every name the notes give them, with citations. If no person fits, say so.";

/// Evidence composition for a who/which question with a description (`--compose-evidence`, opt-in since 2026-10-05 night):
/// when the answer to a question whose answer category the check reads (`Tools::answer_constraint`) still declines,
/// the stated identity links whose source is among its passages (`label_links`, at most 2) become candidates, each as a
/// note ("Bluishsilver and Mabel are the same person ...", with the quote) followed by the scenes naming either name
/// best for the question (`rank_by_question` over P4x; generated kinds left out), within 3,000 tokens per link. ian24
/// ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?") declined with Iris's file,
/// which says Mabel was the deceased Operator Bluishsilver, as passage [1]: the interlude where Iris looks for Mabel Grimm
/// never reached the prompt, and the model does not join two passages on its own. Returns (id, label, text) passages.
fn compose_evidence(rt: &mut Runtime, chrono: &Chrono, names: &HashMap<String, Names>, q: &str, passages: &[String]) -> Vec<(String, String, String)> {
    let rows: Vec<usize> = passages.iter().filter_map(|id| rt.store.row(id)).collect();
    let links = label_links(rt, &rows, chrono);
    let mut out = Vec::new();
    for (at, x, y, rel, quote) in links.iter().take(2) {
        out.push((format!("identity:{x}={y}"), format!("Trevor's identity note (from the game text, {at})"),
                  format!("{x} and {y} are the same person: {y} is the {rel} of {x} (\"{quote}\").")));
        let cands: Vec<usize> = rt.store.chunks.iter().enumerate()
            .filter(|(_, c)| !GENERATED_KINDS.contains(&c.group_id.as_str())
                && (count_word(&c.text, x) > 0 || count_word(&c.text, y) > 0))
            .map(|(i, _)| i).collect();
        let mut used = 0u32;
        for r in rank_by_question(rt, q, &cands).unwrap_or_default() {
            let c = &rt.store.chunks[r];
            // The scoped scenes' budget, shared by the links (2 at most): with 3 scenes ian24's ask lacked Iris's voice
            // line "I'm looking for a girl named Mabel ... twenty years ago" and declined.
            if used + c.token_count > SCOPED_TOKENS / 2 && used > 0 {
                break;
            }
            used += c.token_count;
            let (who, other) = if count_word(&c.text, x) > 0 { (x, y) } else { (y, x) };
            let label = match c.group_id.as_str() {
                "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
                _ => names.get(&c.story_id).map_or_else(|| format!("story: {}", c.story_id), |n| format!("story: {}, {}", n.group, n.story)),
            };
            out.push((c.chunk_id.clone(), format!("{label}; a scene that names {who}, the same person as {other}"), c.text.clone()));
        }
    }
    out
}

/// The scoped scenes take up to this many tokens, two thirds of the default 9,000-token passage budget (the scope is what
/// the question names), at most `SCOPED_K` scenes; the retrieved passages then get `SCOPED_REST`. With 2,400 tokens
/// (4 scenes) Ian's Mon3tr scene ranked 9th of the 20 candidates (8,878 tokens) and was left out.
const SCOPED_K: usize = 16;
const SCOPED_TOKENS: u32 = 6000;
const SCOPED_REST: u32 = 3000;

/// The opinion gate (`--opinion-gate`, opt-in since 2026-10-05 night): the question form's opinion branch gathers
/// candidates from operator files and dossiers only when the question names a character (`people_named`, or a file
/// `named_characters` finds) or a nation, place, race or organization (`Tools::names_group_topic`), or asks who or which;
/// otherwise OPINION_RULE ("the story does not settle it", then the evidence) without candidates. "In lore how strong are
/// my favourite characters?" (real r030) listed Degenbrecher's strength as if she were the asker's favourite (2026-10-05
/// regression run). Measured: responds 10 to 13 and handles 8 to 10 of the 16 real questions it touches by the
/// real-probe judge, but Qwen pairwise 2 wins and 3 losses (r039, r067, r068) over 19; two other answer rules for the
/// gated questions lost more (the keyword rule alone: 2 wins, 5 losses; OPINION_FORM_RULE without candidates: 3 and 3).
fn opinion_needs_candidates(rt: &Runtime, tools: &Tools, q: &str, named: bool) -> bool {
    let n = trevor::tools::norm(q);
    named || tools.names_group_topic(q) || ["who", "which", "whom", "whose"].iter().any(|w| trevor::tools::contains_words(&n, w))
        || !people_named(rt, tools, q).is_empty()
}

/// Whether a word of the question (3 letters or more), capitalized, is a proper name by the corpus (`proper_name`): "How
/// big do you think Fort barron is?" names Barron (13 capitalized, 0 lowercase), a place no topic or character list holds.
fn names_proper_word(rt: &Runtime, q: &str) -> bool {
    q.split(|c: char| !c.is_alphanumeric()).filter(|w| w.chars().count() >= 3).any(|w| {
        let mut cs = w.chars();
        let cap: String = cs.next().map(char::to_uppercase).into_iter().flatten().chain(cs.flat_map(char::to_lowercase)).collect();
        proper_name(rt, &cap)
    })
}

/// The characters a question names: operator names, story speakers and identity-link names it holds as whole words (case
/// and punctuation folded, a possessive allowed, 3 characters or more) that the corpus writes capitalized (`proper_name`),
/// one per person: a name inside a longer matched name, or linked to an earlier one by an identity link, is not counted
/// again ("Margaret" and "Margaret Nearl" are one).
fn people_named(rt: &Runtime, tools: &Tools, q: &str) -> Vec<String> {
    let n = trevor::tools::norm(q);
    let np = trevor::tools::norm(&q.replace('\u{2019}', "'").replace("'s", " "));
    let hit = |x: &str| { let k = trevor::tools::norm(x); k.chars().count() >= 3
        && (trevor::tools::contains_words(&n, &k) || trevor::tools::contains_words(&np, &k)) };
    let mut names: Vec<&str> = tools.operator_names().into_iter().chain(tools.speaker_names()).filter(|x| hit(x)).collect();
    names.extend(tools.identities.iter().flatten().map(String::as_str).filter(|x| hit(x)));
    let mut keys: Vec<String> = names.into_iter().map(trevor::tools::norm).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    keys.dedup();
    let mut out: Vec<String> = Vec::new();
    for k in keys {
        let linked = |a: &str, b: &str| tools.identities.iter().any(|i| i.iter().any(|x| trevor::tools::norm(x) == a)
            && i.iter().any(|x| trevor::tools::norm(x) == b));
        if out.iter().any(|o| trevor::tools::contains_words(o, &k) || linked(o, &k)) {
            continue;
        }
        let shown = tools.operator_names().into_iter().chain(tools.speaker_names())
            .chain(tools.identities.iter().flatten().map(String::as_str)).find(|x| trevor::tools::norm(x) == k).unwrap_or_default().to_owned();
        if proper_name(rt, &shown) || capitalized_in(q, &shown) {
            out.push(k);
        }
    }
    out
}

/// Whether the question itself writes `name` capitalized after its first word ("Is Magallan and Emperor related?"): a
/// name that is also a common word ("Emperor", "Doctor", "Red", "Platinum") is not capitalized in the corpus often enough
/// for `proper_name`, but the asker marks it as a name.
fn capitalized_in(q: &str, name: &str) -> bool {
    let first = name.split_whitespace().next().unwrap_or_default();
    if first.is_empty() || !first.chars().next().is_some_and(char::is_uppercase) {
        return false;
    }
    q.match_indices(first).any(|(i, _)| i > 0
        && !q[..i].chars().next_back().is_some_and(char::is_alphanumeric)
        && !q[i + first.len()..].chars().next().is_some_and(char::is_alphanumeric))
}

/// Whether a relation question asks whom someone admires (one person named, the other asked for).
fn asks_admire(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" fan of ", " idolise ", " idolises ", " idolize ", " idolizes ", " idolised ", " idolized ", " admire ", " admires ",
     " look up to ", " looks up to "].iter().any(|w| ql.contains(w))
}

/// "What is the khagan quest?", "what are the X", "who are the Sarkaz?": a what/who is/are question about a thing of at
/// most 4 words, no possessive (a question about one character's trait is not a definition).
fn asks_term(q: &str) -> bool {
    let l = q.trim().to_lowercase().replace('’', "'");
    let rest = ["what is ", "what are ", "what was ", "what were ", "what's ", "whats ", "who are the ", "who were the "]
        .iter().find_map(|p| l.strip_prefix(p));
    rest.is_some_and(|r| {
        let r = r.trim_end_matches(['?', '.', '!', ' ']);
        !r.contains("'s") && !r.contains('?') && r.split_whitespace().count() <= 4
    })
}

/// Whole-word occurrences of `needle` in `text`, case-sensitive.
fn count_word(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    text.match_indices(needle).filter(|(i, _)| {
        let before = text[..*i].chars().next_back();
        let after = text[i + needle.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    }).count()
}

/// Lowercase letters and digits only: "Khagan quest" and "Khaganquest" are both "khaganquest".
fn compact(x: &str) -> String {
    x.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// The term of a what-is question (`asks_term`), without its article: "What is the khagan quest?" -> "khagan quest".
fn term_of(q: &str) -> Option<String> {
    if !asks_term(q) {
        return None;
    }
    let l = q.trim().to_lowercase().replace('’', "'");
    let rest = ["what is ", "what are ", "what was ", "what were ", "what's ", "whats ", "who are the ", "who were the "]
        .iter().find_map(|p| l.strip_prefix(p))?;
    let rest = rest.trim_end_matches(['?', '.', '!', ' ']);
    let rest = ["the ", "a ", "an "].iter().find_map(|x| rest.strip_prefix(x)).unwrap_or(rest);
    Some(rest.trim().to_owned()).filter(|t| !t.is_empty())
}

/// Chunk kinds Trevor wrote itself, never a term's evidence.
const GENERATED_KINDS: [&str; 3] = ["profile", "summary", "topic"];
/// Chunk kinds that hold records and files, which define a term more often than a scene does.
const RECORD_KINDS: [&str; 7] = ["archive", "gametext", "is", "module", "item", "enemy", "skin"];

/// Term lookup (`--term-lookup`, 2026-10-03 night 9): the corpus's spelling of a term and the chunks naming it most.
/// The term matches one to three consecutive words of a chunk written together or apart ("khagan quest" finds
/// "Khaganquest"); the spelling is the commonest surface form; the chunks are at most `k` of distinct stories, records
/// and files first, then by how often they name it.
fn term_lookup(rt: &Runtime, term: &str, k: usize) -> Option<(String, Vec<usize>)> {
    let key = compact(term);
    if key.chars().count() < 4 {
        return None;
    }
    let mut forms: HashMap<String, usize> = HashMap::new();
    let mut hits: Vec<(bool, usize, usize)> = Vec::new();
    for (row, c) in rt.store.chunks.iter().enumerate() {
        if GENERATED_KINDS.contains(&c.group_id.as_str()) || !compact(&c.text).contains(&key) {
            continue;
        }
        let spans: Vec<(usize, usize)> = {
            let mut v = Vec::new();
            let mut start = None;
            for (i, ch) in c.text.char_indices() {
                match (ch.is_alphanumeric(), start) {
                    (true, None) => start = Some(i),
                    (false, Some(s0)) => { v.push((s0, i)); start = None; }
                    _ => {}
                }
            }
            if let Some(s0) = start { v.push((s0, c.text.len())); }
            v
        };
        let mut n = 0;
        for i in 0..spans.len() {
            let mut acc = String::new();
            for j in i..(i + 3).min(spans.len()) {
                acc.push_str(&compact(&c.text[spans[j].0..spans[j].1]));
                if acc.len() > key.len() || !key.starts_with(acc.as_str()) {
                    break;
                }
                if acc == key {
                    *forms.entry(c.text[spans[i].0..spans[j].1].to_owned()).or_default() += 1;
                    n += 1;
                    break;
                }
            }
        }
        if n > 0 {
            hits.push((!RECORD_KINDS.contains(&c.group_id.as_str()), n, row));
        }
    }
    let spelling = forms.into_iter().max_by(|x, y| x.1.cmp(&y.1).then(y.0.cmp(&x.0)))?.0;
    hits.sort_by(|x, y| x.0.cmp(&y.0).then(y.1.cmp(&x.1)).then(x.2.cmp(&y.2)));
    let mut seen = BTreeSet::new();
    let rows = hits.into_iter().filter(|h| seen.insert(rt.store.chunks[h.2].story_id.clone())).take(k).map(|h| h.2).collect();
    Some((spelling, rows))
}

/// The first `n` sentences of a summary, its section headings skipped (a deep entry starts with "What it is").
fn first_sentences(text: &str, n: usize) -> String {
    let body: Vec<&str> = text.lines().map(str::trim).filter(|l| l.ends_with(['.', '!', '?', ')', '"']) || l.len() > 60).collect();
    let joined = body.join(" ");
    let mut out = String::new();
    let mut count = 0;
    for part in joined.split_inclusive(". ") {
        out.push_str(part);
        count += 1;
        if count >= n {
            break;
        }
    }
    out.trim().to_owned()
}

/// Whole-word, case-sensitive occurrences of `f` in `text`.
fn whole_word_at(text: &str, f: &str) -> Option<usize> {
    text.match_indices(f).find(|(i, _)| {
        let before = text[..*i].chars().next_back();
        let after = text[i + f.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    }).map(|(i, _)| i)
}

/// The glossary of a term lookup: the served topics the term's chunks name (case-sensitive whole words, by first
/// mention), not the term itself, at most `k`, each as the first two sentences of its summary. With `rarity` (default
/// since 2026-10-05), the rarest first: fewest chunks of `rarity` naming one of the topic's forms, first mention breaking
/// ties ("What is the khagan quest?" got Rhodes Island, Terra and Precursors, the generic topics, on night 9).
fn term_glossary(tools: &Tools, texts: &[&str], term: &str, k: usize, rarity: Option<&Runtime>) -> Vec<(String, String)> {
    let key = compact(term);
    let Some(rows) = tools.topics.as_ref() else { return Vec::new() };
    let all = texts.join("\n");
    let mut found: Vec<(usize, usize, String)> = Vec::new();
    for r in rows {
        let name = r["topic"].as_str().unwrap_or_default();
        let forms: Vec<&str> = std::iter::once(name).chain(r["aliases"].as_array().into_iter().flatten().filter_map(|x| x.as_str())).collect();
        if forms.iter().any(|f| compact(f) == key) {
            continue;
        }
        let forms: Vec<&str> = forms.into_iter().filter(|f| f.chars().next().is_some_and(char::is_uppercase) && f.chars().count() >= 4).collect();
        let first = forms.iter().filter_map(|f| whole_word_at(&all, f)).min();
        if let Some(i) = first {
            let df = rarity.map_or(0, |rt| rt.store.chunks.iter().filter(|c| !GENERATED_KINDS.contains(&c.group_id.as_str())
                && forms.iter().any(|f| whole_word_at(&c.text, f).is_some())).count());
            found.push((df, i, name.to_owned()));
        }
    }
    found.sort();
    found.into_iter().filter_map(|(_, _, name)| tools.topic(&name).ok().map(|t| (t.topic, first_sentences(&t.text, 2))))
        .filter(|(_, x)| !x.is_empty()).take(k).collect()
}

/// Identity expansion (`--identity-expand`, 2026-10-03 night 9): for each stated identity link whose source passage is
/// among `rows` (as the identity labels pick them) and names a capitalized, proper word of the question (Iris), the
/// chunks naming a linked name and that word together (the interlude where Iris looks for Mabel Grimm), by the
/// smaller of the two counts, at most `k` in all, inserted right after the link's source passage.
fn identity_expand(rt: &Runtime, chrono: &Chrono, q: &str, mut rows: Vec<usize>, k: usize) -> (Vec<usize>, HashMap<usize, String>) {
    let mut notes: HashMap<usize, String> = HashMap::new();
    let links = label_links(rt, &rows, chrono);
    if links.is_empty() {
        return (rows, notes);
    }
    let mut qwords: Vec<&str> = q.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().next().is_some_and(char::is_uppercase) && w.chars().count() >= 3).collect();
    qwords.dedup();
    let mut proper: HashMap<String, bool> = HashMap::new();
    let mut added = 0;
    for (at, x, y, rel, _) in &links {
        let Some(pos) = rows.iter().rposition(|&r| rt.store.chunks[r].story_id == *at) else { continue };
        let src: Vec<&str> = rows.iter().filter(|&&r| rt.store.chunks[r].story_id == *at).map(|&r| rt.store.chunks[r].text.as_str()).collect();
        let named: Vec<&str> = qwords.iter().copied()
            .filter(|w| w != x && w != y && src.iter().any(|t| count_word(t, w) > 0))
            .filter(|w| *proper.entry((*w).to_owned()).or_insert_with(|| proper_name(rt, w))).collect();
        if named.is_empty() {
            continue;
        }
        let mut cand: Vec<(usize, usize, usize)> = Vec::new();
        for (row, c) in rt.store.chunks.iter().enumerate() {
            if c.story_id == *at || rows.contains(&row) || GENERATED_KINDS.contains(&c.group_id.as_str()) {
                continue;
            }
            let n = count_word(&c.text, x).max(count_word(&c.text, y));
            let m = if n == 0 { 0 } else { named.iter().map(|w| count_word(&c.text, w)).max().unwrap_or(0) };
            if m > 0 {
                cand.push((n.min(m), n + m, row));
            }
        }
        cand.sort_by(|p, r| r.0.cmp(&p.0).then(r.1.cmp(&p.1)).then(p.2.cmp(&r.2)));
        let take: Vec<usize> = cand.iter().take(k - added).map(|c| c.2).collect();
        added += take.len();
        for (i, r) in take.into_iter().enumerate() {
            // The resolved link on the added passage itself: the answer model does not join two passages on its own
            // (ian24 declined with both in its prompt, 2026-10-03 night 9).
            let text = &rt.store.chunks[r].text;
            let (named, other) = if count_word(text, x) > 0 { (x, y) } else { (y, x) };
            notes.insert(r, format!("; this passage names {named}, who is the same person as {other} ({y} being the {rel} of {x}, as \
{at} states)"));
            rows.insert(pos + 1 + i, r);
        }
        if added >= k {
            break;
        }
    }
    (rows, notes)
}

fn is_listing_question(q: &str) -> bool {
    has(q, &["every story", "every event", "all stories", "all the stories", "all events", "all the events",
             "full timeline", "complete timeline", "whole timeline", "entire timeline", "every chapter", "all chapters"])
}

fn is_operator_status_question(q: &str) -> bool {
    has(q, &["operator"]) && has(q, &["dead", "died", "die ", "dies", "death", "killed", "deceased"])
}

// ---------------------------------------------------------------------------------------------------------------
// The keyword router (`--router keywords`, the kill switch): today's parsing of each table route, unchanged, now
// returning a tool and its arguments; the text comes from `trevor::tools`. A parser that matches but whose tool
// fails falls through to the next parser, as each route's `None` did before.

/// Whole-corpus questions answered from Trevor's tables instead of a few passages (Ian, 2026-09-27:
/// "every story in chronological order" listed six items from 16 passages).
fn keyword_route(q: &str, t: &Tools) -> Option<(Route, String)> {
    let parsers: [fn(&str, &Tools) -> Option<Route>; 6] =
        [reading_route, recap_route, listing_route, death_route, real_name_route, attribute_route];
    parsers.iter().filter_map(|p| p(q, t)).find_map(|r| t.run(&r).ok().map(|text| (r, text)))
}

fn listing_route(q: &str, _: &Tools) -> Option<Route> {
    is_listing_question(q).then(|| Route::new("timeline_all", &[]))
}

fn gid(g: &serde_json::Value) -> &str {
    g["groupId"].as_str().unwrap_or_default()
}

/// The place a question names (an affiliation or birthplace in the attribute table).
fn place_filter(q: &str, t: &Tools) -> Option<String> {
    let rows = t.attributes.as_ref()?;
    let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '\''], " "));
    let keys = ["nation", "group", "team", "birthplace"];
    rows.iter().flat_map(|r| keys.iter().filter_map(|k| r[*k].as_str().map(str::to_owned)).collect::<Vec<_>>())
        .filter(|p| p.len() >= 3 && !matches!(p.as_str(), "Unknown" | "Undisclosed") && ql.contains(&format!(" {} ", p.to_lowercase())))
        .max_by_key(String::len)
}

/// Roster questions ("Which operators are Sarkaz?", "How many 6-star Casters are there?", "List all operators from
/// Kazimierz") and one operator's attribute ("What race is Texas?").
fn attribute_route(q: &str, t: &Tools) -> Option<Route> {
    let rows = t.attributes.as_ref()?;
    let s = |r: &serde_json::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '\''], " "));
    let word = |w: &str| !w.is_empty() && ql.contains(&format!(" {} ", w.to_lowercase()));
    // Filters found in the question, as tool arguments.
    let mut args: Vec<(&str, String)> = Vec::new();
    let mut used: Vec<String> = Vec::new();
    // "From X": the game's affiliation (nation, group, team) or the handbook's place of birth.
    if let Some(p) = t.places().iter().filter(|p| word(p)).max_by_key(|p| p.len()).cloned() {
        args.push(("place", p.clone()));
        used.push(p);
    }
    // "Ursus" is a nation and a race; a word already taken as the place is not also a race filter.
    if let Some(race) = t.attr_values("race").iter().filter(|v| (word(v) || word(&format!("{v}s"))) && !used.contains(v)).max_by_key(|v| v.len()).cloned() {
        args.push(("race", race.clone()));
        used.push(race);
    }
    let mut classes: Vec<&str> = Vec::new();
    for c in trevor::tools::CLASSES {
        if word(c) || word(&format!("{c}s")) {
            classes.push(c);
            used.push(c.to_owned());
        }
    }
    if !classes.is_empty() {
        args.push(("class", classes.join(",")));
    }
    // Branches ("Fortress", "Soloblade"); a branch named like its class ("Medic") is left to the class filter.
    let class_words = ["vanguard", "guard", "defender", "sniper", "caster", "medic", "supporter", "specialist"];
    if let Some(b) = t.attr_values("branch").iter().filter(|b| !class_words.contains(&b.to_lowercase().as_str()) && (word(b) || word(&format!("{b}s"))))
        .max_by_key(|b| b.len()).cloned() {
        args.push(("branch", b.clone()));
        used.push(b);
    }
    if let Some(n) = (1..=6).find(|n| ql.contains(&format!(" {n}-star")) || ql.contains(&format!(" {n} star")) || ql.contains(&format!(" {n}*"))) {
        args.push(("rarity", n.to_string()));
        used.push(format!("{n}"));
    }
    if word("female") || word("women") {
        args.push(("gender", "female".into()));
        used.push("female".into());
    } else if word("male") || word("men") {
        args.push(("gender", "male".into()));
        used.push("male".into());
    }
    if word("uninfected") || word("non-infected") || ql.contains(" not infected ") {
        args.push(("infected", "false".into()));
        used.push("infected".into());
    } else if word("infected") {
        args.push(("infected", "true".into()));
        used.push("infected".into());
    }
    // The extra-word guard below keeps a broad trigger safe: "Which female Sankta operators are infected?" has
    // no fixed phrase, and a story question with the same opener keeps more than one content word.
    // Superlatives over height, the one ordered attribute the game data gives every operator (401 of 407).
    let sup = ["tallest", "shortest"].into_iter().find(|w| word(w));
    if let Some(w) = sup {
        used.push(w.to_owned());
    }
    let listing = is_open_listing(q) || ql.starts_with(" which ") || ql.starts_with(" what ") || ql.starts_with(" who ")
        || has(q, &["how many", "list ", "who are the", "name the", "all operators"]);
    if (!args.is_empty() || sup.is_some()) && listing {
        // Only a roster question: after removing the filters and question words no content word may remain,
        // so "Which operators from Rhodes Island went to Londinium?" stays with retrieval.
        let stop = ["which", "what", "who", "how", "many", "are", "is", "there", "the", "a", "an", "all", "list", "of", "in",
                    "from", "operator", "operators", "character", "characters", "every", "known", "with", "and", "or", "star",
                    "one", "person", "people", "branch",
                    "have", "has", "that", "born", "affiliated", "me", "give", "show", "name", "names", "playable", "do",
                    "does", "total", "count", "number", "any"];
        let mut rest = ql.clone();
        for u in &used {
            rest = rest.replace(&format!(" {} ", u.to_lowercase()), " ").replace(&format!(" {}s ", u.to_lowercase()), " ");
        }
        let extra = rest.split_whitespace().filter(|w| !stop.contains(w) && !w.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '*')
            && !w.ends_with("-star")).count();
        // Zero, not one: "Which operators betrayed Rhodes Island?" has one extra word and listed all 84.
        if extra == 0 {
            if let Some(w) = sup {
                args.push(("sort", w.to_owned()));
            }
            return Some(Route { tool: "operator_filter".into(), args: args.into_iter().map(|(k, v)| (k.to_owned(), v)).collect() });
        }
    }
    // One operator's attribute: "What race is Texas?", "When is Exusiai's birthday?".
    let asks = [("race", &["race", "species"][..]), ("birthplace", &["born", "birthplace", "place of birth", "where is", "hometown"][..]),
                ("birthday", &["birthday", "date of birth"][..]), ("height", &["height", "how tall"][..]),
                ("infection", &["infected", "infection", "oripathy"][..]), ("class", &["class", "archetype"][..]),
                ("branch", &["branch", "subclass"][..]), ("rarity", &["rarity", "how many stars", "star"][..]),
                ("gender", &["gender"][..]), ("affiliation", &["affiliation", "faction", "belong", "nation"][..])];
    // Whole words only: "explanation" contains "nation" and had answered with Mostima's affiliation.
    let (field, kws) = asks.iter().find(|(_, ws)| ws.iter().any(|w| ql.contains(&format!(" {w} "))))?;
    let r = rows.iter().filter(|r| {
        let n = s(r, "name").to_lowercase();
        n.len() >= 2 && ql.contains(&format!(" {n} "))
    }).max_by_key(|r| s(r, "name").len())?;
    let mut used: Vec<String> = kws.iter().map(|w| (*w).to_owned()).collect();
    used.push(s(r, "name"));
    used.extend(["born", "birthday", "tall", "many", "stars", "star", "race", "species", "infected", "height", "gender",
                 "class", "branch", "affiliation", "faction", "belong", "to", "rarity"].iter().map(|w| (*w).to_owned()));
    if leftover(q, &used) > 0 {
        return None;
    }
    Some(Route::new("operator_attribute", &[("operator", &s(r, "name")), ("field", field)]))
}

/// Reading-order questions: the whole guide, one event's placement, reading time, first appearance, and reading
/// in chronological order.
fn reading_route(q: &str, t: &Tools) -> Option<Route> {
    let items = t.guide.as_ref()?["items"].as_array()?;
    let s = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_owned();
    // Reading in chronological order: players ask it far more often than they recommend it.
    // Reading words, not "order": "In chronological order, order the death of FrostNova..." asks for a timeline.
    if has(q, &["read", "start", "play"]) && has(q, &["chronolog", "in-world", "in world", "in universe", "in-universe", "timeline order"])
        && !has(q, &["placed", "was before", "were before", "happened before", "take place", "takes place", "took place"]) {
        return Some(Route::new("reading_chronological", &[]));
    }
    // In-world timing ("Is Il Siracusano placed before chapter 2?", "this was before Babel, right?") is chronology,
    // not reading order: those go to retrieval with the timeline notes.
    if has(q, &["placed", "set before", "set after", "take place", "takes place", "took place", "was before", "were before",
                "happened before", "happen before", "timeline", "chronolog", "in-world", "in world"]) {
        return None;
    }
    let group = named_group(q, t).and_then(|g| items.iter().find(|i| i["groupId"].as_str() == Some(gid(g))));
    // Where a character first speaks.
    if has(q, &["first appear", "first appearance", "first show up", "first shows up", "debut", "shows up", "introduced"]) {
        let first = t.first.as_ref()?;
        let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '[', ']'], " "));
        let who = first.as_object()?.keys().filter(|n| n.len() >= 3 && n.chars().next().is_some_and(char::is_uppercase)
            && ql.contains(&format!(" {} ", n.to_lowercase()))).max_by_key(|n| n.len())?.clone();
        // "Did you know the Candle Knight shows up in Catapult's operator record?" is not a first-appearance question.
        let mut used: Vec<String> = vec![who.clone()];
        used.extend(["first", "appear", "appears", "appearance", "show", "shows", "up", "debut", "introduced", "get", "part",
                     "story", "i", "we", "see", "meet", "which", "event", "chapter"].iter().map(|w| (*w).to_owned()));
        if leftover(q, &used) > 0 {
            return None;
        }
        return Some(Route::new("first_appearance", &[("character", &who)]));
    }
    // Reading time of one event or episode.
    if let Some(i) = group {
        // Reading words, not a bare "how long" ("How long has it been in The Masses' Travels?" asks in-world time).
        if has(q, &["how many hours", "hours of reading", "how many words", "reading time", "how long to read", "how long is",
                    "how long does it take", "how long will it take"]) {
            return Some(Route::new("reading_time", &[("event", &s(i, "groupId"))]));
        }
    }
    // What an event builds on and where it sits in the EN release order.
    let per_event = ["before", "when should i read", "when do i read", "when to read", "should i read", "need to read",
                     "read first", "prerequisite", "recommended reading", "stories to play", "what to read", "have to read"];
    if let Some(i) = group {
        let name = s(i, "name");
        let mut used: Vec<String> = name.split_whitespace().map(str::to_owned).collect();
        used.extend(["before", "when", "should", "read", "reading", "need", "needs", "first", "prerequisite", "prerequisites",
                     "recommended", "stories", "story", "play", "playing", "what", "to", "have", "i", "know", "complete",
                     "specific", "chapters", "chapter", "rerun", "event", "regarding", "main", "start", "after", "or",
                     "things", "anything", "about", "it", "you", "do", "that", "which", "episode"].iter().map(|w| (*w).to_owned()));
        if let Some(n) = i["episode"].as_u64() {
            used.push(n.to_string());
        }
        // One stray word is tolerated ("Is it recommended to know about specific chapters before the Babel rerun?");
        // "I was reading some Mansfield Break before Lone Trail comes out. Muelsyse talking about..." has many.
        if has(q, &per_event) && leftover(q, &used) <= 1 {
            return Some(Route::new("reading_event", &[("event", &s(i, "groupId"))]));
        }
    }
    // The whole guide.
    let general = ["reading order", "read order", "story order", "release order", "order to read", "order of events",
                   "where do i start", "where to start", "where should i start", "how do i start", "start with arknights",
                   "best way to start", "catch up", "key to understand", "key events", "major lore", "important events",
                   "essential events", "new player", "recommended order", "which events are key", "which events matter"];
    if group.is_none() && has(q, &general) {
        return Some(Route::new("reading_guide", &[]));
    }
    None
}

/// "What happens in X?", "Explain the ending of X": the event summary.
fn recap_route(q: &str, t: &Tools) -> Option<Route> {
    let words = ["what happens in", "what happened in", "what happens during", "explain the ending", "ending of",
                 "summary of", "summarize", "summarise", "recap", "plot of", "story of", "what is the story",
                 "what was the story", "tl dr", "tldr", "rundown of"];
    if !has(q, &words) {
        return None;
    }
    let g = named_group(q, t)?;
    let id = g["groupId"].as_str()?.to_owned();
    let name = g["name"].as_str().unwrap_or(&id).to_owned();
    let mut used: Vec<String> = name.split_whitespace().map(str::to_owned).collect();
    used.extend(["happens", "happened", "during", "explain", "ending", "summary", "summarize", "summarise", "recap",
                 "plot", "story", "tl", "dr", "tldr", "rundown", "can", "someone", "me", "give", "please", "event", "episode",
                 "chapter", "spoiler", "spoilers", "i", "you", "for", "on", "this", "that"].iter().map(|w| (*w).to_owned()));
    if let Some(n) = g["chapter"].as_u64() {
        used.push(n.to_string());
    }
    if leftover(q, &used) > 0 {
        return None;
    }
    let ending = has(q, &["ending", "end of", "how does it end", "how did it end"]);
    Some(Route::new("recap", &[("event", &id), ("ending", if ending { "true" } else { "false" })]))
}

/// Content words a question has beyond `used` and common question words. A single-subject lookup ("What race is
/// Texas?", "Does W die?") has none; real questions that only mention an operator ("Has there been any explanation
/// why Mostima treats Exusiai the way she does?") have several and must go to retrieval (6 of 7 table answers to
/// 120 real Reddit questions were such misfires, 2026-09-28).
fn leftover(q: &str, used: &[String]) -> usize {
    let stop = ["which", "what", "who", "whom", "how", "when", "where", "is", "are", "was", "were", "the", "a", "an", "of",
                "in", "from", "does", "do", "did", "s", "operator", "operators", "their", "her", "his", "they", "she", "he",
                "it", "its", "to", "and", "or", "there", "any", "ever", "really", "actually", "still", "at", "end", "by"];
    let mut ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '\'', '"'], " "));
    for u in used {
        ql = ql.replace(&format!(" {} ", u.to_lowercase()), " ");
    }
    ql.split_whitespace().filter(|w| !stop.contains(w)).count()
}

/// "List all X of every operator" with no table behind it: retrieval sees a sample, and Ian's real-names
/// question (2026-09-28) got 7 names from 17 passages, presented as the answer.
fn is_open_listing(q: &str) -> bool {
    has(q, &["list all", "list every", "list the", "all the ", "every ", "all known", "complete list", "full list",
             "name all", "all of the", "which operators", "which characters"])
        && has(q, &["operator", "character", "people", "names", "members", "who "])
}

/// Real names from the operator table (`scripts/real_names.py`).
fn real_name_route(q: &str, t: &Tools) -> Option<Route> {
    if !has(q, &["real name", "true name", "birth name", "full name", "actual name"]) {
        return None;
    }
    let rows = t.real_names.as_ref()?;
    let s = |r: &serde_json::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    if is_open_listing(q) || has(q, &["operators' real", "every operator", "all operators"]) {
        // "Real names of every operator from Kazimierz": the place filter of the attribute table narrows the list.
        let place = place_filter(q, t);
        return Some(Route::new("real_names", &place.as_deref().map(|p| vec![("place", p)]).unwrap_or_default()));
    }
    // One operator: the longest codename in the question, as a whole word. No name in the table falls back to
    // retrieval, which may find a story that states it.
    let ql = q.to_lowercase();
    let r = rows.iter().filter(|r| {
        let n = s(r, "name").to_lowercase();
        !n.is_empty() && ql.match_indices(&n).any(|(i, _)| {
            ql[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                && ql[i + n.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
        })
    }).max_by_key(|r| s(r, "name").len())?;
    Some(Route::new("real_name", &[("operator", &s(r, "name"))]))
}

fn is_death_question(q: &str) -> bool {
    has(q, &["dead", "died", "die ", "dies", "die?", "death", "killed", "deceased", "passed away"])
}

/// The main chapter or event a question names: "Episode 7" by chapter, otherwise the longest event
/// name contained in the question.
fn named_group<'a>(q: &str, t: &'a Tools) -> Option<&'a serde_json::Value> {
    let ql = q.to_lowercase();
    if let Some(n) = ql.split("episode").nth(1).and_then(|r| r.trim_start().split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|d| d.parse::<u64>().ok()) {
        if let Some(g) = t.groups.values().find(|g| g["chapter"].as_u64() == Some(n)) {
            return Some(g);
        }
    }
    t.groups.values()
        .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
        .filter(|g| g["name"].as_str().is_some_and(|n| n.len() >= 4 && ql.contains(&n.to_lowercase())))
        .max_by_key(|g| g["name"].as_str().unwrap_or_default().len())
}

/// Death questions answered from the death events (`scripts/deaths.py`).
fn death_route(q: &str, t: &Tools) -> Option<Route> {
    // Ordering and dating questions about deaths ("In chronological order, order the death of FrostNova, death of
    // Patriot...") need the timeline notes that retrieval routing adds, not a list of death events.
    if !is_death_question(q) || is_time_question(q) || has(q, &["before", "after", "chronolog", " order"]) {
        return None;
    }
    t.deaths.as_ref()?;
    let ql = q.to_lowercase();
    let npc_only = has(q, &["non-playable", "nonplayable", "npc", "not playable"]);
    let ops_only = !npc_only && has(q, &["operator", "playable"]);

    if let Some(g) = named_group(q, t) {
        let name = g["name"].as_str().unwrap_or(gid(g));
        // Only "who died in X": "Did Lappland die at the end of Il Siracusano? Or what happened?" and "Was Tin Man's
        // death in Mansfield Break, or an earlier story?" ask something else and go to retrieval.
        let mut used: Vec<String> = name.split_whitespace().map(str::to_owned).collect();
        used.extend(["die", "died", "dies", "dead", "death", "deaths", "killed", "who", "which", "characters", "character",
                     "named", "non-playable", "npcs", "npc", "people", "playable", "episode", "chapter", "list", "all",
                     "everyone", "anyone", "happen", "there", "any"].iter().map(|w| (*w).to_owned()));
        if let Some(n) = g["chapter"].as_u64() {
            used.push(n.to_string());
        }
        if leftover(q, &used) > 0 {
            return None;
        }
        let who = if npc_only { "npc" } else if ops_only { "operators" } else { "all" };
        return Some(Route::new("deaths_in_event", &[("event", gid(g)), ("who", who)]));
    }

    if is_operator_status_question(q) || (ops_only && has(q, &["which", "what", "list", "who"])) {
        return Some(Route::new("dead_operators", &[]));
    }

    // One character: every death event for them in release order, so the answer follows the story.
    let mut names = t.death_names();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let who = names.into_iter().find(|n| {
        let nl = n.to_lowercase();
        ql.match_indices(&nl).any(|(i, _)| {
            let before = ql[..i].chars().last().is_none_or(|c| !c.is_alphanumeric());
            let after = ql[i + nl.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric() && c != '\'');
            before && after
        })
    })?;
    // Only a question about whether or how this one character dies; "Didn't the war end because Theresa died? How
    // can there have been an ascension ceremony..." is about something else and goes to retrieval.
    let dw = ["die", "dies", "died", "dead", "death", "killed", "deceased", "passed", "away", "confirmed", "how", "when"];
    if leftover(q, &std::iter::once(who.clone()).chain(dw.iter().map(|w| (*w).to_owned())).collect::<Vec<_>>()) > 0 {
        return None;
    }
    Some(Route::new("death_of", &[("character", &who)]))
}

/// What a router chose, the table's text when a tool answered, and why a chosen tool did not answer.
struct Routed {
    route: Route,
    text: Option<String>,
    error: Option<ToolError>,
    /// A topic summary to add to retrieval (the topic tool never answers on its own).
    topic: Option<TopicPassage>,
    /// The new dossier `ask --lore v2` adds, by name (shown by `--route-only`).
    dossier: Option<String>,
    /// The characters of the game-data passage `ask --lore v2` adds (shown by `--route-only`).
    game: Option<String>,
    ms: f64,
}

/// Choose a table tool with the router `a.router` and run it. `llm` is needed by the model router only.
async fn route_question(q: &str, a: &Args, tools: &Tools, knn: Option<&mut Knn>, llm: Option<&Llm>) -> Result<Routed> {
    let started = std::time::Instant::now();
    // Detectors that read the question only, before the router (2026-10-03): every router line added for a tool moved
    // 20 to 34 unrelated routes, so these tools are reached without one.
    if a.router == RouterKind::Model && !a.no_route {
        let ms = || started.elapsed().as_secs_f64() * 1000.0;
        if let Some(route) = is_ending_route(q, a, tools) {
            return Ok(Routed { route, text: None, error: None, topic: None, dossier: None, game: None, ms: ms() });
        }
        let pre = (!a.no_appearances).then(|| tools.appearance_question(q)).flatten()
            .map(|c| Route::new("appearances", &[("character", c.as_str())]))
            .or_else(|| (!a.no_cross_ref_trigger).then(|| tools.cross_ref_question(q)).flatten().map(|t| Route::new("cross_ref", &[("table", t)])))
            .or_else(|| (!a.no_design_basis && tools.design_basis(q).is_some()).then(|| Route::new("design_basis", &[("question", q)])));
        if let Some(route) = pre {
            if let Ok(text) = tools.run(&route) {
                return Ok(Routed { route, text: Some(text), error: None, topic: None, dossier: None, game: None, ms: ms() });
            }
        }
        if a.design_deduce && !a.no_design_basis && tools.design_basis(q).is_none() {
            if let (Some(llm), Some((forward, named))) = (llm, tools.design_deduce_question(q)) {
                if let Some(text) = design_deduce(llm, tools, q, forward, &named).await? {
                    let route = Route::new("design_deduce", &[("question", q)]);
                    return Ok(Routed { route, text: Some(text), error: None, topic: None, dossier: None, game: None, ms: ms() });
                }
            }
        }
    }
    let route = match a.router {
        RouterKind::Off => Route::retrieve(),
        RouterKind::Keywords => {
            let (route, text) = keyword_route(q, tools).map_or((Route::retrieve(), None), |(r, t)| (r, Some(t)));
            return Ok(Routed { route, text, error: None, topic: None, dossier: None, game: None, ms: started.elapsed().as_secs_f64() * 1000.0 });
        }
        RouterKind::Model => router::route_model_with(llm.context("the model router needs a llama-server")?, q, &router_prompt(a)).await?,
        RouterKind::Knn => knn.context("the kNN router is not loaded")?.route(q, a.knn_holdout)?.0,
        RouterKind::Hybrid => {
            let knn = knn.context("the kNN router is not loaded")?;
            let (_, v) = knn.route(q, a.knn_holdout)?;
            match knn_sure(&v, q, &knn.dict, tools, a.hybrid_min_sim, a.hybrid_min_share) {
                Some(r) => r,
                None => router::route_model(llm.context("the hybrid router needs a llama-server when kNN is unsure")?, q).await?,
            }
        }
    };
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    let dossier = tools.new_dossier(q).filter(|_| dossier_allowed(q, a, Some(&route))).map(|d| d.topic);
    let game = tools.game_data(q).map(|g| g.topic);
    if route.tool == "topic" {
        if a.no_topics {
            return Ok(Routed { route, text: None, error: None, topic: None, dossier: None, game: None, ms });
        }
        let r = route.args.get("topic").map_or(Err(ToolError::MissingArg("topic")), |t| tools.topic(t));
        let (topic, error) = match r { Ok(p) => (Some(p), None), Err(e) => (None, Some(e)) };
        return Ok(Routed { route, text: None, error, topic, dossier, game, ms });
    }
    // Not an answer: `answer_one` answers from the summary tree (or retrieves, when the tree is missing).
    if route.tool == "overview" {
        let error = tools.overview_passages().err();
        return Ok(Routed { route, text: None, error, topic: None, dossier: None, game: None, ms });
    }
    let (text, error) = if route.is_retrieve() { (None, None) } else {
        match tools.run(&route) { Ok(t) => (Some(t), None), Err(e) => (None, Some(e)) }
    };
    let (dossier, game) = if text.is_none() { (dossier, game) } else { (None, None) };
    Ok(Routed { route, text, error, topic: None, dossier, game, ms })
}

const DEDUCE_REVERSE_SYS: &str = "You deduce what real-world thing an Arknights operator's character design is modelled on, \
using only the game evidence below (the race, operator-file sentences, a model-written description of the art, and vision-model \
readings of the whole art and of its four 2x2 crops) together with your general knowledge of real animals, creatures, plants and \
objects. Name a subject only when specific visible features in the evidence match that subject's distinguishing features (body \
shape, parts, markings); a race name, a setting or a color scheme alone is not enough, and a creature in the background counts \
only when the character's own features or several readings agree with it. Give the most specific species the features justify. \
Give the subject by its common English name only (null when nothing specific matches), then the matching features found in \
the evidence. Output JSON only.";
// The subject comes first: with it after the feature list, Gemma 4 12B wrote broken names at temperature 0 ("bearear",
// "foxrichton fox", "black catsoorted cat"; 9 of the first 15 design answers, 2026-10-05), and clean ones ("bear") first.
const DEDUCE_REVERSE_GRAMMAR: &str = r#"root ::= "{\"subject\": " ( name | "null" ) ", \"kind\": " kind ", \"matching_features\": [" ( str ( ", " str ){0,4} )? "], \"confidence\": " conf "}"
name ::= "\"" [a-zA-Z][a-zA-Z '-]{1,45} "\""
kind ::= "\"animal\"" | "\"mythical creature\"" | "\"plant\"" | "\"object\"" | "\"none\""
conf ::= "\"high\"" | "\"medium\"" | "\"low\""
str ::= "\"" [^"\n\\]{1,90} "\""
"#;
const DEDUCE_LIST_SYS: &str = "A question asks which Arknights operator is based on a real-world thing. List the real-world animals, \
creatures, plants or objects that satisfy the question's constraint, from general knowledge only (for \"an animal with X in its \
name\", animals whose common English name contains X). For each give its common English name, the broader group it belongs to \
(\"bear\", \"jellyfish\", \"bird of prey\") and 3 to 5 distinguishing features \
that would be visible in a character illustration (body shape, parts, colors, markings). At most 6, the best known first. Output \
JSON only.";
const DEDUCE_LIST_GRAMMAR: &str = r#"root ::= "{\"candidates\": [" cand ( ", " cand ){0,5} "]}"
cand ::= "{\"name\": " str ", \"group\": " str ", \"features\": [" str ( ", " str ){1,4} "]}"
str ::= "\"" [^"\n\\]{1,80} "\""
"#;
const DEDUCE_MATCH_SYS: &str = "You match real-world candidates against Arknights operators' game evidence: the race, operator-file \
sentences, a model-written description of the art, and vision-model readings of the whole art and of its four 2x2 crops. An \
operator matches a candidate only when its evidence shows at least two of the candidate's distinguishing features, or names the \
candidate or a near relative and shows one of its features; a color scheme or a setting alone is not a match. List the matching \
operators, best first, at most 3, each with the candidate and the features from its evidence that match; an empty list when no \
operator matches. Output JSON only.";

#[derive(Deserialize)]
struct DeduceReverse { subject: Option<String>, kind: String, matching_features: Vec<String>, confidence: String }
#[derive(Deserialize)]
struct DeduceCand { name: String, group: String, features: Vec<String> }
#[derive(Deserialize)]
struct DeduceList { candidates: Vec<DeduceCand> }
#[derive(Deserialize)]
struct DeduceMatch { operator: String, candidate: String, matching_features: Vec<String>, confidence: String }
#[derive(Deserialize)]
struct DeduceMatches { matches: Vec<DeduceMatch> }

async fn deduce_call<T: serde::de::DeserializeOwned>(llm: &Llm, system: &str, user: &str, grammar: &str, n: u32) -> Result<T> {
    let c = llm.complete(&Request { system, user, grammar: Some(grammar), seed: 1, temperature: 0.0, n_predict: n, stop: &[] }).await?;
    serde_json::from_str(c.content.trim()).with_context(|| format!("design deduction output did not parse: {:?}", c.content))
}

const DEDUCE_LABEL: &str = "Trevor's deduction from the game's art and text (the game never says what a design is based on; \
this matches visible features in the art and the operator files against real-world candidates, so treat it as an inference)";

/// `--design-deduce` (2026-10-05): the two-step design deduction, or `None` when there is nothing to deduce from (no
/// evidence rows; a reverse question naming no operator).
async fn design_deduce(llm: &Llm, tools: &Tools, q: &str, forward: bool, named: &[&serde_json::Value]) -> Result<Option<String>> {
    let st = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_owned();
    if !forward {
        let Some(r) = named.first() else { return Ok(None) };
        let name = st(r, "operator");
        let user = format!("Question: {q}\n\nOperator: {name}\n{}", st(r, "text"));
        let v: DeduceReverse = deduce_call(llm, DEDUCE_REVERSE_SYS, &user, DEDUCE_REVERSE_GRAMMAR, 300).await?;
        let race = st(r, "race");
        let subject = v.subject.filter(|x| !x.trim().is_empty() && !matches!(x.trim().to_lowercase().as_str(), "none" | "null" | "unknown")
            && v.kind != "none" && v.confidence != "low");
        return Ok(Some(match subject {
            Some(sub) => format!("{DEDUCE_LABEL}:\n- {name}: likely modelled on the {sub} ({}, {} confidence). Matching features in the \
game's art and text: {}.\nThe game's own data gives the race as {}.", v.kind, v.confidence, v.matching_features.join("; "),
                if race.is_empty() { "not stated".to_owned() } else { race }),
            None => format!("{DEDUCE_LABEL}:\n- {name}: the game does not show it. Nothing in {name}'s art readings or operator file \
points to a specific real-world animal, creature, plant or object beyond the race ({}).",
                if race.is_empty() { "not stated".to_owned() } else { race }),
        }));
    }
    let list: DeduceList = deduce_call(llm, DEDUCE_LIST_SYS, &format!("Question: {q}"), DEDUCE_LIST_GRAMMAR, 500).await?;
    if list.candidates.is_empty() {
        return Ok(None);
    }
    let mut terms: Vec<(String, f64)> = Vec::new();
    for c in &list.candidates {
        terms.push((c.name.clone(), 2.0));
        terms.push((c.group.clone(), 2.0));
        terms.extend(c.features.iter().map(|f| (f.clone(), 1.0)));
    }
    let short = tools.design_shortlist(&terms, 8);
    if std::env::var("TREVOR_DEDUCE_DEBUG").is_ok() {
        eprintln!("deduce terms {terms:?}\nshortlist {:?}", short.iter().map(|r| r["operator"].as_str().unwrap_or_default()).collect::<Vec<_>>());
    }
    let cands = list.candidates.iter().map(|c| format!("- {} ({}): {}", c.name, c.group, c.features.join("; "))).collect::<Vec<_>>().join("\n");
    let considered = list.candidates.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", ");
    if short.is_empty() {
        return Ok(Some(format!("{DEDUCE_LABEL}:\nNothing in the game shows a match. Real-world candidates for the question (from general \
knowledge): {considered}; no operator's art readings or file share their features.")));
    }
    let names: Vec<String> = short.iter().map(|r| st(r, "operator")).collect();
    let ops = short.iter().enumerate().map(|(i, r)| format!("[{}] {}\n{}", i + 1, st(r, "operator"), st(r, "text"))).collect::<Vec<_>>().join("\n\n");
    let alts = names.iter().map(|n| format!("\"\\\"{}\\\"\"", n.replace(['"', '\\'], ""))).collect::<Vec<_>>().join(" | ");
    let grammar = format!(r#"root ::= "{{\"matches\": [" ( m ( ", " m ){{0,2}} )? "]}}"
m ::= "{{\"operator\": " op ", \"candidate\": " str ", \"matching_features\": [" str ( ", " str ){{0,4}} "], \"confidence\": " conf "}}"
op ::= {alts}
conf ::= "\"high\"" | "\"medium\"" | "\"low\""
str ::= "\"" [^"\n\\]{{1,90}} "\""
"#);
    let user = format!("Question: {q}\n\nCandidates:\n{cands}\n\nOperators:\n{ops}");
    let m: DeduceMatches = deduce_call(llm, DEDUCE_MATCH_SYS, &user, &grammar, 500).await?;
    let near = names.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
    // Grounding check: a match keeps only the features its operator's evidence shares a rare word with, and needs two of
    // them, or one when the evidence names the candidate or its group (the matching step invented "branching dorsal
    // appendages" for Lucilla on the sea-slug dev question).
    let mut m = m;
    for x in &mut m.matches {
        let Some(row) = short.iter().find(|r| r["operator"].as_str() == Some(x.operator.as_str())) else { x.matching_features.clear(); continue };
        x.matching_features.retain(|f| tools.design_grounded(row, f));
        let cand = list.candidates.iter().find(|c| c.name == x.candidate);
        let named = cand.is_some_and(|c| tools.design_grounded(row, &c.name) || tools.design_grounded(row, &c.group)) || tools.design_grounded(row, &x.candidate);
        if x.matching_features.len() < if named { 1 } else { 2 } {
            x.matching_features.clear();
        }
    }
    m.matches.retain(|x| !x.matching_features.is_empty());
    if m.matches.is_empty() {
        return Ok(Some(format!("{DEDUCE_LABEL}:\nNothing in the game shows a match. Real-world candidates for the question (from general \
knowledge): {considered}. The operators whose art readings and files come closest ({near}) do not show their distinguishing \
features, so the game does not show which operator, if any, is based on one.")));
    }
    let lines = m.matches.iter().map(|x| format!("- {}: likely the {} ({} confidence). Matching features in the game's art and text: {}.",
        x.operator, x.candidate, x.confidence, x.matching_features.join("; "))).collect::<Vec<_>>().join("\n");
    Ok(Some(format!("{DEDUCE_LABEL}:\n{lines}\nReal-world candidates considered (from general knowledge): {considered}.")))
}

/// `--route-only`: the keyword router prints as before (the table text, or RETRIEVAL); the model and kNN routers
/// print their choice first, as `ROUTE {"tool": ..., "args": {...}}`.
fn print_route_only(q: &str, a: &Args, r: &Routed) -> Result<()> {
    if a.router != RouterKind::Keywords {
        println!("ROUTE {}", serde_json::to_string(&r.route)?);
    }
    match &r.text {
        Some(t) => println!("{t}"),
        None => {
            let guard = if is_open_listing(q) { " (listing guard)" } else { "" };
            let why = r.error.as_ref().map_or(String::new(), |e| format!(" (tool failed: {e})"));
            let topic = r.topic.as_ref().map_or(String::new(), |t| format!(" (topic passage: {})", t.topic));
            let dossier = r.dossier.as_ref().map_or(String::new(), |d| format!(" (dossier passage: {d})"));
            let game = r.game.as_ref().map_or(String::new(), |g| format!(" (game data: {g})"));
            println!("RETRIEVAL{guard}{why}{topic}{dossier}{game}");
        }
    }
    Ok(())
}

/// The IS-ending route (2026-10-03, item 2): a question naming an Integrated Strategies run and one numbered ending,
/// not asking which ending is canon (the `canon` tool's question).
fn is_ending_route(q: &str, a: &Args, tools: &Tools) -> Option<Route> {
    if a.no_is_ending || has(q, &["canon", "canonical", "official", "true ending"]) {
        return None;
    }
    let (run, _, _, k) = tools.is_ending_question(q)?;
    Some(Route::new("is_ending", &[("run", run.as_str()), ("ending", k?.to_string().as_str())]))
}

/// The passages of an IS-ending route: the run's ending order from game data, then the ending's scene and endbook
/// stories from P4 (`is_rogue_N_ending_K_script`, `is_endbook_rogue_N_K_J`), in order.
fn is_ending_passages(rt: &Runtime, tools: &Tools, q: &str, r: &Route) -> Vec<Pre> {
    let Some((run, run_name, endings, Some(k))) = tools.is_ending_question(q) else { return Vec::new() };
    if r.args.get("run") != Some(&run) {
        return Vec::new();
    }
    let n: usize = run.trim_start_matches("rogue_").parse::<usize>().unwrap_or(0) + 1;
    let name = endings.iter().find(|(e, _)| *e == k).map_or(String::new(), |(_, x)| x.clone());
    let order = endings.iter().map(|(e, x)| format!("Ending {e} '{x}'")).collect::<Vec<_>>().join(", ");
    let mut pre = vec![Pre { id: format!("gamedata:{run}_endings"), label: format!("game data: Integrated Strategies {n}, {run_name}, endings"),
        text: format!("Integrated Strategies {n} ({run_name}) has {} numbered endings, in the game's order: {order}.", endings.len()),
        last: false, rule: false, scoped: false }];
    let script = format!("is_{run}_ending_{k}_script#");
    let book = format!("is_endbook_{run}_{k}_");
    for c in rt.store.chunks.iter().filter(|c| c.chunk_id.starts_with(&script)) {
        pre.push(Pre { id: c.chunk_id.clone(), label: format!("Integrated Strategies {n}, {run_name}: Ending {k} '{name}', ending scene"),
                       text: c.text.clone(), last: false, rule: false, scoped: false });
    }
    for c in rt.store.chunks.iter().filter(|c| c.chunk_id.starts_with(&book)) {
        let j = c.chunk_id[book.len()..].split('#').next().unwrap_or_default();
        pre.push(Pre { id: c.chunk_id.clone(), label: format!("Integrated Strategies {n}, {run_name}: Ending {k} '{name}', endbook story {j}"),
                       text: c.text.clone(), last: false, rule: false, scoped: false });
    }
    pre
}

/// The operator a cited operator-file chunk belongs to ("archive_char_338_iris#0001" -> Iris).
fn file_owner(tools: &Tools, id: &str) -> Option<String> {
    id.starts_with("archive_").then(|| tools.source_owner(id.split('#').next().unwrap_or(id))).flatten()
}

fn table_only(q: &str, text: String) -> Answer {
    Answer { qid: None, question: q.to_owned(), answer: text, cited: Vec::new(), passages: Vec::new(),
             invalid_citations: 0, prompt_tokens: 0, ms: 0.0, retry_query: None, form: None, generated: Default::default() }
}

/// Whether a typed source's own text also retrieves story passages about what it names (not for the keyword
/// router, which keeps retrieval as it was; `--no-source-lore` turns it off).
fn lore_on(a: &Args) -> bool {
    a.router != RouterKind::Keywords && !a.no_source_lore && !a.no_route
}

/// The kinds whose text describes an object (outfit, item, module, enemy) and so names lore it does not explain.
/// Operator files and voice lines keep plain retrieval: all 5 answers the first two lore versions lost were those kinds
/// (2026-09-30: Kal'tsit's voice, Mon3tr's and two gold operator files, a gold voice question).
const LORE_KINDS: [&str; 4] = ["skin", "item", "module", "enemy"];

/// The parts of the model router's prompt that `a` turns on.
fn router_prompt(a: &Args) -> router::RouterPrompt {
    router::RouterPrompt { source: router_source(a), compare: !a.no_reading_compare, overview: a.overview,
                           wide_topics: a.lore == LoreSet::V2, voice_rule: a.voice_rule, art: a.art, cross_ref: a.cross_ref }
}

/// Whether the model router names the source of a retrieval route (default; `--no-router-source` turns it off).
fn router_source(a: &Args) -> bool {
    // Not the hybrid router: its kNN decisions carry no source, so it keeps the keyword list for all of its routes.
    a.router == RouterKind::Model && !a.no_router_source && !a.no_route
}

/// Where retrieval looks: Some(true) for P4 (typed sources), Some(false) for P3b, None to keep the loaded corpus; and
/// the P4 unit kind whose best 3 chunks go first. The model router's source decides when it names one; otherwise
/// the keyword list `source_kind`, as before.
/// Whether a new dossier may join this question's passages: not when it is routed to a typed P4 source, which asks
/// about that text, not about the character (`--dossier-on-typed` allows it).
fn dossier_allowed(q: &str, a: &Args, route: Option<&Route>) -> bool {
    a.dossier_on_typed || !retrieval_plan(q, a, route).1.is_some_and(|k| k != "archive")
}

fn retrieval_plan(q: &str, a: &Args, route: Option<&Route>) -> (Option<bool>, Option<&'static str>) {
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

/// Chunks whose text holds `w` as a whole word, case folded (apostrophes count as letters, so "kal" is not in
/// "Kal'tsit").
fn word_chunks(rt: &Runtime, w: &str) -> usize {
    let w = w.to_lowercase();
    let edge = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '\''));
    rt.store.chunks.iter().filter(|c| {
        let t = c.text.to_lowercase();
        t.match_indices(&w).any(|(i, _)| edge(t[..i].chars().last()) && edge(t[i + w.len()..].chars().next()))
    }).count()
}

/// The question with the full name after each short name it uses ("What did bibeak design for kal" gets
/// "kal (Kal'tsit)"), for retrieval and the answer. A short name counts only when the corpus uses it as a word, but
/// rarely: in 1 to 49 chunks and at most a twentieth as often as the full name (kal: 6 chunks against Kal'tsit's
/// hundreds; "can" 9,817 and "ceo" 10 are left alone, and the typo "adn" (0 chunks) does not become Adnachiel).
fn expand_names(rt: &Runtime, tools: &Tools, q: &str) -> String {
    let mut out = q.to_owned();
    for (w, full) in tools.short_names(q) {
        let lead = full.split_whitespace().next().unwrap_or(&full).to_lowercase();
        let (dw, dn) = (word_chunks(rt, &w), word_chunks(rt, &lead));
        if (1..50).contains(&dw) && dw * 20 <= dn {
            if let Some(i) = out.find(&w) {
                out.insert_str(i + w.len(), &format!(" ({full})"));
            }
        }
    }
    out
}

/// The loaded corpus, and P4 loaded on first need when `--corpus` was left at its default (a batch can mix both).
struct Runtimes {
    main: Runtime,
    p4: Option<Runtime>,
    /// P4 plus the operator art units (`--art`), loaded on the first question routed to the "art" source.
    art: Option<Runtime>,
    /// The P4 directory of the lore served (`--lore`).
    p4_dir: &'static str,
    switch: bool,
    args: RuntimeArgs,
    /// Load the reranker (`--rerank-add`).
    rerank: bool,
}

impl Runtimes {
    fn pick(&mut self, p4: Option<bool>) -> Result<&mut Runtime> {
        if self.switch && p4 == Some(true) && self.args.corpus != std::path::Path::new(self.p4_dir) {
            if self.p4.is_none() {
                let mut a = self.args.clone();
                a.corpus = PathBuf::from(self.p4_dir);
                self.p4 = Some(Runtime::load(&a, true, true, self.rerank)?);
            }
            return Ok(self.p4.as_mut().expect("loaded above"));
        }
        Ok(&mut self.main)
    }

    /// The corpus for the "art" source: `ART_DIR` (P4 plus the art units), P4 when it has not been built.
    fn pick_art(&mut self) -> Result<&mut Runtime> {
        if !self.switch || !std::path::Path::new(ART_DIR).join("chunks.jsonl").exists() {
            return self.pick(Some(true));
        }
        if self.art.is_none() {
            let mut a = self.args.clone();
            a.corpus = PathBuf::from(ART_DIR);
            self.art = Some(Runtime::load(&a, true, true, self.rerank)?);
        }
        Ok(self.art.as_mut().expect("loaded above"))
    }
}

/// P4 plus one unit per operator art image (`scripts/art_captions.py units`, then `build-units --units
/// artifacts/art/p4_art_units.jsonl --out artifacts/p4-art`). Read only for the router's "art" source (`--art`), so P4
/// and every other answer stay as they were.
const ART_DIR: &str = "artifacts/p4-art";

/// A generated passage placed before the retrieved ones: a topic summary or a new dossier (`id` is what answers cite).
struct Pre {
    id: String,
    label: String,
    text: String,
    /// After the retrieved passages instead of before them (a new dossier: placed first it lost g0107, 2026-10-02).
    last: bool,
    /// Adds INFER_RULE to the system prompt (the game-data passage).
    rule: bool,
    /// A scene of the event the question names where its character speaks or is named (`scoped_scenes`): adds
    /// SCOPED_RULE, and the same chunk is not repeated among the retrieved passages.
    scoped: bool,
}

/// The answer rule that comes with a game-data passage (`--lore v2`, 2026-10-02): "Do the Nearls have wings?" was declined
/// though the operator files list every Nearl as Kuranta, and "how old is angelina" was declined though the text implies it.
const INFER_RULE: &str = " A game-data passage lists the operator files' fields (race, birthplace, nation, faction, height) for the \
characters the question names, and the start of each race's topic summary. When the question asks about a body feature, race \
trait, ability or age that no story passage states, do not just decline: infer it from the character's race and what the \
passages say about that race, and label the inference, for example \"X is a Kuranta per the game data; the passages describe \
Kuranta as ...; so ...\". When the passages imply a value without stating it (an age from a timeline, a height), give a \
labelled estimate with its evidence. When the game-data passage lists the races whose summaries mention the asked feature and \
the character's race is not among them, answer that they most likely do not have it, as a labelled inference naming the races \
that do. Say plainly when the passages give no basis at all.";

/// With the game-data passage, for a question asking someone's age (2026-10-03): "how old is angelina" cited "a high
/// school girl" and still gave no number under INFER_RULE alone. The default since 2026-10-03 (r027 and ian13, the only age
/// questions of the five sets: 0 losses, ian13 now gives "about 15 to 18"); `--no-age-estimate` is the kill switch.
const AGE_RULE: &str = " The question asks an age. If no passage states it, end with an explicit estimate as a number or a \
range of years, labelled as an estimate, with the evidence it rests on (for example \"Estimate: about 16 to 18, because she \
is described as a high school student [3]\"); give no estimate only when the passages hold nothing that bears on it.";

/// Whether a question asks someone's age ("how old", "age", "aged", "years old").
fn asks_age(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" how old ", " age ", " aged ", " years old ", " ages "].iter().any(|w| ql.contains(w))
}

/// Why/how-it-began questions (default since 2026-10-03, `--no-origin-chain`): "Why did swire join the LGD?" answered from her
/// grandfather's shadow alone (story_swire_set_1_story_1#0009/#0010), though the same story opens with the kidnapping
/// after which she decided to become a police officer (#0003).
const ORIGIN_RULE: &str = " The question asks why or how something began. Trace the causes back to the earliest one the \
passages show (an event in someone's childhood, an earlier decision, the first proposal of a project), then follow the chain \
forward to what the question asks about, one step per sentence with citations. Do not stop at the latest or most direct \
cause when a passage shows an earlier one.";

/// Whether a question asks why, when or how something began (a why/when/how word and a beginning word).
fn asks_origin(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    let wh = [" why ", " when ", " how "].iter().any(|w| ql.contains(w));
    wh && [" start ", " started ", " starts ", " begin ", " began ", " begun ", " beginning ", " origin ", " origins ",
           " originate ", " originated ", " founded ", " founding ", " join ", " joined ", " joins ", " become ", " became ",
           " decide ", " decided ", " first "].iter().any(|w| ql.contains(w))
}

/// Relationship and admiration questions (`--relation-rule`, 2026-10-03): "What is the relationship between Kristen and
/// Friston?" answered that Friston "refers to Kristen as his daughter" (he calls her "my sunshine"; she reminds him of
/// his own daughter), and Lava's music teacher and the Logos she admires are both her "master". The first wording made
/// ian19 worse (07:45 run: "Trevor Friston is the father of Kristen", from the dream scene's narration "his daughter"), so
/// the rule now also says that a relation shown only in a dream, vision or memory is not a fact.
const RELATION_RULE: &str = " The question asks how people are related or whom someone admires. Keep what they literally are \
to each other (family, teacher, colleague, stranger), as a passage states it, apart from what one calls the other or whom one \
reminds the other of (\"he calls her 'my sunshine'\", \"she reminds him of his daughter\"): report both, and never turn a \
nickname, a term of endearment or a resemblance into a family relation. Two people with the same title or role (two different \
\"masters\" or \"teachers\") are different people unless a passage says they are one; name each with the passage that shows them. \
A dream, vision, illusion or memory shows what a character believes or wishes, not what is so: when a relation appears only \
there, say so and do not state it as a fact.";

/// Whether a question asks how people are related, or whom someone admires.
fn asks_relation(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" relationship ", " relationships ", " relation ", " related ", " relatives ", " fan of ", " idolise ", " idolises ",
     " idolize ", " idolizes ", " idolised ", " idolized ", " admire ", " admires ", " look up to ", " looks up to "]
        .iter().any(|w| ql.contains(w))
}

/// When-questions with timeline notes (`--date-estimate`, 2026-10-03): "When was Rhodes Island founded?" was declined
/// ("no specific date is provided") although the text places it relative to Babel; the age rule's counterpart.
const DATE_RULE: &str = " The question asks when something happened. If no passage or timeline note states the year, end \
with an explicit estimate labelled as such, a year or a range from the timeline notes and the passages' relative times \
(\"X years ago\", \"after the fall of Y\"), with the evidence it rests on (for example \"Estimate: around 1080 to 1085, because \
... [3]\"); give no estimate only when nothing in the passages or notes bears on it.";

/// Whether a question asks when something happened ("when", "what year", "which year", "how long ago").
fn asks_when(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" when ", " what year ", " which year ", " how long ago "].iter().any(|w| ql.contains(w))
}

/// The off-topic guard (`--lore-only`, 2026-10-03): "give me instructions for baking a tray of brownies". The first
/// wording ("say in one sentence that Trevor only answers questions about Arknights lore") was ignored on that question
/// (07:50 run: "I could not find instructions for baking a tray of brownies in the story text."), so the answer is now
/// the fixed sentence itself, and a lore question the passages do not answer is still declined as before.
const LORE_ONLY_RULE: &str = " Trevor answers questions about Arknights lore only. If the question is not about Arknights \
(its story, characters, world or game text), your whole answer is the sentence \"Trevor only answers questions about \
Arknights lore.\" and nothing else, whatever the question asks you to do. A question about Arknights that the passages do \
not answer is declined as usual.";

/// P4 plus the game text the coverage audit found unindexed (`scripts/gametext.py`, 2026-10-03, item 6): story intros,
/// unfetched story scripts, record notes, event archives, mail, operator record intros, world tips and event text, as
/// "gametext" units. Read for P4 routes by default since 2026-10-03; `--no-game-text` reads P4 as before.
const GAME_TEXT_DIR: &str = "artifacts/p4x";

/// The P4 directory `a` reads: `GAME_TEXT_DIR` when it is built (unless `--no-game-text`), else the lore's P4.
fn p4_dir(a: &Args) -> &'static str {
    if !a.no_game_text && std::path::Path::new(GAME_TEXT_DIR).join("chunks.jsonl").exists() { GAME_TEXT_DIR } else { a.lore.dirs().1 }
}

/// The unit kinds the typed fallback tries, in no order: the best-ranked unit of the named operator decides the kind.
const FALLBACK_KINDS: [&str; 3] = ["voice", "module", "art"];

/// The operator a typed unit belongs to, from its first line ("Voice lines: X", "Module story: ... (X's module, ...)",
/// "Operator art: X, E2 ...").
fn typed_owner(heading: &str) -> Option<String> {
    if let Some(r) = heading.strip_prefix("Operator art: ") {
        return r.split(',').next().map(|n| n.trim().to_owned());
    }
    unit_owner(heading)
}

/// The one operator a question names (codenames of the attribute table, whole words after `norm`, the longest match
/// kept when one name holds another); None for none or several.
fn one_operator(q: &str, tools: &Tools) -> Option<String> {
    let nq = trevor::tools::norm(q);
    let mut found: Vec<&str> = tools.operator_names().into_iter()
        .filter(|n| { let x = trevor::tools::norm(n); x.chars().count() >= 3 && trevor::tools::contains_words(&nq, &x) }).collect();
    found.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let mut keep: Vec<&str> = Vec::new();
    for n in found {
        if !keep.iter().any(|k| trevor::tools::contains_words(&trevor::tools::norm(k), &trevor::tools::norm(n))) {
            keep.push(n);
        }
    }
    (keep.len() == 1).then(|| keep[0].to_owned())
}

/// [`one_operator`], or else the operator whose multi-word name starts with a word of the question that starts no other
/// operator's name and is no operator's whole name ("Mutsumi" is Mutsumi Wakaba, 2026-10-03); for the own-file fallback.
fn one_operator_wide(q: &str, tools: &Tools) -> Option<String> {
    if let Some(op) = one_operator(q, tools) {
        return Some(op);
    }
    let nq = trevor::tools::norm(q);
    let names = tools.operator_names();
    let firsts: Vec<(String, &str)> = names.iter().filter_map(|n| {
        let x = trevor::tools::norm(n);
        let first = x.split(' ').next()?.to_owned();
        (x.contains(' ') && first.chars().count() >= 4).then_some((first, *n))
    }).collect();
    let mut hit: Vec<&str> = firsts.iter().filter(|(f, _)| trevor::tools::contains_words(&nq, f)
        && firsts.iter().filter(|(g, _)| g == f).count() == 1 && !names.iter().any(|n| trevor::tools::norm(n) == *f))
        .map(|(_, n)| *n).collect();
    hit.dedup();
    (hit.len() == 1).then(|| hit[0].to_owned())
}

/// The answer rule for operator art units (`--art`, 2026-10-03): their descriptions were written by Gemma from the image.
const ART_RULE: &str = " Passages labelled \"Operator art\" hold a description written by a vision model from the picture, \
not the game's text, and it may be wrong. When you use one, say the claim comes from a model-written description of that art, \
not from game text. Quote lettering in the picture only from the part headed \"Text visible in the art\", and say the reading \
may be wrong.";

/// The table tools and the kNN router when `--router knn` loaded it.
struct Tables {
    tools: Tools,
    knn: Option<Knn>,
    /// The topic summary the router chose for the single question, routed before the index loads.
    topic: Option<TopicPassage>,
    /// The route of the single question, routed before the index loads.
    route: Option<Route>,
}

/// One answer, with the answer category check (default since 2026-10-05, `--no-answer-check`): a who/which question that constrains its
/// answer's category ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?") and whose
/// answer's first named character fails it by the game data (Sakiko Togawa is a playable operator) is asked once more
/// with a passage saying so; when the second answer's first name fails too, the data note is appended to it.
async fn answer_one(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, a: &Args,
                    tables: &mut Tables) -> Result<Answer> {
    let first = answer_one_inner(rts, llm, names, chrono, q, a, tables).await?;
    if a.no_answer_check || a.no_route || first.passages.is_empty() || !asks_identity(q) {
        return Ok(first);
    }
    let Some(c) = tables.tools.answer_constraint(q) else { return Ok(first) };
    let Some((name, reason)) = tables.tools.failing_answer_name(q, &first.answer, &c) else {
        return compose_after(rts, llm, names, chrono, q, a, tables, first).await;
    };
    let wants = match &c {
        AnswerConstraint::NonPlayable => "someone who is not a playable operator".to_owned(),
        AnswerConstraint::Playable => "a playable operator".to_owned(),
        AnswerConstraint::Nation(x) => format!("an operator from {x}"),
    };
    let note = format!("An earlier answer to this question named {name}. By the game data, {reason}, and the question asks for {wants}. Do not give {name} as the answer. Name only someone the passages show fits the question, or say that the passages do not identify one.");
    let again = Args { check_note: Some(note), routed: false, ..a.clone() };
    let mut second = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    if let Some((n2, r2)) = tables.tools.failing_answer_name(q, &second.answer, &c) {
        second.answer = format!("{}\n\nTrevor's data check: {r2}, and the question asks for {wants}, so {n2} does not fit.", second.answer);
    }
    compose_after(rts, llm, names, chrono, q, &again, tables, second).await
}

/// An answer whose first sentence says the passages do not specify or identify what was asked ("The story text does not
/// specify which non-playable Rhodes Island Operator visited Iris's castle of dreams"): a decline for the evidence
/// composition, which `declines` (the retry's and the typed fallback's test) does not count.
fn leaves_open(answer: &str) -> bool {
    let first = answer.trim().split(['.', '!', '?']).next().unwrap_or_default().to_lowercase();
    ["does not specify", "do not specify", "does not identify", "do not identify", "does not name", "do not name"].iter().any(|w| first.contains(w))
}

/// The evidence composition after the answer check (`compose_evidence`): only for a declined answer to a question whose
/// answer category the check reads; the composed answer is kept when it names someone and that name passes the check.
#[allow(clippy::too_many_arguments)]
async fn compose_after(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, a: &Args,
                       tables: &mut Tables, done: Answer) -> Result<Answer> {
    let Some(c) = tables.tools.answer_constraint(q) else { return Ok(done) };
    if !a.compose_evidence || a.no_route || !(declines(&done.answer) || leaves_open(&done.answer)) {
        return Ok(done);
    }
    // P4x: voice lines and records name people too (Iris's "I'm looking for a girl named Mabel ... twenty years ago").
    let compose = compose_evidence(rts.pick(Some(true))?, chrono, names, q, &done.passages);
    if compose.is_empty() {
        return Ok(done);
    }
    let again = Args { compose, routed: false, ..a.clone() };
    let composed = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    if std::env::var("TREVOR_SCOPED_DEBUG").is_ok() {
        eprintln!("compose: {:?}; composed answer: {}", again.compose.iter().map(|x| &x.0).collect::<Vec<_>>(), composed.answer);
    }
    if declines(&composed.answer) || leaves_open(&composed.answer) || tables.tools.failing_answer_name(q, &composed.answer, &c).is_some() {
        return Ok(done);
    }
    Ok(composed)
}

async fn answer_one_inner(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, a: &Args,
                          tables: &mut Tables) -> Result<Answer> {
    // The relation rule is about people (2026-10-05): not for a question naming a nation, place, race or organization topic.
    let people_only;
    let a = if a.relation_rule && !a.relation_rule_all && tables.tools.names_group_topic(q) {
        people_only = Args { relation_rule: false, ..a.clone() };
        &people_only
    } else { a };
    let mut topic = if a.routed { tables.topic.take() } else { None };
    let mut route = if a.routed { tables.route.take() } else { None };
    if !a.no_route && !a.routed {
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_question(q, a, &tables.tools, tables.knn.as_mut(), Some(llm)).await?;
        if let Some(text) = r.text {
            return Ok(table_only(q, text));
        }
        topic = r.topic;
        route = Some(r.route);
    }
    // An IS-ending route: the ending's own passages first, then a short plain retrieval.
    let mut ending_pre: Vec<Pre> = Vec::new();
    if route.as_ref().is_some_and(|r| r.tool == "is_ending") {
        let r = route.take().unwrap_or_else(Route::retrieve);
        ending_pre = is_ending_passages(rts.pick(Some(true))?, &tables.tools, q, &r);
        route = Some(Route::retrieve());
    }
    let short;
    let a = if ending_pre.is_empty() { a } else {
        short = Args { max_context_tokens: 3000, ..a.clone() };
        &short
    };
    // The topic the question names, when the router chose none and retrieval is not a typed source (item 5).
    if topic.is_none() && a.named_topic && !a.no_topics && !a.no_route && a.router != RouterKind::Keywords
        && route.as_ref().is_some_and(|r| r.is_retrieve() && matches!(r.args.get("source").map(String::as_str), None | Some("story" | "any")))
        // Since 2026-10-05 only a question about the topic as a whole: the night 7 package lost held-out h004 ("What's
        // the difference between the Silverlance Pegasi and the rest of Campaign/gendarmerie knights?"), whose
        // Gendarmerie summary and event summaries displaced the story passage on the Pegasi's speed.
        && (a.named_topic_all || tables.tools.broad_topic_question(q)) {
        topic = tables.tools.named_topic(q);
    }
    if route.as_ref().is_some_and(|r| r.tool == "overview") {
        if let Ok(passages) = tables.tools.overview_passages() {
            return answer_overview(llm, q, a, &passages).await;
        }
        route = Some(Route::retrieve());
    }
    // Passages before the retrieved ones: the topic summary the router chose, then (`--lore v2`) the new dossier of a
    // character the question names. Under v1 the prompt is the one before, byte for byte.
    let mut pre: Vec<Pre> = ending_pre;
    let topic_label = |t: &TopicPassage| if tables.tools.deep_entry(&t.topic).is_some() {
        format!("Trevor's topic entry (generated from the story passages, operator files and records that name it; prefer the story passages): {}", t.topic)
    } else {
        format!("Trevor's topic summary (generated; prefer the story passages): {}", t.topic)
    };
    pre.extend(topic.iter().map(|t| Pre { id: format!("topic:{}", t.topic), label: topic_label(t), text: t.text.clone(), last: false,
        rule: false, scoped: false }));
    // A topic summary comes with the event summaries of the 3 groups that name the topic most (items 3 and 4).
    if a.topic_events {
        if let Some(t) = &topic {
            for (gid, name, text) in tables.tools.topic_events(&t.topic, 3) {
                pre.push(Pre { id: format!("event:{gid}"), label: format!("Trevor's event summary (generated; prefer the story passages): {name}"),
                               text, last: false, rule: false, scoped: false });
            }
        }
    }
    // A place's nation topic after it (item 5): "factions in dossoles" are Bolívar's.
    if a.place_nation {
        if let Some(n) = topic.as_ref().and_then(|t| tables.tools.place_nation(&t.topic, !a.place_nation_summary)) {
            pre.push(Pre { id: format!("topic:{}", n.topic), label: format!("Trevor's topic summary (generated; prefer the story passages): {}, \
the nation of {}", n.topic, topic.as_ref().map_or("", |t| t.topic.as_str())), text: n.text, last: false, rule: false, scoped: false });
        }
    }
    if let Some(d) = tables.tools.new_dossier(q).filter(|_| dossier_allowed(q, a, route.as_ref())) {
        pre.push(Pre { id: format!("dossier:{}", d.topic),
                       label: format!("Trevor's character dossier (generated; prefer the story passages): {}", d.topic), text: d.text,
                       last: true, rule: false, scoped: false });
    }
    if !a.compose.is_empty() {
        pre.clear();
    }
    for (i, (id, label, text)) in a.compose.iter().enumerate() {
        pre.insert(i, Pre { id: id.clone(), label: label.clone(), text: text.clone(), last: false, rule: false, scoped: false });
    }
    if let Some(note) = &a.check_note {
        pre.push(Pre { id: "check".into(), label: "Trevor's check of an earlier answer against the game data".into(), text: note.clone(),
                       last: true, rule: false, scoped: false });
    }
    if let Some(g) = tables.tools.game_data(q) {
        pre.push(Pre { id: format!("gamedata:{}", g.topic), label: format!("game data: {}", g.topic), text: g.text, last: true, rule: true, scoped: false });
    }
    // Term lookup (night 9): the term's spelling in P4x, its two best record chunks and a glossary, before the passages.
    let mut term_search: Option<String> = None;
    if !a.no_term_lookup && !a.no_route {
        if let Some(term) = term_of(q) {
            let rt4 = rts.pick(Some(true))?;
            if let Some((spelling, rows)) = term_lookup(rt4, &term, 2) {
                let texts: Vec<&str> = rows.iter().map(|&r| rt4.store.chunks[r].text.as_str()).collect();
                let glossary = term_glossary(&tables.tools, &texts, &term, 3, (!a.no_glossary_rarity).then_some(&*rt4));
                let mut at = 0;
                for &r in &rows {
                    let c = &rt4.store.chunks[r];
                    let label = match c.group_id.as_str() {
                        "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
                        g if RECORD_KINDS.contains(&g) || g == "voice" => c.text.lines().next().unwrap_or_default().to_owned(),
                        _ => names.get(&c.story_id).map_or_else(|| format!("story: {}", c.story_id), |n| format!("story: {}, {}", n.group, n.story)),
                    };
                    pre.insert(at, Pre { id: c.chunk_id.clone(), label: format!("{label}; it names {spelling}"), text: c.text.clone(), last: false, rule: false, scoped: false });
                    at += 1;
                }
                if !glossary.is_empty() {
                    let text = glossary.iter().map(|(n, t)| format!("{n}: {t}")).collect::<Vec<_>>().join("\n");
                    pre.insert(at, Pre { id: "glossary".into(), label: "Trevor's glossary of the terms these passages use (generated from its topic \
summaries; prefer the passages)".into(), text, last: false, rule: false, scoped: false });
                }
                if !q.to_lowercase().contains(&spelling.to_lowercase()) {
                    term_search = Some(format!("{q} {spelling}"));
                }
            }
        }
    }
    let (p4, kind) = retrieval_plan(q, a, route.as_ref());
    let rt = if kind == Some("art") { rts.pick_art()? } else { rts.pick(p4)? };
    let two_people;
    let a = if a.relation_rule && !a.relation_rule_any && asks_relation(q) && !asks_admire(q) && people_named(rt, &tables.tools, q).len() < 2 {
        two_people = Args { relation_rule: false, ..a.clone() };
        &two_people
    } else { a };
    let expanded = if a.router == RouterKind::Keywords || a.no_name_expansion || a.no_route { q.to_owned() }
        else { expand_names(rt, &tables.tools, q) };
    let q = expanded.as_str();
    let story_q = if a.story_query && !a.no_route && a.router != RouterKind::Keywords { story_words(llm, q).await? } else { None };
    let mut form = if form_on(a) { Some(question_form(llm, q, a.form_v1, a.model_flags).await?) } else { None };
    if let Some(sq) = story_q {
        form.get_or_insert_with(|| Form { form: "fact".into(), ..Form::default() }).story = Some(sq);
    }
    if let Some(f) = form.as_mut().filter(|f| f.form == "opinion") {
        f.named = named_characters(rt, &tables.tools, q);
    }
    if let Some(f) = form.as_mut().filter(|f| f.form == "opinion" && (a.opinion_gate || (!a.no_opinion_gate && is_opinion_question(q)))) {
        if !opinion_needs_candidates(rt, &tables.tools, q, !f.named.is_empty()) && (a.opinion_gate || !names_proper_word(rt, q)) {
            f.gated = true;
            f.evidence = None;
        }
    }
    // The scoped scenes go first (before a term lookup's records), and are not repeated among the retrieved passages.
    let scoped = if !a.no_scoped_scenes && !a.no_route && kind.is_none() { scoped_scenes(rt, &tables.tools, q) } else { None };
    if let Some((ename, who, rows)) = &scoped {
        for (i, &r) in rows.iter().enumerate() {
            let c = &rt.store.chunks[r];
            let label = names.get(&c.story_id).map_or_else(|| format!("story: {}", c.story_id), |n| format!("story: {}, {}", n.group, n.story));
            pre.insert(i, Pre { id: c.chunk_id.clone(), label: format!("{label}; a scene of {ename} in which {} speaks or is named",
                who.join(" or ")), text: c.text.clone(), last: false, rule: false, scoped: true });
        }
    }
    // Character-scoped scenes (2026-10-06): no event named, one character named as the subject of a said or done act.
    // They are added before the retrieved passages, which keep their whole budget (with the scoped scenes' 3,000-token
    // rest, gold g0004 lost the retrieved line "I won't burn Rhodes Island, or my homework").
    if scoped.is_none() && !a.no_character_scenes && !a.no_route && kind.is_none() {
        if let Some((who, rows)) = character_scenes(rt, &a.runtime, &tables.tools, q) {
            for (i, (r, text, _)) in rows.iter().enumerate() {
                let c = &rt.store.chunks[*r];
                let label = names.get(&c.story_id).map_or_else(|| format!("story: {}", c.story_id), |n| format!("story: {}, {}", n.group, n.story));
                pre.insert(i, Pre { id: c.chunk_id.clone(), label: format!("{label}; {CHAR_LABEL} {who} speaks"), text: text.clone(),
                                    last: false, rule: false, scoped: true });
            }
        }
    }
    // With scoped scenes the retrieved passages get the rest of the budget, 3,000 tokens, as with IS-ending passages.
    let scoped_budget;
    let a = if scoped.is_some() {
        scoped_budget = Args { max_context_tokens: a.max_context_tokens.min(SCOPED_REST), ..a.clone() };
        &scoped_budget
    } else { a };
    let form = form.as_ref();
    let offtopic = a.lore_only && !tables.tools.names_anything(q);
    let first = answer_from(rt, llm, names, chrono, q, term_search.as_deref().unwrap_or(q), a, &pre, kind, form, offtopic).await?;
    // The typed fallback (2026-10-03, item 10; `--no-typed-fallback`): a declined answer to a question that names exactly one operator and was
    // not routed to a typed source tries that operator's own voice lines, module stories and (when artifacts/p4-art is
    // built) art units, without touching the router prompt: every added router line moved about 20 unrelated routes.
    // Since 2026-10-03 (leftover b) also an answer that cites only the operator's own file: "who helped Mutsumi care
    // for the plants?" cited her file, which says only that others admire her; the robots are in her module story.
    let own_file_only = |op: &str| !a.no_own_file_fallback && !first.cited.is_empty() && first.cited.iter()
        .all(|id| file_owner(&tables.tools, id).is_some_and(|o| trevor::tools::norm(&o) == trevor::tools::norm(op)));
    let retry_op = one_operator(q, &tables.tools).filter(|_| declines(&first.answer))
        .or_else(|| one_operator_wide(q, &tables.tools).filter(|op| own_file_only(op)));
    if !a.no_typed_fallback && !a.no_route && kind.is_none() && retry_op.is_some() {
        if let Some(op) = retry_op {
            let rt2 = rts.pick_art()?;
            let wide = rt2.retrieve(q, &RetrievalConfig { mode: Mode::Hybrid, k: 60, ..RetrievalConfig::default() })?;
            let own = wide.iter().map(|h| &rt2.store.chunks[h.row]).find(|c| FALLBACK_KINDS.contains(&c.group_id.as_str())
                && typed_owner(c.text.lines().next().unwrap_or_default()).is_some_and(|o| trevor::tools::norm(&o) == trevor::tools::norm(&op)))
                .map(|c| c.group_id.clone());
            if let Some(k) = own {
                let k: &'static str = FALLBACK_KINDS.iter().find(|x| **x == k).copied().unwrap_or("voice");
                let second = answer_from(rt2, llm, names, chrono, q, q, a, &pre, Some(k), form, offtopic).await?;
                // Kept only when it cites a unit of the operator itself: "what operator is based off an animal with
                // phantom as part of its name" (ian11) named Phantom, got Melantha's voice lines first and answered
                // "Melantha" where the default declines (2026-10-03 07:50 run).
                let cites_own = second.cited.iter().any(|id| rt2.store.chunks.iter().find(|c| c.chunk_id == *id)
                    .is_some_and(|c| FALLBACK_KINDS.contains(&c.group_id.as_str())
                        && typed_owner(c.text.lines().next().unwrap_or_default()).is_some_and(|o| trevor::tools::norm(&o) == trevor::tools::norm(&op))));
                if !declines(&second.answer) && cites_own {
                    return Ok(second);
                }
            }
        }
    }
    let rt = if kind == Some("art") { rts.pick_art()? } else { rts.pick(p4)? };
    if !a.retry || !declines(&first.answer) {
        return Ok(first);
    }
    // Retry once with the question rewritten into the story's own terms. Real questions miss passages that hold the
    // answer when the wording differs ("When did Ceobe get high on drugs?" never retrieves the mushroom scene of The
    // Great Chief Returns); the research doc ranks a decline-only retry among the cheapest fixes.
    let rw = llm
        .complete(&Request {
            system: REWRITE,
            user: q,
            grammar: None,
            seed: 1,
            temperature: 0.0,
            n_predict: 60,
            stop: &["\n"],
        })
        .await?;
    let query = rw.content.trim().trim_matches('"').to_owned();
    if query.is_empty() {
        return Ok(first);
    }
    // The rewrite alone found nothing on the first real run (0 of 36 declines answered: Gemma cannot put the question in
    // the story's terms without knowing the story), so the retry also widens retrieval: twice the passages, one more
    // neighbour each side, a 14,000-token budget (the research survey's passage-budget evidence on 9 to 12B readers).
    let mut wide = a.clone();
    wide.k = a.k * 2;
    wide.neighbors = a.neighbors + 1;
    wide.max_context_tokens = a.max_context_tokens.max(14_000);
    let mut second = answer_from(rt, llm, names, chrono, q, &format!("{q} {query}"), &wide, &pre, kind, form, offtopic).await?;
    if declines(&second.answer) {
        return Ok(first);
    }
    second.retry_query = Some(query);
    Ok(second)
}

/// Whether the topic tool serves deep entries for this question (default since 2026-10-05, lore v2 only; `--no-deep-topics`):
/// only for a question about the topic as a whole (`Tools::broad_topic_question`); `--deep-topics-all` is night 9's reach.
fn deep_for(a: &Args, tools: &Tools, q: &str) -> bool {
    !a.no_deep_topics && a.lore != LoreSet::V1 && (a.deep_topics_all || tools.broad_topic_question(q))
}

/// A decline opens the answer ("I could not find it in the story text"), the rule `answer-eval.py` uses.
fn declines(answer: &str) -> bool {
    let first = answer.trim().split(['.', '!', '?']).next().unwrap_or_default().to_lowercase();
    ["could not find", "couldn't find", "cannot find", "can't find", "no information", "not mentioned", "not stated",
     "does not say", "does not state", "does not mention", "do not contain", "does not contain"].iter().any(|w| first.contains(w))
}

/// One answer from the passages retrieved for `search`, answering `q`. A topic summary, when the router chose one,
/// is passage [1] and the retrieved passages follow.
#[allow(clippy::too_many_arguments)]
async fn answer_from(rt: &mut Runtime, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, search: &str,
                     a: &Args, pre: &[Pre], kind: Option<&str>, form: Option<&Form>, offtopic: bool) -> Result<Answer> {
    let opinion_form = form.is_some_and(|f| f.form == "opinion");
    let (mut rows, lore) = passages(rt, search, a, kind, form)?;
    // The evidence composition's ask reads the composed candidates alone: with the retrieved passages after them, ian24's
    // composed ask named the collaboration's Sakiko Togawa again (2026-10-05 night).
    if !a.compose.is_empty() {
        rows.clear();
    }
    let scoped: Vec<&str> = pre.iter().filter(|p| p.scoped).map(|p| p.id.as_str()).collect();
    if !scoped.is_empty() {
        rows.retain(|&r| !scoped.contains(&rt.store.chunks[r].chunk_id.as_str()));
    }
    let (rows, expand_notes) = if a.identity_expand && !a.no_route && asks_identity(q) { identity_expand(rt, chrono, q, rows, 2) } else { (rows, HashMap::new()) };
    let pool = if a.line_pool && !a.no_route && kind.is_none() && scoped.is_empty() { line_pool(rt, &a.runtime, names, q, search, &rows, a.line_pool_all, a.line_pool_min)? } else { Vec::new() };
    if a.line_pool && std::env::var("TREVOR_LINE_POOL_DETECT").is_ok() {
        return Ok(Answer { qid: None, question: q.to_owned(), answer: format!("line-pool detect: {}", pool.len()), cited: Vec::new(),
                           passages: Vec::new(), invalid_citations: 0, prompt_tokens: 0, ms: 0.0, retry_query: None, form: None,
                           generated: std::collections::BTreeMap::new() });
    }
    let (first, post): (Vec<&Pre>, Vec<&Pre>) = pre.iter().partition(|p| !p.last);
    let post: Vec<&Pre> = post.into_iter().chain(pool.iter()).collect();
    let off = first.len();
    let flags = form.and_then(|f| f.flags).unwrap_or_else(|| Flags::keywords(q));
    let time_q = !a.no_route && flags.asks_time;
    let synth_q = !a.no_route && flags.asks_synthesis;
    let mut user = String::from("PASSAGES\n");
    for (i, p) in first.iter().enumerate() {
        user.push_str(&format!("\n[{}] ({})\n{}\n", i + 1, p.label, p.text));
    }
    let label_links = if a.identity_labels && !a.no_route && asks_identity(q) { label_links(rt, &rows, chrono) } else { Vec::new() };
    for (i, &r) in rows.iter().enumerate() {
        let c = &rt.store.chunks[r];
        // The source kind leads the label, so an answer can say where a claim comes from (SOURCE_RULE).
        let label = match c.group_id.as_str() {
            "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
            "profile" => format!("Trevor's character dossier (a summary written by Trevor): {}", c.story_id),
            "summary" => format!("Trevor's event summary (written by Trevor): {}", c.story_id),
            "module" | "voice" | "is" | "enemy" | "skin" | "item" | "art" | "gametext" => c.text.lines().next().unwrap_or_default().to_owned(),
            _ => names.get(&c.story_id).map_or_else(|| format!("story: {}", c.story_id), |n| format!("story: {}, {}", n.group, n.story)),
        };
        // A file found through another name of its character says so: with "Margaret vs Degenbrecher" and Nearl the
        // Radiant Knight's file among the passages, the answer still said it found nothing about "a character named
        // Margaret" (2026-09-30).
        let label = match form.and_then(|f| f.named.iter().find(|(sid, asked, file)| *sid == c.story_id
            && trevor::tools::norm(asked) != trevor::tools::norm(file))) {
            Some((_, asked, file)) => format!("{label}; this is the file of {file}, whom the question calls {asked}"),
            None => label,
        };
        let label = if label_links.is_empty() { label } else { identity_label(&label, &c.story_id, &label_links) };
        let label = match expand_notes.get(&r) { Some(n) => format!("{label}{n}"), None => label };
        user.push_str(&format!("\n[{}] ({label})\n{}\n", i + 1 + off, c.text));
    }
    for (i, p) in post.iter().enumerate() {
        user.push_str(&format!("\n[{}] ({})\n{}\n", off + rows.len() + i + 1, p.label, p.text));
    }
    if time_q {
        user.push_str(&timeline_notes(rt, chrono, &rows, off));
    }
    let subject_lines = if time_q && !a.no_chrono_subject && asks_when(q) { chronology_for(q, chrono) } else { String::new() };
    let time_evidence = !a.no_time_evidence && !subject_lines.is_empty();
    if !time_evidence {
        user.push_str(&subject_lines);
    }
    if a.identity_notes && !a.no_route && asks_identity(q) {
        user.push_str(&identity_notes(rt, &rows, chrono));
    }
    if time_evidence {
        user.push_str(&time_evidence_block(&subject_lines));
    }
    user.push_str(&format!("\nQUESTION: {q}"));
    if time_evidence {
        user.push_str(TIME_EVIDENCE_TAIL);
    }
    let started = std::time::Instant::now();
    let mut system = if a.partial { format!("{SYSTEM}{PARTIAL}") } else { SYSTEM.to_owned() };
    if pre.iter().any(|p| p.rule) {
        system.push_str(INFER_RULE);
        if !a.no_age_estimate && asks_age(q) {
            system.push_str(AGE_RULE);
        }
    }
    if rows.iter().any(|&r| rt.store.chunks[r].group_id == "art") {
        system.push_str(ART_RULE);
    }
    if time_q {
        system.push_str(TIME_RULE);
        if (a.date_estimate && asks_when(q)) || !subject_lines.is_empty() {
            system.push_str(DATE_RULE);
        }
    }
    if !a.no_origin_chain && !a.no_route && asks_origin(q) {
        system.push_str(ORIGIN_RULE);
    }
    if !a.no_term_rule && !a.no_route && asks_term(q) {
        system.push_str(TERM_RULE);
    }
    if a.relation_rule && !a.no_route && asks_relation(q) {
        system.push_str(RELATION_RULE);
    }
    if offtopic {
        system.push_str(LORE_ONLY_RULE);
    }
    if !scoped.is_empty() {
        let tail = if pre.iter().any(|p| p.scoped && p.label.contains(CHAR_LABEL)) { CHAR_RULE_TAIL } else { SCOPED_RULE_TAIL };
        system.push_str(&format!("{SCOPED_RULE_HEAD}{}{tail}", scoped.len()));
    }
    if !a.compose.is_empty() {
        system.push_str(COMPOSE_RULE);
    }
    if synth_q {
        system.push_str(SYNTH_RULE);
    }
    if !a.no_route && flags.asks_canon {
        system.push_str(CANON_RULE);
    }
    if form.is_some_and(|f| f.gated) {
        system.push_str(OPINION_RULE);
    } else if opinion_form {
        system.push_str(OPINION_FORM_RULE);
    } else if !a.no_route && is_opinion_question(q) && (!form_on(a) || a.keyword_opinion) {
        system.push_str(OPINION_RULE);
    }
    if form.is_some_and(|f| f.form == "yes_no") {
        system.push_str(if a.verdict_first { YES_NO_VERDICT_RULE } else { YES_NO_RULE });
    }
    match lore {
        // The typed units lead the passages, so the source-kind rule's "say Trevor lacks that text" does not apply:
        // it made the Angelina answer open with a decline although passage [1] was her outfit (2026-09-30).
        // Two parts, spelled out: with the lore passages merely present, Gemma answered from [1] alone (Angelina,
        // 2026-09-30 08:40).
        Some(l) => system.push_str(&format!(" Passages [{}] to [{}] are the source text the question asks about (their label \
names the kind); never say Trevor lacks that text. Answer in two parts. First, what that source text says, citing [{}] to [{}]. \
Then a paragraph that begins \"Lore context:\" and explains the people, places, powers and events the source text mentions, \
from passages [{}] to [{}] (story and operator-file passages about them), with citations; if those passages say nothing about \
what it mentions, say so in one sentence.", 1 + off, l.units + off, 1 + off, l.units + off, l.units + 1 + off,
            l.units + l.lore.max(1) + off)),
        None => {
            if !a.no_route && flags.asks_source_text {
                system.push_str(SOURCE_RULE);
            }
        }
    }
    let mut c = llm
        .complete(&Request {
            system: &system,
            user: &user,
            grammar: None,
            seed: 1,
            temperature: 0.0,
            n_predict: a.n_predict,
            stop: &[],
        })
        .await?;
    // An answer cut off at the limit is asked again with twice the limit (`--no-answer-extend`): greedy decoding of the
    // same prompt repeats the cut answer and goes on.
    if !a.no_answer_extend && c.stop_type == "limit" {
        c = llm.complete(&Request { system: &system, user: &user, grammar: None, seed: 1, temperature: 0.0,
                                    n_predict: a.n_predict * 2, stop: &[] }).await?;
    }
    let mut cited = Vec::new();
    let mut invalid = 0usize;
    let text = c.content.as_str();
    let mut i = 0;
    while let Some(open) = text[i..].find('[') {
        let start = i + open + 1;
        let Some(close) = text[start..].find(']') else { break };
        // "[4]", "[4, 5]" and "[4][5]" all cite; anything else in brackets is prose.
        let inner = &text[start..start + close];
        let nums: Vec<usize> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).filter_map(|t| t.parse().ok()).collect();
        if !nums.is_empty() && nums.len() == inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).count() {
            for n in nums {
                if (1..=off).contains(&n) {
                    let id = first[n - 1].id.clone();
                    if !cited.contains(&id) {
                        cited.push(id);
                    }
                    continue;
                }
                match rows.get(n.wrapping_sub(1 + off)) {
                    Some(&r) => {
                        let id = rt.store.chunks[r].chunk_id.clone();
                        if !cited.contains(&id) {
                            cited.push(id);
                        }
                    }
                    None => match post.get(n.wrapping_sub(1 + off + rows.len())) {
                        Some(p) => {
                            if !cited.contains(&p.id) {
                                cited.push(p.id.clone());
                            }
                        }
                        None => invalid += 1,
                    },
                }
            }
        }
        i = start + close;
    }
    let mut answer = c.content.trim().to_owned();
    if !a.no_route && is_open_listing(q) {
        answer = format!("Trevor has no complete table for this question, so this list comes only from the {} passages it \
read and is not complete.\n\n{answer}", rows.len());
    }
    Ok(Answer {
        qid: None,
        question: q.to_owned(),
        answer,
        cited,
        passages: first.iter().map(|p| p.id.clone())
            .chain(rows.iter().map(|&r| rt.store.chunks[r].chunk_id.clone())).chain(post.iter().map(|p| p.id.clone())).collect(),
        invalid_citations: invalid,
        prompt_tokens: c.prompt_tokens,
        ms: started.elapsed().as_secs_f64() * 1000.0,
        retry_query: None,
        generated: pre.iter().filter(|p| !p.id.starts_with("topic:")).map(|p| (p.id.clone(), p.text.clone())).collect(),
        form: form.map(|f| {
            let base = f.evidence.as_ref().map_or_else(|| f.form.clone(), |e| format!("{}: {e}", f.form));
            f.story.as_ref().map_or(base.clone(), |sq| format!("{base}; story words: {sq}"))
        }),
    })
}

/// A whole-story answer from the summary tree: `passages` as (id, label, text), cited as "overview:<id>".
async fn answer_overview(llm: &Llm, q: &str, a: &Args, passages: &[(String, String, String)]) -> Result<Answer> {
    let started = std::time::Instant::now();
    let mut user = String::from("PASSAGES\n");
    for (i, (_, label, text)) in passages.iter().enumerate() {
        user.push_str(&format!("\n[{}] (Trevor's summary (generated): {label})\n{text}\n", i + 1));
    }
    user.push_str(&format!("\nQUESTION: {q}"));
    let system = format!("{SYSTEM}{OVERVIEW_RULE}");
    let c = llm.complete(&Request { system: &system, user: &user, grammar: None, seed: 1, temperature: 0.0,
                                    n_predict: a.n_predict, stop: &[] }).await?;
    let ids: Vec<String> = passages.iter().map(|(id, _, _)| format!("overview:{id}")).collect();
    let (mut cited, mut invalid) = (Vec::new(), 0usize);
    for inner in c.content.split('[').skip(1).filter_map(|t| t.split_once(']').map(|(x, _)| x)) {
        let parts: Vec<&str> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).collect();
        let nums: Vec<usize> = parts.iter().filter_map(|t| t.parse().ok()).collect();
        if nums.is_empty() || nums.len() != parts.len() {
            continue;
        }
        for n in nums {
            match ids.get(n.wrapping_sub(1)) {
                Some(id) if !cited.contains(id) => cited.push(id.clone()),
                Some(_) => {}
                None => invalid += 1,
            }
        }
    }
    Ok(Answer { qid: None, question: q.to_owned(), answer: c.content.trim().to_owned(), cited, passages: ids,
                invalid_citations: invalid, prompt_tokens: c.prompt_tokens, ms: started.elapsed().as_secs_f64() * 1000.0,
                retry_query: None, form: None, generated: Default::default() })
}

/// A llama-server this process started; killed when dropped, so it never outlives `ask`.
struct Spawned(std::process::Child, Option<PathBuf>);

impl Drop for Spawned {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
        // The lock this process wrote for its server, only while it still names this process.
        if let Some(l) = &self.1 {
            let me = std::process::id().to_string();
            if std::fs::read_to_string(l).is_ok_and(|t| t.split_whitespace().next() == Some(me.as_str())) {
                let _ = std::fs::remove_file(l);
            }
        }
    }
}

/// The model lock's holder, when it is another job: (pid, label). A lock whose pid is this process or one of its
/// ancestors is ours (the wrapper script that took the lock and started `ask`).
fn foreign_lock(path: &std::path::Path) -> Option<(u32, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    let (pid, label) = text.trim().split_once(' ').unwrap_or((text.trim(), ""));
    let pid: u32 = pid.parse().ok()?;
    let mut p = std::process::id();
    for _ in 0..64 {
        if p == pid {
            return None;
        }
        let out = std::process::Command::new("ps").args(["-o", "ppid=", "-p", &p.to_string()]).output().ok()?;
        match String::from_utf8_lossy(&out.stdout).trim().parse::<u32>() {
            Ok(pp) if pp > 1 && pp != p => p = pp,
            _ => break,
        }
    }
    Some((pid, label.to_owned()))
}

/// Wait while another job holds the model lock (or fail with `--no-wait`).
async fn wait_for_lock(a: &Args) -> Result<()> {
    if a.ignore_lock {
        return Ok(());
    }
    let mut told = false;
    while let Some((pid, label)) = foreign_lock(&a.lock) {
        if a.no_wait {
            anyhow::bail!("the model is busy with {label} (pid {pid}, lock {}); --no-wait was given", a.lock.display());
        }
        if !told {
            eprintln!("the model is busy with {label} (pid {pid}); waiting for {} to go", a.lock.display());
            told = true;
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
    Ok(())
}

/// Connect to the server at `a.server`, or start one on its port with the answer model and wait
/// until it is healthy (the model loads in about 15 s). Waits first while another job holds the model lock.
async fn connect_or_spawn(a: &Args) -> Result<(Llm, Option<Spawned>)> {
    let (llm, guard) = connect_or_spawn_plain(a).await?;
    let llm = if a.no_cache_prompt { llm.without_prompt_cache() } else { llm };
    let llm = match &a.dump_prompt { Some(p) => llm.with_dump(p)?, None => llm };
    Ok((llm, guard))
}

async fn connect_or_spawn_plain(a: &Args) -> Result<(Llm, Option<Spawned>)> {
    wait_for_lock(a).await?;
    if let Ok(llm) = Llm::connect(&a.server).await {
        return Ok((llm, None));
    }
    if a.no_spawn {
        return Err(anyhow::anyhow!("no llama-server at {} (and --no-spawn was given)", a.server));
    }
    let port = a.server.rsplit(':').next().unwrap_or("8081").trim_end_matches('/').to_owned();
    eprintln!("no llama-server at {}; starting {} (about 15 s)...", a.server, a.model.display());
    let child = std::process::Command::new("llama-server")
        .args(["-m"]).arg(&a.model)
        .args(["--host", "127.0.0.1", "--port", &port, "-np", "1", "--kv-unified-per-slot", "16384", "-fa", "on",
               "-cram", "512", "--no-webui", "--reasoning", "off", "-ngl", "all"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("starting llama-server (brew install llama.cpp)")?;
    let guard = Spawned(child, (!a.ignore_lock).then(|| a.lock.clone()));
    if let Some(l) = &guard.1 {
        let _ = std::fs::write(l, format!("{} ask\n", std::process::id()));
    }
    for _ in 0..180 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Ok(llm) = Llm::connect(&a.server).await {
            return Ok((llm, Some(guard)));
        }
    }
    Err(anyhow::anyhow!("llama-server did not become ready in 180 s"))
}

fn load_knn(a: &Args, tools: &Tools) -> Result<Knn> {
    let p = KnnParams { k: a.knn_k, min_sim: a.knn_min_sim, min_share: a.knn_min_share };
    Knn::load(&a.knn_model_dir, &a.knn_onnx, &a.intents, a.knn_synth.as_deref(), p, tools)
}

/// `--label-intents`: the model router's choice for every question of the three real-question sets, resumable.
/// Route-check expectations are hand-verified, so they win: a question expected to go to retrieval is labelled
/// `retrieve`, and one expected at a table takes the keyword router's tool and arguments (61 of 61 pass).
async fn label_intents(tools: &Tools, llm: &Llm, out_path: &std::path::Path) -> Result<()> {
    let done: BTreeSet<String> = std::fs::read_to_string(out_path).unwrap_or_default().lines()
        .filter_map(|l| serde_json::from_str::<Intent>(l).ok().map(|i| i.q)).collect();
    let mut todo: Vec<(String, String, Option<serde_json::Value>)> = Vec::new();
    let mut seen = done.clone();
    for (file, source) in [("eval/routes.jsonl", "routes"), ("eval/reddit-lore-questions.all.jsonl", "reddit"),
                           ("eval/discord-lore-questions.jsonl", "discord")] {
        for l in std::fs::read_to_string(file).with_context(|| format!("reading {file}"))?.lines() {
            let v: serde_json::Value = serde_json::from_str(l)?;
            // X is noise and P out of scope (gameplay), design/trevor-questions.md section 2; Discord has neither.
            if matches!(v["category"].as_str(), Some("X" | "P")) {
                continue;
            }
            let q = v["q"].as_str().context("item without q")?.to_owned();
            if seen.insert(q.clone()) {
                // A case naming its tool (canon, topic, Storylines cases) is checked on the tool, not table or retrieval.
                let expect = (source == "routes").then(|| if v["tool"].is_string() { v["tool"].clone() } else { v["expect"].clone() });
                todo.push((q, source.to_owned(), expect));
            }
        }
    }
    eprintln!("label-intents: {} to do, {} done, model {}", todo.len(), done.len(), llm.model);
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(out_path)?;
    let started = std::time::Instant::now();
    let mut ms = Vec::new();
    for (n, (q, source, expect)) in todo.iter().enumerate() {
        let t0 = std::time::Instant::now();
        let m = router::route_model(llm, q).await?;
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        let mut it = Intent { q: q.clone(), tool: m.tool.clone(), args: m.args.clone(), source: source.clone(), model_tool: None,
                               source_q: None, verified: None };
        match expect.as_ref().and_then(|e| e.as_str()) {
            Some("retrieval") if !m.is_retrieve() => {
                it.model_tool = Some(m.tool.clone());
                it.tool = trevor::tools::RETRIEVE.into();
                it.args.clear();
            }
            Some(tool) if tool != "table" && tool != "retrieval" => {
                if m.tool != tool {
                    it.model_tool = Some(m.tool.clone());
                    it.tool = tool.to_owned();
                    it.args.clear();
                }
            }
            Some("table") => {
                if let Some((kr, _)) = keyword_route(q, tools) {
                    if kr != m {
                        it.model_tool = Some(m.tool.clone());
                        it.tool = kr.tool;
                        it.args = kr.args;
                    }
                }
            }
            _ => {}
        }
        serde_json::to_writer(&mut out, &it)?;
        out.write_all(b"\n")?;
        if (n + 1) % 50 == 0 {
            #[allow(clippy::cast_precision_loss)]
            let each = started.elapsed().as_secs_f64() / (n + 1) as f64;
            eprintln!("label-intents {}/{}, {each:.2} s each", n + 1, todo.len());
        }
    }
    if !ms.is_empty() {
        ms.sort_by(f64::total_cmp);
        #[allow(clippy::cast_precision_loss)]
        let mean = ms.iter().sum::<f64>() / ms.len() as f64;
        eprintln!("model router latency over {} questions: mean {mean:.0} ms, median {:.0} ms, p90 {:.0} ms", ms.len(),
                  ms[ms.len() / 2], ms[ms.len() * 9 / 10]);
    }
    Ok(())
}

/// A seeded shuffle (xorshift64), so the split is the same on every run.
fn shuffled(n: usize, seed: u64) -> Vec<usize> {
    let mut x = seed.max(1);
    let mut idx: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let j = usize::try_from(x % (i as u64 + 1)).unwrap_or(0);
        idx.swap(i, j);
    }
    idx
}

/// The seeded 80/20 split of the real labelled intents: (train, test) indices. Paraphrases are generated from the
/// training part only, so the test questions and anything written from them stay unseen.
fn intent_split(n: usize) -> (Vec<usize>, Vec<usize>) {
    let order = shuffled(n, 20_260_929);
    let cut = n * 4 / 5;
    (order[..cut].to_vec(), order[cut..].to_vec())
}

fn read_intents(p: &std::path::Path) -> Result<Vec<Intent>> {
    Ok(std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?.lines()
        .map(serde_json::from_str).collect::<std::result::Result<_, _>>()?)
}

/// Two routes reach the same table target: the same tool and either the same arguments (folded) or the same answer.
fn same_target(tools: &Tools, a: &Route, b: &Route) -> bool {
    if a.tool != b.tool {
        return false;
    }
    let fold = |r: &Route| r.args.iter().map(|(k, v)| (k.clone(), trevor::tools::norm(v))).collect::<Vec<_>>();
    if fold(a) == fold(b) || a.args.is_empty() {
        return true;
    }
    if a.tool == "topic" {
        let t = |r: &Route| r.args.get("topic").and_then(|x| tools.topic(x).ok()).map(|p| p.topic);
        return t(a).is_some() && t(a) == t(b);
    }
    matches!((tools.run(a), tools.run(b)), (Ok(x), Ok(y)) if x == y)
}

/// Asks for paraphrases of one question (`--paraphrase-intents`).
const PARAPHRASE: &str = "You rewrite one question about the Arknights story the way different players would ask it on \
Reddit or Discord. Vary the wording, length and register: casual, lowercase, slang, a typo, a short one, one with a line of \
context before the question. Every rewrite must ask for exactly the same thing about the same subject (the same event, \
character, operator, Integrated Strategies run or topic, by its name or a common nickname), so that the same answer fits \
it. Output a JSON array of strings only.";

/// `--paraphrase-intents OUT`: `--paraphrases` rewrites of every table-labelled question of the TRAINING part of the
/// split (canon and topic included), each routed by the model router and marked verified when it reaches the source's
/// tool and target. Resumable by source question.
async fn paraphrase_intents(a: &Args, tools: &Tools, llm: &Llm, out_path: &std::path::Path) -> Result<()> {
    let items = read_intents(&a.intents)?;
    let (train, test) = intent_split(items.len());
    let test_q: BTreeSet<&str> = test.iter().map(|&i| items[i].q.as_str()).collect();
    let done: BTreeSet<String> = std::fs::read_to_string(out_path).unwrap_or_default().lines()
        .filter_map(|l| serde_json::from_str::<Intent>(l).ok()?.source_q).collect();
    let todo: Vec<&Intent> = train.iter().map(|&i| &items[i])
        .filter(|i| i.tool != trevor::tools::RETRIEVE && !done.contains(&i.q) && !test_q.contains(i.q.as_str())).collect();
    let n = a.paraphrases.max(1);
    let grammar = format!("root ::= \"[\" item (\", \" item){{{}}} \"]\"\nitem ::= \"\\\"\" [^\"\\\\\\x00-\\x1F]{{8,220}} \"\\\"\"\n", n - 1);
    eprintln!("paraphrase-intents: {} table-labelled training questions to do, {} done, {n} each", todo.len(), done.len());
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(out_path)?;
    let (mut kept, mut all) = (0usize, 0usize);
    let started = std::time::Instant::now();
    for (k, it) in todo.iter().enumerate() {
        let c = llm.complete(&Request { system: PARAPHRASE, user: &format!("Write {n} rewrites of: {}", it.q), grammar: Some(&grammar),
                                        seed: 1, temperature: 0.8, n_predict: 3000, stop: &[] }).await?;
        let list: Vec<String> = serde_json::from_str(c.content.trim()).unwrap_or_default();
        let label = Route { tool: it.tool.clone(), args: it.args.clone() };
        let mut seen = BTreeSet::new();
        for p in list.into_iter().filter(|p| p.trim() != it.q.trim() && seen.insert(p.trim().to_lowercase())) {
            let m = router::route_model(llm, &p).await?;
            let ok = same_target(tools, &label, &m);
            all += 1;
            kept += usize::from(ok);
            let row = Intent { q: p, tool: it.tool.clone(), args: it.args.clone(), source: "synthetic".into(),
                               model_tool: (!ok).then(|| m.tool.clone()), source_q: Some(it.q.clone()), verified: Some(ok) };
            serde_json::to_writer(&mut out, &row)?;
            out.write_all(b"\n")?;
        }
        if (k + 1) % 10 == 0 {
            #[allow(clippy::cast_precision_loss)]
            let each = started.elapsed().as_secs_f64() / (k + 1) as f64;
            eprintln!("paraphrase-intents {}/{}, {kept}/{all} verified, {each:.1} s per source", k + 1, todo.len());
        }
    }
    eprintln!("paraphrase-intents done: {kept}/{all} verified");
    Ok(())
}

/// Whether the kNN vote is sure enough for the hybrid router to skip the model: the thresholds, and for a table
/// tool also arguments found in the question and a tool that answers.
fn knn_sure(v: &router::Vote, q: &str, d: &router::Dict, tools: &Tools, min_sim: f32, min_share: f32) -> Option<Route> {
    if v.top < min_sim || v.share < min_share {
        return None;
    }
    if v.tool == trevor::tools::RETRIEVE {
        return Some(Route::retrieve());
    }
    let r = router::extract_args(&v.tool, q, d)?;
    let works = if r.tool == "topic" { r.args.get("topic").is_some_and(|t| tools.topic(t).is_ok()) } else { tools.run(&r).is_ok() };
    works.then_some(r)
}

/// `--knn-eval`: the kNN and hybrid routers on the seeded 80/20 split of the REAL labelled intents; paraphrases
/// (`--knn-synth`) join the training part only. Thresholds are swept by leave-one-out on the training part (a
/// question's own paraphrases left out with it) and reported on the test part at the training pick.
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn knn_eval(a: &Args, tools: &Tools) -> Result<()> {
    let knn = load_knn(a, tools)?;
    let items = read_intents(&a.intents)?;
    let (train, test) = intent_split(items.len());
    let test_q: BTreeSet<&str> = test.iter().map(|&i| items[i].q.as_str()).collect();
    // Training examples: the real training questions, then paraphrases of training questions only.
    let mut tr: Vec<usize> = train.clone();
    let mut leaked = 0usize;
    for i in knn.n_real..knn.examples.len() {
        if test_q.contains(knn.origins[i].as_str()) { leaked += 1 } else { tr.push(i) }
    }
    let ex: Vec<(Vec<f32>, String)> = tr.iter().map(|&i| knn.examples[i].clone()).collect();
    let origin: Vec<&str> = tr.iter().map(|&i| knn.origins[i].as_str()).collect();
    eprintln!("knn-eval: {} real labelled (train {}, test {}), {} paraphrases in training ({leaked} of test questions refused), k {}, model {}",
              items.len(), train.len(), test.len(), tr.len() - train.len(), a.knn_k, a.knn_model_dir.display());
    let d = &knn.dict;
    let runs = |r: &Route| !r.is_retrieve() && (if r.tool == "topic" { r.args.get("topic").is_some_and(|t| tools.topic(t).is_ok()) } else { tools.run(r).is_ok() });
    let label = |i: usize| {
        let r = Route { tool: items[i].tool.clone(), args: items[i].args.clone() };
        if runs(&r) || (r.tool == "topic" && r.args.is_empty()) { r } else { Route::retrieve() }
    };
    // What the model router chose (the label unless a route-check expectation overrode it).
    let model_tool = |i: usize| items[i].model_tool.clone().unwrap_or_else(|| label(i).tool);
    let kmax = a.knn_k.max(1);
    // Neighbours once per question: training queries leave out themselves and their paraphrases.
    let train_neigh: Vec<Vec<(f32, usize)>> = (0..train.len()).map(|j| {
        let o = origin[j];
        router::neighbours(&ex, &knn.examples[train[j]].0, kmax, |x| origin[x] == o)
    }).collect();
    let test_neigh: Vec<Vec<(f32, usize)>> = test.iter().map(|&i| router::neighbours(&ex, &knn.examples[i].0, kmax, |_| false)).collect();
    let decide_knn = |v: &router::Vote, q: &str, p: KnnParams| {
        let r = router::decide(v, p, q, d);
        if runs(&r) { r } else { Route::retrieve() }
    };
    let score = |pairs: &[(Route, Route)]| {
        let acc = pairs.iter().filter(|(p, l)| p.tool == l.tool).count() as f64 / pairs.len().max(1) as f64;
        let tab = pairs.iter().filter(|(p, _)| !p.is_retrieve()).count();
        let ok = pairs.iter().filter(|(p, l)| !p.is_retrieve() && p.tool == l.tool).count();
        let lab = pairs.iter().filter(|(_, l)| !l.is_retrieve()).count();
        (acc, ok, tab, lab)
    };
    println!("kNN sweep (train, leave-one-out): min_sim min_share -> tool accuracy, table answers right/given, table labels");
    let mut rows: Vec<(KnnParams, f64, f64)> = Vec::new();
    for min_sim in [0.0f32, 0.80, 0.85, 0.88, 0.90, 0.92, 0.94] {
        for min_share in [0.5f32, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let p = KnnParams { k: a.knn_k, min_sim, min_share };
            let pairs: Vec<(Route, Route)> = train.iter().enumerate()
                .map(|(j, &i)| (decide_knn(&router::vote_from(&ex, &train_neigh[j], p.k), &items[i].q, p), label(i))).collect();
            let (acc, ok, tab, lab) = score(&pairs);
            let wrong = (tab - ok) as f64 / pairs.len().max(1) as f64;
            println!("  {min_sim:.2} {min_share:.1} -> {acc:.3} {ok}/{tab} of {lab}");
            rows.push((p, acc, wrong));
        }
    }
    // A trade, not a derivation: a wrong table answer counts twice a missed one (see round 1).
    let picked = rows.iter().max_by(|x, y| (x.1 - 2.0 * x.2).total_cmp(&(y.1 - 2.0 * y.2))).map_or(
        KnnParams { k: a.knn_k, min_sim: a.knn_min_sim, min_share: a.knn_min_share }, |r| r.0);
    let flags = KnnParams { k: a.knn_k, min_sim: a.knn_min_sim, min_share: a.knn_min_share };
    for (what, p) in [("the training pick", picked), ("the flags", flags)] {
        let pairs: Vec<(Route, Route)> = test.iter().enumerate()
            .map(|(j, &i)| (decide_knn(&router::vote_from(&ex, &test_neigh[j], p.k), &items[i].q, p), label(i))).collect();
        let (acc, ok, tab, lab) = score(&pairs);
        let agree = pairs.iter().zip(&test).filter(|(pl, i)| pl.0.tool == model_tool(**i)).count();
        let right_lab = pairs.iter().filter(|(p, l)| !l.is_retrieve() && p.tool == l.tool).count();
        println!("\nkNN on test at {what}: min_sim {:.2}, min_share {:.1} (k {})", p.min_sim, p.min_share, p.k);
        println!("  tool accuracy {acc:.3}; agreement with the model router {:.3} ({agree}/{}); table answers {ok} right of {tab} given; \
table recall {right_lab}/{lab}", agree as f64 / test.len() as f64, test.len());
        let tools_seen: BTreeSet<String> = pairs.iter().flat_map(|(p, l)| [p.tool.clone(), l.tool.clone()]).collect();
        println!("  per tool: label n, kNN n, right, precision, recall");
        for t in &tools_seen {
            let ln = pairs.iter().filter(|(_, l)| &l.tool == t).count();
            let pn = pairs.iter().filter(|(p, _)| &p.tool == t).count();
            let r = pairs.iter().filter(|(p, l)| &p.tool == t && &l.tool == t).count();
            println!("    {t:22} {ln:4} {pn:4} {r:4} {:.3} {:.3}", r as f64 / pn.max(1) as f64, r as f64 / ln.max(1) as f64);
        }
        for ((pr, lab), &i) in pairs.iter().zip(&test) {
            if !pr.is_retrieve() && pr.tool != lab.tool {
                println!("  wrong: {} {} (label {}): {}", pr.tool, serde_json::to_string(&pr.args)?, lab.tool, items[i].q);
            }
        }
    }
    // Hybrid: kNN when sure, else the model router's own choice (its recorded label), so no model runs here.
    let hybrid = |neigh: &[(f32, usize)], i: usize, hs: f32, hsh: f32| -> (String, bool) {
        let v = router::vote_from(&ex, neigh, a.knn_k);
        match knn_sure(&v, &items[i].q, d, tools, hs, hsh) {
            Some(r) => (r.tool, true),
            None => (model_tool(i), false),
        }
    };
    println!("\nhybrid sweep (train, leave-one-out): min_sim min_share -> agreement with the model router, share with no LLM call");
    let mut hrows = Vec::new();
    for hs in [0.0f32, 0.60, 0.70, 0.75, 0.80, 0.85, 0.88, 0.90, 0.92, 0.94, 0.96] {
        for hsh in [0.6f32, 0.7, 0.8, 0.9, 1.0] {
            let r: Vec<(String, bool)> = train.iter().enumerate().map(|(j, &i)| hybrid(&train_neigh[j], i, hs, hsh)).collect();
            let agree = r.iter().zip(&train).filter(|(x, i)| x.0 == model_tool(**i)).count() as f64 / train.len() as f64;
            let free = r.iter().filter(|x| x.1).count() as f64 / train.len() as f64;
            println!("  {hs:.2} {hsh:.1} -> {agree:.3} {free:.3}");
            hrows.push((hs, hsh, agree, free));
        }
    }
    // A trade: the most questions without an LLM call while agreeing with the model router on 0.98 of them.
    let hp = hrows.iter().filter(|r| r.2 >= 0.98).max_by(|x, y| x.3.total_cmp(&y.3).then(x.2.total_cmp(&y.2)))
        .map_or((a.hybrid_min_sim, a.hybrid_min_share), |r| (r.0, r.1));
    for (what, (hs, hsh)) in [("the training pick", hp), ("the flags", (a.hybrid_min_sim, a.hybrid_min_share))] {
        let r: Vec<(String, bool)> = test.iter().enumerate().map(|(j, &i)| hybrid(&test_neigh[j], i, hs, hsh)).collect();
        let agree = r.iter().zip(&test).filter(|(x, i)| x.0 == model_tool(**i)).count();
        let free = r.iter().filter(|x| x.1).count();
        let labs: Vec<Route> = test.iter().map(|&i| label(i)).collect();
        let tab = r.iter().filter(|(t, _)| t != trevor::tools::RETRIEVE).count();
        let ok = r.iter().zip(&labs).filter(|((t, _), l)| t != trevor::tools::RETRIEVE && *t == l.tool).count();
        let nlab = labs.iter().filter(|l| !l.is_retrieve()).count();
        let knn_wrong = r.iter().zip(&test).filter(|(x, i)| x.1 && x.0 != model_tool(**i)).count();
        println!("\nhybrid on test at {what}: min_sim {hs:.2}, min_share {hsh:.1}");
        println!("  agreement with the model router {:.3} ({agree}/{}); no LLM call {:.3} ({free}/{}); kNN decisions that differ from \
the model {knn_wrong}; table answers {ok} right of {tab}; table recall {ok}/{nlab}", agree as f64 / test.len() as f64, test.len(),
                 free as f64 / test.len() as f64, test.len());
    }
    let model_ok = test.iter().filter(|&&i| model_tool(i) == label(i).tool).count();
    println!("\nmodel router alone on test: tool accuracy against the labels {:.3} ({model_ok}/{})", model_ok as f64 / test.len() as f64, test.len());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // `ask` answers from P3b (P3a plus the 200 dossiers and 88 event summaries) unless --corpus is given: on 120
    // real Reddit questions it responds 0.575 against 0.550 (4 gained, 1 lost) and on gold set v2 0.641 against
    // 0.618 (Ian, 2026-09-28). Search and eval keep P3a, where P3b costs 0.015 retrieval recall.
    let m = <Args as clap::CommandFactory>::command().get_matches();
    let mut a = <Args as clap::FromArgMatches>::from_arg_matches(&m)?;
    let corpus_default = m.value_source("corpus") == Some(clap::parser::ValueSource::DefaultValue);
    // RELATION_RULE is the default since 2026-10-05 night (`--no-relation-rule`).
    a.relation_rule = !a.no_relation_rule;
    if corpus_default {
        // P4 (P3b plus 6,347 chunks of module stories, voice lines, IS, enemies, outfits and items) only for a question
        // that names one of those sources: on gold set v2 it cost 0.008 correct and 0.025 faithfulness against P3b,
        // while source-specific real questions went from 1 to 3 of 7 (2026-09-29).
        // With the model router naming the source, the corpus follows its route instead (set after routing).
        let source_q = !router_source(&a) && a.batch.is_none() && a.question.as_deref().is_some_and(|q| source_kind(q).is_some());
        let (p3b, p4) = (a.lore.dirs().0, p4_dir(&a));
        a.runtime.corpus = PathBuf::from(if source_q { p4 } else { p3b });
    }
    let chrono = load_chrono();
    let mut tools = Tools::load(std::path::Path::new("."));
    if a.lore == LoreSet::V1 {
        tools.topics_v1();
    } else {
        tools.load_new_dossiers(std::path::Path::new("."));
        tools.game_data = !a.no_game_data;
        tools.topics_v2_text = true;
        tools.topics_speaker_check = a.speaker_trait_check;
        tools.topics_race_rule = !a.no_race_trait_rule;
        // Deep entries are switched on per question (`deep_for`, the broad-question gate).
        tools.topics_deep = false;
    }
    // The keyword router is the kill switch for all routing work, so it keeps the tools as they were.
    // The exact recap match changes the keyword router's "Explain the ending of Episode 1" (and Episode 0), so it
    // applies to the model and kNN routers only.
    tools.opts = if a.router == RouterKind::Keywords { ToolOptions { canon_v1: a.canon_v1, ..ToolOptions::KEYWORDS } }
        else { ToolOptions { storylines: !a.no_storylines, exact_recap: !a.recap_substring, is_offset: !a.is_numbering_v1,
                             canon_v1: a.canon_v1, wiki_legacy: a.wiki_legacy } };
    if a.wiki_legacy {
        tools.opts.wiki_legacy = true;
        tools.design_basis_wiki = trevor::reference::wiki_design_basis(std::path::Path::new("."));
    }
    if a.knn_eval {
        return knn_eval(&a, &tools);
    }
    if a.show_expansion {
        let rt = Runtime::load(&a.runtime, false, false, false)?;
        let qs: Vec<String> = match &a.batch {
            Some(b) => std::fs::read_to_string(b)?.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                .filter_map(|v| v["question"].as_str().map(str::to_owned)).collect(),
            None => a.question.clone().into_iter().collect(),
        };
        for q in qs {
            let e = expand_names(&rt, &tools, &q);
            if e != q {
                println!("{q}\t{e}");
            }
        }
        return Ok(());
    }
    if a.form_only {
        let rt = Runtime::load(&a.runtime, false, false, false)?;
        let (llm, _server) = connect_or_spawn(&a).await?;
        let qs: Vec<(String, String)> = match &a.batch {
            Some(b) => std::fs::read_to_string(b)?.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                .filter_map(|v| Some((v["qid"].as_str().unwrap_or_default().to_owned(), v["question"].as_str()?.to_owned()))).collect(),
            None => a.question.clone().map(|q| (String::new(), q)).into_iter().collect(),
        };
        for (id, q) in qs {
            let e = if a.no_name_expansion { q.clone() } else { expand_names(&rt, &tools, &q) };
            let f = question_form(&llm, &e, a.form_v1, a.model_flags).await?;
            match f.flags {
                // With the flags: the model's flags, then the keyword tests' (to compare).
                Some(fl) => println!("{id}\t{}\t{}\t{}\t{}", f.form, f.evidence.unwrap_or_default(), fl.show(),
                                     Flags::keywords(&e).show()),
                None => println!("{id}\t{}\t{}", f.form, f.evidence.unwrap_or_default()),
            }
        }
        return Ok(());
    }
    if let Some(out) = a.paraphrase_intents.clone() {
        let (llm, _server) = connect_or_spawn(&a).await?;
        return paraphrase_intents(&a, &tools, &llm, &out).await;
    }
    if let Some(out) = a.label_intents.clone() {
        let (llm, _server) = connect_or_spawn(&a).await?;
        return label_intents(&tools, &llm, &out).await;
    }
    let knn = if matches!(a.router, RouterKind::Knn | RouterKind::Hybrid) && !a.no_route { Some(load_knn(&a, &tools)?) } else { None };
    let mut tables = Tables { tools, knn, topic: None, route: None };
    // Table-routed questions need no index: answer them before loading it. The keyword, kNN and off routers need
    // no model either, so they answer before starting or connecting to a server.
    let mut early: Option<(Llm, Option<Spawned>)> = None;
    // `--route-only --batch FILE`: one line per question, "qid<TAB>route<TAB>first line of the route-only output", in
    // one process (2026-10-03; the per-question process loads the tools each time).
    if let (Some(b), true) = (&a.batch, a.detect_only) {
        // The chunk store alone, for the proper-name check of the scoped scenes and the opinion gate's named characters.
        let store_rt = Runtime::load(&a.runtime, false, false, false).ok();
        for l in std::fs::read_to_string(b)?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(l)?;
            let (id, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
            let t = &tables.tools;
            let mut f: Vec<String> = Vec::new();
            if let Some(r) = is_ending_route(q, &a, t) { f.push(format!("is_ending={}", serde_json::to_string(&r.args)?)); }
            if let Some(c) = t.appearance_question(q) { f.push(format!("appearances={c}")); }
            if let Some(x) = t.cross_ref_question(q) { f.push(format!("cross_ref={x}")); }
            if t.design_basis(q).is_some() { f.push("design_basis".into()); }
            if let Some(term) = term_of(q) { f.push(format!("term={term}")); }
            if asks_relation(q) {
                let people = store_rt.as_ref().map_or(0, |rt| people_named(rt, t, q).len());
                f.push(if t.names_group_topic(q) { "relation-group".into() } else { format!("relation(people={people}{})",
                    if asks_admire(q) { ",admire" } else { "" }) });
            }
            if Flags::keywords(q).asks_time && asks_when(q) && !chronology_for(q, load_chrono_cached()).is_empty() { f.push("chrono".into()); }
            if let Some(tp) = t.named_topic(q) { f.push(format!("named_topic={}", tp.topic)); }
            if asks_identity(q) { if let Some(c) = t.answer_constraint(q) { f.push(format!("constraint={c:?}")); } }
            if t.broad_topic_question(q) { f.push("broad_topic".into()); }
            if let Some(g) = t.event_in_question(q) {
                let gid = g["groupId"].as_str().unwrap_or_default();
                let who: Vec<String> = t.event_characters(q, gid, g["name"].as_str().unwrap_or_default()).into_iter()
                    .filter(|(n, _)| store_rt.as_ref().is_none_or(|rt| proper_name(rt, n))).map(|(n, _)| n).collect();
                f.push(if who.is_empty() { format!("event={gid}") } else { format!("scoped={gid}:{}", who.join("|")) });
            }
            if let Some(rt) = store_rt.as_ref() {
                let e = expand_names(rt, t, q);
                if !opinion_needs_candidates(rt, t, &e, !named_characters(rt, t, &e).is_empty()) {
                    f.push(if is_opinion_question(&e) && !names_proper_word(rt, &e) { "opinion-gate".into() } else { "opinion-gate-wide".into() });
                }
            }
            if let Some(rt) = store_rt.as_ref().filter(|_| t.event_in_question(q).is_none() && t.appearance_question(q).is_none()) {
                if let [k] = people_named(rt, t, q).as_slice() {
                    let forms: Vec<String> = person_forms(t, k).iter().map(|x| trevor::tools::norm(x)).collect();
                    if !t.names_group_topic(k) && asks_scene(q, &forms) { f.push(format!("charscope={k}")); }
                }
            }
            if let Some(op) = one_operator(q, t) { f.push(format!("one_operator={op}")); }
            else if let Some(op) = one_operator_wide(q, t) { f.push(format!("one_operator_wide={op}")); }
            println!("{id}\t{}", f.join(" "));
        }
        return Ok(());
    }
    if let (Some(b), true, false) = (&a.batch, a.route_only, a.no_route) {
        let llm = if matches!(a.router, RouterKind::Model | RouterKind::Hybrid) { Some(connect_or_spawn(&a).await?) } else { None };
        for l in std::fs::read_to_string(b)?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(l)?;
            let (id, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
            tables.tools.topics_deep = deep_for(&a, &tables.tools, q);
            let r = route_question(q, &a, &tables.tools, tables.knn.as_mut(), llm.as_ref().map(|x| &x.0)).await?;
            let head = r.text.as_deref().map_or_else(|| {
                let topic = r.topic.as_ref().map_or(String::new(), |t| format!(" (topic passage: {})", t.topic));
                let dossier = r.dossier.as_ref().map_or(String::new(), |d| format!(" (dossier passage: {d})"));
                let game = r.game.as_ref().map_or(String::new(), |g| format!(" (game data: {g})"));
                // `--lore-only`: whether the off-topic sentence would be added (the question names nothing Trevor knows).
                let off = if a.lore_only && !tables.tools.names_anything(q) { " (off-topic rule)" } else { "" };
                format!("RETRIEVAL{topic}{dossier}{game}{off}")
            }, |t| format!("TABLE {}", t.lines().next().unwrap_or_default()));
            println!("{id}\t{}\t{head}", serde_json::to_string(&r.route)?);
        }
        return Ok(());
    }
    if let (None, Some(q), false) = (&a.batch, &a.question, a.no_route) {
        // The hybrid router connects only when kNN is unsure, which it learns after the vote; it connects up front
        // like the model router for simplicity, since a question it routes to retrieval needs the server anyway.
        let llm = if matches!(a.router, RouterKind::Model | RouterKind::Hybrid) { Some(connect_or_spawn(&a).await?) } else { None };
        tables.tools.topics_deep = deep_for(&a, &tables.tools, q);
        let r = route_question(q, &a, &tables.tools, tables.knn.as_mut(), llm.as_ref().map(|x| &x.0)).await?;
        if a.route_only {
            if a.router == RouterKind::Model {
                eprintln!("model router: {:.0} ms", r.ms);
            }
            return print_route_only(q, &a, &r);
        }
        if let Some(text) = r.text {
            println!("{text}");
            return Ok(());
        }
        tables.topic = r.topic;
        if corpus_default && retrieval_plan(q, &a, Some(&r.route)).0 == Some(true) {
            a.runtime.corpus = PathBuf::from(p4_dir(&a));
        }
        tables.route = Some(r.route);
        early = llm;
    }
    // Loaded after the table routes, which need no index (a table answer no longer waits on the ONNX load).
    let mut rts = Runtimes { main: Runtime::load(&a.runtime, true, true, a.rerank_add > 0)?, p4: None, art: None, p4_dir: p4_dir(&a),
                             switch: corpus_default && router_source(&a), args: a.runtime.clone(), rerank: a.rerank_add > 0 };
    if let (Some(b), true) = (&a.batch, a.line_study) {
        return line_study(&mut rts.main, &rts.args, b);
    }
    if let (Some(b), true) = (&a.batch, a.scoped_only) {
        for l in std::fs::read_to_string(b)?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(l)?;
            let (id, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
            if let Some(ps) = v["passages"].as_array() {
                let ps: Vec<String> = ps.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect();
                let names: HashMap<String, Names> = std::fs::read("artifacts/goldgen/names.json").ok()
                    .and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
                for (cid, label, _) in compose_evidence(&mut rts.main, &load_chrono(), &names, q, &ps) {
                    println!("{id}\tcompose\t{cid}\t{label}");
                }
            }
            if let Some((e, who, rows)) = scoped_scenes(&mut rts.main, &tables.tools, q) {
                let ids: Vec<&str> = rows.iter().map(|&r| rts.main.store.chunks[r].chunk_id.as_str()).collect();
                println!("{id}\t{e}\t{}\t{}", who.join("|"), ids.join(" "));
            }
            if let Some((who, rows)) = character_scenes(&mut rts.main, &rts.args, &tables.tools, q) {
                let ids: Vec<String> = rows.iter().map(|(r, _, n)| format!("{}:{n}", rts.main.store.chunks[*r].chunk_id)).collect();
                println!("{id}\tcharacter\t{who}\t{}", ids.join(" "));
            }
        }
        return Ok(());
    }
    // `--detect-only --batch` with the index: which identity questions would get identity notes from their plain hybrid
    // top k over P3b or P4 (the prefilter of the 2026-10-03 A/B; the answer's own passages may differ by the form's query).
    if let (Some(b), true) = (&a.batch, a.detect_identity) {
        let chrono = load_chrono();
        for l in std::fs::read_to_string(b)?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(l)?;
            let (id, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
            if !asks_identity(q) {
                continue;
            }
            let cfg = RetrievalConfig { mode: Mode::Hybrid, k: a.k, ..RetrievalConfig::default() };
            let mut fired = Vec::new();
            for p4 in [false, true] {
                let rt = if p4 { rts.pick(Some(true))? } else { &mut rts.main };
                let rows: Vec<usize> = rt.retrieve(q, &cfg)?.iter().map(|h| h.row).collect();
                if !identity_notes(rt, &rows, &chrono).is_empty() {
                    fired.push(if p4 { "p4" } else { "p3b" });
                }
                if !label_links(rt, &rows, &chrono).is_empty() {
                    fired.push(if p4 { "labels-p4" } else { "labels-p3b" });
                }
                if identity_expand(rt, &chrono, q, rows.clone(), 2).0.len() > rows.len() {
                    fired.push(if p4 { "expand-p4" } else { "expand-p3b" });
                }
            }
            println!("{id}\t{}", fired.join(" "));
        }
        return Ok(());
    }
    let (llm, _server) = match early { Some(x) => x, None => connect_or_spawn(&a).await? };
    let names: HashMap<String, Names> = std::fs::read("artifacts/goldgen/names.json")
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if let Some(batch) = &a.batch {
        let done: BTreeSet<String> = std::fs::read_to_string(&a.out)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str::<Answer>(l).ok()?.qid)
            .collect();
        let items: Vec<serde_json::Value> = std::fs::read_to_string(batch)
            .with_context(|| format!("reading {}", batch.display()))?
            .lines()
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        let mut out = std::fs::OpenOptions::new().create(true).append(true).open(&a.out)?;
        let todo: Vec<&serde_json::Value> =
            items.iter().filter(|i| !done.contains(i["qid"].as_str().unwrap_or(""))).collect();
        eprintln!("ask: {} to do, {} done, model {}", todo.len(), done.len(), llm.model);
        let started = std::time::Instant::now();
        for (n, it) in todo.iter().enumerate() {
            let q = it["question"].as_str().context("item without question")?;
            let mut ans = answer_one(&mut rts, &llm, &names, &chrono, q, &a, &mut tables).await?;
            ans.qid = it["qid"].as_str().map(str::to_owned);
            serde_json::to_writer(&mut out, &ans)?;
            out.write_all(b"\n")?;
            if (n + 1) % 10 == 0 {
                #[allow(clippy::cast_precision_loss)]
                let each = started.elapsed().as_secs_f64() / (n + 1) as f64;
                eprintln!("ask {}/{}, {each:.1} s each", n + 1, todo.len());
            }
        }
        return Ok(());
    }
    let q = a.question.clone().context("give a question, or --batch")?;
    // The single question was routed above; answer it from retrieval.
    let mut once = a.clone();
    once.routed = true;
    let ans = answer_one(&mut rts, &llm, &names, &chrono, &q, &once, &mut tables).await?;
    println!("{}\n", ans.answer);
    // Sources under the same [n] the answer uses.
    for (i, id) in ans.passages.iter().enumerate() {
        if ans.cited.contains(id) {
            println!("  [{}] {id}", i + 1);
        }
    }
    if ans.invalid_citations > 0 {
        println!("  ({} citation(s) pointed at no passage)", ans.invalid_citations);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_lookup_helpers_find_the_term_and_its_spelling_parts() {
        assert_eq!(term_of("What is the khagan quest?").as_deref(), Some("khagan quest"));
        assert_eq!(term_of("who are the sarkaz?").as_deref(), Some("sarkaz"));
        assert_eq!(term_of("What is Amiya's power?"), None, "a possessive is not a term question");
        assert_eq!(compact("Khagan quest"), compact("Khaganquest"));
        assert_eq!(compact("Kal'tsit"), "kaltsit");
        assert_eq!(count_word("Iris looked for Mabel. Iris'", "Iris"), 2);
        assert_eq!(count_word("Irisa and Iris", "Iris"), 1);
        assert_eq!(first_sentences("What it is\nA land of ice. It is cold. It is far.", 2), "A land of ice. It is cold.");
    }

    #[test]
    fn rule_triggers_fire_on_their_questions_only() {
        assert!(asks_origin("Why did swire join the LGD?") && asks_origin("When did the abyssal hunter project start?"));
        assert!(!asks_origin("Why is Kal'tsit so old?") && !asks_origin("What did Swire join?"));
        assert!(asks_relation("What is the relationship between Kristen and Friston?") && asks_relation("Which elite op is Lava a fan of / idolises?"));
        assert!(!asks_relation("Who is Ed?"));
        assert!(asks_when("When was Rhodes Island founded?") && asks_when("in what year did X die") && !asks_when("Who founded Rhodes Island?"));
        assert_eq!(typed_owner("Operator art: Pallas, E2 (Elite 2) art").as_deref(), Some("Pallas"));
        assert_eq!(typed_owner("Voice lines: Aroma").as_deref(), Some("Aroma"));
        assert_eq!(typed_owner("Module story: New Friends (Mutsumi Wakaba's module, PUM-Y)").as_deref(), Some("Mutsumi Wakaba"));
        assert!(asks_term("What is the khagan quest?") && asks_term("Who are the sarkaz?") && asks_term("What is sami guarding?"));
        assert!(!asks_term("What is Kal'tsit's real name?") && !asks_term("What is the name of the operator who visited Iris's castle of dreams?"));
        assert!(asks_identity("Which Rhodes Island Operator had visited Iris' castle of dreams?") && !asks_identity("Why did swire join the LGD?"));
        let mon3tr = ["mon3tr".to_owned()];
        assert!(asks_scene("During which event Mon3tr suggested assassinating a child", &mon3tr));
        assert!(asks_scene("What did Mon3tr say to the Doctor?", &mon3tr) && asks_scene("In which story Mon3tr ambushed a Sarkaz?", &mon3tr));
        assert!(!asks_scene("Why does Mon3tr's hat say KEE?", &mon3tr) && !asks_scene("What does the captain say about Mon3tr?", &mon3tr));
        assert!(!asks_scene("Is Mon3tr related to the Aggeloi?", &mon3tr));
    }

    #[test]
    fn chronology_lines_name_the_subject_and_one_word_names_widen_only_when_unique() {
        let c = Chrono { history: vec![(1086.0, "1086: After excavating the Rhodes Island landship, Babel sought an engineer.".into()),
                                       (1094.0, "1094 (scene dated): Babelite detonates explosives on Rhodes Island.".into()),
                                       (1097.0, "1097: Ch'en leaves Lungmen.".into())], ..Chrono::default() };
        let out = chronology_for("When was Rhodes Island founded?", &c);
        assert!(out.contains("RHODES ISLAND") && out.contains("1086") && out.contains("1094") && !out.contains("Lungmen"));
        assert!(chronology_for("when was it founded?", &c).is_empty());
        let te = time_evidence_block(&out);
        assert!(te.starts_with("\nTIME EVIDENCE") && te.contains("T1. 1086") && te.contains("T2. 1094") && !te.contains("T3."));
        let t = Tools::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        if t.attributes.is_some() {
            assert_eq!(one_operator_wide("who helped Mutsumi care for the plants?", &t).as_deref(), Some("Mutsumi Wakaba"));
            assert_eq!(one_operator("who helped Mutsumi care for the plants?", &t), None);
        }
    }
}
