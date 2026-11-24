//! Rows of the keyboard table in the About dialog.

use crate::app::context::Platform;
use crate::app::i18n::Text;

/// One key cap, or the punctuation between caps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Cap {
    /// A named key with a catalog label.
    Key(Text),
    /// A key written the same in every language, such as a letter or a function key.
    Literal(&'static str),
    /// The platform's command key: Cmd on macOS, Ctrl elsewhere.
    Command,
    /// Joins the keys of one chord.
    Plus,
    /// Separates two alternative chords.
    Or,
}

impl Cap {
    /// Whether this entry is drawn as a key cap; the separators are plain text.
    #[must_use]
    pub(super) const fn is_key(self) -> bool {
        !matches!(self, Self::Plus | Self::Or)
    }
}

/// One keyboard shortcut: what it does and the keys that do it.
#[derive(Clone, Copy, Debug)]
pub(super) struct KeyRow {
    /// Catalog text of the action.
    pub(super) action: Text,
    /// Keys in reading order.
    pub(super) caps: &'static [Cap],
}

/// The shortcuts of the layout document's keyboard table that matter to a reader of the About
/// dialog, in the order they are listed there.
pub(super) const ROWS: [KeyRow; 10] = [
    KeyRow {
        action: Text::AboutKeysSearch,
        caps: &[Cap::Command, Cap::Plus, Cap::Literal("F")],
    },
    KeyRow {
        action: Text::AboutKeysRegions,
        caps: &[
            Cap::Literal("F6"),
            Cap::Or,
            Cap::Key(Text::KeycapShift),
            Cap::Plus,
            Cap::Literal("F6"),
        ],
    },
    KeyRow {
        action: Text::AboutKeysFocus,
        caps: &[
            Cap::Key(Text::KeycapTab),
            Cap::Or,
            Cap::Key(Text::KeycapShift),
            Cap::Plus,
            Cap::Key(Text::KeycapTab),
        ],
    },
    KeyRow {
        action: Text::AboutKeysRows,
        caps: &[
            Cap::Key(Text::KeycapArrows),
            Cap::Key(Text::KeycapPageUp),
            Cap::Key(Text::KeycapPageDown),
            Cap::Key(Text::KeycapHome),
            Cap::Key(Text::KeycapEnd),
        ],
    },
    KeyRow {
        action: Text::AboutKeysColumns,
        caps: &[Cap::Key(Text::KeycapArrows)],
    },
    KeyRow {
        action: Text::AboutKeysSort,
        caps: &[Cap::Key(Text::KeycapEnter)],
    },
    KeyRow {
        action: Text::AboutKeysResize,
        caps: &[
            Cap::Key(Text::KeycapShift),
            Cap::Plus,
            Cap::Key(Text::KeycapArrows),
        ],
    },
    KeyRow {
        action: Text::AboutKeysDetails,
        caps: &[Cap::Key(Text::KeycapSpace)],
    },
    KeyRow {
        action: Text::AboutKeysToggle,
        caps: &[Cap::Key(Text::KeycapSpace)],
    },
    KeyRow {
        action: Text::AboutKeysClose,
        caps: &[Cap::Key(Text::KeycapEscape)],
    },
];

/// Catalog text of the platform's command key. macOS desktop uses Cmd; the browser and the
/// other desktops use Ctrl, which is also what the shortcut handler accepts there.
#[must_use]
pub(super) fn command_key(platform: Platform) -> Text {
    if cfg!(target_os = "macos") && platform.has_desktop_controls() {
        Text::KeycapCmd
    } else {
        Text::KeycapCtrl
    }
}

#[cfg(test)]
mod tests {
    use super::{Cap, ROWS, command_key};
    use crate::app::context::Platform;
    use crate::app::i18n::Text;

    #[test]
    fn every_row_has_at_least_one_key_and_balanced_separators() {
        for row in &ROWS {
            assert!(row.caps.iter().any(|cap| cap.is_key()), "{:?}", row.action);
            assert!(row.caps.first().is_some_and(|cap| cap.is_key()));
            assert!(row.caps.last().is_some_and(|cap| cap.is_key()));
            for pair in row.caps.windows(2) {
                assert!(
                    pair.iter().any(|cap| cap.is_key()),
                    "two separators in a row for {:?}",
                    row.action
                );
            }
        }
    }

    #[test]
    fn actions_are_listed_once() {
        for (index, row) in ROWS.iter().enumerate() {
            for other in &ROWS[index + 1..] {
                assert_ne!(row.action, other.action);
            }
        }
    }

    #[test]
    fn the_search_chord_uses_the_command_key() {
        assert_eq!(ROWS[0].action, Text::AboutKeysSearch);
        assert_eq!(ROWS[0].caps[0], Cap::Command);
        assert_eq!(command_key(Platform::Browser), Text::KeycapCtrl);
        let desktop = command_key(Platform::Desktop);
        assert!(matches!(desktop, Text::KeycapCmd | Text::KeycapCtrl));
    }
}
