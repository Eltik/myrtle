//! The Themes panel: what the squad seats have in common. The support is left
//! out. Values are the game's own strings: handbook gender, race and
//! birthplace by their table names (gender in the server's own recruitment tag
//! name), faction names, the profession as the game names it (`PIONEER`),
//! `sub_profession_id`, position as the server's recruitment tag name, star
//! count, artists, voice actors for one dub, and birthdays verbatim.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::gacha::{
    FEMALE_TAG_ID, MALE_TAG_ID, MELEE_TAG_ID, RANGED_TAG_ID,
};
use crate::core::gamedata::types::handbook::{OperatorBirthPlace, OperatorGender, OperatorRace};
use crate::core::gamedata::types::operator::{OperatorPosition, OperatorProfession};

use super::build::Seat;
use super::policy::{
    THEME_CLOSE_MAX_MISSING, THEME_CLOSE_MIN_SEATS, THEME_CLOSE_MIN_SHARED, THEME_LABEL_MIN_SEATS,
    THEME_SHARED_MIN,
};
use super::types::{ThemeKind, ThemeShare, ThemeStats, Themes};

/// The table's own name for an enum value (its serde name).
fn serde_name<T: Serialize>(v: &T) -> Option<String> {
    serde_json::to_value(v)
        .ok()
        .and_then(|j| j.as_str().map(str::to_string))
}

/// This server's `gacha_table.GachaTags` name of a tag id.
fn tag_name(data: &GameData, id: i32) -> Option<String> {
    data.squad.tags.name(id).map(str::to_string)
}

const fn kind_rank(k: ThemeKind) -> u8 {
    match k {
        ThemeKind::Gender => 0,
        ThemeKind::Race => 1,
        ThemeKind::Birthplace => 2,
        ThemeKind::Faction => 3,
        ThemeKind::Class => 4,
        ThemeKind::Branch => 5,
        ThemeKind::Position => 6,
        ThemeKind::Rarity => 7,
        ThemeKind::Artist => 8,
        ThemeKind::Voice => 9,
        ThemeKind::Birthday => 10,
    }
}

fn values(s: &Seat<'_>, data: &GameData, dub: &str) -> Vec<(ThemeKind, String)> {
    let op = s.op;
    let mut v = Vec::new();
    if let Some(info) = op.profile.as_ref().map(|p| &p.basic_info) {
        // The server's own gender tag name; Arene's table entry reads
        // "Male]" (`MaleBugged`). Another gender keeps its table name.
        let tag = match info.gender {
            OperatorGender::Male | OperatorGender::MaleBugged => Some(MALE_TAG_ID),
            OperatorGender::Female => Some(FEMALE_TAG_ID),
            _ => None,
        };
        if info.gender != OperatorGender::Unknown {
            v.extend(
                tag.and_then(|id| tag_name(data, id))
                    .or_else(|| serde_name(&info.gender))
                    .map(|n| (ThemeKind::Gender, n)),
            );
        }
        if info.race != OperatorRace::Unknown {
            v.extend(serde_name(&info.race).map(|n| (ThemeKind::Race, n)));
        }
        if info.place_of_birth != OperatorBirthPlace::Unknown {
            v.extend(serde_name(&info.place_of_birth).map(|n| (ThemeKind::Birthplace, n)));
        }
        if !info.date_of_birth.trim().is_empty() {
            v.push((ThemeKind::Birthday, info.date_of_birth.trim().to_string()));
        }
    }
    // Card factions only: `hidden_factions` sit behind the reader's spoiler
    // setting, so a theme never names them.
    for f in [&op.nation_name, &op.group_name, &op.team_name]
        .into_iter()
        .flatten()
    {
        v.push((ThemeKind::Faction, f.clone()));
    }
    v.push((ThemeKind::Class, op.profession.to_raw_str().to_string()));
    if !op.sub_profession_id.is_empty() {
        v.push((ThemeKind::Branch, op.sub_profession_id.clone()));
    }
    let position = match op.position {
        OperatorPosition::Melee => Some(MELEE_TAG_ID),
        OperatorPosition::Ranged => Some(RANGED_TAG_ID),
        _ => None,
    };
    if let Some(id) = position {
        v.extend(
            tag_name(data, id)
                .or_else(|| serde_name(&op.position))
                .map(|n| (ThemeKind::Position, n)),
        );
    }
    v.push((ThemeKind::Rarity, op.rarity.to_star_int().to_string()));
    for a in &op.artists {
        v.push((ThemeKind::Artist, a.clone()));
    }
    if let Some(entry) = data
        .voices
        .voice_lang_dict
        .get(&s.char_id)
        .and_then(|l| l.dict.get(dub))
    {
        for n in &entry.cv_name {
            v.push((ThemeKind::Voice, n.clone()));
        }
    }
    // One operator counts once per value.
    v.sort_by(|a, b| (kind_rank(a.0), &a.1).cmp(&(kind_rank(b.0), &b.1)));
    v.dedup();
    v
}

