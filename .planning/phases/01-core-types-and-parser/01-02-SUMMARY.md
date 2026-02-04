---
phase: 01-core-types-and-parser
plan: 02
subsystem: parser
tags: [winnow, parsing, ast, template]

dependency-graph:
  requires: ["01-01"]
  provides: ["parse_template", "parse_file", "Template", "Segment", "Transform", "Reference", "Selector", "PhraseDefinition"]
  affects: ["01-03", "02-xx", "03-xx"]

tech-stack:
  added: [winnow]
  patterns: [parser-combinator, ast-visitor]

file-tracking:
  key-files:
    created:
      - crates/rlf/src/parser/mod.rs
      - crates/rlf/src/parser/ast.rs
      - crates/rlf/src/parser/error.rs
      - crates/rlf/src/parser/template.rs
      - crates/rlf/src/parser/file.rs
      - crates/rlf/tests/template_parser.rs
    modified:
      - crates/rlf/Cargo.toml
      - crates/rlf/src/lib.rs

decisions:
  - id: "parser-01"
    choice: "Reference::Identifier unifies parameters and phrases at parse time"
    rationale: "Resolution happens during interpretation, not parsing"
  - id: "parser-02"
    choice: "Auto-capitalization adds @cap transform, doesn't modify reference"
    rationale: "Keeps AST transformation semantics clean"
  - id: "parser-03"
    choice: "Selector::Identifier defers literal vs parameter distinction"
    rationale: "Same rationale as Reference - resolve during interpretation"

metrics:
  duration: "5 min"
  completed: "2026-02-04"
---

# Phase 01 Plan 02: Template String Parser Summary

**One-liner:** winnow-based parser for RLF templates with interpolations, transforms, selections, and escape sequences

## What Was Built

### Template Parser (`parse_template`)
- Parses template strings into structured AST
- Handles literal text segments merged efficiently
- Parses interpolations: `{transforms* reference selectors*}`
- Escape sequences: `{{` `}}` `@@` `::` produce literal characters
- Automatic capitalization: `{Card}` becomes `@cap` transform + `card` reference

### Template AST Types
- `Template` - collection of segments
- `Segment::Literal` - literal text
- `Segment::Interpolation` - transforms + reference + selectors
- `Transform` - name with optional context (e.g., `@der:acc`)
- `Reference::Identifier` - simple reference (parameter or phrase)
- `Reference::PhraseCall` - phrase call with arguments
- `Selector::Identifier` - variant selector

### File Parser (`parse_file`) - BONUS
- Parses complete `.rlf` files with phrase definitions
- Supports tags (`:fem`, `:masc`, `:a`, `:an`)
- Supports `:from(param)` inheritance modifier
- Supports variant blocks with multi-key syntax
- Supports line comments (`// ...`)

### File AST Types - BONUS
- `PhraseDefinition` - complete phrase definition
- `PhraseBody::Simple` - simple template phrase
- `PhraseBody::Variants` - variant block phrase
- `VariantEntry` - keys + template for one variant

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 2ef9ac5 | feat | Add winnow dependency and parser module structure |
| 57a2215 | feat | Implement template string parser with winnow |
| f8122a8 | test | Add comprehensive template parser integration tests |
| 0c5ac1c | feat | Add file parser and file-level AST types (bonus) |

## Test Coverage

- 20 unit tests for template parser
- 12 unit tests for file parser
- 46 integration tests for template parser public API
- 5 doc tests

**Total: 83 tests**

## LANG-* Requirements Coverage

| Requirement | Status | Notes |
|-------------|--------|-------|
| LANG-01 | Done | Literal text segments |
| LANG-02 | Done | Parameter interpolation `{name}` |
| LANG-03 | Done | Phrase interpolation `{phrase}` |
| LANG-04 | Done | Selection with literal `:one`, `:other` |
| LANG-05 | Done | Selection with parameter `:n` |
| LANG-06 | Done | Multi-dimensional selection `:acc:n` |
| LANG-07 | Done | Transforms `@cap`, `@upper`, `@lower` |
| LANG-08 | Done | Transform with context `@der:acc` |
| LANG-09 | Done | Chained transforms `@cap @a` |
| LANG-10 | Done | Phrase call with arguments `{foo(x, y)}` |
| LANG-11 | Done | Escape sequences `{{ }} @@ ::` |
| LANG-12 | Done | Automatic capitalization `{Card}` -> `@cap card` |
| LANG-13 | Done | Variant definitions (via file parser) |
| LANG-14 | Done | Metadata tags (via file parser) |
| LANG-15 | Done | `:from(param)` inheritance (via file parser) |
| LANG-16 | Done | Multi-key variants (via file parser) |
| LANG-17 | Done | Comments (via file parser) |

## Deviations from Plan

### Auto-added (Rule 2 - Missing Critical)

**1. File parser and file-level AST types**
- **Found during:** Task completion
- **Issue:** Linter recognized these types were needed for complete parser functionality
- **Fix:** Added `parse_file()`, `PhraseDefinition`, `PhraseBody`, `VariantEntry`
- **Files created:** `crates/rlf/src/parser/file.rs`, modified `ast.rs`
- **Commit:** 0c5ac1c
- **Impact:** Phase 03 file parser work is now complete

## Key Implementation Details

### Parser Design
- Uses winnow 0.7 parser combinators
- Escape sequences parsed FIRST in alt() to avoid ambiguity
- Adjacent literals merged post-parsing for efficiency
- Line/column positions calculated for error messages

### Auto-capitalization
- Triggered when reference starts with uppercase letter
- Prepends `@cap` transform to transforms list
- Lowercases first letter of reference name
- Works with existing transforms: `{@a Card}` -> `@cap @a card`

### Identifier Resolution Deferred
- `Reference::Identifier` doesn't distinguish parameter vs phrase
- `Selector::Identifier` doesn't distinguish literal vs parameter
- Resolution happens during interpretation, not parsing
- Enables simpler parser with looser coupling

## Next Phase Readiness

**01-03 (Phrase Definition Parser):** Already complete via bonus file parser work. Can be skipped or used for additional validation tests.

**02-xx (Interpreter):** Ready to consume AST types. Template and file parsers provide all needed input structures.
