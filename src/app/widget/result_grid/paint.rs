//! Drawing primitives shared by the grid's draw passes: rectangles and single-line labels.
//!
//! Labels are drawn with infinite bounds so a narrow cell never wraps, and clipped by the
//! caller's layer. `fill_text` positions text by the anchor its alignment selects, and iced only
//! relayouts a single right-to-left line for `Alignment::Default` (anchored at its left edge),
//! so right-aligned or centred labels that contain a right-to-left run are measured once and
//! drawn with `Default` from the computed left edge. Everything else needs no measurement.

use iced::advanced::renderer::{Quad, Renderer as _};
use iced::advanced::text::{
    Alignment, Paragraph as _, Renderer as TextRenderer, Shaping, Text, Wrapping,
};
use iced::alignment::Vertical;
use iced::{Border, Color, Font, Pixels, Point, Rectangle, Size};

use crate::app::theme::LINE_HEIGHT;
use crate::app::view_model::CellAlign;

/// Paragraph type of the application renderer, used only to measure right-to-left labels.
type Measure = <iced::Renderer as TextRenderer>::Paragraph;

/// Screen side a label hugs inside its span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Side {
    /// Left edge.
    Left,
    /// Centre.
    Center,
    /// Right edge.
    Right,
}

impl Side {
    /// Screen side of a logical cell alignment.
    pub(super) const fn of(align: CellAlign, rtl: bool) -> Self {
        match (align, rtl) {
            (CellAlign::Start, false) | (CellAlign::End, true) => Self::Left,
            (CellAlign::Start, true) | (CellAlign::End, false) => Self::Right,
        }
    }
}

/// Font, size and colour of one label.
#[derive(Clone, Copy, Debug)]
pub(super) struct Style {
    /// Face.
    pub(super) font: Font,
    /// Size in logical pixels.
    pub(super) size: f32,
    /// Colour.
    pub(super) color: Color,
}

/// Fills a rectangle when there is one.
pub(super) fn fill_rect(renderer: &mut iced::Renderer, bounds: Option<Rectangle>, color: Color) {
    if let Some(bounds) = bounds {
        renderer.fill_quad(
            Quad {
                bounds,
                ..Quad::default()
            },
            color,
        );
    }
}

/// Fills a rectangle with a border and corner radius.
pub(super) fn fill_bordered(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    color: Color,
    border: Border,
) {
    renderer.fill_quad(
        Quad {
            bounds,
            border,
            ..Quad::default()
        },
        color,
    );
}

/// Whether text may hold a right-to-left run: Hebrew, Arabic, Syriac, Thaana, NKo and their
/// presentation forms, and the historic right-to-left blocks. ASCII text returns at once.
pub(super) fn may_be_rtl(content: &str) -> bool {
    !content.is_ascii()
        && content.chars().any(|character| {
            matches!(
                character,
                '\u{0590}'..='\u{08FF}'
                    | '\u{FB1D}'..='\u{FDFF}'
                    | '\u{FE70}'..='\u{FEFF}'
                    | '\u{10800}'..='\u{10FFF}'
                    | '\u{1E800}'..='\u{1EFFF}'
            )
        })
}

/// Text value with this module's fixed shaping, wrapping and line height.
fn text<C: AsRef<str>>(
    content: C,
    bounds: Size,
    style: Style,
    align_x: Alignment,
    align_y: Vertical,
) -> Text<C, Font> {
    let font = crate::app::theme::font_for_text(style.font, content.as_ref());
    Text {
        content,
        bounds,
        size: Pixels(style.size),
        line_height: crate::app::theme::cached_line_height(LINE_HEIGHT),
        font,
        align_x,
        align_y,
        shaping: Shaping::Auto,
        wrapping: Wrapping::None,
    }
}

/// Width of a label laid out within `max_width`.
fn measured_width(content: &str, style: Style, max_width: f32) -> f32 {
    let mut probe = text(
        content,
        Size::new(max_width, f32::INFINITY),
        style,
        Alignment::Default,
        Vertical::Top,
    );
    if max_width.is_finite() {
        probe.wrapping = Wrapping::Word;
    }
    <Measure as iced::advanced::text::Paragraph>::with_text(probe)
        .min_bounds()
        .width
}

/// Anchor x and alignment that put a label against `side` of `left..right`.
pub(super) fn anchor(
    content: &str,
    side: Side,
    left: f32,
    right: f32,
    style: Style,
) -> (f32, Alignment) {
    match side {
        Side::Left => (left, Alignment::Default),
        Side::Right if !may_be_rtl(content) => (right, Alignment::Right),
        Side::Center if !may_be_rtl(content) => ((left + right) / 2.0, Alignment::Center),
        Side::Right => (
            right - measured_width(content, style, f32::INFINITY),
            Alignment::Default,
        ),
        Side::Center => {
            let width = measured_width(content, style, f32::INFINITY);
            ((left + right - width) / 2.0, Alignment::Default)
        }
    }
}

/// Draws one line of text against `side` of `area`, vertically centred in it, clipped to `clip`
/// (which only clips on wgpu; callers also wrap it in a layer).
pub(super) fn fill_line(
    renderer: &mut iced::Renderer,
    content: &str,
    side: Side,
    area: Rectangle,
    style: Style,
    clip: Rectangle,
) {
    let (x, align_x) = anchor(content, side, area.x, area.x + area.width, style);
    renderer.fill_text(
        text(
            content.to_owned(),
            Size::new(f32::INFINITY, area.height),
            style,
            align_x,
            Vertical::Center,
        ),
        Point::new(x, area.center_y()),
        style.color,
        clip,
    );
}

/// Draws wrapping text centred horizontally on `center_x` within `width`, starting at `top`.
pub(super) fn fill_block(
    renderer: &mut iced::Renderer,
    content: &str,
    center_x: f32,
    top: f32,
    width: f32,
    style: Style,
    clip: Rectangle,
) {
    let (x, align_x) = if may_be_rtl(content) {
        (
            center_x - measured_width(content, style, width) / 2.0,
            Alignment::Default,
        )
    } else {
        (center_x, Alignment::Center)
    };
    let mut block = text(
        content.to_owned(),
        Size::new(width, clip.height),
        style,
        align_x,
        Vertical::Top,
    );
    block.wrapping = Wrapping::Word;
    renderer.fill_text(block, Point::new(x, top), style.color, clip);
}
