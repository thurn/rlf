# Requirements: RLF

**Defined:** 2026-02-04
**Core Value:** When you add a phrase to `strings.rlf.rs`, it immediately appears in IDE autocomplete

## v1 Requirements

Requirements derived from design documents. All features documented in DESIGN.md and appendices.

### Core Language

- [ ] **LANG-01**: Phrase definitions with `name = "text";` syntax
- [ ] **LANG-02**: Parameters with `name(p1, p2) = "{p1} and {p2}";` syntax
- [ ] **LANG-03**: Variants with `name = { key1: "val1", key2: "val2" };` syntax
- [ ] **LANG-04**: Selection with `{phrase:selector}` syntax
- [ ] **LANG-05**: Metadata tags with `:tag` before phrase content
- [ ] **LANG-06**: Transforms with `@transform` prefix in interpolations
- [ ] **LANG-07**: Transform context with `@transform:context` syntax
- [ ] **LANG-08**: Escape sequences `{{`, `}}`, `@@`, `::` for literals
- [ ] **LANG-09**: Multi-dimensional variants with dot notation (`nom.one`, `acc.many`)
- [ ] **LANG-10**: Multi-key shorthand (`nom, acc: "value"`)
- [ ] **LANG-11**: Wildcard fallbacks (partial keys as fallbacks)
- [ ] **LANG-12**: Metadata inheritance with `:from(param)` modifier
- [ ] **LANG-13**: Automatic capitalization (`{Card}` -> `{@cap card}`)
- [ ] **LANG-14**: Phrase calls with arguments `{phrase(arg1, arg2)}`
- [ ] **LANG-15**: Chained selectors `{phrase:sel1:sel2}`
- [ ] **LANG-16**: Chained transforms `{@t1 @t2 phrase}`
- [ ] **LANG-17**: Comments with `//` to end of line

### Macro System

- [ ] **MACRO-01**: Parse phrase definitions from `rlf!` block
- [ ] **MACRO-02**: Generate typed Rust function per phrase
- [ ] **MACRO-03**: Functions accept `impl Into<Value>` parameters
- [ ] **MACRO-04**: Functions return `Phrase` type
- [ ] **MACRO-05**: Embed source phrases as const string for interpreter
- [ ] **MACRO-06**: Generate `register_source_phrases()` function
- [ ] **MACRO-07**: Generate `phrase_ids` module with PhraseId constants
- [ ] **MACRO-08**: Validate undefined phrase references at compile time
- [ ] **MACRO-09**: Validate undefined parameter references at compile time
- [ ] **MACRO-10**: Validate literal selector against defined variants
- [ ] **MACRO-11**: Validate transform names exist
- [ ] **MACRO-12**: Validate transform tags on literal phrase arguments
- [ ] **MACRO-13**: Validate tag-based selection compatibility on literals
- [ ] **MACRO-14**: Detect cyclic references at compile time
- [ ] **MACRO-15**: Reject parameter shadowing phrase names
- [ ] **MACRO-16**: Provide helpful error messages with source spans
- [ ] **MACRO-17**: Suggest similar names for typos (Levenshtein distance)

### Types

