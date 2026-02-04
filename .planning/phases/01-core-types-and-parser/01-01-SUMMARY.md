---
phase: 01-core-types-and-parser
plan: 01
subsystem: types
tags: [rust, bon, serde, const-fn, fnv1a-hash, newtype]

# Dependency graph
requires: []
provides:
  - Phrase struct with builder pattern and variant fallback resolution
  - Value enum with Into impls for common Rust types
  - PhraseId with const fn hash-based identifier
  - VariantKey and Tag newtypes for type safety
affects: [01-02, 01-03, 02-interpreter, 03-macro]

# Tech tracking
tech-stack:
  added: [bon 3.8, thiserror 2.0, serde 1.0, const-fnv1a-hash 1.1, insta 1.42]
  patterns: [newtype pattern with Deref, bon builder derive, const fn construction]

key-files:
  created:
    - crates/rlf/src/types/phrase.rs
    - crates/rlf/src/types/value.rs
    - crates/rlf/src/types/phrase_id.rs
    - crates/rlf/src/types/variant_key.rs
    - crates/rlf/src/types/tag.rs
    - crates/rlf/Cargo.toml
    - Cargo.toml
    - justfile
  modified: []

key-decisions:
  - "Used const-fnv1a-hash crate for PhraseId (const fn support verified)"
  - "Added additional Into<Value> impls for u32, u64, usize, f32 beyond plan"
  - "Added Phrase::has_tag() and first_tag() helpers for future interpreter use"
  - "Added Value accessor methods (as_number, as_phrase, etc.) for type introspection"

patterns-established:
  - "Newtype pattern: wrap String, impl Deref to str, From<&str>, From<String>, Display"
  - "Builder pattern: use bon::Builder derive with #[builder(default)] for optional fields"
  - "Const construction: use const-fnv1a-hash for compile-time hash computation"

# Metrics
duration: 3min
completed: 2026-02-04
---

# Phase 01 Plan 01: Core Types Summary

**Phrase, Value, PhraseId, VariantKey, Tag types with bon builder, FNV-1a const fn hashing, and variant fallback resolution**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-04T21:11:29Z
- **Completed:** 2026-02-04T21:14:13Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments
- Rust workspace with `rlf` crate using edition 2024
- Phrase struct with builder pattern (via bon) and variant() fallback resolution per DESIGN.md
- PhraseId with const fn from_name() enabling compile-time constants
- VariantKey and Tag newtypes providing type safety over raw strings
- Value enum with comprehensive Into impls for i32, i64, u32, u64, usize, f32, f64, String, &str, Phrase
- All 5 doctests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Create workspace and crate structure** - `cb0ce9f` (chore)
2. **Task 2: Implement VariantKey and Tag newtypes** - `25375a0` (feat)
3. **Task 3: Implement Phrase, Value, and PhraseId types** - `4f0fe8c` (feat)

## Files Created/Modified
- `Cargo.toml` - Workspace root with resolver 2
- `crates/rlf/Cargo.toml` - Crate dependencies (bon, thiserror, serde, const-fnv1a-hash)
- `crates/rlf/src/lib.rs` - Public API re-exports
- `crates/rlf/src/types/mod.rs` - Module organization
- `crates/rlf/src/types/phrase.rs` - Phrase struct with builder and variant resolution
- `crates/rlf/src/types/value.rs` - Value enum with Into implementations
- `crates/rlf/src/types/phrase_id.rs` - PhraseId with const fn hash
- `crates/rlf/src/types/variant_key.rs` - VariantKey newtype
- `crates/rlf/src/types/tag.rs` - Tag newtype
- `justfile` - Build commands (check, test, build, fmt, lint)

## Decisions Made
- Used `const-fnv1a-hash` crate API: `fnv1a_hash_str_64(name)` takes single argument (research doc had outdated API with two args)
- Added additional numeric type conversions (u32, u64, usize, f32) to Value for broader compatibility
- Added helper methods to Phrase (has_tag, first_tag) and Value (as_number, as_phrase, etc.) for interpreter use

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed const-fnv1a-hash API call**
- **Found during:** Task 3 (PhraseId implementation)
- **Issue:** Research doc showed `fnv1a_hash_str_64(name, None)` but actual API is `fnv1a_hash_str_64(name)`
- **Fix:** Removed second argument
- **Files modified:** crates/rlf/src/types/phrase_id.rs
- **Verification:** `cargo check` passes, const fn works in doctests
- **Committed in:** 4f0fe8c (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minor API correction, no scope change.

## Issues Encountered
None - all tasks executed smoothly after API fix.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Core types ready for parser implementation (Plan 02)
- All types exported from crate root: `use rlf::{Phrase, Value, PhraseId, VariantKey, Tag};`
- Phrase::variant() implements fallback resolution per DESIGN.md
- PhraseId::from_name() is const fn, verified with compile-time constant in doctest

---
*Phase: 01-core-types-and-parser*
*Completed: 2026-02-04*
