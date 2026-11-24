//! Helpers shared by the theme tests.

use iced::Color;

/// Whether two scalars agree to within float noise.
pub(super) fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

/// Whether every component of two colours agrees to within float noise.
pub(super) fn colors_close(a: Color, b: Color) -> bool {
    close(a.r, b.r) && close(a.g, b.g) && close(a.b, b.b) && close(a.a, b.a)
}

/// WCAG relative luminance of an sRGB colour.
fn luminance(color: Color) -> f32 {
    let linear = |channel: f32| {
        if channel <= 0.03928 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// WCAG contrast ratio between two opaque colours, at least 1.
pub(super) fn contrast(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}
