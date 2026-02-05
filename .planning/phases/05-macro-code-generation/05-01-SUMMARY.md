---
phase: 05-macro-code-generation
plan: 01
subsystem: proc-macro
tags: [syn, quote, proc-macro, parsing, tokenstream]

dependency-graph:
  requires: []
  provides: [rlf-macros-crate, macro-ast-types, tokenstream-parsing]
  affects: [05-02, 05-03, 05-04]

tech-stack:
  added: [syn-2.0, quote-1.0, proc-macro2-1.0, trybuild-1.0]
  patterns: [pipeline-architecture, spanned-identifiers, parse-trait-impl]

key-files:
  created:
    - crates/rlf-macros/Cargo.toml
    - crates/rlf-macros/src/lib.rs
    - crates/rlf-macros/src/input.rs
    - crates/rlf-macros/src/parse.rs

decisions:
  - id: macro-ast-separate-from-parser-ast
    choice: "Macro AST types (input.rs) are separate from runtime parser AST"
    rationale: "Macro AST needs proc_macro2::Span for compile errors; parser AST doesn't"
  - id: template-string-manual-parsing
    choice: "Parse template strings manually from LitStr content"
    rationale: "Interpolations are inside string literals, not token trees"
  - id: dead-code-allow
    choice: "Allow dead_code at module level in input.rs"
    rationale: "Fields are parsed but not consumed until Plan 02/03"

metrics:
  duration: 4 min
  completed: 2026-02-05
---

# Phase 05 Plan 01: TokenStream Parsing Foundation Summary

**One-liner:** rlf-macros proc-macro crate with syn-based TokenStream parsing into spanned AST types

## What Was Built

Created the `rlf-macros` crate as the foundation for the `rlf!` proc-macro. This implements the first stage of the macro pipeline: parsing TokenStream input into a structured AST with span preservation for error messages.

### Architecture

```
TokenStream (from rlf! { ... })
    |
    v
[MacroInput] --> Vec<PhraseDefinition>
    |
    +-> name: SpannedIdent (with Span for error messages)
    +-> parameters: Vec<SpannedIdent>
    +-> tags: Vec<SpannedIdent>
    +-> from_param: Option<SpannedIdent>
    +-> body: PhraseBody
            |
            +-> Simple(Template) or Variants(Vec<VariantEntry>)
                    |
                    +-> segments: Vec<Segment>
                            |
                            +-> Literal(String)
                            +-> Interpolation { transforms, reference, selectors }
```

### Key Implementation Details

1. **SpannedIdent wrapper**: All user-provided identifiers are wrapped in `SpannedIdent` which carries both the name string and the source `Span`. This enables precise error pointing in future validation.

2. **Template string parsing**: Since interpolations (`{...}`) appear inside string literals, we parse the `LitStr` value and manually extract segments. The span for the entire template is preserved.

3. **Supported syntax**:
   - Simple phrases: `name = "template";`
   - Phrases with parameters: `name(p1, p2) = "...";`
   - Tags: `:tag1 :tag2 name = "...";`
   - Inheritance: `:from(param) name = "...";`
   - Variants: `name = { one: "x", other: "y" };`
   - Multi-key variants: `nom, acc: "shared"`
   - Dotted keys: `nom.one, nom.few: "values"`
   - Interpolations: `{ref}`, `{ref:selector}`, `{@transform ref}`, `{foo(x, y)}`
   - Escaped braces: `{{` and `}}`

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| bcec203 | feat | Create rlf-macros proc-macro crate |
| 94ce7b2 | feat | Define internal AST types with span preservation |
| fac832f | feat | Implement syn Parse traits for macro AST |

## Deviations from Plan

None - plan executed exactly as written.

## Technical Decisions

### Macro AST Separate from Parser AST

The existing `crates/rlf/src/parser/ast.rs` defines types like `Template`, `Segment`, etc. for runtime parsing. The macro crate defines its own parallel types with `proc_macro2::Span` fields.

Why separate:
- Runtime AST has no span information (not needed after parsing)
- Macro AST must preserve spans for compile error messages
- Keeps proc-macro dependencies isolated to the macro crate

### Template String Manual Parsing

Interpolations appear inside string literals: `"Hello {name}"`. These are not separate tokens - they're embedded in a `LitStr`. We parse the string value character by character.

Handles:
- Escaped braces: `{{` -> `{`
- Nested depth tracking (for `{foo(x, y)}`)
- Error on unclosed braces or empty interpolations

## Files Created

| File | Purpose |
|------|---------|
| `crates/rlf-macros/Cargo.toml` | Proc-macro crate configuration with syn/quote deps |
| `crates/rlf-macros/src/lib.rs` | Macro entry point with `#[proc_macro] pub fn rlf` |
| `crates/rlf-macros/src/input.rs` | AST types: MacroInput, PhraseDefinition, Template, etc. |
| `crates/rlf-macros/src/parse.rs` | Parse implementations for all AST types |

## Dependencies Added

| Crate | Version | Purpose |
|-------|---------|---------|
| syn | 2.0 | TokenStream parsing with full/parsing/printing features |
| quote | 1.0 | Code generation (used in Plan 03) |
| proc-macro2 | 1.0 | TokenStream wrapper for testing |
| strsim | 0.11 | Levenshtein distance for typo suggestions (Plan 02) |
| trybuild | 1.0 (dev) | Compile-fail testing |

## Next Phase Readiness

Plan 02 (Validation) requires:
- [x] MacroInput type to traverse
- [x] SpannedIdent for error spans
- [x] Reference enum for dependency graph
- [x] Selector for variant validation

Ready for validation phase.

## Verification

```
$ cargo build -p rlf-macros
   Compiling rlf-macros v0.1.0
    Finished

$ cargo build
   Compiling rlf-macros v0.1.0
    Finished

$ just review
Format OK
No inline tests
Check passed
Clippy passed
Tests passed
```
