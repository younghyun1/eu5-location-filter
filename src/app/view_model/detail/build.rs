//! Detail construction for one location.

use super::groups::Fields;
use super::model::LocationDetail;
use crate::app::i18n::{Locale, Text, format, t};
use crate::app::view_model::row::Digits;
use crate::lang_pack::Labels;
use crate::model::LocationId;

/// Builds the detail for a location, or `None` when the ID is not in the dataset.
#[must_use]
pub(crate) fn build(labels: Labels<'_>, locale: Locale, id: LocationId) -> Option<LocationDetail> {
    let dataset = labels.dataset();
    let record = dataset.location(id)?;
    let none = t(locale, Text::ValueNone);
    let [red, green, blue] = record.color.components();
    let (red, green, blue) = (
        Digits::new(u64::from(red)),
        Digits::new(u64::from(green)),
        Digits::new(u64::from(blue)),
    );
    let rgb = format(
        t(locale, Text::FormatRgb),
        &[
            ("red", red.as_str()),
            ("green", green.as_str()),
            ("blue", blue.as_str()),
        ],
    );
    let hierarchy = &record.hierarchy;
    let breadcrumb = [
        hierarchy.continent,
        hierarchy.subcontinent,
        hierarchy.region,
        hierarchy.area,
        hierarchy.province,
    ]
    .map(|symbol| labels.label(symbol).unwrap_or(none).to_owned())
    .to_vec();
    let view = Fields::new(labels, locale, record, none);
    Some(LocationDetail {
        id,
        color: record.color,
        hex: record.color.hex(),
        rgb,
        name: labels.name(id).unwrap_or(none).to_owned(),
        key: dataset.symbol(record.key).unwrap_or(none).to_owned(),
        breadcrumb,
        groups: vec![
            view.classification(),
            view.society(),
            view.economy(),
            view.coast(),
            view.river(),
            view.capacity(),
        ],
    })
}

