use super::buff_registry::BuffResolutionStrategy;
use super::types::EvalContext;

pub fn evaluate_buff(strategy: &BuffResolutionStrategy, ctx: &EvalContext) -> f64 {
    match strategy {
        BuffResolutionStrategy::DirectEfficiency { value } => *value,

        BuffResolutionStrategy::FacilityCountScaling {
            target_room,
            per_unit_pct,
            per_level,
            nullifies_others: _,
        } => {
            if *per_level {
                // "+X% per level of each Dormitory" → per_unit_pct * total_dorm_levels
                per_unit_pct * ctx.total_dorm_levels as f64
            } else {
                // "+X% per Power Plant" → per_unit_pct * count
                let count = ctx
                    .facility_counts
                    .get(target_room.as_str())
                    .copied()
                    .unwrap_or(0);
                per_unit_pct * count as f64
            }
            // Note: nullifies_others is not handled here — the assignment
            // algorithm handles it by zeroing teammates' contributions
        }

        BuffResolutionStrategy::TeammateSkillScaling {
            target_buff_pattern,
            per_match_pct,
        } => {
            // Count how many teammates have a buff whose ID starts with target_buff_pattern
            let matches = ctx
                .room_teammates
                .iter()
                .filter(|t| {
                    t.buff_ids
                        .iter()
                        .any(|b| b.starts_with(target_buff_pattern.as_str()))
                })
                .count();
            per_match_pct * matches as f64
        }

        BuffResolutionStrategy::TeammateOutputMirroring { ratio, cap_pct } => {
            // Sum teammates' direct efficiency, multiply by ratio, cap at cap_pct
            let teammate_total: f64 = ctx.room_teammates.iter().map(|t| t.direct_efficiency).sum();
            (teammate_total * ratio).min(*cap_pct)
        }

        BuffResolutionStrategy::GlobalEffect { bonus_pct, .. } => {
            // Global effects are not evaluated per-room — they're summed
            // separately by the assignment algorithm and added to each
            // target room. Return the raw value here.
            *bonus_pct
        }

        BuffResolutionStrategy::TagBased { bonus_pct, .. } => {
            // Similar to GlobalEffect — evaluated at the assignment level
            // where we know which operators are in which rooms.
            *bonus_pct
        }

        BuffResolutionStrategy::MoraleModifier {
            recovery_per_hour, ..
        } => {
            // Return raw value; scored in the morale dimension, not production
            *recovery_per_hour
        }

        BuffResolutionStrategy::CapacityOnly => 0.0,

        BuffResolutionStrategy::NonProduction { value } => *value,

        BuffResolutionStrategy::Complex { estimated_pct } => *estimated_pct,

        BuffResolutionStrategy::MoraleDecayEfficiency {
            time_averaged_value,
        } => *time_averaged_value,
        BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. } => {
            // Order limit is tracked separately via TeammateInfo
            *efficiency
        }
        BuffResolutionStrategy::OrderLimitScaling {
            per_cap_threshold,
            bonus_per_threshold,
            cap_pct,
        } => {
            let total_cap: i32 = ctx
                .room_teammates
                .iter()
                .map(|t| t.order_limit_contribution)
                .sum();
            // Only count positive CAP for threshold calculation
            let effective_cap = total_cap.max(0) as f64;
            let thresholds_met = (effective_cap / per_cap_threshold).floor();
            (thresholds_met * bonus_per_threshold).min(*cap_pct)
        }
    }
}
