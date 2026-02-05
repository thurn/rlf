---
phase: 10-cli-tools
plan: 01
subsystem: cli
tags: [clap, miette, owo-colors, cli, diagnostics]

# Dependency graph
requires:
  - phase: 01-core-types-and-parser
    provides: ParseError type, parse_file function
provides:
  - rlf-cli crate with rlf binary
  - rlf check command for syntax validation
  - RlfDiagnostic miette wrapper for compiler-quality errors
  - JSON output mode for CI integration
  - strict mode for translation validation
affects: [10-02-PLAN.md, 10-03-PLAN.md]

# Tech tracking
tech-stack:
  added: [clap 4, miette 7, owo-colors 4, exitcode 1]
  patterns: [git-style subcommands, miette diagnostic wrapper]

key-files:
  created:
    - crates/rlf-cli/Cargo.toml
    - crates/rlf-cli/src/main.rs
    - crates/rlf-cli/src/commands/mod.rs
    - crates/rlf-cli/src/commands/check.rs
    - crates/rlf-cli/src/output/mod.rs
    - crates/rlf-cli/src/output/diagnostic.rs
  modified: []

key-decisions:
  - "RlfDiagnostic wraps ParseError with NamedSource for source context display"
  - "Byte offset calculated from line:column by summing line lengths"
  - "Exit code 65 (DATAERR) for syntax errors, 0 for success"
  - "Module-level unused_assignments allow for miette derive false positive"

patterns-established:
  - "CLI commands in commands/ module with run_X function pattern"
  - "Output formatting in output/ module with diagnostic wrappers"
  - "JSON output via serde Serialize with --json flag gating"

# Metrics
duration: 5min
completed: 2026-02-05
---

# Phase 10 Plan 01: CLI Check Command Summary

**rlf-cli crate with `rlf check` command for syntax validation using clap subcommands and miette diagnostics**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-05T14:43:40Z
- **Completed:** 2026-02-05T14:48:36Z
- **Tasks:** 2
- **Files created:** 6

## Accomplishments

- Created rlf-cli crate with clap derive for git-style subcommands
- Implemented `rlf check` command with miette compiler-quality diagnostics
- Added --json flag for machine-readable output and CI integration
- Added --strict flag to validate translations against source file

## Task Commits

Each task was committed atomically:

1. **Task 1: Create rlf-cli crate with clap subcommand structure** - `8434d1d` (feat)
2. **Task 2: Implement check command with miette diagnostics** - `5b6f7ff` (feat)

## Files Created

- `crates/rlf-cli/Cargo.toml` - CLI crate with clap, miette, owo-colors dependencies
- `crates/rlf-cli/src/main.rs` - CLI entry point with Cli struct and Commands enum
- `crates/rlf-cli/src/commands/mod.rs` - Command module re-exports
- `crates/rlf-cli/src/commands/check.rs` - Check command with file validation logic
- `crates/rlf-cli/src/output/mod.rs` - Output module re-exports
- `crates/rlf-cli/src/output/diagnostic.rs` - RlfDiagnostic miette wrapper

## Decisions Made

- **RlfDiagnostic byte offset:** Convert line:column to byte offset by summing (line_length + 1) for preceding lines
- **Exit codes:** Use exitcode crate constants (OK=0, DATAERR=65) for CI-friendly differentiated exits
- **Module-level lint exception:** Added `#![allow(unused_assignments)]` to diagnostic.rs for miette derive false positive

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- **Clippy unused_assignments false positive:** The miette derive macro generates code that reads struct fields, but rustc cannot track this and warns about "value assigned is never read". Fixed with module-level allow attribute and documented the reason.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CLI foundation complete with subcommand pattern established
- Ready for eval and coverage commands in subsequent plans
- Commands module ready to expand with additional subcommands

---
*Phase: 10-cli-tools*
*Completed: 2026-02-05*
