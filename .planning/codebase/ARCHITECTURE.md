# Architecture

**Analysis Date:** 2026-02-04

## Pattern Overview

**Overall:** Macro-driven interpreter architecture with compile-time validation and runtime evaluation.

**Key Characteristics:**
- Unified interpreter evaluates all languages (source and translations) at runtime
- Compile-time validation of source language via procedural macro
- Embedded source phrases as data constant
- Load-time validation of translation files
- Two-layer error model: compile-time panics for static phrases, `Result` for data-driven content

## Layers

**Macro Layer (Compile-Time):**
- Purpose: Parse source language definitions, validate syntax and references, generate Rust functions
- Location: `rlf!` procedural macro (external crate, integrated at compile-time)
- Contains: Phrase definitions with parameters, variants, metadata tags, transforms
- Depends on: Rust proc-macro API
- Used by: Application source files (e.g., `src/localization/strings.rlf.rs`)

**Code Generation Layer:**
- Purpose: Produce callable Rust functions and embed source data
- Output from macro: Generated `strings.rs` module with functions, constants, and registration code
- Contains: Generated phrase functions, `PhraseId` constants, `SOURCE_PHRASES` constant, registration function
- Depends on: Compiler to execute macro
- Used by: Rust application code importing the generated module

**Interpreter Layer (Runtime):**
- Purpose: Parse and evaluate phrase templates, load translations, manage phrase registries
- Location: `RlfInterpreter` (external runtime crate)
- Contains: Parser, Evaluator, PhraseRegistry, TransformRegistry
- Depends on: `icu_plurals` for CLDR plural rules, `icu_locale_core` for locale handling
- Used by: Generated phrase functions, data-driven template evaluation, translation file loading

**Locale Management Layer:**
- Purpose: Manage language selection and translation loading
- Location: `Locale` type (external runtime crate)
- Contains: Current language setting, interpreter reference, translation file paths
- Depends on: Interpreter
- Used by: Application code, passed as parameter to all phrase functions

## Data Flow

**Startup Initialization:**
1. Application creates `Locale::new()`
2. Calls `strings::register_source_phrases(locale.interpreter_mut())`
3. Macro-embedded `SOURCE_PHRASES` constant is loaded into interpreter's English registry
4. Application loads translation files via `locale.load_translations("ru", "path/to/ru.rlf")`
5. Interpreter parses `.rlf` file, validates phrases, stores in language registry

**Phrase Evaluation (Static):**
1. Application code calls `strings::draw(&locale, 3)`
2. Generated function delegates to `locale.interpreter().call_phrase("en", "draw", &[3.into()])`
3. Interpreter looks up "draw" in current language registry
4. Parser converts template "Draw {n} {card:n}." to AST (cached after first parse)
5. Evaluator traverses AST:
   - Resolves `{n}` parameter → gets value 3
   - Resolves `{card:n}` → looks up "card" phrase, applies selector `:n` → maps 3 to CLDR plural "other"
   - Returns phrase with default text and variants
6. Generated function calls `.expect()` on Result, panics if error

**Phrase Evaluation (Data-Driven):**
1. Application calls `locale.interpreter().eval_str(template, lang, params)`
2. Returns `Result`, allowing graceful error handling
3. Parser converts template string to AST
4. Evaluator processes with provided parameters
5. Result returned or error propagated to caller

**Transform Application:**
1. During evaluation, when `@transform:context phrase:selector` is encountered
2. Evaluator resolves phrase to value
3. Applies selectors (if any) from right-to-left
4. Applies transforms right-to-left (innermost first)
5. Returns transformed text

**Metadata Inheritance (`:from`):**
1. Phrase marked with `:from(param)` is evaluated
2. Parameter value is a `Phrase` with tags and variants
3. Evaluator reads source phrase's tags and variants
4. Template evaluated once per source variant, substituting `{param}` with variant text
5. Result is new `Phrase` with inherited tags and computed variants

**Selection Resolution:**
1. Parameter-based selection: `{card:n}` where n=3
   - Interpreter maps 3 → CLDR plural category "other"
   - Looks up phrase variant with key "other"
2. Literal selection: `{card:other}`
   - Uses "other" directly as variant key
3. Tag-based selection: `{destroyed:target}` where target is `Phrase` with `:fem` tag
   - Reads first tag from target phrase ("fem")
   - Looks up variant with key "fem"
4. Fallback resolution: tries exact key, then progressively shorter keys (e.g., "nom.many" → "nom")

**State Management:**
- `Locale` object holds current language and interpreter reference
- `RlfInterpreter` holds phrase registries per language
- Each language's registry is a HashMap of phrase name → phrase definition
- ASTs are cached; parsed once per phrase per interpreter instance
- CLDR plural rules are cached per language via `icu_plurals`

