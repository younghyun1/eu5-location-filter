//! Verification that a written font maps every required codepoint.

use std::collections::BTreeSet;

use crate::AppError;

/// Most missing codepoints listed in an error message.
const LISTED: usize = 32;

/// Returns the required codepoints absent from `available`, in ascending order.
pub(super) fn missing(required: &BTreeSet<u32>, available: &BTreeSet<u32>) -> Vec<u32> {
    required.difference(available).copied().collect()
}

/// Fails with the missing codepoints when `available` does not contain every required one.
///
/// `label` names the font in the message, for example the output file.
pub(super) fn verify(
    label: &str,
    required: &BTreeSet<u32>,
    available: &BTreeSet<u32>,
) -> Result<(), AppError> {
    let absent = missing(required, available);
    if absent.is_empty() {
        return Ok(());
    }
    let mut listed: Vec<String> = absent
        .iter()
        .take(LISTED)
        .map(|code| format!("U+{code:04X}"))
        .collect();
    if absent.len() > LISTED {
        listed.push(format!("and {} more", absent.len() - LISTED));
    }
    Err(AppError::InvalidData(format!(
        "{label} lacks {} required codepoints: {}",
        absent.len(),
        listed.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{missing, verify};
    use crate::AppError;

    #[test]
    fn full_coverage_passes_and_extra_glyphs_are_fine() {
        let required = BTreeSet::from([0x41, 0xAC00]);
        let available = BTreeSet::from([0x41, 0x42, 0xAC00]);
        assert!(missing(&required, &available).is_empty());
        assert!(verify("font.ttf", &required, &available).is_ok());
        assert!(verify("font.ttf", &BTreeSet::new(), &BTreeSet::new()).is_ok());
    }

    #[test]
    fn gaps_are_reported_in_order_with_hex_codes() {
        let required = BTreeSet::from([0xAC01, 0x41, 0xAC00]);
        let available = BTreeSet::from([0x41]);
        assert_eq!(missing(&required, &available), [0xAC00, 0xAC01]);
        assert!(matches!(
            verify("kr.ttf", &required, &available),
            Err(AppError::InvalidData(message))
                if message.starts_with("kr.ttf lacks 2 required codepoints")
                    && message.ends_with("U+AC00, U+AC01")
        ));
    }

    #[test]
    fn long_gap_lists_are_truncated() {
        let required: BTreeSet<u32> = (0x4E00..0x4E00 + 40).collect();
        assert!(matches!(
            verify("sc.ttf", &required, &BTreeSet::new()),
            Err(AppError::InvalidData(message)) if message.ends_with("and 8 more")
        ));
    }
}
