//! Placeholder substitution and number formatting.

use crate::app::i18n::{format, format_into, push_fixed, push_grouped};

#[test]
fn substitutes_named_placeholders() {
    assert_eq!(
        format("EU5 {version} | Build {build}", &[("version", "1.4.8"), ("build", "26547371")]),
        "EU5 1.4.8 | Build 26547371"
    );
}

#[test]
fn placeholders_may_repeat_and_reorder() {
    assert_eq!(format("{b} {a} {b}", &[("a", "1"), ("b", "2")]), "2 1 2");
}

#[test]
fn argument_text_is_not_reparsed() {
    assert_eq!(format("{a}", &[("a", "{b}")]), "{b}");
    assert_eq!(format("{a}", &[("a", "{{")]), "{{");
}

#[test]
fn doubled_braces_are_decoded() {
    assert_eq!(format("{{count}}", &[("count", "5")]), "{count}");
    assert_eq!(format("{{{count}}}", &[("count", "5")]), "{5}");
    assert_eq!(format("a {{ b }} c", &[]), "a { b } c");
}

#[test]
fn unknown_and_unterminated_braces_stay_visible() {
    assert_eq!(format("{missing} x", &[("count", "5")]), "{missing} x");
    assert_eq!(format("open { x", &[]), "open { x");
    assert_eq!(format("lone } x", &[]), "lone } x");
    assert_eq!(format("{a", &[("a", "1")]), "{a");
}

#[test]
fn non_ascii_text_survives() {
    assert_eq!(format("{count} Éléments 한국", &[("count", "3")]), "3 Éléments 한국");
    assert_eq!(format("عدد: {count}", &[("count", "١")]), "عدد: ١");
}

#[test]
fn format_into_appends() {
    let mut out = String::from("> ");
    format_into(&mut out, "{x}", &[("x", "1")]);
    format_into(&mut out, "{x}", &[("x", "2")]);
    assert_eq!(out, "> 12");
}

#[test]
fn empty_template_and_arguments() {
    assert_eq!(format("", &[]), "");
    assert_eq!(format("plain", &[]), "plain");
    assert_eq!(format("{}", &[("", "x")]), "x");
}

fn grouped(value: u64) -> String {
    let mut out = String::new();
    push_grouped(&mut out, value);
    out
}

#[test]
fn groups_digits_by_three() {
    assert_eq!(grouped(0), "0");
    assert_eq!(grouped(7), "7");
    assert_eq!(grouped(999), "999");
    assert_eq!(grouped(1_000), "1,000");
    assert_eq!(grouped(12_345), "12,345");
    assert_eq!(grouped(123_456), "123,456");
    assert_eq!(grouped(1_234_567), "1,234,567");
    assert_eq!(grouped(u64::MAX), "18,446,744,073,709,551,615");
}

#[test]
fn grouping_appends_to_existing_text() {
    let mut out = String::from("n=");
    push_grouped(&mut out, 4_321);
    assert_eq!(out, "n=4,321");
}

fn fixed(value: f32, decimals: usize) -> String {
    let mut out = String::new();
    push_fixed(&mut out, value, decimals);
    out
}

#[test]
fn fixed_decimals_use_a_dot() {
    assert_eq!(fixed(1.0, 2), "1.00");
    assert_eq!(fixed(3.14159, 2), "3.14");
    assert_eq!(fixed(0.126, 2), "0.13");
    assert_eq!(fixed(12.4, 0), "12");
    assert_eq!(fixed(2.0, 1), "2.0");
    assert_eq!(fixed(-1.5, 1), "-1.5");
}

#[test]
fn negative_zero_has_no_sign() {
    assert_eq!(fixed(-0.0, 2), "0.00");
}

#[test]
fn non_finite_values_append_nothing() {
    assert_eq!(fixed(f32::NAN, 2), "");
    assert_eq!(fixed(f32::INFINITY, 2), "");
    assert_eq!(fixed(f32::NEG_INFINITY, 2), "");
}
