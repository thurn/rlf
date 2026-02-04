# Roadmap: RLF

## Overview

RLF (Rust Localization Framework) is built from the ground up: core types and parser first, then interpreter engine, followed by macro code generation that depends on both. Transforms are layered in groups by linguistic similarity, starting with universal transforms and ICU4X integration, then European languages, then Asian/Middle Eastern. The CLI comes last once the library is feature-complete. This order ensures each layer has stable foundations before it ships.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Core Types and Parser** - Foundation types (Phrase, Value, PhraseId) and parser for template/file syntax
- [x] **Phase 2: Interpreter Engine** - Evaluation engine for templates with variant resolution and phrase calls
- [x] **Phase 3: Universal Transforms and ICU4X** - Case transforms and CLDR plural rules integration
- [ ] **Phase 4: Locale Management and Error Handling** - Locale struct, phrase loading, and comprehensive error types
- [ ] **Phase 5: Macro Code Generation** - rlf! macro parsing, code generation, and compile-time validation
- [ ] **Phase 6: English and Germanic Transforms** - Article transforms for English, German, and Dutch
- [ ] **Phase 7: Romance Language Transforms** - Article and contraction transforms for Spanish, French, Portuguese, Italian
- [ ] **Phase 8: Greek, Romanian, and Middle Eastern Transforms** - Greek articles, Romanian postposed articles, Arabic/Persian
- [ ] **Phase 9: Asian Language Transforms** - CJK counters, Korean particles, Turkish inflection, Indonesian plural
- [ ] **Phase 10: CLI Tools** - rlf check, rlf eval, rlf coverage commands

## Phase Details

### Phase 1: Core Types and Parser
**Goal**: Foundational types and parsing exist so interpreter and macro can build on them
**Depends on**: Nothing (first phase)
**Requirements**: TYPE-01, TYPE-02, TYPE-03, TYPE-04, TYPE-05, TYPE-06, TYPE-07, TYPE-08, TYPE-09, TYPE-10, TYPE-11, INTERP-01, INTERP-02, LANG-01, LANG-02, LANG-03, LANG-04, LANG-05, LANG-06, LANG-07, LANG-08, LANG-09, LANG-10, LANG-11, LANG-12, LANG-13, LANG-14, LANG-15, LANG-16, LANG-17
**Success Criteria** (what must be TRUE):
  1. Phrase struct can hold text, variants HashMap, and tags Vec
  2. PhraseId can be constructed at const time from phrase name and used in HashMap keys
  3. Parser can parse phrase definitions with parameters, variants, metadata, and transforms from string
  4. Parser can parse .rlf file format with multiple phrase definitions
  5. All escape sequences and syntax forms from DESIGN.md are recognized by parser
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Core types (Phrase, Value, PhraseId, VariantKey, Tag) and crate setup
- [x] 01-02-PLAN.md — Template string parser (interpolations, transforms, selections, escapes)
- [x] 01-03-PLAN.md — File format parser (.rlf phrase definitions, variants, metadata)

### Phase 2: Interpreter Engine
**Goal**: Interpreter can evaluate templates and resolve phrases with variants and parameters
**Depends on**: Phase 1
**Requirements**: INTERP-03, INTERP-04, INTERP-05, INTERP-06, INTERP-07, INTERP-08, INTERP-09, INTERP-10, INTERP-11, INTERP-12, INTERP-13, INTERP-14, INTERP-15, INTERP-16, INTERP-17
**Success Criteria** (what must be TRUE):
  1. eval_str() evaluates a template with parameter map and returns formatted string
  2. call_phrase() resolves phrase by name, passes arguments, returns result
  3. Variant selection works with dot-notation keys and fallback resolution
  4. Tag-based selection reads phrase metadata and uses as variant key
  5. Cycle detection prevents infinite recursion and max depth limit enforced
**Plans**: 2 plans

Plans:
- [x] 02-01-PLAN.md — Interpreter foundation: EvalError, PhraseRegistry, EvalContext, CLDR plural rules
- [x] 02-02-PLAN.md — Template evaluation: eval_str, call_phrase, get_phrase, variant resolution, phrase calls

### Phase 3: Universal Transforms and ICU4X
**Goal**: Case transforms and plural rules work for all languages
**Depends on**: Phase 2
**Requirements**: XFORM-01, XFORM-02, XFORM-03, ICU-01, ICU-02, ICU-03
**Success Criteria** (what must be TRUE):
  1. @cap, @upper, @lower transforms work on any input string
  2. Numeric selection uses CLDR plural category (zero, one, two, few, many, other)
  3. All 24 documented languages have working plural rules via ICU4X
**Plans**: 2 plans

Plans:
- [x] 03-01-PLAN.md — TransformKind enum with ICU4X case transforms (@cap, @upper, @lower)
- [x] 03-02-PLAN.md — Evaluator transform wiring and comprehensive tests

### Phase 4: Locale Management and Error Handling
**Goal**: Users can manage language selection and get clear errors on failures
**Depends on**: Phase 3
**Requirements**: LOC-01, LOC-02, LOC-03, LOC-04, LOC-05, LOC-06, LOC-07, LOC-08, LOC-09, ERR-01, ERR-02, ERR-03, ERR-04, ERR-05, ERR-06, ERR-07, ERR-08, ERR-09
**Success Criteria** (what must be TRUE):
  1. Locale struct can be created, language can be changed, translations loaded from file or string
  2. Hot-reloading via reload_translations() updates phrases without restart
  3. LoadError provides file, line, column for parse failures
  4. EvalError variants clearly indicate what failed (phrase not found, missing variant, etc.)
  5. Missing translations return error, not silent fallback
