---
phase: 07-romance-language-transforms
plan: 01
subsystem: interpreter-transforms
tags: [spanish, portuguese, romance-languages, articles, contractions]

dependency-graph:
  requires: [06-english-and-germanic-transforms]
  provides: [spanish-transforms, portuguese-transforms, romance-helpers]
  affects: [07-02-french-italian-transforms]

tech-stack:
  added: []
  patterns: [romance-gender-helpers, plural-context-resolution]

key-files:
  created: []
  modified:
    - crates/rlf/src/interpreter/transforms.rs
    - crates/rlf/tests/interpreter_transforms.rs

decisions:
  - key: romance-gender-reuse
    choice: Shared RomanceGender and RomancePlural types across Spanish/Portuguese
    rationale: Both languages use same masculine/feminine + singular/plural pattern
  - key: portuguese-indefinite-no-plural
    choice: Portuguese @um ignores plural context
    rationale: Portuguese indefinite article (um/uma) doesn't have plural forms

metrics:
  duration: 5min
  completed: 2026-02-05
---

# Phase 07 Plan 01: Spanish and Portuguese Article Transforms Summary

Spanish and Portuguese article transforms with gender tags and plural context, plus Portuguese preposition contractions.

## What Was Built

### Spanish Transforms

1. **@el/@la - Definite Article**
   - Reads :masc/:fem tags for gender
   - Uses context for plural: el/los (masc), la/las (fem)
   - @la alias resolves to @el

2. **@un/@una - Indefinite Article**
   - Reads :masc/:fem tags for gender
   - Uses context for plural: un/unos (masc), una/unas (fem)
   - @una alias resolves to @un

### Portuguese Transforms

1. **@o/@a - Definite Article**
   - Reads :masc/:fem tags for gender
   - Uses context for plural: o/os (masc), a/as (fem)
   - @a alias resolves to @o (in Portuguese context)

2. **@um/@uma - Indefinite Article**
   - Reads :masc/:fem tags for gender
   - No plural context (Portuguese indefinite doesn't pluralize)
   - @uma alias resolves to @um

3. **@de - Preposition Contraction**
   - Contracts de + article: do/da/dos/das
   - Uses context for plural selection

4. **@em - Preposition Contraction**
   - Contracts em + article: no/na/nos/nas
   - Uses context for plural selection

### Shared Infrastructure

- `RomanceGender` enum (Masculine, Feminine)
- `RomancePlural` enum (One, Other)
- `parse_romance_gender()` - extracts gender from :masc/:fem tags
- `parse_romance_plural()` - parses context for plural (string or numeric)

## Commits

| Hash | Type | Description |
|------|------|-------------|
| e07ad00 | feat | Add Spanish article transforms |
| 272ee04 | feat | Add Portuguese article and contraction transforms |
| d9b11f3 | test | Add Spanish and Portuguese integration tests |

## Test Coverage

- 10 Spanish unit tests (gender, plural, aliases, errors)
- 12 Portuguese unit tests (articles, contractions, aliases, errors)
- 6 integration tests (template evaluation, cross-language)

Total: 28 new tests (110 total transform tests)

## Deviations from Plan

### Test Adjustment
**Task 3:** Modified `spanish_el_with_plural_context` test
- **Original plan:** Used phrase parameter with variant lookup
- **Actual:** Used literal context selector `:other` on transform
- **Reason:** Selector on transform affects article selection, not variant lookup
- **Result:** Same behavior verified, clearer test intent

## Key Implementation Details

### Alias Resolution
```rust
// Language-specific alias resolution
("la", "es") => "el",  // Spanish: @la -> @el
("a", "pt") => "o",    // Portuguese: @a -> @o
("una", _) => "un",    // Spanish: @una -> @un
("uma", _) => "um",    // Portuguese: @uma -> @um
```

### Plural Context Handling
```rust
fn parse_romance_plural(context: Option<&Value>) -> RomancePlural {
    match context {
        Some(Value::String(s)) if s == "other" => RomancePlural::Other,
        Some(Value::Number(n)) if *n != 1 => RomancePlural::Other,
        _ => RomancePlural::One,  // Default to singular
    }
}
```

## Usage Examples

### Spanish
```
carta = :fem "carta";
enemigo = :masc "enemigo";

// Definite articles
"Roba {@el carta}."      // -> "Roba la carta."
"Los {@el:other enemigo}"// -> "Los los enemigo"

// Indefinite articles
"Roba {@un carta}."      // -> "Roba una carta."
```

### Portuguese
```
vazio = :masc "vazio";
mao = :fem "mao";

// Definite articles
"{@o vazio}"             // -> "o vazio"
"{@a mao}"               // -> "a mao" (alias works)

// Contractions
"Saiu {@de vazio}."      // -> "Saiu do vazio." (de+o=do)
"Estava {@em mao}."      // -> "Estava na mao." (em+a=na)
```

## Next Phase Readiness

Phase 07 Plan 02 (French and Italian) can proceed:
- RomanceGender/RomancePlural types ready for reuse
- parse_romance_* functions available
- Pattern established for elision handling (French)

## Files Modified

- `crates/rlf/src/interpreter/transforms.rs`: +170 lines (6 transforms, 4 lookup tables, 2 helper types)
- `crates/rlf/tests/interpreter_transforms.rs`: +171 lines (28 tests)
