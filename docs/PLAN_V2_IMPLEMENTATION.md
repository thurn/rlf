# RLF v2 Implementation Plan

## Overview

This document describes the high-level milestones for implementing the v2
syntax design specified in `DESIGN_V2.md`. The key v2 changes:

1. **`$` prefix** — parameters are always `$name`, bare names are always
   terms/phrases
2. **Term vs phrase** — terms have variants and `:`, phrases have parameters
   and `()`
3. **`:match` keyword** — phrases branch on parameter values via `:match($p)`
   with a required `*` default
4. **`*` default marker** — variant blocks mark one key with `*` as the default
5. **Numeric keys in `:match`** — exact-number matching before CLDR categories
6. **Dynamic transform context `()`** — `{@count($n) card}` not
   `{@count:$n card}`
7. **Literal arguments** — `{phrase(42)}` and `{phrase("text")}` in phrase calls

### Breaking changes

All milestones except Milestone 5 (numeric keys) are breaking changes to the
RLF syntax. The downstream consumer (dreamtides) must be migrated in lockstep.
Each milestone should be a self-contained commit (or branch) that leaves all
tests passing.

### Architecture reminder

Two parsers must be updated in parallel for every syntax change:

- **Macro parser** (`crates/rlf-macros/src/parse.rs`) — compile-time, uses
  `syn`, produces `input.rs` AST with `Span` info
- **Runtime parser** (`crates/rlf/src/parser/template.rs` + `file.rs`) —
  runtime, uses `winnow`, produces `ast.rs` AST

The evaluator (`crates/rlf/src/interpreter/evaluator.rs`) must be updated when
AST types change. Code generation (`crates/rlf-macros/src/codegen.rs`) and
validation (`crates/rlf-macros/src/validate.rs`) are macro-only.

---

## Milestone 1: `$` Parameter Prefix

**Goal:** All parameter references use `$` everywhere — declarations, template
bodies, selectors, `:from`, phrase call arguments. Bare names always refer to
terms or phrases.

**Why first:** This is the most foundational change. Every subsequent milestone
assumes `$` is in place. With `$`, selector disambiguation is handled by the
prefix — `{card:$n}` vs `{card:other}` is unambiguous from syntax alone.

### Changes

**AST types** (`ast.rs` and `input.rs`):
- `Reference` enum gains `Parameter(String)` variant (for `{$n}`)
- `Selector` enum gains `Parameter(String)` variant (for `:$n`)
- Phrase call `Argument` type distinguishes `Parameter` from `TermRef`
- Parameter declarations store names without the `$` prefix internally

**Both parsers:**
- Parameter declarations require `$`: `draw($n)` not `draw(n)`
- `:from($s)` requires `$`
- In template bodies: `$name` → parameter, bare `name` → term/phrase reference
- After `:` in selectors: `$name` → dynamic, bare `name` → literal key
- In phrase call arguments: `$arg` → parameter value, bare `name` → term ref
- `$$` escape sequence produces literal `$`

**Evaluator:**
- `resolve_reference()` uses AST distinction instead of runtime param lookup
- `resolve_selector_candidates()` matches on `Selector::Parameter` vs
  `Selector::Literal` directly
- Remove the "check if name is a parameter, else treat as literal" fallback

**Validation (macro):**
- `{$x}` where `$x` is not declared → compile error
- `{n}` where `$n` is a declared parameter → compile error with suggestion
- `draw(n)` without `$` → compile error

**Tests:**
- Update all existing tests to use `$` syntax
- New tests for `$` parameter resolution, `$$` escaping, error cases

### Migration impact

Mechanical find-and-replace: every parameter gains `$` in every position. The
compiler catches anything missed.

---

## Milestone 2: Term vs Phrase Distinction

**Goal:** Every definition is formally either a term (no parameters, variants
via `{}`) or a phrase (parameters, template body). The AST, parsers, and
evaluator enforce this distinction.

**Why second:** The term/phrase distinction is already implicit — every existing
definition follows the rule. This milestone makes it explicit and enforced,
enabling better error messages and setting the stage for `:match`.

### Changes

