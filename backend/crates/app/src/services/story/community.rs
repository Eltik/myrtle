use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use myrtle_db::repos::story as repo;
use myrtle_engine_story::StoryLibrary;
use myrtle_engine_story::verdict::stage_satisfies;
use myrtle_gamedata_raw::tables::story::StoryRequiredStage;
use myrtle_types::Server;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::cache::CachedJson;
use crate::{AppContext, AppError};

pub const MAX_AGE: Duration = Duration::from_hours(6);

const BATCH: usize = 128;

#[must_use]
pub fn cache_key(server: Server, res: &str) -> String {
    format!("story:community:{}:{res}", server.data_server().as_str())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryReaders {
    pub id: String,
    pub readers: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryGroupReaders {
    pub id: String,
    pub readers: u32,
    pub finished: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryCommunity {
    pub players: u32,
    pub computed_at: i64,
    pub stories: Vec<StoryReaders>,
    pub groups: Vec<StoryGroupReaders>,
}

#[derive(Debug, Clone)]
struct PlanGroup {
    id: String,
    stories: Vec<u32>,
    measurable: Vec<u32>,
    curve: bool,
}

pub type PlanGroupInput = (String, Vec<(String, bool)>, bool);

#[derive(Debug, Clone)]
pub struct CommunityPlan {
    story_ids: Vec<String>,
    ix: HashMap<String, u32>,
    gated: Vec<(u32, Vec<StoryRequiredStage>)>,
    groups: Vec<PlanGroup>,
}

impl CommunityPlan {
    #[must_use]
    pub fn new(
        groups: Vec<PlanGroupInput>,
        gates: &HashMap<String, Vec<StoryRequiredStage>>,
    ) -> Self {
        let mut story_ids: Vec<String> = Vec::new();
        let mut ix: HashMap<String, u32> = HashMap::new();
        let mut plan_groups = Vec::with_capacity(groups.len());
        for (id, stories, curve) in groups {
            let mut all = Vec::with_capacity(stories.len());
            let mut measurable = Vec::new();
            for (story_id, has_script) in stories {
                let next = u32::try_from(story_ids.len()).unwrap_or(u32::MAX);
                let i = *ix.entry(story_id.clone()).or_insert_with(|| {
                    story_ids.push(story_id);
                    next
                });
                all.push(i);
                if has_script {
                    measurable.push(i);
                }
            }
            plan_groups.push(PlanGroup {
                id,
                stories: all,
                measurable,
                curve,
            });
        }
        let mut gated: Vec<(u32, Vec<StoryRequiredStage>)> = gates
            .iter()
            .filter(|(_, g)| !g.is_empty())
            .filter_map(|(id, g)| ix.get(id).map(|&i| (i, g.clone())))
            .collect();
        gated.sort_by_key(|(i, _)| *i);
        let has_gate: HashSet<u32> = gated.iter().map(|(i, _)| *i).collect();
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

    #[must_use]
    pub fn from_library(library: &StoryLibrary) -> Self {
        let stories = |entries: &[myrtle_engine_story::dto::StoryEntry]| {
            entries
                .iter()
                .map(|s| (s.id.clone(), s.has_script))
                .collect()
        };
        let index = &library.index;
        let groups = index
            .groups
            .iter()
            .map(|g| (g.id.clone(), stories(&g.stories), true))
            .chain(
                index
                    .records
                    .iter()
                    .map(|r| (r.char_id.clone(), stories(&r.stories), false)),
            )
            .collect();
        Self::new(groups, &library.gates)
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

#[derive(Debug, Clone, Default)]
pub struct AccountRead {
    pub verdict: Vec<String>,
    pub stages: Option<String>,
    pub document: Option<String>,
}

impl AccountRead {
    const fn has_source(&self) -> bool {
        !self.verdict.is_empty() || self.stages.is_some() || self.document.is_some()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommunityCensus {
    pub scanned: u32,
    pub players: u32,
    pub from_verdict: u32,
    pub from_gates: u32,
    pub from_document: u32,
    pub empty: u32,
    pub reads: u64,
}

#[derive(Debug)]
pub struct Tally {
    story_readers: Vec<u32>,
    group_readers: Vec<u32>,
    group_finished: Vec<u32>,
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
        let at = i as usize;
        if self.seen[at] != self.epoch {
            self.seen[at] = self.epoch;
            self.marks.push(i);
        }
    }

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
        for &i in &self.marks {
            if self.seen[i as usize] == epoch {
                self.story_readers[i as usize] += 1;
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

    pub fn merge(&mut self, other: &Self) {
        for (a, b) in [
            (&mut self.story_readers, &other.story_readers),
            (&mut self.group_readers, &other.group_readers),
            (&mut self.group_finished, &other.group_finished),
        ] {
            for (x, y) in a.iter_mut().zip(b) {
                *x += *y;
            }
        }
        let (c, o) = (&mut self.census, &other.census);
        c.scanned += o.scanned;
        c.players += o.players;
        c.from_verdict += o.from_verdict;
        c.from_gates += o.from_gates;
        c.from_document += o.from_document;
        c.empty += o.empty;
        c.reads += o.reads;
    }

    #[must_use]
    pub fn finish(&self, plan: &CommunityPlan, computed_at: i64) -> StoryCommunity {
        StoryCommunity {
            players: self.census.players,
            computed_at,
            stories: plan
                .story_ids
                .iter()
                .zip(&self.story_readers)
                .map(|(id, &readers)| StoryReaders {
                    id: id.clone(),
                    readers,
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

pub async fn compute(
    ctx: &AppContext,
    server: Server,
) -> Result<(StoryCommunity, CommunityCensus), AppError> {
    let snapshot = super::snapshot(ctx, server).await?;
    let plan = Arc::new(
        ctx.cpu
            .offload("story_community_plan", move || {
                snapshot
                    .story()
                    .map(|s| CommunityPlan::from_library(&s.library))
            })
            .await??,
    );
    let mut conn = ctx.db.conn().await?;
    let with_verdict: HashSet<Uuid> = repo::accounts_with_game_reads(&mut conn)
        .await?
        .into_iter()
        .collect();
    let mut total = Tally::new(&plan);
    let mut after = None;
    loop {
        let page = repo::community_accounts(&mut conn, after, BATCH as i64).await?;
        let Some(last) = page.last() else { break };
        after = Some(last.id);
        let full = page.len() == BATCH;
        let wanted: Vec<Uuid> = page
            .iter()
            .map(|r| r.id)
            .filter(|id| with_verdict.contains(id))
            .collect();
        let mut by_user: HashMap<Uuid, Vec<String>> = HashMap::new();
        if !wanted.is_empty() {
            for row in repo::game_reads_of(&mut conn, &wanted).await? {
                by_user.entry(row.user_id).or_default().push(row.story_id);
            }
        }
        let batch: Vec<AccountRead> = page
            .into_iter()
            .map(|r| AccountRead {
                verdict: by_user.remove(&r.id).unwrap_or_default(),
                stages: r.stages,
                document: r.progress,
            })
            .collect();
        let plan = Arc::clone(&plan);
        let folded = ctx
            .cpu
            .offload("story_community_batch", move || {
                let mut tally = Tally::new(&plan);
                for account in &batch {
                    tally.add(&plan, account);
                }
                tally
            })
            .await?;
        total.merge(&folded);
        if !full {
            break;
        }
    }
    let out = total.finish(&plan, chrono::Utc::now().timestamp());
    Ok((out, total.census))
}

async fn build_body(ctx: &AppContext, server: Server) -> Result<String, AppError> {
    let started = std::time::Instant::now();
    let (built, census) = compute(ctx, server).await?;
    tracing::info!(
        server = server.as_str(),
        players = census.players,
        scanned = census.scanned,
        from_verdict = census.from_verdict,
        from_gates = census.from_gates,
        from_document = census.from_document,
        empty = census.empty,
        reads = census.reads,
        stories = built.stories.len(),
        groups = built.groups.len(),
        compute_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        "story community aggregate computed"
    );
    serde_json::to_string(&built).map_err(AppError::internal)
}

pub async fn community(ctx: &AppContext, server: Server) -> Result<CachedJson, AppError> {
    let res = ctx
        .snapshots
        .require(server)?
        .model
        .meta
        .game_version
        .res
        .clone();
    let owner = ctx.clone();
    ctx.cache
        .json_detached(&cache_key(server, &res), MAX_AGE, move || async move {
            build_body(&owner, server).await
        })
        .await
}

pub async fn aggregate(ctx: &AppContext, server: Server) -> Result<(), AppError> {
    let res = ctx
        .snapshots
        .require(server)?
        .model
        .meta
        .game_version
        .res
        .clone();
    let body = build_body(ctx, server).await?;
    ctx.cache
        .set_json(&cache_key(server, &res), &CachedJson::new(body), MAX_AGE)
        .await;
    Ok(())
}
