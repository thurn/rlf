# Phase 5: Macro Code Generation - Context

**Gathered:** 2026-02-05
**Status:** Ready for planning

<domain>
## Phase Boundary

The `rlf!` proc-macro parses phrase definitions and generates typed Rust functions with compile-time validation. Users write macro invocations and see generated functions in their IDE immediately. This phase covers parsing, validation, and code generation — not runtime evaluation (already done in Phases 2-4).

</domain>

<decisions>
## Implementation Decisions

### Generated Function API
- Functions panic on error (programming errors caught during development, not runtime handling)
- Separate arguments for each phrase parameter: `draw(locale, n)` not `draw(locale, &[n])`
- Parameters use `impl Into<Value>` for ergonomic conversion from i32, String, Phrase, etc.
- Locale is always the first parameter: `draw(&locale, n)`
- Functions return `Phrase` type

### Compile-time Error Messages
- Concise error with help line: "error: unknown phrase 'cards'" + "help: did you mean 'card'?"
- Typo suggestion threshold matches runtime: distance ≤ 1 for keys ≤ 3 chars, distance ≤ 2 for longer
- Show available variants when literal selector doesn't match: "note: available variants: one, other"
- Show full cycle chain for cyclic references: "error: cyclic reference: a -> b -> c -> a"

### Generated Artifacts
- PhraseId constants use SCREAMING_CASE: `phrase_ids::FIRE_ELEMENTAL`
- Simple doc comments on functions: `/// Returns the "card" phrase.`
- SOURCE_PHRASES embedding: Claude's discretion on verbatim vs normalized
- register_source_phrases hardcodes "en" as source language

### Macro Invocation Syntax
- Standard .rs file naming (e.g., `src/strings.rs`)
- One rlf! block per file
- Re-export common types (Locale, Phrase, Value) from the module
- No cross-file phrase references — each rlf! block is self-contained

### Claude's Discretion
- SOURCE_PHRASES embedding format (verbatim or normalized)
- Internal macro AST structure details
- Span preservation implementation approach
- Order of generated items in output

</decisions>

<specifics>
## Specific Ideas

- Match existing codebase patterns: use strsim for Levenshtein distance (already a dependency)
- Follow APPENDIX_RUST_INTEGRATION.md examples for generated code structure
- Validation checks should match the table in the appendix (7 check types)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-macro-code-generation*
*Context gathered: 2026-02-05*
