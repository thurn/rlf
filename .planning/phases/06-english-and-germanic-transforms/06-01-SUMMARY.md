---
phase: 06-english-and-germanic-transforms
plan: 01
subsystem: transforms
tags: [english, articles, a, an, the, tag-based-selection]

# Dependency graph
requires:
  - phase: 03-universal-transforms-and-icu4x
    provides: TransformKind enum, TransformRegistry, transform execution pipeline
  - phase: 04-locale-management-and-error-handling
    provides: Locale API, MissingTag error variant
provides:
  - EnglishA transform reading :a/:an tags
  - EnglishThe transform for definite article
  - @an alias resolving to @a
  - Value-based transform execution (tags accessible to transforms)
affects: [07-romance-transforms, 08-slavic-transforms, 09-numeric-transforms]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Tag-based article selection (not phonetic guessing)
    - Value passed to transforms preserves Phrase type with tags
    - Transform aliases in TransformRegistry

key-files:
  created: []
  modified:
    - crates/rlf/src/interpreter/evaluator.rs
    - crates/rlf/src/interpreter/transforms.rs
    - crates/rlf/tests/interpreter_transforms.rs

key-decisions:
  - "apply_selectors returns Value (not String) to preserve Phrase tags"
  - "apply_transforms accepts Value, first transform sees original Phrase with tags"
  - "Selector application strips tags (variant lookup returns String)"
  - "@an alias resolves to @a in TransformRegistry"

patterns-established:
  - "Language-specific transforms check (lang, name) tuple in registry"
  - "Tag-based article selection: transforms read tags, not phonetic analysis"
  - "Transform result wrapping: after first transform, Value becomes String"

# Metrics
duration: 5min
completed: 2026-02-05
---

# Phase 06 Plan 01: English Article Transforms Summary

**English @a/@an and @the transforms with tag-based article selection and Value-passing for tag access**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-05T03:54:52Z
- **Completed:** 2026-02-05T03:59:42Z
- **Tasks:** 4 (1a, 1b, 2, 3)
- **Files modified:** 3

## Accomplishments

- Refactored evaluator to pass Value (not String) to transforms, enabling tag access
- Implemented EnglishA transform reading :a/:an tags from Phrase values
- Implemented EnglishThe transform prepending "the" unconditionally
- Added @an alias resolving to @a in TransformRegistry
- Comprehensive test coverage: 19 English transform tests (unit + integration)

## Task Commits

Each task was committed atomically:

1. **Task 1a: Modify apply_selectors to return Value** - `b46fd18` (refactor)
2. **Task 1b: Modify apply_transforms to accept Value** - `84991a7` (refactor)
3. **Task 2: Add English transforms with TDD** - `bf1ebb1` (feat)
4. **Task 3: Integration tests with full evaluation** - `2bc19a7` (test)

## Files Created/Modified

- `crates/rlf/src/interpreter/evaluator.rs` - apply_selectors returns Value, apply_transforms accepts Value
- `crates/rlf/src/interpreter/transforms.rs` - EnglishA, EnglishThe variants, helper functions, registry aliases
- `crates/rlf/tests/interpreter_transforms.rs` - 19 new tests for English transforms

## Decisions Made

- **apply_selectors returns Value:** When no selectors present, original Value (Phrase with tags) is preserved. After selector application, returns String (variant lookup result loses tags).
- **apply_transforms accepts &Value:** First transform receives full Value (can read tags). Subsequent transforms receive Value::String.
- **@an alias:** Resolves to @a in TransformRegistry.get() before looking up language-specific transforms.
- **English transforms only for "en" language:** registry.get("a", "de") returns None.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Minor: Test imports needed adjustment (`Locale` import path, `Tag::new()` instead of `String`)
- Minor: Clippy warning on `to_string()` in format! macro - fixed with inline formatting

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Transform infrastructure now supports tag-based selection
- Ready for additional language-specific transforms (German articles, Romance gender agreement)
- Pattern established: language-specific transforms check (lang, canonical_name) tuple

---
*Phase: 06-english-and-germanic-transforms*
*Completed: 2026-02-05*
