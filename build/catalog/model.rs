//! Parsed catalog data shared by the checks and the code generator.

use std::collections::HashMap;

/// CLDR plural category written in square brackets after a key.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Category {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl Category {
    /// Every category in CLDR order.
    const ALL: [Self; 6] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Few,
        Self::Many,
        Self::Other,
    ];

    /// Lower-case CLDR name, as written in catalog files.
    pub fn name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    /// Category for a name written in a catalog file.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == text)
    }
}

/// One decoded value with its source line.
#[derive(Debug)]
pub struct Form {
    /// Value with backslash escapes decoded; brace syntax is kept when the value has
    /// placeholders and decoded when it has none.
    pub text: String,
    /// One-based source line for error messages.
    pub line: usize,
    /// Placeholder names in order of appearance, duplicates included.
    pub placeholders: Vec<String>,
}

/// The value of one base key.
#[derive(Debug)]
pub enum Value {
    /// A string without plural forms.
    Plain(Form),
    /// Plural forms in file order.
    Plural(Vec<(Category, Form)>),
}

/// One base key with its value.
#[derive(Debug)]
pub struct Item {
    /// Dotted key without the plural suffix.
    pub key: String,
    /// Line of the first entry of this key.
    pub line: usize,
    /// Plain value or plural forms.
    pub value: Value,
}

/// One parsed locale file.
#[derive(Debug)]
pub struct Catalog {
    /// Site locale tag, one of `LOCALE_TAGS`.
    pub tag: &'static str,
    /// File path used in error messages.
    pub path: String,
    /// Items in order of first appearance.
    pub items: Vec<Item>,
    /// Position of each key in `items`.
    index: HashMap<String, usize>,
}

impl Catalog {
    /// Builds a catalog and its key index.
    pub fn new(tag: &'static str, path: String, items: Vec<Item>) -> Self {
        let index = items
            .iter()
            .enumerate()
            .map(|(position, item)| (item.key.clone(), position))
            .collect();
        Self {
            tag,
            path,
            items,
            index,
        }
    }

    /// Item for a base key.
    pub fn find(&self, key: &str) -> Option<&Item> {
        self.index
            .get(key)
            .and_then(|position| self.items.get(*position))
    }
}
