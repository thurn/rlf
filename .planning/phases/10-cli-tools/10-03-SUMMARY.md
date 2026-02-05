---
phase: 10-cli-tools
plan: 03
subsystem: cli
tags: [coverage, comfy-table, ascii-table, ci-enforcement]

# Dependency graph
requires:
  - phase: 10-02
    provides: CLI eval command and parameter parsing patterns
  - phase: 01-03
    provides: parse_file function for extracting phrase definitions
provides:
  - rlf coverage command for translation coverage reporting
  - comfy-table integration for ASCII table output
  - --strict flag for CI pipeline enforcement
affects: []

# Tech tracking
tech-stack:
  added: [comfy-table]
  patterns: [coverage-comparison-via-set-intersection]

key-files:
  created: [crates/rlf-cli/src/commands/coverage.rs, crates/rlf-cli/src/output/table.rs]
  modified: [crates/rlf-cli/src/commands/mod.rs, crates/rlf-cli/src/main.rs, crates/rlf-cli/Cargo.toml, crates/rlf-cli/src/output/mod.rs]

key-decisions:
  - "comfy-table with UTF8_BORDERS_ONLY preset for clean ASCII table output"
  - "Set intersection for coverage: source names vs translated names"
  - "Exit code 65 (DATAERR) for --strict with incomplete translations"

patterns-established:
  - "Coverage table structure: Language, Coverage (X/Y), Missing columns"

# Metrics
duration: 4min
completed: 2026-02-05
---

# Phase 10 Plan 03: Coverage Command Summary

**rlf coverage command with comfy-table ASCII output, --strict CI enforcement, and per-language missing phrase lists**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-05T14:55:40Z
- **Completed:** 2026-02-05T14:59:49Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Coverage command compares source and translation phrase names
- ASCII table shows absolute counts (X/Y) not percentages
- Missing phrase names listed per language after table
- --strict flag exits non-zero for CI pipeline enforcement
- JSON output mode for machine-readable results
- Missing translation files handled gracefully as 0% coverage

## Task Commits

Each task was committed atomically:

1. **Task 1: Add comfy-table dependency and create table formatter** - `511ec1c` (feat)
2. **Task 2: Implement coverage command** - `31a9272` (feat)

## Files Created/Modified
- `crates/rlf-cli/Cargo.toml` - Added comfy-table dependency
- `crates/rlf-cli/src/output/table.rs` - LanguageCoverage struct and format_coverage_table function
- `crates/rlf-cli/src/output/mod.rs` - Export table module
- `crates/rlf-cli/src/commands/coverage.rs` - CoverageArgs and run_coverage implementation
- `crates/rlf-cli/src/commands/mod.rs` - Export coverage module
- `crates/rlf-cli/src/main.rs` - Add Coverage variant to Commands enum

## Decisions Made
- Used comfy-table with UTF8_BORDERS_ONLY preset for clean terminal output
- Coverage computed as set intersection of phrase names (not template comparison)
- Exit code 65 (DATAERR) for --strict with incomplete translations (same as check command)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 10 complete: all three CLI commands implemented (check, eval, coverage)
- CLI tools ready for localization workflow integration
- No blockers for project completion

---
*Phase: 10-cli-tools*
*Completed: 2026-02-05*
