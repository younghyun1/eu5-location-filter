//! Interaction state kept in the grid's widget tree between frames.

use std::cell::RefCell;

use iced::Point;
use iced::advanced::widget::operation::Focusable;
use iced::keyboard::Modifiers;
use iced::touch::Finger;

use crate::app::view_model::ColumnId;
use crate::model::LocationId;

/// Dominant axis a touch drag locked to after `TOUCH_SLOP`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Axis {
    /// Horizontal pan.
    Horizontal,
    /// Vertical scroll.
    Vertical,
}

/// Pointer drag in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GridDrag {
    /// Header edge drag of a column.
    Resize {
        /// Column being resized.
        column: ColumnId,
        /// Screen x at drag start.
        start_x: f32,
        /// Column width at drag start.
        start_width: f32,
    },
    /// Vertical thumb drag; `grab` is the pointer offset inside the thumb.
    VerticalThumb {
        /// Pointer offset from the thumb start.
        grab: f32,
    },
    /// Horizontal thumb drag.
    HorizontalThumb {
        /// Pointer offset from the thumb start.
        grab: f32,
    },
    /// Middle-button pan; content follows the pointer physically in both directions.
    Pan {
        /// Previous pointer position.
        last: Point,
    },
}

/// Touch drag in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TouchPan {
    /// Finger being tracked.
    pub(crate) finger: Finger,
    /// Press position, for the tap-versus-drag decision.
    pub(crate) start: Point,
    /// Previous position.
    pub(crate) last: Point,
    /// Locked axis once movement passed `TOUCH_SLOP`; `None` is still a tap.
    pub(crate) axis: Option<Axis>,
}

/// Grid tree state.
#[derive(Debug, Default)]
pub(crate) struct GridState {
    /// Horizontal offset from the inline start of the content.
    pub(crate) offset_x: f32,
    /// Vertical offset from the first row.
    pub(crate) offset_y: f32,
    /// Row under the pointer.
    pub(crate) hovered_row: Option<usize>,
    /// Column whose resize edge is under the pointer.
    pub(crate) hovered_edge: Option<ColumnId>,
    /// Pointer drag in progress.
    pub(crate) drag: Option<GridDrag>,
    /// Touch drag in progress.
    pub(crate) touch: Option<TouchPan>,
    /// Keyboard-active column; starts at the sorted column.
    pub(crate) active_column: Option<ColumnId>,
    /// Keyboard focus.
    pub(crate) focused: bool,
    /// Latest modifiers, for Shift with the wheel and Shift with arrows.
    pub(crate) modifiers: Modifiers,
    /// Selection seen in the previous update, so an external selection change scrolls it into
    /// view once.
    pub(crate) last_selected: Option<LocationId>,
    /// Reused cell text buffer; `draw` only has shared access to the tree, hence the cell. Use
    /// `try_borrow_mut` and skip the cell on failure rather than panicking.
    pub(crate) buffer: RefCell<String>,
}

impl Focusable for GridState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}
