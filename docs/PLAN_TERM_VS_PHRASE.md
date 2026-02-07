# Plan: Term vs Phrase Distinction

## Overview

RLF currently treats all definitions as "phrases" regardless of whether they
declare variants or parameters. This plan introduces a formal distinction
between two kinds of definitions:

- **Term**: Has variants (forms) but no parameters. Represents a lexical
  entry — a noun, adjective, or fixed expression with multiple forms.
- **Phrase**: Has parameters but no variants. Represents a template with
  blanks to fill in.

The rule is: **a definition can have parameters or variants, but not both.**

## Motivation

### Clearer mental model

Today, everything is a "phrase." But the two concepts are fundamentally
different:

```rust
rlf! {
    // This is a dictionary entry — it has forms you can select from
    card = :a { one: "card", other: "cards" };

    // This is a template — it takes values and produces text
    draw(n) = "Draw {n} {card:n}.";
}
```

Calling both "phrase" obscures this difference. A term is *data* (a table of
forms). A phrase is a *function* (a template that evaluates with arguments).

### Already true in practice

Analysis of the English source file, Russian translation, and Spanish
translation (see appendices) shows that **every existing definition already
follows this rule**. No definition combines parameters with variant blocks:

| Category | Has Params | Has Variants | Examples |
|----------|-----------|-------------|----------|
| Terms | No | Yes | `card`, `character`, `ally`, `allied_adj`, `destroyed` |
| Terms (single-form) | No | No | `hello`, `this_character`, `dissolve` |
| Terms (tagged) | No | Optional | `card = :a "card"`, `card = :fem { ... }` |
| Phrases | Yes | No | `draw(n)`, `allied(entity)`, `abandon_n(n, target)` |

The distinction is implicit today. This plan makes it explicit and enforced.

### Better error messages

With the distinction formalized:

```
error: term 'card' cannot have parameters
  --> strings.rlf.rs:3:5
   |
   = help: terms have variants (forms), phrases have parameters
   = help: did you mean to define a phrase without variants?

error: phrase 'draw' cannot have variant block
  --> strings.rlf.rs:5:5
   |
   = help: phrases have parameters, terms have variants (forms)
   = help: did you mean to define a term without parameters?
```

### Foundation for future syntax work

The term/phrase distinction makes `name(x)` unambiguous in principle: if `name`
is a term, `(x)` would mean dynamic selection; if `name` is a phrase, `(x)` is
a call. This enables future syntax changes (like parenthesized dynamic values)
without introducing ambiguity. See `PLAN_PARENTHESIZED_DYNAMIC_VALUES.md`.

## Definition

### Term

A **term** is a definition without parameters. It can have:

- A single text body: `hello = "Hello, world!";`
- A text body with tags: `card = :a "card";`
- A variant block: `card = { one: "card", other: "cards" };`
- A variant block with tags: `card = :a { one: "card", other: "cards" };`

A term's body can contain interpolations that reference other terms:

```
all_cards = "All {card:other}.";
energy_symbol = "<e>●</e>";
draw_one = "Draw {@a card}.";
```

A term **cannot** have parameters declared in parentheses after its name.

### Phrase

A **phrase** is a definition with parameters. It has:

- A parameter list: `draw(n)`, `allied(entity)`, `abandon_n(n, target)`
- A template body: `"Draw {n} {card:n}."`
- Optional tags: `subtype(s) = :from(s) "..."`

A phrase's template can reference parameters and terms, apply transforms, and
use selectors:

```
draw(n) = "Draw {n} {card:n}.";
allied(entity) = "{allied_adj:entity} {entity:nom.one}";
destroy(thing) = "{thing} fue {destroyed:thing}.";
```

A phrase **cannot** have a variant block `{ key: "value", ... }` as its body.

### Summary Table

| | Term | Phrase |
|---|------|--------|
| Parameters | No | Yes (required) |
| Variant block | Yes (optional) | No |
| Tags | Yes | Yes |
| Template body | Yes | Yes |
| `:from` inheritance | No | Yes |
| Interpolations in body | Yes | Yes |

## Interaction with `:from`

The `:from(param)` modifier is only valid on phrases. It causes the phrase to
inherit tags and variants from a parameter **at runtime**:

```
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
```

This phrase has a parameter `s` but no statically declared variant block. The
variants are computed at evaluation time by evaluating the template once per
variant of `s`. The result is a `Phrase` value that carries inherited variants
and tags.

This is consistent with the rule: the definition has parameters and no variant
block. The fact that the evaluated *result* has variants is a runtime property,
not a definition-time property.

## Interaction with Numeric Selectors

The `PLAN_NUMERIC_SELECTORS.md` proposal included a pattern that combines
parameters with variants:

```
// This would be INVALID under the term/phrase rule:
item_count(n) = {
    0: "no items",
    1: "one item",
    other: "{n} items",
};
```

With the term/phrase distinction, this pattern must be expressed differently.
The recommended approach is to handle count branching in Rust:

```rust
match n {
    0 => strings::item_count_zero(locale),
    1 => strings::item_count_one(locale),
    n => strings::item_count_n(locale, n),
}
```

