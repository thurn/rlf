# RLF v2 Syntax Design

## Overview

RLF v2 introduces three syntax changes that eliminate all ambiguity from the
language:

1. **$ prefix**: Parameters are always marked with `$`. Bare names always refer
   to definitions.
2. **`: static`, `(): dynamic`**: The `:` operator is exclusively for
   static/literal values. Parentheses `()` are used for dynamic/parameter-based
   values.
3. **Numeric variant keys**: Variant blocks can use numeric keys (`0`, `1`, `2`)
   alongside CLDR category keys. Dynamic selection tries exact numbers first.

Together, these changes make every reference in a template body unambiguous from
syntax alone — no implicit resolution rules needed.

---

## Definitions

RLF has one kind of definition. A definition has a name and can optionally have
**parameters**, **tags**, **variants**, and **`:from` inheritance** — in any
combination.

```
// Simple text
hello = "Hello, world!";

// With tags
card = :a "card";

// With variants
card = :a { one: "card", other: "cards" };

// With parameters
draw($n) = "Draw {$n} {card($n)}.";

// With parameters and variants
cards($n) = :match($n) {
    1: "a card",
    other: "{$n} cards",
};

// With parameters, tags, and :from
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

### Variant blocks

A variant block provides multiple forms keyed by name or number:

```
card = { one: "card", other: "cards" };

go = { present: "go", past: "went", participle: "gone" };

multiplier = {
    1: "single",
    2: "double",
    3: "triple",
    other: "multiple",
};
```

Multi-dimensional variants use dot notation:

```
// Russian
card = :fem {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};
```

Multi-key shorthand assigns the same value to multiple keys:

```
card = {
    nom.one, acc.one: "card",
    nom.other, acc.other: "cards",
};
```

Wildcard fallbacks omit the final dimension:

```
card = {
    nom: "карта",        // Fallback for nom.*
    nom.many: "карт",    // Override for nom.many
};
```

### Definitions with parameters and variants

When a definition has both parameters and a variant block, the `:match($param)`
modifier specifies which parameter selects the variant. Each branch is a
template with access to all parameters:

```
cards($n) = :match($n) {
    1: "a card",
    other: "{$n} cards",
};
// cards(1) → "a card"
// cards(3) → "3 cards"

