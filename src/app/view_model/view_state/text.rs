//! Text escaping: bytes outside `A-Z a-z 0-9 _ - .` become `!` and two uppercase hex digits.
//!
//! The escape alphabet keeps `~`, `*` and `!` free for the grammar, so a lone `!` can mean the
//! missing value without colliding with any encoded text.

/// Uppercase hexadecimal digits used by the encoder.
const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Whether a byte is written as itself.
#[must_use]
pub(super) const fn is_raw(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

/// Appends the escaped UTF-8 bytes of `text` to `out`.
pub(crate) fn escape_into(out: &mut String, text: &str) {
    for byte in text.bytes() {
        if is_raw(byte) {
            out.push(char::from(byte));
        } else {
            out.push('!');
            push_hex_digit(out, byte >> 4);
            push_hex_digit(out, byte & 0x0F);
        }
    }
}

/// Decodes an escaped value of at most `max_bytes` decoded bytes. Returns `None` for an
/// incomplete escape, a byte sequence that is not UTF-8, or an over-long result. Either hex case
/// and escaped raw bytes are accepted.
#[must_use]
pub(crate) fn unescape(value: &str, max_bytes: usize) -> Option<String> {
    let input = value.as_bytes();
    // Every decoded byte needs at least one input byte, so this bounds the allocation.
    let mut bytes = Vec::with_capacity(input.len().min(max_bytes));
    let mut index = 0;
    while let Some(&byte) = input.get(index) {
        let decoded = if byte == b'!' {
            let high = input.get(index + 1).copied().and_then(hex_value)?;
            let low = input.get(index + 2).copied().and_then(hex_value)?;
            index += 3;
            (high << 4) | low
        } else if is_raw(byte) {
            index += 1;
            byte
        } else {
            return None;
        };
        if bytes.len() >= max_bytes {
            return None;
        }
        bytes.push(decoded);
    }
    String::from_utf8(bytes).ok()
}

/// Whether decoded text contains U+0000 to U+001F or U+007F, which the search field rejects.
#[must_use]
pub(super) fn has_control(text: &str) -> bool {
    text.bytes().any(|byte| byte < 0x20 || byte == 0x7F)
}

/// Value of one hexadecimal digit in either case.
const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Appends the uppercase digit for a nibble; values above 15 cannot occur after masking.
fn push_hex_digit(out: &mut String, nibble: u8) {
    match HEX.get(usize::from(nibble)) {
        Some(&digit) => out.push(char::from(digit)),
        None => out.push('0'),
    }
}
