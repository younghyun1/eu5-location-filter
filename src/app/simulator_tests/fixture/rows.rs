//! The table of synthetic locations behind the simulator fixture dataset.

use crate::model::LocationKind;

/// One synthetic location before it is interned into the dictionary.
pub(super) struct Row {
    pub(super) name: &'static str,
    pub(super) key: &'static str,
    pub(super) kind: LocationKind,
    pub(super) topography: &'static str,
    pub(super) continent: &'static str,
    pub(super) coastal: bool,
    pub(super) harbor: Option<f32>,
    pub(super) river: Option<u8>,
}

/// Locations in identifier order.
pub(super) const ROWS: [Row; 9] = [
    row(
        "Castellane",
        "tarn",
        LocationKind::Land,
        "mountains",
        "europe",
        false,
        None,
        None,
    ),
    row(
        "Aldoria",
        "umber",
        LocationKind::Land,
        "flatland",
        "europe",
        true,
        Some(0.5),
        Some(1),
    ),
    row(
        "Fenmark",
        "ridge",
        LocationKind::Land,
        "flatland",
        "asia",
        true,
        Some(0.25),
        None,
    ),
    row(
        "Brevik",
        "quill",
        LocationKind::Land,
        "hills",
        "europe",
        false,
        None,
        None,
    ),
    row(
        "Eskarra",
        "pyre",
        LocationKind::Land,
        "hills",
        "asia",
        false,
        None,
        Some(1),
    ),
    row(
        "Drovnik",
        "oxbow",
        LocationKind::Land,
        "flatland",
        "asia",
        false,
        None,
        None,
    ),
    row(
        "Hale Sound",
        "nave_sea",
        LocationKind::Sea,
        "ocean",
        "ocean_world",
        false,
        None,
        None,
    ),
    row(
        "Ivory Deep",
        "mist_sea",
        LocationKind::Sea,
        "ocean",
        "ocean_world",
        false,
        None,
        None,
    ),
    row(
        "Jagged Waste",
        "ash_waste",
        LocationKind::Impassable,
        "mountain_wasteland",
        "asia",
        false,
        None,
        None,
    ),
];

#[allow(
    clippy::too_many_arguments,
    reason = "a flat table row reads best as one call"
)]
const fn row(
    name: &'static str,
    key: &'static str,
    kind: LocationKind,
    topography: &'static str,
    continent: &'static str,
    coastal: bool,
    harbor: Option<f32>,
    river: Option<u8>,
) -> Row {
    Row {
        name,
        key,
        kind,
        topography,
        continent,
        coastal,
        harbor,
        river,
    }
}
