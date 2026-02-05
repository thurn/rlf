# Phase 7: Romance Language Transforms - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Article and contraction transforms for Spanish, French, Portuguese, and Italian. Implements `@el`, `@un`, `@le`, `@de`, `@au`, `@o`, `@um`, `@em`, `@il`, `@di`, `@a` as specified in APPENDIX_STDLIB.md. Does NOT include other Romance language features like adjective agreement or complex verb forms.

</domain>

<decisions>
## Implementation Decisions

### Gender/Number Tag Naming
- Use `:masc`/`:fem`/`:neut` consistently with Germanic languages (Phase 6)
- Invalid gender tag for language (e.g., `:neut` for Spanish) produces MissingTag error
- Missing required gender tag produces error, no silent defaults
- Plural context via context parameter (`:one`/`:other`) per APPENDIX_STDLIB spec

### Contraction Behavior
- Capitalization handled via separate `@cap` transform (contractions always lowercase)
- Portuguese: distinct transform names only (@de, @em), no aliases like @do/@no
- Italian: sound tags (:s_imp, :vowel) required when applicable — error if ambiguous
- Spanish: no dedicated contraction transforms (handled via `"de {@el x}"` patterns per APPENDIX_STDLIB)

### Elision Rules (French/Italian)
- French elision requires `:vowel` tag on phrase (no automatic vowel detection)
- Italian `:s_imp` words must have explicit tag — missing tag produces error
- ASCII apostrophe (`'`) for elision, not Unicode right quote
- French and Italian share `:vowel` tag name (same semantics)

### Cross-Language Consistency
- Indefinite article transforms follow APPENDIX_STDLIB: @un (ES/FR/IT), @um (PT)
- Feminine aliases added: @la→@el (ES), @la→@le (FR), @la→@il (IT), @a→@o (PT)
- Two plans: Plan 1 for Spanish+Portuguese (simpler), Plan 2 for French+Italian (elision/contractions)

### Claude's Discretion
- Exact error message wording for MissingTag errors
- Internal implementation of article lookup tables
- Test organization within each plan

</decisions>

<specifics>
## Specific Ideas

- Follow APPENDIX_STDLIB.md exactly for transform names and behavior
- Pattern established by Phase 6: aliases resolve to canonical transform in registry
- Context resolution for plural forms follows German case pattern (@el:other → los/las)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-romance-language-transforms*
*Context gathered: 2026-02-04*
