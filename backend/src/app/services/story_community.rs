//! What the community has READ: how many accounts have opened each story, and
//! how far into each group they get.
//!
//! The aggregate is anonymous. Nothing on the wire names an account; every
//! field is a count over accounts, so the only privacy decision is which
//! accounts are counted at all. Two `user_settings` flags could govern it and
//! they do not name the same 2,630 minus N: `public_profile` gates the named
//! surfaces (a profile by uid, the leaderboards, user search) and `share_stats`
//! gates the anonymous ones (`operator_ownership_stats`, medal ownership). This
//! one takes the UNION of the two opt-outs, so an account that turned either
//! off is not counted, which is stricter than any existing aggregate and is
//! the conservative reading of an opt-out.
//!
//! Per account the read set has three sources, in the order the reader's own
//! library applies them. `user_game_story_read.read_in_game` is the whole
//! verdict when the account has any row, because that table is written by the
//! refresh from the game's own flags, its Archive block and its stage records
//! together. An account with no such row still has `user_stage_progress.stages`,
//! and the same gate rule `story_progress::stage_satisfies` applies there
//! recovers what the game played: a story whose every `RequiredStages` gate is
//! satisfied was played for that account. Then the reader's own document adds
//! its `read` keys and withdraws its `unread` keys, which is the only source
//! that can mark a story the game cannot know about and the only one that can
//! take a mark away.
//!
//! The build is CPU-bound over 43 MB of jsonb, so it runs on the blocking pool
//! in batches and is cached per server with a `computed_at`. Six hours is the
//! staleness ceiling: a background task computes it at boot and again on that
//! clock, and a request older than the ceiling recomputes under the slot lock.
//! That lock is `story::ServerCache`, the SAME single flight the library index
//! is built under, so a reader that arrives during a build waits for it rather
//! than starting a second one.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use ts_rs::TS;
use uuid::Uuid;

use crate::app::services::story::{ServerCache, StoryIndexCache, cached_index};
use crate::app::services::story_progress::stage_satisfies;
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::types::story_review::StoryRequiredStage;
use crate::core::hypergryph::constants::Server;

/// How long a computed aggregate is served before it is rebuilt.
pub const MAX_AGE: Duration = Duration::from_hours(6);

/// How many accounts one blocking hand-off folds.
///
/// The stage documents average 27 KB each, so a batch of 128 is about 3.4 MB
/// in flight and 21 hand-offs over the 2,585 counted accounts. It is a TRADE
/// and not a derived number: small enough that the resident set is bounded,
/// large enough that `spawn_blocking` overhead is noise against the parse.
const BATCH: usize = 128;

// ============================================================================
// The wire shape
// ============================================================================

/// One story and how many accounts have read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StoryReaders {
    pub id: String,
    pub readers: u32,
}

/// One group, its readers, its finishers and its depth curve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StoryGroupReaders {
    /// A `StoryGroup.id` (`main_8`, `act17side`) or, for an operator's
    /// records, that operator's `charId`.
    pub id: String,
    /// Accounts that have read at least one story in the group.
    pub readers: u32,
    /// Accounts that have read every GATED story in the group, the stories
    /// the stage records can speak for. A group with no gated story counts 0,
    /// and the tab prints that as not measurable rather than as nobody:
    /// 494 EN stories carry no gate (every operator record, 20 mini vignettes,
    /// `main_0`'s two Prologue guide entries), and 2,313 of 2,576 accounts are
    /// known only through their stage records, so "every scripted story"
    /// would have made chapter 0 finished by 0 of its 2,314 readers.
    pub finished: u32,
    /// Readers per story, in the group's own story order: the "how far people
    /// get" curve. It is the same number `stories[]` carries for each id,
    /// arranged so a consumer can draw the curve without a join.
    ///
    /// ABSENT on an operator's records, and only there. Carrying it for all
    /// 766 groups put the document at 153,950 bytes, over the 150 KB the wire
    /// contract allows; dropping it from the 315 record groups, which hold 367
    /// one-story-deep curves that say nothing their `readers` count does not,
    /// brings it to 150,066. That is 3,534 bytes under the ceiling and about 70
    /// more stories, which is a THIN margin: the real-data test asserts the
    /// ceiling, so the next EN batch that crosses it says so before the wire
    /// does, and the fix then is to drop the curve everywhere and let the
    /// consumer join it. A consumer that wants a record's curve reads
    /// `stories[]`, which is where every one of these numbers came from.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub depth: Option<Vec<u32>>,
}

