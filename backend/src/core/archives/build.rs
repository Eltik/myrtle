//! Build every Archives view from one server's game data, once.
//!
//! The population is the operator FILES: every `char_*` operator with its own
//! `handbook_info_table` entry, Amiya's alternate forms folded into her
//! (they share her file). Story appearances come in from the caller, already
//! attributed by the story reader's speaker walk.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use super::dto::{
    ArchiveAppearance, ArchiveBlock, ArchiveBucket, ArchiveCell, ArchiveChartRef, ArchiveCount,
    ArchiveDayCount, ArchiveDemographics, ArchiveDistribution, ArchiveDossier, ArchiveDossiers,
    ArchiveEdge, ArchiveExamColumn, ArchiveFactionRef, ArchiveFiller, ArchiveForce,
    ArchiveGenderCount, ArchiveGroup, ArchiveGroupMean, ArchiveHex, ArchiveHexLink,
    ArchiveHiddenFactions, ArchiveHolder, ArchiveInfectionRow, ArchiveLine, ArchiveNationBehind,
    ArchiveNetwork, ArchiveNumber, ArchiveOperator, ArchivePair, ArchivePerson, ArchivePoint,
    ArchivePrefix, ArchiveRecord, ArchiveScaleStep, ArchiveSceneMate, ArchiveSecret,
    ArchiveShownMember,
};
use super::network::{RawConnection, RawNetwork, Vec2};
use super::profile::{
    ArchiveGender, ArchiveProfile, ExamAxis, ExamStep, InfectionState, Lang, parse_archive_profile,
};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::handbook_team::{HandbookTeam, NO_FACTION};
use crate::core::gamedata::types::operator::Operator;

/// Buckets per histogram.
pub const BUCKETS: usize = 16;
/// A per-group figure (average height by race, infection by birthplace) needs
/// at least this many files behind it: one operator is not a range. A
/// presentation threshold, not a property of the data.
pub const MIN_GROUP: u32 = 6;
/// Scene partners listed per operator.
pub const SCENE_MATES: usize = 12;
/// A birthday is listed as shared when at least this many files give it.
pub const SHARED_DAY: u32 = 2;

/// Every view, built together because they share the parse.
#[derive(Debug, Clone, Default)]
pub struct Archive {
    pub network: ArchiveNetwork,
    pub dossiers: ArchiveDossiers,
    pub demographics: ArchiveDemographics,
    pub hidden: ArchiveHiddenFactions,
    pub operators: HashMap<String, ArchiveOperator>,
}

/// Story appearances per char id, in library order.
pub type SceneIndex = HashMap<String, Vec<ArchiveAppearance>>;

struct Ctx<'a> {
    gd: &'a GameData,
    teams: &'a HashMap<String, HandbookTeam>,
}

/// Steps on the exam scale, as the operator DTO sends it.
#[allow(clippy::cast_possible_truncation)]
const EXAM_STEPS: u32 = ExamStep::ALL.len() as u32;

/// The most specific of `ids` by `handbook_team_table.PowerLevel` (2 team >
/// 1 group > 0 nation); an id with no row ranks below every row, and on
/// equal levels the earlier id stands.
fn deepest<'s>(
    teams: &HashMap<String, HandbookTeam>,
    ids: impl IntoIterator<Item = Option<&'s str>>,
) -> Option<String> {
    ids.into_iter()
        .filter_map(real)
        .enumerate()
        .max_by_key(|(i, id)| {
            (
                teams.get(*id).map_or(-1, |t| t.power_level),
                std::cmp::Reverse(*i),
            )
        })
        .map(|(_, id)| id.to_owned())
}

/// The faction a card shows: the `character_table.MainPower` id with the
/// highest `PowerLevel`. An operator with no `MainPower` falls back to the
/// top-level `NationId` / `GroupId` / `TeamId`.
#[must_use]
pub fn card_faction(teams: &HashMap<String, HandbookTeam>, op: &Operator) -> Option<String> {
    match &op.main_power {
        Some(p) => deepest(
            teams,
            [
                p.team_id.as_deref(),
                p.group_id.as_deref(),
                p.nation_id.as_deref(),
            ],
        ),
        None => deepest(
            teams,
            [
                op.team_id.as_deref(),
                op.group_id.as_deref(),
                Some(op.nation_id.as_str()),
            ],
        ),
    }
}

fn real(id: Option<&str>) -> Option<&str> {
    // `NO_FACTION` is a real `handbook_team_table` row (`PowerCode: NONE`).
    id.filter(|id| !id.is_empty() && *id != NO_FACTION)
}

