//! One seat resolved against the game data: the chosen skill at its rank,
//! the talents live at the build's promotion, level and potential, and the
//! chosen module's trait and talent changes merged in. Every panel reads a
//! `Seat`, so the build rules live here once.

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::module::{
    ModuleBlackboard, ModulePhase, ModuleTarget, ModuleType,
};
use crate::core::gamedata::types::operator::{
    AttributeData, Blackboard, Operator, OperatorModule, UnlockCondition,
};
use crate::core::gamedata::types::skill::{SkillBlackboard, SkillLevel};

use super::index::TextKey;
use super::rules::{SkillRules, module_ranks, reached, unlock_of};
use super::types::{SUPPORT_SEAT, SourceKind, SourceRef, SquadMemberBuild};

/// A blackboard flattened to `(key, value)`, keys lowercased so a rule can
/// name `hp_recovery_per_sec_by_max_hp_ratio` whatever the table's casing.
pub type Board = Vec<(String, f64)>;

/// One blackboard entry of any table's shape.
trait BoardEntry {
    fn entry(&self) -> (&str, f64);
}

impl BoardEntry for Blackboard {
    fn entry(&self) -> (&str, f64) {
        (&self.key, self.value)
    }
}

impl BoardEntry for SkillBlackboard {
    fn entry(&self) -> (&str, f64) {
        (&self.key, self.value)
    }
}

impl BoardEntry for ModuleBlackboard {
    fn entry(&self) -> (&str, f64) {
        (&self.key, self.value)
    }
}

/// A table's blackboard as a `Board`, keys lowercased.
fn board<T: BoardEntry>(entries: &[T]) -> Board {
    entries
        .iter()
        .map(|e| {
            let (k, v) = e.entry();
            (k.to_lowercase(), v)
        })
        .collect()
}

/// The value of `key` on `board`, if present.
pub fn bb(board: &Board, key: &str) -> Option<f64> {
    board.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

/// Merge `over` into `base`, `over` winning on a shared key.
fn merge(base: &mut Board, over: Board) {
    for (k, v) in over {
        if let Some(slot) = base.iter_mut().find(|(bk, _)| *bk == k) {
            slot.1 = v;
        } else {
            base.push((k, v));
        }
    }
}

/// A talent as the build has it.
#[derive(Debug, Clone)]
pub struct LiveTalent {
    /// Index into `Operator::talents` (the module part's `talent_index`).
    pub index: usize,
    pub name: String,
    /// Raw rich text, tags kept.
    pub description: String,
    pub board: Board,
    /// `TalentCandidate::token_key`: the talent places a summon.
    pub token_key: Option<String>,
    /// The module name when a module part replaced this talent's candidate.
    pub module: Option<String>,
    /// Where `description` sits in the tables, for the English text rules.
    pub key: TextKey,
}

impl LiveTalent {
    /// The talent as a fact's source.
    pub fn source(&self) -> SourceRef {
        SourceRef {
            kind: SourceKind::Talent,
            name: Some(self.name.clone()),
            skill_index: None,
        }
    }
}

/// The chosen skill at the chosen rank.
#[derive(Debug, Clone)]
pub struct LiveSkill<'a> {
    pub index: usize,
    pub level: &'a SkillLevel,
    pub board: Board,
    /// Where the rank's description sits, for the English text rules.
    pub key: TextKey,
}

impl LiveSkill<'_> {
    /// The skill as a fact's source.
    pub fn source(&self) -> SourceRef {
        SourceRef {
            kind: SourceKind::Skill,
            name: Some(self.level.name.clone()),
            skill_index: i32::try_from(self.index).ok(),
        }
    }
}

/// A filled seat.
#[derive(Debug, Clone)]
pub struct Seat<'a> {
    pub seat: i32,
    pub support: bool,
    pub char_id: String,
    pub op: &'a Operator,
    /// The request's build with every number clamped to what the operator has.
    pub build: SquadMemberBuild,
    pub phase: usize,
    pub skill: Option<LiveSkill<'a>>,
    pub talents: Vec<LiveTalent>,
    /// The trait text (`description`, or a candidate's / module's override,
    /// plus a module's additional line), raw.
    pub trait_text: String,
    /// The live trait candidate's blackboard with the module trait part merged.
    pub trait_board: Board,
    /// The live trait candidate's own blackboard, before any module: the
    /// branch's archetype, which a module's added keys (a Crusher module's
    /// self-heal `heal_scale`) must not change.
    pub branch_board: Board,
    /// The pieces `trait_text` is made of, in order, for the text rules.
    pub trait_keys: Vec<TextKey>,
    pub module: Option<&'a OperatorModule>,
    pub module_phase: Option<&'a ModulePhase>,
    /// The operator's recruitment tags as `GachaTags` ids on its server.
    pub tag_ids: Vec<i32>,
    pub rules: SkillRules,
}

impl Seat<'_> {
    /// The live key frame data of the build's promotion (constant within a
    /// promotion for cost, block and redeploy, so key frame 0 is enough).
    pub fn key_frame(&self) -> Option<&AttributeData> {
        self.op
            .phases
            .get(self.phase)
            .and_then(|p| p.attributes_key_frames.first())
            .map(|k| &k.data)
    }

    pub fn module_name(&self) -> Option<String> {
        self.module.map(|m| m.module.uni_equip_name.clone())
    }
}

