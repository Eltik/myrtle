#[derive(Debug, Clone, Copy)]
pub struct StageClear {
    pub state: i16,
    /// Upper bound on `state` for a record inferred from surviving mission, medal and
    /// unlock evidence (`stage_evidence`); equals `state` for a battle record.
    pub state_max: i16,
    pub inferred: bool,
    pub complete_times: i32,
    pub practice_times: i32,
}

impl StageClear {
    pub const fn is_cleared(&self) -> bool {
        // state >= 2 = cleared. Some auto-passed stages (easy_*, mainline cutscene
        // variants) have state=3 and no completeTimes/practiceTimes; gating on those
        // zeroes them.
        self.state >= 2
    }

    pub const fn is_three_starred(&self) -> bool {
        self.state >= 3
    }

    pub const fn clear_score(&self) -> f64 {
        if !self.is_cleared() {
            0.0
        } else if self.is_three_starred() {
            1.0
        } else {
            0.7
        }
    }
}
