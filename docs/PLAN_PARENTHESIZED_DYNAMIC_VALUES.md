# Plan: Parenthesized Dynamic Values

## Motivation

In the current RLF syntax, the `:` operator is overloaded—it serves as both a
literal variant selector (`:other`, `:acc`) and a dynamic parameter reference
(`:n`, `:thing`). The evaluator resolves the ambiguity at runtime by checking
whether the name after `:` is a declared parameter. This creates several
problems:

1. **Readability**: `{card:n}` looks identical to `{card:other}` syntactically,
   but they mean fundamentally different things. A reader must know whether `n`
   is a parameter or a variant key.

2. **Fragility**: If a parameter is renamed to match a variant key (or vice
   versa), the meaning silently changes. For example, if a phrase has a variant
   key `"one"` and a parameter named `one`, `{card:one}` is ambiguous.

3. **Learnability**: New users must understand the implicit resolution rules.
   The distinction between "this is a name I'm using directly" vs. "this is a
   variable whose value determines the selection" is invisible in the syntax.

4. **Transform context**: The same ambiguity exists for transform contexts—
   `@count:n` (dynamic, parameter n) looks like `@der:acc` (static, literal
   "acc").

## Proposed Change

Use **parentheses** `()` for dynamic (parameter-based) values and **colon** `:`
exclusively for static (literal) values:

| Meaning              | Current Syntax    | New Syntax        |
| -------------------- | ----------------- | ----------------- |
| Literal selection    | `{card:other}`    | `{card:other}`    |
| Dynamic selection    | `{card:n}`        | `{card(n)}`       |
| Multi-dim (both lit) | `{card:acc:dat}`  | `{card:acc:dat}`  |
| Multi-dim (lit+dyn)  | `{card:acc:n}`    | `{card:acc(n)}`   |
| Tag-based selection  | `{destroyed:thing}` | `{destroyed(thing)}` |
| Static xform context | `{@der:acc karte}`  | `{@der:acc karte}` |
| Dynamic xform ctx    | `{@count:n card}`   | `{@count(n) card}` |

### Rule

After `:`, only literal/static identifiers are allowed—never parameter names.
Inside `()`, only parameter names are allowed—never literals.

## Syntax Specification

### Selection

```
{phrase:literal}         Static variant selection
{phrase(param)}          Dynamic selection using parameter value
{phrase:lit1:lit2}       Multi-dimensional static
{phrase:lit(param)}      Mixed: static dimension + dynamic resolution
```

**Constraints:**
- At most one `(param)` per reference, and it must be the final selector
- Zero or more `:literal` selectors can precede the `(param)`
- `{phrase(p)(q)}` is **not valid** (only one dynamic selector)
- `{phrase(p):lit}` is **not valid** (dynamic must be last)

### Transform Context

```
{@transform phrase}              No context
{@transform:literal phrase}      Static context
{@transform(param) phrase}       Dynamic context
```

**Constraints:**
- A transform has either `:literal` or `(param)` context, not both
- `{@transform:lit(param) phrase}` is **not valid**

### Phrase Calls

Phrase calls already use parentheses and are unchanged:

```
{phrase_name(arg1, arg2)}        Phrase call (existing syntax)
```

### Interaction Between Phrase Calls and Selection

Phrase calls can be followed by static selectors:

```
{subtype(s):other}      Call subtype(s), then select "other" variant
```

This is unambiguous because the phrase call arguments are comma-separated, while
the trailing `(param)` dynamic selector takes exactly one identifier.

### Disambiguation Rule

When `name(x)` appears in an interpolation:

- If `name` is a **phrase declared with parameters**, this is a **phrase call**
- If `name` is a **phrase without parameters** or a **parameter**, and `x` is a
  parameter name, this is a **dynamic selection**

In practice, the parser does not need to distinguish these at parse time. Both
parse as "reference with argument." The evaluator determines the semantics based
on phrase definitions, exactly as it does today for `:`.

However, there is a structural difference: phrase calls can have multiple
comma-separated arguments (`phrase(a, b, c)`), while dynamic selection has
exactly one argument (`phrase(n)`). The parser can use this heuristic, or simply
defer to evaluation.

## Complete Before/After Examples

### Basic plural selection

```
// Before
draw(n) = "Draw {n} {card:n}.";

// After
draw(n) = "Draw {n} {card(n)}.";
```

### Literal selection

```
// Before (unchanged)
all_cards = "All {card:other}.";

// After (same)
all_cards = "All {card:other}.";
```

