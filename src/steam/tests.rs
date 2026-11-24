use std::fs;
use std::path::PathBuf;

use super::{GameInstallation, atom_at, object_at};
use crate::AppError;
use crate::lang_pack::GameLanguage;
use crate::parser::{VdfEntry, VdfValue};

#[test]
fn finds_values_case_insensitively() {
    let entries = vec![VdfEntry {
        key: "Root".to_owned(),
        value: VdfValue::Object(vec![VdfEntry {
            key: "BuildID".to_owned(),
            value: VdfValue::Atom("42".to_owned()),
        }]),
    }];
    let root = object_at(&entries, "root");
    assert!(root.is_some());
    assert_eq!(root.and_then(|value| atom_at(value, "buildid")), Some("42"));
}

#[test]
fn localization_roots_follow_the_language_directory() -> Result<(), AppError> {
    let root =
        std::env::temp_dir().join(format!("eu5-location-filter-roots-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let scoped = |scope: &str, language: GameLanguage| -> PathBuf {
        root.join("game")
            .join(scope)
            .join("localization")
            .join(language.directory())
    };
    for path in [
        scoped("main_menu", GameLanguage::English),
        scoped("loading_screen", GameLanguage::English),
        scoped("main_menu", GameLanguage::SimplifiedChinese),
        root.join("game/loading_screen/localization/jomini"),
    ] {
        fs::create_dir_all(&path).map_err(|source| AppError::io("create", &path, source))?;
    }
    let installation = GameInstallation {
        root: root.clone(),
        build_id: 0,
    };
    assert_eq!(
        installation.localization_roots(GameLanguage::English),
        vec![
            scoped("loading_screen", GameLanguage::English),
            scoped("main_menu", GameLanguage::English),
        ]
    );
    assert_eq!(
        installation.localization_roots(GameLanguage::SimplifiedChinese),
        vec![scoped("main_menu", GameLanguage::SimplifiedChinese)]
    );
    assert!(
        installation
            .localization_roots(GameLanguage::Korean)
            .is_empty()
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}
