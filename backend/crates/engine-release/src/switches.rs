pub const DEFAULT_WINDOW: usize = 10;

/// Recency half-life of the lag window
pub const DEFAULT_HALF_LIFE: f64 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Switches {
    /// `RELEASE_LAG_WINDOW`
    pub lag_window: usize,
    /// `RELEASE_LAG_HALF_LIFE`
    pub lag_half_life: f64,
    /// `RELEASE_KEEP_UNSOLD_SKINS=1`
    pub keep_unsold_skins: bool,
    /// `RELEASE_REVIEWS_ALL_GROUPS=1`
    pub reviews_on_every_group: bool,
    /// `RELEASE_TRUST_SKIPPED_TAGS=0`
    pub trust_skipped_tags: bool,
}

impl Switches {
    pub const SHIPPED: Self = Self {
        lag_window: DEFAULT_WINDOW,
        lag_half_life: DEFAULT_HALF_LIFE,
        keep_unsold_skins: false,
        reviews_on_every_group: false,
        trust_skipped_tags: true,
    };

    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            lag_window: var("RELEASE_LAG_WINDOW").map_or(DEFAULT_WINDOW, |v| {
                v.trim().parse().unwrap_or(DEFAULT_WINDOW)
            }),
            lag_half_life: var("RELEASE_LAG_HALF_LIFE")
                .map_or(DEFAULT_HALF_LIFE, |v| parse_half_life(&v)),
            keep_unsold_skins: var("RELEASE_KEEP_UNSOLD_SKINS").as_deref() == Some("1"),
            reviews_on_every_group: var("RELEASE_REVIEWS_ALL_GROUPS").as_deref() == Some("1"),
            trust_skipped_tags: var("RELEASE_TRUST_SKIPPED_TAGS").as_deref() != Some("0"),
        }
    }
}

#[must_use]
pub fn parse_half_life(v: &str) -> f64 {
    match v.trim().parse::<f64>() {
        Ok(h) if h.is_finite() => {
            if h > 0.0 {
                h
            } else {
                0.0
            }
        }
        _ => DEFAULT_HALF_LIFE,
    }
}
