# Phase 1: Core Types and Parser - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Foundational Rust types (Phrase, Value, PhraseId) and parser for template strings and .rlf file format. This phase builds the data structures and parsing logic that all other phases depend on. The interpreter engine that evaluates these structures is Phase 2.

</domain>

<decisions>
## Implementation Decisions

### Type API Style
- Use the `bon` crate for builder patterns on types
- Phrase struct fields are public (pub text, pub variants, pub tags)
- Types are exhaustive (no #[non_exhaustive])

### Type Safety
- Variant keys use a newtype: `VariantKey` wrapping String
- Tags use a newtype: `Tag` wrapping String
- This provides type safety over plain HashMap<String, String>

### Parser Error Messages
- Standard verbosity with context: show offending line, caret pointing to error, brief explanation
- Use Display + thiserror for error types (no fancy diagnostic libraries)
- Error spans use line:column only (no byte offsets)
- Fail fast on first error (no error recovery/multiple error collection)

### File Format
- Line comments only: `// comment`
- UTF-8 encoding only (reject non-UTF-8 with clear error)
- Allow trailing commas in variant lists: `{ one: "x", other: "y", }`
- Phrase names use snake_case only (underscores, no hyphens)

### Extensibility
- Transforms use an enum: `pub enum Transform { Cap, Upper, Lower, A, Der, ... }` (closed set)
- Parser syntax is strict to current spec (no reserved keywords for future)
- Parser AST types are public (enables external tooling)

### Claude's Discretion
- Internal parser implementation details
- Exact error message wording
- Performance optimizations in parsing

</decisions>

<specifics>
## Specific Ideas

- PhraseId is a u64 containing FNV-1a hash of the phrase name (from DESIGN.md)
- Design docs in `docs/` are comprehensive and should be followed precisely
- Reference docs: DESIGN.md, APPENDIX_RUST_INTEGRATION.md for type/API details

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-core-types-and-parser*
*Context gathered: 2026-02-04*
