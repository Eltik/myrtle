//! Squad Builder analysis: coverage, economy, synergy, readiness, themes and
//! the DPS calculator hand-off for one squad, from the loaded game data alone.
//!
//! Pure functions of `GameData` and a `SquadAnalyzeRequest`; no database, no
//! state, no text parsing: every English text rule was read once at load
//! (`index`). The route runs `analyze_with` through `cpu::run`.

mod build;
mod coverage;
mod economy;
mod handoff;
pub mod index;
pub mod policy;
mod readiness;
pub mod rules;
mod synergy;
mod targets;
mod text;
mod themes;
pub mod types;

use crate::core::gamedata::types::GameData;
use index::{TextFacts, TextIndex};
use types::{SQUAD_SIZE, SUPPORT_SEAT, SquadAnalysis, SquadAnalyzeRequest};

/// The reader's server data and the English readings of its texts, which the
/// panels read through.
pub struct Ctx<'a> {
    pub data: &'a GameData,
    /// EN's text readings, `None` when no English data is loaded.
    pub text: Option<&'a TextIndex>,
}

impl Ctx<'_> {
    /// The readings of a seat's trait pieces read as one, `None` when any
    /// piece has no English reading (an operator EN does not have yet).
    pub fn trait_facts(&self, s: &build::Seat<'_>) -> Option<TextFacts> {
        let text = self.text?;
        let parts: Option<Vec<&TextFacts>> = s.trait_keys.iter().map(|k| text.get(k)).collect();
        parts.map(TextFacts::union)
    }

    pub fn facts(&self, key: &index::TextKey) -> Option<&TextFacts> {
        self.text?.get(key)
    }

    /// Whether every text a seat shows has an English reading.
    pub fn read(&self, s: &build::Seat<'_>) -> bool {
        self.trait_facts(s).is_some()
            && s.talents.iter().all(|t| self.facts(&t.key).is_some())
            && s.skill
                .as_ref()
                .is_none_or(|sk| self.facts(&sk.key).is_some())
    }
}

/// Analyse one squad against `data`, reading its texts through `data`'s own
/// text index: right for the EN data, which is what the tests load.
pub fn analyze(data: &GameData, request: &SquadAnalyzeRequest) -> SquadAnalysis {
    analyze_with(data, Some(data), request)
}

/// Analyse one squad against the reader's `data`, with the English readings
/// from `english` (the EN server's data). Never fails: an unknown `char_id`
/// is skipped, an out-of-range build is clamped, an empty squad yields empty
/// panels, and a seat with no English reading is listed in `unread_seats`
/// with its text-read rows left out.
pub fn analyze_with(
    data: &GameData,
    english: Option<&GameData>,
    request: &SquadAnalyzeRequest,
) -> SquadAnalysis {
    let ctx = Ctx {
        data,
        text: english.map(|e| &e.squad.text).filter(|t| !t.is_empty()),
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