pub fn compute(seats: &[Seat<'_>], data: &GameData, dub: Option<&str>) -> Themes {
    // No dub asked for: the server's own default voice language
    // (`charword_table.DefaultLangType`, `JP` on every server on 2026-10-11).
    let dub = dub
        .filter(|d| !d.is_empty())
        .unwrap_or(&data.voices.default_lang_type)
        .to_string();
    let squad: Vec<&Seat<'_>> = seats.iter().filter(|s| !s.support).collect();
    let n = squad.len();

    let mut shares: BTreeMap<(u8, String), (ThemeKind, Vec<i32>)> = BTreeMap::new();
    for s in &squad {
        for (kind, value) in values(s, data, &dub) {
            shares
                .entry((kind_rank(kind), value))
                .or_insert_with(|| (kind, Vec::new()))
                .1
                .push(s.seat);
        }
    }

    let mut labels = Vec::new();
    let mut close = Vec::new();
    let mut shared = Vec::new();
    for ((_, value), (kind, seats)) in shares {
        let c = seats.len();
        let share = ThemeShare { kind, value, seats };
        if n >= THEME_LABEL_MIN_SEATS && c == n {
            labels.push(share);
        } else if n >= THEME_CLOSE_MIN_SEATS
            && c >= THEME_CLOSE_MIN_SHARED
            && c + THEME_CLOSE_MAX_MISSING >= n
        {
            close.push(share);
        } else if c >= THEME_SHARED_MIN {
            shared.push(share);
        }
    }
    for list in [&mut labels, &mut close, &mut shared] {
        list.sort_by(|a, b| {
            b.seats
                .len()
                .cmp(&a.seats.len())
                .then(kind_rank(a.kind).cmp(&kind_rank(b.kind)))
                .then(a.value.cmp(&b.value))
        });
    }

    let mut professions: Vec<&OperatorProfession> =
        squad.iter().map(|s| &s.op.profession).collect();
    professions.sort_by_key(|p| p.to_raw_str());
    professions.dedup();
    let stats = ThemeStats {
        seats: i32::try_from(n).unwrap_or(0),
        total_rarity: squad
            .iter()
            .map(|s| i32::from(s.op.rarity.to_star_int()))
            .sum(),
        top_rarity: i32::from(data.squad.top_rarity),
        top_rarity_seats: i32::try_from(
            squad
                .iter()
                .filter(|s| s.op.rarity.to_star_int() == data.squad.top_rarity)
                .count(),
        )
        .unwrap_or(0),
        medics: i32::try_from(
            squad
                .iter()
                .filter(|s| s.op.profession == OperatorProfession::Medic)
                .count(),
        )
        .unwrap_or(0),
        classes: i32::try_from(professions.len()).unwrap_or(0),
        class_total: i32::try_from(data.squad.professions.len()).unwrap_or(0),
        missing: data
            .squad
            .professions
            .iter()
            .filter(|p| !professions.contains(p))
            .map(|p| p.to_raw_str().to_string())
            .collect(),
    };
    Themes {
        stats,
        labels,
        close,
        shared,
        dub,
    }
}
