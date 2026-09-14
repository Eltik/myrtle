use crate::core::{
    gamedata::types::GameData,
    release::{LagModel, YearlyModel, estimate},
};

const LOOKBACK_GRACE_SECS: i64 = 60 * 86_400;

pub struct Models {
    pub pairs: Vec<estimate::ActivityPair>,
    pub general: LagModel,
    pub yearly: YearlyModel,
}

impl Models {
    pub fn build(cn: &GameData, en: &GameData, window: usize) -> Self {
        let all = estimate::activity_pairs(cn, en);
        let types = estimate::yearly_types(&all);
        let (general_pairs, yearly_pairs) = estimate::split_yearly(&all, &types);
        Self {
            general: estimate::build_lag_model(&general_pairs, window),
            yearly: YearlyModel {
                types,
                model: estimate::build_yearly_model(&yearly_pairs),
            },
            pairs: general_pairs,
        }
    }

    pub fn is_yearly(&self, activity_type: &str) -> bool {
        self.yearly.types.iter().any(|t| t == activity_type)
    }

    pub fn lookback_secs(&self) -> i64 {
        let longest = if self.yearly.model.n > 0 {
            self.yearly.model.median_days.max(self.general.p75_days)
        } else {
            self.general.p75_days
        };
        (longest * 86_400.0).round() as i64 + LOOKBACK_GRACE_SECS
    }

    pub fn for_type(&self, activity_type: &str) -> &LagModel {
        if self.is_yearly(activity_type) {
            &self.yearly.model
        } else {
            &self.general
        }
    }
}
