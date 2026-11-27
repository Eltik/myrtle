use std::collections::HashMap;

use myrtle_gamedata::{
    index::squad::{SquadIndex, TextFacts, TextIndex, TextKey},
    model::GameModel,
};
use myrtle_gamedata_raw::tables::{handbook_team::HandbookTeam, operator::Operator, voice::Voices};

mod build;
mod coverage;
mod economy;
mod handoff;
pub mod policy;
mod readiness;
pub mod rules;
mod synergy;
mod targets;
mod themes;
pub mod types;

pub use types::{SquadAnalysis, SquadAnalyzeRequest};

use types::{SQUAD_SIZE, SUPPORT_SEAT};

#[derive(Debug, Clone, Copy)]
pub struct SquadData<'a> {
    pub operators: &'a HashMap<String, Operator>,
    pub voices: &'a Voices,
    pub profession_names: &'a HashMap<String, String>,
    pub factions: &'a HashMap<String, HandbookTeam>,
    pub squad: &'a SquadIndex,
}

impl<'a> From<&'a GameModel> for SquadData<'a> {
    fn from(model: &'a GameModel) -> Self {
        Self {
            operators: &model.operators,
            voices: &model.voices,
            profession_names: &model.profession_names,
            factions: &model.factions,
            squad: &model.squad,
        }
    }
}

pub(crate) struct Ctx<'a> {
    pub data: &'a SquadData<'a>,
    /// EN's text readings, `None` when no English data is loaded
    pub text: Option<&'a TextIndex>,
}

impl Ctx<'_> {
    /// `None` when any trait piece has no English reading
    pub fn trait_facts(&self, s: &build::Seat<'_>) -> Option<TextFacts> {
        let text = self.text?;
        let parts: Option<Vec<&TextFacts>> = s.trait_keys.iter().map(|k| text.get(k)).collect();
        parts.map(TextFacts::union)
    }

    pub fn facts(&self, key: &TextKey) -> Option<&TextFacts> {
        self.text?.get(key)
    }

    pub fn read(&self, s: &build::Seat<'_>) -> bool {
        self.trait_facts(s).is_some()
            && s.talents.iter().all(|t| self.facts(&t.key).is_some())
            && s.skill
                .as_ref()
                .is_none_or(|sk| self.facts(&sk.key).is_some())
    }
}

#[must_use]
pub fn analyze(
    data: &SquadData<'_>,
    english: Option<&TextIndex>,
    request: &SquadAnalyzeRequest,
) -> SquadAnalysis {
    let ctx = Ctx {
        data,
        text: english.filter(|t| !t.is_empty()),
    };
    let mut seats = Vec::new();
    for (i, m) in request.members.iter().take(SQUAD_SIZE).enumerate() {
        if let Some(b) = m {
            seats.extend(build::resolve(data, i32::try_from(i).unwrap_or(0), b));
        }
    }
    if let Some(b) = &request.support {
        seats.extend(build::resolve(data, SUPPORT_SEAT, b));
    }

    let eco = economy::compute(&ctx, &seats, request);
    let synergy = synergy::compute(&ctx, &seats, &eco.cost_by_seat);
    let coverage = coverage::compute(&ctx, &seats, &eco.economy);
    let readiness = readiness::compute(&seats);
    let themes = themes::compute(&seats, data, request.dub.as_deref());
    let handoff = handoff::compute(&ctx, &seats, &synergy);
    let mut unread_seats: Vec<i32> = seats
        .iter()
        .filter(|s| !ctx.read(s))
        .map(|s| s.seat)
        .collect();
    unread_seats.sort_unstable();

    SquadAnalysis {
        coverage,
        economy: eco.economy,
        synergy,
        readiness,
        themes,
        handoff,
        unread_seats,
    }
}
