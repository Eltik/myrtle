#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_release::switches::*;

fn vars(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |key| {
        pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| (*v).to_owned())
    }
}

#[test]
fn nothing_set_is_what_shipped() {
    assert_eq!(Switches::from_vars(|_| None), Switches::SHIPPED);
}

#[test]
fn each_variable_moves_its_own_switch() {
    let s = Switches::from_vars(vars(&[
        ("RELEASE_LAG_WINDOW", " 6 "),
        ("RELEASE_LAG_HALF_LIFE", "0"),
        ("RELEASE_KEEP_UNSOLD_SKINS", "1"),
        ("RELEASE_REVIEWS_ALL_GROUPS", "1"),
        ("RELEASE_TRUST_SKIPPED_TAGS", "0"),
    ]));
    assert_eq!(s.lag_window, 6);
    assert!(s.lag_half_life.abs() < f64::EPSILON);
    assert!(s.keep_unsold_skins && s.reviews_on_every_group && !s.trust_skipped_tags);
}

#[test]
fn a_value_that_does_not_parse_is_the_default() {
    let s = Switches::from_vars(vars(&[
        ("RELEASE_LAG_WINDOW", "ten"),
        ("RELEASE_LAG_HALF_LIFE", "NaN"),
        ("RELEASE_KEEP_UNSOLD_SKINS", "yes"),
        ("RELEASE_TRUST_SKIPPED_TAGS", "off"),
    ]));
    assert_eq!(s, Switches::SHIPPED);
    let off = Switches::from_vars(vars(&[("RELEASE_LAG_HALF_LIFE", "-2")]));
    assert!(off.lag_half_life.abs() < f64::EPSILON);
}