/// The community reading aggregate for one server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StoryCommunity {
    /// Accounts counted: those that passed the privacy gate and carry at least
    /// one of the three sources. It is the denominator for every count here.
    pub players: u32,
    /// Unix seconds this aggregate was computed at.
    #[ts(type = "number")]
    pub computed_at: i64,
    /// Every story in the library index, in index order.
    pub stories: Vec<StoryReaders>,
    /// Every group in the library index, the operator record groups included,
    /// in index order.
    pub groups: Vec<StoryGroupReaders>,
}

// ============================================================================
// The plan: the library index reduced to what the fold needs
// ============================================================================

/// One group as the fold sees it: its stories as dense indices, and the subset
/// of them that has a script.
#[derive(Debug, Clone)]
struct PlanGroup {
    id: String,
    stories: Vec<u32>,
    /// The scripted stories that carry a gate: what `finished` is measured
    /// over. Empty on a group the stage records cannot speak for.
    measurable: Vec<u32>,
    /// Whether this group puts its depth curve on the wire. False on the
    /// operator record groups, which is the whole of the payload trim.
    curve: bool,
}

/// One group as [`CommunityPlan::new`] takes it: its id, its stories as
/// `(story id, has a script)` in the order the group lists them, and whether it
/// puts a depth curve on the wire.
pub type PlanGroupInput = (String, Vec<(String, bool)>, bool);

/// The library index reduced to indices, built once per compute.
///
/// Story ids are interned to a dense `u32` so the per-account fold is array
/// indexing rather than hashing: one hash lookup per id the sources name, and
/// none at all for the group pass.
#[derive(Debug, Clone)]
pub struct CommunityPlan {
    story_ids: Vec<String>,
    ix: HashMap<String, u32>,
    /// Only the stories that actually carry a gate. An empty `RequiredStages`
    /// is not a gate that passes, it is a story the stages cannot speak for,
    /// which is the same cut `story_progress::parse_game_story_read` makes.
    gated: Vec<(u32, Vec<StoryRequiredStage>)>,
    groups: Vec<PlanGroup>,
}

impl CommunityPlan {
    /// Build from the parts, so a test can hand-build a library.
    #[must_use]
    pub fn new(
        groups: Vec<PlanGroupInput>,
        gates: &HashMap<String, Vec<StoryRequiredStage>>,
    ) -> Self {
        let mut story_ids: Vec<String> = Vec::new();
        let mut ix: HashMap<String, u32> = HashMap::new();
        let mut plan_groups = Vec::with_capacity(groups.len());
        for (group_id, stories, curve) in groups {
            let mut all = Vec::with_capacity(stories.len());
            let mut scripted = Vec::new();
            for (story_id, has_script) in stories {
                let next = u32::try_from(story_ids.len()).unwrap_or(u32::MAX);
                let i = *ix.entry(story_id.clone()).or_insert_with(|| {
                    story_ids.push(story_id);
                    next
                });
                all.push(i);
                if has_script {
                    scripted.push(i);
                }
            }
            plan_groups.push(PlanGroup {
                id: group_id,
                stories: all,
                measurable: scripted,
                curve,
            });
        }
        let mut gated: Vec<(u32, Vec<StoryRequiredStage>)> = gates
            .iter()
            .filter(|(_, g)| !g.is_empty())
            .filter_map(|(id, g)| ix.get(id).map(|&i| (i, g.clone())))
            .collect();
        gated.sort_by_key(|(i, _)| *i);
        let has_gate: std::collections::HashSet<u32> = gated.iter().map(|(i, _)| *i).collect();
        for g in &mut plan_groups {
            g.measurable.retain(|i| has_gate.contains(i));
        }
        Self {
            story_ids,
            ix,
            gated,
            groups: plan_groups,
        }
    }