/// `"PHASE_2"` (module tables keep the phase as text) to 2.
pub fn phase_text_num(p: &str) -> i32 {
    p.strip_prefix("PHASE_")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// Whether a talent or trait candidate is live at the build: its unlock
/// condition met and its `required_potential_rank` within the potential.
const fn live(cond: &UnlockCondition, required_potential: i32, b: &SquadMemberBuild) -> bool {
    reached(&unlock_of(cond), b.phase, b.level) && required_potential <= b.potential
}

/// The operator's module `id`, if it has one.
fn find_module<'a>(op: &'a Operator, id: &str) -> Option<&'a OperatorModule> {
    op.modules.iter().find(|m| m.module.uni_equip_id == id)
}

/// The request's build clamped to what the operator has: its promotions and
/// their level caps (`Phases[p].MaxLevel`), its potential ranks, its skills,
/// its top skill rank and the chosen skill's masteries (`rules::SkillRules`),
/// and the module's ranks. What the build has UNLOCKED is not clamped here;
/// `readiness` flags it.
fn clamp_build(op: &Operator, rules: &SkillRules, b: &SquadMemberBuild) -> SquadMemberBuild {
    let max_phase = op.phases.len().saturating_sub(1);
    let phase = usize::try_from(b.phase.max(0)).unwrap_or(0).min(max_phase);
    let level_cap = op.phases.get(phase).map_or(1, |p| p.max_level.max(1));
    let skill_max = op.skills.len().saturating_sub(1);
    let max_pot = i32::try_from(op.potential_ranks.len()).unwrap_or(0);
    let skill_index = b
        .skill_index
        .clamp(0, i32::try_from(skill_max).unwrap_or(0));
    let max_rank = rules.max_rank();
    let skill_level = b.skill_level.clamp(1, max_rank);
    let max_mastery = rules.mastery_cap(usize::try_from(skill_index).unwrap_or(0), skill_level);
    let module_max = b
        .module_id
        .as_deref()
        .and_then(|id| find_module(op, id))
        .map_or(1, |m| module_ranks(m).max(1));
    SquadMemberBuild {
        char_id: b.char_id.clone(),
        phase: i32::try_from(phase).unwrap_or(0),
        level: b.level.clamp(1, level_cap),
        potential: b.potential.clamp(0, max_pot),
        skill_index,
        skill_level,
        mastery: b.mastery.clamp(0, max_mastery),
        module_id: b.module_id.clone(),
        module_level: b.module_level.clamp(1, module_max),
    }
}

/// The live talents: for each talent, the last candidate whose unlock
/// condition is met and whose `required_potential_rank` is within the build's
/// potential (candidates are listed in unlock then potential order).
fn live_talents(char_id: &str, op: &Operator, b: &SquadMemberBuild) -> Vec<LiveTalent> {
    let mut out = Vec::new();
    for (index, t) in op.talents.iter().enumerate() {
        let pick = t
            .candidates
            .iter()
            .enumerate()
            .rfind(|(_, c)| live(&c.unlock_condition, c.required_potential_rank, b));
        let Some((cand, c)) = pick else { continue };
        if c.is_hide_talent == Some(true) {
            continue;
        }
        out.push(LiveTalent {
            index,
            name: c.name.clone().unwrap_or_default(),
            description: c.description.clone().unwrap_or_default(),
            board: board(&c.blackboard),
            token_key: c.token_key.clone().filter(|k| !k.is_empty()),
            module: None,
            key: TextKey::Talent {
                char_id: char_id.to_string(),
                talent: index,
                cand,
            },
        });
    }
    out
}

