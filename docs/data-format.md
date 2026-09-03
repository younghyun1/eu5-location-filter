# Data format

The default `eu5-locations.bitcode.zst` file is a zstd frame containing a fixed header and a bitcode payload. The header is `EU5LOC\0\1`, followed by the decompressed payload length as a little-endian `u64`. The encoded `StoredDataset` repeats the schema version and contains the EU5 app and build IDs, river-width metadata, one string dictionary, location records, and import diagnostics.

No timestamps or machine paths are stored. Import order and recursive localization traversal are deterministic. Repeated strings are represented by bounded `SymbolId` values; locations, map colors, and river levels use separate typed integer representations.

Readers reject unknown schema versions, malformed headers, compressed files over 256 MiB, expanded payloads over 128 MiB, out-of-range symbols, duplicate identifiers or colors, and records beyond the configured location limit. Writes use a same-directory temporary file. During replacement, the prior valid blob is moved to a backup and restored if validation or rename fails.