    /// Build from the warmed library index: every library group and then every
    /// operator's records, each keyed by the id the illustrations route takes.
    #[must_use]
    pub fn from_index(cache: &StoryIndexCache) -> Self {
        let mut groups: Vec<PlanGroupInput> =
            Vec::with_capacity(cache.index.groups.len() + cache.index.records.len());
        for g in &cache.index.groups {
            groups.push((
                g.id.clone(),
                g.stories
                    .iter()
                    .map(|s| (s.id.clone(), s.has_script))
                    .collect(),
                true,
            ));
        }
        for r in &cache.index.records {
            groups.push((
                r.char_id.clone(),
                r.stories
                    .iter()
                    .map(|s| (s.id.clone(), s.has_script))
                    .collect(),
                false,
            ));
        }
        Self::new(groups, &cache.gates)
    }

    #[must_use]
    pub const fn stories(&self) -> usize {
        self.story_ids.len()
    }

    #[must_use]
    pub const fn groups(&self) -> usize {
        self.groups.len()
    }
}

// ============================================================================
// The fold
// ============================================================================

/// One account's three sources, unparsed. The jsonb columns arrive as text so
/// the JSON parse lands on the blocking pool with the rest of the work rather
/// than on the async worker that reads the stream.
#[derive(Debug, Clone, Default)]
pub struct AccountRead {
    /// Story ids from `user_game_story_read` where `read_in_game`.
    pub verdict: Vec<String>,
    /// `user_stage_progress.stages` as stored.
    pub stages: Option<String>,
    /// `user_story_progress.progress` as stored.
    pub document: Option<String>,
}

impl AccountRead {
    const fn has_source(&self) -> bool {
        !self.verdict.is_empty() || self.stages.is_some() || self.document.is_some()
    }
}

/// What each source contributed, over accounts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommunityCensus {
    /// Accounts the privacy gate admitted and the query returned.
    pub scanned: u32,
    /// Of those, accounts carrying at least one source.
    pub players: u32,
    /// Accounts whose read set came from `user_game_story_read`.
    pub from_verdict: u32,
    /// Accounts with no verdict row whose stages satisfied at least one gate.
    pub from_gates: u32,
    /// Accounts whose reader document named at least one story the index knows.
    pub from_document: u32,
    /// Accounts whose read set came out empty.
    pub empty: u32,
    /// The sum of every account's read-set size.
    pub reads: u64,
}

/// The counters, plus the scratch the per-account pass reuses.
#[derive(Debug)]
pub struct Tally {
    story_readers: Vec<u32>,
    group_readers: Vec<u32>,
    group_finished: Vec<u32>,
    /// `seen[i] == epoch` means story `i` is in the current account's read set.
    /// Stamping beats clearing 1,887 flags per account, and `0` is never an
    /// epoch, so withdrawing a mark is a single store.
    seen: Vec<u32>,
    marks: Vec<u32>,
    epoch: u32,
    pub census: CommunityCensus,
}

impl Tally {
    #[must_use]
    pub fn new(plan: &CommunityPlan) -> Self {
        Self {
            story_readers: vec![0; plan.story_ids.len()],
            group_readers: vec![0; plan.groups.len()],
            group_finished: vec![0; plan.groups.len()],
            seen: vec![0; plan.story_ids.len()],
            marks: Vec::with_capacity(2_048),
            epoch: 0,
            census: CommunityCensus::default(),
        }
    }

    fn mark(&mut self, i: u32) {
        let i = i as usize;
        if self.seen[i] != self.epoch {
            self.seen[i] = self.epoch;
            self.marks.push(i as u32);
        }
    }

    /// Fold one account in.
    ///
    /// The three sources are applied in the order the reader's own library
    /// applies them, and the WITHDRAWAL is last: an `unread` key takes a mark
    /// away whichever source put it there, which is what makes a hand clear on
    /// a story the game thinks was played stick.
    pub fn add(&mut self, plan: &CommunityPlan, account: &AccountRead) {
        self.census.scanned += 1;
        if !account.has_source() {
            return;
        }
        self.census.players += 1;
        self.epoch += 1;
        self.marks.clear();

        if account.verdict.is_empty() {
            self.add_gates(plan, account.stages.as_deref());
        } else {
            for id in &account.verdict {
                if let Some(&i) = plan.ix.get(id) {
                    self.mark(i);
                }
            }
            self.census.from_verdict += 1;
        }
        self.add_document(plan, account.document.as_deref());

        let epoch = self.epoch;
        let mut read = 0_u64;
        for k in 0..self.marks.len() {
            let i = self.marks[k] as usize;
            if self.seen[i] == epoch {
                self.story_readers[i] += 1;
                read += 1;
            }
        }
        self.census.reads += read;
        if read == 0 {
            self.census.empty += 1;
        }

        for (gi, g) in plan.groups.iter().enumerate() {
            if g.stories.iter().any(|&i| self.seen[i as usize] == epoch) {
                self.group_readers[gi] += 1;
            }
            if !g.measurable.is_empty()
                && g.measurable.iter().all(|&i| self.seen[i as usize] == epoch)
            {
                self.group_finished[gi] += 1;
            }
        }
    }

