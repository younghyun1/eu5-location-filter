//! Which identifiers carry localized labels, and which localization keys an import requests.
//!
//! Localization keys are global in EU5, so one label per symbol is correct even when the
//! symbol is also a location key or appears in two fields. Static modifiers are the exception
//! to keying by identifier (`STATIC_MODIFIER_NAME_<key>`), so a symbol used both as a modifier
//! and as another field would need two labels and is rejected.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::hierarchy::RawHierarchy;
use super::templates::RawTemplate;
use crate::AppError;
use crate::lang_pack::LabelField;

/// Records that `key` is shown through `field`, keeping the first field seen.
///
/// Fails when one key is used both as a static modifier and as another field's value, because
/// the two would resolve their label through different localization keys. `name` is the
/// identifier text for the error message.
pub(crate) fn record_use<K: Ord>(
    uses: &mut BTreeMap<K, LabelField>,
    key: K,
    name: &str,
    field: LabelField,
) -> Result<(), AppError> {
    match uses.get(&key) {
        None => {
            uses.insert(key, field);
            Ok(())
        }
        Some(existing) => compatible(*existing, field, name),
    }
}

/// Two uses of one identifier can share a label unless exactly one of them is a modifier.
fn compatible(existing: LabelField, field: LabelField, name: &str) -> Result<(), AppError> {
    if (existing == LabelField::Modifier) == (field == LabelField::Modifier) {
        return Ok(());
    }
    Err(AppError::InvalidData(format!(
        "{name} is used as both a {} and a {} value",
        existing.key(),
        field.key()
    )))
}

/// Every identifier used in a labelled field, ordered by identifier text.
pub(super) fn labelled_identifiers(
    templates: &[RawTemplate],
    hierarchy: &HashMap<String, RawHierarchy>,
) -> Result<BTreeMap<String, LabelField>, AppError> {
    let mut uses = BTreeMap::new();
    for template in templates {
        let optional = [
            (LabelField::Vegetation, &template.vegetation),
            (LabelField::Climate, &template.climate),
            (LabelField::Religion, &template.religion),
            (LabelField::Culture, &template.culture),
            (LabelField::RawMaterial, &template.raw_material),
            (LabelField::Modifier, &template.modifier),
        ];
        add(&mut uses, &template.topography, LabelField::Topography)?;
        for (field, value) in optional {
            if let Some(value) = value {
                add(&mut uses, value, field)?;
            }
        }
        if let Some(value) = hierarchy.get(&template.key) {
            add(&mut uses, &value.continent, LabelField::Continent)?;
            add(&mut uses, &value.subcontinent, LabelField::Subcontinent)?;
            add(&mut uses, &value.region, LabelField::Region)?;
            add(&mut uses, &value.area, LabelField::Area)?;
            add(&mut uses, &value.province, LabelField::Province)?;
        }
    }
    Ok(uses)
}

fn add(
    uses: &mut BTreeMap<String, LabelField>,
    value: &str,
    field: LabelField,
) -> Result<(), AppError> {
    // Looks up by `&str` first so the common repeated identifier costs no allocation.
    match uses.get(value) {
        Some(existing) => compatible(*existing, field, value),
        None => {
            uses.insert(value.to_owned(), field);
            Ok(())
        }
    }
}

/// Localization keys the English import reads: every location key for names, plus each
/// labelled identifier's key under its field's rule.
pub(super) fn requested_keys(
    templates: &[RawTemplate],
    labelled: &BTreeMap<String, LabelField>,
) -> BTreeSet<String> {
    let mut keys: BTreeSet<String> = templates
        .iter()
        .map(|template| template.key.clone())
        .collect();
    for (identifier, field) in labelled {
        keys.insert(field.localization_key(identifier).into_owned());
    }
    keys
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use super::{labelled_identifiers, record_use, requested_keys};
    use crate::import::hierarchy::RawHierarchy;
    use crate::import::templates::RawTemplate;
    use crate::lang_pack::LabelField;

    fn template(key: &str, culture: Option<&str>, modifier: Option<&str>) -> RawTemplate {
        RawTemplate {
            key: key.to_owned(),
            topography: "flatland".to_owned(),
            vegetation: None,
            climate: None,
            religion: None,
            culture: culture.map(str::to_owned),
            raw_material: None,
            modifier: modifier.map(str::to_owned),
            harbor_suitability: None,
            movement_assistance: None,
        }
    }

    fn hierarchy(keys: &[&str]) -> HashMap<String, RawHierarchy> {
        keys.iter()
            .map(|key| {
                let value = RawHierarchy {
                    continent: "europe".to_owned(),
                    subcontinent: "western_europe".to_owned(),
                    region: "iberia_region".to_owned(),
                    area: "limousin_province".to_owned(),
                    province: "limousin_province".to_owned(),
                };
                ((*key).to_owned(), value)
            })
            .collect()
    }

    #[test]
    fn keeps_location_key_collisions_and_prefixes_modifiers() {
        let templates = vec![
            template("pende", Some("teke"), Some("potosi_base")),
            template("teke", Some("pende"), None),
        ];
        let labelled = labelled_identifiers(&templates, &hierarchy(&["pende", "teke"]));
        assert!(labelled.is_ok());
        let Ok(labelled) = labelled else { return };
        assert_eq!(labelled.get("pende"), Some(&LabelField::Culture));
        assert_eq!(labelled.get("limousin_province"), Some(&LabelField::Area));
        assert_eq!(labelled.get("potosi_base"), Some(&LabelField::Modifier));
        let keys = requested_keys(&templates, &labelled);
        assert!(keys.contains("STATIC_MODIFIER_NAME_potosi_base"));
        assert!(!keys.contains("potosi_base"));
        assert!(keys.contains("pende") && keys.contains("teke"));
        assert!(keys.contains("flatland") && keys.contains("europe"));
    }

    #[test]
    fn rejects_a_modifier_that_is_also_another_value() {
        let templates = vec![template("pende", Some("teke"), Some("teke"))];
        let labelled = labelled_identifiers(&templates, &hierarchy(&["pende"]));
        assert!(labelled.is_err());
        let mut uses = BTreeMap::new();
        assert!(record_use(&mut uses, 1_u32, "a", LabelField::Area).is_ok());
        assert!(record_use(&mut uses, 1_u32, "a", LabelField::Province).is_ok());
        assert!(record_use(&mut uses, 1_u32, "a", LabelField::Modifier).is_err());
        assert_eq!(uses.get(&1), Some(&LabelField::Area));
    }
}
