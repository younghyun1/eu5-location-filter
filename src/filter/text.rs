//! Search normalization for ASCII input against localized names.

/// Lowercases text and folds common Latin letters to searchable ASCII aliases.
///
/// Internal EU5 identifiers remain in the search index as a second fallback for
/// non-Latin names. This function intentionally avoids a runtime-growing cache.
pub(crate) fn fold_search(value: &str) -> String {
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
            other => output.push(other),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::fold_search;

    #[test]
    fn folds_latin_names_for_ascii_search() {
        assert_eq!(fold_search("Kouřim"), "kourim");
        assert_eq!(fold_search("Łódź"), "lodz");
        assert_eq!(fold_search("Ærøskøbing"), "aeroskobing");
        assert_eq!(fold_search("Crème brûlée"), "creme brulee");
    }
}
