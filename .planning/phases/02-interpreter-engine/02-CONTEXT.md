# Phase 2: Interpreter Engine - Context

**Gathered:** 2026-02-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Evaluation engine that takes parsed templates and produces formatted strings. Resolves phrase calls, applies variant selection based on parameters, and substitutes values. Does not include transforms (Phase 3), locale management (Phase 4), or macro generation (Phase 5).

</domain>

<decisions>
## Implementation Decisions

### Evaluation API Shape
- Return type: Always `Result<Phrase, EvalError>` — per APPENDIX_RUNTIME_INTERPRETER.md, Phrase carries variants/tags needed for downstream selection
- Parameter passing: `HashMap<String, Value>` (params!() macro helper deferred to Phase 5)
- Context required: Full locale context (phrases + current language + transform functions)
- Evaluation timing: Eager — `eval_str()` runs immediately and returns the result

### Variant Resolution
- Missing variant: Error immediately — no silent fallback to 'other'
- Key matching semantics: Follow DESIGN.md specification exactly

### Phrase Call Semantics
- Reference resolution: Both parse-time validation available AND runtime validation
- Missing parameters: Error — all required parameters must be provided
- Extra parameters: Silently ignored

### Recursion & Limits
- Max recursion depth: 64 levels
- Cycle detection approach: Claude's discretion
- Error detail: Simple message ("Maximum recursion depth exceeded"), no call stack
- Additional limits: None — depth limit is sufficient

### Claude's Discretion
- Scope inheritance (whether child phrases see parent parameters)
- Dot-notation key matching semantics (per DESIGN.md)
- Multiple selector combination semantics (per DESIGN.md)
- Cycle detection implementation approach

</decisions>

<specifics>
## Specific Ideas

No specific requirements — follow DESIGN.md and appendices for all behavioral semantics.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-interpreter-engine*
*Context gathered: 2026-02-04*
