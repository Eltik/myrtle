//! The statistics the gate needs, with no dependency: Wilson intervals, the
//! paired t-test, a seeded paired bootstrap, and McNemar's exact test.
//! Every function is checked against scipy in the tests.

/// z for a two-sided 95% interval.
pub const Z95: f64 = 1.959_963_984_540_054;

/// Wilson score interval for `successes` of `n`. The honest way to state a
/// rate from a small sample, including zero: 0 of 50 is "at most 7.1%".
#[must_use]
pub fn wilson(successes: usize, n: usize, z: f64) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let (s, n) = (successes as f64, n as f64);
    let p = s / n;
    let d = 1.0 + z * z / n;
    let c = (p + z * z / (2.0 * n)) / d;
    let h = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / d;
    ((c - h).max(0.0), (c + h).min(1.0))
}

#[must_use]
pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = xs.len() as f64;
    xs.iter().sum::<f64>() / n
}

pub struct PairedT {
    pub mean: f64,
    pub t: f64,
    pub df: usize,
    /// Two-sided. 1.0 when every difference is identical (no variance).
    pub p: f64,
}

/// One-sample t-test on paired differences against 0.
#[must_use]
pub fn paired_t(deltas: &[f64]) -> PairedT {
    let n = deltas.len();
    let m = mean(deltas);
    if n < 2 {
        return PairedT {
            mean: m,
            t: 0.0,
            df: 0,
            p: 1.0,
        };
    }
    #[allow(clippy::cast_precision_loss)]
    let nf = n as f64;
    let var = deltas.iter().map(|d| (d - m) * (d - m)).sum::<f64>() / (nf - 1.0);
    let se = (var / nf).sqrt();
    if se == 0.0 {
        let p = if m == 0.0 { 1.0 } else { 0.0 };
        return PairedT {
            mean: m,
            t: if m == 0.0 {
                0.0
            } else {
                f64::INFINITY.copysign(m)
            },
            df: n - 1,
            p,
        };
    }
    let t = m / se;
    let df = n - 1;
    PairedT {
        mean: m,
        t,
        df,
        p: student_t_two_sided(t, df),
    }
}

/// Two-sided p for Student's t: `I_{df/(df+t^2)}(df/2, 1/2)`.
#[must_use]
pub fn student_t_two_sided(t: f64, df: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let v = df as f64;
    reg_inc_beta(v / 2.0, 0.5, v / (v + t * t))
}

/// Percentile bootstrap 95% CI on the mean of paired differences, seeded so a
/// run is reproducible bit for bit.
#[must_use]
pub fn paired_bootstrap_ci(deltas: &[f64], iters: usize, seed: u64) -> (f64, f64) {
    let n = deltas.len();
    if n == 0 || iters == 0 {
        return (0.0, 0.0);
    }
    let mut rng = SplitMix64(seed);
    let mut means: Vec<f64> = (0..iters)
        .map(|_| {
            let mut s = 0.0;
            for _ in 0..n {
                s += deltas[rng.below(n)];
            }
            #[allow(clippy::cast_precision_loss)]
            let m = s / n as f64;
            m
        })
        .collect();
    means.sort_by(f64::total_cmp);
    let at = |q: f64| {
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let i = ((q * iters as f64).floor() as usize).min(iters - 1);
        means[i]
    };
    (at(0.025), at(0.975))
}

/// McNemar's exact test on discordant pairs: `b` queries only the baseline
/// hit, `c` only the candidate hit. Two-sided binomial p at 0.5.
#[must_use]
pub fn mcnemar_exact(b: usize, c: usize) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let k = b.min(c);
    // Sum C(n, i) / 2^n for i <= k in log space.
    #[allow(clippy::cast_precision_loss)]
    let ln2n = n as f64 * std::f64::consts::LN_2;
    let mut ln_term = -ln2n; // i = 0
    let mut total = ln_term.exp();
    for i in 0..k {
        #[allow(clippy::cast_precision_loss)]
        let ratio = (n - i) as f64 / (i + 1) as f64;
        ln_term += ratio.ln();
        total += ln_term.exp();
    }
    (2.0 * total).min(1.0)
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in `0..n` by rejection, so no index is favoured.
    fn below(&mut self, n: usize) -> usize {
        let n = n as u64;
        let zone = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next();
            if x < zone {
                #[allow(clippy::cast_possible_truncation)]
                return (x % n) as usize;
            }
        }
    }
}

