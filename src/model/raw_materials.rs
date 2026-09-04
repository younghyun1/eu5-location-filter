//! Vanilla raw-material classifications used by filters and row presentation.

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
    use super::{is_food_producing, is_gold_or_silver};

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