```
item_count_zero = "no items";
item_count_one = "one item";
item_count_n(n) = "{n} items";
```

This aligns with the "logic in Rust, text in RLF" philosophy. The branching
decision is logic; the text for each branch belongs in RLF.

Numeric variant keys on **terms** remain valid and useful:

```
// Valid: term with numeric variant keys
item_label = {
    0: "no items",
    1: "one item",
    one: "item",
    other: "items",
};
```

These can be selected statically (`{item_label:0}`) or dynamically
(`{item_label:n}` where `n` is a parameter of an enclosing phrase).

## Implementation Plan

### Phase 1: Documentation and Naming

**Files:**
- `docs/DESIGN.md`
- `CLAUDE.md`

Update documentation to use "term" and "phrase" terminology consistently:

- The Overview section should introduce both concepts
- The Primitives section should distinguish term definitions from phrase
  definitions
- Error message documentation should reference the correct concept

### Phase 2: AST Type Changes

**Files:**
- `crates/rlf-macros/src/input.rs`
- `crates/rlf/src/parser/ast.rs`

Introduce distinct AST types or a discriminant:

```rust
// In input.rs (macro)
pub enum Definition {
    /// A term: has variants, no parameters.
    Term(TermDefinition),
    /// A phrase: has parameters, no variants.
    Phrase(PhraseDefinition),
}

pub struct TermDefinition {
    pub name: SpannedIdent,
    pub tags: Vec<SpannedIdent>,
    pub body: TermBody,
}

pub enum TermBody {
    /// Single text: `hello = "Hello!";`
    Simple(Template),
    /// Variant block: `card = { one: "card", other: "cards" };`
    Variants(Vec<VariantEntry>),
}

pub struct PhraseDefinition {
    pub name: SpannedIdent,
    pub parameters: Vec<SpannedIdent>,
    pub tags: Vec<SpannedIdent>,
    pub from_param: Option<SpannedIdent>,
    pub body: Template,
}
```

The runtime AST in `ast.rs` should have a parallel structure.

### Phase 3: Parser Validation

**Files:**
- `crates/rlf-macros/src/parse.rs`
- `crates/rlf/src/parser/template.rs` (or phrase parser)

Add validation rules:

1. If a definition has parameters `()`, it must not have a variant block `{ }`
2. If a definition has a variant block `{ }`, it must not have parameters `()`
3. `:from(param)` is only valid on phrases (definitions with parameters)

**Macro parser** — emit compile-time errors:

```
error: cannot combine parameters with variant block
  --> strings.rlf.rs:3:5
   |
3  |     item_count(n) = { 0: "no items", other: "{n} items" };
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: terms have variants, phrases have parameters — not both
   = help: consider splitting into separate term and phrase definitions
```

**Runtime parser** — return `EvalError` at load time with a clear message.

### Phase 4: Evaluator Awareness

**Files:**
- `crates/rlf/src/interpreter/evaluator.rs`

The evaluator can use the term/phrase distinction for clearer error paths:

- When evaluating `{name:selector}` on a term, apply variant selection
- When evaluating `{name(args)}` on a phrase, perform a phrase call
- When evaluating `{name(args)}` on a term, perform dynamic selection
  (if/when parenthesized syntax is adopted — for now, this case doesn't arise)

Error messages should use the correct terminology:

```
error: term 'card' does not have variant 'accusative'
  available variants: one, other

error: phrase 'draw' expects 1 parameter, got 2
```

### Phase 5: Generated Code

**Files:**
- `crates/rlf-macros/src/codegen.rs` (or equivalent)

The macro generates different function signatures for terms vs phrases:

```rust
// Term: no parameters (aside from locale)
pub fn card(locale: &Locale) -> Phrase { ... }

// Phrase: has parameters
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }
```

This is already the current behavior — terms generate zero-parameter functions,
phrases generate parameterized functions. The change is making this distinction
explicit in the codegen logic rather than implicit.

### Phase 6: Validation Pass

**Files:**
- `crates/rlf/src/interpreter/validate.rs` (or equivalent)

Add a validation check for translation files: if the source language defines
`card` as a term, the translation must also define it as a term (not a phrase).
Similarly, if the source defines `draw` as a phrase with one parameter, the
translation's `draw` must also be a phrase with one parameter.

This catches mismatches between source and translation:

```
error: 'card' is a term in source language but a phrase in ru.rlf
  source: card = { one: "card", other: "cards" };
  ru.rlf: card(n) = "карта {n}";
```

### Phase 7: Test Updates

**Files:**
- `crates/rlf/tests/`
- `crates/rlf-macros/tests/`

Add tests for:

- Parsing a term (single-form, multi-form, with tags)
- Parsing a phrase (with parameters, with `:from`)
- Rejection of combined parameters + variants
- Error messages use correct terminology
- Source/translation definition kind mismatch detection
- Existing tests continue to pass (no behavioral change expected)

## Migration

This is a **non-breaking change** for existing code. Every existing definition
already follows the rule. The change adds:

1. Explicit enforcement of the implicit pattern
2. Clearer terminology in documentation and errors
3. Distinct AST representation

No existing `.rlf` files or `rlf!` invocations need modification.

## Error Handling

Clear, specific error messages are critical for this change. Users will encounter
these errors when they accidentally mix concepts or use the wrong syntax, and the
messages must guide them toward the correct approach.

### Definition-Site Errors

**Combining parameters and variants:**

```rust
rlf! {
    item_count(n) = { 0: "no items", other: "{n} items" };
}
```

```
error: phrase 'item_count' cannot have a variant block
  --> strings.rlf.rs:2:5
   |
2  |     item_count(n) = { 0: "no items", other: "{n} items" };
   |     ^^^^^^^^^^---   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |               |     |
   |               |     variant block not allowed on phrases
   |               parameters declared here
   |
   = help: definitions with parameters are phrases; definitions with
           variant blocks are terms — a definition cannot be both
   = help: consider splitting into a term and a phrase:
           item_count_label = { 0: "no items", other: "items" };
           item_count(n) = "{n} {item_count_label:n}";
```

**`:from` on a term:**

```rust
rlf! {
    formatted = :from(s) "<b>{s}</b>";
}
```

```
error: ':from' requires a parameter, but 'formatted' is a term
  --> strings.rlf.rs:2:17
   |
2  |     formatted = :from(s) "<b>{s}</b>";
   |                 ^^^^^^^^
   |
   = help: ':from' inherits variants from a parameter — add a parameter:
           formatted(s) = :from(s) "<b>{s}</b>";
```

### Usage-Site Errors (Compile-Time, Macro)

**Passing arguments to a term:**

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    bad(n) = "Draw {card(n)}.";
}
```

```
error: 'card' is a term and cannot be called with arguments
  --> strings.rlf.rs:3:18
   |