/// Resolve one seat. `None` for an unknown `char_id`.
pub fn resolve<'a>(data: &'a GameData, seat: i32, b: &SquadMemberBuild) -> Option<Seat<'a>> {
    let op = data.operators.get(&b.char_id)?;
    let rules = SkillRules::of(op);
    let build = clamp_build(op, &rules, b);
    let phase = usize::try_from(build.phase).unwrap_or(0);

    let skill = op
        .skills
        .get(usize::try_from(build.skill_index).unwrap_or(0))
        .and_then(|s| {
            let levels = &s.static_data.as_ref()?.levels;
            if levels.is_empty() {
                return None;
            }
            // Rank r maps to levels[r - 1], mastery m on top of the top rank
            // to levels[top - 1 + m]; `clamp_build` keeps both in range, and
            // the index is clamped to the levels the table has.
            let mastery = if build.skill_level == rules.max_rank() {
                build.mastery
            } else {
                0
            };
            let idx = usize::try_from(build.skill_level - 1 + mastery)
                .unwrap_or(0)
                .min(levels.len() - 1);
            let level = &levels[idx];
            Some(LiveSkill {
                index: usize::try_from(build.skill_index).unwrap_or(0),
                key: TextKey::Skill {
                    skill_id: s.skill_id.clone(),
                    level: idx,
                },
                level,
                board: board(&level.blackboard),
            })
        });

    // The trait: the last candidate unlocked at this build.
    let mut trait_text = op.description.clone();
    let mut trait_keys = vec![TextKey::Trait {
        char_id: b.char_id.clone(),
    }];
    let mut trait_board: Board = Vec::new();
    if let Some(t) = &op.trait_data
        && let Some((cand, c)) = t
            .candidates
            .iter()
            .enumerate()
            .rfind(|(_, c)| live(&c.unlock_condition, c.required_potential_rank, &build))
    {
        trait_board = board(&c.blackboard);
        if let Some(d) = c.override_description.as_ref().filter(|d| !d.is_empty()) {
            trait_text.clone_from(d);
            trait_keys = vec![TextKey::TraitCandidate {
                char_id: b.char_id.clone(),
                cand,
            }];
        }
    }

    let branch_board = trait_board.clone();
    let mut talents = live_talents(&b.char_id, op, &build);

    let module = build
        .module_id
        .as_deref()
        .and_then(|id| find_module(op, id));
    let module_phase_index = module.map(|m| {
        usize::try_from(build.module_level - 1)
            .unwrap_or(0)
            .min(m.data.phases.len().saturating_sub(1))
    });
    let module_phase = module
        .zip(module_phase_index)
        .and_then(|(m, i)| m.data.phases.get(i));
    if let (Some(m), Some(mp), Some(phase_i)) = (module, module_phase, module_phase_index) {
        let mname = m.module.uni_equip_name.clone();
        let module_id = &m.module.uni_equip_id;
        for (part_i, part) in mp.parts.iter().enumerate() {
            match part.target {
                ModuleTarget::Talent | ModuleTarget::TalentDataOnly => {
                    let Some(cands) = &part.add_or_override_talent_data_bundle.candidates else {
                        continue;
                    };
                    let Some((cand_i, c)) = cands
                        .iter()
                        .enumerate()
                        .rfind(|(_, c)| c.required_potential_rank <= build.potential)
                    else {
                        continue;
                    };
                    let key = TextKey::ModuleTalent {
                        module_id: module_id.clone(),
                        phase: phase_i,
                        part: part_i,
                        cand: cand_i,
                    };
                    // Hidden parts (`talent_index` -1, `is_hide_talent`) carry
                    // engine numbers with no text; the visible part of the same
                    // talent is the one the player reads.
                    if c.is_hide_talent == Some(true) || c.talent_index < 0 {
                        continue;
                    }
                    let index = usize::try_from(c.talent_index).unwrap_or(0);
                    let part_board = board(&c.blackboard);
                    if let Some(t) = talents.iter_mut().find(|t| t.index == index) {
                        if !c.name.is_empty() {
                            t.name.clone_from(&c.name);
                        }
                        if !c.upgrade_description.is_empty() {
                            t.description.clone_from(&c.upgrade_description);
                            t.key = key;
                        }
                        merge(&mut t.board, part_board);
                        t.module = Some(mname.clone());
                    } else if !c.name.is_empty() {
                        talents.push(LiveTalent {
                            index,
                            name: c.name.clone(),
                            description: if c.upgrade_description.is_empty() {
                                c.description.clone().unwrap_or_default()
                            } else {
                                c.upgrade_description.clone()
                            },
                            board: part_board,
                            token_key: c.token_key.clone().filter(|k| !k.is_empty()),
                            module: Some(mname.clone()),
                            key,
                        });
                    }
                }
                ModuleTarget::Trait | ModuleTarget::TraitDataOnly => {
                    let Some(cands) = &part.override_trait_data_bundle.candidates else {
                        continue;
                    };
                    let Some((cand_i, c)) = cands
                        .iter()
                        .enumerate()
                        .rfind(|(_, c)| c.required_potential_rank <= build.potential)
                    else {
                        continue;
                    };
                    let key = |additional| TextKey::ModuleTrait {
                        module_id: module_id.clone(),
                        phase: phase_i,
                        part: part_i,
                        cand: cand_i,
                        additional,
                    };
                    merge(&mut trait_board, board(&c.blackboard));
                    if let Some(d) = c.override_description.as_ref().filter(|d| !d.is_empty()) {
                        trait_text.clone_from(d);
                        trait_keys = vec![key(false)];
                    }
                    if !c.additional_description.is_empty() {
                        trait_text.push('\n');
                        trait_text.push_str(&c.additional_description);
                        trait_keys.push(key(true));
                    }
                }
                _ => {}
            }
        }
    }

    Some(Seat {
        seat,
        support: seat == SUPPORT_SEAT,
        char_id: b.char_id.clone(),
        op,
        build,
        phase,
        skill,
        talents,
        trait_text,
        trait_board,
        branch_board,
        trait_keys,
        module,
        module_phase,
        tag_ids: data.squad.tags.ids_of(op),
        rules,
    })
}

/// Whether the operator has a module past the default badge.
pub fn has_advanced_module(op: &Operator) -> bool {
    op.modules
        .iter()
        .any(|m| m.module.module_type == ModuleType::Advanced)
}
