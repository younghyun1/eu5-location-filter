//! Checklist option lists per facet with labels folded once per build, and option search.
//!
//! Dataset options are sorted by display label; static options keep their contract order. Each
//! option carries its wire key (symbol identifier, kind word, `yes` or `no`, `present` or
//! `missing`, tier digits, or `!` for the missing value), which is how view-state items resolve
//! without scanning the dictionary.

mod facet_options;
mod list;
mod statics;
#[cfg(test)]
pub(in crate::app::view_model) mod test_data;
mod texts;

pub(crate) use facet_options::FacetOptions;
pub(crate) use list::{FacetOption, OptionList, OptionValue};
pub(super) use texts::{kind_text, river_tier_text};
