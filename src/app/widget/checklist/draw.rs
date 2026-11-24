//! Drawing of the checklist. Only rows inside the visible part of the bounds are drawn, so a
//! list of thousands of options costs the same as one of seven. Quads and checkboxes share
//! one clip layer over the visible bounds; labels get a second layer over the label strip,
//! because tiny-skia ignores per-text clip bounds. Labels use infinite bounds and
//! `text::Shaping::Auto`, following the result grid.

use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::text::{self, Paragraph as _, Renderer as _, Text};
use iced::alignment::Vertical;
use iced::{Border, Color, Font, Pixels, Point, Rectangle, Size};

use super::geometry::{Geometry, has_rtl_script};
use super::list::Checklist;
use super::model::ChecklistModel;
use super::state::ChecklistState;
use crate::app::layout::{ACTIVE_OPTION_OUTLINE, BORDER_WIDTH, CHECKBOX_SIZE, RADIUS};
use crate::app::theme::{Icon, LINE_HEIGHT, TEXT_BODY, ThumbState};

/// Size of the check glyph relative to the box, the factor iced's stock checkbox uses.
const CHECK_GLYPH_SCALE: f32 = 0.7;

/// Concrete paragraph type of the application renderer, used to measure right-to-left labels.
type RendererParagraph = <iced::Renderer as text::Renderer>::Paragraph;

