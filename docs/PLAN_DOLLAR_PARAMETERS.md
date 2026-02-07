# Plan: Dollar-Sign Parameter References

## Overview

All parameter references in template bodies are prefixed with `$`. Bare names
always refer to terms or phrases defined in the RLF file. This follows the
convention established by [Mozilla Fluent](https://projectfluent.org/).

**The rule:** Parameters always have `$`, everywhere — declarations,
template bodies, `:from`, phrase calls. Bare names are always terms, phrases,
or literal keys.

## Related Plans

This plan builds on and interacts with three other proposals:

- **`PLAN_TERM_VS_PHRASE.md`**: Introduces the term/phrase distinction. The `$`
  prefix complements this by making parameter references visually distinct from
  term references in template bodies. Together they eliminate all naming
  ambiguity in the language.

- **`PLAN_PARENTHESIZED_DYNAMIC_VALUES.md`**: Proposed using `()` for dynamic
  selection to disambiguate `{card:n}` (is `n` a literal key or parameter?).
  With `$`, this disambiguation is handled by the prefix instead: `{card:$n}`
  vs `{card:other}`. The parenthesized syntax change for selectors becomes
  **unnecessary** — the `:` operator is unambiguous when `$` is present.
  Parenthesized dynamic values may still be desirable for other reasons
  (aesthetics, consistency) but are no longer required for correctness.

- **`PLAN_NUMERIC_SELECTORS.md`**: Numeric literal selectors like `{card:3}`
  are unambiguous with `$` — a bare `3` after `:` is always a literal key.
  `{card:$n}` where `n=3` is always dynamic resolution. No prefix like ICU
  MessageFormat's `=3` is needed.

## Syntax

### Parameter declarations

Parameters use `$` in declarations too:

```rust
rlf! {
    greet($name) = "Hello, {$name}!";
    draw($n) = "Draw {$n} {card:$n}.";
    abandon_n($n, $target) = "abandon {$n} {$target:$n}";
}
```

The `$` makes it immediately clear what's a parameter at every point in the
definition — declaration, template body, selectors, phrase calls.

### Interpolation

```
{$name}              Interpolate parameter 'name'
{card}               Reference term 'card' (default variant)
{Card}               Reference term 'card' with @cap (auto-capitalization)
```

### Selection

```
{card:other}         Static: select literal variant key 'other'
{card:$n}            Dynamic: select variant using parameter n
{card:acc:$n}        Mixed: literal 'acc' dimension, then dynamic n
{card:acc:one}       Static: both dimensions are literal keys
{destroyed:$thing}   Dynamic: select by parameter thing's tags
```

After `:`, the presence or absence of `$` is definitive:
- `$name` → always a parameter lookup
- `name` → always a literal variant key

### Transform context

```
{@der:acc karte}     Static context: literal 'acc'
{@count:$n card}     Dynamic context: parameter n
{@el:other card}     Static context: literal 'other'
{@cap card}          No context
```

Same rule: `$` after `:` means parameter, bare name means literal.

### Phrase calls

Arguments in phrase calls also use `$`:

```
{subtype($s)}                    Phrase call with parameter s
{energy($e)}                     Phrase call with parameter e
{subtype($s):other}              Phrase call, then static selection
{@a subtype($s)}                 Transform applied to phrase call result
```

### `:from` declarations

The `:from` modifier references a parameter and uses `$` consistently:

```
subtype($s) = :from($s) "<b>{$s}</b>";
```

`$s` appears everywhere — declaration, `:from`, and template body.

## Complete Before/After Examples

### Basic interpolation

```
// Before
greet(name) = "Hello, {name}!";
damage(amount, target) = "Deal {amount} damage to {target}.";

// After
greet($name) = "Hello, {$name}!";
damage($amount, $target) = "Deal {$amount} damage to {$target}.";
```

### Plural selection

```
// Before
draw(n) = "Draw {n} {card:n}.";

// After
draw($n) = "Draw {$n} {card:$n}.";
```

### Literal selection (unchanged)

```
// Before
all_cards = "All {card:other}.";

// After (same — no parameters involved)
all_cards = "All {card:other}.";
```

### Tag-based selection

```
// Before
destroy(thing) = "{thing} fue {destroyed:thing}.";

// After
destroy($thing) = "{$thing} fue {destroyed:$thing}.";
```

### Russian multi-dimensional

```
// Before
allied(entity) = "{allied_adj:entity} {entity:nom.one}";
with_cost_less_than_allied(base, counting) =
    "{base:nom.one} со стоимостью меньше количества союзных {counting:gen.many}";

// After
allied($entity) = "{allied_adj:$entity} {$entity:nom.one}";
with_cost_less_than_allied($base, $counting) =
    "{$base:nom.one} со стоимостью меньше количества союзных {$counting:gen.many}";
```

### Spanish article transforms

```
// Before
abandon_one(target) = "abandona {@un target}";
return_all(target) = "devuelve {@el:other target} a la mano";

// After
abandon_one($target) = "abandona {@un $target}";
return_all($target) = "devuelve {@el:other $target} a la mano";
```

### Phrase calls with `:from`

```
// Before
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
dissolve_subtype(s) = "Dissolve {@a subtype(s)}.";
dissolve_all(s) = "Dissolve all {subtype(s):other}.";

// After
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";
```

### Chinese transform context

```
// Before
draw(n) = "抽{@count:n card}";

// After
draw($n) = "抽{@count:$n card}";
```

### Term referencing other terms (unchanged — no parameters)

```
// Before
draw_one = "Draw {@a card}.";
energy_symbol = "<e>●</e>";
heading = "{@cap card}";

// After (same — these are terms, no parameters)
draw_one = "Draw {@a card}.";
energy_symbol = "<e>●</e>";
heading = "{@cap card}";
```

### Mixed static and dynamic selection

```
// Before
draw_ru(n) = "Возьмите {n} {card:acc:n}.";
get_card = "Nimm {@der:acc karte:one}.";

// After
draw_ru($n) = "Возьмите {$n} {card:acc:$n}.";
get_card = "Nimm {@der:acc karte:one}.";       // unchanged — no parameters
```

### Real-world phrase (dreamtides example)

```
// Before
prevent_event(e) = "{Prevent} a played event unless the opponent pays {energy(e)}.";

// After
prevent_event($e) = "{Prevent} a played event unless the opponent pays {energy($e)}.";
```

## Disambiguation Summary

With `$` and the term/phrase distinction (from `PLAN_TERM_VS_PHRASE.md`),
every reference in a template body is unambiguous:

| Syntax | Meaning | How it's clear |
|--------|---------|----------------|
| `{$n}` | Parameter interpolation | `$` prefix |
| `{card}` | Term reference | No `$`, no `()` |
| `{Card}` | Term reference + `@cap` | No `$`, uppercase |
| `{energy($e)}` | Phrase call | `()` with `$` arg |
| `{card:other}` | Static variant selection | No `$` after `:` |
| `{card:$n}` | Dynamic variant selection | `$` after `:` |
| `{@a $target}` | Transform on parameter | `$` prefix |
| `{@a card}` | Transform on term | No `$` |
| `{@der:acc karte}` | Transform with static context | No `$` after `:` |
| `{@count:$n card}` | Transform with dynamic context | `$` after `:` |

No case requires knowing the definition of the referenced name to determine
the meaning. The syntax alone is sufficient.

## Interaction with Escape Sequences

The `$` character needs an escape sequence for literal use in templates.
Use `$$` to produce a literal `$`:

```
price = "The cost is $$5.";
// → "The cost is $5."
```

This follows the same doubling convention as `{{`/`}}` and `::`.

## Implementation Plan

### Phase 1: Parser Changes

**Files:**
- `crates/rlf-macros/src/parse.rs`
- `crates/rlf/src/parser/template.rs`

Update both parsers to recognize `$` as the parameter marker everywhere:

**In parameter declarations:**
- `draw($n)` → parameter named `$n`
- `draw(n)` → parse error: parameters must use `$`

**In `:from`:**
- `:from($s)` → inherits from parameter `$s`
- `:from(s)` → parse error: must use `$`

**In interpolation position:**
- `{$name}` → parsed as parameter reference
- `{name}` → parsed as term/phrase reference
- `$$` in literal text → escaped literal `$`

**After `:` in selectors:**
- `:$name` → `Selector::Parameter(name)`
- `:name` → `Selector::Literal(name)`
- `:3` → `Selector::Literal("3")` (numeric literal, per `PLAN_NUMERIC_SELECTORS.md`)

**After `:` in transform context:**
- `@transform:$name` → dynamic context (parameter)
- `@transform:name` → static context (literal)

**In phrase call arguments:**
- `phrase($arg1, $arg2)` → arguments are parameter references
- Bare names in phrase call arguments are term references (passing a
  term's value to a phrase)

### Phase 2: AST Changes

**Files:**
- `crates/rlf-macros/src/input.rs`
- `crates/rlf/src/parser/ast.rs`

The `Reference` type in interpolations must distinguish parameter from
term/phrase:

```rust
// In ast.rs (runtime)
pub enum Reference {
    /// A term or phrase reference: {card}, {draw(...)}, {Card}
    Name(String),
    /// A parameter reference: {$n}, {$name}
    Parameter(String),
}

pub enum Selector {
    /// A literal variant key: :other, :acc, :3
    Literal(String),
    /// A parameter reference: :$n, :$thing
    Parameter(String),
}
```

Transform context similarly uses `Selector` which already distinguishes
literal from parameter.

Phrase call arguments need a similar distinction:

```rust
pub enum Argument {
    /// Pass a parameter value: phrase($arg)
    Parameter(String),
    /// Pass a term reference: phrase(term_name)
    TermRef(String),
}
```

### Phase 3: Evaluator Changes

**Files:**
- `crates/rlf/src/interpreter/evaluator.rs`

Update evaluation logic to use the explicit AST distinction instead of
runtime parameter-name lookup:

```rust
// Before (ambiguous, checks at runtime):
fn resolve_reference(name: &str, ctx: &EvalContext) -> Value {
    if let Some(param) = ctx.get_param(name) {
        param.clone()
    } else {
        ctx.get_phrase(name)
    }
}

// After (explicit from AST):
fn resolve_reference(reference: &Reference, ctx: &EvalContext) -> Value {
    match reference {
        Reference::Parameter(name) => ctx.get_param(name).expect("parameter exists"),
        Reference::Name(name) => ctx.get_phrase(name),
    }
}
```

The same pattern applies to selector resolution (no more "check if it's a
parameter, otherwise treat as literal") and transform context resolution.

### Phase 4: Compile-Time Validation (Macro)

**Files:**
- `crates/rlf-macros/src/parse.rs` (or validation pass)

The macro validates `$` usage at compile time:

- Parameter declarations must use `$`: `draw($n)` not `draw(n)`
- `{$n}` where `$n` is not a declared parameter → compile error
- `{:$n}` where `$n` is not a declared parameter → compile error
- `{name}` where `$name` is a declared parameter → compile error
  (bare name matches a parameter — must use `$`)
- `:from($s)` where `$s` is not a declared parameter → compile error

```
error: unknown parameter '$count'
  --> strings.rlf.rs:3:18
   |
3  |     draw($n) = "Draw {$count} cards.";
   |                       ^^^^^^
   |
   = help: declared parameters: $n
   = help: did you mean {$n}?

error: 'n' matches parameter '$n' — use {$n} to reference it
  --> strings.rlf.rs:3:18
   |
3  |     draw($n) = "Draw {n} cards.";
   |                       ^
   |
   = help: parameter references require '$': {$n}
   = note: without '$', 'n' would reference a term named 'n'
```

### Phase 5: Test Updates

**Files:**
- `crates/rlf/tests/`
- `crates/rlf-macros/tests/`

Update all existing tests to use `$` syntax. Add new tests:

- `$` required in parameter declarations: `draw($n)` not `draw(n)`
- `$` required in `:from`: `subtype($s) = :from($s) ...`
- `{$n}` resolves to parameter value
- `{name}` resolves to term (not parameter, even if parameter `$name` exists)
- `{card:$n}` performs dynamic selection
- `{card:other}` performs static selection
- `$$` produces literal `$` in output
- Compile error for `{$unknown}` (not a declared parameter)
- Compile error for `{n}` when `$n` is a declared parameter
- Compile error for `draw(n)` without `$`
- Phrase call with `$` arguments: `{energy($e)}`
- Transform with `$` context: `{@count:$n card}`
- Phrase call with term argument: `{subtype(ancient)}` (bare = term ref)

### Phase 6: Documentation Updates

**Files:**
- `docs/DESIGN.md`
- `docs/APPENDIX_STDLIB.md`
- `docs/APPENDIX_RUSSIAN_TRANSLATION.md`
- `docs/APPENDIX_SPANISH_TRANSLATION.md`
- `docs/APPENDIX_DREAMTIDES_ADOPTION.md`
- `CLAUDE.md`

Update all examples to use `$` prefix for parameter references.

## Error Handling

### Missing `$` prefix on parameter declaration

```rust
rlf! {
    draw(n) = "Draw {$n} {card:$n}.";
}
```

```
error: parameter 'n' is missing '$' prefix
  --> strings.rlf.rs:2:10
   |
2  |     draw(n) = "Draw {$n} {card:$n}.";
   |          ^
   |
   = help: parameters must use '$': draw($n)
```

### Missing `$` prefix in template body

```rust
rlf! {
    draw($n) = "Draw {n} {card:n}.";
}
```

```
error: 'n' matches parameter '$n' but is missing '$' prefix
  --> strings.rlf.rs:2:19
   |
2  |     draw($n) = "Draw {n} {card:n}.";
   |                       ^        ^
   |
   = help: use {$n} and {card:$n} to reference parameter '$n'
   = note: without '$', 'n' is treated as a term reference
```

### `$` on a non-parameter in template body

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    bad = "Draw {$card}.";
}
```

```
error: '$card' is not a declared parameter
  --> strings.rlf.rs:3:15
   |
3  |     bad = "Draw {$card}.";
   |                  ^^^^^
   |
   = help: 'card' is a term — remove '$' to reference it: {card}
   = note: '$' is only for parameters declared in the phrase signature
```

### `$` in phrase call for a non-parameter

```rust
rlf! {
    energy($e) = "{$e}";
    bad($n) = "pays {energy($n)}.";
}
```

If `$n` is a declared parameter of `bad`, this is valid — pass `$n`'s value to
`energy`. If not:

```
error: unknown parameter '$n' in phrase call
  --> strings.rlf.rs:3:28
   |
3  |     bad($x) = "pays {energy($n)}.";
   |                             ^^
   |
   = help: declared parameters: $x
```

### Literal `$` in text

```rust
rlf! {
    price = "Costs $5.";
}
```

```
error: unexpected '$' — use '$$' for a literal dollar sign
  --> strings.rlf.rs:2:20
   |
2  |     price = "Costs $5.";
   |                    ^
   |
   = help: '$' begins a parameter reference inside templates
   = help: use '$$' for a literal '$': "Costs $$5."
```

### Runtime errors (translation files)

The same validations apply when loading `.rlf` files at runtime, returned as
`EvalError` variants:

```rust
pub enum EvalError {
    // ...

    /// Parameter reference '$name' not found in scope.
    UnknownParameter {
        name: String,
    },

    /// '$' used on a term name, not a parameter.
    DollarOnTerm {
        name: String,
        help: String,
    },

    /// Bare name matches a parameter — likely missing '$'.
    SuspectedMissingDollar {
        name: String,
        help: String,
    },
}
```

## Migration

This is a **breaking change**. All parameter declarations and references must
be updated to add `$`.

**Migration steps:**

1. Update parsers to require `$` on parameters
2. Update all `rlf!` macro invocations — declarations and template bodies
3. Update all `.rlf` translation files
4. Run `just review` to verify everything compiles and passes

The changes are mechanical — every parameter gains a `$` prefix in every
position. A simple find-and-replace per phrase handles most cases, and the
compiler catches anything missed.