count_allied_subtype($n, $s) = :match($n) {
    1: "an allied {subtype($s)}",
    other: "{$n} allied {subtype($s):other}",
};
// count_allied_subtype(1, warrior) → "an allied <b>Warrior</b>"
// count_allied_subtype(3, warrior) → "3 allied <b>Warriors</b>"
```

`:match($n)` works like `:from($s)` — both are modifiers that specify how a
parameter relates to the definition's variant structure. The resolution order
follows the same rules as [dynamic selection](#dynamic-resolution-order): exact
numeric key first, then CLDR plural category.

Definitions without `:match` expose their variants for external selection.
Definitions with `:match` select internally and return resolved text.

A variant block without `:match` is an error if the definition has parameters —
either use `:match($param)` to specify the selector, or remove the parameters.

---

## The $ Prefix

All parameter references use `$`, everywhere. Bare names always refer to
definitions. This follows the convention from
[Mozilla Fluent](https://projectfluent.org/).

### In declarations

```
draw($n) = "Draw {$n} {card($n)}.";
subtype($s) = :from($s) "<b>{$s}</b>";
```

### In template bodies

| Syntax | Meaning |
|--------|---------|
| `{$name}` | Interpolate parameter `$name` |
| `{card}` | Reference definition `card` (default form) |
| `{Card}` | Reference definition `card` with `@cap` |
| `{energy($e)}` | Call `energy` with parameter `$e` |
| `{subtype(ancient)}` | Call `subtype` passing definition `ancient` |

### In selectors

| Syntax | Meaning |
|--------|---------|
| `{card:other}` | Static: literal variant key `other` |
| `{card:3}` | Static: literal variant key `3` |
| `{card($n)}` | Dynamic: select using parameter `$n` |
| `{card:acc($n)}` | Mixed: static `acc`, dynamic `$n` |
| `{card:acc:one}` | Fully static: literal `acc` and `one` |
| `{destroyed($thing)}` | Dynamic: select by `$thing`'s tags |

### In transform context

| Syntax | Meaning |
|--------|---------|
| `{@der:acc karte}` | Static context: literal `acc` |
| `{@count($n) card}` | Dynamic context: parameter `$n` |
| `{@el:other $t}` | Static context, applied to parameter |

---

## Static vs Dynamic: `:` and `()`

The `:` operator is exclusively for static/literal values. Parentheses `()` are
for dynamic/parameter-based values.

| Operation | Static (literal) | Dynamic (parameter) |
|-----------|------------------|---------------------|
| Variant selection | `{card:other}` | `{card($n)}` |
| Numeric selection | `{card:2}` | `{card($n)}` where n=2 |
| Multi-dimensional | `{card:acc:one}` | `{card:acc($n)}` |
| Transform context | `{@der:acc karte}` | `{@count($n) card}` |

### Rules

- After `:` — only literal identifiers or numbers. Never `$`-prefixed names.
- Inside `()` for selection — exactly one `$`-prefixed parameter.
- Dynamic `()` must be the **last** selector: `{card:acc($n)}` is valid,
  `{card($n):acc}` is not.
- A reference can have zero or more `:literal` selectors followed by at most
  one `($param)` selector.

### How `name(...)` is resolved

When the evaluator sees `{name(...)}`:

- If `name` has **declared parameters** — it's a **call** (bind arguments,
  evaluate body)
- If `name` has **no parameters** — it's **dynamic selection** on variants

| Expression | `name` has no params | `name` has params |
|------------|---------------------|-------------------|
| `{name($x)}` | Dynamic selection | Call with param |
| `{name(other_def)}` | — | Call with definition arg |
| `{name($x, $y)}` | Error (one arg max) | Call with two params |

Call arguments use `$` for parameters and bare names for definition references:

```
{energy($e)}         → call 'energy', pass parameter $e
{subtype(ancient)}   → call 'subtype', pass definition 'ancient'
{subtype($s)}        → call 'subtype', pass parameter $s
```

---

## Numeric Variant Keys

Variant blocks can use numeric literals as keys alongside CLDR category names:

```
card = {
    0: "no cards",
    1: "a card",
    2: "a pair of cards",
    one: "card",
    other: "cards",
};
```

Numeric keys and CLDR category keys coexist in the same variant map. They are
syntactically disjoint — CLDR categories are alphabetic (`one`, `other`), numeric
keys are digits (`0`, `1`, `2`). No prefix like ICU MessageFormat's `=0` is
needed.

### Static numeric selection

Use `:` with a number to select a specific numeric variant:

```
zero_cards = "You have {card:0}.";    // → "You have no cards."
one_card = "You have {card:1}.";      // → "You have a card."
pair = "You have {card:2}.";          // → "You have a pair of cards."
```

### Dynamic resolution order

When `{card($n)}` is evaluated with a numeric parameter:

1. **Exact numeric key**: Try the literal number (n=0 → key `"0"`)
2. **CLDR plural category**: Map through plural rules (n=3 → `"other"`)
3. **Fallback chain**: Dot-separated fallback for multi-dimensional keys

```
card = {
    0: "no cards",
    1: "a card",
    one: "card",
    other: "cards",
};
draw($n) = "Draw {card($n)}.";

// n=0 → try "0" ✓ → "no cards"       (exact match wins)
// n=1 → try "1" ✓ → "a card"         (exact match wins over CLDR "one")
// n=2 → try "2" ✗ → CLDR "other" ✓   → "cards"
// n=5 → try "5" ✗ → CLDR "other" ✓   → "cards"
```

This matches ICU MessageFormat's precedence where exact values override plural
categories.

### Multi-dimensional numeric keys

Numeric keys compose with dot notation:

```
// Russian with zero override
card = {
    0: "нет карт",
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};
draw($n) = "Возьмите {card:acc($n)}.";

// n=0  → try "acc.0" ✗ → try "0" ✓ → "нет карт"  (fallback to shorter key)
// n=1  → try "acc.1" ✗ → CLDR "acc.one" ✓ → "карту"
// n=5  → try "acc.5" ✗ → CLDR "acc.many" ✓ → "карт"
```

### Restrictions

- **No negative numbers**: `-1` is not supported as a variant key.
- **No floats**: `1.5` is not supported — would conflict with dot notation.

---

## Transforms

The `@` operator applies a transform. Transforms modify text and apply
right-to-left when chained:

```
card = "card";
draw_one = "Draw {@a card}.";        // → "Draw a card."
title = "{@cap card}";               // → "Card"
heading = "{@cap @a card}";          // → "A card"
```

**Automatic capitalization:** `{Card}` is equivalent to `{@cap card}`. This
works for definition references only. For parameters, use `{@cap $name}`
explicitly.

### Transform context

Transforms can take context — static with `:`, dynamic with `()`:

```
{@transform ref}                 No context
{@transform:literal ref}        Static context
{@transform($param) ref}        Dynamic context
```

A transform has either `:literal` or `($param)` context, not both.

Examples:

```
// German — static context
destroy_card = "Zerstöre {@der:acc karte}.";