impl<Message, M: ChecklistModel> Checklist<'_, Message, M> {
    /// Draws the list into `bounds`, clipped to `viewport`.
    pub(super) fn draw_list(
        &self,
        state: &ChecklistState,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        viewport: &Rectangle,
    ) {
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let palette = self.palette;
        let count = self.model.row_count();
        let geometry = Geometry::new(bounds, self.row_height, count, self.direction.is_rtl());
        let offset = geometry.clamp(state.offset);
        let range = geometry.visible(offset, clip);
        let hovered = state.hovered.filter(|row| *row < count);
        // Focus that arrives by Tab has no active row yet; outline the row Space would toggle.
        let active = if state.focused {
            state
                .active
                .filter(|row| *row < count)
                .or_else(|| geometry.first_visible(offset))
        } else {
            None
        };

        fill(renderer, bounds, palette.surface, Border::default());
        renderer.with_layer(clip, |renderer| {
            for row in range.start..range.end {
                let rect = geometry.row_rect(row, offset);
                let row_hovered = hovered == Some(row);
                if row_hovered {
                    fill(
                        renderer,
                        rect,
                        palette.hover(palette.surface),
                        Border::default(),
                    );
                }
                self.draw_checkbox(
                    renderer,
                    &geometry,
                    rect,
                    self.model.is_checked(row),
                    row_hovered,
                    clip,
                );
                if active == Some(row) {
                    let outline = Border {
                        color: palette.accent,
                        width: ACTIVE_OPTION_OUTLINE,
                        radius: RADIUS.into(),
                    };
                    fill(renderer, rect, Color::TRANSPARENT, outline);
                }
            }
            if let Some(thumb) = geometry.thumb_rect(offset, self.touch) {
                let thumb_state = match (state.thumb_grab, state.lane_hovered) {
                    (Some(_), _) => ThumbState::Dragged,
                    (None, true) => ThumbState::Hovered,
                    (None, false) => ThumbState::Idle,
                };
                let radius = thumb.width / 2.0;
                fill(
                    renderer,
                    thumb,
                    palette.scrollbar_thumb(thumb_state),
                    Border {
                        radius: radius.into(),
                        ..Border::default()
                    },
                );
            }
        });

        if let Some(lane) = geometry.label_lane().intersection(&clip) {
            renderer.with_layer(lane, |renderer| {
                if count == 0 {
                    let rect = geometry.row_rect(0, 0.0);
                    self.draw_label(
                        renderer,
                        &geometry,
                        rect,
                        self.empty_text,
                        palette.ink_faint,
                        lane,
                    );
                }
                for row in range.start..range.end {
                    let rect = geometry.row_rect(row, offset);
                    self.draw_label(
                        renderer,
                        &geometry,
                        rect,
                        self.model.label(row),
                        palette.ink,
                        lane,
                    );
                }
            });
        }

        // Quads issued after a layer closes go back to the parent layer, which renders first, so
        // the frame gets its own last layer to stay above hovered rows.
        let frame = Border {
            color: palette.line,
            width: BORDER_WIDTH,
            radius: RADIUS.into(),
        };
        renderer.with_layer(clip, |renderer| {
            fill(renderer, bounds, Color::TRANSPARENT, frame)
        });
    }

    /// Draws one checkbox: an outlined square, or an accent square with the check glyph.
    fn draw_checkbox(
        &self,
        renderer: &mut iced::Renderer,
        geometry: &Geometry,
        row: Rectangle,
        checked: bool,
        hovered: bool,
        clip: Rectangle,
    ) {
        let palette = self.palette;
        let rect = geometry.checkbox_rect(row);
        if !checked {
            let border = Border {
                color: palette.line_strong,
                width: BORDER_WIDTH,
                radius: RADIUS.into(),
            };
            fill(renderer, rect, palette.surface, border);
            return;
        }
        let ground = if hovered {
            palette.accent_strong
        } else {
            palette.accent
        };
        fill(
            renderer,
            rect,
            ground,
            Border {
                color: ground,
                width: BORDER_WIDTH,
                radius: RADIUS.into(),
            },
        );
        let Some(glyph_clip) = rect.intersection(&clip) else {
            return;
        };
        renderer.fill_text(
            Text {
                content: Icon::Check.glyph(self.direction).to_string(),
                bounds: rect.size(),
                size: Pixels(CHECKBOX_SIZE * CHECK_GLYPH_SCALE),
                line_height: text::LineHeight::default(),
                font: Icon::Check.font(),
                align_x: text::Alignment::Center,
                align_y: Vertical::Center,
                shaping: text::Shaping::Auto,
                wrapping: text::Wrapping::None,
            },
            rect.center(),
            palette.on_accent(),
            glyph_clip,
        );
    }

    /// Draws a label starting at the label strip's inline start. Right to left, a label must
    /// end at the strip's right edge: a label without right-to-left script is anchored there
    /// with `Right`, and one with right-to-left script is measured once and placed by its
    /// left edge with `Default`, which is the only alignment iced lays out correctly for
    /// such text against an infinite width.
    fn draw_label(
        &self,
        renderer: &mut iced::Renderer,
        geometry: &Geometry,
        row: Rectangle,
        label: &str,
        color: Color,
        clip: Rectangle,
    ) {
        if label.is_empty() {
            return;
        }
        let Some(row_clip) = row.intersection(&clip) else {
            return;
        };
        let lane = geometry.label_lane();
        let (x, align_x) = if !geometry.rtl {
            (lane.x, text::Alignment::Default)
        } else if has_rtl_script(label) {
            (
                lane.x + lane.width - measure(label, self.font, row.height),
                text::Alignment::Default,
            )
        } else {
            (lane.x + lane.width, text::Alignment::Right)
        };
        renderer.fill_text(
            label_text(label.to_owned(), self.font, row.height, align_x),
            Point::new(x, row.center_y()),
            color,
            row_clip,
        );
    }
}

/// Label text with the checklist's size, line height and shaping.
fn label_text<C: AsRef<str>>(
    content: C,
    font: Font,
    row_height: f32,
    align_x: text::Alignment,
) -> Text<C, Font> {
    let font = crate::app::theme::font_for_text(font, content.as_ref());
    Text {
        content,
        bounds: Size::new(f32::INFINITY, row_height),
        size: Pixels(TEXT_BODY),
        line_height: text::LineHeight::Relative(LINE_HEIGHT),
        font,
        align_x,
        align_y: Vertical::Center,
        shaping: text::Shaping::Auto,
        wrapping: text::Wrapping::None,
    }
}

/// Measured width of a single-line label.
fn measure(label: &str, font: Font, row_height: f32) -> f32 {
    RendererParagraph::with_text(label_text(
        label,
        font,
        row_height,
        text::Alignment::Default,
    ))
    .min_width()
}

/// Fills a quad with a border drawn on its inside.
fn fill(renderer: &mut iced::Renderer, bounds: Rectangle, color: Color, border: Border) {
    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border,
            ..renderer::Quad::default()
        },
        color,
    );
}
