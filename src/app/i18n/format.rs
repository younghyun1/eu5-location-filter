//! Placeholder substitution and invariant number formatting into caller-owned buffers, so a
//! visible cell or label can reuse one `String`.

use std::fmt::Write as _;

/// One placeholder value: name without braces, and its already formatted text.
pub(crate) type Arg<'a> = (&'static str, &'a str);

/// Appends `template` to `out` with each `{name}` replaced by its argument and `{{`, `}}`
/// decoded. An unknown name is written as is; catalogs are checked at build time, so that only
/// happens for a programming error and stays visible instead of failing. A `{` without a closing
/// `}` and a single `}` are also written as is.
pub(crate) fn format_into(out: &mut String, template: &str, args: &[Arg<'_>]) {
    let bytes = template.as_bytes();
    // Start of the literal run not yet copied. Braces are ASCII, so every index used to cut the
    // template lies on a character boundary.
    let mut start = 0;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let doubled = bytes.get(index + 1) == Some(&byte);
        match byte {
            b'{' | b'}' if doubled => {
                // Keep the first brace of the pair and skip the second.
                push_slice(out, template, start, index + 1);
                index += 2;
                start = index;
            }
            b'{' => match placeholder(template, index, args) {
                Some((value, next)) => {
                    push_slice(out, template, start, index);
                    out.push_str(value);
                    index = next;
                    start = next;
                }
                None => index += 1,
            },
            _ => index += 1,
        }
    }
    push_slice(out, template, start, template.len());
}

/// Value and end offset of the `{name}` that opens at `open`, when the name is an argument.
fn placeholder<'a>(template: &str, open: usize, args: &[Arg<'a>]) -> Option<(&'a str, usize)> {
    let rest = template.get(open + 1..)?;
    let length = rest.find('}')?;
    let name = rest.get(..length)?;
    let (_, value) = args.iter().find(|(known, _)| *known == name)?;
    Some((*value, open + 1 + length + 1))
}

fn push_slice(out: &mut String, template: &str, start: usize, end: usize) {
    if let Some(slice) = template.get(start..end) {
        out.push_str(slice);
    }
}

/// `format_into` into a new string, for text built once per state change rather than per frame.
#[must_use]
pub(crate) fn format(template: &str, args: &[Arg<'_>]) -> String {
    let mut out = String::with_capacity(template.len());
    format_into(&mut out, template, args);
    out
}

/// Appends `value` with `,` between groups of three digits, in every locale.
pub(crate) fn push_grouped(out: &mut String, value: u64) {
    // u64::MAX has 20 digits; they are collected least significant first.
    let mut digits = [0_u8; 20];
    let mut length = 0;
    let mut rest = value;
    loop {
        if let Some(slot) = digits.get_mut(length) {
            // The remainder is below ten, so the conversion cannot fail.
            *slot = u8::try_from(rest % 10).unwrap_or(0);
        }
        length += 1;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    for position in (0..length).rev() {
        let digit = digits.get(position).copied().unwrap_or(0);
        out.push(char::from(b'0' + digit));
        if position > 0 && position % 3 == 0 {
            out.push(',');
        }
    }
}

/// Appends `value` with exactly `decimals` digits after a `.`, in every locale; non-finite values
/// append nothing, because stored values are validated finite.
pub(crate) fn push_fixed(out: &mut String, value: f32, decimals: usize) {
    if value.is_finite() {
        // A negative zero would print as "-0.00"; the sign carries no meaning here.
        let value = if value == 0.0 { 0.0 } else { value };
        // Writing to a String cannot fail.
        let _ = write!(out, "{value:.decimals$}");
    }
}
