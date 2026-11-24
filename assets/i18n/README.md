# Interface catalogs

Each file in this directory holds the interface text for one locale. `build.rs` compiles every catalog into static tables indexed by the generated `Text` enum, and a unit test checks every locale against English. Location names and other game text are not here; they come from the game's own localization through the language packs.

## File naming

A catalog is named after the site's locale tag with a `.txt` extension: `en-US.txt`, `ko-KR.txt`, `zh-Hans.txt`, `ar.txt`, `sh.txt`. The tag is written exactly as the site writes it, including case. The supported tags are `en-US`, `ko-KR`, `fr-FR`, `es-ES`, `zh-Hans`, `zh-Hant`, `ja-JP`, `de-DE`, `ar`, `hi-IN`, `bn-BD`, `te-IN`, `pt-BR`, `id-ID`, `ru-RU`, `ur-PK`, `vi-VN`, `tl-PH`, `fa-IR`, `it-IT`, `pl-PL`, `sv-SE`, `sh`, `ro-RO`, `nl-NL`, `uk-UA`, and `tr-TR`. A `.txt` file with any other name is a build error; `README.md` is ignored.

## Encoding and lines

Files are UTF-8 without a byte order mark and use LF line endings. The file is read line by line:

- A line that is empty or contains only spaces and tabs is ignored.
- A line whose first non-blank character is `#` is a comment and is ignored. A `#` anywhere else is ordinary text, so `rgb.placeholder = #RRGGBB` is a normal entry.
- Every other line is an entry of the form `key = value`. The line is split at the first `=`; later `=` characters belong to the value. Spaces and tabs around the key and around the value are removed.
- An entry cannot span lines. A line without `=`, an empty key, or an empty value is a build error.

Group entries under comment headings in the same order as `en-US.txt`, so files can be compared side by side.

## Keys

A key is lower-case dotted snake case: segments of `a-z`, `0-9` and `_`, each starting with a letter, joined by `.`, for example `header.search.placeholder`. Keys are identifiers, never translated. A key may appear only once per file, counting each plural form separately.

The generated enum variant is the key split at `.` and `_`, each part capitalized and joined: `header.search.placeholder` becomes `Text::HeaderSearchPlaceholder` and `column.raw_material` becomes `Text::ColumnRawMaterial`. Two keys that produce the same variant are a build error, so new keys must be checked against that rule.

## Escapes

Values are literal text except for these sequences:

| Sequence | Meaning |
| --- | --- |
| `\n` | line break |
| `\s` | one space, needed only at the start or end of a value, where plain spaces are trimmed |
| `\\` | one backslash |
| `{{` | literal `{` |
| `}}` | literal `}` |

Any other backslash sequence, and any `{` or `}` that is not part of a placeholder or one of the doubled forms, is a build error. Do not add quotation marks around values; they would become part of the text.

## Placeholders

`{name}` marks a value inserted at run time. Names use `a-z`, `0-9` and `_` and start with a letter. A translation may reorder placeholders and must keep their names unchanged. For an entry without plural forms, the set of placeholder names must equal the English set for that key, each name appearing exactly once. Numbers, identifiers, paths and colour codes are inserted already formatted; do not add formatting inside the braces.

## Plural forms

A counted string is written as several entries that share a base key and carry a CLDR plural category in square brackets directly after the key:

```text
header.result_count[one] = {count} location
header.result_count[other] = {count} locations
```

The categories are `zero`, `one`, `two`, `few`, `many` and `other`. Each locale supplies exactly the cardinal categories its language uses, as listed below, and `other` is always required. The `other` form must contain every placeholder of the English `other` form; another form may leave out `{count}` where the language says "one location" without the digit, but may not add placeholders. A key either has plural forms in every locale or in none.

| Categories | Locales |
| --- | --- |
| `other` | `ko-KR`, `ja-JP`, `zh-Hans`, `zh-Hant`, `vi-VN`, `id-ID` |
| `one`, `other` | `en-US`, `de-DE`, `nl-NL`, `sv-SE`, `tr-TR`, `hi-IN`, `bn-BD`, `te-IN`, `ur-PK`, `fa-IR`, `tl-PH` |
| `one`, `many`, `other` | `fr-FR`, `es-ES`, `it-IT`, `pt-BR` |
| `one`, `few`, `other` | `ro-RO`, `sh` |
| `one`, `few`, `many`, `other` | `ru-RU`, `uk-UA`, `pl-PL` |
| `zero`, `one`, `two`, `few`, `many`, `other` | `ar` |

The plural selector in `src/app/i18n.rs` implements the CLDR cardinal rules for these groups for whole numbers, and this table must change together with it.

## Key set parity

Every locale file must contain exactly the English key set: the same base keys, no missing key and no extra key. Plural keys are compared by base key; their categories follow the table above rather than the English categories. The parity test also checks placeholder names for every entry and every plural form.

## Writing rules

- Write the text as it should appear, including capitalization. The interface never changes case, because case mapping is language-specific (Turkish dotted and dotless i, for example).
- The interface never builds a sentence from fragments. If a string needs a value, it has a placeholder; if a control needs a different sentence, it has its own key.
- Directional marks such as arrows, chevrons and the breadcrumb separator are drawn by the interface according to text direction and are not part of any value.
- Controls have fixed sizes and clip long labels, showing the full text as a tooltip, so a longer translation is acceptable but a short one reads better.
- Product and proper names (`EU5 Location DB`, `Europa Universalis V`, `Paradox Interactive`, `iced`, `IBM Plex`, `Noto Sans`, `SIL Open Font License`, `MIT`, `Apache-2.0`, `GitHub`, `LinkedIn`, `cyhdev.com`) stay unchanged.
- `en-US.txt` and `ko-KR.txt` are written directly. The other catalogs are drafts that need review by a native reader before they are presented as finished.