    /// The gate verdict over one account's stage records.
    ///
    /// `stage_satisfies` takes a `dungeon.cowLevel` entry as its second
    /// argument and gets NONE here: the refresh copies a special story stage's
    /// first-open timestamp onto the stored record as `cowFirstTs`, and that
    /// is the field the rule reads when no cowLevel entry is passed. A store
    /// written before that copy loses those stages, which is the same caveat
    /// `story_progress::reverdict` carries.
    fn add_gates(&mut self, plan: &CommunityPlan, stages: Option<&str>) {
        let Some(raw) = stages else { return };
        let Ok(Value::Object(stages)) = serde_json::from_str::<Value>(raw) else {
            return;
        };
        let mut hits = 0_u32;
        for (i, gates) in &plan.gated {
            if gates
                .iter()
                .all(|g| stage_satisfies(stages.get(&g.stage_id), None, &g.min_state))
            {
                self.mark(*i);
                hits += 1;
            }
        }
        if hits > 0 {
            self.census.from_gates += 1;
        }
    }

    /// The reader's own marks and clears.
    fn add_document(&mut self, plan: &CommunityPlan, document: Option<&str>) {
        let Some(raw) = document else { return };
        let Ok(Value::Object(doc)) = serde_json::from_str::<Value>(raw) else {
            return;
        };
        let mut touched = 0_u32;
        if let Some(Value::Object(read)) = doc.get("read") {
            for id in read.keys() {
                if let Some(&i) = plan.ix.get(id) {
                    self.mark(i);
                    touched += 1;
                }
            }
        }
        if let Some(Value::Object(unread)) = doc.get("unread") {
            for id in unread.keys() {
                if let Some(&i) = plan.ix.get(id) {
                    self.seen[i as usize] = 0;
                    touched += 1;
                }
            }
        }
        if touched > 0 {
            self.census.from_document += 1;
        }
    }

    /// Fold another tally's counters in, so the batches can be summed.
    pub fn merge(&mut self, other: &Self) {
        for (a, b) in self.story_readers.iter_mut().zip(&other.story_readers) {
            *a += *b;
        }
        for (a, b) in self.group_readers.iter_mut().zip(&other.group_readers) {
            *a += *b;
        }
        for (a, b) in self.group_finished.iter_mut().zip(&other.group_finished) {
            *a += *b;
        }
        self.census.scanned += other.census.scanned;
        self.census.players += other.census.players;
        self.census.from_verdict += other.census.from_verdict;
        self.census.from_gates += other.census.from_gates;
        self.census.from_document += other.census.from_document;
        self.census.empty += other.census.empty;
        self.census.reads += other.census.reads;
    }

    /// The wire shape. `depth` is read off the per-story counters, so a group's
    /// curve and the story rows can never disagree.
    #[must_use]
    pub fn finish(&self, plan: &CommunityPlan, computed_at: i64) -> StoryCommunity {
        StoryCommunity {
            players: self.census.players,
            computed_at,
            stories: plan
                .story_ids
                .iter()
                .enumerate()
                .map(|(i, id)| StoryReaders {
                    id: id.clone(),
                    readers: self.story_readers[i],
                })
                .collect(),
            groups: plan
                .groups
                .iter()
                .enumerate()
                .map(|(gi, g)| StoryGroupReaders {
                    id: g.id.clone(),
                    readers: self.group_readers[gi],
                    finished: self.group_finished[gi],
                    depth: g.curve.then(|| {
                        g.stories
                            .iter()
                            .map(|&i| self.story_readers[i as usize])
                            .collect()
                    }),
                })
                .collect(),
        }
    }
}

