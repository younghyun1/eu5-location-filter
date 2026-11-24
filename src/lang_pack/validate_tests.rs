//! Every pack validation failure, in contract order, plus the validated lookups.

use super::{GameLanguage, PackLabel, PackStringId, StoredLanguagePack, validate_pack};
use crate::AppError;
use crate::filter::test_fixture::{fixture, fixture_pack, section_for};
use crate::model::{ImportDiagnostic, LocationId, SymbolId};

/// Validates the Korean fixture pack after `mutate`, returning the error if any.
fn rejected(mutate: impl FnOnce(&mut StoredLanguagePack)) -> Option<AppError> {
    rejected_as(GameLanguage::Korean, mutate)
}

fn rejected_as(
    expected: GameLanguage,
    mutate: impl FnOnce(&mut StoredLanguagePack),
) -> Option<AppError> {
    let dataset = fixture();
    let mut pack = fixture_pack(&dataset);
    mutate(&mut pack);
    validate_pack(pack, &dataset, expected).err()
}

fn mismatch(error: Option<AppError>) -> bool {
    matches!(error, Some(AppError::PackMismatch(_)))
}

fn invalid(error: Option<AppError>) -> bool {
    matches!(error, Some(AppError::InvalidData(_)))
}

#[test]
fn valid_pack_exposes_localized_text_with_english_gaps() -> Result<(), AppError> {
    let dataset = fixture();
    let pack = validate_pack(fixture_pack(&dataset), &dataset, GameLanguage::Korean)?;
    assert_eq!(pack.language(), GameLanguage::Korean);
    assert_eq!(pack.name(LocationId(0)), Some("에이레"));
    assert_eq!(pack.name(LocationId(9)), None);
    assert_eq!(pack.label(SymbolId(2)), Some("평지"));
    assert_eq!(pack.label(SymbolId(3)), None);
    assert_eq!(pack.label(SymbolId(u32::MAX)), None);
    assert_eq!(pack.section().searchable.len(), 2);
    Ok(())
}

#[test]
fn schema_and_application_failures_are_invalid_data() {
    assert!(invalid(rejected(|pack| pack.format_version += 1)));
    assert!(invalid(rejected(|pack| pack.app_id += 1)));
    // The schema check runs first, so a stale schema is never reported as a mismatch.
    assert!(invalid(rejected(|pack| {
        pack.format_version += 1;
        pack.build_id += 1;
    })));
}

#[test]
fn every_pairing_failure_is_a_mismatch() {
    assert!(mismatch(rejected(|pack| pack.build_id += 1)));
    assert!(mismatch(rejected(|pack| pack.location_count += 1)));
    assert!(mismatch(rejected(|pack| pack.symbol_count += 1)));
    assert!(mismatch(rejected(|pack| pack.dictionary_digest ^= 1)));
    assert!(mismatch(rejected(|pack| {
        pack.language = GameLanguage::Japanese;
    })));
    assert!(mismatch(rejected_as(GameLanguage::Japanese, |_| {})));
    assert!(mismatch(rejected_as(GameLanguage::English, |pack| {
        pack.language = GameLanguage::English;
    })));
}

#[test]
fn string_table_failures_are_invalid_data() {
    assert!(invalid(rejected(|pack| pack.strings.push(String::new()))));
    assert!(invalid(rejected(|pack| pack
        .strings
        .push("평\0지".to_owned()))));
    assert!(invalid(rejected(|pack| pack
        .strings
        .push("에이레".to_owned()))));
    assert!(invalid(rejected(|pack| pack
        .strings
        .push("가".repeat(342)))));
    // Two locations and five symbols allow at most seven strings.
    assert!(invalid(rejected(|pack| {
        pack.strings
            .extend(["가", "나", "다", "라", "마"].map(str::to_owned));
    })));
}

#[test]
fn name_and_label_reference_failures_are_invalid_data() {
    assert!(invalid(rejected(|pack| {
        pack.location_names.pop();
    })));
    assert!(invalid(rejected(|pack| {
        pack.location_names[0] = Some(PackStringId(99));
    })));
    let label = |key, value| PackLabel {
        key: SymbolId(key),
        value: PackStringId(value),
    };
    assert!(invalid(rejected(|pack| pack.labels.push(label(1, 0)))));
    assert!(invalid(rejected(|pack| pack.labels.push(label(2, 0)))));
    assert!(invalid(rejected(|pack| pack.labels.push(label(5, 0)))));
    assert!(invalid(rejected(
        |pack| pack.labels[0].value = PackStringId(9)
    )));
}

#[test]
fn oversized_diagnostics_are_invalid_data() {
    let diagnostic = |code: String| ImportDiagnostic { code, count: 1 };
    assert!(invalid(rejected(|pack| {
        pack.diagnostics = (0..65)
            .map(|index| diagnostic(format!("code_{index}")))
            .collect();
    })));
    assert!(invalid(rejected(|pack| {
        pack.diagnostics = vec![diagnostic("x".repeat(65))];
    })));
    assert!(
        rejected(|pack| pack.diagnostics = vec![diagnostic("unresolved_reference".to_owned())])
            .is_none()
    );
}

#[test]
fn section_failures_are_invalid_data() {
    // Changing a name without rebuilding the section leaves search text for other labels.
    assert!(invalid(rejected(
        |pack| pack.strings[0] = "에이리".to_owned()
    )));
    assert!(invalid(rejected(|pack| {
        pack.section.searchable.pop();
    })));
    assert!(invalid(rejected(|pack| {
        if let Some(posting) = pack.section.trigrams.first_mut() {
            posting.ids.push(LocationId(9));
        }
    })));
    assert!(invalid(rejected(|pack| {
        pack.section.orders[0].ascending = vec![LocationId(0), LocationId(0)];
    })));
    assert!(invalid(rejected(|pack| {
        pack.section.orders[0].ascending_ranks.reverse();
    })));
    assert!(invalid(rejected(|pack| {
        pack.section.orders.pop();
    })));
    assert!(invalid(rejected(|pack| pack.section.orders.swap(0, 1))));
}

#[test]
fn a_rebuilt_section_repairs_changed_text() -> Result<(), AppError> {
    let dataset = fixture();
    let mut pack = fixture_pack(&dataset);
    pack.strings[0] = "에이리".to_owned();
    pack.section = section_for(&dataset, &pack);
    let validated = validate_pack(pack, &dataset, GameLanguage::Korean)?;
    assert_eq!(validated.name(LocationId(0)), Some("에이리"));
    Ok(())
}
