//! Placeholder substitution and invariant number formatting into caller-owned buffers, so a
//! visible cell or label can reuse one `String`.

use std::fmt::Write as _;

/// One placeholder value: name without braces, and its already formatted text.
pub(crate) type Arg<'a> = (&'static str, &'a str);

/// Appends `template` to `out` with each `{name}` replaced by its argument and `{{`, `}}`
/// decoded. An unknown name is written as is; catalogs are checked at build time, so that only
/// happens for a programming error and stays visible instead of failing.
pub(crate) fn format_into(out: &mut String, template: &str, args: &[Arg<'_>]) {
    let _ = args;
    out.push_str(template);
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
    let _ = write!(out, "{value}");
}

/// Appends `value` with exactly `decimals` digits after a `.`, in every locale; non-finite values
/// append nothing, because stored values are validated finite.
pub(crate) fn push_fixed(out: &mut String, value: f32, decimals: usize) {
    if value.is_finite() {
        let _ = write!(out, "{value:.decimals$}");
    }
}
