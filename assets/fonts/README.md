# Fonts

This directory holds the generated font files that the application embeds (the IBM Plex faces) or serves as lazily fetched packs (the script faces), plus `manifest.json`. The files are produced by the `fonts` subcommand from source fonts that are not stored in the repository. Nothing here is edited by hand.

## Source fonts

Place these files in one directory and pass it as `--source-dir`. Use static instances at regular, medium or semibold weight as named; a variable font must be instanced to the named weight first.

| Output | Source file | Source family | Content |
| --- | --- | --- | --- |
| `plex-sans-regular.ttf` | `IBMPlexSans-Regular.ttf` | IBM Plex Sans | Latin, Latin Extended including Vietnamese, Cyrillic, punctuation |
| `plex-sans-medium.ttf` | `IBMPlexSans-Medium.ttf` | IBM Plex Sans | same |
| `plex-sans-semibold.ttf` | `IBMPlexSans-SemiBold.ttf` | IBM Plex Sans | same |
| `plex-mono-regular.ttf` | `IBMPlexMono-Regular.ttf` | IBM Plex Mono | same |
| `plex-mono-medium.ttf` | `IBMPlexMono-Medium.ttf` | IBM Plex Mono | same |
| `latin-fallback.ttf` | `NotoSans-Regular.ttf` | Noto Sans | only the characters of the embedded-font locales that IBM Plex Sans lacks |
| `plex-sans-kr.ttf` | `IBMPlexSansKR-Regular.ttf` | IBM Plex Sans KR | Korean catalog and language pack text |
| `plex-sans-jp.ttf` | `IBMPlexSansJP-Regular.ttf` | IBM Plex Sans JP | Japanese catalog and language pack text |
| `sans-sc.ttf` | `IBMPlexSansSC-Regular.ttf` | IBM Plex Sans SC | Simplified Chinese catalog and language pack text |
| `sans-tc.ttf` | `IBMPlexSansTC-Regular.ttf` | IBM Plex Sans TC | Traditional Chinese catalog text |
| `noto-sans-arabic.ttf` | `NotoSansArabic-Regular.ttf` | Noto Sans Arabic | whole font |
| `noto-sans-devanagari.ttf` | `NotoSansDevanagari-Regular.ttf` | Noto Sans Devanagari | whole font |
| `noto-sans-bengali.ttf` | `NotoSansBengali-Regular.ttf` | Noto Sans Bengali | whole font |
| `noto-sans-telugu.ttf` | `NotoSansTelugu-Regular.ttf` | Noto Sans Telugu | whole font |

The output names are fixed. The five Plex names and `latin-fallback` are copied by `build.rs` into the binary, and the eight script names are the `fonts/<name>.ttf` paths that the host protocol allows the browser build to fetch. The script packs must not be renamed without changing both.

## What each output contains

- **Plex faces.** Printable ASCII, Latin-1, Latin Extended-A, the Vietnamese letters, basic Cyrillic, and the punctuation the interface draws, including U+00D7 (multiplication sign) and the guillemets U+2039 and U+203A. Wider Latin Extended, Cyrillic Extended and general punctuation ranges are kept when the source has them. The three Sans faces also contain every character of the catalogs and language packs of the locales that use the embedded font, so a translated string never lacks a glyph.
- **CJK packs.** Printable ASCII plus every character in the matching locale's catalog (`assets/i18n/<tag>.txt`, values only) and in the matching language pack's strings, which hold the localized labels and location names. Traditional Chinese has no language pack, so only its catalog counts.
- **Complex-script packs.** The source file is copied unchanged, because subsetting would risk the shaping tables. The tool still checks that the script characters of the matching catalogs are mapped.

## Command

Language packs must exist first, so run the import (which writes `assets/lang`) before this command.

```sh
eu5-location-filter fonts --source-dir <directory with the source fonts>
```

The defaults are `--output-dir assets/fonts`, `--catalog-dir assets/i18n` and the global `--lang-dir`. Every output is built and verified before any file is written, so a failed run leaves the previous files unchanged. For each output the tool reads the written font's `cmap` and fails, listing the missing codepoints, when a required codepoint is absent. It also checks the source first, so a source font that lacks a script character is named directly. Packs larger than the 16 MiB fetch limit are rejected.

A catalog that does not exist yet is noted in the manifest and contributes nothing. A missing or ambiguous language pack is an error.

## Manifest

`manifest.json` has one entry per output: file name, source file, SHA-256 of the source, the source's version string from its `name` table, the number of mapped codepoints, byte size, SHA-256 of the output, whether the source was copied whole, and notes. Commit it together with the fonts so the source version of every file is on record.

## Subsetting

The subsetting step is one function, `font_pack::subset::subset_font`, built on the `fontcull-klippa` subsetter. It keeps the family and style names, because the application selects faces by family name, and the layout tables of the kept glyphs.

Location names use a few transliteration letters that IBM Plex Sans does not have. The Plex faces therefore keep catalog and pack text only where the source has it, and `latin-fallback.ttf` carries exactly the characters the regular Plex face lacks, so those names never show a missing glyph. Game text also contains an occasional character from another script's repertoire, such as a Simplified Chinese form inside Japanese text. Up to 16 such characters per CJK pack are waived and listed in that pack's manifest notes; more than that fails the run, because it means the wrong source font.

## Licences

IBM Plex (including Plex Sans KR, JP, SC and TC) and the Noto fonts are published under the SIL Open Font License 1.1. The application's About dialog states this. Each family's licence text must be stored with the fonts and the manifest, and the reserved font name clauses of each licence must be checked before shipping a modified subset under the original family name.

| Family | Upstream |
| --- | --- |
| IBM Plex Sans, Mono, Sans KR, Sans JP, Sans SC, Sans TC | https://github.com/IBM/plex |
| Noto Sans Arabic, Devanagari, Bengali, Telugu | https://github.com/notofonts |
