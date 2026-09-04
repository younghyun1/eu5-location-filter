//! Target-specific bounded zstd decompression.

use std::io::Read;

use crate::AppError;

pub(crate) fn decode_limited(compressed: &[u8], limit: u64) -> Result<Vec<u8>, AppError> {
    let mut expanded = Vec::new();
    decoder(compressed, limit)?
        .take(limit.saturating_add(1))
        .read_to_end(&mut expanded)
        .map_err(|error| AppError::Compression(error.to_string()))?;
    if expanded.len() as u64 > limit {
        return Err(AppError::InvalidData(
            "decompressed payload exceeds its size limit".to_owned(),
        ));
    }
    Ok(expanded)
}

#[cfg(not(target_family = "wasm"))]
fn decoder(compressed: &[u8], _limit: u64) -> Result<impl Read, AppError> {
    zstd::stream::read::Decoder::new(compressed)
        .map_err(|error| AppError::Compression(error.to_string()))
}

#[cfg(target_family = "wasm")]
fn decoder(compressed: &[u8], limit: u64) -> Result<impl Read, AppError> {
    ruzstd::decoding::StreamingDecoder::new_with_max_window_size(compressed, limit)
        .map_err(|error| AppError::Compression(error.to_string()))
}
