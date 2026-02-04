---
phase: 03-universal-transforms-and-icu4x
plan: 01
subsystem: interpreter
tags: [icu4x, unicode, case-mapping, grapheme, transforms]

# Dependency graph
requires:
  - phase: 02-interpreter-engine
    provides: EvalError, TransformRegistry stub, Value type
provides:
  - TransformKind enum with Cap/Upper/Lower variants
  - ICU4X-based locale-sensitive case mapping
  - Grapheme-aware @cap transform using unicode-segmentation
  - UnknownTransform error variant
affects: [04-declension-framework, 05-gendered-pronouns, 06-latin-declension]

# Tech tracking
tech-stack:
  added: [icu_casemap, unicode-segmentation]
  patterns: [static dispatch via enum, locale-sensitive transforms]

key-files:
  created: []
  modified:
    - crates/rlf/Cargo.toml
    - crates/rlf/src/interpreter/transforms.rs
    - crates/rlf/src/interpreter/error.rs
    - crates/rlf/src/interpreter/mod.rs

key-decisions:
  - "Static dispatch via TransformKind enum, no trait objects or function pointers"
  - "ICU4X CaseMapper for locale-sensitive case mapping (Turkish dotted-I)"
  - "unicode-segmentation graphemes(true) for proper first-character handling"

patterns-established:
  - "TransformKind enum pattern: Add variants to enum, implement execute() method"
  - "Locale parsing: Use lang.parse().unwrap_or(langid!(und)) for fallback"

# Metrics
duration: 2min
completed: 2026-02-04
---

# Phase 03 Plan 01: Universal Case Transforms Summary

**TransformKind enum with @cap/@upper/@lower using ICU4X CaseMapper for locale-sensitive case mapping and unicode-segmentation for grapheme awareness**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-04T23:43:50Z
- **Completed:** 2026-02-04T23:45:39Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- TransformKind enum with Cap, Upper, Lower variants for static dispatch
- ICU4X CaseMapper integration for Turkish dotted-I handling (istanbul -> ISTANBUL)
- Grapheme-aware @cap using unicode-segmentation
- UnknownTransform error variant added to EvalError

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ICU4X casemap and unicode-segmentation dependencies** - `fa92059` (chore)
2. **Task 2: Implement TransformKind enum and case transforms** - `1e199ab` (feat)

## Files Created/Modified
- `crates/rlf/Cargo.toml` - Added icu_casemap and unicode-segmentation dependencies
- `crates/rlf/src/interpreter/transforms.rs` - TransformKind enum, case transform implementations
- `crates/rlf/src/interpreter/error.rs` - UnknownTransform error variant
- `crates/rlf/src/interpreter/mod.rs` - Export TransformKind

## Decisions Made
- Static dispatch via TransformKind enum per CONTEXT.md guidelines
- ICU4X CaseMapper returns Cow<str>, use .into_owned() for String
- unwrap_or() instead of unwrap_or_else() for langid! macro (clippy lint)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Cow<str> return type mismatch**
- **Found during:** Task 2 (case transform implementation)
- **Issue:** ICU4X CaseMapper::uppercase_to_string returns Cow<str>, not String
- **Fix:** Added .into_owned() to convert to owned String
- **Files modified:** crates/rlf/src/interpreter/transforms.rs
- **Verification:** cargo check passes
- **Committed in:** 1e199ab (Task 2 commit)

**2. [Rule 1 - Bug] Fixed clippy unnecessary_lazy_evaluations lint**
- **Found during:** Task 2 (case transform implementation)
- **Issue:** unwrap_or_else with closure on langid! macro triggers clippy
- **Fix:** Changed to unwrap_or() which works with const macro
- **Files modified:** crates/rlf/src/interpreter/transforms.rs
- **Verification:** just review passes
- **Committed in:** 1e199ab (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both were type system and lint fixes. No scope creep.

## Issues Encountered
None - ICU4X API discovery and lint fixes handled within task execution.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- TransformKind enum ready for extension with language-specific transforms
- TransformRegistry::get() pattern established for transform lookup
- Ready for 03-02 (Transform Integration) to wire transforms into evaluator

---
*Phase: 03-universal-transforms-and-icu4x*
*Completed: 2026-02-04*
