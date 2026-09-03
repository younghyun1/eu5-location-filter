# Data format

`assets/eu5-locations.bitcode.zst` is a level-22 zstd frame containing a fixed header and bitcode payload. The header is `EU5LOC\0\1`, followed by the decompressed payload length as a little-endian `u64`. The encoded `StoredDataset` repeats the schema version and contains the EU5 app and build IDs, river-width metadata, one string dictionary, location records, and import diagnostics.

`assets/eu5-indexes.bitcode.zst` is a separate level-22 zstd frame with the `EU5IDX\0\1` envelope. Its bitcode payload contains ASCII-folded name and identifier search strings plus ascending and descending `LocationId` permutations for every result column. It records the paired app ID, build ID, and location count. Runtime loading rejects a mismatched pair.

No timestamps or machine paths are stored. Import order and recursive localization traversal are deterministic. Repeated strings are represented by bounded `SymbolId` values; locations, map colors, and river levels use separate typed integer representations.

Readers reject unknown schema versions, malformed headers, compressed files over 256 MiB, expanded payloads over 128 MiB, out-of-range symbols, duplicate identifiers or colors, and records beyond the configured location limit. Writes use a same-directory temporary file. During replacement, the prior valid blob is moved to a backup and restored if validation or rename fails.

Both committed frames are copied into Cargo's output directory by `build.rs` and embedded with `include_bytes!`. The build script performs no Steam discovery, EU5 parsing, or bundle generation; a missing committed bundle fails the build.