**AST types** (`ast.rs` and `input.rs`):
- Split the unified `PhraseDefinition` into `TermDefinition` and
  `PhraseDefinition` (or add a `DefinitionKind` discriminant)
- `TermDefinition`: name, tags, body (`Simple` or `Variants`), no parameters
- `PhraseDefinition`: name, parameters, tags, optional `:from`, template body
  (no variant block yet — `:match` comes in Milestone 4)

**Both parsers:**
- If definition has `()` parameters AND `{}` variant block → parse error
- Empty parameter list `name() = ...` → parse error ("use a term instead")
- `:from` only valid on phrases (definitions with parameters)

**Evaluator:**
- Use definition kind for clearer error messages
- `{name(...)}` on a term → error: "term cannot be called with arguments"
- `{name:key}` on a bare phrase name → error: "phrase requires `()`"
- Remove the v1 auto-forwarding logic where `{name:sel}` on a phrase
  auto-forwards selectors as arguments when count matches parameter count
  (evaluator.rs ~lines 107-134). In v2, `{name:sel}` on a phrase is always
  an error — the user must write `{name(...):sel}` instead

**Registry:**
- `PhraseRegistry` stores `DefinitionKind` per entry
- Translation validation: source term must be translated as term, source phrase
  as phrase

**Code generation:**
- Terms generate zero-parameter functions
- Phrases generate parameterized functions
- (Already the current behavior, just make it explicit in codegen logic)

**Tests:**
- Rejection of combined parameters + variants
- Empty parameter list `name() = ...` → parse error
- Error messages use "term" and "phrase" terminology
- Source/translation kind mismatch detection
- Auto-forwarding removal: `{cards:$n}` where `cards` is a phrase → error

### Migration impact

Non-breaking for existing code (all existing definitions already follow the
rule). The change adds enforcement of the implicit pattern.

---

## Milestone 3: `*` Default Variant Marker

**Goal:** One variant in a variant block can be marked with `*` as the default.
A bare reference to a term (no `:` selector) returns the `*`-marked variant.
If no `*` is present, the first declared variant is the default.

**Why third:** This is a prerequisite for `:match`, where the `*` marks the
required default/fallback branch. It also improves the term system
independently.

### Changes

**AST types:**
- `VariantEntry` gains a `is_default: bool` field
- Validation: at most one `*` per variant block (for single-dimension keys)

**Both parsers:**
- Recognize `*` before a variant key: `*one: "card"`, `*other: "{$n} cards"`
- `*` only valid on top-level keys (not on multi-dimensional like `*nom.one`)

**Evaluator:**
- When resolving a bare term reference (no selector), return the `*`-marked
  variant's text, or the first variant if no `*`
- A bare reference to a term with only multi-dimensional keys and no `*` → error

**Tests:**
- `*` marks default, bare reference returns it
- First variant is default when no `*`
- Multiple `*` markers → parse error
- `*` on multi-dimensional key → parse error

### Migration impact

Additive — existing code without `*` markers works identically (first variant
is used as default, matching current behavior).

---

## Milestone 4: `:match` Keyword

**Goal:** Phrases can branch on parameter values using `:match($param)`. This
replaces the v1 pattern where phrases had variant blocks (which violated the
term/phrase rule from Milestone 2). One branch must be marked with `*` as the
default.

This is the largest and most complex milestone.

### Changes

**AST types:**
- `PhraseDefinition` gains an optional `match_params: Vec<String>` field
- Phrase body becomes: either `Simple(Template)` or `MatchBlock(Vec<MatchBranch>)`
- `MatchBranch`: keys (with `is_default`), template body
- Support multi-parameter match: `:match($n, $entity)` with dot-notation keys

**Both parsers:**
- Parse `:match($param)` after `=` in phrase definitions
- Parse `:match($p1, $p2)` for multi-parameter match
- Accept both orderings: `:from($s) :match($n)` and `:match($n) :from($s)`
  are equivalent — the declaration order does not matter
- Match block body: `{ key: "template", *default: "template" }`
- Named keys (`one`, `other`, `masc`, `fem`) and numeric keys (`0`, `1`, `2`)
- Multi-key shorthand in match branches: `one, other: "template"` assigns the
  same template to multiple keys, same syntax as term variant blocks
