use serde::{Deserialize, Serialize};

use myrtle_gamedata_raw::tables::operator::{
    Operator, OperatorModule, OperatorPhase, UnlockCondition,
};

use crate::types::Unlock;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRule {
    pub unlock: Unlock,
    pub mastery_unlocks: Vec<Unlock>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRules {
    pub rank_unlocks: Vec<Unlock>,
    pub skills: Vec<SkillRule>,
}

#[must_use]
pub const fn phase_num(p: &OperatorPhase) -> i32 {
    match p {
        OperatorPhase::Elite0 => 0,
        OperatorPhase::Elite1 => 1,
        OperatorPhase::Elite2 => 2,
    }
}

#[must_use]
pub const fn unlock_of(c: &UnlockCondition) -> Unlock {
    Unlock {
        phase: phase_num(&c.phase),
        level: c.level,
    }
}

impl SkillRules {
    #[must_use]
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

    #[must_use]
    pub fn max_rank(&self) -> i32 {
        i32::try_from(self.rank_unlocks.len()).map_or(1, |n| n + 1)
    }

    #[must_use]
    pub fn max_mastery(&self, index: usize) -> i32 {
        self.skills
            .get(index)
            .map_or(0, |s| i32::try_from(s.mastery_unlocks.len()).unwrap_or(0))
    }

    #[must_use]
    pub fn mastery_cap(&self, index: usize, skill_level: i32) -> i32 {
        if skill_level == self.max_rank() {
            self.max_mastery(index)
        } else {
            0
        }
    }

    #[must_use]
    pub fn rank_unlock(&self, rank: i32) -> Option<Unlock> {
        let i = usize::try_from(rank - 2).ok()?;
        self.rank_unlocks.get(i).cloned()
    }

    #[must_use]
    pub fn mastery_unlock(&self, index: usize, m: i32) -> Option<Unlock> {
        let i = usize::try_from(m - 1).ok()?;
        self.skills.get(index)?.mastery_unlocks.get(i).cloned()
    }

    #[must_use]
    pub fn skill_unlock(&self, index: usize) -> Option<Unlock> {
        self.skills.get(index).map(|s| s.unlock.clone())
    }
}

#[must_use]
pub const fn reached(u: &Unlock, phase: i32, level: i32) -> bool {
    u.phase < phase || (u.phase == phase && u.level <= level)
}

#[must_use]
pub fn module_ranks(m: &OperatorModule) -> i32 {
    i32::try_from(m.data.phases.len()).unwrap_or(0)
}
