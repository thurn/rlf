# Technology Stack

**Analysis Date:** 2026-02-04

## Languages

**Primary:**
- Rust - Core framework and procedural macro system for RLF implementation

**Secondary:**
- RLF (domain-specific language) - Embedded DSL for localization definitions, compiles via Rust macros

## Runtime

**Environment:**
- Rust standard library (no_std compatible with appropriate features)

**Package Manager:**
- Cargo - Rust package manager
- Dependencies defined in `Cargo.toml`

## Frameworks

**Core:**
- Procedural Macro (`proc-macro`) - Powers the `rlf!` macro for compile-time phrase validation and code generation
- ICU4X - Unicode International Components for Unicode-compliant pluralization and locale handling

**Testing:**
- No external test framework specified (testing patterns documented but implementation framework not specified in docs)

**Build/Dev:**
- Cargo build system
- rustfmt/rust-analyzer for IDE integration

## Key Dependencies

**Critical:**
- `icu_plurals` v2 - CLDR plural rule evaluation for all supported languages (zero, one, two, few, many, other categories)
- `icu_locale_core` v2 - Locale parsing and locale-specific plural rules

**Infrastructure:**
- No external database or API client dependencies documented

## Core Components

**Macro System:**
- `rlf!` procedural macro - Parses `.rlf.rs` source files, validates references, generates Rust functions
- Located conceptually in: procedural macro crate (not visible in docs repo)

**Runtime Interpreter:**
- `RlfInterpreter` - Evaluates templates and translations at runtime
  - Supports multi-language phrase lookup
  - Executes transforms (universal: `@cap`, `@upper`, `@lower`; language-specific)
  - Applies plural category selection via ICU plural rules
  - Handles metadata tag-based variant selection

**Locale Management:**
- `Locale` struct - Manages language selection and interpreter state
  - Methods: `set_language()`, `load_translations()`, `interpreter()`, `interpreter_mut()`
  - Carries current language context through evaluation

**Type System:**
- `Phrase` - Result type from phrase functions, carries text, variants, and metadata tags
- `PhraseId` - 8-byte FNV-1a hash-based serializable phrase reference (Copy, Eq, Hash, Serialize)
- `Value` enum - Runtime parameter type supporting Number (i64), Float (f64), String, Phrase

## File Organization

**Source Language Files:**
- `.rlf.rs` extension - Source language (typically English) compiled via `rlf!` macro
- Generated as: embedded in compiled binary and accessible via `register_source_phrases()` function

**Translation Files:**
- `.rlf` extension - Translation files loaded at runtime via interpreter
- Syntax identical to source language but loaded dynamically
- No build-time compilation required

## Configuration

**Environment:**
- Translation files loaded via `Locale::load_translations(language, file_path)` API
- No environment variables required for core functionality (implementation-specific)

**Build:**
- `Cargo.toml` - Defines dependencies (icu_plurals, icu_locale_core at minimum)
- No special build scripts documented

## Platform Requirements

**Development:**
- Rust toolchain (edition 2018 or later based on use of proc-macros)
- Cargo
- IDE with rust-analyzer support (for proc-macro IDE features)

**Production:**
- Target: Any platform supported by Rust std library
- No external runtime dependencies beyond ICU data (included with icu_plurals/icu_locale_core crates)

## Language Support

**Built-in Plural Categories (via CLDR):**
- English: one, other
- Russian: one, few, many, other
- Spanish: one, other
- Mandarin Chinese (zh_cn): other (no plural)
- German: one, other
- French: one, other
- Hindi: one, other
- And additional languages via ICU4X

**Supported Metadata Transforms:**
- Universal: `@cap`, `@upper`, `@lower`
- English: `@a`, `@an`, `@the` (reads `:a`/`:an` tags)
- German: `@der`, `@die` (reads `:masc`, `:fem`, `:neut` with case support)
- Spanish: `@el`, `@la` (reads `:masc`, `:fem`)
- French: `@le`, `@la` (reads `:masc`, `:fem`, `:vowel`)
- Romance languages: `@un`, `@una` (indefinite articles)
- Mandarin Chinese: `@count` (measure word insertion via tags `:zhang`, `:ge`, `:ming`, etc.)

---

*Stack analysis: 2026-02-04*