- Validate exactly one `*` default per dimension

**Evaluator — match resolution:**
- Numeric matching: exact number → CLDR category → `*` default
- Tag-based matching: iterate phrase tags in order → first match → `*` default
- Multi-parameter: resolve each dimension independently, combine with dot
  notation
- Each branch template has access to all phrase parameters

**Combining `:from` and `:match`:**
- A phrase can use both: `name($n, $s) = :from($s) :match($n) { ... }`
- `:from` determines inherited tag/variant structure
- `:match` branches within each inherited variant's evaluation
- Bare interpolation of `:from` param (`{$s}`) sees per-variant text;
  phrase calls like `{subtype($s)}` see the full Phrase value

**Validation:**
- Phrase with variant block MUST use `:match`
- `:match` parameter must be declared in phrase signature
- Exactly one `*` default per dimension

**Tests:**
- Single-parameter numeric match (exact, CLDR, default)
- Single-parameter tag-based match
- Multi-parameter match with dot-notation keys
- Multi-key shorthand in match branches
- Combined `:from` + `:match`
- Combined `:match` + `:from` (reversed order, equivalent)
- Missing `*` default → error
- Wrong number of `:match` params → error

### Migration impact

Breaking — v1 phrases that had variant blocks must be rewritten with `:match`.
For example:

```
// v1
card(n) = { one: "card", other: "cards" };

// v2
cards($n) = :match($n) { 1: "a card", *other: "{$n} cards" };
```

---

## Milestone 5: Numeric Keys in `:match`

**Goal:** `:match` branches support numeric keys (`0`, `1`, `2`) with
exact-match-first resolution priority, following ICU MessageFormat precedent.

**Why separate from Milestone 4:** Numeric keys add parsing and resolution
complexity. Milestone 4 can ship with only named keys (CLDR categories and tag
names). Numeric keys are additive on top.

### Changes

**Both parsers:**
- Accept digit sequences as `:match` branch keys
- Validate: no negative numbers, no floats
- Reject numeric keys in term variant blocks — numeric branching is exclusive
  to `:match` (DESIGN_V2.md: "Variant keys are always named identifiers")

**Evaluator:**
- Resolution order for numeric parameters:
  1. Exact numeric key (n=0 → try `"0"`)
  2. CLDR plural category (n=3 → `"other"`)
  3. `*` default branch
- Multi-dimensional: numeric priority preserved per-dimension
- `resolve_selector_candidates()` returns ordered candidate list
- Literal numeric selection on terms (`{card:3}`) remains unsupported — the `:`
  operator on terms only accepts named identifiers and `$`-prefixed parameters

**Tests:**
- Exact numeric match wins over CLDR: `1:` beats `one:`
- Numeric fallthrough to CLDR when no exact match
- Multi-dimensional numeric keys: `acc.0:` vs `acc.one:`
- Numeric keys in term variant blocks → parse error
- Literal numeric selector on term `{card:3}` → parse error

### Migration impact

Purely additive — no existing syntax changes. Numeric keys are new capability.

---

## Milestone 6: Dynamic Transform Context `()`

**Goal:** Transform context uses `()` for dynamic/parameter values and `:`
exclusively for static/literal values. This aligns transforms with the general
`$`/`:` pattern.

### Changes

**AST types:**
- Transform `context` field becomes an enum: `StaticContext(String)` vs
  `DynamicContext(String)`

**Both parsers:**
- `@transform:literal` → static context
- `@transform($param)` → dynamic context
- `@transform:literal($param)` → both (rare but supported)

**Evaluator:**
- Static context: use literal string directly
- Dynamic context: look up parameter value, use for transform logic

**Tests:**
- `{@der:acc card}` → static context "acc"
- `{@count($n) card}` → dynamic context with parameter `$n`
- Combined static + dynamic: `{@transform:lit($param) ref}`

### Migration impact

Breaking for any code using dynamic transform context. The change:
- `{@count:$n card}` → `{@count($n) card}`