## Key Abstractions

**Phrase:**
- Purpose: Represents localized text with variants and metadata
- Location: Runtime type in RLF crate (`src/lib.rs` or equivalent)
- Pattern: Struct with fields: `text: String`, `variants: HashMap<String, String>`, `tags: Vec<String>`
- Behavior: Displays as default text; `variant()` method accesses specific forms with fallback resolution

**PhraseId:**
- Purpose: Serializable, Copy-able reference to any phrase for storage in game data
- Pattern: FNV-1a hash of phrase name, 8 bytes
- Generation: Generated by macro as constants in `phrase_ids` submodule
- Usage: Stored in data structures, resolved at runtime via `resolve()` or `call()` methods

**Value:**
- Purpose: Runtime parameter type that accepts numbers, strings, or phrases
- Pattern: Enum with variants `Number(i64)`, `Float(f64)`, `String(String)`, `Phrase(Phrase)`
- Behavior: Auto-converts from common types via `Into<Value>`; supports tag/variant queries

**Template/AST:**
- Purpose: Parsed representation of phrase definition or template string
- Pattern: Recursive AST with Segment, Interpolation, Transform, Reference, Selector nodes
- Behavior: Cached after first parse; re-used for all languages evaluating same template

**Phrase Registry:**
- Purpose: Lookup table of phrase definitions per language
- Pattern: HashMap from name → phrase definition (or parsed AST + variants + tags)
- Behavior: O(1) lookup; populated at startup and on load_translations

**Transform Registry:**
- Purpose: Lookup table of transform implementations
- Pattern: Map from transform name → function taking Value and returning String
- Behavior: Built-in transforms (always available); language-specific transforms registered on load

## Entry Points

**Application Startup:**
- Location: Application main/setup code
- Triggers: Application initialization
- Responsibilities: Create Locale, register source phrases, load translation files

**Generated Phrase Functions:**
- Location: Generated `strings` module (or custom name)
- Triggers: Called from application code like `strings::draw(&locale, 3)`
- Responsibilities: Delegate to interpreter, panic on errors (for static content)

**Interpreter Direct API:**
- Location: Anywhere accessing `locale.interpreter()`
- Triggers: Data-driven template evaluation
- Responsibilities: Parse template, evaluate with parameters, return Result

**Translation File Loading:**
- Location: Application setup or development workflow
- Triggers: `locale.load_translations(lang, path)` call
- Responsibilities: Read file, parse phrases, validate, register in interpreter

**Command-Line Tools:**
- Locations: `rlf check`, `rlf eval`, `rlf coverage` commands
- Triggers: Developer invokes tools
- Responsibilities: Syntax validation, template testing, translation coverage reporting

## Error Handling

**Strategy:** Two-layer model separating static from dynamic content

**Compile-Time (Macro Phase):**
- Unknown phrase references → compiler error with suggestion
- Unknown parameters → compiler error with context
- Invalid literal selectors → compiler error showing available variants
- Unknown transforms → compiler error
- Transform tag mismatches (literal phrases) → compiler error
- Cyclic references → compiler error
- Parameter shadowing → compiler error

**Load-Time (Translation Files):**
- Syntax errors → `LoadError` with line/column information
- Unknown phrases → warning (not error; gracefully loaded)
- Parameter count mismatches → warning

**Runtime (Evaluation):**
- Phrase not found → `EvalError::PhraseNotFound`
- Variant missing → `EvalError::MissingVariant`
- Transform requires missing tag → `EvalError::MissingTag`
- Argument count mismatch → `EvalError::ArgumentCount`
- Cyclic reference detected → `EvalError::CyclicReference`

**No Language Fallback:** If phrase exists in English but not Russian, requesting Russian version returns `PhraseNotFound`—does not fall back to English. Translations must be complete.

## Cross-Cutting Concerns

**Logging:** Not part of RLF core; application responsible for logging phrase lookups, loads, errors if needed

**Validation:** Multi-stage approach
- Compile-time: Macro validates source language syntax and references
- Load-time: Interpreter validates translation file syntax, warns on issues
- Runtime: Evaluator catches undefined phrases, missing variants, tag mismatches during evaluation

**Authentication/Authorization:** Not part of RLF; application responsible for controlling which translations are accessible

**Pluralization:** Integrated via `icu_plurals` crate for CLDR-compliant plural rules; maps numbers to plural categories (`zero`, `one`, `two`, `few`, `many`, `other`)

**Localization:** Fully integrated; all languages use same interpreter, same evaluation code path; differences handled via registry lookup and language-specific transforms

**Performance:** Acceptable because localized text is rarely on critical path; interpreter uses caching (ASTs, plural rules); HashMap-based O(1) lookup for phrase names

---

*Architecture analysis: 2026-02-04*
