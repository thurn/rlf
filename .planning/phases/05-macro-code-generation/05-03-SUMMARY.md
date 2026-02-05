---
phase: 05-macro-code-generation
plan: 03
subsystem: proc-macro
tags: [quote, code-generation, proc-macro, phrase-functions, phrase-ids]

dependency-graph:
  requires: [05-01]
  provides: [phrase-functions, source-phrases-const, phrase-ids-module]
  affects: [05-04, rlf-tests]

tech-stack:
  added: []
  patterns: [quote-macro-codegen, source-reconstruction, screaming-case-constants]

key-files:
  created:
    - crates/rlf-macros/src/codegen.rs
  modified:
    - crates/rlf-macros/src/lib.rs

decisions:
  - id: fully-qualified-paths
    choice: "Use ::rlf::* paths in generated code"
    rationale: "Macro hygiene - avoid conflicts with user imports"
  - id: expect-for-errors
    choice: "Generated functions use expect() not Result"
    rationale: "Programming errors caught during development, not runtime"
  - id: screaming-case-ids
    choice: "PhraseId constants use SCREAMING_CASE"
    rationale: "Rust convention for constants, matches user expectations"

metrics:
  duration: 7 min
  completed: 2026-02-05
---

# Phase 05 Plan 03: Code Generation Summary

**One-liner:** Complete codegen module generating phrase functions, SOURCE_PHRASES const, and phrase_ids module with SCREAMING_CASE constants

## What Was Built

Implemented the final stage of the parse -> validate -> codegen pipeline for the `rlf!` macro. The codegen module transforms validated MacroInput AST into Rust code.

### Generated Output Structure

```rust
// From: rlf! { card = { one: "card", other: "cards" }; draw(n) = "Draw {n} {card:n}."; }

/// Returns the "card" phrase.
pub fn card(locale: &::rlf::Locale) -> ::rlf::Phrase {
    locale.get_phrase("card")
        .expect(concat!("phrase '", "card", "' should exist"))
}

/// Returns the "draw" phrase.
pub fn draw(locale: &::rlf::Locale, n: impl Into<::rlf::Value>) -> ::rlf::Phrase {
    locale.call_phrase("draw", &[n.into()])
        .expect(concat!("phrase '", "draw", "' should exist"))
}

/// Source language phrases embedded as data.
const SOURCE_PHRASES: &str = "card = { one: \"card\", other: \"cards\" };\ndraw(n) = \"Draw {n} {card:n}.\";";

/// Registers source language phrases with the locale.
pub fn register_source_phrases(locale: &mut ::rlf::Locale) {
    locale.load_translations_str("en", SOURCE_PHRASES)
        .expect("source phrases should parse successfully");
}

/// PhraseId constants for all defined phrases.
pub mod phrase_ids {
    /// ID for the "card" phrase.
    pub const CARD: ::rlf::PhraseId = ::rlf::PhraseId::from_name("card");

    /// ID for the "draw" phrase. Call with 1 argument(s) (n).
    pub const DRAW: ::rlf::PhraseId = ::rlf::PhraseId::from_name("draw");
}
```

### Key Implementation Details

1. **Phrase Function Generation**:
   - Parameterless: `fn name(&Locale) -> Phrase`
   - With parameters: `fn name(&Locale, p1: impl Into<Value>, ...) -> Phrase`
   - Doc comments describe each phrase
   - Uses `expect()` for errors (programming errors, not runtime)

2. **Source Reconstruction**:
   - Rebuilds RLF source text from macro AST
   - Handles tags, parameters, simple templates, and variants
   - Escapes special characters ({, }, \, ")
   - Reconstructs transforms and selectors

3. **PhraseId Constants**:
   - One constant per phrase in `phrase_ids` module
   - SCREAMING_CASE naming: `fire_elemental` -> `FIRE_ELEMENTAL`
   - Doc comments note parameter count for parameterized phrases
   - Uses `::rlf::PhraseId::from_name()` for const construction

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| 8ecabb5 | feat | Implement phrase function code generation |

## Deviations from Plan

None - all three tasks were implemented in a single cohesive codegen module.

## Technical Decisions

### Fully Qualified Paths

All generated code uses `::rlf::*` paths:
- `::rlf::Locale` instead of `Locale`
- `::rlf::Phrase` instead of `Phrase`
- `::rlf::Value` instead of `Value`
- `::rlf::PhraseId` instead of `PhraseId`

This ensures macro hygiene - the generated code works regardless of what the user has imported.

### Source Reconstruction

Rather than storing the original token stream, we reconstruct valid RLF source from the AST. This:
- Normalizes whitespace and formatting
- Ensures consistency between macro input and runtime parsing
- Allows the same interpreter code path for all languages

### SCREAMING_CASE for Constants

Following Rust convention, PhraseId constants use SCREAMING_CASE:
- `phrase_ids::FIRE_ELEMENTAL`
- `phrase_ids::DRAW_CARDS`

This matches user expectations for constants and provides clear visual distinction from functions.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `crates/rlf-macros/src/codegen.rs` | Created | Code generation logic |
| `crates/rlf-macros/src/lib.rs` | Modified | Wired codegen into macro pipeline |

## API Generated

### Per-Phrase Functions

```rust
// Parameterless
pub fn phrase_name(locale: &::rlf::Locale) -> ::rlf::Phrase

// With parameters
pub fn phrase_name(locale: &::rlf::Locale, p1: impl Into<::rlf::Value>, ...) -> ::rlf::Phrase
```

### Module-Level Items

```rust
// Embedded source phrases
const SOURCE_PHRASES: &str = "...";

// Registration function
pub fn register_source_phrases(locale: &mut ::rlf::Locale)

// PhraseId constants
pub mod phrase_ids {
    pub const PHRASE_NAME: ::rlf::PhraseId = ...;
}
```

## Next Phase Readiness

Plan 04 (Integration Testing) requires:
- [x] Phrase functions generated and callable
- [x] SOURCE_PHRASES const available
- [x] register_source_phrases() function works
- [x] phrase_ids module with constants

Ready for integration testing phase.

## Verification

```
$ cargo build -p rlf-macros
   Compiling rlf-macros v0.1.0
    Finished

$ just review
Format OK
No inline tests
Check passed
Clippy passed
Tests passed
```