impl Ctx<'_> {
    fn deepest<'s>(&self, ids: impl IntoIterator<Item = Option<&'s str>>) -> Option<String> {
        deepest(self.teams, ids)
    }

    fn card_faction(&self, op: &Operator) -> Option<String> {
        card_faction(self.teams, op)
    }

    fn faction_name(&self, id: &str) -> String {
        self.teams
            .get(id)
            .map(|t| t.power_name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.to_owned())
    }

    fn faction_ref(&self, id: &str) -> ArchiveFactionRef {
        ArchiveFactionRef {
            id: id.to_owned(),
            name: self.faction_name(id),
        }
    }

    fn person(&self, id: &str) -> ArchivePerson {
        if let Some(op) = self.gd.operators.get(id) {
            return ArchivePerson {
                id: id.to_owned(),
                name: op.name.clone(),
                number: some_str(&op.display_number),
                rarity: u8::try_from(op.rarity.to_star_int()).ok(),
                profession: Some(op.profession.to_raw_str().to_owned()),
                npc: false,
                faction_id: self.card_faction(op),
            };
        }
        if let Some(npc) = self.gd.handbook.npc_dict.get(id) {
            return ArchivePerson {
                id: id.to_owned(),
                name: npc.name.clone(),
                number: some_str(&npc.display_number),
                rarity: None,
                profession: None,
                npc: true,
                faction_id: self.deepest([
                    npc.team_id.as_deref(),
                    npc.group_id.as_deref(),
                    npc.nation_id.as_deref(),
                ]),
            };
        }
        ArchivePerson {
            id: id.to_owned(),
            name: id.to_owned(),
            number: None,
            rarity: None,
            profession: None,
            npc: !id.starts_with("char_"),
            faction_id: None,
        }
    }

    fn people<'b>(&self, ids: impl IntoIterator<Item = &'b String>) -> Vec<ArchivePerson> {
        let set: BTreeSet<&String> = ids.into_iter().collect();
        set.into_iter().map(|id| self.person(id)).collect()
    }
}

fn some_str(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_owned())
}

/// The letters a personnel number leads with (`PL02` -> `PL`).
#[must_use]
pub fn number_prefix(number: &str) -> Option<String> {
    let p: String = number
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect();
    (!p.is_empty()).then_some(p)
}

/// The ids whose files the Archives count, sorted.
#[must_use]
pub fn file_ids(gd: &GameData) -> Vec<String> {
    let mut ids: Vec<String> = gd
        .operators
        .iter()
        .filter(|(id, op)| {
            id.starts_with("char_")
                && gd.handbook.handbook_dict.contains_key(id.as_str())
                && op.tmpl_default.as_deref().is_none_or(|d| d == id.as_str())
        })
        .map(|(id, _)| id.clone())
        .collect();
    ids.sort();
    ids
}