// Chinese — dynamic context
draw($n) = "抽{@count($n) card}";

// Spanish — static context for plural article form
return_all($t) = "devuelve {@el:other $t} a la mano";

// German — static context + static selection on definition
get_card = "Nimm {@der:acc karte:one}.";
```

### Transforms with selection

Transforms combine with selection:

```
card = { one: "card", other: "cards" };
draw($n) = "Draw {$n} {@cap card($n)}.";
// n=1 → "Draw 1 Card."
// n=3 → "Draw 3 Cards."
```

### Universal transforms

| Transform | Effect |
|-----------|--------|
| `@cap` | Capitalize first letter |
| `@upper` | All uppercase |
| `@lower` | All lowercase |
| `@plural` | Select the `other` variant |

### Metadata-driven transforms

| Transform | Languages | Reads Tags | Effect |
|-----------|-----------|------------|--------|
| `@a` | English | `:a`, `:an` | Indefinite article |
| `@the` | English | — | Definite article |
| `@der` | German | `:masc`, `:fem`, `:neut` | Definite article + case |
| `@ein` | German | `:masc`, `:fem`, `:neut` | Indefinite article + case |
| `@el` | Spanish | `:masc`, `:fem` | Definite article |
| `@le` | French | `:masc`, `:fem`, `:vowel` | Definite article |
| `@un` | Spanish, French, Italian | `:masc`, `:fem` | Indefinite article |
| `@o` | Portuguese, Greek | `:masc`, `:fem` | Definite article |
| `@count` | CJK, Vietnamese | measure word tags | Classifier |
| `@inflect` | Turkish, Finnish, Hungarian | vowel harmony tags | Suffix chain |

---

## Metadata Tags

A definition can declare metadata tags using `:` before its content:

```
card = :a "card";
event = :an "event";

// Spanish
carta = :fem "carta";
personaje = :masc "personaje";
```

Tags serve two purposes:
1. **Selection**: `{destroyed($thing)}` reads tags from `$thing` to select a
   variant
2. **Transforms**: `@a` reads `:a`/`:an` to choose article form

### Tag-based selection

```
// Spanish
card = :fem "carta";
character = :masc "personaje";

destroyed = {
    masc: "destruido",
    fem: "destruida",
};

destroy($thing) = "{$thing} fue {destroyed($thing)}.";
// thing=card      → "carta fue destruida."
// thing=character → "personaje fue destruido."
```

### :from inheritance

The `:from($param)` modifier causes a definition to inherit tags and variants
from a parameter at runtime:

```
ancient = :an { one: "Ancient", other: "Ancients" };
child = :a { one: "Child", other: "Children" };

subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

Evaluation of `subtype(ancient)`:

| Step | Action | Result |
|------|--------|--------|
| 1 | Read `ancient.tags` | `["an"]` |
| 2 | Evaluate with `$s:one` | `"<color=#2E7D32><b>Ancient</b></color>"` |
| 3 | Evaluate with `$s:other` | `"<color=#2E7D32><b>Ancients</b></color>"` |
| 4 | Return Phrase | Tags: `["an"]`, variants: `{one: ..., other: ...}` |

Usage:

```
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";
// dissolve_subtype(ancient) → "Dissolve an <b>Ancient</b>."
// dissolve_all(ancient)     → "Dissolve all <b>Ancients</b>."
```

---

## Escape Sequences

| Sequence | Output | Purpose |
|----------|--------|---------|
| `{{` | `{` | Literal brace |
| `}}` | `}` | Literal brace |
| `::` | `:` | Literal colon |
| `@@` | `@` | Literal at sign |
| `$$` | `$` | Literal dollar sign |

```
syntax_help = "Use {{$$name}} for parameters and @@ for transforms.";
ratio = "The ratio is 1::2.";
price = "The cost is $$5.";
```

---

## Generated Rust API

Given:

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    draw($n) = "Draw {$n} {card($n)}.";
    cards($n) = :match($n) {
        1: "a card",
        other: "{$n} cards",
    };
}
```

RLF generates:

```rust
/// Returns the "card" definition.
pub fn card(locale: &Locale) -> Phrase { ... }

/// Evaluates the "draw" definition with parameter n.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }

/// Evaluates the "cards" definition with parameter n.
pub fn cards(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }
```

All definitions return `Phrase`. Parameter names in generated functions drop the
`$` prefix.

### PhraseId constants

The macro also generates `PhraseId` constants:

```rust
pub mod phrase_ids {
    pub const CARD: PhraseId = ...;
    pub const DRAW: PhraseId = ...;
    pub const CARDS: PhraseId = ...;
}
```

---

## File Structure

```
src/
  localization/
    mod.rs
    strings.rlf.rs     # Source language — uses rlf!
  assets/
    localization/
      ru.rlf           # Russian — loaded at runtime
      es.rlf           # Spanish — loaded at runtime
```

Translation files use the same v2 syntax:

```
// ru.rlf
card = :fem {
    0: "нет карт",
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};

draw($n) = "Возьмите {$n} {card:acc($n)}.";

cards($n) = :match($n) {
    1: "карту",
    few: "{$n} карты",
    other: "{$n} карт",
};
```

---

## Complete Example: Game Localization

Based on real-world usage in the Dreamtides card game.

### English source

```rust
rlf! {
    // =====================================================
    // Symbols and keywords
    // =====================================================

    energy_symbol = "<color=#00838F>\u{25CF}</color>";
    points_symbol = "<color=#F57F17>\u{234F}</color>";

    dissolve = "<color=#AA00FF>dissolve</color>";
    banish = "<color=#AA00FF>banish</color>";
    prevent = "<color=#AA00FF>prevent</color>";
    reclaim = "<color=#AA00FF>reclaim</color>";

    // =====================================================
    // Nouns with article metadata and plural forms
    // =====================================================

    card = :a { one: "card", other: "cards" };
    character = :a { one: "character", other: "characters" };
    ally = :an { one: "ally", other: "allies" };

    // Pronoun agreement
    it_or_them = { one: "it", other: "them" };

    // Character subtypes
    ancient = :an { one: "Ancient", other: "Ancients" };
    warrior = :a { one: "Warrior", other: "Warriors" };
    explorer = :an { one: "Explorer", other: "Explorers" };

    // =====================================================
    // Parameterized definitions (template body)
    // =====================================================

    // Energy amount with colored symbol
    energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";

    // Keyword with value
    kindle($k) = "<color=#AA00FF>kindle</color> {$k}";
    foresee($n) = "<color=#AA00FF>foresee</color> {$n}";

    // Subtype display with inherited metadata
    subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";

    // Composing definitions
    dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
    dissolve_all($s) = "Dissolve all {subtype($s):other}.";

    // Dynamic entity selection
    draw_entities($n, $entity) = "Draw {$n} {$entity($n)}.";

    // Definition referencing another definition
    pay_energy_button($e) = "Spend {energy($e)}";

    // =====================================================
    // Parameterized definitions (variant body)
    // =====================================================

    // "a card" / "3 cards"
    cards($n) = :match($n) {
        1: "a card",
        other: "{$n} cards",
    };

    // "top card" / "top 3 cards"
    top_n_cards($n) = :match($n) {
        1: "top card",
        other: "top {$n} cards",
    };

    // "one" / "3" (word for 1, numeral otherwise)
    text_number($n) = :match($n) {
        1: "one",
        other: "{$n}",
    };

    // "a copy" / "two copies"
    copies($n) = :match($n) {
        1: "a copy",
        other: "{text_number($n)} copies",
    };

    // "an ally" / "3 allies"
    count_allies($n) = :match($n) {
        1: "an ally",
        other: "{$n} allies",
    };

    // "an allied <Warrior>" / "3 allied <Warriors>"
    count_allied_subtype($n, $s) = :match($n) {
        1: "an allied {subtype($s)}",
        other: "{$n} allied {subtype($s):other}",
    };

    // "a <Celestial> Figment" / "two <Shadow> Figments"
    n_figments($n, $f) = :match($n) {
        1: "a {figment($f)}",
        other: "{text_number($n)} {figments_plural($f)}",
    };

    // "this turn" / "this turn two times"
    this_turn_times($n) = :match($n) {
        1: "this turn",
        other: "this turn {text_number($n)} times",
    };
}
```

### Usage in Rust

```rust
let locale = Locale::with_language("en");

strings::energy(&locale, 3);                 // → "<color=#00838F>3●</color>"
strings::cards(&locale, 1);                  // → "a card"
strings::cards(&locale, 3);                  // → "3 cards"
strings::top_n_cards(&locale, 1);            // → "top card"
strings::top_n_cards(&locale, 5);            // → "top 5 cards"
strings::copies(&locale, 1);                 // → "a copy"
strings::copies(&locale, 2);                 // → "two copies"

let ancient = strings::ancient(&locale);
strings::dissolve_subtype(&locale, ancient); // → "Dissolve an <b>Ancient</b>."
strings::dissolve_all(&locale, ancient);     // → "Dissolve all <b>Ancients</b>."

