# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-04)

**Core value:** When you add a phrase to `strings.rlf.rs`, it immediately appears in IDE autocomplete
**Current focus:** Phase 8 - Greek, Romanian, and Middle Eastern Transforms (IN PROGRESS)

## Current Position

Phase: 8 of 10 (Greek, Romanian, and Middle Eastern Transforms)
Plan: 1 of 2 in current phase (COMPLETE)
Status: In progress
Last activity: 2026-02-05 - Completed 08-01-PLAN.md

Progress: [###################] 63% (19/30 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 19
- Average duration: 4.6 min
- Total execution time: 1.48 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-core-types-and-parser | 3 | 14 min | 5 min |
| 02-interpreter-engine | 2 | 8 min | 4 min |
| 03-universal-transforms-and-icu4x | 2 | 6 min | 3 min |
| 04-locale-management-and-error-handling | 2 | 10 min | 5 min |
| 05-macro-code-generation | 4 | 20 min | 5 min |
| 06-english-and-germanic-transforms | 3 | 14 min | 5 min |
| 07-romance-language-transforms | 2 | 13 min | 7 min |
| 08-greek-romanian-and-middle-eastern-transforms | 1 | 6 min | 6 min |

**Recent Trend:**
- Last 5 plans: 5 min, 4 min, 5 min, 8 min, 6 min
- Trend: Stable

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Used const-fnv1a-hash crate for PhraseId (const fn support verified)
- Newtype pattern established: wrap String, impl Deref to str, From impls, Display
- Builder pattern established: use bon::Builder derive with #[builder(default)]
- Reference::Identifier unifies parameters and phrases at parse time (resolved during interpretation)
- Auto-capitalization adds @cap transform, doesn't modify reference name pattern
- Selector::Identifier defers literal vs parameter distinction to interpretation
- Variant keys use Vec<String> for multi-key support (nom, acc: "shared")
- PhraseBody enum distinguishes Simple(Template) from Variants
- ICU4X v2 API uses PluralRulesOptions instead of direct PluralRuleType
- 24 languages supported for CLDR plural rules with English fallback
- Hash collision detection in PhraseRegistry prevents silent overwrites
- No scope inheritance: child phrase contexts don't see parent parameters
- Selector syntax uses chained colons (:nom:one), variant keys use dots (nom.one)
- :from modifier inherits both tags and variants from source phrase
- Static dispatch via TransformKind enum, no trait objects or function pointers
- ICU4X CaseMapper for locale-sensitive case mapping (Turkish dotted-I)
- unicode-segmentation graphemes(true) for proper first-character handling
- Public API encapsulates TransformRegistry creation (callers don't manage it)
- Max edit distance for suggestions: 1 for short keys (<=3 chars), 2 for longer keys
- Limit suggestions to 3, sorted by distance
- Locale owns TransformRegistry (not borrowed)
- Per-language registries use HashMap<String, PhraseRegistry>
- Loading same language replaces all phrases (not merge)
- Fallback only tried on PhraseNotFound errors
- Macro AST types separate from runtime parser AST (need proc_macro2::Span)
- Template string parsing is manual (interpolations inside LitStr, not token trees)
- ValidationContext built upfront with phrase index, variants, and tags
- DFS three-color algorithm for cycle detection
- Generated code uses ::rlf::* fully qualified paths for hygiene
- Generated functions use expect() for errors (programming errors)
- PhraseId constants use SCREAMING_CASE
- Deterministic cycle detection via sorted key/ref iteration for stable trybuild tests
- apply_selectors returns Value (not String) to preserve Phrase tags for transforms
- apply_transforms accepts Value, first transform sees original Phrase with tags
- Selector application strips tags (variant lookup returns String)
- @an alias resolves to @a in TransformRegistry
- Context selector resolved to Value via param lookup or literal string
- German gender tags: :masc, :fem, :neut
- German case context: nom (default), acc, dat, gen as literal strings
- @die/@das resolve to @der; @eine resolves to @ein in registry
- Dutch uses :de/:het tag names matching article names (not :masc/:fem/:neut)
- Dutch @een is invariant - no gender check needed
- @het alias resolves to @de in registry
- Shared RomanceGender and RomancePlural types across Spanish/Portuguese
- Portuguese @um ignores plural context (no plural indefinite forms)
- Spanish @la->@el, @una->@un aliases; Portuguese @a->@o, @uma->@um aliases
- French @un has no plural forms (per APPENDIX_STDLIB)
- ItalianSound enum (Normal, Vowel, SImpura) for three-way article distinction
- @liaison outputs only selected variant, not context
- Spanish @una alias made language-specific to avoid shadowing Italian
- Greek reuses RomancePlural type for singular/plural distinction
- Romanian suffix is simple append without morphological merging
- Greek dative case included for completeness though archaic in modern Greek
- Greek @i/@to -> @o, @mia/@ena -> @enas aliases in registry

### Pending Todos

None.

### Blockers/Concerns

None.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 001 | Move tests to separate crate with public API testing | 2026-02-04 | 8751eb4 | [001-move-tests-to-separate-crate-with-public](./quick/001-move-tests-to-separate-crate-with-public/) |

## Phase 1 Completion Summary

Phase 1 (Core Types and Parser) is now complete with:
- **01-01:** Core types (Phrase, Value, PhraseId, VariantKey, Tag)
- **01-02:** Template string parser (winnow-based, full interpolation support)
- **01-03:** File format parser (parse_file, phrase definitions, variants)

## Phase 2 Completion Summary

Phase 2 (Interpreter Engine) is now complete with:
- **02-01:** Interpreter Foundation
  - EvalError enum with 7 variants
  - PhraseRegistry for phrase storage/lookup
  - EvalContext for evaluation state
  - plural_category function with ICU4X
  - TransformRegistry stub for Phase 3

- **02-02:** Evaluation Logic
  - eval_template for Template AST processing
  - resolve_reference for parameter/phrase lookup
  - apply_selectors with compound key building
  - variant_lookup with progressive fallback
  - Public API: eval_str, call_phrase, get_phrase
  - PhraseId resolution: resolve_with_registry, call_with_registry

## Phase 3 Completion Summary

Phase 3 (Universal Transforms and ICU4X) is now complete with:
- **03-01:** Universal Case Transforms
  - TransformKind enum with Cap/Upper/Lower variants
  - ICU4X CaseMapper for locale-sensitive case mapping
  - unicode-segmentation for grapheme-aware @cap
  - UnknownTransform error variant

- **03-02:** Transform Execution
  - apply_transforms function with right-to-left execution
  - TransformRegistry parameter threading through evaluator
  - Public API encapsulation of TransformRegistry
  - 30 comprehensive transform tests

## Phase 4 Completion Summary

Phase 4 (Locale Management and Error Handling) is now complete with:
- **04-01:** Error Types and Suggestions
  - LoadError enum with Io, Parse, NoPathForReload variants
  - compute_suggestions function with Levenshtein distance
  - MissingVariant enhanced with did-you-mean suggestions
  - 7 new error type tests

- **04-02:** Locale API Implementation
  - Locale struct with builder pattern
  - Per-language phrase storage (HashMap<String, PhraseRegistry>)
  - Owned TransformRegistry for shared transforms
  - Translation loading from file and string
  - Hot-reload support via reload_translations()
  - Fallback language support
  - 27 new locale integration tests

## Phase 5 Completion Summary

Phase 5 (Macro Code Generation) is now complete with:
- **05-01:** TokenStream Parsing Foundation
  - Created rlf-macros proc-macro crate
  - syn 2.0, quote 1.0, proc-macro2 1.0 dependencies
  - MacroInput, PhraseDefinition, Template AST types with spans
  - Parse trait implementations for all AST types
  - Template string parsing with interpolation extraction

- **05-02:** Compile-time Validation
  - ValidationContext with phrase index, variants, tags
  - 7 validation check types implemented
  - Spanned errors with source location
  - Typo suggestions using Levenshtein distance
  - DFS cycle detection with full chain reporting

- **05-03:** Code Generation
  - Phrase function generation (parameterless and parameterized)
  - SOURCE_PHRASES const with embedded RLF source
  - register_source_phrases() for loading phrases
  - phrase_ids module with SCREAMING_CASE constants
  - Fully qualified paths for macro hygiene

- **05-04:** Macro Integration
  - Refined expand() helper for clean error handling
  - rlf crate re-exports rlf! macro
  - trybuild tests for compile-time error verification
  - Deterministic cycle detection for stable tests

## Phase 6 Completion Summary

Phase 6 (English and Germanic Transforms) is now complete with:
- **06-01:** English Article Transforms
  - EnglishA transform reading :a/:an tags
  - EnglishThe transform prepending "the"
  - @an alias resolving to @a
  - Value-based transform execution preserves tags
  - 19 new English transform tests

- **06-02:** German Article Transforms
  - Context resolution in evaluator for case parameters
  - GermanDer transform with 12 definite article forms (4 cases x 3 genders)
  - GermanEin transform with 12 indefinite article forms
  - @die/@das/@eine aliases
  - 15 German tests (8 unit + 7 integration)

- **06-03:** Dutch Article Transforms
  - DutchDe transform reading :de/:het tags
  - DutchEen transform prepending invariant "een"
  - @het alias resolving to @de
  - 19 Dutch tests (9 unit + 6 integration + 4 cross-language)

## Phase 7 Completion Summary

Phase 7 (Romance Language Transforms) is now complete with:
- **07-01:** Spanish and Portuguese Transforms
  - SpanishEl/SpanishUn transforms with :masc/:fem tags and plural context
  - PortugueseO/PortugueseUm article transforms
  - PortugueseDe/PortugueseEm preposition contractions (do/da/no/na)
  - RomanceGender and RomancePlural shared types
  - 28 new tests (10 Spanish + 12 Portuguese + 6 integration)

- **07-02:** French and Italian Transforms
  - FrenchLe/FrenchUn article transforms with elision (l' before vowels)
  - FrenchDe/FrenchAu contraction transforms
  - FrenchLiaison for prevocalic form selection (beau/bel, ce/cet)
  - ItalianIl/ItalianUn article transforms with three-way sound distinction
  - ItalianDi/ItalianA contraction transforms
  - ItalianSound enum (Normal, Vowel, SImpura)
  - 59 new tests (21 French + 24 Italian + 14 integration)

## Phase 8 Progress

Phase 8 (Greek, Romanian, and Middle Eastern Transforms) in progress:
- **08-01:** Greek and Romanian Article Transforms (COMPLETE)
  - GreekO transform with 24 definite article forms (4 cases x 3 genders x 2 numbers)
  - GreekEnas transform with 12 indefinite article forms (4 cases x 3 genders)
  - RomanianDef transform with postposed suffix appending
  - Greek @i/@to -> @o, @mia/@ena -> @enas aliases
  - 38 new tests (29 Greek + 9 Romanian)

All 207 transform tests passing (369 total tests):
- 33 file parser integration tests
- 46 template parser integration tests
- 10 interpreter foundation tests
- 27 interpreter evaluation tests
- 207 interpreter transform tests
- 7 interpreter error tests
- 25 locale integration tests
- 4 trybuild compile tests (1 pass, 3 fail)
- 14 doctests

## Session Continuity

Last session: 2026-02-05T05:34:00Z
Stopped at: Completed 08-01-PLAN.md
Resume file: None
