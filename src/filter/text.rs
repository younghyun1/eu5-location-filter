//! Sort and search normalization for localized names.

#[derive(Clone, Copy, PartialEq, Eq)]
enum FoldMode {
    Sort,
    Search,
}

/// Lowercases text, folds common Latin letters, and ignores search separators.
///
/// Internal EU5 identifiers remain in the search index as a second fallback for
/// non-Latin names. Alphanumeric characters from every script are retained,
/// while punctuation and whitespace are optional for user input. The NUL
/// separator between a location's name and identifier remains a hard boundary.
pub(crate) fn fold_search(value: &str) -> String {
    fold(value, FoldMode::Search)
}

/// Produces the previous punctuation-preserving key used for alphabetical sort.
pub(super) fn fold_sort(value: &str) -> String {
    fold(value, FoldMode::Sort)
}

fn fold(value: &str, mode: FoldMode) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars().flat_map(char::to_lowercase) {
        match character {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ǟ' | 'ǡ' | 'ǻ' | 'ȁ'
            | 'ȃ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => {
                output.push('a')
            }
            'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => output.push('c'),
            'ď' | 'đ' | 'ḋ' | 'ḍ' | 'ḏ' | 'ḓ' => output.push('d'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ȅ' | 'ȇ' | 'ẹ' | 'ẻ' | 'ẽ'
            | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => output.push('e'),
            'ƒ' => output.push('f'),
            'ĝ' | 'ğ' | 'ġ' | 'ģ' | 'ǧ' | 'ǵ' | 'ḡ' => output.push('g'),
            'ĥ' | 'ħ' | 'ȟ' | 'ḣ' | 'ḥ' | 'ḧ' | 'ḩ' | 'ḫ' => output.push('h'),
            'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' | 'ȉ' | 'ȋ' | 'ị' | 'ỉ' => {
                output.push('i')
            }
            'ĵ' | 'ǰ' => output.push('j'),
            'ķ' | 'ƙ' | 'ǩ' | 'ḱ' | 'ḳ' | 'ḵ' => output.push('k'),
            'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' | 'ḷ' | 'ḹ' | 'ḻ' | 'ḽ' => output.push('l'),
            'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' | 'ǹ' | 'ṅ' | 'ṇ' | 'ṉ' | 'ṋ' => {
                output.push('n');
            }
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' | 'ǒ' | 'ǫ' | 'ǭ' | 'ǿ' | 'ȍ'
            | 'ȏ' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => {
                output.push('o')
            }
            'ŕ' | 'ŗ' | 'ř' | 'ȑ' | 'ȓ' | 'ṙ' | 'ṛ' | 'ṝ' | 'ṟ' => output.push('r'),
            'ś' | 'ŝ' | 'ş' | 'š' | 'ș' | 'ṡ' | 'ṣ' | 'ṥ' | 'ṧ' | 'ṩ' => {
                output.push('s');
            }
            'ţ' | 'ť' | 'ŧ' | 'ț' | 'ṫ' | 'ṭ' | 'ṯ' | 'ṱ' => output.push('t'),
            'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ǔ' | 'ǖ' | 'ǘ' | 'ǚ'
            | 'ǜ' | 'ȕ' | 'ȗ' | 'ụ' | 'ủ' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => {
                output.push('u')
            }
            'ŵ' | 'ẁ' | 'ẃ' | 'ẅ' | 'ẇ' | 'ẉ' => output.push('w'),
            'ý' | 'ÿ' | 'ŷ' | 'ȳ' | 'ẏ' | 'ỳ' | 'ỵ' | 'ỷ' | 'ỹ' => output.push('y'),
            'ź' | 'ż' | 'ž' | 'ẑ' | 'ẓ' | 'ẕ' => output.push('z'),
            'æ' | 'ǽ' | 'ǣ' => output.push_str("ae"),
            'œ' => output.push_str("oe"),
            'ß' => output.push_str("ss"),
            'þ' => output.push_str("th"),
            'ð' => output.push('d'),
            '\u{0300}'..='\u{036f}' => {}
            other if mode == FoldMode::Search && ignored_search_character(other) => {}
            other => output.push(other),
        }
    }
    output
}

fn ignored_search_character(character: char) -> bool {
    character != '\0'
        && (!character.is_alphanumeric()
            || matches!(
                character,
                '\u{02b9}'
                    | '\u{02bb}'
                    | '\u{02bc}'
                    | '\u{02bd}'
                    | '\u{02be}'
                    | '\u{02bf}'
                    | '\u{02c8}'
                    | '\u{a78c}'
            ))
}

#[cfg(test)]
mod tests {
    use super::{fold_search, fold_sort};

    #[test]
    fn folds_latin_names_for_ascii_search() {
        assert_eq!(fold_search("Kouřim"), "kourim");
        assert_eq!(fold_search("Łódź"), "lodz");
        assert_eq!(fold_search("Ærøskøbing"), "aeroskobing");
        assert_eq!(fold_search("Crème brûlée"), "cremebrulee");
    }

    #[test]
    fn ignores_apostrophe_and_transliteration_variants() {
        for value in [
            "N'Goussa",
            "N’Goussa",
            "N‘Goussa",
            "N‛Goussa",
            "NʹGoussa",
            "NʻGoussa",
            "NʼGoussa",
            "NʽGoussa",
            "NʾGoussa",
            "NʿGoussa",
            "NˈGoussa",
            "NꞌGoussa",
            "N＇Goussa",
        ] {
            assert_eq!(fold_search(value), "ngoussa", "failed to fold {value:?}");
        }
    }

    #[test]
    fn ignores_whitespace_width_and_multiplicity() {
        for value in [
            "Abu Dhabi",
            "  Abu   Dhabi  ",
            "Abu\tDhabi",
            "Abu\nDhabi",
            "Abu\u{00a0}Dhabi",
            "Abu\u{2003}Dhabi",
            "Abu\u{202f}Dhabi",
            "Abu\u{3000}Dhabi",
        ] {
            assert_eq!(fold_search(value), "abudhabi", "failed to fold {value:?}");
        }
    }

    #[test]
    fn ignores_dashes_underscores_and_general_punctuation() {
        for value in [
            "Al-Qa'im",
            "Al_Qa'im",
            "Al‐Qa’im",
            "Al‑Qaʼim",
            "Al–Qaʿim",
            "Al—Qaʾim",
            "Al/Qa.im",
            "Al (Qa'im)",
        ] {
            assert_eq!(fold_search(value), "alqaim", "failed to fold {value:?}");
        }
        assert_eq!(fold_search("St. John's / East (Old)"), "stjohnseastold");
        assert_eq!(fold_search("A+B&C"), "abc");
    }

    #[test]
    fn retains_alphanumeric_characters_from_other_scripts() {
        assert_eq!(fold_search("القاهرة"), "القاهرة");
        assert_eq!(fold_search("東京 42"), "東京42");
        assert_eq!(fold_search("Αθήνα"), "αθήνα");
    }

    #[test]
    fn preserves_name_identifier_boundary() {
        let folded = fold_search("north\0south");
        assert_eq!(folded, "north\0south");
        assert!(!folded.contains("hs"));
    }

    #[test]
    fn sort_folding_keeps_punctuation_and_spacing() {
        assert_eq!(fold_sort("N’Goussa  East-West"), "n’goussa  east-west");
        assert_eq!(fold_sort("Crème brûlée"), "creme brulee");
    }
}
