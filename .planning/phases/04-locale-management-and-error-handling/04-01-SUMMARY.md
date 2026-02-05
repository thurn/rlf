---
phase: 04-locale-management-and-error-handling
plan: 01
subsystem: interpreter
tags: [error-handling, strsim, levenshtein, did-you-mean]

# Dependency graph
requires:
  - phase: 02-interpreter-engine
    provides: EvalError enum and error infrastructure
provides:
  - LoadError enum for translation file loading errors
  - Enhanced MissingVariant with did-you-mean suggestions
  - compute_suggestions function using Levenshtein distance
affects: [04-02, locale-api, translation-loading]

# Tech tracking
tech-stack:
  added: [strsim 0.11]
  patterns: [did-you-mean suggestions via Levenshtein distance]

key-files:
  created: [crates/rlf/tests/interpreter_errors.rs]
  modified: [crates/rlf/src/interpreter/error.rs, crates/rlf/src/interpreter/evaluator.rs]

key-decisions:
  - "Max edit distance: 1 for short keys (<=3 chars), 2 for longer keys"
  - "Limit suggestions to 3, sorted by distance"

patterns-established:
  - "Error suggestions: use strsim::levenshtein with distance thresholds"
  - "PathBuf in errors: LoadError uses PathBuf for file paths in error messages"

# Metrics
duration: 3min
completed: 2026-02-05
---

# Phase 04 Plan 01: Error Types and Suggestions Summary

**LoadError enum for file loading failures with path/line/column context, plus MissingVariant enhanced with strsim-based did-you-mean suggestions**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-05T00:32:40Z
- **Completed:** 2026-02-05T00:35:46Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- LoadError enum with Io, Parse, and NoPathForReload variants for translation loading
- compute_suggestions function using Levenshtein distance for did-you-mean suggestions
- MissingVariant error now includes suggestions field with helpful alternatives
- 7 new error type tests verifying formatting and suggestion computation

## Task Commits

Each task was committed atomically:

1. **Task 1: Add strsim dependency and create LoadError** - `23b7810` (feat)
2. **Task 2: Enhance MissingVariant with suggestions** - `1c1509d` (feat)
3. **Task 3: Add error type tests** - `ca1d9b1` (test)

## Files Created/Modified

- `crates/rlf/Cargo.toml` - Added strsim 0.11 dependency
- `crates/rlf/src/interpreter/error.rs` - LoadError enum, compute_suggestions function, enhanced MissingVariant
- `crates/rlf/src/interpreter/evaluator.rs` - Updated MissingVariant call sites with suggestions
- `crates/rlf/src/interpreter/mod.rs` - Exported LoadError and compute_suggestions
- `crates/rlf/src/lib.rs` - Exported LoadError and compute_suggestions from crate
- `crates/rlf/tests/interpreter_errors.rs` - New test file for error types

## Decisions Made

- Used edit distance threshold of 1 for short keys (<=3 chars) and 2 for longer keys to avoid noise
- Limited suggestions to maximum of 3 to keep error messages concise
- Sorted suggestions by edit distance (closest match first)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Test expectation mismatch: Initial test for "oter" expected only "other" but got ["other", "one"] because both are within max_distance=2 for 4-character keys. Fixed test to verify "other" is returned first (closest match).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Error types ready for Locale API in Plan 02
- LoadError will be used for file-based translation loading
- compute_suggestions available for other error types if needed

---
*Phase: 04-locale-management-and-error-handling*
*Completed: 2026-02-05*
