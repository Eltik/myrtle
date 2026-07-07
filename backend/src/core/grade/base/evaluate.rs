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
            base_pct,
            cap_pct,
        } => {
            let scaled = if *per_level {
                // "+X% per level of each Dormitory" → per_unit_pct * total_dorm_levels
                per_unit_pct * f64::from(ctx.total_dorm_levels)
            } else {
                // "+X% per Power Plant" → per_unit_pct * count
                let count = ctx
                    .facility_counts
                    .get(target_room.as_str())
                    .copied()
                    .unwrap_or(0);
                per_unit_pct * count as f64
            };
            // A stated ceiling ("Max +25%") clamps the scaled part.
            *base_pct + cap_pct.map_or(scaled, |cap| scaled.min(cap))
            // Note: nullifies_others is not handled here - the assignment
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

        BuffResolutionStrategy::MatchCountScaling {
            token,
            per_match_pct,
            cap_pct,
            bonus_char_id,
            bonus_pct,
        } => {
            // Count teammates whose match tags include this buff's keyword.
            let matches = ctx
                .room_teammates
                .iter()
                .filter(|t| t.match_tags.iter().any(|tag| tag == token))
                .count();
            let value = per_match_pct * matches as f64;
            let scaled = cap_pct.map_or(value, |cap| value.min(cap));
            // Optional named-teammate rider (Morgan's +35% with Siege).
            let rider = match bonus_char_id {
                Some(req) if ctx.room_teammates.iter().any(|t| t.char_id == *req) => *bonus_pct,
                _ => 0.0,
            };
            scaled + rider
        }

        // Converters reclassify teammates' tags (handled in the assignment layer
        // before evaluation); they contribute no efficiency themselves.
        BuffResolutionStrategy::SkillTypeConversion { .. } => 0.0,

        BuffResolutionStrategy::NullifyTeammatesSelfScaling { per_teammate_pct } => {
            // This operator's own value scales with the number of teammates
            // present. The zeroing of those teammates is enforced by the
            // assignment layer (solo room mode), not here.
            per_teammate_pct * ctx.room_teammates.len() as f64
        }

        // Order value is an LMD-per-order multiplier; treated as efficiency here.
        BuffResolutionStrategy::OrderValue { estimated_pct, .. } => *estimated_pct,

        BuffResolutionStrategy::GlobalEffect { bonus_pct, .. } => {
            // Global effects are not evaluated per-room - they're summed
            // separately by the assignment algorithm and added to each
            // target room. Return the raw value here.
            *bonus_pct
        }

        BuffResolutionStrategy::TagBased { bonus_pct, .. } => {
            // Similar to GlobalEffect - evaluated at the assignment level
            // where we know which operators are in which rooms.
            *bonus_pct
        }

        // Control Center faction-gated globals are applied per production room
        // against that room's actual team (see `compute_team_efficiency`), not as
        // an in-room operator buff - so they contribute nothing here.
        BuffResolutionStrategy::ConditionalGlobalEffect { .. } => 0.0,

        BuffResolutionStrategy::MoraleModifier {
            recovery_per_hour, ..
        } => {
            // Return raw value; scored in the morale dimension, not production
            *recovery_per_hour
        }

        BuffResolutionStrategy::CapacityOnly { .. } => 0.0,

        BuffResolutionStrategy::NonProduction { value } => *value,

        BuffResolutionStrategy::Complex { estimated_pct } => *estimated_pct,

        BuffResolutionStrategy::MoraleDecayEfficiency {
            time_averaged_value,
        } => *time_averaged_value,
        BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. } => {
            // Order limit is tracked separately via TeammateInfo
            *efficiency
        }
        BuffResolutionStrategy::ConditionalOnTeammate {
            required_char_id,
            base_efficiency,
            efficiency,
            ..
        } => {
            // The base efficiency always applies; the bonus efficiency is only
            // credited when the required teammate shares the room. Order limit is
            // tracked separately via TeammateInfo. An unresolved requirement
            // (None) never matches → only the base contributes.
            let bonus = match required_char_id {
                Some(req) if ctx.room_teammates.iter().any(|t| t.char_id == *req) => *efficiency,
                _ => 0.0,
            };
            *base_efficiency + bonus
        }
        BuffResolutionStrategy::ConditionalOnBaseWide {
            base_efficiency, ..
        } => {
            // The bonus depends on a named operator actively working a Work Area somewhere in the
            // base. The optimizer resolves that to a flat efficiency up front (two-pass over the
            // deployed roster) and scores against the resolved value, so here - where only the room
            // is known - just the always-on base applies.
            *base_efficiency
        }
        BuffResolutionStrategy::ConditionalOnFaction {
            faction_token,
            base_efficiency,
            efficiency,
        } => {
            // Base always applies; the bonus applies when ANY teammate carries the
            // faction tag (e.g. a Glasgow Gang operator shares the Trading Post).
            let present = ctx
                .room_teammates
                .iter()
                .any(|t| t.match_tags.iter().any(|tag| tag == faction_token));
            *base_efficiency + if present { *efficiency } else { 0.0 }
        }
        BuffResolutionStrategy::OrderLimitScaling {
            per_cap_threshold,
            bonus_per_threshold,
            cap_pct,
            includes_self,
        } => {
            let mut total_cap: i32 = ctx
                .room_teammates
                .iter()
                .map(|t| t.order_limit_contribution)
                .sum();
            // A self-counting scaler (Vermeil) also converts its OWN capacity (her +8); a
            // "from others" scaler (Jaye/Degenbrecher) does not.
            if *includes_self {
                total_cap += ctx.self_order_limit;
            }
            // Only count positive CAP for threshold calculation
            let effective_cap = f64::from(total_cap.max(0));
            let thresholds_met = (effective_cap / per_cap_threshold).floor();
            (thresholds_met * bonus_per_threshold).min(*cap_pct)
        }

        // No direct productivity; its effect is folded into the facility counts the optimizer
        // scores against (see the effective-count adjustment in the assignment).
        BuffResolutionStrategy::FacilityCountModifier { .. } => 0.0,

        BuffResolutionStrategy::CapacityTierScaling {
            threshold,
            low_pct,
            high_pct,
        } => {
            // Each operator in the room (this one + teammates) earns the high tier when its own
            // capacity bonus clears the threshold, else the low tier; the room gains the sum.
            let tier = |cap: i32| {
                if cap > *threshold {
                    *high_pct
                } else {
                    *low_pct
                }
            };
            let mut total = tier(ctx.self_order_limit);
            for t in &ctx.room_teammates {
                total += tier(t.order_limit_contribution);
            }
            total
        }
    }
}
