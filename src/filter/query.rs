//! AND across fields, OR within a field, using two fixed-size query masks.

use bitcode::{Decode, Encode};

use super::{
    FilterSet, FloatRange,
    bitmap::Bitmap,
    facets::{self, FacetKey as K, StoredFacet},
    numeric::{NumericField as N, NumericIndex},
    search::{self, Trigram},
};
use crate::{
    AppError,
    model::{Dataset, LocationKind},
};

#[derive(Clone, Debug, Default, Decode, Encode, PartialEq)]
pub(super) struct QueryIndex {
    pub(super) facets: Vec<StoredFacet>,
    pub(super) numeric: Vec<NumericIndex>,
    pub(super) trigrams: Vec<Trigram>,
}

impl QueryIndex {
    pub(super) fn build(dataset: &Dataset, searchable: &[String]) -> Self {
        Self {
            facets: facets::build(dataset),
            numeric: N::ALL
                .into_iter()
                .map(|field| NumericIndex::build(dataset, field))
                .collect(),
            trigrams: search::build(searchable),
        }
    }

    pub(super) fn validate(
        &self,
        dataset: &Dataset,
        searchable: &[String],
    ) -> Result<(), AppError> {
        facets::validate(&self.facets, dataset)?;
        if self.numeric.len() != N::ALL.len()
            || self
                .numeric
                .iter()
                .zip(N::ALL)
                .any(|(index, field)| index.field != field)
        {
            return Err(AppError::InvalidData(
                "numeric index fields are missing or duplicated".to_owned(),
            ));
        }
        for index in &self.numeric {
            index.validate(dataset)?;
        }
        search::validate(&self.trigrams, searchable)
    }

    pub(super) fn apply(
        &self,
        dataset: &Dataset,
        searchable: &[String],
        filters: &FilterSet,
    ) -> Bitmap {
        let count = dataset.stored.locations.len();
        let mut mask = Bitmap::full(count);
        let mut scratch = Bitmap::empty(count);
        if let Some(rgb) = filters.rgb {
            mask.clear();
            if let Some(id) = dataset.by_color.get(&rgb) {
                mask.insert(*id);
            }
        }
        if !filters.show_impassable {
            self.union(K::Kind(LocationKind::Impassable), &mut scratch);
            mask.subtract(&scratch);
        }
        self.select(
            filters.kinds.iter().copied().map(K::Kind),
            &mut mask,
            &mut scratch,
        );
        self.select(
            filters.topographies.iter().copied().map(K::Topography),
            &mut mask,
            &mut scratch,
        );
        for (values, key) in [
            (&filters.vegetation, K::Vegetation as fn(_) -> K),
            (&filters.climates, K::Climate),
            (&filters.continents, K::Continent),
            (&filters.subcontinents, K::Subcontinent),
            (&filters.regions, K::Region),
            (&filters.areas, K::Area),
            (&filters.provinces, K::Province),
            (&filters.religions, K::Religion),
            (&filters.cultures, K::Culture),
            (&filters.raw_materials, K::RawMaterial),
            (&filters.modifiers, K::Modifier),
        ] {
            self.select(values.iter().copied().map(key), &mut mask, &mut scratch);
        }
        for (values, key) in [
            (&filters.coastal, K::Coastal as fn(_) -> K),
            (&filters.river_presence, K::River),
            (&filters.harbor_presence, K::Harbor),
            (&filters.movement_presence, K::Movement),
        ] {
            self.select(values.iter().copied().map(key), &mut mask, &mut scratch);
        }
        if filters.food_producing_only {
            self.select(std::iter::once(K::Food(true)), &mut mask, &mut scratch);
        }
        self.range(
            N::River,
            FloatRange {
                min: filters.river_level_min.iter().min().copied().map(f32::from),
                max: filters.river_level_max.iter().max().copied().map(f32::from),
            },
            filters
                .river_level_min
                .is_empty()
                .then_some(K::River(false)),
            &mut mask,
            &mut scratch,
        );
        self.range(
            N::Harbor,
            filters.harbor_range,
            None,
            &mut mask,
            &mut scratch,
        );
        self.range(
            N::MovementX,
            filters.movement_x,
            None,
            &mut mask,
            &mut scratch,
        );
        self.range(
            N::MovementY,
            filters.movement_y,
            None,
            &mut mask,
            &mut scratch,
        );
        search::apply(
            &self.trigrams,
            searchable,
            &super::fold_search(&filters.search),
            &mut mask,
            &mut scratch,
        );
        mask
    }

    fn union(&self, key: K, target: &mut Bitmap) {
        if let Some(posting) = facets::lookup(&self.facets, key) {
            posting.union_into(target);
        }
    }

    fn select(
        &self,
        keys: impl ExactSizeIterator<Item = K>,
        mask: &mut Bitmap,
        scratch: &mut Bitmap,
    ) {
        if keys.len() == 0 {
            return;
        }
        scratch.clear();
        for key in keys {
            self.union(key, scratch);
        }
        mask.intersect(scratch);
    }

    fn range(
        &self,
        field: N,
        range: FloatRange,
        missing: Option<K>,
        mask: &mut Bitmap,
        scratch: &mut Bitmap,
    ) {
        if range.min.is_none() && range.max.is_none() {
            return;
        }
        scratch.clear();
        if let Some(index) = self.numeric.iter().find(|index| index.field == field) {
            index.union_range(range, scratch);
        }
        // A maximum river tier historically includes riverless locations; minimum
        // tiers and bounded harbor/movement values exclude missing data.
        if let Some(key) = missing {
            self.union(key, scratch);
        }
        mask.intersect(scratch);
    }
}
