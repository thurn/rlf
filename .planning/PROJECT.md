# RLF - Rust Localization Framework

## What This Is

A localization DSL embedded in Rust via proc macros. Source language phrases (typically English) are defined using the `rlf!` macro, which provides compile-time validation and immediate IDE autocomplete. Translations are loaded at runtime via an interpreter. The framework handles complex grammatical patterns across 20+ languages including gender agreement, case declension, measure words, and plural forms.

## Core Value

When you add a phrase to `strings.rlf.rs`, it immediately appears in IDE autocomplete. No build steps, no external tools, no waiting.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] `rlf!` macro parses phrase definitions and generates typed Rust functions
- [ ] Interpreter evaluates templates for source and translated languages
- [ ] Four primitives: phrase, parameter, variant, selection
- [ ] Metadata tags for grammatical information (`:masc`, `:fem`, `:a`, `:an`, etc.)
- [ ] Transforms for language-specific operations (`@cap`, `@a`, `@el`, `@der`, `@count`, etc.)
- [ ] Metadata inheritance via `:from(param)` for phrase-returning phrases
- [ ] `PhraseId` type for serializable phrase references (8-byte hash)
- [ ] Multi-dimensional variants with fallback resolution (e.g., `nom.many` → `nom`)
- [ ] CLDR plural rules via ICU4X for all supported languages
- [ ] Compile-time validation for source language (unknown phrases, parameters, cycles)
- [ ] Runtime validation for translations (load-time syntax check)
- [ ] Standard transforms for 20+ languages (see APPENDIX_STDLIB.md)
- [ ] Hot-reloading for translation files during development
- [ ] CLI tool: `rlf check` - validate .rlf file syntax
- [ ] CLI tool: `rlf eval` - evaluate templates interactively
- [ ] CLI tool: `rlf coverage` - show translation coverage vs source

### Out of Scope

- GUI translation editor — use existing tools like Poedit or Localazy
- Automatic translation via LLM — humans translate, RLF manages
- Build-time code generation from external files — macro-only approach
- Language fallback — missing translations are errors, not silent fallbacks
- Custom user transforms — transforms are built-in per language

## Context

**Primary motivation:** Replacing Fluent-based localization in the Dreamtides card game. Dreamtides has ~150 UI strings and complex data-driven card text with grammatical patterns (plurals, gender agreement for Spanish, case declension for Russian, measure words for Chinese/Japanese).

**Existing design work:** Comprehensive design documents exist in `docs/`:
- `DESIGN.md` - Core language design with four primitives
- `APPENDIX_RUST_INTEGRATION.md` - Macro implementation, code generation, PhraseId
- `APPENDIX_RUNTIME_INTERPRETER.md` - Interpreter architecture, evaluation
- `APPENDIX_STDLIB.md` - Standard transforms for 20+ languages
- `APPENDIX_SPANISH_TRANSLATION.md` - Spanish walkthrough (cost_serializer)
- `APPENDIX_RUSSIAN_TRANSLATION.md` - Russian walkthrough (predicate_serializer)
- `APPENDIX_DREAMTIDES_ADOPTION.md` - Migration guide from Fluent

**Design principle:** "Pass Phrase, not String" — Rust passes `Phrase` values with grammatical metadata, RLF handles selection and agreement. Rust identifies *what* to say, RLF decides *how* to say it.

**Reference project for patterns:** Game code at `~/Documents/GoogleDrive/dreamtides/rules_engine/src` if implementation details are needed.

## Constraints

- **Crate structure**: Workspace with `rlf` (library), `rlf-macros` (proc-macro), `rlf-cli` (binary)
- **Rust version**: Latest stable (edition 2024)
- **Testing**: Integration tests in `tests/` directory only (no `#[test]` in src/), insta for snapshots, proptest for fuzzing
- **Build**: `justfile` as central command runner
- **Dependencies**: ICU4X for CLDR plural rules, proc-macro2/quote/syn for macro
- **Validation**: Every task MUST include `just review` in its `<verify>` tag to ensure code quality

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Unified interpreter for all languages | Simplicity over micro-optimization; one code path to maintain | — Pending |
| No language fallback | Missing translations should be caught in CI, not silently papered over | — Pending |
| Hash-based PhraseId | Stable across phrase additions, 8 bytes, Copy, const-constructible | — Pending |
| Built-in transforms only | Predictable behavior, no user-defined complexity | — Pending |

---
*Last updated: 2026-02-04 - added validation constraint*
