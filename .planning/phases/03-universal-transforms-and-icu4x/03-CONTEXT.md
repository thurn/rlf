# Phase 3: Universal Transforms and ICU4X - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the transform execution system and universal case transforms (@cap, @upper, @lower). Ensure CLDR plural rules work for all 24 documented languages via ICU4X. This phase creates the infrastructure that language-specific transforms (Phases 6-9) will build upon.

Note: Phase 2 already implemented `plural_category` with ICU4X support. This phase focuses on transform execution.

</domain>

<decisions>
## Implementation Decisions

### Transform Execution Model
- Transforms receive `Value`, return `String` (as documented in APPENDIX_RUST_INTEGRATION)
- Transforms have access to full evaluation context (language code, phrase registry)
- Chain execution is right-to-left (innermost first): `{@cap @a card}` → @a first, then @cap
- Transform errors return `EvalError` (generated functions unwrap, causing panic)

### Transform Context Handling
- Context values can be literals OR parameters: `@der:acc` (literal) and `@count:n` (parameter) both work
- Context is resolved to a `Value` before passing to transform function
- Multiple context values are allowed: `@transform:ctx1:ctx2` syntax supported
- Parser distinguishes context vs selection by position: colons immediately after `@name` are context, colons after reference name are selection

### Universal Transform Behavior
- Full Unicode support: capitalize/uppercase/lowercase works for all Unicode letters (Cyrillic, Greek, etc.)
- Grapheme-aware: use unicode-segmentation crate to handle combining characters correctly
- Empty string input returns empty string (not an error)
- Locale-aware case mapping: Turkish i/ı/I/İ handled correctly based on `Locale` current language
- Use ICU4X for locale-sensitive case operations

### Registry Architecture
- Static dispatch via enum for transform functions (no trait objects or function pointers)
- Transform aliases (e.g., @an → @a) map to same enum variant in parser
- Language family fallback: pt-BR → pt → universal when looking up transforms
- Unknown transforms are compile-time errors for source language (rlf! macro rejects them)

### Claude's Discretion
- Specific enum variant names for transforms
- Internal TransformRegistry data structure layout
- Caching strategy for ICU4X case mappers
- Error message formatting for transform failures

</decisions>

<specifics>
## Specific Ideas

- "This is absolutely the sort of thing RLF should be handling explicitly via its `Locale` struct. It can default to current language semantics there, definitely do Turkish correctly."
- Design documents extensively describe transform behavior — implementation should follow DESIGN.md and APPENDIX_STDLIB.md closely

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-universal-transforms-and-icu4x*
*Context gathered: 2026-02-04*
