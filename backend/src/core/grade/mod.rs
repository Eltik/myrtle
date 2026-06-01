pub mod base;
pub mod calculate;
pub mod grade_medals;
pub mod grade_operators;
pub mod grade_roguelike;
pub mod sandbox;
pub mod stages;

/// A scoring dimension: `(weight, score)` where score is 0.0-1.0.
pub(crate) type Dimension = (f64, f64);

/// The weight-normalized average of scoring dimensions, or 0.0 when total weight
/// is non-positive.
pub(crate) fn weighted_average(dims: &[Dimension]) -> f64 {
    let total_weight: f64 = dims.iter().map(|(w, _)| w).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    dims.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight
}
