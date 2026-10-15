pub mod base;
pub mod calculate;
pub mod grade_medals;
pub mod grade_operators;
pub mod grade_roguelike;
pub mod sandbox;
pub mod stages;

/// A scoring dimension: `(weight, score)` where score is 0.0-1.0.
pub(crate) type Dimension = (f64, f64);

/// Weight-normalized average; 0.0 when total weight is non-positive.
pub(crate) fn weighted_average(dims: &[Dimension]) -> f64 {
    let total_weight: f64 = dims.iter().map(|(w, _)| w).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }
    dims.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight
}

#[cfg(test)]
mod tests {
    use super::weighted_average;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn averages_by_weight() {
        assert!(close(weighted_average(&[(30.0, 1.0), (10.0, 0.0)]), 0.75));
        assert!(close(weighted_average(&[(1.0, 0.2), (1.0, 0.4)]), 0.3));
    }

    #[test]
    fn a_single_dimension_is_its_own_score() {
        assert!(close(weighted_average(&[(25.0, 0.6)]), 0.6));
    }

    #[test]
    fn non_positive_total_weight_reads_zero() {
        assert!(close(weighted_average(&[]), 0.0));
        assert!(close(weighted_average(&[(0.0, 1.0)]), 0.0));
        assert!(close(weighted_average(&[(-2.0, 1.0), (1.0, 1.0)]), 0.0));
    }
}