/// Regularized incomplete beta `I_x(a, b)` by Lentz's continued fraction
/// (Numerical Recipes, `betacf`), with the symmetry swap for convergence.
fn reg_inc_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let ln_front = ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln();
    if x < (a + 1.0) / (a + b + 2.0) {
        ln_front.exp() * beta_cf(a, b, x) / a
    } else {
        1.0 - ln_front.exp() * beta_cf(b, a, 1.0 - x) / b
    }
}

fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    const TINY: f64 = 1e-300;
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=300 {
        let m = f64::from(m);
        let m2 = 2.0 * m;
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        h *= d * c;
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < TINY {
            d = TINY;
        }
        c = 1.0 + aa / c;
        if c.abs() < TINY {
            c = TINY;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-15 {
            break;
        }
    }
    h
}

/// Lanczos approximation (g = 7, n = 9), accurate to ~1e-15 for x > 0.
fn ln_gamma(x: f64) -> f64 {
    const G: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = G[0];
    let t = x + 7.5;
    for (i, g) in G.iter().enumerate().skip(1) {
        #[allow(clippy::cast_precision_loss)]
        let fi = i as f64;
        a += g / (x + fi);
    }
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn student_t_matches_scipy() {
        // scipy: 2 * stats.t.sf(t, df)
        assert!(close(
            student_t_two_sided(2.0, 10),
            0.073_388_034_770_740_37,
            1e-10
        ));
        assert!(close(
            student_t_two_sided(0.5, 3),
            0.651_447_964_848_151,
            1e-10
        ));
        assert!(close(
            student_t_two_sided(3.3, 149),
            0.001_209_730_904_156_325_1,
            1e-10
        ));
    }

    #[test]
    fn paired_t_matches_scipy_ttest_1samp() {
        let d = [0.1, -0.05, 0.2, 0.0, 0.15, 0.05, -0.1, 0.3];
        let r = paired_t(&d);
        assert!(close(r.t, 1.721_892_064_184_557, 1e-10));
        assert!(close(r.p, 0.128_761_713_218_269_99, 1e-10));
        assert_eq!(r.df, 7);
    }

    #[test]
    fn mcnemar_matches_scipy_binomtest() {
        assert!(close(mcnemar_exact(1, 9), 0.021_484_375, 1e-12));
        assert!(close(mcnemar_exact(3, 3), 1.0, 1e-12));
        assert!(close(mcnemar_exact(0, 5), 0.0625, 1e-12));
        assert!(close(
            mcnemar_exact(20, 40),
            0.013_489_293_731_191_86,
            1e-10
        ));
    }

    #[test]
    fn wilson_matches_the_closed_form() {
        let (lo, hi) = wilson(35, 50, Z95);
        assert!(
            close(lo, 0.562_496_495_355_466, 1e-10) && close(hi, 0.808_964_464_991_190_6, 1e-10)
        );
        // The spec's "0 leaks in 50 probes is at most 7.1%".
        assert!(close(wilson(0, 50, Z95).1, 0.071_347_599_133_358_72, 1e-10));
        assert!(close(wilson(0, 150, Z95).1, 0.024_970_244_368_076_6, 1e-10));
    }

    #[test]
    fn bootstrap_is_seeded_and_brackets_the_mean() {
        let d = [0.1, -0.05, 0.2, 0.0, 0.15, 0.05, -0.1, 0.3];
        let a = paired_bootstrap_ci(&d, 10_000, 7);
        assert_eq!(
            a,
            paired_bootstrap_ci(&d, 10_000, 7),
            "same seed, same interval"
        );
        assert!(a.0 < mean(&d) && mean(&d) < a.1);
        let flat = paired_bootstrap_ci(&[0.0; 20], 1_000, 1);
        assert_eq!(flat, (0.0, 0.0));
    }
}
