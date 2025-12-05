use crate::core::local::types::trust::Favor;

/// Calculate trust level from favor points
/// Returns the index (trust level) where favor_point >= trust
pub fn calculate_trust(favor_table: &Favor, trust: i32) -> Option<usize> {
    favor_table
        .favor_frames
        .iter()
        .position(|frame| frame.data.favor_point >= trust)
}

/// Get detailed trust info for a given favor point value
pub fn get_trust_info(favor_table: &Favor, trust: i32) -> Option<TrustInfo> {
    let index = calculate_trust(favor_table, trust)?;
    let frame = favor_table.favor_frames.get(index)?;

    Some(TrustInfo {
        level: index,
        favor_point: frame.data.favor_point,
        percent: frame.data.percent,
        battle_phase: frame.data.battle_phase,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustInfo {
    pub level: usize,
    pub favor_point: i32,
    pub percent: f64,
    pub battle_phase: i32,
}