// ============================================================================
// The compute
// ============================================================================

/// The privacy gate, as one SQL predicate.
///
/// Both flags default to true and both are an opt-out, so an account is
/// counted only while it has turned neither off.
const VISIBLE: &str = "s.public_profile AND s.share_stats";

/// Compute the aggregate for one server, end to end.
///
/// The account stream holds one connection for the whole walk and the fold
/// runs on the blocking pool a batch at a time, so no JSON parse and no gate
/// pass ever lands on an async worker. The verdict rows are fetched per batch
/// on a second connection, and only for the accounts a cheap `DISTINCT` says
/// have any: today that is 1 account of 2,585, and the shape still holds when
/// it is all of them.
///
/// Scale: 2,585 admitted accounts times 43 MB of jsonb, folded in 21 hand-offs
/// of [`BATCH`], against a plan of 1,887 interned story ids and 766 groups. The
/// per-account pass is O(marks + groups) with the story ids already dense, so
/// nothing here is quadratic in the library; the cost is the JSON parse, and
/// that is why every batch goes through `cpu::offload`.
pub async fn compute(
    state: &AppState,
    server: Server,
) -> Result<(StoryCommunity, CommunityCensus, u128), ApiError> {
    let started = Instant::now();
    let index = cached_index(state, server).await?;
    let plan = Arc::new(
        cpu::offload("story_community_plan", move || {
            CommunityPlan::from_index(&index)
        })
        .await?,
    );

    let with_verdict: std::collections::HashSet<Uuid> = sqlx::query(
        "SELECT DISTINCT user_id FROM user_game_story_read WHERE read_in_game AND user_id IS NOT NULL",
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|r| r.get::<Uuid, _>("user_id"))
    .collect();

    let mut total = Tally::new(&plan);
    let mut conn = state.db.acquire().await?;
    let sql = format!(
        "SELECT u.id AS id, sp.stages::text AS stages, gp.progress::text AS progress \
         FROM users u \
         JOIN user_settings s ON s.user_id = u.id \
         LEFT JOIN user_stage_progress sp ON sp.user_id = u.id \
         LEFT JOIN user_story_progress gp ON gp.user_id = u.id \
         WHERE {VISIBLE} ORDER BY u.id"
    );
    let mut stream = sqlx::query(&sql).fetch(&mut *conn);
    let mut batch: Vec<(Uuid, AccountRead)> = Vec::with_capacity(BATCH);
    loop {
        let row = stream.try_next().await?;
        if let Some(row) = row {
            batch.push((
                row.get::<Uuid, _>("id"),
                AccountRead {
                    verdict: Vec::new(),
                    stages: row.get::<Option<String>, _>("stages"),
                    document: row.get::<Option<String>, _>("progress"),
                },
            ));
            if batch.len() < BATCH {
                continue;
            }
        } else if batch.is_empty() {
            break;
        }

        let wanted: Vec<Uuid> = batch
            .iter()
            .map(|(id, _)| *id)
            .filter(|id| with_verdict.contains(id))
            .collect();
        if !wanted.is_empty() {
            let rows = sqlx::query(
                "SELECT user_id, story_id FROM user_game_story_read \
                 WHERE read_in_game AND user_id = ANY($1)",
            )
            .bind(&wanted)
            .fetch_all(&state.db)
            .await?;
            let mut by_user: HashMap<Uuid, Vec<String>> = HashMap::new();
            for r in rows {
                by_user
                    .entry(r.get::<Uuid, _>("user_id"))
                    .or_default()
                    .push(r.get::<String, _>("story_id"));
            }
            for (id, account) in &mut batch {
                if let Some(ids) = by_user.remove(id) {
                    account.verdict = ids;
                }
            }
        }

        let plan_for_batch = Arc::clone(&plan);
        let taken = std::mem::take(&mut batch);
        let folded = cpu::offload("story_community_batch", move || {
            let mut tally = Tally::new(&plan_for_batch);
            for (_, account) in &taken {
                tally.add(&plan_for_batch, account);
            }
            tally
        })
        .await?;
        total.merge(&folded);
        batch = Vec::with_capacity(BATCH);
    }
    drop(stream);
    drop(conn);

    let computed_at = chrono::Utc::now().timestamp();
    let out = total.finish(&plan, computed_at);
    Ok((out, total.census, started.elapsed().as_millis()))
}

