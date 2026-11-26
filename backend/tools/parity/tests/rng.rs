use myrtle_parity::rng::{SplitMix64, fnv1a64, sample_indices};

#[test]
fn splitmix_sequence_is_pinned() {
    let mut rng = SplitMix64::new(0);
    assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdaf); // published seed-0 values; a change reshuffles every corpus
    assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4);
}

#[test]
fn fnv_is_pinned() {
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn sample_is_distinct_sorted_and_deterministic() {
    let a = sample_indices(1_000_000, 50, &mut SplitMix64::new(7));
    let b = sample_indices(1_000_000, 50, &mut SplitMix64::new(7));
    let c = sample_indices(1_000_000, 50, &mut SplitMix64::new(8));
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.len(), 50);
    assert!(a.windows(2).all(|w| w[0] < w[1]));
    assert!(a.iter().all(|&i| i < 1_000_000));
}

#[test]
fn sample_larger_than_population_returns_all() {
    assert_eq!(
        sample_indices(4, 10, &mut SplitMix64::new(1)),
        vec![0, 1, 2, 3]
    );
    assert!(sample_indices(0, 3, &mut SplitMix64::new(1)).is_empty());
}

#[test]
fn below_stays_in_range() {
    let mut rng = SplitMix64::new(42);
    for bound in [1_u64, 2, 3, 7, 1000] {
        for _ in 0..200 {
            assert!(rng.below(bound) < bound);
        }
    }
}