- [ ] **TYPE-01**: `Phrase` struct with text, variants HashMap, tags Vec
- [ ] **TYPE-02**: `Phrase::variant(&str)` method with fallback resolution
- [ ] **TYPE-03**: `Phrase` implements `Display` (returns text)
- [ ] **TYPE-04**: `Value` enum with Number, Float, String, Phrase variants
- [ ] **TYPE-05**: `Into<Value>` for common types (i32, i64, f64, String, &str, Phrase)
- [ ] **TYPE-06**: `PhraseId` as 8-byte hash wrapper, Copy, Eq, Hash
- [ ] **TYPE-07**: `PhraseId::from_name()` as const fn using FNV-1a
- [ ] **TYPE-08**: `PhraseId::resolve(&Locale)` for parameterless phrases
- [ ] **TYPE-09**: `PhraseId::call(&Locale, &[Value])` for phrases with params
- [ ] **TYPE-10**: `PhraseId::name()` for debugging (returns Option<&'static str>)
- [ ] **TYPE-11**: `PhraseId` serializable with serde

### Interpreter

- [ ] **INTERP-01**: Parse template strings into AST
- [ ] **INTERP-02**: Parse .rlf files into phrase definitions
- [ ] **INTERP-03**: `eval_str()` to evaluate template with params
- [ ] **INTERP-04**: `call_phrase()` to call phrase by name with args
- [ ] **INTERP-05**: `get_phrase()` to get parameterless phrase as Phrase
- [ ] **INTERP-06**: `call_phrase_by_id()` for PhraseId lookup
- [ ] **INTERP-07**: `get_phrase_by_id()` for PhraseId lookup
- [ ] **INTERP-08**: `load_phrases()` to load phrases from string
- [ ] **INTERP-09**: Phrase registry per language
- [ ] **INTERP-10**: Transform registry with universal and language-specific
- [ ] **INTERP-11**: Variant resolution with fallback (exact -> progressively shorter)
- [ ] **INTERP-12**: Numeric selection via CLDR plural category
- [ ] **INTERP-13**: Tag-based selection (read phrase tag, use as variant key)
- [ ] **INTERP-14**: Transform execution with optional context
- [ ] **INTERP-15**: Metadata inheritance evaluation (`:from` modifier)
- [ ] **INTERP-16**: Cycle detection during evaluation
- [ ] **INTERP-17**: Max depth limit (default 64) for recursion

### Locale

- [ ] **LOC-01**: `Locale` struct managing language selection and interpreter
- [ ] **LOC-02**: `Locale::new()` constructor
- [ ] **LOC-03**: `Locale::with_language(&str)` constructor
- [ ] **LOC-04**: `Locale::set_language(&str)` method
- [ ] **LOC-05**: `Locale::language()` getter
- [ ] **LOC-06**: `Locale::interpreter()` and `interpreter_mut()` accessors
- [ ] **LOC-07**: `Locale::load_translations(lang, path)` for file loading
- [ ] **LOC-08**: `Locale::load_translations_str(lang, content)` for string loading
- [ ] **LOC-09**: `Locale::reload_translations(lang)` for hot-reloading

### Error Handling

- [ ] **ERR-01**: `LoadError` for parse failures with line/column
- [ ] **ERR-02**: `EvalError::PhraseNotFound` with phrase name
- [ ] **ERR-03**: `EvalError::MissingVariant` with phrase, key, available
- [ ] **ERR-04**: `EvalError::MissingTag` with transform, expected, phrase
- [ ] **ERR-05**: `EvalError::ArgumentCount` with phrase, expected, got
- [ ] **ERR-06**: `EvalError::CyclicReference` with chain
- [ ] **ERR-07**: Generated functions panic on error (programming errors)
- [ ] **ERR-08**: Interpreter methods return Result (handle gracefully)
- [ ] **ERR-09**: No language fallback (missing = error, not silent fallback)

### Universal Transforms

- [ ] **XFORM-01**: `@cap` - Capitalize first letter
- [ ] **XFORM-02**: `@upper` - All uppercase
- [ ] **XFORM-03**: `@lower` - All lowercase

### ICU4X Integration

- [ ] **ICU-01**: CLDR plural rules for cardinal numbers
- [ ] **ICU-02**: Plural categories: zero, one, two, few, many, other
- [ ] **ICU-03**: Support all 24 documented languages

### English Transforms

- [ ] **EN-01**: `@a` / `@an` - Indefinite article from `:a`/`:an` tags
- [ ] **EN-02**: `@the` - Definite article

### Spanish Transforms

- [ ] **ES-01**: `@el` / `@la` - Definite article from `:masc`/`:fem` tags
- [ ] **ES-02**: `@un` / `@una` - Indefinite article from `:masc`/`:fem` tags
- [ ] **ES-03**: Context selector for plural forms (`:one`/`:other`)

### French Transforms

- [ ] **FR-01**: `@le` / `@la` - Definite article with elision from `:vowel`
- [ ] **FR-02**: `@un` / `@une` - Indefinite article
- [ ] **FR-03**: `@de` - "de" + article contraction
- [ ] **FR-04**: `@au` - "a" + article contraction
- [ ] **FR-05**: `@liaison` - Prevocalic form selection

### German Transforms

- [ ] **DE-01**: `@der` / `@die` / `@das` - Definite article with case
- [ ] **DE-02**: `@ein` / `@eine` - Indefinite article with case

### Portuguese Transforms

- [ ] **PT-01**: `@o` / `@a` - Definite article
- [ ] **PT-02**: `@um` / `@uma` - Indefinite article
- [ ] **PT-03**: `@de` - "de" + article contraction
- [ ] **PT-04**: `@em` - "em" + article contraction

### Italian Transforms

- [ ] **IT-01**: `@il` / `@lo` / `@la` - Definite article with sound rules
- [ ] **IT-02**: `@un` / `@uno` / `@una` - Indefinite article
- [ ] **IT-03**: `@di` - "di" + article contraction
- [ ] **IT-04**: `@a` - "a" + article contraction

### Dutch Transforms

- [ ] **NL-01**: `@de` / `@het` - Definite article from tags
- [ ] **NL-02**: `@een` - Indefinite article

### Greek Transforms

- [ ] **EL-01**: `@o` / `@i` / `@to` - Definite article with gender/case
- [ ] **EL-02**: `@enas` / `@mia` / `@ena` - Indefinite article

### Romanian Transforms

- [ ] **RO-01**: `@def` - Postposed definite article

### Arabic Transforms

- [ ] **AR-01**: `@al` - Definite article with sun/moon letter assimilation

### Persian Transforms

- [ ] **FA-01**: `@ezafe` - Ezafe connector from `:vowel` tag

### CJK Count Transforms

- [ ] **CJK-01**: Chinese `@count` - Number + measure word from tags
- [ ] **CJK-02**: Japanese `@count` - Number + counter from tags
- [ ] **CJK-03**: Korean `@count` - Number + counter (Korean or Sino-Korean)
- [ ] **CJK-04**: Vietnamese `@count` - Number + classifier
- [ ] **CJK-05**: Thai `@count` - Number + classifier
- [ ] **CJK-06**: Bengali `@count` - Number + classifier

### Korean Transforms

- [ ] **KO-01**: `@particle` - Context-sensitive particle based on final sound

### Turkish Transforms

- [ ] **TR-01**: `@inflect` - Agglutinative suffix chains with vowel harmony

### Indonesian Transforms

- [ ] **ID-01**: `@plural` - Reduplication for plural

### CLI: rlf check

- [ ] **CLI-01**: Parse and validate .rlf file syntax
- [ ] **CLI-02**: Report errors with file, line, column
- [ ] **CLI-03**: Exit 0 on success, non-zero on failure
- [ ] **CLI-04**: `--strict` mode to check against source file

### CLI: rlf eval

- [ ] **CLI-05**: `--lang <lang>` to specify language
- [ ] **CLI-06**: `--template <template>` to evaluate
- [ ] **CLI-07**: `--param <name>=<value>` for parameters (repeatable)
- [ ] **CLI-08**: `--phrases <file>` to load phrase definitions

### CLI: rlf coverage

- [ ] **CLI-09**: `--source <file>` for source language file
- [ ] **CLI-10**: `--lang <langs>` comma-separated language list
- [ ] **CLI-11**: Output table with phrases, translated, missing counts
- [ ] **CLI-12**: List missing phrase names per language

## v2 Requirements

(None - all documented features are v1)

## Out of Scope

| Feature | Reason |
|---------|--------|
| GUI translation editor | Use existing tools (Poedit, Localazy, Weblate) |
| Automatic LLM translation | Humans translate, RLF manages |
| Build-time codegen from external files | Macro-only approach for IDE support |
| Language fallback | Missing translations are errors, caught in CI |
| Custom user transforms | Built-in transforms only for predictability |
| ICU MessageFormat compatibility | Different design philosophy |
| gettext compatibility | Different design philosophy |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| LANG-01 | Phase 1 | Complete |
| LANG-02 | Phase 1 | Complete |
| LANG-03 | Phase 1 | Complete |
| LANG-04 | Phase 1 | Complete |
| LANG-05 | Phase 1 | Complete |
| LANG-06 | Phase 1 | Complete |
| LANG-07 | Phase 1 | Complete |
| LANG-08 | Phase 1 | Complete |
| LANG-09 | Phase 1 | Complete |
| LANG-10 | Phase 1 | Complete |
| LANG-11 | Phase 1 | Complete |
| LANG-12 | Phase 1 | Complete |
| LANG-13 | Phase 1 | Complete |
| LANG-14 | Phase 1 | Complete |
| LANG-15 | Phase 1 | Complete |
| LANG-16 | Phase 1 | Complete |
| LANG-17 | Phase 1 | Complete |
| MACRO-01 | Phase 5 | Complete |
| MACRO-02 | Phase 5 | Complete |
| MACRO-03 | Phase 5 | Complete |
| MACRO-04 | Phase 5 | Complete |
| MACRO-05 | Phase 5 | Complete |
| MACRO-06 | Phase 5 | Complete |
| MACRO-07 | Phase 5 | Complete |
| MACRO-08 | Phase 5 | Complete |
| MACRO-09 | Phase 5 | Complete |
| MACRO-10 | Phase 5 | Complete |
| MACRO-11 | Phase 5 | Complete |
| MACRO-12 | Phase 5 | Complete |
| MACRO-13 | Phase 5 | Complete |
| MACRO-14 | Phase 5 | Complete |
| MACRO-15 | Phase 5 | Complete |
| MACRO-16 | Phase 5 | Complete |
| MACRO-17 | Phase 5 | Complete |
| TYPE-01 | Phase 1 | Complete |
| TYPE-02 | Phase 1 | Complete |
| TYPE-03 | Phase 1 | Complete |
| TYPE-04 | Phase 1 | Complete |
| TYPE-05 | Phase 1 | Complete |
| TYPE-06 | Phase 1 | Complete |
| TYPE-07 | Phase 1 | Complete |
| TYPE-08 | Phase 1 | Pending |
| TYPE-09 | Phase 1 | Pending |
| TYPE-10 | Phase 1 | Pending |
| TYPE-11 | Phase 1 | Complete |
| INTERP-01 | Phase 1 | Complete |
| INTERP-02 | Phase 1 | Complete |
| INTERP-03 | Phase 2 | Complete |
| INTERP-04 | Phase 2 | Complete |
| INTERP-05 | Phase 2 | Complete |
| INTERP-06 | Phase 2 | Complete |
| INTERP-07 | Phase 2 | Complete |
| INTERP-08 | Phase 2 | Complete |
| INTERP-09 | Phase 2 | Complete |
| INTERP-10 | Phase 2 | Complete |
| INTERP-11 | Phase 2 | Complete |
| INTERP-12 | Phase 2 | Complete |
| INTERP-13 | Phase 2 | Complete |
| INTERP-14 | Phase 2 | Complete |
| INTERP-15 | Phase 2 | Complete |
| INTERP-16 | Phase 2 | Complete |
| INTERP-17 | Phase 2 | Complete |
| LOC-01 | Phase 4 | Complete |
| LOC-02 | Phase 4 | Complete |
| LOC-03 | Phase 4 | Complete |
| LOC-04 | Phase 4 | Complete |
| LOC-05 | Phase 4 | Complete |
| LOC-06 | Phase 4 | Complete |
| LOC-07 | Phase 4 | Complete |
| LOC-08 | Phase 4 | Complete |
| LOC-09 | Phase 4 | Complete |
| ERR-01 | Phase 4 | Complete |
| ERR-02 | Phase 4 | Complete |
| ERR-03 | Phase 4 | Complete |
| ERR-04 | Phase 4 | Complete |
| ERR-05 | Phase 4 | Complete |
| ERR-06 | Phase 4 | Complete |
| ERR-07 | Phase 4 | Complete |
| ERR-08 | Phase 4 | Complete |
| ERR-09 | Phase 4 | Complete |
| XFORM-01 | Phase 3 | Complete |
| XFORM-02 | Phase 3 | Complete |
| XFORM-03 | Phase 3 | Complete |
| ICU-01 | Phase 3 | Complete |
| ICU-02 | Phase 3 | Complete |
| ICU-03 | Phase 3 | Complete |
| EN-01 | Phase 6 | Complete |
| EN-02 | Phase 6 | Complete |
| ES-01 | Phase 7 | Pending |
| ES-02 | Phase 7 | Pending |
| ES-03 | Phase 7 | Pending |
| FR-01 | Phase 7 | Pending |
| FR-02 | Phase 7 | Pending |
| FR-03 | Phase 7 | Pending |
| FR-04 | Phase 7 | Pending |
| FR-05 | Phase 7 | Pending |
| DE-01 | Phase 6 | Complete |
| DE-02 | Phase 6 | Complete |
| PT-01 | Phase 7 | Pending |
| PT-02 | Phase 7 | Pending |
| PT-03 | Phase 7 | Pending |
| PT-04 | Phase 7 | Pending |
| IT-01 | Phase 7 | Pending |
| IT-02 | Phase 7 | Pending |
| IT-03 | Phase 7 | Pending |
| IT-04 | Phase 7 | Pending |
| NL-01 | Phase 6 | Complete |
| NL-02 | Phase 6 | Complete |
| EL-01 | Phase 8 | Pending |
| EL-02 | Phase 8 | Pending |
| RO-01 | Phase 8 | Pending |
| AR-01 | Phase 8 | Pending |
| FA-01 | Phase 8 | Pending |
| CJK-01 | Phase 9 | Pending |
| CJK-02 | Phase 9 | Pending |
| CJK-03 | Phase 9 | Pending |
| CJK-04 | Phase 9 | Pending |
| CJK-05 | Phase 9 | Pending |
| CJK-06 | Phase 9 | Pending |
| KO-01 | Phase 9 | Pending |
| TR-01 | Phase 9 | Pending |
| ID-01 | Phase 9 | Pending |
| CLI-01 | Phase 10 | Pending |
| CLI-02 | Phase 10 | Pending |
| CLI-03 | Phase 10 | Pending |
| CLI-04 | Phase 10 | Pending |
| CLI-05 | Phase 10 | Pending |
| CLI-06 | Phase 10 | Pending |
| CLI-07 | Phase 10 | Pending |
| CLI-08 | Phase 10 | Pending |
| CLI-09 | Phase 10 | Pending |
| CLI-10 | Phase 10 | Pending |
| CLI-11 | Phase 10 | Pending |
| CLI-12 | Phase 10 | Pending |

**Coverage:**
- v1 requirements: 111 total
- Mapped to phases: 111
- Unmapped: 0

---
*Requirements defined: 2026-02-04*
*Last updated: 2026-02-05 after Phase 6 completion*