// ============================================================================
// The cache
// ============================================================================

/// The aggregate's own slot, on the same [`ServerCache`] the library index
/// uses. It differs from the index's only in that a value also ages out: the
/// library is retired by a hot reload alone, the aggregate by the clock too.
static AGGREGATE: ServerCache<StoryCommunity> = ServerCache::new();

/// How many times the aggregate has actually been COMPUTED in this process.
/// Exported so a test can assert the single flight.
#[must_use]
pub fn computes() -> u64 {
    AGGREGATE.builds()
}

/// The aggregate for one server, computed at most every [`MAX_AGE`].
///
/// The slot's lock is held across the compute, so a second caller that arrives
/// mid-build WAITS for it and then reads the fresh value. Computing inline on
/// the request path is the fallback the warm task exists to avoid; it happens
/// once per staleness window at worst.
pub async fn cached(state: &AppState, server: Server) -> Result<Arc<StoryCommunity>, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    let now = chrono::Utc::now().timestamp();
    let ceiling = i64::try_from(MAX_AGE.as_secs()).unwrap_or(i64::MAX);
    AGGREGATE
        .get_or_build(
            server,
            gd,
            |hit| now.saturating_sub(hit.computed_at) < ceiling,
            || async {
                let (built, census, ms) = compute(state, server).await?;
                tracing::info!(
                    server = ?server,
                    players = census.players,
                    scanned = census.scanned,
                    from_verdict = census.from_verdict,
                    from_gates = census.from_gates,
                    from_document = census.from_document,
                    empty = census.empty,
                    reads = census.reads,
                    stories = built.stories.len(),
                    groups = built.groups.len(),
                    compute_ms = ms,
                    "story community aggregate computed"
                );
                Ok(Arc::new(built))
            },
        )
        .await
}

