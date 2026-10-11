//! The Readiness panel: each seat's build against the squad's usual build and
//! against what the operator has unlocked.
//!
//! Reads `phases[phase].max_level`, `phases.len()`, the operator's skill
//! rules (`rules::SkillRules`: `Skills[i].UnlockCond`, `AllSkillLvlup`,
//! `LevelUpCostCond`) and the module's `unlock_evolve_phase` /
//! `unlock_level`.

use std::collections::HashMap;

use super::build::{Seat, has_advanced_module, phase_text_num};
use super::policy::LEVEL_GAP;
use super::rules::reached;
use super::types::{
    Readiness, ReadinessFlag, ReadinessFlagKind, ReadinessRow, ReadinessStats, Unlock,
};

const fn flag(
    kind: ReadinessFlagKind,
    value: Option<i32>,
    reference: Option<i32>,
) -> ReadinessFlag {
    ReadinessFlag {
        kind,
        value,
        reference,
    }
}

pub fn compute(seats: &[Seat<'_>]) -> Readiness {
    let squad: Vec<&Seat<'_>> = seats.iter().filter(|s| !s.support).collect();

    // The most common promotion, ties to the higher.
    let mut phase_counts: HashMap<i32, i32> = HashMap::new();
    for s in &squad {
        *phase_counts.entry(s.build.phase).or_insert(0) += 1;
    }
    let (usual_phase, usual_phase_count) = phase_counts
        .iter()
        .max_by_key(|(p, c)| (**c, **p))
        .map_or((0, 0), |(p, c)| (*p, *c));
    let mut levels: Vec<i32> = squad
        .iter()
        .filter(|s| s.build.phase == usual_phase)
        .map(|s| s.build.level)
        .collect();
    levels.sort_unstable();
    // Lower median on an even count.
    let median_level = (!levels.is_empty()).then(|| levels[(levels.len() - 1) / 2]);

    let count = |f: &dyn Fn(&Seat<'_>) -> bool| {
        i32::try_from(squad.iter().filter(|s| f(s)).count()).unwrap_or(0)
    };
    let stats = ReadinessStats {
        seats: i32::try_from(squad.len()).unwrap_or(0),
        usual_phase,
        usual_phase_count,
        median_level,
        // At the chosen skill's top mastery (`LevelUpCostCond` count), for
        // the skills that have any.
        mastery3: count(&|s| {
            let top = s
                .rules
                .max_mastery(usize::try_from(s.build.skill_index).unwrap_or(0));
            s.skill.is_some() && top > 0 && s.build.mastery == top
        }),
        chosen_skills: count(&|s| !s.op.skills.is_empty()),
        modules_on: count(&|s| s.module.is_some() && s.build.module_level >= 1),
        modules_possible: count(&|s| has_advanced_module(s.op)),
    };

    let mut rows: Vec<ReadinessRow> = seats
        .iter()
        .map(|s| {
            let b = &s.build;
            let max_phase = i32::try_from(s.op.phases.len().saturating_sub(1)).unwrap_or(0);
            let level_cap = s.op.phases.get(s.phase).map_or(1, |p| p.max_level);
            let skill_unlock = s
                .skill
                .as_ref()
                .and_then(|sk| s.rules.skill_unlock(sk.index));
            let module_unlock = s.module.map(|m| Unlock {
                phase: phase_text_num(&m.module.unlock_evolve_phase),
                level: m.module.unlock_level,
            });
            let mut flags = Vec::new();
            if s.support {
                flags.push(flag(ReadinessFlagKind::SupportLeftOut, None, None));
            }
            if let Some(u) = &skill_unlock
                && !reached(u, b.phase, b.level)
            {
                flags.push(flag(
                    ReadinessFlagKind::SkillLocked,
                    Some(u.phase),
                    Some(u.level),
                ));
            }
            if let Some(sk) = &s.skill {
                // The mastery's unlock when one is chosen, else the rank's.
                let need = s
                    .rules
                    .mastery_unlock(sk.index, b.mastery)
                    .filter(|_| b.mastery > 0)
                    .or_else(|| s.rules.rank_unlock(b.skill_level));
                if let Some(u) = need.filter(|u| !reached(u, b.phase, b.level)) {
                    flags.push(flag(
                        ReadinessFlagKind::SkillRankLocked,
                        Some(u.phase),
                        Some(u.level),
                    ));
                }
            }
            if let Some(u) = &module_unlock
                && (b.phase < u.phase || (b.phase == u.phase && b.level < u.level))
            {
                flags.push(flag(
                    ReadinessFlagKind::ModuleLocked,
                    Some(u.phase),
                    Some(u.level),
                ));
            }
            if !s.support {
                if b.phase < usual_phase && max_phase >= usual_phase {
                    flags.push(flag(
                        ReadinessFlagKind::Promotion,
                        Some(b.phase),
                        Some(usual_phase),
                    ));
                }
                if let Some(m) = median_level
                    && b.phase == usual_phase
                    && m - b.level >= LEVEL_GAP
                {
                    flags.push(flag(ReadinessFlagKind::Level, Some(b.level), Some(m)));
                }
            }
            ReadinessRow {
                seat: s.seat,
                char_id: s.char_id.clone(),
                support: s.support,
                phase: b.phase,
                level: b.level,
                level_cap,
                max_phase,
                potential: b.potential,
                skill_index: b.skill_index,
                skill_level: b.skill_level,
                mastery: b.mastery,
                skill_unlock,
                module_id: s.module.map(|m| m.module.uni_equip_id.clone()),
                module_level: if s.module.is_some() {
                    b.module_level
                } else {
                    0
                },
                module_unlock,
                flags,
            }
        })
        .collect();
    rows.sort_by_key(|r| r.seat);
    Readiness { stats, rows }
}
