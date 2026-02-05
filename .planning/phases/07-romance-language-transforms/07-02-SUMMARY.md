---
phase: 07-romance-language-transforms
plan: 02
subsystem: interpreter-transforms
tags: [french, italian, romance-languages, articles, contractions, elision]

dependency-graph:
  requires: [07-01-spanish-portuguese-transforms]
  provides: [french-transforms, italian-transforms, liaison-transform]
  affects: []

tech-stack:
  added: []
  patterns: [italian-sound-categories, french-elision, liaison-variant-selection]

key-files:
  created: []
  modified:
    - crates/rlf/src/interpreter/transforms.rs
    - crates/rlf/tests/interpreter_transforms.rs

decisions:
  - key: french-un-no-plural
    choice: French @un has no plural forms
    rationale: Per APPENDIX_STDLIB, only singular indefinite (un/une)
  - key: italian-sound-categories
    choice: ItalianSound enum (Normal, Vowel, SImpura) for three-way distinction
    rationale: Italian has distinct article forms for consonant, vowel, and s+consonant
  - key: liaison-output
    choice: "@liaison outputs only selected variant, not context"
    rationale: Per APPENDIX_STDLIB example, context determines selection but appears separately
  - key: alias-language-scope
    choice: Spanish @una alias made language-specific ("es")
    rationale: Prevents shadowing Italian @una alias in match statement

metrics:
  duration: 8min
  completed: 2026-02-05
---

# Phase 07 Plan 02: French and Italian Article Transforms Summary

French and Italian article transforms with elision, sound rules, and contractions - all producing lowercase output.

## What Was Built

### French Transforms

