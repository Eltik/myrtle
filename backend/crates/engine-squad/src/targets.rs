use myrtle_gamedata::index::squad::{targets::Target, types::SynergyTargetKind};
use myrtle_gamedata_raw::tables::operator::{Operator, OperatorPosition};

use crate::build::Seat;

pub fn power_ids(op: &Operator) -> impl Iterator<Item = &str> {
    let main = [
        Some(&op.nation_id),
        op.group_id.as_ref(),
        op.team_id.as_ref(),
    ];
    let sub = op.sub_power.iter().flatten().flat_map(|p| {
        [
            p.nation_id.as_ref(),
            p.group_id.as_ref(),
            p.team_id.as_ref(),
        ]
    });
    main.into_iter()
        .chain(sub)
        .flatten()
        .map(String::as_str)
        .filter(|id| !id.is_empty())
}

pub fn fits(target: &Target, seat: &Seat<'_>, cost: i32, tag_ids: &[i32]) -> bool {
    let op = seat.op;
    let class_ok = target
        .class
        .as_deref()
        .is_none_or(|c| op.profession.to_raw_str() == c);
    let v = target.value.as_deref().unwrap_or("");
    match target.kind {
        SynergyTargetKind::Everyone => true,
        SynergyTargetKind::Class => op.profession.to_raw_str() == v,
        SynergyTargetKind::Faction => power_ids(op).any(|id| target.powers.iter().any(|p| p == id)),
        SynergyTargetKind::Tag => v.parse::<i32>().is_ok_and(|id| tag_ids.contains(&id)),
        SynergyTargetKind::SpType => {
            class_ok
                && seat
                    .skill
                    .as_ref()
                    .is_some_and(|s| s.level.sp_data.sp_type == v)
        }
        SynergyTargetKind::Position => match op.position {
            OperatorPosition::Melee => v == "MELEE",
            OperatorPosition::Ranged => v == "RANGED",
            OperatorPosition::All => true,
            _ => false,
        },
        SynergyTargetKind::CostAtMost => v.parse::<i32>().is_ok_and(|n| cost <= n),
        SynergyTargetKind::Next | SynergyTargetKind::Unknown => false,
    }
}