**Plans**: TBD

Plans:
- [ ] 04-01: TBD
- [ ] 04-02: TBD

### Phase 5: Macro Code Generation
**Goal**: rlf! macro generates typed Rust functions with compile-time validation
**Depends on**: Phase 4
**Requirements**: MACRO-01, MACRO-02, MACRO-03, MACRO-04, MACRO-05, MACRO-06, MACRO-07, MACRO-08, MACRO-09, MACRO-10, MACRO-11, MACRO-12, MACRO-13, MACRO-14, MACRO-15, MACRO-16, MACRO-17
**Success Criteria** (what must be TRUE):
  1. rlf! block generates one Rust function per phrase definition
  2. Generated functions accept typed parameters and return Phrase
  3. Undefined phrase/parameter references cause compile error with helpful message
  4. Cyclic references detected at compile time
  5. IDE autocomplete works immediately after adding phrase to rlf! block
**Plans**: TBD

Plans:
- [ ] 05-01: TBD
- [ ] 05-02: TBD
- [ ] 05-03: TBD

### Phase 6: English and Germanic Transforms
**Goal**: Article transforms work for English, German, and Dutch
**Depends on**: Phase 3
**Requirements**: EN-01, EN-02, DE-01, DE-02, NL-01, NL-02
**Success Criteria** (what must be TRUE):
  1. English @a/@an selects article from :a/:an tags, @the produces "the"
  2. German @der/@die/@das and @ein/@eine select by gender and case
  3. Dutch @de/@het selects by gender tag, @een produces indefinite
**Plans**: TBD

Plans:
- [ ] 06-01: TBD

### Phase 7: Romance Language Transforms
**Goal**: Article and contraction transforms work for Spanish, French, Portuguese, Italian
**Depends on**: Phase 3
**Requirements**: ES-01, ES-02, ES-03, FR-01, FR-02, FR-03, FR-04, FR-05, PT-01, PT-02, PT-03, PT-04, IT-01, IT-02, IT-03, IT-04
**Success Criteria** (what must be TRUE):
  1. Spanish @el/@la and @un/@una select by :masc/:fem tags with plural context
  2. French @le/@la handles elision from :vowel, contractions work (@de, @au)
  3. Portuguese articles work with @de/@em contractions
  4. Italian @il/@lo/@la handles sound rules, contractions work
**Plans**: TBD

Plans:
- [ ] 07-01: TBD
- [ ] 07-02: TBD

### Phase 8: Greek, Romanian, and Middle Eastern Transforms
**Goal**: Article transforms for Greek/Romanian and special transforms for Arabic/Persian
**Depends on**: Phase 3
**Requirements**: EL-01, EL-02, RO-01, AR-01, FA-01
**Success Criteria** (what must be TRUE):
  1. Greek @o/@i/@to and indefinite articles select by gender and case
  2. Romanian @def produces postposed definite article
  3. Arabic @al handles sun/moon letter assimilation
  4. Persian @ezafe produces connector based on :vowel tag
**Plans**: TBD

Plans:
- [ ] 08-01: TBD

### Phase 9: Asian Language Transforms
**Goal**: Counter/classifier systems and special transforms for Asian languages
**Depends on**: Phase 3
**Requirements**: CJK-01, CJK-02, CJK-03, CJK-04, CJK-05, CJK-06, KO-01, TR-01, ID-01
**Success Criteria** (what must be TRUE):
  1. Chinese/Japanese/Korean/Vietnamese/Thai/Bengali @count produces number + classifier
  2. Korean @particle selects particle based on final sound of preceding word
  3. Turkish @inflect applies agglutinative suffixes with vowel harmony
  4. Indonesian @plural produces reduplication
**Plans**: TBD

Plans:
- [ ] 09-01: TBD
- [ ] 09-02: TBD

### Phase 10: CLI Tools
**Goal**: Command-line tools for validation, evaluation, and coverage checking
**Depends on**: Phase 5
**Requirements**: CLI-01, CLI-02, CLI-03, CLI-04, CLI-05, CLI-06, CLI-07, CLI-08, CLI-09, CLI-10, CLI-11, CLI-12
**Success Criteria** (what must be TRUE):
  1. `rlf check <file>` validates .rlf syntax and reports errors with line/column
  2. `rlf eval --lang <lang> --template <template>` evaluates template with params
  3. `rlf coverage --source <file> --lang <langs>` shows translation coverage table
  4. Exit codes are correct (0 success, non-zero failure)
**Plans**: TBD

Plans:
- [ ] 10-01: TBD
- [ ] 10-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10
Note: Phases 6-9 (language transforms) can proceed in parallel after Phase 3 completes.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Types and Parser | 3/3 | Complete | 2026-02-04 |
| 2. Interpreter Engine | 2/2 | Complete | 2026-02-04 |
| 3. Universal Transforms and ICU4X | 2/2 | Complete | 2026-02-04 |
| 4. Locale Management and Error Handling | 0/2 | Not started | - |
| 5. Macro Code Generation | 0/3 | Not started | - |
| 6. English and Germanic Transforms | 0/1 | Not started | - |
| 7. Romance Language Transforms | 0/2 | Not started | - |
| 8. Greek, Romanian, and Middle Eastern Transforms | 0/1 | Not started | - |
| 9. Asian Language Transforms | 0/2 | Not started | - |
| 10. CLI Tools | 0/2 | Not started | - |

---
*Roadmap created: 2026-02-04*
*Last updated: 2026-02-04 after Phase 3 execution*
