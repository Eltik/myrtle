use crate::rotation::dorms::manager_can_sustain;

#[test]
fn a_manager_sustains_drain_up_to_her_own_rate() {
    assert!(manager_can_sustain(2.0, 2.0));
    assert!(manager_can_sustain(2.0, 1.5));
    assert!(manager_can_sustain(2.0, 0.0));
    assert!(manager_can_sustain(2.0, 2.0 + 1e-10));
    assert!(!manager_can_sustain(2.0, 2.1));
    assert!(!manager_can_sustain(2.0, 3.0));
}

#[test]
fn no_swap_rate_sustains_nothing() {
    assert!(!manager_can_sustain(0.0, 0.0));
    assert!(!manager_can_sustain(-1.0, -5.0));
}
