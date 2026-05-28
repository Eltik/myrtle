#[derive(Debug, Clone, Copy)]
pub struct StageClear {
    pub state: i16,
    pub complete_times: i32,
    pub practice_times: i32,
}

impl StageClear {
    pub const fn is_cleared(&self) -> bool {
        // state >= 2 means "cleared" in Arknights' dungeon record. Some auto-passed
        // stages (easy_*, mainline cutscene variants) carry state=3 with no
        // completeTimes/practiceTimes; gating on those zeroes them out.
        self.state >= 2
    }

    pub const fn clear_score(&self) -> f64 {
        if !self.is_cleared() {
            0.0
        } else if self.state >= 3 {
            1.0
        } else {
            0.7
        }
    }
}