Note: with `$` prefix from Milestone 1, `{@count:$n card}` could also work
(`:$n` is unambiguous). The decision to use `()` instead is an aesthetic choice
for consistency with phrase calls. This milestone could be deferred or made
optional.

---

## Milestone 7: Literal Arguments in Phrase Calls

**Goal:** Phrase calls accept literal numbers and strings as arguments:
`{cards(2)}`, `{trigger("Attack")}`.

### Changes

**AST types:**
- Phrase call `Argument` enum gains `Number(i64)` and `StringLiteral(String)`
  variants (in addition to `Parameter` and `TermRef`)

**Both parsers:**
- Inside `()`: recognize integer literals and quoted strings
- String escaping: `\"` for literal quote, `\\` for literal backslash

**Evaluator:**
- `Number(n)` → `Value::Number(n)`
- `StringLiteral(s)` → `Value::String(s)` (no tags, no variants)

**Validation:**
- String values passed as arguments have no tags/variants — selecting a variant
  from one is a runtime error
- Nested calls not allowed: `{f(g($x))}` → error
- Expressions not allowed: `{f(card:one)}` → error

**Tests:**
- `{cards(2)}` passes literal 2
- `{trigger("Attack")}` passes literal string
- Nested call → error
- Expression as argument → error

### Migration impact

Purely additive — new capability.

---

## Milestone 8: Escape Sequence Cleanup

**Goal:** Align escape sequences with v2 design. Only `{{` and `}}` are needed
in regular text. Inside `{}` expressions: `::`, `@@`, `$$`.

### Changes

- `$` in regular text outside `{}` is literal (no escaping needed)
- `$` inside `{}` starts a parameter reference; `$$` for literal `$`
- `:`, `@` in regular text are literal
- Verify `{{` and `}}` work in all contexts
- Verify `@plural` is English-only (DESIGN_V2.md restricts it because other
  languages need case/gender-aware plural selection via `:` instead)

This is mostly already handled by Milestone 1's `$$` escape. This milestone
ensures consistency and removes any unnecessary escape requirements.

### Migration impact

Simplification — fewer escapes needed in regular text. The only potential break
is if existing code used `$$` for something else (unlikely).

---

## Milestone 9: Comprehensive Validation and Error Messages

**Goal:** Both compile-time (macro) and runtime (interpreter) validation
produce clear, actionable error messages for all v2 rule violations.

### Compile-time errors (macro)

| Violation | Example | Message |
|-----------|---------|---------|
| `$` on non-parameter | `{$card}` | `'$card' is not a declared parameter — remove '$' to reference term 'card'` |
| Bare name matches param | `{n}` in `draw($n)` | `'n' matches parameter '$n' — use {$n}` |
| Phrase used with `:` | `{cards:other}` | `'cards' is a phrase — use cards(...):other` |
| Term used with `()` | `{card($n)}` | `'card' is a term — use {card:$n}` |
| Arity mismatch | `{cards($n, $m)}` | `phrase 'cards' expects 1 parameter, got 2` |
| Nested call | `{f(g($x))}` | `nested phrase calls not supported as arguments` |
| Missing `*` default | `:match($n) { 1: "a" }` | `':match' requires a '*' default branch` |
| `:from` on term | `card = :from(...)` | `':from' requires parameters` |
| Empty param list | `name() = ...` | `empty parameter list — use a term instead` |
| Numeric term key | `card = { 1: "x" }` | `term variant keys must be named identifiers — use ':match' for numeric branching` |

### Runtime errors (interpreter)

- Same validations applied to `.rlf` translation files at load time
- `EvalError` variants for each violation
- Source/translation kind mismatch detection

### New `EvalError` variants

- `ArgumentsToTerm { name }` — `()` on a term
- `SelectorOnPhrase { name }` — bare `:` on a phrase without `()`
- `DefinitionKindMismatch { name, source_kind, translation_kind }`
- `UnknownParameter { name }` — `$name` not in scope
- `MissingMatchDefault { name }` — `:match` without `*`

---

## Milestone 10: Documentation and Downstream Migration

**Goal:** Update all documentation and migrate the dreamtides consumer.