3  |     bad(n) = "Draw {card(n)}.";
   |                    ^^^^^^^^
   |
   = help: 'card' is a term with variants — select a variant instead:
           {card:n}     select variant using parameter n
           {card:other} select the 'other' variant
   = note: term 'card' defined here:
           card = :a { one: "card", other: "cards" };
```

**Selecting a variant from a phrase:**

```rust
rlf! {
    draw(n) = "Draw {n} cards.";
    bad = "{draw:other}";
}
```

```
error: 'draw' is a phrase and does not have variants
  --> strings.rlf.rs:3:11
   |
3  |     bad = "{draw:other}";
   |            ^^^^^^^^^^^
   |
   = help: 'draw' is a phrase with parameters — call it instead:
           {draw(1)}
   = note: phrase 'draw' defined here:
           draw(n) = "Draw {n} cards.";
```

**Wrong number of arguments to a phrase:**

```rust
rlf! {
    draw(n) = "Draw {n} cards.";
    bad = "{draw(1, 2)}";
}
```

```
error: phrase 'draw' expects 1 parameter, but 2 were provided
  --> strings.rlf.rs:3:11
   |
3  |     bad = "{draw(1, 2)}";
   |            ^^^^^^^^^^^
   |
   = note: phrase 'draw' defined here:
           draw(n) = "Draw {n} cards.";
```

### Usage-Site Errors (Runtime, Interpreter)

The same errors apply when loading `.rlf` translation files at runtime. These
are returned as `EvalError` variants rather than compile-time errors:

**Passing arguments to a term in a translation file:**

```
// ru.rlf
bad(n) = "Возьмите {card(n)}.";
```

```
EvalError::ArgumentsToTerm {
    term_name: "card",
    location: "ru.rlf:2",
    help: "'card' is a term with variants — use a selector like {card:n}",
}
```

**Translation mismatches source definition kind:**

```
// Source (English): card is a term
card = :a { one: "card", other: "cards" };

// ru.rlf: translator accidentally made it a phrase
card(n) = "карта {n}";
```

```
EvalError::DefinitionKindMismatch {
    name: "card",
    source_kind: "term",
    translation_kind: "phrase",
    location: "ru.rlf:1",
    help: "'card' is defined as a term in the source language but as a \
           phrase in ru.rlf — these must match",
}
```

### Error Variants

New `EvalError` variants to support these messages:

```rust
pub enum EvalError {
    // Existing variants...

    /// Attempted to pass arguments to a term.
    ArgumentsToTerm {
        name: String,
        help: String,
    },

    /// Attempted to select a variant from a phrase.
    SelectorOnPhrase {
        name: String,
        help: String,
    },

    /// A translation file defines a name as a different kind than source.
    DefinitionKindMismatch {
        name: String,
        source_kind: &'static str,
        translation_kind: &'static str,
    },

    /// ':from' used on a definition without parameters.
    FromOnTerm {
        name: String,
    },

    /// A definition has both parameters and a variant block.
    TermWithParameters {
        name: String,
    },
}
```

### Compile-Time Error Variants (Macro)

The macro should produce `syn::Error` with spans pointing to the relevant
syntax. Each error should include:

1. **What went wrong**: Clear statement of the violation
2. **Where**: Span highlighting the conflicting parts
3. **Why**: Brief explanation of the term/phrase distinction
4. **How to fix**: Concrete suggestion with example code
