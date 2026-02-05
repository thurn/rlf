---
phase: 09-asian-language-transforms
plan: 02
subsystem: transforms
tags: [vietnamese, thai, bengali, indonesian, classifiers, reduplication, sea, southeast-asian]

# Dependency graph
requires:
  - phase: 09-01
    provides: CJK @count transforms, find_classifier helper, context_to_count helper
provides:
  - Vietnamese @count transform with 5 classifiers (cai, con, nguoi, chiec, to)
  - Thai @count transform with 4 classifiers (bai, tua, khon, an)
  - Bengali @count transform with 4 classifiers (ta, ti, khana, jon)
  - Indonesian @plural transform with reduplication pattern
affects: [09-03-korean-particle, future-classifier-languages]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Vietnamese uses spaces between count elements"
    - "Thai uses no spaces between count elements"
    - "Bengali attaches classifier to number, space before noun"
    - "Indonesian @plural uses simple reduplication"

key-files:
  created: []
  modified:
    - "crates/rlf/src/interpreter/transforms.rs"
    - "crates/rlf/tests/interpreter_transforms.rs"

key-decisions:
  - "Vietnamese @count format: '{count} {classifier} {text}' with spaces"
  - "Thai @count format: '{count}{classifier}{text}' no spaces"
  - "Bengali @count format: '{count}{classifier} {text}' classifier attached to number"
  - "Indonesian @plural: simple text reduplication with hyphen"

patterns-established:
  - "SEA @count transforms follow CJK pattern but with language-specific spacing"
  - "Indonesian @plural is stateless reduplication, no tags needed"

# Metrics
duration: 8min
completed: 2026-02-05
---

# Phase 9 Plan 2: SEA Count Transforms Summary

**Vietnamese, Thai, Bengali @count transforms with classifiers plus Indonesian @plural reduplication**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-05T06:30:00Z
- **Completed:** 2026-02-05T06:38:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Implemented Vietnamese @count with 5 classifiers (cai, con, nguoi, chiec, to) and space-separated format
- Implemented Thai @count with 4 classifiers (bai, tua, khon, an) and no-space format
- Implemented Bengali @count with 4 classifiers (ta, ti, khana, jon) and classifier-attached format
- Implemented Indonesian @plural with simple hyphenated reduplication (text-text)
- Added 19 comprehensive tests covering all transforms and edge cases

## Task Commits

Tasks 1-2 were combined as they form a cohesive implementation unit:

1. **Tasks 1-2: Add TransformKind variants, classifiers, and transform implementations** - `ca25e81` (feat)
2. **Task 3: Add comprehensive tests for SEA transforms** - `dddff3d` (test)

## Files Created/Modified
- `crates/rlf/src/interpreter/transforms.rs` - Added 4 new TransformKind variants, 3 classifier arrays, 4 transform functions, and registry entries
- `crates/rlf/tests/interpreter_transforms.rs` - Added 19 tests for Vietnamese, Thai, Bengali, and Indonesian transforms

## Decisions Made
- Vietnamese uses ASCII tag names (cai, nguoi, chiec) mapped to ASCII classifier words (no diacritics in tags)
- Vietnamese format includes spaces between all elements: "{count} {classifier} {text}"
- Thai format follows CJK pattern with no spaces: "{count}{classifier}{text}"
- Bengali classifier attaches to number with space before noun: "{count}{classifier} {text}"
- Indonesian @plural is the simplest transform - pure reduplication with no tags or context

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - implementation was straightforward using patterns established in Plan 01.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Southeast Asian @count transforms complete
- Phase 9 Plan 3 (Korean @particle) can proceed independently
- Total test count: 490 passing (19 new tests added)

---
*Phase: 09-asian-language-transforms*
*Plan: 02*
*Completed: 2026-02-05*
