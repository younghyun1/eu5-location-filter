#![deny(
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used
)]
#![warn(missing_docs)]

//! Import, storage, filtering, and UI support for EU5 map locations.

pub mod embedded;
pub mod error;
pub mod filter;
pub mod import;
pub mod index_storage;
pub mod model;
pub mod parser;
pub mod steam;
pub mod storage;
pub mod ui;

pub use error::AppError;
pub use model::{Dataset, LocationId, LocationRecord, MapColor, StoredDataset, SymbolId};
