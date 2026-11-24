use super::{diff, diff_text};
use crate::AppError;
use crate::filter::test_fixture::{fixture, fixture_pack};
use crate::lang_pack::{GameLanguage, Labels, validate_pack};

#[test]
fn reports_changed_fields_and_removed_locations() {
    let loaded = crate::embedded::load();
    assert!(loaded.is_ok());
    let Ok((old, _)) = loaded else { return };
    let unchanged = diff(&old, &old);
    assert!(unchanged.added.is_empty());
    assert!(unchanged.removed.is_empty());
    assert!(unchanged.fields.is_empty());

    let mut new = old.clone();
    let removed = new.stored.locations.pop();
    assert!(removed.is_some());
    let Some(first) = new.stored.locations.first_mut() else {
        return;
    };
    first.coastal = !first.coastal;
    let changed = diff(&old, &new);
    assert_eq!(changed.removed.len(), 1);
    assert!(changed.added.is_empty());
    assert_eq!(
        changed.fields.get("coastal").map(|field| field.count),
        Some(1)
    );
    assert!(changed.report().contains("coastal: 1 changed"));
}

#[test]
fn text_diff_compares_names_and_labels_by_key() -> Result<(), AppError> {
    let old = fixture();
    let new = fixture();
    let pack = validate_pack(fixture_pack(&new), &new, GameLanguage::Korean)?;
    let english = diff_text(
        GameLanguage::English,
        Labels::new(&old, None),
        Labels::new(&new, None),
    );
    assert_eq!(english.names.count, 0);
    assert!(english.labels.is_empty());
    assert!(
        english
            .report()
            .contains("packs: old embedded, new embedded")
    );

    let korean = diff_text(
        GameLanguage::Korean,
        Labels::new(&old, None),
        Labels::new(&new, Some(&pack)),
    );
    assert!(!korean.old_pack && korean.new_pack);
    assert_eq!(korean.names.count, 2);
    assert_eq!(
        korean.labels.get("topography").map(|changes| changes.count),
        Some(1)
    );
    let report = korean.report();
    assert!(report.starts_with("## ko\npacks: old missing, new present\n"));
    assert!(report.contains("names: 2 changed\n  eire: Éire -> 에이레\n"));
    assert!(report.contains("labels.topography: 1 changed\n  flatland: flatland -> 평지\n"));
    Ok(())
}
