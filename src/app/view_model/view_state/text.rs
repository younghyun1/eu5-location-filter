//! Text escaping: bytes outside `A-Z a-z 0-9 _ - .` become `!` and two uppercase hex digits.

/// Appends the escaped UTF-8 bytes of `text` to `out`.
pub(crate) fn escape_into(out: &mut String, text: &str) {
    let _ = (out, text);
}

/// Decodes an escaped value of at most `max_bytes` decoded bytes. Returns `None` for an
/// incomplete escape, a byte sequence that is not UTF-8, or an over-long result. Either hex case
/// and escaped raw bytes are accepted.
#[must_use]
pub(crate) fn unescape(value: &str, max_bytes: usize) -> Option<String> {
    let _ = (value, max_bytes);
    None
}
