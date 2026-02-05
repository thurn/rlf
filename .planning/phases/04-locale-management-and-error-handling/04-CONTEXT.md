# Phase 4: Locale Management and Error Handling - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement the Locale struct that users interact with to manage language selection, load translations, and get clear error messages. This is the user-facing API for managing localized content — language state management, translation loading/reloading, and comprehensive error types.

</domain>

<decisions>
## Implementation Decisions

### Locale API Shape
- Builder pattern for construction: `Locale::builder().language("en").build()`
- Locale owns RlfInterpreter (self-contained, not borrowed)
- Mutable language change: `locale.set_language("ru")` via `&mut self`
- Expose `interpreter()` accessor returning `&RlfInterpreter` for direct access
- Also expose `interpreter_mut()` for loading phrases

### Translation Loading
- Both file path and string content methods:
  - `load_translations(language, path)` — reads from file
  - `load_translations_str(language, content)` — loads from string
- Loading same language twice **replaces** previous phrases (not merge)
- `reload_translations(language)` remembers original path and re-reads
- Calling `reload_translations()` on string-loaded content returns error

### Error Message Design
- LoadError includes original file path: `LoadError { path: PathBuf, line: usize, column: usize, message: String }`
- EvalError::MissingVariant includes available keys AND "did you mean" suggestions
- All errors implement `std::error::Error`
- Display format: Claude's discretion based on error type (single vs multi-line)

### Language Fallback Policy
- Fallback is configurable, not hardcoded
- Default: **no fallback** — missing phrase returns error
- Users opt-in via builder: `.fallback_language("en")`
- Single fallback step only (not a chain)
- API: `Locale::builder().language("ru").fallback_language("en").build()`

### Claude's Discretion
- Exact Display formatting for errors (single-line vs multi-line as appropriate)
- Internal data structure for storing path → language mappings
- Whether to cache parsed templates on load vs parse on demand

</decisions>

<specifics>
## Specific Ideas

- Design matches DESIGN.md and APPENDIX_RUNTIME_INTERPRETER.md specifications
- Generated functions use `.expect()` for static phrases (programming errors)
- Interpreter methods return `Result` for data-driven content (graceful handling)
- No silent fallback by default — forces complete translations during development

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-locale-management-and-error-handling*
*Context gathered: 2026-02-04*