### Multi-dimensional (Russian)

```
// Before
draw(n) = "Возьмите {n} {card:acc:n}.";

// After
draw(n) = "Возьмите {n} {card:acc(n)}.";
```

### Tag-based selection (Spanish)

```
// Before
destroy(thing) = "{thing} fue {destroyed:thing}.";

// After
destroy(thing) = "{thing} fue {destroyed(thing)}.";
```

### Selection on phrase parameter

```
// Before
with_allies(base, counting) =
    "{base} with cost less than the number of allied {counting:other}";

// After (unchanged — :other is literal)
with_allies(base, counting) =
    "{base} with cost less than the number of allied {counting:other}";
```

### Dynamic entity selection

```
// Before
draw_entities(n, entity) = "Draw {n} {entity:n}.";

// After
draw_entities(n, entity) = "Draw {n} {entity(n)}.";
```

### Transform with static context (German)

```
// Before (unchanged)
destroy_card = "Zerstöre {@der:acc karte}.";

// After (same)
destroy_card = "Zerstöre {@der:acc karte}.";
```

### Transform with dynamic context (Chinese)

```
// Before
draw(n) = "抽{@count:n card}";

// After
draw(n) = "抽{@count(n) card}";
```

### Transform with static context + selection

```
// Before
get_card = "Nimm {@der:acc karte:one}.";

// After (unchanged — both :acc and :one are literal)
get_card = "Nimm {@der:acc karte:one}.";
```

### Transform with dynamic context and dynamic selection

```
// Before
draw(n) = "抽{@count:n card:n}";

// After
draw(n) = "抽{@count(n) card(n)}";
```

### Phrase call + selection (from :from)

```
// Before (unchanged — subtype(s) is a phrase call, :other is literal)
dissolve_all(s) = "Dissolve all {subtype(s):other}.";

// After (same)
dissolve_all(s) = "Dissolve all {subtype(s):other}.";
```

### Spanish plural article

```
// Before
return_all(t) = "devuelve {@el:other t} a mano";

// After (unchanged — :other is literal context)
return_all(t) = "devuelve {@el:other t} a mano";
```

### Auto-capitalization with selection

```
// Before
draw(n) = "Draw {n} {@cap card:n}.";

// After
draw(n) = "Draw {n} {@cap card(n)}.";
```

## Implementation Plan

### Phase 1: AST Changes

**Files:**
- `crates/rlf/src/parser/ast.rs`
- `crates/rlf-macros/src/input.rs`

Change `Selector` from a single variant to distinguish static from dynamic:

```rust
// In ast.rs (runtime)
pub enum Selector {
    /// Static literal key, e.g. :other, :acc
    Literal(String),
    /// Dynamic parameter reference, e.g. (n), (thing)
    Parameter(String),
}

// In input.rs (macro) — parallel change
pub enum SelectorKind {
    Literal(SpannedIdent),
    Parameter(SpannedIdent),
}
```

Similarly for transform context:

```rust
// In ast.rs
pub struct Transform {
    pub name: String,
    pub context: Option<Selector>,  // Selector now distinguishes Literal vs Parameter
}
```

### Phase 2: Parser Changes

**Files:**
- `crates/rlf/src/parser/template.rs`
- `crates/rlf-macros/src/parse.rs`

Update both parsers to handle the new syntax:

**Runtime parser (`template.rs`):**
- `:identifier` parses as `Selector::Literal(identifier)`
- `(identifier)` parses as `Selector::Parameter(identifier)`
- For transforms: `@name:lit` → static context, `@name(param)` → dynamic context
- Enforce constraints: `(param)` must be last selector, at most one

**Macro parser (`parse.rs`):**
- Parallel changes to runtime parser
- Can additionally validate at compile time that names inside `()` are declared
  parameters and names after `:` are NOT declared parameters

### Phase 3: Evaluator Changes

**Files:**
- `crates/rlf/src/interpreter/evaluator.rs`

Update `resolve_selector_candidates()` to use the new AST distinction:

