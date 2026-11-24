//! Deterministic pseudo-random and adversarial strings: decoding never fails except on the
//! whole-string checks, and the canonical form is bounded and idempotent.

use super::fixture::Loaded;
use crate::app::view_model::view_state::{VIEW_STATE_MAX_LEN, decode, passes_whole_checks};

/// Fragments that exercise every field and the grammar's edge cases.
const FRAGMENTS: [&str; 24] = [
    "~", "*", "!", ".", "q.", "k.", "tp.", "vg.", "cl.!", "rm.!*wheat", "c.", "hr.", "mx.-", "s.",
    "l.", "si.0", "fo.1", "lm.", "!EC", "!8", "name*d", "1.5", "-0", "zz.",
];

/// Characters of the view-state charset.
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.~*!-";

/// xorshift64, enough to spread inputs without a dependency.
struct Generator(u64);

impl Generator {
    fn step(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound).unwrap_or(u64::MAX).max(1);
        usize::try_from(self.step() % bound).unwrap_or(0)
    }

    fn string(&mut self) -> String {
        // Starting with a separator keeps most strings past the whole-string checks.
        let mut text = String::from("s1~");
        let pieces = self.below(80);
        for _ in 0..pieces {
            if self.below(2) == 0 {
                text.push('~');
            }
            if self.below(3) == 0 {
                let index = self.below(CHARSET.len());
                text.push(char::from(CHARSET.get(index).copied().unwrap_or(b'a')));
            } else {
                let index = self.below(FRAGMENTS.len());
                text.push_str(FRAGMENTS.get(index).copied().unwrap_or(""));
            }
        }
        text
    }
}

#[test]
fn random_strings_decode_and_canonicalize_idempotently() {
    let loaded = Loaded::new();
    let mut generator = Generator(0x9E37_79B9_7F4A_7C15);
    for _ in 0..3_000 {
        let text = generator.string();
        let whole = passes_whole_checks(&text);
        assert_eq!(decode(&text).is_ok(), whole, "{text}");
        if let Some(first) = loaded.canonical(&text) {
            assert!(first.len() <= VIEW_STATE_MAX_LEN, "{text}");
            assert!(passes_whole_checks(&first), "{text} -> {first}");
            assert_eq!(loaded.canonical(&first).as_deref(), Some(first.as_str()), "{text}");
        }
    }
}

#[test]
fn adversarial_strings_are_bounded() {
    let loaded = Loaded::new();
    let many_fields = format!("s1{}", "~q.a".repeat(499));
    let many_items = format!("s1~tp.{}", "hills*".repeat(330));
    let many_bangs = format!("s1~q.{}", "!".repeat(1_995));
    let deep_escapes = format!("s1~l.{}", "!41".repeat(600));
    let star_storm = format!("s1~k.{}", "*".repeat(1_990));
    for text in [many_fields, many_items, many_bangs, deep_escapes, star_storm] {
        assert!(text.len() <= VIEW_STATE_MAX_LEN, "{}", text.len());
        let canonical = loaded.canonical(&text);
        assert!(canonical.as_ref().is_some_and(|value| value.len() <= VIEW_STATE_MAX_LEN));
    }
    assert_eq!(loaded.canonical(&format!("s1~q.{}", "a".repeat(1_996))), None);
    assert_eq!(loaded.canonical("s1~k.~k.~k.").as_deref(), Some("s1~k."));
}
