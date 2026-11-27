#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use myrtle_app::AppError;
use myrtle_app::services::i18n::{referenced_placeholders, source_hash, validate_message};

fn declared(names: &[&str]) -> serde_json::Value {
    serde_json::json!(names)
}

fn set(names: &[&str]) -> HashSet<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn accepts_a_translation_using_exactly_the_declared_placeholders() {
    assert!(validate_message("{count} operators", &declared(&["count"])).is_ok());
    assert!(validate_message("No operators", &declared(&[])).is_ok());
}

#[test]
fn a_branch_body_is_text_not_a_placeholder() {
    let message = "+ {names}{extra} {count, plural, one {tier} other {tiers}}";
    assert_eq!(
        referenced_placeholders(message).unwrap(),
        set(&["names", "extra", "count"])
    );
    assert!(validate_message(message, &declared(&["count", "extra", "names"])).is_ok());
}

#[test]
fn branch_bodies_of_every_selector_are_text() {
    for message in [
        "{count, plural, one {Delete plan?} other {Delete # plans?}}",
        "{names} and {count, plural, one {its} other {their}} goals are removed.",
        "{scale, select, day {Previous day} week {Previous week} other {Previous month}}",
        "{count, plural, =0 {None} one {# copy} other {# copies}}",
        "{noun, select, healer {Add a healer} other {Add an operator}}",
        "{n, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}",
    ] {
        let found = referenced_placeholders(message).unwrap();
        for phantom in [
            "Delete", "its", "their", "Previous", "None", "Add", "day", "week",
        ] {
            assert!(
                !found.contains(phantom),
                "{message:?} leaked branch text {phantom:?}"
            );
        }
    }
}

#[test]
fn a_nested_argument_inside_a_branch_is_still_collected() {
    let found = referenced_placeholders(
        "{count, plural, one {# of {total}} other {# of {total}, {extra}}}",
    )
    .unwrap();
    assert_eq!(found, set(&["count", "total", "extra"]));
}

#[test]
fn formatted_arguments_keep_their_name_only() {
    for (message, name) in [
        ("{n, number}", "n"),
        ("{ts, date, short}", "ts"),
        ("{ts, time}", "ts"),
    ] {
        assert_eq!(referenced_placeholders(message).unwrap(), set(&[name]));
    }
}

#[test]
fn reads_the_argument_name_out_of_icu_forms() {
    let found =
        referenced_placeholders("{count, plural, one {# operator} other {# operators}}").unwrap();
    assert_eq!(found, set(&["count"]));
}

#[test]
fn rejects_an_invented_placeholder_naming_both_sides() {
    let Err(AppError::Validation(errors)) =
        validate_message("{count} of {total}", &declared(&["count"]))
    else {
        panic!("expected a validation error");
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].field, "value");
    assert_eq!(
        errors[0].message,
        "unknown placeholder(s): {total}. The source string declares: {count}"
    );
}

#[test]
fn rejects_a_dropped_placeholder() {
    let Err(AppError::Validation(errors)) =
        validate_message("many operators", &declared(&["count"]))
    else {
        panic!("expected a validation error");
    };
    assert_eq!(errors[0].message, "missing placeholder(s): {count}");
}

#[test]
fn lists_names_in_name_order_and_says_none_when_nothing_is_declared() {
    let Err(AppError::Validation(errors)) = validate_message("{ab} {a}", &declared(&[])) else {
        panic!("expected a validation error");
    };
    assert_eq!(
        errors[0].message,
        "unknown placeholder(s): {a}, {ab}. The source string declares: none"
    );
}

#[test]
fn rejects_unbalanced_braces() {
    assert!(referenced_placeholders("{count operators").is_err());
    assert!(referenced_placeholders("count} operators").is_err());
    assert!(matches!(
        validate_message("{count", &declared(&["count"])),
        Err(AppError::Validation(_))
    ));
}

#[test]
fn quoted_braces_are_literals_not_arguments() {
    assert!(
        referenced_placeholders("'{'not an arg'}'")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_contraction_does_not_hide_the_arguments_after_it() {
    let found = referenced_placeholders("don't show {count} of {total}").unwrap();
    assert_eq!(found, set(&["count", "total"]));
    assert!(validate_message("don't show {count}", &declared(&["count"])).is_ok());
}

#[test]
fn a_doubled_apostrophe_is_a_literal_and_hides_nothing() {
    assert_eq!(
        referenced_placeholders("Most E2''d: {count}").unwrap(),
        set(&["count"])
    );
}

#[test]
fn source_hash_is_stable_and_distinguishes_edits() {
    assert_eq!(source_hash("Next"), source_hash("Next"));
    assert_ne!(source_hash("Next"), source_hash("Next "));
    assert_eq!(source_hash("Next").len(), 16);
    // sha256("abc") starts ba7816bf8f01cfea
    assert_eq!(source_hash("abc"), "ba7816bf8f01cfea");
}
