//! Header: brand, global search, result count and build text, and the header buttons; the
//! narrow layout stacks Filters, Details and About, the search, and a status line.

mod frame;
mod narrow;
mod search;
mod status;
#[cfg(test)]
mod tests;
mod view;
mod wide;

pub(crate) use status::HeaderStatus;
pub(crate) use view::view;