1. **@le/@la - Definite Article**
   - Reads :masc/:fem tags for gender
   - Reads :vowel tag for elision (l' before vowels, singular only)
   - Uses context for plural: le/l'/la (singular), les (plural)
   - @la alias resolves to @le

2. **@un/@une - Indefinite Article**
   - Reads :masc/:fem tags for gender
   - No plural forms (per APPENDIX_STDLIB)
   - @une alias resolves to @un

3. **@de - Preposition Contraction**
   - Contracts de + article: du/de la/de l'/des
   - Handles elision: de l' before vowels
   - Uses context for plural selection

4. **@au - Preposition Contraction**
   - Contracts a + article: au/a la/a l'/aux
   - Handles elision: a l' before vowels
   - Uses context for plural selection

5. **@liaison - Prevocalic Form Selection**
   - Selects between `standard` and `vowel` variants
   - Based on context's :vowel tag
   - Used for beau/bel, ce/cet, nouveau/nouvel, vieux/vieil

### Italian Transforms

1. **@il/@lo/@la - Definite Article**
   - Reads :masc/:fem tags for gender
   - Reads :vowel tag for elision (l' before vowels)
   - Reads :s_imp tag for s-impura forms (lo/gli)
   - Full form table: il/l'/lo/la (singular), i/gli/le (plural)
   - @lo/@la aliases resolve to @il

2. **@un/@uno/@una - Indefinite Article**
   - Reads gender and sound tags
   - Forms: un/un/uno/una (singular), dei/degli/delle (plural)
   - @uno/@una aliases resolve to @un

3. **@di - Preposition Contraction**
   - Contracts di + article: del/dell'/dello/della/dei/degli/delle
   - Full sound category support

4. **@a - Preposition Contraction**
   - Contracts a + article: al/all'/allo/alla/ai/agli/alle
   - Full sound category support

### Shared Infrastructure

- `ItalianSound` enum (Normal, Vowel, SImpura)
- `parse_italian_sound()` - extracts sound category from tags
- Reuses `RomanceGender` and `RomancePlural` from Plan 01

## Commits

| Hash | Type | Description |
|------|------|-------------|
| fc30155 | feat | Add French transforms with elision |
| 2bc6069 | feat | Add Italian transforms with sound rules |
| 49d00d5 | feat | Add French and Italian integration tests |

## Test Coverage

- 21 French unit tests (articles, elision, contractions, aliases)
- 24 Italian unit tests (articles, sound variants, contractions, aliases)
- 5 French lowercase validation tests
- 5 Italian lowercase validation tests
- 16 French integration tests
- 14 Italian integration tests
- 2 cross-language tests

Total: 59 new tests (169 total transform tests)

## Deviations from Plan

### Fix: Liaison Transform Output

**Task 3:** Fixed @liaison transform behavior
- **Original implementation:** Output "variant context" (e.g., "bel ami")
- **Correct behavior:** Output only "variant" (e.g., "bel")
- **Reason:** Per APPENDIX_STDLIB, context is used only for selection, appears separately in template
- **Impact:** None - fixed during integration test development

### Fix: Spanish Alias Shadowing

**Task 3:** Fixed Spanish @una alias shadowing Italian
- **Issue:** `("una", _) => "un"` matched all languages, including Italian
- **Fix:** Changed to `("una", "es") => "un"` for language-specific alias
- **Impact:** Italian @una alias now works correctly

## Key Implementation Details

### French Elision

```rust
fn french_definite_article(gender: RomanceGender, has_vowel: bool, plural: RomancePlural) -> &'static str {
    match (gender, has_vowel, plural) {
        // Elision: l' before vowel (singular only)
        (_, true, RomancePlural::One) => "l'",
        // Masculine singular: le
        (RomanceGender::Masculine, false, RomancePlural::One) => "le",
        // Feminine singular: la
        (RomanceGender::Feminine, false, RomancePlural::One) => "la",
        // Plural (same for both genders, no elision)
        (_, _, RomancePlural::Other) => "les",
    }
}
```

### Italian Sound Categories

```rust
enum ItalianSound {
    Normal,   // Standard consonant: il/un
    Vowel,    // Vowel-initial: l'/un
    SImpura,  // s+consonant, z, gn, ps, x, y: lo/uno
}

fn parse_italian_sound(value: &Value) -> ItalianSound {
    if value.has_tag("s_imp") { ItalianSound::SImpura }
    else if value.has_tag("vowel") { ItalianSound::Vowel }
    else { ItalianSound::Normal }
}
```

### Apostrophe Handling

All elided forms (l', dell', all', etc.) attach directly without space:
```rust
if article.ends_with('\'') {
    Ok(format!("{}{}", article, text))  // l'ami
} else {
    Ok(format!("{} {}", article, text)) // le livre
}
```

## Usage Examples

### French

```
livre = :masc "livre";
ami = :masc :vowel "ami";
maison = :fem "maison";

// Definite articles
"{@le livre}"      // -> "le livre"
"{@le ami}"        // -> "l'ami" (elision)
"{@la maison}"     // -> "la maison"

// Contractions
"{@de livre}"      // -> "du livre" (de+le=du)
"{@de ami}"        // -> "de l'ami" (de+l'=de l')
"{@au livre}"      // -> "au livre" (a+le=au)

// Liaison
beau = { standard: "beau", vowel: "bel" };
"{@liaison beau ami}"  // -> "bel" (before vowel)
"{@liaison beau livre}" // -> "beau" (before consonant)
```

### Italian

```
libro = :masc "libro";
amico = :masc :vowel "amico";
studente = :masc :s_imp "studente";
casa = :fem "casa";

// Definite articles
"{@il libro}"      // -> "il libro"
"{@il amico}"      // -> "l'amico" (elision)
"{@il studente}"   // -> "lo studente" (s-impura)
"{@la casa}"       // -> "la casa"

// Contractions
"{@di libro}"      // -> "del libro"
"{@di amico}"      // -> "dell'amico"
"{@di studente}"   // -> "dello studente"
"{@a libro}"       // -> "al libro"
```

## Next Phase Readiness

Phase 07 (Romance Language Transforms) is now complete:
- Spanish/Portuguese transforms from Plan 01
- French/Italian transforms from Plan 02
- All contraction transforms produce lowercase output
- 169 total transform tests passing

Ready for Phase 08 or further milestone work.

## Files Modified

- `crates/rlf/src/interpreter/transforms.rs`: +504 lines (9 transforms, 8 lookup tables, ItalianSound type)
- `crates/rlf/tests/interpreter_transforms.rs`: +505 lines (59 new tests)
