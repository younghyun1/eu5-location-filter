//! Cross-locale checks: key set, entry kind, plural categories and placeholder names, each
//! against the English catalog.

use std::collections::{BTreeSet, HashMap};

use super::locales::required_categories;
use super::model::{Catalog, Category, Form, Item, Value};
use super::render::variant_name;

/// Checks that every English key yields a distinct, valid enum variant.
pub fn check_variants(english: &Catalog) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen: HashMap<String, &str> = HashMap::new();
    for item in &english.items {
        let variant = variant_name(&item.key);
        if variant == "Self" {
            errors.push(format!(
                "{}:{}: key `{}` would generate the reserved variant `Self`",
                english.path, item.line, item.key
            ));
        }
        if let Some(other) = seen.insert(variant.clone(), &item.key) {
            errors.push(format!(
                "{}:{}: key `{}` and key `{other}` both generate the variant `{variant}`",
                english.path, item.line, item.key
            ));
        }
    }
    errors
}

/// Checks one catalog against English. Passing English as both arguments checks its plural
/// categories and placeholder uniqueness.
pub fn check_catalog(english: &Catalog, catalog: &Catalog) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(required) = required_categories(catalog.tag) else {
        errors.push(format!("{}: unknown locale tag", catalog.path));
        return errors;
    };
    for reference in &english.items {
        match catalog.find(&reference.key) {
            None => errors.push(format!("{}: missing key `{}`", catalog.path, reference.key)),
            Some(item) => check_item(catalog, required, reference, item, &mut errors),
        }
    }
    for item in &catalog.items {
        if english.find(&item.key).is_none() {
            errors.push(format!(
                "{}:{}: key `{}` is not in en-US.txt",
                catalog.path, item.line, item.key
            ));
        }
    }
    errors
}

fn check_item(
    catalog: &Catalog,
    required: &[Category],
    reference: &Item,
    item: &Item,
    errors: &mut Vec<String>,
) {
    let path = &catalog.path;
    let key = &item.key;
    match (&reference.value, &item.value) {
        (Value::Plain(expected), Value::Plain(form)) => {
            let got = names_of(path, key, form, errors);
            let want = name_set(expected);
            if got != want {
                errors.push(format!(
                    "{path}:{}: key `{key}` has placeholders {} but en-US.txt has {}",
                    form.line,
                    show(&got),
                    show(&want)
                ));
            }
        }
        (Value::Plural(expected), Value::Plural(forms)) => {
            let want = expected
                .iter()
                .find(|(category, _)| *category == Category::Other)
                .map(|(_, form)| name_set(form))
                .unwrap_or_default();
            for (category, form) in forms {
                if !required.contains(category) {
                    errors.push(format!(
                        "{path}:{}: key `{key}` has a [{}] form, which {} does not use",
                        form.line,
                        category.name(),
                        catalog.tag
                    ));
                }
                let got = names_of(path, key, form, errors);
                if !got.is_subset(&want) {
                    errors.push(format!(
                        "{path}:{}: [{}] form of `{key}` has placeholders {} but the English other form has {}",
                        form.line,
                        category.name(),
                        show(&got),
                        show(&want)
                    ));
                } else if *category == Category::Other && got != want {
                    errors.push(format!(
                        "{path}:{}: the other form of `{key}` must contain every placeholder: {}",
                        form.line,
                        show(&want)
                    ));
                }
            }
            for category in required {
                if !forms.iter().any(|(known, _)| known == category) {
                    errors.push(format!(
                        "{path}:{}: key `{key}` is missing its [{}] form",
                        item.line,
                        category.name()
                    ));
                }
            }
        }
        (Value::Plain(_), Value::Plural(_)) => errors.push(format!(
            "{path}:{}: key `{key}` has plural forms but is a plain string in en-US.txt",
            item.line
        )),
        (Value::Plural(_), Value::Plain(_)) => errors.push(format!(
            "{path}:{}: key `{key}` is a plain string but has plural forms in en-US.txt",
            item.line
        )),
    }
}

/// Placeholder names of a form as a set, reporting a name that appears more than once.
fn names_of<'a>(
    path: &str,
    key: &str,
    form: &'a Form,
    errors: &mut Vec<String>,
) -> BTreeSet<&'a str> {
    let set = name_set(form);
    if set.len() != form.placeholders.len() {
        errors.push(format!(
            "{path}:{}: key `{key}` repeats a placeholder; each name may appear once",
            form.line
        ));
    }
    set
}

fn name_set(form: &Form) -> BTreeSet<&str> {
    form.placeholders.iter().map(String::as_str).collect()
}

fn show(names: &BTreeSet<&str>) -> String {
    if names.is_empty() {
        return "none".to_owned();
    }
    let list: Vec<String> = names.iter().map(|name| format!("{{{name}}}")).collect();
    list.join(", ")
}
