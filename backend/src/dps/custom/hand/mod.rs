//! Hand-written DPS / HPS formulas, consulted BEFORE the generated dispatch.
//!
//! The generated layer (`generated*.rs`, `dispatch*.rs`, `config/operator_formulas.json`,
//! `config/heal_formulas.json`) is rewritten wholesale by `generate-dps` from the
//! upstream Python repo, which updates rarely. An operator it does not cover, or
//! covers wrongly, lives here instead:
//!
//! - one file per operator, exposing `dps` and / or `hps` with the generated
//!   signature (`fn(&OperatorUnit, &EnemyStats) -> Option<f64>` /
//!   `Option<HpsResult>`), so the engine runs it through the same path: zones,
//!   secondary targets, cycle v2 and conditionals apply unchanged;
//! - one row in `DPS` / `HPS` below (the registry);
//! - one entry in `formulas.json` (`dps` / `hps` section): the same metadata shape as
//!   `operator_formulas.json`, merged over the generated table by
//!   `engine::load_formulas` / `load_heal_formulas`.
//!
//! An id registered here wins over a generated formula for the same id. The
//! generator never writes under `custom/hand/` (it refuses to, see
//! `generate_dps.rs`), and it prints every id that is in both layers so a hand
//! formula that upstream now covers gets reviewed rather than silently shadowing
//! the transpiled one. How to add an operator: `backend/DPS_CALCULATOR.md`.

use super::super::engine::HpsResult;
use super::super::operator_unit::{EnemyStats, OperatorUnit};

mod angelina_alter;

pub type DpsFn = fn(&OperatorUnit, &EnemyStats) -> Option<f64>;
pub type HpsFn = fn(&OperatorUnit, &EnemyStats) -> Option<HpsResult>;

/// Metadata for the hand formulas, `{"dps": {id: OperatorFormula}, "hps": {...}}`.
pub const FORMULAS_JSON: &str = include_str!("formulas.json");

/// Hand DPS formulas by operator id.
pub const DPS: &[(&str, DpsFn)] = &[("char_1015_aglna2", angelina_alter::dps)];

/// Hand HPS formulas by operator id. Called with a zero-DEF / zero-RES enemy, the
/// way the generated HPS dispatch calls its functions.
pub const HPS: &[(&str, HpsFn)] = &[];

pub fn dps(op_id: &str) -> Option<DpsFn> {
    DPS.iter().find(|(id, _)| *id == op_id).map(|(_, f)| *f)
}

pub fn hps(op_id: &str) -> Option<HpsFn> {
    HPS.iter().find(|(id, _)| *id == op_id).map(|(_, f)| *f)
}
