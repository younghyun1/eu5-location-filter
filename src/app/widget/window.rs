//! Pure windowing math for fixed-height rows and variable-width columns, shared by the result
//! grid and the checklist. All values are logical pixels; offsets are measured from the inline
//! start of the content, and `mirror` maps them to screen x for right-to-left layouts.

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

/// Rows intersecting `offset..offset + viewport`, clamped to `count`; row cost is independent
/// of `count`.
#[must_use]
pub(crate) fn visible_rows(
    offset: f32,
    viewport: f32,
    row_height: f32,
    count: usize,
) -> VisibleRange {
    let _ = (offset, viewport, row_height, count);
    VisibleRange::default()
}

/// Largest scroll offset; zero when the content fits.
#[must_use]
pub(crate) fn max_offset(content: f32, viewport: f32) -> f32 {
    (content - viewport).max(0.0)
}

/// Offset clamped to `0..=max_offset`; a non-finite offset becomes zero.
#[must_use]
pub(crate) fn clamp_offset(offset: f32, content: f32, viewport: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(0.0, max_offset(content, viewport))
    } else {
        0.0
    }
}

/// Row under a y coordinate relative to the body top, or `None` past the last row.
#[must_use]
pub(crate) fn row_at(y: f32, offset: f32, row_height: f32, count: usize) -> Option<usize> {
    let _ = (y, offset, row_height, count);
    None
}

/// Smallest offset change that shows `row` fully.
#[must_use]
pub(crate) fn scroll_into_view(offset: f32, row: usize, row_height: f32, viewport: f32) -> f32 {
    let _ = (row, row_height, viewport);
    offset
}

/// Smallest offset change that shows the span `start..start + width` fully.
#[must_use]
pub(crate) fn span_into_view(offset: f32, start: f32, width: f32, viewport: f32) -> f32 {
    let _ = (start, width, viewport);
    offset
}

/// Rows a Page Up or Page Down moves: the visible row count minus one, at least one.
#[must_use]
pub(crate) fn page_rows(viewport: f32, row_height: f32) -> usize {
    let _ = (viewport, row_height);
    1
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
    let _ = (offset, content, viewport, track, min_length);
    None
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
    let _ = (start, content, viewport, track, min_length);
    0.0
}

/// Column whose span contains content x (measured after the inline inset), given visible
/// widths in display order.
#[must_use]
pub(crate) fn column_at(x: f32, widths: &[f32]) -> Option<usize> {
    let _ = (x, widths);
    None
}

/// Column whose inline-end edge is within `hit_width / 2` of content x.
#[must_use]
pub(crate) fn edge_at(x: f32, widths: &[f32], hit_width: f32) -> Option<usize> {
    let _ = (x, widths, hit_width);
    None
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