```rust
fn resolve_selector_candidates(
    selector: &Selector,
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<Vec<String>, EvalError> {
    match selector {
        Selector::Literal(name) => {
            // Always use as literal key — no parameter lookup
            Ok(vec![name.clone()])
        }
        Selector::Parameter(name) => {
            // Always look up parameter — error if not found
            let value = ctx.get_param(name).ok_or_else(|| {
                EvalError::UnknownParameter { name: name.clone() }
            })?;
            match value {
                Value::Number(n) => Ok(vec![plural_category(lang, *n).to_string()]),
                Value::Float(f) => Ok(vec![plural_category(lang, *f as i64).to_string()]),
                Value::Phrase(phrase) => {
                    let tags: Vec<String> =
                        phrase.tags.iter().map(ToString::to_string).collect();
                    if tags.is_empty() {
                        return Err(EvalError::MissingTag { ... });
                    }
                    Ok(tags)
                }
                Value::String(s) => {
                    if let Ok(n) = s.parse::<i64>() {
                        Ok(vec![plural_category(lang, n).to_string()])
                    } else {
                        Ok(vec![s.clone()])
                    }
                }
            }
        }
    }
}
```

Update `apply_transforms()` similarly for transform context resolution.

### Phase 4: Compile-Time Validation (Macro)

**Files:**
- `crates/rlf-macros/src/parse.rs` (or a validation pass)

Add compile-time checks in the macro:
- Names inside `()` must be declared parameters → error if not
- Names after `:` must NOT be declared parameters → warning or error if they are
  (this catches accidental use of the old syntax)

### Phase 5: Test Updates

**Files:**
- `crates/rlf/tests/` — runtime interpreter tests
- `crates/rlf-macros/tests/` — macro compile tests and trybuild tests

Update all existing tests to use the new syntax. Add new tests:
- Parser correctly distinguishes `Selector::Literal` from `Selector::Parameter`
- Dynamic selector `(n)` with various value types (number, phrase, string)
- Mixed `{phrase:lit(param)}` multi-dimensional selection
- Transform with `(param)` context
- Error cases: `(param)` not last, multiple `()`, unknown parameter in `()`
- Compile-time error when using parameter name after `:`

### Phase 6: Documentation Updates

**Files:**
- `docs/DESIGN.md`
- `docs/APPENDIX_STDLIB.md`
- `docs/APPENDIX_RUSSIAN_TRANSLATION.md`
- `docs/APPENDIX_SPANISH_TRANSLATION.md`
- `docs/APPENDIX_DREAMTIDES_ADOPTION.md`
- `CLAUDE.md`

Update all examples in documentation to use the new syntax.

### Phase 7: Error Messages and Migration Help

Add helpful error messages for common mistakes during the transition:

```
error: parameter name 'n' used after ':' — use parentheses for dynamic selection
  --> src/strings.rlf.rs:3:28
   |
   = help: change {card:n} to {card(n)}
```

## Edge Cases and Considerations

### Escape sequences

The current escape `::` (produces literal `:`) is unaffected. No new escape is
needed for `()` inside templates since bare parentheses in text are only
meaningful inside `{}` interpolation blocks.

### Backward compatibility

This is a **breaking change**. All existing `.rlf` files and `rlf!` macro
invocations using dynamic selectors must be updated. The compile-time validation
(Phase 4) and error messages (Phase 7) will guide migration.

### Parameter names matching variant keys

This change **eliminates** the current ambiguity where a parameter name could
shadow a variant key. With the new syntax:
- `{card:one}` always means literal key "one"
- `{card(one)}` always means parameter `one`

Even if a parameter is named `other`, `{card:other}` still means the literal
variant key "other", not the parameter.

### Phrase call vs. dynamic selection

`name(x)` could be either a phrase call or a dynamic selection. The parser does
not need to distinguish them—both produce the same AST node ("reference with
arguments"). The evaluator resolves semantics based on phrase definitions. This
mirrors the current design where the evaluator resolves `:x` ambiguity.

If we want to make this syntactically unambiguous in the future, we could
require phrase calls to always use the `phrase_name(args)` form while dynamic
selection uses a different delimiter. However, this is not necessary for the
initial change since the current ambiguity resolution already works correctly.

### Runtime interpreter compatibility

Translation `.rlf` files loaded at runtime must also use the new syntax. The
runtime parser (`template.rs`) must be updated in lockstep with the macro parser.
There is no need for a compatibility mode since translation files are controlled
by the application developer.

## Migration Checklist

1. [ ] Update AST types in both parsers
2. [ ] Update runtime parser (`template.rs`)
3. [ ] Update macro parser (`parse.rs`)
4. [ ] Update evaluator (`evaluator.rs`)
5. [ ] Add compile-time validation in macro
6. [ ] Update all tests
7. [ ] Add migration error messages
8. [ ] Update all documentation
9. [ ] Update any `.rlf` translation files in the repo
10. [ ] Run `just review` to validate everything passes
