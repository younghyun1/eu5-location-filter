//! Deterministic bounded string interning used only during import.

use std::collections::HashMap;

use super::{MAX_DICTIONARY_BYTES, MAX_SYMBOLS, SymbolId};
use crate::AppError;

/// Replaces repeated text with stable dictionary indexes.
pub struct StringInterner {
    values: Vec<String>,
    lookup: HashMap<String, SymbolId>,
    bytes: usize,
}

impl StringInterner {
    /// Creates an empty interner.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            lookup: HashMap::new(),
            bytes: 0,
        }
    }

    /// Interns a value while enforcing dictionary count and byte limits.
    pub fn intern(&mut self, value: &str) -> Result<SymbolId, AppError> {
        if let Some(id) = self.lookup.get(value) {
            return Ok(*id);
        }
        if self.values.len() >= MAX_SYMBOLS
            || self.bytes.saturating_add(value.len()) > MAX_DICTIONARY_BYTES
        {
            return Err(AppError::InvalidData(
                "string dictionary exceeds configured limits".to_owned(),
            ));
        }
        let raw_id = u32::try_from(self.values.len()).map_err(|error| {
            AppError::InvalidData(format!("dictionary index overflow: {error}"))
        })?;
        let owned = value.to_owned();
        let id = SymbolId(raw_id);
        self.bytes += owned.len();
        self.values.push(owned.clone());
        self.lookup.insert(owned, id);
        Ok(id)
    }

    /// Resolves an interned value.
    #[must_use]
    pub fn resolve(&self, id: SymbolId) -> Option<&str> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.values.get(index))
            .map(String::as_str)
    }

    /// Consumes the interner into storage order.
    #[must_use]
    pub fn into_values(self) -> Vec<String> {
        self.values
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::StringInterner;

    #[test]
    fn reuses_ids() {
        let mut interner = StringInterner::new();
        let first = interner.intern("stockholm");
        let second = interner.intern("stockholm");
        assert!(first.is_ok());
        assert_eq!(first.ok(), second.ok());
    }
}