### Documentation updates

- `docs/DESIGN_V2.md` → becomes the canonical `docs/DESIGN.md`
- Archive or remove `docs/DESIGN.md` (v1 spec)
- `CLAUDE.md` — update terminology and concepts
- All appendices — update examples to v2 syntax
- Remove superseded planning documents

### Downstream migration (dreamtides)

- Update `rlf!` macro invocations in `rules_engine/src/strings/src/phrases.rs`
- Mechanical changes: add `$` to all parameters, convert phrase variant blocks
  to `:match`, add `*` defaults
- Update any `.rlf` translation files
- Run dreamtides test suite

---

## Implementation Order and Dependencies

```
Milestone 1: $ prefix
    │
    ├─► Milestone 2: Term/phrase distinction
    │       │
    │       ├─► Milestone 3: * default markers
    │       │       │
    │       │       └─► Milestone 4: :match keyword ◄── largest milestone
    │       │               │
    │       │               └─► Milestone 5: Numeric keys in :match
    │       │
    │       └─► Milestone 9: Validation & errors (can start after M2)
    │
    ├─► Milestone 6: Transform context () (independent of M2-M5)
    │
    ├─► Milestone 7: Literal arguments (independent of M2-M5)
    │
    └─► Milestone 8: Escape cleanup (independent, small)

Milestone 10: Documentation (after all other milestones)
```

Milestones 6, 7, and 8 are independent of Milestones 2-5 and can be done in
any order or in parallel. Milestone 9 can begin after Milestone 2 and
accumulate error cases as each subsequent milestone lands.

### Suggested execution order

1. **Milestone 1** — `$` prefix (foundational)
2. **Milestone 2** — Term/phrase distinction
3. **Milestone 3** — `*` default markers
4. **Milestone 7** — Literal arguments (quick win, independent)
5. **Milestone 4** — `:match` keyword (largest, depends on M2+M3)
6. **Milestone 5** — Numeric keys in `:match`
7. **Milestone 6** — Transform context `()` (could be deferred)
8. **Milestone 8** — Escape cleanup
9. **Milestone 9** — Validation & errors (accumulates throughout)
10. **Milestone 10** — Documentation & migration

### Effort estimates (relative)

| Milestone | Scope | Risk |
|-----------|-------|------|
| M1: `$` prefix | Medium — touches all parsers, evaluator, all tests | Low — mechanical changes |
| M2: Term/phrase | Medium — AST refactor, parser validation | Low — already implicit |
| M3: `*` default | Small — parser + evaluator | Low |
| M4: `:match` | **Large** — new keyword, match resolution, `:from` combo | Medium — complex evaluation semantics |
| M5: Numeric keys | Small-Medium — resolution order change | Low |
| M6: Transform `()` | Small — parser + evaluator for transforms | Low |
| M7: Literal args | Small — parser + evaluator | Low |
| M8: Escapes | Small — parser cleanup | Low |
| M9: Validation | Medium — many error cases | Low |
| M10: Docs | Medium — many files | Low |

---

## Open Questions

1. **Milestone 6 necessity:** With `$` prefix, `{@count:$n card}` is already
   unambiguous. Is `{@count($n) card}` worth the breaking change? Could keep
   `:$n` for transform context and skip M6 entirely.

2. **Incremental vs big-bang migration:** Should dreamtides migrate per-milestone
   or wait for all milestones and migrate once? Per-milestone is safer but
   requires more coordination.

3. **v1 compatibility mode:** Should the runtime parser accept both v1 and v2
   syntax during a transition period? Adds complexity but eases migration.

4. **`:match` on terms:** DESIGN_V2.md limits `:match` to phrases. Should terms
   support a similar branching mechanism, or is parameterized selection on terms
   (`{card:$n}`) sufficient?

5. **`Float` type in `Value`:** `Float(f64)` is preserved as a runtime value
   type. Floats can be interpolated and passed as arguments. For `:match`
   resolution and parameterized selection (`:$n`), floats are truncated to
   integers before CLDR plural category lookup. Float literals as match branch
   keys remain disallowed (only integer keys).
