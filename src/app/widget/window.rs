//! Pure windowing math for fixed-height rows and variable-width columns, shared by the result
//! grid and the checklist. All values are logical pixels; offsets are measured from the inline
//! start of the content, and `mirror` maps them to screen x for right-to-left layouts.
//!
//! Every function accepts any `f32`, including NaN and infinities, without panicking: a
//! non-finite or non-positive size yields an empty or neutral result instead. `f32::clamp` is
//! avoided because it panics when its bounds are inverted or NaN.

/// Half-open range of row indices.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct VisibleRange {
    /// First row drawn.
    pub(crate) start: usize,
    /// One past the last row drawn.
    pub(crate) end: usize,
}

impl VisibleRange {
    /// Number of rows in the range.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range is empty.
    #[must_use]
    pub(crate) const fn is_empty(self) -> bool {
        self.end <= self.start
    }
}

/// Scrollbar thumb along its track.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Thumb {
    /// Offset of the thumb from the track start.
    pub(crate) start: f32,
    /// Thumb length, at least the minimum length.
    pub(crate) length: f32,
}

/// Whether a size is usable as a divisor or extent.
fn positive(value: f32) -> bool {
    value.is_finite() && value > 0.0
}

/// A finite offset, or zero.
fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

/// Converts a row position to an index: NaN and non-positive values give zero, and values past
/// `usize::MAX` (including positive infinity) saturate, which is what `as` does for floats.
fn to_index(value: f32) -> usize {
    if value.is_nan() || value <= 0.0 {
        0
    } else {
        value as usize
    }
}

/// Rows intersecting `offset..offset + viewport`, clamped to `count`; row cost is independent
/// of `count`.
#[must_use]
pub(crate) fn visible_rows(
    offset: f32,
    viewport: f32,
    row_height: f32,
    count: usize,
) -> VisibleRange {
    if count == 0 || !positive(viewport) || !positive(row_height) {
        return VisibleRange::default();
    }
    let offset = finite_or_zero(offset).max(0.0);
    let start = to_index((offset / row_height).floor()).min(count);
    let end = to_index(((offset + viewport) / row_height).ceil())
        .min(count)
        .max(start);
    VisibleRange { start, end }
}

/// Largest scroll offset; zero when the content fits or an input is not finite.
#[must_use]
pub(crate) fn max_offset(content: f32, viewport: f32) -> f32 {
    if !content.is_finite() || !viewport.is_finite() {
        return 0.0;
    }
    let overflow = content - viewport.max(0.0);
    if overflow.is_finite() && overflow > 0.0 {
        overflow
    } else {
        0.0
    }
}

/// Offset clamped to `0..=max_offset`; a non-finite offset becomes zero.
#[must_use]
pub(crate) fn clamp_offset(offset: f32, content: f32, viewport: f32) -> f32 {
    if offset.is_finite() {
        offset.max(0.0).min(max_offset(content, viewport))
    } else {
        0.0
    }
}

/// Row under a y coordinate relative to the body top, or `None` past the last row.
#[must_use]
pub(crate) fn row_at(y: f32, offset: f32, row_height: f32, count: usize) -> Option<usize> {
    if count == 0 || !positive(row_height) || !y.is_finite() || y < 0.0 {
        return None;
    }
    let position = y + finite_or_zero(offset);
    if position < 0.0 {
        return None;
    }
    let index = to_index((position / row_height).floor());
    (index < count).then_some(index)
}

/// Smallest offset change that shows `row` fully.
#[must_use]
pub(crate) fn scroll_into_view(offset: f32, row: usize, row_height: f32, viewport: f32) -> f32 {
    if !positive(row_height) {
        return finite_or_zero(offset);
    }
    span_into_view(offset, row as f32 * row_height, row_height, viewport)
}

/// Smallest offset change that shows the span `start..start + width` fully. A span longer than
/// the viewport aligns its start with the viewport start.
#[must_use]
pub(crate) fn span_into_view(offset: f32, start: f32, width: f32, viewport: f32) -> f32 {
    let offset = finite_or_zero(offset);
    if !start.is_finite() || !width.is_finite() || !viewport.is_finite() || viewport <= 0.0 {
        return offset;
    }
    let width = width.max(0.0);
    if start < offset || width >= viewport {
        start
    } else if start + width > offset + viewport {
        start + width - viewport
    } else {
        offset
    }
}

