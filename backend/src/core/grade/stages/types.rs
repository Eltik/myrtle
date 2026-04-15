#[derive(Debug, Clone, Copy)]
pub struct StageClear {
    pub state: i16,
    pub complete_times: i32,
    pub practice_times: i32,
}

impl StageClear {
    pub fn is_cleared(&self) -> bool {
        self.state >= 2 && (self.complete_times > 0 || self.practice_times > 0)
    }

    pub fn clear_score(&self) -> f64 {
        if !self.is_cleared() {
            0.0
        } else if self.state >= 3 {
            1.0
        } else {
            0.7
        }
    }
}
