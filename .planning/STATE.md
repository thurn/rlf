# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-04)

**Core value:** When you add a phrase to `strings.rlf.rs`, it immediately appears in IDE autocomplete
**Current focus:** Phase 10 - CLI Tools (In progress)

## Current Position

Phase: 10 of 10 (CLI Tools)
Plan: 2 of 3 in current phase
Status: In progress
Last activity: 2026-02-05 - Completed 10-02-PLAN.md

Progress: [###########################] 83% (25/30 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 25
- Average duration: 4.4 min
- Total execution time: 1.95 hours

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
| 08-greek-romanian-and-middle-eastern-transforms | 2 | 12 min | 6 min |
| 09-asian-language-transforms | 3 | 15 min | 5 min |
| 10-cli-tools | 2 | 7 min | 4 min |

**Recent Trend:**
- Last 5 plans: 3 min, 8 min, 4 min, 5 min, 2 min
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
- Arabic @al uses :sun/:moon tags (no automatic first-letter detection)
- Arabic shadda placed AFTER consonant per Unicode standard
- Persian @ezafe uses ZWNJ before ye for proper rendering
- Persian kasra (U+0650) for consonant-final, ye (U+06CC) for vowel-final
- CJK @count format: {count}{classifier}{text} with no spaces
- Classifier lookup via tag-to-character array with find_classifier helper
- context_to_count defaults to 1 when no context provided
- Vietnamese @count format: {count} {classifier} {text} with spaces
- Thai @count format: {count}{classifier}{text} no spaces (like CJK)
- Bengali @count format: {count}{classifier} {text} classifier attached to number
- Indonesian @plural: simple reduplication with hyphen
- Korean @particle uses hangeul::ends_with_jongseong for consonant detection
- Non-Hangul text treated as vowel-ending for Korean particles
- Korean @particle returns only particle string (not prepended to text)
- Turkish @inflect requires :front/:back tags (no auto-detection)
- Turkish uses simplified 2-way harmony for all suffixes (ignoring voicing)
- Suffix chains parsed as dot-separated names (pl.dat)
- RlfDiagnostic wraps ParseError with NamedSource for source context display
- Byte offset calculated from line:column by summing line lengths
- Exit code 65 (DATAERR) for syntax errors, 0 for success
- CLI commands in commands/ module with run_X function pattern
- Parameter parsing: parse_key_val helper for name=value format
- Eval command handles errors internally, returns exit code (not propagate miette error)

### Pending Todos

None.

### Blockers/Concerns

None.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 001 | Move tests to separate crate with public API testing | 2026-02-04 | 8751eb4 | [001-move-tests-to-separate-crate-with-public](./quick/001-move-tests-to-separate-crate-with-public/) |
| 002 | Improve rlf-macro test coverage by 10x (64 unit + 14 trybuild) | 2026-02-05 | a6e761f | [002-improve-rlf-macro-test-coverage-by-10x](./quick/002-improve-rlf-macro-test-coverage-by-10x/) |

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

## Phase 8 Completion Summary

Phase 8 (Greek, Romanian, and Middle Eastern Transforms) is now complete with:
- **08-01:** Greek and Romanian Article Transforms
  - GreekO transform with 24 definite article forms (4 cases x 3 genders x 2 numbers)
  - GreekEnas transform with 12 indefinite article forms (4 cases x 3 genders)
  - RomanianDef transform with postposed suffix appending
  - Greek @i/@to -> @o, @mia/@ena -> @enas aliases
  - 38 new tests (29 Greek + 9 Romanian)

- **08-02:** Arabic and Persian Transforms
  - ArabicAl transform with sun/moon letter assimilation using shadda
  - PersianEzafe transform with kasra/ZWNJ connectors
  - Byte-level Unicode verification for RTL text testing
  - 14 new tests (7 Arabic + 7 Persian)

## Phase 9 Completion Summary

Phase 9 (Asian Language Transforms) is now COMPLETE with:
- **09-01:** CJK Count Transforms
  - ChineseCount transform with 7 classifiers (zhang, ge, ming, wei, tiao, ben, zhi)
  - JapaneseCount transform with 6 counters (mai, nin, hiki, hon, ko, satsu)
  - KoreanCount transform with 5 counters (jang, myeong, mari, gae, gwon)
  - hangeul dependency added for Korean @particle in Plan 03
  - find_classifier helper for tag-based lookup
  - 24 new tests (7 Chinese + 6 Japanese + 6 Korean + 5 registry/edge cases)

- **09-02:** SEA Count Transforms
  - VietnameseCount transform with 5 classifiers (cai, con, nguoi, chiec, to)
  - ThaiCount transform with 4 classifiers (bai, tua, khon, an)
  - BengaliCount transform with 4 classifiers (ta, ti, khana, jon)
  - IndonesianPlural transform with simple reduplication
  - 19 new tests (4 Vietnamese + 4 Thai + 4 Bengali + 4 Indonesian + 3 registry)

- **09-03:** Korean Particle and Turkish Inflect Transforms
  - KoreanParticle transform with phonology-based selection (ga/i, reul/eul, neun/eun)
  - TurkishInflect transform with vowel harmony suffix chains (pl, dat, loc, abl)
  - hangeul crate integration for jongseong detection
  - 19 new tests (9 Korean + 10 Turkish)

Total tests: 511 passing

## Phase 10 Progress

Phase 10 (CLI Tools) is in progress with:
- **10-01:** CLI Check Command (COMPLETE)
  - rlf-cli crate with clap derive for git-style subcommands
  - `rlf check` command with miette compiler-quality diagnostics
  - JSON output mode (--json) and strict mode (--strict)
  - Exit codes: 0 (success), 65 (DATAERR for syntax errors)

- **10-02:** CLI Eval Command (COMPLETE)
  - `rlf eval` command for template evaluation
  - --lang, --template, --phrases, -p/--param flags
  - Parameter parsing with numeric detection
  - JSON output mode for machine-readable results

## Session Continuity

Last session: 2026-02-05T14:53:33Z
Stopped at: Completed 10-02-PLAN.md
Resume file: None