let warrior = strings::warrior(&locale);
strings::count_allied_subtype(&locale, 1, warrior);  // → "an allied <b>Warrior</b>"
strings::count_allied_subtype(&locale, 3, warrior);  // → "3 allied <b>Warriors</b>"
```

### Spanish translation

```
// es.rlf
card = :fem "carta";
character = :masc "personaje";

destroyed = {
    masc: "destruido",
    fem: "destruida",
};

destroy($thing) = "{$thing} fue {destroyed($thing)}.";
// $thing=card      → "carta fue destruida."
// $thing=character → "personaje fue destruido."

abandon_one($target) = "abandona {@un $target}";
return_all($target) = "devuelve {@el:other $target} a la mano";
```

### Russian translation

```
// ru.rlf
card = :fem {
    0: "нет карт",
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};

draw($n) = "Возьмите {$n} {card:acc($n)}.";

// Russian needs more plural branches than English
cards($n) = :match($n) {
    1: "карту",
    few: "{$n} карты",
    other: "{$n} карт",
};

inventory($n) = :match($n) {
    0: "У вас нет предметов.",
    1: "У вас один предмет.",
    few: "У вас {$n} предмета.",
    other: "У вас {$n} предметов.",
};
```

### Chinese translation

```
// zh_cn.rlf
card = :zhang "牌";
character = :ge "角色";

draw($n) = "抽{@count($n) card}";
// $n=3 → "抽3张牌"
```

### German translation

```
// de.rlf
karte = :fem "Karte";
charakter = :masc "Charakter";

destroy_card = "Zerstöre {@der:acc karte}.";
get_card = "Nimm {@der:acc karte:one}.";
```

---

## Disambiguation Summary

With `$` and `()` vs `:`, every reference is unambiguous from syntax alone:

| Syntax | Meaning | How it's clear |
|--------|---------|----------------|
| `{$n}` | Parameter interpolation | `$` prefix |
| `{card}` | Definition reference | No `$`, no `()` |
| `{Card}` | Definition + `@cap` | No `$`, uppercase |
| `{energy($e)}` | Call with param | `$` inside `()` |
| `{subtype(ancient)}` | Call with definition arg | Bare name inside `()` |
| `{card:other}` | Static variant selection | No `$` after `:` |
| `{card:3}` | Static numeric selection | Numeric literal after `:` |
| `{card($n)}` | Dynamic variant selection | `$` inside `()` |
| `{card:acc($n)}` | Mixed static + dynamic | `:` static, `()` dynamic |
| `{$entity($n)}` | Dynamic selection on param | `$` on both |
| `{@a $target}` | Transform on parameter | `$` prefix |
| `{@a card}` | Transform on definition | No `$` |
| `{@der:acc karte}` | Transform with static ctx | No `$` after `:` |
| `{@count($n) card}` | Transform with dynamic ctx | `$` inside `()` |

---

## v1 → v2 Migration

| Syntax | v1 | v2 |
|--------|----|----|
| Parameter declaration | `draw(n)` | `draw($n)` |
| Parameter interpolation | `{n}` | `{$n}` |
| Dynamic selection | `{card:n}` | `{card($n)}` |
| Mixed selection | `{card:acc:n}` | `{card:acc($n)}` |
| Dynamic transform ctx | `{@count:n card}` | `{@count($n) card}` |
| Call argument | `{energy(e)}` | `{energy($e)}` |
| Tag-based selection | `{destroyed:thing}` | `{destroyed($thing)}` |
| Static selection | `{card:other}` | `{card:other}` (unchanged) |
| Static transform ctx | `{@der:acc karte}` | `{@der:acc karte}` (unchanged) |
| Numeric variant key | N/A | `0: "no cards"` (new) |
| Escape `$` | N/A | `$$` (new) |

---

## Design Principles

**One definition primitive.** Definitions can have any combination of
parameters, variants, and tags. These features compose naturally — no need
for separate "term" and "phrase" categories.

**$ means parameter.** Every `$` is a parameter. Every bare name is a
definition. No lookup rules needed — the syntax is self-describing.

**`: static`, `(): dynamic`.** After `:`, only literals. Inside `()`, only
parameters. No ambiguity even when a parameter name matches a variant key.

**Exact numbers first.** Dynamic selection tries `"3"` before CLDR `"other"`.
This matches ICU MessageFormat precedence and user expectations.

**Logic in Rust, text in RLF.** Complex branching stays in Rust. Parameterized
variant blocks handle the common case of count-dependent text.

**Pass Phrase, not String.** When composing definitions, pass `Phrase` values to
preserve variants and tags for downstream selection and transforms.
