//! Vanilla raw-material classifications used by filters and row presentation.

const RAW_MATERIAL_ICONS: [(&str, &str); 52] = [
    ("alum", "🧪"),
    ("amber", "🟠"),
    ("beeswax", "🕯️"),
    ("chili", "🌶️"),
    ("clay", "🏺"),
    ("cloves", "🌸"),
    ("coal", "⚫"),
    ("cocoa", "🍫"),
    ("coffee", "☕"),
    ("copper", "🟤"),
    ("cotton", "☁️"),
    ("dyes", "🎨"),
    ("elephants", "🐘"),
    ("fiber_crops", "🌿"),
    ("fish", "🐟"),
    ("fruit", "🍎"),
    ("fur", "🦊"),
    ("gems", "💎"),
    ("goods_gold", "🥇"),
    ("horses", "🐎"),
    ("incense", "🪔"),
    ("iron", "🔩"),
    ("ivory", "🦷"),
    ("lead", "⚙️"),
    ("legumes", "🫘"),
    ("livestock", "🐄"),
    ("lumber", "🪵"),
    ("maize", "🌽"),
    ("marble", "🏛️"),
    ("medicaments", "💊"),
    ("mercury", "🌡️"),
    ("millet", "🌾"),
    ("olives", "🫒"),
    ("pearls", "🦪"),
    ("pepper", "⚱️"),
    ("potato", "🥔"),
    ("rice", "🍚"),
    ("saffron", "🌼"),
    ("salt", "🧂"),
    ("saltpeter", "💥"),
    ("sand", "🏖️"),
    ("silk", "🧵"),
    ("silver", "🥈"),
    ("stone", "🪨"),
    ("sugar", "🍬"),
    ("tea", "🍵"),
    ("tin", "🥫"),
    ("tobacco", "🚬"),
    ("wheat", "🥖"),
    ("wild_game", "🦌"),
    ("wine", "🍷"),
    ("wool", "🐑"),
];

/// Prefixes a known vanilla raw material with its stable, unique glyph.
pub(crate) fn raw_material_display(key: &str, label: &str) -> String {
    raw_material_icon(key).map_or_else(|| label.to_owned(), |icon| format!("{icon}  {label}"))
}

pub(crate) fn raw_material_icon(key: &str) -> Option<&'static str> {
    RAW_MATERIAL_ICONS
        .binary_search_by_key(&key, |(candidate, _)| *candidate)
        .ok()
        .and_then(|index| RAW_MATERIAL_ICONS.get(index))
        .map(|(_, icon)| *icon)
}

/// Returns whether the vanilla good contributes food when produced by an RGO.
pub(crate) fn is_food_producing(key: &str) -> bool {
    matches!(
        key,
        "wool"
            | "wild_game"
            | "fur"
            | "fish"
            | "wheat"
            | "maize"
            | "rice"
            | "millet"
            | "legumes"
            | "potato"
            | "livestock"
            | "olives"
            | "fruit"
            | "beeswax"
    )
}

/// Returns whether the row should use the precious-metal highlight.
pub(crate) fn is_gold_or_silver(key: &str) -> bool {
    matches!(key, "goods_gold" | "silver")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{RAW_MATERIAL_ICONS, is_food_producing, is_gold_or_silver, raw_material_icon};

    #[test]
    fn every_vanilla_raw_material_has_a_unique_icon() {
        let icons: HashSet<&str> = RAW_MATERIAL_ICONS.iter().map(|(_, icon)| *icon).collect();
        assert_eq!(icons.len(), RAW_MATERIAL_ICONS.len());
        for (material, icon) in RAW_MATERIAL_ICONS {
            assert_eq!(raw_material_icon(material), Some(icon));
        }
    }

    #[test]
    fn recognizes_every_vanilla_food_producing_raw_material() {
        for key in [
            "wool",
            "wild_game",
            "fur",
            "fish",
            "wheat",
            "maize",
            "rice",
            "millet",
            "legumes",
            "potato",
            "livestock",
            "olives",
            "fruit",
            "beeswax",
        ] {
            assert!(is_food_producing(key), "food classification omitted {key}");
        }
    }

    #[test]
    fn rejects_non_food_raw_materials() {
        for key in ["gold", "silver", "salt", "wine", "sugar", "iron", ""] {
            assert!(!is_food_producing(key), "misclassified {key} as food");
        }
    }

    #[test]
    fn recognizes_only_gold_and_silver_as_highlighted_metals() {
        assert!(is_gold_or_silver("goods_gold"));
        assert!(is_gold_or_silver("silver"));
        for key in [
            "gold",
            "goods_silver",
            "copper",
            "iron",
            "tin",
            "lead",
            "gems",
            "",
        ] {
            assert!(
                !is_gold_or_silver(key),
                "misclassified {key} as gold or silver"
            );
        }
    }
}