/// `lang` is the language of the server's handbook (see
/// [`Lang::for_server`]); `None` leaves every file unread.
#[must_use]
pub fn build_archive(
    gd: &GameData,
    teams: &HashMap<String, HandbookTeam>,
    network: Option<&RawNetwork>,
    scenes: &SceneIndex,
    lang: Option<Lang>,
) -> Archive {
    let ctx = Ctx { gd, teams };
    let ids = file_ids(gd);
    let profiles: HashMap<&str, ArchiveProfile> = lang
        .map(|lang| {
            ids.iter()
                .filter_map(|id| {
                    let item = gd.handbook.handbook_dict.get(id)?;
                    Some((
                        id.as_str(),
                        parse_archive_profile(lang, &item.story_text_audio)?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    let net = network.map_or_else(ArchiveNetwork::default, |n| build_network(&ctx, n));
    let links = links_by_char(&net);
    let charts = charts_by_char(&net);
    let mates = scene_mates(scenes);
    let prefixes = build_prefixes(&ctx, &ids);

    let mut operators = HashMap::new();
    let mut dossiers = Vec::new();
    for id in &ids {
        let Some(op) = gd.operators.get(id) else {
            continue;
        };
        let profile = profiles.get(id.as_str()).cloned();
        let appearances = scenes.get(id).cloned().unwrap_or_default();
        let linked = links
            .get(id.as_str())
            .map(|s| fold_npc_twins(&ctx, s))
            .unwrap_or_default();
        let records: Vec<ArchiveRecord> = gd
            .handbook
            .handbook_dict
            .get(id)
            .map(|h| {
                h.handbook_avg_list
                    .iter()
                    .map(|r| ArchiveRecord {
                        id: r.story_set_id.clone(),
                        name: r.story_set_name.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let shares = mates.get(id.as_str()).cloned().unwrap_or_default();
        let faction_id = ctx.card_faction(op);
        let p = profile.clone().unwrap_or_default();
        dossiers.push(ArchiveDossier {
            id: id.clone(),
            name: op.name.clone(),
            rarity: u8::try_from(op.rarity.to_star_int()).unwrap_or(0),
            profession: op.profession.to_raw_str().to_owned(),
            sub_profession_id: op.sub_profession_id.clone(),
            sub_profession_name: op.sub_profession_name.clone(),
            faction_id: faction_id.clone(),
            nation: real(Some(op.nation_id.as_str())).map(|f| ctx.faction_ref(f)),
            affiliations: [real(op.group_id.as_deref()), real(op.team_id.as_deref())]
                .into_iter()
                .flatten()
                .map(|f| ctx.faction_ref(f))
                .collect(),
            gender_key: lang.and_then(|l| ArchiveGender::read(l.vocab(), p.gender.as_deref()?)),
            gender: p.gender,
            race: p.race,
            birthplace: p.birthplace,
            height: p.height,
            height_cm: p.height_cm,
            combat_experience: p.combat_experience,
            combat_years: p.combat_years,
            infection_state: p.infection_state,
            assimilation: p.assimilation,
            assimilation_pct: p.assimilation_pct,
            story_count: count_u32(appearances.len()),
            record_count: count_u32(records.len()),
            link_count: count_u32(linked.len()),
        });
        let number = number_prefix(&op.display_number).map(|prefix| {
            let row = prefixes.iter().find(|p| p.prefix == prefix);
            ArchiveNumber {
                display: op.display_number.clone(),
                holders: row.map_or(0, |r| count_u32(r.holders.len())),
                faction: row.and_then(|r| r.faction.clone()),
                faction_holders: row.map_or(0, |r| r.faction_holders),
                prefix,
            }
        });
        let factions = [
            real(Some(op.nation_id.as_str())),
            real(op.group_id.as_deref()),
            real(op.team_id.as_deref()),
        ]
        .into_iter()
        .flatten()
        .map(|f| ctx.faction_ref(f))
        .collect();
        let people = ctx.people(shares.iter().map(|m| &m.char_id).chain(linked.iter()));
        operators.insert(
            id.clone(),
            ArchiveOperator {
                id: id.clone(),
                name: op.name.clone(),
                rarity: u8::try_from(op.rarity.to_star_int()).unwrap_or(0),
                profession: op.profession.to_raw_str().to_owned(),
                sub_profession_id: op.sub_profession_id.clone(),
                sub_profession_name: op.sub_profession_name.clone(),
                number,
                factions,
                hidden_factions: op.hidden_factions.clone(),
                profile,
                records,
                appearances,
                shares_scenes: shares,
                linked,
                charts: charts.get(id.as_str()).cloned().unwrap_or_default(),
                people,
                exam_steps: EXAM_STEPS,
            },
        );
    }
    dossiers.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));

    let demographics = build_demographics(&ctx, &ids, &profiles, prefixes, lang);
    let hidden = build_hidden(&ctx, &ids);
    Archive {
        network: net,
        dossiers: ArchiveDossiers { dossiers },
        demographics,
        hidden,
        operators,
    }
}

fn count_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn point(v: Vec2) -> ArchivePoint {
    ArchivePoint { x: v.x, y: v.y }
}

fn edges(list: &[RawConnection]) -> Vec<ArchiveEdge> {
    list.iter()
        .map(|c| ArchiveEdge {
            direction: c.direction,
            char_id: c.char_id.clone(),
            force_id: c.force_id.clone(),
        })
        .collect()
}

fn build_network(ctx: &Ctx, raw: &RawNetwork) -> ArchiveNetwork {
    let pos = &raw.positions;
    let order = |id: &str| ctx.teams.get(id).map_or(i32::MAX, |t| t.order_num);

    let mut groups: Vec<ArchiveGroup> = pos
        .group_list
        .values()
        .map(|g| {
            let blocks: Vec<ArchiveBlock> = g
                .force_data_list
                .iter()
                .map(|b| ArchiveBlock {
                    force_id: b.force_id.clone(),
                    name: ctx.faction_name(&b.force_id),
                    color: b.color.clone(),
                    pos: point(b.pos),
                    focus: point(b.focus_pos),
                    home: b.is_this_group,
                    members: b.char_list.clone(),
                })
                .collect();
            let block_of = |char_id: &str| {
                let mut found = blocks
                    .iter()
                    .filter(|b| b.members.iter().any(|m| m == char_id));
                let all: Vec<&ArchiveBlock> = found.by_ref().collect();
                all.iter()
                    .find(|b| b.home)
                    .or_else(|| all.first())
                    .map(|b| b.force_id.clone())
            };
            let cells = g
                .char_list
                .iter()
                .map(|c| ArchiveCell {
                    char_id: c.char_id.clone(),
                    x: c.pos.x,
                    y: c.pos.y,
                    scale: c.scale,
                    outline: c.have_out_line,
                    edges: edges(&c.connection_list),
                    force_id: block_of(&c.char_id),
                })
                .collect();
            let fillers = g
                .force_data_list
                .iter()
                .flat_map(|b| &b.color_blocks)
                .map(|f| ArchiveFiller {
                    id: f.id.clone(),
                    force_id: f.force_id.clone(),
                    color: f.color.clone(),
                    pos: point(f.pos),
                    edges: edges(&f.connection_list),
                })
                .collect();
            let mut seen = HashSet::new();
            let lines = g
                .line_list
                .iter()
                .filter(|l| {
                    let key = if l.char1 < l.char2 {
                        (l.char1.clone(), l.char2.clone())
                    } else {
                        (l.char2.clone(), l.char1.clone())
                    };
                    seen.insert(key)
                })
                .map(|l| ArchiveLine {
                    a: l.char1.clone(),
                    b: l.char2.clone(),
                })
                .collect();
            // `kazdel` is a chart with no `handbook_team_table` row; it is
            // named after the force that lives on it (Babel).
            let name = ctx
                .teams
                .get(&g.group_id)
                .map(|t| t.power_name.clone())
                .or_else(|| blocks.iter().find(|b| b.home).map(|b| b.name.clone()))
                .unwrap_or_else(|| g.group_id.clone());
            ArchiveGroup {
                id: g.group_id.clone(),
                name,
                min_scale: g.min_scale,
                max_scale: g.max_scale,
                cells,
                blocks,
                fillers,
                lines,
            }
        })
        .collect();
    groups.sort_by(|a, b| {
        order(&a.id)
            .cmp(&order(&b.id))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut forces: Vec<ArchiveForce> = raw
        .force_map
        .force_list
        .iter()
        .map(|f| {
            let group_id = pos
                .force_to_group_dict
                .get(&f.force_id)
                .cloned()
                .unwrap_or_else(|| f.force_id.clone());
            let member_count = groups
                .iter()
                .find(|g| g.id == group_id)
                .and_then(|g| g.blocks.iter().find(|b| b.force_id == f.force_id && b.home))
                .map_or(0, |b| count_u32(b.members.len()));
            let team = ctx.teams.get(&f.force_id);
            ArchiveForce {
                id: f.force_id.clone(),
                name: ctx.faction_name(&f.force_id),
                code: team.map_or_else(|| f.force_id.clone(), |t| t.power_code.clone()),
                level: team.map_or(0, |t| t.power_level),
                color: f.color.clone(),
                card_color: if f.card_color.is_empty() {
                    f.color.clone()
                } else {
                    f.card_color.clone()
                },
                group_id,
                logo: point(f.logo_pos),
                logo_scale: f.logo_scale,
                card: point(f.card_pos),
                card_scale: f.card_scale,
                hexes: f
                    .point_list
                    .iter()
                    .map(|p| ArchiveHex {
                        index: p.point_index,
                        x: p.pos.x,
                        y: p.pos.y,
                    })
                    .collect(),
                member_count,
            }
        })
        .collect();
    forces.sort_by_key(|f| f.id.clone());
    forces.sort_by_key(|f| {
        raw.force_map
            .force_list
            .iter()
            .find(|r| r.force_id == f.id)
            .map_or(i32::MAX, |r| r.force_index)
    });

    let everyone: BTreeSet<&String> = groups
        .iter()
        .flat_map(|g| {
            g.cells
                .iter()
                .map(|c| &c.char_id)
                .chain(g.blocks.iter().flat_map(|b| &b.members))
        })
        .collect();
    let people = ctx.people(everyone);

    ArchiveNetwork {
        imported: true,
        force_links: raw
            .force_map
            .force_line_list
            .iter()
            .map(|l| ArchivePair {
                a: l.force_id1.clone(),
                b: l.force_id2.clone(),
            })
            .collect(),
        hex_links: raw
            .force_map
            .point_line_list
            .iter()
            .map(|l| ArchiveHexLink {
                from: l.from_idx,
                to: l.to_idx,
            })
            .collect(),
        forces,
        groups,
        people,
    }
}

/// `ids` with every NPC dropped whose name an operator in `ids` also carries.
/// The network places some characters twice, as an operator and as an NPC
/// (`char_003_kalts` and `npc_003_kalts`, `char_010_chen` and
/// `npc_010_chen`); counted per id, Amiya would be linked to Kal'tsit twice.
fn fold_npc_twins(ctx: &Ctx, ids: &BTreeSet<String>) -> Vec<String> {
    let people: Vec<ArchivePerson> = ids.iter().map(|id| ctx.person(id)).collect();
    people
        .iter()
        .filter(|p| !(p.npc && people.iter().any(|o| !o.npc && o.name == p.name)))
        .map(|p| p.id.clone())
        .collect()
}

/// Who the network ties each character to. The game draws two kinds of tie
/// between two people on a chart: a LINE (`lineList`, a drawn line between
/// two portraits anywhere on the chart) and a CONNECTION (a cell's
/// `connectionList` naming the neighbour it is bridged to). Both are counted;
/// a connection into a force block or a filler hex names no person and is not.
fn links_by_char(net: &ArchiveNetwork) -> HashMap<&str, BTreeSet<String>> {
    let mut out: HashMap<&str, BTreeSet<String>> = HashMap::new();
    let lines = net
        .groups
        .iter()
        .flat_map(|g| &g.lines)
        .map(|l| (l.a.as_str(), l.b.as_str()));
    let connections = net.groups.iter().flat_map(|g| {
        g.cells.iter().flat_map(|c| {
            c.edges
                .iter()
                .filter_map(|e| e.char_id.as_deref())
                .map(|b| (c.char_id.as_str(), b))
        })
    });
    for (a, b) in lines.chain(connections) {
        if a == b {
            continue;
        }
        out.entry(a).or_default().insert(b.to_owned());
        out.entry(b).or_default().insert(a.to_owned());
    }
    out
}

fn charts_by_char(net: &ArchiveNetwork) -> HashMap<&str, Vec<ArchiveChartRef>> {
    let mut out: HashMap<&str, Vec<ArchiveChartRef>> = HashMap::new();
    for g in &net.groups {
        for cell in &g.cells {
            let home = cell.force_id.as_deref().is_some_and(|f| {
                g.blocks
                    .iter()
                    .any(|b| b.force_id == f && b.home && b.members.contains(&cell.char_id))
            });
            let list = out.entry(cell.char_id.as_str()).or_default();
            if list.iter().any(|c| c.group_id == g.id) {
                continue;
            }
            list.push(ArchiveChartRef {
                group_id: g.id.clone(),
                group_name: g.name.clone(),
                force_id: cell.force_id.clone(),
                home,
            });
        }
    }
    for list in out.values_mut() {
        // Home first, then the network's own chart order.
        list.sort_by_key(|c| !c.home);
    }
    out
}

/// For each operator, who else speaks in the most of the same stories.
fn scene_mates(scenes: &SceneIndex) -> HashMap<&str, Vec<ArchiveSceneMate>> {
    let mut by_story: HashMap<&str, Vec<&str>> = HashMap::new();
    for (char_id, list) in scenes {
        for a in list {
            by_story
                .entry(a.story_id.as_str())
                .or_default()
                .push(char_id.as_str());
        }
    }
    let mut pairs: HashMap<&str, HashMap<&str, u32>> = HashMap::new();
    for chars in by_story.values() {
        for a in chars {
            for b in chars {
                if a != b {
                    *pairs.entry(a).or_default().entry(b).or_default() += 1;
                }
            }
        }
    }
    pairs
        .into_iter()
        .map(|(a, counts)| {
            let mut list: Vec<ArchiveSceneMate> = counts
                .into_iter()
                .map(|(b, n)| ArchiveSceneMate {
                    char_id: b.to_owned(),
                    stories: n,
                })
                .collect();
            list.sort_by(|x, y| {
                y.stories
                    .cmp(&x.stories)
                    .then_with(|| x.char_id.cmp(&y.char_id))
            });
            list.truncate(SCENE_MATES);
            (a, list)
        })
        .collect()
}

/// Every personnel-number prefix, the characters carrying it (operator files
/// and the handbook's NPCs), and the faction most of them show.
fn build_prefixes(ctx: &Ctx, ids: &[String]) -> Vec<ArchivePrefix> {
    let mut holders: BTreeMap<String, Vec<(ArchiveHolder, Option<String>)>> = BTreeMap::new();
    for id in ids {
        let Some(op) = ctx.gd.operators.get(id) else {
            continue;
        };
        if let Some(prefix) = number_prefix(&op.display_number) {
            holders.entry(prefix).or_default().push((
                ArchiveHolder {
                    char_id: id.clone(),
                    number: op.display_number.clone(),
                },
                ctx.card_faction(op),
            ));
        }
    }
    for (id, npc) in &ctx.gd.handbook.npc_dict {
        if ctx.gd.operators.contains_key(id) {
            continue;
        }
        if let Some(prefix) = number_prefix(&npc.display_number) {
            holders.entry(prefix).or_default().push((
                ArchiveHolder {
                    char_id: id.clone(),
                    number: npc.display_number.clone(),
                },
                ctx.person(id).faction_id,
            ));
        }
    }
    let mut out: Vec<ArchivePrefix> = holders
        .into_iter()
        .map(|(prefix, mut list)| {
            list.sort_by(|a, b| {
                a.0.number
                    .cmp(&b.0.number)
                    .then_with(|| a.0.char_id.cmp(&b.0.char_id))
            });
            let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
            for (_, f) in &list {
                if let Some(f) = f {
                    *counts.entry(f.as_str()).or_default() += 1;
                }
            }
            // Most holders wins; a tie goes to the smaller id, so the answer
            // does not depend on map order.
            let top = counts
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                .map(|(f, n)| ((*f).to_owned(), *n));
            ArchivePrefix {
                prefix,
                faction: top.as_ref().map(|(f, _)| ctx.faction_ref(f)),
                faction_holders: top.map_or(0, |(_, n)| n),
                holders: list.into_iter().map(|(h, _)| h).collect(),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.holders
            .len()
            .cmp(&a.holders.len())
            .then_with(|| a.prefix.cmp(&b.prefix))
    });
    out
}

/// Counts of a free-text field, spellings that differ only in case folded
/// together under their most common spelling (`RIM Billiton` into `Rim
/// Billiton`). Most first, then by value.
fn facet<'a>(values: impl Iterator<Item = &'a str>) -> Vec<ArchiveCount> {
    let mut by_key: HashMap<String, HashMap<&str, u32>> = HashMap::new();
    for v in values {
        *by_key
            .entry(v.to_lowercase())
            .or_default()
            .entry(v)
            .or_default() += 1;
    }
    let mut out: Vec<ArchiveCount> = by_key
        .into_values()
        .map(|spellings| {
            let count = spellings.values().sum();
            let value = spellings
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
                .map(|(s, _)| s.to_owned())
                .unwrap_or_default();
            ArchiveCount { value, count }
        })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
    out
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

#[allow(clippy::cast_precision_loss)]
fn distribution(values: &[(&str, f64)]) -> ArchiveDistribution {
    if values.is_empty() {
        return ArchiveDistribution::default();
    }
    let min = values.iter().map(|v| v.1).fold(f64::INFINITY, f64::min);
    let max = values.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().map(|v| v.1).sum::<f64>() / values.len() as f64;
    let at = |target: f64| {
        let mut ids: Vec<String> = values
            .iter()
            .filter(|v| (v.1 - target).abs() < f64::EPSILON)
            .map(|v| v.0.to_owned())
            .collect();
        ids.sort();
        ids
    };
    let width = (max - min) / BUCKETS as f64;
    let mut buckets: Vec<ArchiveBucket> = (0..BUCKETS)
        .map(|i| ArchiveBucket {
            from: round2(min + width * i as f64),
            to: round2(min + width * (i + 1) as f64),
            count: 0,
        })
        .collect();
    for (_, v) in values {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let i = if width > 0.0 {
            (((v - min) / width) as usize).min(BUCKETS - 1)
        } else {
            0
        };
        buckets[i].count += 1;
    }
    ArchiveDistribution {
        counted: count_u32(values.len()),
        min,
        max,
        mean: round2(mean),
        low: at(min),
        high: at(max),
        buckets,
    }
}

fn infection_rows<'a>(
    rows: impl Iterator<Item = (&'a str, InfectionState)>,
) -> Vec<ArchiveInfectionRow> {
    let mut by: HashMap<String, (String, u32, u32, u32)> = HashMap::new();
    for (value, state) in rows {
        let e = by
            .entry(value.to_lowercase())
            .or_insert_with(|| (value.to_owned(), 0, 0, 0));
        e.1 += 1;
        match state {
            InfectionState::Infected => e.2 += 1,
            InfectionState::Clear => e.3 += 1,
            _ => {}
        }
    }
    let mut out: Vec<ArchiveInfectionRow> = by
        .into_values()
        .filter(|e| e.1 >= MIN_GROUP)
        .map(|(value, total, infected, clear)| ArchiveInfectionRow {
            value,
            total,
            infected,
            clear,
        })
        .collect();
    // Highest infected share first.
    out.sort_by(|a, b| {
        (u64::from(b.infected) * u64::from(a.total))
            .cmp(&(u64::from(a.infected) * u64::from(b.total)))
            .then_with(|| b.total.cmp(&a.total))
            .then_with(|| a.value.cmp(&b.value))
    });
    out
}

fn state_name(s: InfectionState) -> &'static str {
    match s {
        InfectionState::Infected => "infected",
        InfectionState::Clear => "clear",
        InfectionState::Unclear => "unclear",
        InfectionState::Unstated => "unstated",
    }
}

/// The spelling most `words` use; a tie goes to the smaller string, so the
/// answer does not depend on map order.
fn most_common<'a>(words: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for w in words {
        *counts.entry(w).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(w, _)| w.to_owned())
}

/// One bar per gender key, named by this server's most common spelling;
/// words with no reading counted as written.
fn gender_counts(lang: Option<Lang>, raws: &[&str]) -> Vec<ArchiveGenderCount> {
    let key_of = |g: &str| lang.and_then(|l| ArchiveGender::read(l.vocab(), g));
    let mut out: Vec<ArchiveGenderCount> = [ArchiveGender::Female, ArchiveGender::Male]
        .into_iter()
        .filter_map(|key| {
            let words: Vec<&str> = raws
                .iter()
                .copied()
                .filter(|g| key_of(g) == Some(key))
                .collect();
            Some(ArchiveGenderCount {
                key: Some(key),
                count: count_u32(words.len()),
                value: most_common(words.into_iter())?,
            })
        })
        .collect();
    out.extend(
        facet(raws.iter().copied().filter(|g| key_of(g).is_none()))
            .into_iter()
            .map(|c| ArchiveGenderCount {
                key: None,
                value: c.value,
                count: c.count,
            }),
    );
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
    out
}

fn build_demographics(
    ctx: &Ctx,
    ids: &[String],
    profiles: &HashMap<&str, ArchiveProfile>,
    prefixes: Vec<ArchivePrefix>,
    lang: Option<Lang>,
) -> ArchiveDemographics {
    let files: Vec<(&str, &ArchiveProfile)> = ids
        .iter()
        .filter_map(|id| profiles.get(id.as_str()).map(|p| (id.as_str(), p)))
        .collect();
    let num = |f: fn(&ArchiveProfile) -> Option<f64>| -> Vec<(&str, f64)> {
        files
            .iter()
            .filter_map(|(id, p)| f(p).map(|v| (*id, v)))
            .collect()
    };

    let mut height_by_race: HashMap<String, (String, Vec<f64>)> = HashMap::new();
    for (_, p) in &files {
        if let (Some(race), Some(h)) = (&p.race, p.height_cm) {
            height_by_race
                .entry(race.to_lowercase())
                .or_insert_with(|| (race.clone(), Vec::new()))
                .1
                .push(h);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let mut height_by_race: Vec<ArchiveGroupMean> = height_by_race
        .into_values()
        .filter(|(_, hs)| count_u32(hs.len()) >= MIN_GROUP)
        .map(|(value, hs)| ArchiveGroupMean {
            counted: count_u32(hs.len()),
            mean: round2(hs.iter().sum::<f64>() / hs.len() as f64),
            value,
        })
        .collect();
    height_by_race.sort_by(|a, b| {
        b.mean
            .total_cmp(&a.mean)
            .then_with(|| a.value.cmp(&b.value))
    });

    let exam = ExamAxis::ALL
        .iter()
        .map(|axis| {
            let mut steps = vec![0u32; ExamStep::ALL.len()];
            for (_, p) in &files {
                if let Some(step) = p.exam.iter().find(|r| r.axis == *axis).and_then(|r| r.step) {
                    steps[usize::from(step)] += 1;
                }
            }
            let rated: u32 = steps.iter().sum();
            ArchiveExamColumn {
                axis: *axis,
                unrecorded: count_u32(files.len()) - rated,
                steps,
            }
        })
        .collect();

    let mut birth_months = vec![0u32; 12];
    let mut days: HashMap<(u8, u8), u32> = HashMap::new();
    let mut dated = 0;
    for (_, p) in &files {
        if let (Some(m), Some(d)) = (p.birth_month, p.birth_day) {
            birth_months[usize::from(m - 1)] += 1;
            *days.entry((m, d)).or_default() += 1;
            dated += 1;
        }
    }
    let mut birth_days: Vec<ArchiveDayCount> = days
        .into_iter()
        .filter(|(_, n)| *n >= SHARED_DAY)
        .map(|((month, day), count)| ArchiveDayCount { month, day, count })
        .collect();
    birth_days.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| (a.month, a.day).cmp(&(b.month, b.day)))
    });

    let mut infection: HashMap<&str, u32> = HashMap::new();
    for (_, p) in &files {
        *infection.entry(state_name(p.infection_state)).or_default() += 1;
    }
    let mut infection: Vec<ArchiveCount> = infection
        .into_iter()
        .map(|(v, count)| ArchiveCount {
            value: v.to_owned(),
            count,
        })
        .collect();
    infection.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));

    let height = distribution(&num(|p| p.height_cm));
    let combat = distribution(&num(|p| p.combat_years));
    let blood = distribution(&num(|p| p.blood_density_ul));
    let assimilation = distribution(&num(|p| p.assimilation_pct));
    let named: Vec<String> = [&height, &combat, &blood, &assimilation]
        .iter()
        .flat_map(|d| d.low.iter().chain(&d.high))
        .chain(
            prefixes
                .iter()
                .flat_map(|p| p.holders.iter().map(|h| &h.char_id)),
        )
        .cloned()
        .collect();

    ArchiveDemographics {
        total: count_u32(files.len()),
        race: facet(files.iter().filter_map(|(_, p)| p.race.as_deref())),
        birthplace: facet(files.iter().filter_map(|(_, p)| p.birthplace.as_deref())),
        // One bar per gender: JP's 女 and 女性 both count as Female.
        gender: gender_counts(
            lang,
            &files
                .iter()
                .filter_map(|(_, p)| p.gender.as_deref())
                .collect::<Vec<_>>(),
        ),
        infection,
        infection_by_birthplace: infection_rows(
            files
                .iter()
                .filter_map(|(_, p)| p.birthplace.as_deref().map(|b| (b, p.infection_state))),
        ),
        infection_by_race: infection_rows(
            files
                .iter()
                .filter_map(|(_, p)| p.race.as_deref().map(|r| (r, p.infection_state))),
        ),
        height,
        combat,
        blood,
        assimilation,
        height_by_race,
        min_group: MIN_GROUP,
        scale: ExamStep::ALL
            .iter()
            .map(|step| ArchiveScaleStep {
                key: *step,
                word: most_common(files.iter().flat_map(|(_, p)| {
                    p.exam
                        .iter()
                        .filter(|r| r.step == Some(step.rank()))
                        .map(|r| r.raw.trim())
                })),
            })
            .collect(),
        exam,
        birth_months,
        birth_days,
        dated,
        manufactured: facet(files.iter().filter_map(|(_, p)| p.manufacturer.as_deref())),
        people: ctx.people(named.iter()),
        prefixes,
    }
}

fn build_hidden(ctx: &Ctx, ids: &[String]) -> ArchiveHiddenFactions {
    let mut secret = Vec::new();
    let mut nations: BTreeMap<String, Vec<ArchiveShownMember>> = BTreeMap::new();
    for id in ids {
        let Some(op) = ctx.gd.operators.get(id) else {
            continue;
        };
        let Some(shown) = ctx.card_faction(op) else {
            continue;
        };
        for h in &op.hidden_factions {
            secret.push(ArchiveSecret {
                char_id: id.clone(),
                shown: ctx.faction_ref(&shown),
                hidden: ArchiveFactionRef {
                    id: h.id.clone(),
                    name: h.name.clone().unwrap_or_else(|| ctx.faction_name(&h.id)),
                },
            });
        }
        if let Some(nation) = real(Some(op.nation_id.as_str()))
            && shown != nation
        {
            nations
                .entry(nation.to_owned())
                .or_default()
                .push(ArchiveShownMember {
                    char_id: id.clone(),
                    shown: ctx.faction_ref(&shown),
                });
        }
    }
    secret.sort_by(|a, b| {
        (
            a.shown.name.as_str(),
            a.hidden.name.as_str(),
            a.char_id.as_str(),
        )
            .cmp(&(
                b.shown.name.as_str(),
                b.hidden.name.as_str(),
                b.char_id.as_str(),
            ))
    });
    let mut nations: Vec<ArchiveNationBehind> = nations
        .into_iter()
        .map(|(nation, mut members)| {
            members.sort_by(|a, b| {
                a.shown
                    .name
                    .cmp(&b.shown.name)
                    .then_with(|| a.char_id.cmp(&b.char_id))
            });
            ArchiveNationBehind {
                nation: ctx.faction_ref(&nation),
                members,
            }
        })
        .collect();
    nations.sort_by(|a, b| {
        b.members
            .len()
            .cmp(&a.members.len())
            .then_with(|| a.nation.name.cmp(&b.nation.name))
    });
    let named: Vec<&String> = secret
        .iter()
        .map(|s| &s.char_id)
        .chain(
            nations
                .iter()
                .flat_map(|n| n.members.iter().map(|m| &m.char_id)),
        )
        .collect();
    ArchiveHiddenFactions {
        people: ctx.people(named),
        secret,
        nations,
    }
}