/// Rows a Page Up or Page Down moves: the visible row count minus one, at least one.
#[must_use]
pub(crate) fn page_rows(viewport: f32, row_height: f32) -> usize {
    if !positive(viewport) || !positive(row_height) {
        return 1;
    }
    to_index((viewport / row_height).floor())
        .saturating_sub(1)
        .max(1)
}

/// Thumb length for a track, or `None` when no scrollbar is drawn.
fn thumb_length(content: f32, viewport: f32, track: f32, min_length: f32) -> Option<f32> {
    if !positive(content) || !positive(viewport) || !positive(track) || content <= viewport {
        return None;
    }
    let min_length = if positive(min_length) {
        min_length
    } else {
        0.0
    };
    Some((viewport / content * track).max(min_length).min(track))
}

/// Thumb for a track, or `None` when the content fits and no scrollbar is drawn.
#[must_use]
pub(crate) fn thumb(
    offset: f32,
    content: f32,
    viewport: f32,
    track: f32,
    min_length: f32,
) -> Option<Thumb> {
    let length = thumb_length(content, viewport, track, min_length)?;
    let travel = track - length;
    let max = max_offset(content, viewport);
    let ratio = if max > 0.0 {
        clamp_offset(offset, content, viewport) / max
    } else {
        0.0
    };
    Some(Thumb {
        start: (travel * ratio).max(0.0),
        length,
    })
}

/// Offset for a dragged thumb start, the inverse of `thumb`.
#[must_use]
pub(crate) fn offset_for_thumb(
    start: f32,
    content: f32,
    viewport: f32,
    track: f32,
    min_length: f32,
) -> f32 {
    let Some(length) = thumb_length(content, viewport, track, min_length) else {
        return 0.0;
    };
    let travel = track - length;
    if !positive(travel) {
        return 0.0;
    }
    let start = finite_or_zero(start).max(0.0).min(travel);
    start / travel * max_offset(content, viewport)
}

/// Column whose span contains content x (measured after the inline inset), given visible
/// widths in display order. Non-finite or negative widths count as zero.
#[must_use]
pub(crate) fn column_at(x: f32, widths: &[f32]) -> Option<usize> {
    if !x.is_finite() || x < 0.0 {
        return None;
    }
    let mut end = 0.0_f32;
    for (index, width) in widths.iter().enumerate() {
        end += if positive(*width) { *width } else { 0.0 };
        if x < end {
            return Some(index);
        }
    }
    None
}

/// Column whose inline-end edge is within `hit_width / 2` of content x. When two edges are in
/// reach, the nearer wins; on a tie, the earlier column.
#[must_use]
pub(crate) fn edge_at(x: f32, widths: &[f32], hit_width: f32) -> Option<usize> {
    if !x.is_finite() || !positive(hit_width) {
        return None;
    }
    let reach = hit_width / 2.0;
    let mut edge = 0.0_f32;
    let mut best: Option<(usize, f32)> = None;
    for (index, width) in widths.iter().enumerate() {
        edge += if positive(*width) { *width } else { 0.0 };
        let distance = (x - edge).abs();
        if distance <= reach {
            match best {
                Some((_, nearest)) if nearest <= distance => {}
                _ => best = Some((index, distance)),
            }
        }
    }
    best.map(|(index, _)| index)
}

/// Screen x of the left edge of a span given in inline coordinates within
/// `origin..origin + extent`: identity for left to right, reflected for right to left.
#[must_use]
pub(crate) fn mirror(rtl: bool, origin: f32, extent: f32, x: f32, width: f32) -> f32 {
    if rtl {
        origin + extent - x - width
    } else {
        origin + x
    }
}

/// Pixels for a wheel delta given in lines, the factor iced's scrollable uses.
#[must_use]
pub(crate) const fn lines_to_pixels(lines: f32) -> f32 {
    lines * crate::app::layout::WHEEL_LINE_PIXELS
}
