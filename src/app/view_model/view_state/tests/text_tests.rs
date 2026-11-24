//! Escaping and unescaping.

use crate::app::view_model::view_state::{MAX_SEARCH_BYTES, escape_into, unescape};

fn escaped(text: &str) -> String {
    let mut out = String::new();
    escape_into(&mut out, text);
    out
}

#[test]
fn escapes_every_byte_outside_the_raw_set() {
    assert_eq!(escaped("new york"), "new!20york");
    assert_eq!(escaped("서울"), "!EC!84!9C!EC!9A!B8");
    assert_eq!(escaped("a~b*c!d"), "a!7Eb!2Ac!21d");
    assert_eq!(escaped("Az09_-."), "Az09_-.");
    assert_eq!(escaped(""), "");
}

#[test]
fn escape_appends_without_clearing() {
    let mut out = String::from("q.");
    escape_into(&mut out, "é");
    assert_eq!(out, "q.!C3!A9");
}

#[test]
fn unescape_accepts_either_hex_case_and_escaped_raw_bytes() {
    assert_eq!(unescape("!ec!84!9c!EC!9A!B8", 256).as_deref(), Some("서울"));
    assert_eq!(unescape("!61b", 256).as_deref(), Some("ab"));
    assert_eq!(unescape("", 256).as_deref(), Some(""));
}

#[test]
fn unescape_rejects_malformed_escapes_and_bytes() {
    assert_eq!(unescape("!2", 256), None);
    assert_eq!(unescape("!", 256), None);
    assert_eq!(unescape("a!G0", 256), None);
    assert_eq!(unescape("!FF", 256), None);
    assert_eq!(unescape("!C3", 256), None);
    assert_eq!(unescape("a~b", 256), None);
    assert_eq!(unescape("a*b", 256), None);
    assert_eq!(unescape("a b", 256), None);
}

#[test]
fn unescape_enforces_the_decoded_byte_limit() {
    let exact = "a".repeat(MAX_SEARCH_BYTES);
    let over = "a".repeat(MAX_SEARCH_BYTES + 1);
    assert_eq!(unescape(&exact, MAX_SEARCH_BYTES).map(|text| text.len()), Some(MAX_SEARCH_BYTES));
    assert_eq!(unescape(&over, MAX_SEARCH_BYTES), None);
    // Three escaped bytes count as one decoded byte each.
    assert_eq!(unescape("!EC!84!9C", 3).as_deref(), Some("서"));
    assert_eq!(unescape("!EC!84!9C", 2), None);
}

#[test]
fn escape_and_unescape_round_trip() {
    for text in ["river", "new york", "서울", "東京", "évora", "~*!", "a.b-c_d", "\u{1F600}"] {
        assert_eq!(unescape(&escaped(text), 1_024).as_deref(), Some(text), "{text}");
    }
}