/// Compute the aggregate OFF the request path: once at boot, then on the
/// staleness clock, so no reader ever pays the walk. A failure is logged and
/// the loop keeps its clock, because the next window may well succeed and the
/// lazy path in [`cached`] stays the fallback either way.
pub fn spawn_warm(state: AppState, server: Server) {
    tokio::spawn(async move {
        loop {
            let started = Instant::now();
            match cached(&state, server).await {
                Ok(built) => tracing::info!(
                    server = ?server,
                    warm_ms = started.elapsed().as_millis(),
                    players = built.players,
                    "story community aggregate warmed"
                ),
                Err(e) => tracing::warn!(
                    server = ?server,
                    error = %e,
                    "story community warm failed; the next reader will compute it"
                ),
            }
            tokio::time::sleep(MAX_AGE).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{AccountRead, CommunityPlan, Tally};
    use crate::core::gamedata::types::story_review::StoryRequiredStage;
    use std::collections::HashMap;

    fn gate(stage: &str, min_state: &str) -> Vec<StoryRequiredStage> {
        vec![StoryRequiredStage {
            stage_id: stage.into(),
            min_state: min_state.into(),
            max_state: String::new(),
        }]
    }

    /// Two chapters: `main_0` is three scripted stories behind three gates,
    /// `rec_a` is one ungated record.
    fn plan() -> CommunityPlan {
        let mut gates = HashMap::new();
        gates.insert("s1".to_owned(), gate("main_00-01", "PLAYED"));
        gates.insert("s2".to_owned(), gate("main_00-02", "PLAYED"));
        gates.insert("s3".to_owned(), gate("main_00-03", "PASS"));
        gates.insert("r1".to_owned(), Vec::new());
        CommunityPlan::new(
            vec![
                (
                    "main_0".to_owned(),
                    vec![
                        ("s1".to_owned(), true),
                        ("s2".to_owned(), true),
                        ("s3".to_owned(), true),
                    ],
                    true,
                ),
                (
                    "rec_a".to_owned(),
                    vec![("r1".to_owned(), true)],
                    // The records carry no curve on the wire, so the fixture
                    // pins the empty one the trim produces.
                    false,
                ),
            ],
            &gates,
        )
    }

    /// Three accounts, one per source, against one hand-built library.
    ///
    /// The verdict account read the whole chapter and the record; the
    /// gate-only account cleared the first two stages and left the third
    /// locked; the document account cleared its first stage but hand-marked
    /// the record read and hand-cleared `s1`. So `s1` is read by two of three,
    /// `s2` by two, `s3` by one, `r1` by two, and only the first account
    /// FINISHED `main_0`.
    #[test]
    fn three_accounts_one_per_source() {
        let plan = plan();
        let mut tally = Tally::new(&plan);

        tally.add(
            &plan,
            &AccountRead {
                verdict: vec![
                    "s1".to_owned(),
                    "s2".to_owned(),
                    "s3".to_owned(),
                    "r1".to_owned(),
                ],
                stages: Some(r#"{"main_00-01":{"state":3}}"#.to_owned()),
                document: None,
            },
        );
        tally.add(
            &plan,
            &AccountRead {
                verdict: Vec::new(),
                stages: Some(
                    r#"{"main_00-01":{"state":3,"startTimes":2,"completeTimes":1},
                        "main_00-02":{"state":2,"startTimes":1,"completeTimes":0},
                        "main_00-03":{"state":0,"startTimes":0,"completeTimes":0}}"#
                        .to_owned(),
                ),
                document: None,
            },
        );
        tally.add(
            &plan,
            &AccountRead {
                verdict: Vec::new(),
                stages: Some(r#"{"main_00-01":{"state":3,"startTimes":1}}"#.to_owned()),
                document: Some(r#"{"v":2,"read":{"r1":1},"unread":{"s1":1},"pos":{}}"#.to_owned()),
            },
        );

        let out = tally.finish(&plan, 1_700_000_000);
        assert_eq!(out.players, 3);
        assert_eq!(out.computed_at, 1_700_000_000);

        let readers: HashMap<&str, u32> = out
            .stories
            .iter()
            .map(|s| (s.id.as_str(), s.readers))
            .collect();
        assert_eq!(readers["s1"], 2, "the third account hand-cleared s1");
        assert_eq!(readers["s2"], 2);
        assert_eq!(readers["s3"], 1);
        assert_eq!(readers["r1"], 2, "the gates cannot reach an ungated record");

        let main = &out.groups[0];
        assert_eq!(main.id, "main_0");
        assert_eq!(main.readers, 2, "the third account holds no main_0 mark");
        assert_eq!(main.finished, 1);
        assert_eq!(
            main.depth.as_deref(),
            Some(&[2, 2, 1][..]),
            "the depth curve falls away"
        );

        let rec = &out.groups[1];
        assert_eq!(rec.id, "rec_a");
        assert_eq!(rec.readers, 2);
        // An ungated record is never finished: nothing can measure it.
        assert_eq!(rec.finished, 0);
        assert!(rec.depth.is_none(), "a record group carries no curve");

        assert_eq!(tally.census.from_verdict, 1);
        assert_eq!(tally.census.from_gates, 2);
        assert_eq!(tally.census.from_document, 1);
        assert_eq!(tally.census.empty, 0);
        assert_eq!(tally.census.reads, 4 + 2 + 1);
    }

    /// An account with nothing stored is not a player, and a batch merge is
    /// the same arithmetic as folding the accounts into one tally.
    #[test]
    fn an_empty_account_is_not_counted_and_batches_sum() {
        let plan = plan();
        let mut tally = Tally::new(&plan);
        tally.add(&plan, &AccountRead::default());
        assert_eq!(tally.census.scanned, 1);
        assert_eq!(tally.census.players, 0);

        let one = AccountRead {
            verdict: vec!["s1".to_owned()],
            stages: None,
            document: None,
        };
        let mut a = Tally::new(&plan);
        a.add(&plan, &one);
        let mut b = Tally::new(&plan);
        b.add(&plan, &one);
        a.merge(&b);
        let out = a.finish(&plan, 0);
        assert_eq!(out.stories[0].readers, 2);
        assert_eq!(out.groups[0].readers, 2);
        assert_eq!(out.groups[0].finished, 0);
        assert_eq!(out.groups[0].depth.as_deref(), Some(&[2, 0, 0][..]));
        assert_eq!(a.census.players, 2);
    }
}
