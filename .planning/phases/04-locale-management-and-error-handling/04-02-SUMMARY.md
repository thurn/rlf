---
phase: 04-locale-management-and-error-handling
plan: 02
subsystem: interpreter
tags: [locale, builder-pattern, multi-language, hot-reload, fallback]
requires:
  - phase-03 (transforms)
  - plan-04-01 (error types)
provides:
  - Locale struct for user-facing localization API
  - Per-language phrase storage
  - Hot-reload support
  - Fallback language support
affects:
  - phase-05 (may extend Locale API)
tech-stack:
  added: [tempfile]
  patterns: [builder-pattern, per-language-registry]
key-files:
  created:
    - crates/rlf/src/interpreter/locale.rs
    - crates/rlf/tests/locale.rs
  modified:
    - crates/rlf/src/interpreter/mod.rs
    - crates/rlf/src/lib.rs
    - crates/rlf/Cargo.toml
decisions:
  - Locale owns TransformRegistry (not borrowed)
  - Per-language registries use HashMap<String, PhraseRegistry>
  - Loading same language replaces all phrases (not merge)
  - Fallback only tried on PhraseNotFound errors
metrics:
  duration: 7 min
  completed: 2026-02-05
---

# Phase 04 Plan 02: Locale API Implementation Summary

**One-liner:** User-facing Locale struct with builder pattern, per-language phrase storage, hot-reload, and fallback support.

## What Was Built

### Locale Struct (`crates/rlf/src/interpreter/locale.rs`)

The central user-facing API for RLF localization management:

```rust
#[derive(Builder)]
pub struct Locale {
    language: String,                              // Current language code
    fallback_language: Option<String>,             // Optional fallback
    registries: HashMap<String, PhraseRegistry>,   // Per-language storage
    transforms: TransformRegistry,                 // Owned transforms
    loaded_paths: HashMap<String, PathBuf>,        // For hot-reload
}
```

**Key Methods:**
- `Locale::new()` / `Locale::builder()` - Creation with defaults (English)
- `Locale::with_language(lang)` - Shorthand constructor
- `language()` / `set_language(lang)` - Language management
- `load_translations(lang, path)` - Load from file
- `load_translations_str(lang, content)` - Load from string
- `reload_translations(lang)` - Hot-reload from original file
- `get_phrase(name)` - Get parameterless phrase (with fallback)
- `call_phrase(name, args)` - Call phrase with arguments (with fallback)
- `eval_str(template, params)` - Evaluate template string

### Design Decisions

1. **Per-Language Registries:** Each language has its own `PhraseRegistry` in a HashMap. This provides:
   - Clean "replace" semantics when reloading
   - Language-scoped phrase lookup
   - Independent phrase storage

2. **Owned TransformRegistry:** Locale owns its TransformRegistry rather than borrowing. Shared across all languages.

3. **Replace Semantics:** Loading translations for the same language replaces all previous phrases (not merge). This simplifies hot-reload logic.

4. **Fallback Chain:** When a phrase is not found:
   - First tries current language
   - If fallback configured and different from current, tries fallback
   - Returns original error if both fail

5. **Hot-Reload Support:** File paths are stored when loading from files, enabling `reload_translations()` to re-read from disk. String-loaded translations cannot be reloaded (returns `NoPathForReload` error).

### Test Coverage

27 new locale integration tests covering:
- Builder pattern (4 tests)
- Translation loading from string (3 tests)
- Per-language storage (2 tests)
- File loading (2 tests)
- Hot-reload (3 tests)
- Phrase evaluation (4 tests)
- Fallback language (4 tests)
- Registry access (3 tests)
- Transform access (1 test)

Total project tests: 192 passing

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 1ee6ca9 | feat | Locale struct with per-language registries |
| ebcea9e | test | Comprehensive Locale integration tests |

## Files Changed

**Created:**
- `crates/rlf/src/interpreter/locale.rs` (446 lines) - Locale implementation
- `crates/rlf/tests/locale.rs` (345 lines) - Integration tests

**Modified:**
- `crates/rlf/src/interpreter/mod.rs` - Added locale module export
- `crates/rlf/src/lib.rs` - Export Locale from crate root
- `crates/rlf/Cargo.toml` - Added tempfile dev-dependency
- `Cargo.lock` - Updated dependencies

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

All verification criteria met:
- [x] `cargo build` compiles without errors
- [x] `just review` passes all checks
- [x] `cargo test` runs all tests including new Locale tests (192 passing)
- [x] Locale owns `HashMap<String, PhraseRegistry>` (per-language storage)
- [x] Locale owns `TransformRegistry` (not creating new instances)
- [x] load_translations stores phrases in language-specific registry
- [x] Loading same language twice replaces previous phrases
- [x] Fallback language is used when configured and primary missing

## Phase 4 Status

Phase 4 is now **COMPLETE**:
- Plan 01: Error types and suggestions (COMPLETE)
- Plan 02: Locale API implementation (COMPLETE)

**Ready for Phase 5.**
