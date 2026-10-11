//! What an operator's skills allow, read off `character_table`: when each
//! skill unlocks (`Skills[i].UnlockCond`), when each shared skill rank
//! unlocks (`AllSkillLvlup[r - 2].UnlockCond`) and how many masteries each
//! skill has and when (`Skills[i].LevelUpCostCond[m - 1].UnlockCond`).
//!
//! The one source for the analysis (`build`, `readiness`), the save checks
//! (`services::squad`) and the page (`skillRules` on the operator index
//! entry), so no side re-derives a rule. A 3-star and the eight 4-star
//! Reserve Operators have empty `LevelUpCostCond`, so they have no mastery;
//! a 1- or 2-star has no `AllSkillLvlup`, so its skill stays at rank 1.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::core::gamedata::types::operator::{Operator, OperatorPhase, UnlockCondition};

use super::types::Unlock;

/// One skill's unlock and masteries.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRule {
    /// `Skills[i].UnlockCond`.
    pub unlock: Unlock,
    /// Mastery `m` unlocks at `mastery_unlocks[m - 1]`; the length is the
    /// skill's mastery count (0 for 3-stars and Reserve Operators).
    pub mastery_unlocks: Vec<Unlock>,
}

/// An operator's skill rules.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRules {
    /// Skill rank `r` (2 and up) unlocks at `rank_unlocks[r - 2]`; the top rank
    /// is `rank_unlocks.len() + 1`.
    pub rank_unlocks: Vec<Unlock>,
    /// One per skill, in skill order.
    pub skills: Vec<SkillRule>,
}

const fn phase_num(p: &OperatorPhase) -> i32 {
    match p {
        OperatorPhase::Elite0 => 0,
        OperatorPhase::Elite1 => 1,
        OperatorPhase::Elite2 => 2,
    }
}

const fn unlock_of(c: &UnlockCondition) -> Unlock {
    Unlock {
        phase: phase_num(&c.phase),
        level: c.level,
    }
}

impl SkillRules {
    pub fn of(op: &Operator) -> Self {
        Self {
            rank_unlocks: op
                .all_skill_level_up
                .iter()
                .map(|l| unlock_of(&l.unlock_cond))
                .collect(),
            skills: op
                .skills
                .iter()
                .map(|s| SkillRule {
                    // A skill with no `UnlockCond` has no condition: Elite 0 Lv 1.
                    unlock: s
                        .unlock_cond
                        .as_ref()
                        .map_or(Unlock { phase: 0, level: 1 }, unlock_of),
                    mastery_unlocks: s
                        .level_up_cost_cond
                        .iter()
                        .map(|c| unlock_of(&c.unlock_cond))
                        .collect(),
                })
                .collect(),
        }
    }

    /// The top skill rank, 1 when the operator has no rank-ups.
    pub fn max_rank(&self) -> i32 {
        i32::try_from(self.rank_unlocks.len()).map_or(1, |n| n + 1)
    }

    /// How many masteries skill `index` has; 0 for a skill it does not have.
    pub fn max_mastery(&self, index: usize) -> i32 {
        self.skills
            .get(index)
            .map_or(0, |s| i32::try_from(s.mastery_unlocks.len()).unwrap_or(0))
    }

    /// What rank `rank` needs; `None` for rank 1 or a rank past the top.
    pub fn rank_unlock(&self, rank: i32) -> Option<Unlock> {
        let i = usize::try_from(rank - 2).ok()?;
        self.rank_unlocks.get(i).cloned()
    }

    /// What mastery `m` of skill `index` needs; `None` for 0 or past the count.
    pub fn mastery_unlock(&self, index: usize, m: i32) -> Option<Unlock> {
        let i = usize::try_from(m - 1).ok()?;
        self.skills.get(index)?.mastery_unlocks.get(i).cloned()
    }

    pub fn skill_unlock(&self, index: usize) -> Option<Unlock> {
        self.skills.get(index).map(|s| s.unlock.clone())
    }
}

/// Whether a build at `phase`/`level` has reached `u`.
pub const fn reached(u: &Unlock, phase: i32, level: i32) -> bool {
    u.phase < phase || (u.phase == phase && u.level <= level)
}
