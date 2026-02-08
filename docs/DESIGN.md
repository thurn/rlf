# RLF

The Rust Localization Framework: a localization DSL embedded in Rust via macros.

## Overview

RLF generates a **language-agnostic API** from phrase definitions. The source
language (typically English) is compiled via the `rlf!` macro into Rust
functions. All other languages are loaded at runtime via the interpreter.

```rust
// strings.rlf.rs - The source language (English)
rlf! {
    hello = "Hello, world!";
    cards($n) = :match($n) {
        1: "a card",
        *other: "{$n} cards",
    };
    draw($n) = "Draw {cards($n)}.";
}
```

This generates functions that take a `Locale` parameter:

```rust
// Generated API - usage
let mut locale = Locale::with_language("en");
strings::hello(&locale);       // -> "Hello, world!"
strings::draw(&locale, 3);     // -> "Draw 3 cards."

// Switch to Russian
locale.set_language("ru");
strings::draw(&locale, 3);     // -> "Возьмите 3 карты." (via interpreter)
```

**How it works:**

1. The `rlf!` macro parses the source language and generates one function per definition
2. The macro also embeds the source definitions as data for the interpreter
3. At startup, the source definitions are registered with the interpreter
4. All evaluation (source and translations) goes through the interpreter

**Key benefit:** When you add a new definition to `strings.rlf.rs`, it
immediately appears in IDE autocomplete for all Rust code. No build steps, no
external tools -- just write the definition and use it.

**Two syntax guarantees** eliminate ambiguity:

1. **`$` prefix**: Parameters are always marked with `$`. Bare names always
   refer to definitions.
2. **Terms and phrases**: Definitions are divided into **terms** (no parameters,
   selected with `:`) and **phrases** (with parameters, called with `()`). It
   is an error to use `:` on a phrase name or `()` on a term name.

Together, these make every reference in a template body unambiguous from syntax
alone -- no implicit resolution rules needed.

---

## Terms

A **term** is a definition without parameters. Terms can have tags and/or
variants.

```
// Simple text
hello = "Hello, world!";

// With a tag
card = :a "card";

// With tags and variants
card = :a { one: "card", other: "cards" };

// Irregular forms
go = { present: "go", past: "went", participle: "gone" };
```

Terms are referenced in templates by name. A bare term reference (no `:` selector)
returns the **default variant**: the variant marked with `*`, or the first
declared variant if no `*` is present. For a simple string term, the string
itself is the only variant.

The `:` operator selects a specific variant:

```
all_cards = "All {card:other}.";     // -> "All cards."
title = "{@cap card}";               // -> "Card"       (default = "one")
```

### Variant blocks

A variant block provides multiple forms keyed by name:

```
card = { one: "card", other: "cards" };
go = { present: "go", past: "went", participle: "gone" };
```

One variant can be marked with `*` as the default. A bare reference to the term
(no `:` selector) returns the `*`-marked variant. If no `*` is present, the
first declared variant is the default:

```
card = { *one: "card", other: "cards" };
example = "{card}";                       // -> "card" (*one is default)
```

The `*` marker can only appear on top-level variant keys (e.g., `*one`), not on
multi-dimensional keys (e.g., `*nom.one` is not valid). A bare reference to a
term with only multi-dimensional variant keys (no `:` selector) is a
compile-time error -- always use an explicit selector.

Variant keys are always named identifiers. Numeric keys are not supported in
term variant blocks -- use `:match` in a phrase for numeric branching.

Multi-dimensional variants use dot notation in variant block keys. In template
selection expressions, the `:` operator chains dimensions with successive `:`
separators. Dots are for declaration; colons are for selection.

```
// Declaration -- dot notation in variant keys
card = :fem {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};

// Selection -- colon chains in template expressions
example = "{card:acc:one}";        // -> "карту" (selects acc.one)
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

Multi-key shorthand and wildcards can be combined:

```
card = {
    nom, acc: "карты",     // Fallback for nom.* and acc.*
    nom.many: "карт",      // Override for nom.many
};
```

**Wildcard resolution.** When selecting a multi-dimensional variant, resolution
uses most-specific-wins:

1. **Exact match**: Try the full key (e.g., `nom.one`)
2. **Wildcard fallback**: Try the prefix key (e.g., `nom`)
3. **Error**: If neither matches, this is a runtime error (see
   [Selection Errors](#selection-errors))

### Parameterized selection on terms

The `:` operator can use a `$`-prefixed parameter to select a variant
dynamically. When the parameter is a number, it maps through CLDR plural rules.
When the parameter is a Phrase, it reads the Phrase's tags:

```
// Select variant based on numeric parameter (CLDR mapping)
cards_numeral($n) = "{$n} {card:$n}";
// n=1 -> "1 card"   (CLDR maps 1 -> "one")
// n=5 -> "5 cards"  (CLDR maps 5 -> "other")

// Select variant based on a Phrase parameter's tags
allied_adj = { masc: "allied", fem: "allied", neut: "allied" };
modified($entity) = "{allied_adj:$entity} {$entity:one}";
```

Note: parameterized selection on a term (`{card:$n}`) maps numbers through
CLDR plural categories only. It does **not** try exact numeric keys -- use
`:match` in a phrase for exact-number matching. Literal numeric selection on
terms (`{card:3}`) is not supported.

---

## Phrases

A **phrase** is a definition with one or more parameters. A definition with
an empty parameter list (`name() = ...`) is a syntax error -- use a term
instead. Phrases use two keywords to control how parameters affect the output:

- **`:match($param)`** -- branches on a parameter value, similar to Rust's
  `match`
- **`:from($param)`** -- inherits tags and variants from a parameter

```
// Simple template
energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";

// With :match -- branches on $n
cards($n) = :match($n) {
    1: "a card",
    *other: "{$n} cards",
};

// With :from -- inherits metadata from $s
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

Phrases are called with `()` in template bodies:

```
draw($n) = "Draw {cards($n)}.";
pay($e) = "Spend {energy($e)}";
dissolve($s) = "Dissolve {@a subtype($s)}.";
```

Phrases accept literal values as arguments:

```
pair = "You have {cards(2)}.";       // -> "You have 2 cards."
```

### The `:match` keyword

`:match` branches on a parameter value, similar to Rust's `match`. Each branch
is a template with access to all parameters. One branch must be marked with `*`
as the default fallback (following
[Fluent's default variant](https://projectfluent.org/) convention):

```
cards($n) = :match($n) {
    1: "a card",
    *other: "{$n} cards",
};
// cards(1) -> "a card"
// cards(3) -> "3 cards"
```

**Numeric matching.** When the matched parameter is a number, resolution
follows this order:

1. **Exact numeric key**: Try the literal number (e.g., `0`, `1`, `2`)
2. **CLDR plural category**: Map through locale plural rules (`zero`, `one`,
   `two`, `few`, `many`, `other`)
3. **Default branch**: The `*`-marked fallback

```
cards($n) = :match($n) {
    0: "no cards",
    1: "a card",
    2: "a pair of cards",
    *other: "{$n} cards",
};
// n=0 -> "no cards"        (exact match)
// n=1 -> "a card"          (exact match)
// n=2 -> "a pair of cards" (exact match)
// n=5 -> "5 cards"         (CLDR "other" -> default)
```

This matches ICU MessageFormat's precedence where exact values override plural
categories. Exact numeric keys and CLDR-based resolution are exclusive to
`:match` -- parameterized selection on terms (`{card:$n}`) only uses CLDR.

**Tag-based matching.** When the matched parameter is a `Phrase` value,
`:match` reads its tags and selects the **first tag that matches** a branch key:

```
// Spanish
card = :fem "carta";
character = :masc "personaje";

destroyed($thing) = :match($thing) {
    masc: "destruido",
    *fem: "destruida",
};
// destroyed(card)      -> "destruida"  (card has tag :fem -> matches "fem")
// destroyed(character) -> "destruido"  (character has tag :masc -> matches "masc")
```

Resolution for tag-based matching:

1. Read tags from the parameter's Phrase value (e.g., `[:fem, :inan]`)
2. Iterate through tags in declaration order
3. Return the **first** tag that matches a branch key
4. If no tag matches, use the `*`-marked default branch

This means tag order matters when a Phrase has multiple tags. For example, if a
term is declared `:masc :anim`, `:match` tries `masc` first, then `anim`.

**Matching on multiple parameters.** `:match` can accept multiple parameters,
producing a multi-dimensional variant block using dot notation for branch keys.
Each dimension matches independently using the same rules (exact numeric, then
CLDR, then tags, then default). Exactly one value per dimension must be marked
with `*` as the default for that dimension:

```
// Russian -- branch on both count ($n) and gender ($entity)
n_allied($n, $entity) = :match($n, $entity) {
    1.masc: "союзный {$entity:nom:one}",
    1.fem: "союзная {$entity:nom:one}",
    1.*neut: "союзное {$entity:nom:one}",
    *other.masc: "{$n} союзных {$entity:gen:many}",
    other.fem: "{$n} союзных {$entity:gen:many}",
    other.*neut: "{$n} союзных {$entity:gen:many}",
};
```

Resolution for `:match($n, $entity)` with `$n=3, $entity=card` (tagged `:fem`):
1. Match `$n=3`: exact `3` (miss) -> CLDR `other` (hit)
2. Match `$entity=card`: tags `[:fem]` -> first match `fem` (hit)
3. Select branch `other.fem` -> `"{$n} союзных {$entity:gen:many}"`

Each dimension in a multi-match uses the same resolution algorithm as a
single-parameter match. Wildcard fallbacks work on intermediate dimensions just
like in variant blocks -- `other` matches `other.*` if no more-specific key
matches.

**Match restrictions:**

- A phrase with a variant block **must** use `:match` to specify the selector
  parameter(s).
- A phrase without `:match` has a simple template body (no variants).
- For single-parameter `:match`, exactly one branch must be marked with `*`.
  For multi-parameter `:match`, exactly one value per dimension must be marked
  with `*` (e.g., `*other` for dimension 1 and `*neut` for dimension 2).
- Match branches support both numeric keys (`0`, `1`, `2`) and named keys
  (`one`, `other`, `masc`).
- No negative numbers or floats as match keys.

### The `:from` keyword

`:from($param)` causes a phrase to inherit tags and variants from a parameter at
runtime. This enables phrase-returning phrases that transform one phrase into
another while preserving grammatical metadata.

```
ancient = :an { one: "Ancient", other: "Ancients" };
child = :a { one: "Child", other: "Children" };

subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

Evaluation of `subtype(ancient)`:

| Step | Action | Result |
|------|--------|--------|
| 1 | Read `ancient.tags` | `["an"]` |
| 2 | Evaluate with `$s` = one | `"<color=#2E7D32><b>Ancient</b></color>"` |
| 3 | Evaluate with `$s` = other | `"<color=#2E7D32><b>Ancients</b></color>"` |
| 4 | Return Phrase | Tags: `["an"]`, variants: `{one: ..., other: ...}` |

A bare reference to a `:from` phrase result (no `:` selector) returns its
default variant, inherited from the source parameter's `*`-marked or first
variant -- the same default rules as terms.

Usage:

```
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";
// dissolve_subtype(ancient) -> "Dissolve an <b>Ancient</b>."
// dissolve_all(ancient)     -> "Dissolve all <b>Ancients</b>."
```

**`:from` and variant selection.** A `:from` phrase returns a Phrase with a
full variant table. The `:` operator can select variants from a phrase call
result, just like it can from a term or parameter:

```
// English
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";

// Russian -- select case and number from the inherited variants
dissolve_subtype($s) = "Растворите {subtype($s):acc:one}.";
all_subtypes($s) = "все {subtype($s):nom:other}";
```

### Combining `:from` and `:match`

A phrase can use both `:from` and `:match` together. `:from` determines the
inherited tags and variant structure, while `:match` branches on a separate
parameter within each inherited variant's evaluation:

```
count_allied_subtype($n, $s) = :from($s) :match($n) {
    1: "союзный {subtype($s)}",
    *other: "{$n} союзных {subtype($s):gen:many}",
};
```

The order of `:from` and `:match` in the declaration does not matter --
`:from($s) :match($n)` and `:match($n) :from($s)` are equivalent.

---

## The `$` Prefix

All parameter references use `$`, everywhere. Bare names always refer to terms
or phrases. This follows the convention from
[Mozilla Fluent](https://projectfluent.org/).

### In declarations

```
draw($n) = "Draw {$n} {cards($n)}.";
subtype($s) = :from($s) "<b>{$s}</b>";
```

### In template bodies

| Syntax | Meaning |
|--------|---------|
| `{$name}` | Interpolate parameter `$name` |
| `{card}` | Reference term `card` (default variant) |
| `{Card}` | Reference term `card` with `@cap` |
| `{energy($e)}` | Call phrase `energy` with parameter `$e` |
| `{subtype(ancient)}` | Call phrase `subtype` passing term `ancient` |
| `{cards(2)}` | Call phrase `cards` with literal number `2` |

### Selection with `:`

| Syntax | Meaning |
|--------|---------|
| `{card:other}` | Static: select variant `other` from term `card` |
| `{card:acc:one}` | Static: select `acc.one` from term `card` |
| `{card:$n}` | Parameterized: select from term using `$n` (CLDR) |
| `{card:acc:$n}` | Mixed: static first dim, parameterized second dim |
| `{$base:nom:one}` | Static: select `nom.one` from parameter's Phrase |
| `{$entity:one}` | Static: select `one` from parameter's Phrase |
| `{allied_adj:$entity}` | Parameterized: select from term using `$entity`'s tags |
| `{subtype($s):other}` | Static: select `other` from phrase call result |
| `{subtype($s):acc:one}` | Static: multi-dim select from phrase call result |
| `{subtype($s):$n}` | Parameterized: select from phrase call result using `$n` |

### In transform context

| Syntax | Meaning |
|--------|---------|
| `{@der:acc card}` | Static context: literal `acc` |
| `{@count($n) card}` | Dynamic context: parameter `$n` |

---

## The `:` Selection Operator

The `:` operator selects variants. It works on term names, parameter references,
phrase call results, and transform context. It does **not** work on bare phrase
names (without `()`).

### On terms -- static keys

```
{card:other}            // -> "cards"
{card:acc:one}          // -> "карту" (Russian)
```

### On terms -- parameterized

A `$`-prefixed parameter after `:` selects dynamically:

```
cards_numeral($n) = "{$n} {card:$n}";
// n=1 -> "1 card"  (CLDR: 1 -> "one")
// n=5 -> "5 cards" (CLDR: 5 -> "other")
```

When the parameter is a Phrase, the first matching tag is used:

```
// Russian
allied_adj = { masc: "союзный", fem: "союзная", neut: "союзное" };
allied($entity) = "{allied_adj:$entity} {$entity:nom:one}";
// entity=card (tags: [:fem])     -> "союзная карта"
// entity=character (tags: [:masc]) -> "союзный персонаж"
```

Static and parameterized selectors can be mixed in a selector chain:

```
// Russian -- static case, parameterized number
draw($n) = "Возьмите {card:acc:$n}.";
// n=1 -> "Возьмите карту."  (acc + CLDR "one" -> acc.one)
// n=5 -> "Возьмите карт."   (acc + CLDR "many" -> acc.many)
```

Resolution for `{term:$param}`:

1. If `$param` is a **number** -> map through CLDR plural rules -> use as key
2. If `$param` is a **Phrase** -> iterate tags, use first matching variant key
3. If `$param` is a **string** -> use directly as variant key

### On parameters -- static keys

Parameters that hold Phrase values can have their variants selected:

```
with_cost_less_than_allied($base, $counting) =
    "{$base:one} with cost less than the number of allied {$counting:other}";
```

### On phrase call results

The `:` operator can select variants from a phrase call result:

```
{subtype($s):other}     // -> select "other" from subtype result
{subtype($s):acc:one}   // -> select "acc.one" from subtype result (Russian)
{subtype($s):$n}        // -> parameterized select from subtype result
```

### Not on bare phrase names

Using `:` on a bare phrase name (without `()`) is an error:

```
{cards:other}           // ERROR: 'cards' is a phrase -- use cards(...):other
```

---

## Terms and Phrases: Restrictions

| Syntax | Valid? | Why |
|--------|--------|-----|
| `{card:other}` | Yes | Static selection on term |
| `{card:$n}` | Yes | Parameterized selection on term |
| `{cards($n)}` | Yes | Phrase call with parameter |
| `{cards(2)}` | Yes | Phrase call with literal number |
| `{trigger("Attack")}` | Yes | Phrase call with literal string |
| `{subtype($s):other}` | Yes | Selection on phrase call result |
| `{cards:other}` | **Error** | `cards` is a phrase -- use `cards(...):other` |
| `{card($n)}` | **Error** | `card` is a term -- use `:` |
| `{cards($n, $m)}` | **Error** | Wrong number of arguments (arity mismatch) |
| `{f(g($x))}` | **Error** | Nested phrase calls not supported |
| `{f(card:one)}` | **Error** | Expressions not supported as arguments |

Phrase call arguments can be:

| Argument | Meaning | Value type |
|----------|---------|------------|
| `$param` | Pass a parameter value | Whatever the parameter holds |
| `term_name` | Pass a term as a `Phrase` value | `Phrase` (with tags and variants) |
| `42` | Pass a literal integer | Number |
| `"text"` | Pass a literal string | String |

---

## Metadata Tags

A term can declare metadata tags using `:` before its content:

```
card = :a "card";
event = :an "event";

// Spanish
carta = :fem "carta";
personaje = :masc "personaje";

// Russian -- multiple tags
card = :fem :inan { ... };
character = :masc :anim { ... };
```

Tags serve three purposes:

1. **`:match` branches**: `:match($thing)` reads tags from a Phrase parameter to
   select a branch (first matching tag wins)
2. **Parameterized selection**: `{term:$param}` reads tags from `$param` to
   select a variant from the term (first matching tag wins)
3. **Transforms**: `@a` reads `:a`/`:an` to choose article form

---

## Transforms

The `@` operator applies a transform. Transforms modify text and apply
right-to-left when chained:

```
card = "card";
draw_one = "Draw {@a card}.";        // -> "Draw a card."
title = "{@cap card}";               // -> "Card"
heading = "{@cap @a card}";          // -> "A card"
```

**Automatic capitalization:** `{Card}` is equivalent to `{@cap card}`. This
works for term references only. For parameters, use `{@cap $name}` explicitly.

Transforms combine with selection:

```
card = { one: "card", other: "cards" };
draw($n) = "Draw {$n} {@cap card:$n}.";
// n=1 -> "Draw 1 Card."
// n=3 -> "Draw 3 Cards."
```

### Transform context

Transforms can take context -- static with `:`, dynamic with `()`:

```
{@transform ref}                   No context
{@transform:literal ref}          Static context
{@transform($param) ref}          Dynamic context
{@transform:literal($param) ref}  Both (extremely rare)
```

Examples:

```
// German -- static context
destroy_card = "Zerstöre {@der:acc card}.";

// Chinese -- dynamic context
draw($n) = "抽{@count($n) card}";

// Spanish -- static context for plural article form
return_all($t) = "devuelve {@el:other $t} a la mano";
```

### Universal transforms

| Transform | Effect |
|-----------|--------|
| `@cap` | Capitalize first letter |
| `@upper` | All uppercase |
| `@lower` | All lowercase |

### Metadata-driven transforms

Language-specific transforms read metadata tags:

```
card = :a "card";
event = :an "event";
draw_one = "Draw {@a card}.";   // -> "Draw a card."
play_one = "Play {@a event}.";  // -> "Play an event."
```

Representative transforms (subset -- RLF supports 20+ languages):

| Transform   | Languages              | Reads Tags                   | Effect                     |
| ----------- | ---------------------- | ---------------------------- | -------------------------- |
| `@a`        | English                | `:a`, `:an`                  | Indefinite article         |
| `@the`      | English                | --                           | Definite article           |
| `@plural`   | English                | --                           | Select `other` variant     |
| `@der`      | German                 | `:masc`, `:fem`, `:neut`     | Definite article + case    |
| `@ein`      | German                 | `:masc`, `:fem`, `:neut`     | Indefinite article + case  |
| `@el`       | Spanish                | `:masc`, `:fem`              | Definite article           |
| `@le`       | French                 | `:masc`, `:fem`, `:vowel`    | Definite article           |
| `@un`       | Spanish, French, Italian | `:masc`, `:fem`            | Indefinite article         |
| `@o`        | Portuguese, Greek      | `:masc`, `:fem`              | Definite article           |
| `@count`    | CJK, Vietnamese, etc.  | measure word tags            | Measure word / classifier  |
| `@inflect`  | Turkish, Finnish, Hungarian | vowel harmony tags      | Agglutinative suffix chain |

Each language has its own implementation. Aliases map alternative names to the
canonical form: `@an` -> `@a`, `@die` -> `@der`, `@la` -> `@el`, etc. See
**APPENDIX_STDLIB.md** for complete per-language documentation.

---

## Selection Errors

Selecting a variant that does not exist is a **runtime error**. RLF does not
silently fall back to empty strings or default variants on selection failure.

| Scenario | Example | Error |
|----------|---------|-------|
| Named variant not found | `{card:dat}` on `{ one: ..., other: ... }` | `MissingVariant` |
| Multi-dim key not found | `{card:acc:one}` on `{ one: ..., other: ... }` | `MissingVariant` |
| Selection on a String | `{$param:one}` when `$param` is `"hello"` | `MissingVariant` |
| Selection on a Number | `{$param:one}` when `$param` is `42` | `MissingVariant` |
| No matching tag | `{adj:$entity}` when `$entity` has no matching tags | Falls to `*` default or error |
| Arity mismatch | `{cards($n, $m)}` when `cards` takes 1 parameter | `ArityMismatch` |

**Compile-time detection.** When selectors are static literals (e.g.,
`{card:dat}`), the compiler detects missing variants at compile time.
Parameterized selection (`{card:$n}`) can only be validated at runtime.

---

## Escape Sequences

The special characters `:`, `@`, and `$` are only meaningful inside
`{}`-delimited expressions. In regular string text, they are literal and need
no escaping:

```
help_text = "Dissolve: Send a character to the void";
ratio = "The ratio is 1:2.";
email = "user@example.com";
price = "The cost is $5.";
```

Only `{` and `}` need escaping in text, since they delimit expressions:

| Sequence | Output | Where needed |
|----------|--------|--------------|
| `{{` | `{` | Anywhere in text |
| `}}` | `}` | Anywhere in text |

Inside `{}`-delimited expressions, use doubled characters for literals:

| Sequence | Output | Where needed |
|----------|--------|--------------|
| `::` | `:` | Inside `{}` only |
| `@@` | `@` | Inside `{}` only |
| `$$` | `$` | Inside `{}` only |

```
syntax_help = "Use {{$name}} for parameters.";
// -> "Use {$name} for parameters."
```

---

## File Structure

```
src/
  localization/
    mod.rs
    strings.rlf.rs     # Source language (English) - uses rlf!
  assets/
    localization/
      ru.rlf           # Russian translation - loaded at runtime
      es.rlf           # Spanish translation - loaded at runtime
      zh_cn.rlf        # Chinese translation - loaded at runtime
```

The source language (`strings.rlf.rs`) defines the API via the `rlf!` macro.
Translation files (`.rlf`) use the same syntax but are loaded by the interpreter
at runtime.

**Translation file format:**

```
// Comment
hello = "Привет, мир!";
card = :fem { one: "карта", few: "карты", many: "карт" };
cards($n) = :match($n) {
    1: "карту",
    few: "{$n} карты",
    *other: "{$n} карт",
};
```

---

## The Locale Object

The `Locale` object manages language selection and translation data. Create with
`Locale::new()` (defaults to English) or `Locale::with_language("ru")`:

```rust
fn setup_localization() -> Locale {
    let mut locale = Locale::new();
    strings::register_source_phrases(&mut locale);
    locale.load_translations("ru", "assets/localization/ru.rlf")?;
    locale.load_translations("es", "assets/localization/es.rlf")?;
    locale.set_language(&user_preferences.language);
    locale
}
```

`Locale::builder()` offers additional options like `string_context` (see
**APPENDIX_RUST_INTEGRATION.md**). Loaded translations can be validated with
`validate_translations()` and hot-reloaded with `reload_translations()`.

See **APPENDIX_RUNTIME_INTERPRETER.md** for complete API documentation.

---

## Global Locale

The `global-locale` Cargo feature stores the locale in global state, removing
the `locale` parameter from all generated functions and auto-registering
source definitions on first use.

Enable it in `Cargo.toml`:

```toml
[dependencies]
rlf = { version = "0.1", features = ["global-locale"] }
```

With `global-locale`, generated functions take no `locale` parameter:

```rust
// Without global-locale:
let text = strings::card(&locale);

// With global-locale:
let text = strings::card();
```

Source definitions are registered automatically on first call. To switch
languages or load translations, use the global locale API:

```rust
// At startup
rlf::set_language("ru");
rlf::with_locale_mut(|locale| {
    locale.load_translations("ru", "assets/localization/ru.rlf").unwrap();
});

// Usage -- no locale parameter needed
let text = strings::draw(3);

// Switch language at runtime
rlf::set_language("es");
```

The four public functions are:
- `set_language(lang)` -- sets the current language
- `language() -> String` -- returns the current language
- `with_locale(|locale| ...)` -- read access to the global `Locale`
- `with_locale_mut(|locale| ...)` -- write access to the global `Locale`

See **APPENDIX_RUST_INTEGRATION.md** for complete API documentation.

---

## Generated API

Given:

```rust
// strings.rlf.rs
rlf! {
    card = :a { one: "card", other: "cards" };
    cards($n) = :match($n) {
        1: "a card",
        *other: "{$n} cards",
    };
    draw($n) = "Draw {cards($n)}.";
}
```

RLF generates:

```rust
// strings.rs (generated)

/// Returns the "card" term.
pub fn card(locale: &Locale) -> Phrase {
    locale.get_phrase("card")
        .expect("term 'card' should exist")
}

/// Evaluates the "cards" phrase with parameter n.
pub fn cards(locale: &Locale, n: impl Into<Value>) -> Phrase {
    locale.call_phrase("cards", &[n.into()])
        .expect("phrase 'cards' should exist")
}

/// Evaluates the "draw" phrase with parameter n.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase {
    locale.call_phrase("draw", &[n.into()])
        .expect("phrase 'draw' should exist")
}

/// Registers source language definitions with the locale.
/// Call once at startup.
pub fn register_source_phrases(locale: &mut Locale) {
    locale.load_translations_str("en", SOURCE_PHRASES)
        .expect("source definitions should parse successfully");
}

const SOURCE_PHRASES: &str = r#"
    card = :a { one: "card", other: "cards" };
    cards($n) = :match($n) {
        1: "a card",
        *other: "{$n} cards",
    };
    draw($n) = "Draw {cards($n)}.";
"#;
```

**Usage:**

```rust
use localization::strings;

fn render_card_text(locale: &Locale) {
    let text = strings::draw(locale, 3);
    // English: "Draw 3 cards."
    // Russian: "Возьмите 3 карты."
}
```

**Note:** All functions return `Phrase`. For definitions without `:from` or
declared variants/tags, the returned `Phrase` has empty variants and tags,
behaving like a simple string.

---

## Runtime Templates

For data-driven content (templates stored in data files), use `Locale` directly:

```rust
let template = "Draw {cards($n)} for each {$target}.";
let params = params!{ "n" => 2, "target" => strings::ally(&locale) };
let phrase: Phrase = locale.eval_str(template, params)?;
phrase.to_string()
```

Parameters work identically to phrase parameters. See **APPENDIX_RUNTIME_INTERPRETER.md**.

---

## Runtime Values

All parameters accept a `Value` type:

```rust
strings::cards(&locale, 3);                           // number
strings::cards(&locale, "3");                         // string
strings::greet(&locale, "World");                     // string
strings::destroy(&locale, strings::card(&locale));    // phrase
```

**Runtime behavior:**

| Operation              | Value Type      | Behavior                                    |
| ---------------------- | --------------- | ------------------------------------------- |
| `{$x}`                 | Any             | Display the value                           |
| `{card:$x}` (selection)| Number          | Select plural category via CLDR             |
| `{card:$x}` (selection)| Float           | Convert to integer, select plural category  |
| `{card:$x}` (selection)| String          | Parse as number, or use as literal key      |
| `{card:$x}` (selection)| Phrase          | Look up matching tag                        |
| `{@a $x}`              | Phrase with tag | Use the tag                                 |
| `{@a $x}`              | Other           | **Runtime error**                           |

---

## Compile-Time Errors

RLF validates the source file at compile time:

**Unknown term or phrase:**

```rust
rlf! {
    draw($n) = "Draw {$n} {cards($n)}.";  // 'cards' not defined
}
```

```
error: unknown phrase or parameter 'cards'
 --> src/main.rs:3:28
  |
  = help: did you mean 'card'?
```

**Unknown parameter:**

```rust
rlf! {
    draw($n) = "Draw {$count} cards.";
}
```

```
error: unknown parameter '$count'
  --> strings.rlf.rs:2:18
   |
   = help: declared parameters: $n
```

**Additional compile-time checks:**

- **Cyclic references**: Definitions that reference each other in a cycle are rejected
- **Parameter shadowing**: A parameter cannot have the same name as a term or phrase
- **Term/phrase misuse**: Using `()` on a term or `:` on a bare phrase name
- **Missing `*` default**: `:match` blocks require a `*` default branch
- **Arity mismatch**: Wrong number of arguments in a phrase call

Translation files are validated at load time, not compile time.

---

## Runtime Errors

RLF uses a two-layer error model:

**Generated functions** (from `rlf!`) use `.expect()` and **panic** on errors.
These are for static definitions where errors indicate programming mistakes:

```rust
strings::draw(&locale, 3);  // Panics if "draw" phrase is missing or malformed
```

**Locale methods** return `Result` for data-driven content where errors may come
from external data:

```rust
locale.eval_str(template, params)?;  // Returns Result
```

**No language fallback:** If a definition exists in English but not in Russian,
requesting the Russian version returns `PhraseNotFound` -- it does not fall back
to English. Translations must be complete; missing definitions are errors.

See **APPENDIX_RUST_INTEGRATION.md** for the full error type definitions.

---

## Phrase Type

All functions return a `Phrase` that carries metadata:

```rust
pub struct Phrase {
    /// Default text.
    pub text: String,
    /// Variant key -> variant text.
    pub variants: HashMap<VariantKey, String>,
    /// Metadata tags.
    pub tags: Vec<Tag>,
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution.
    /// Tries exact match first, then progressively shorter keys.
    /// Panics if no match found.
    pub fn variant(&self, key: &str) -> &str;
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

`VariantKey` and `Tag` are newtype wrappers around `String` with `Deref<Target=str>`, `From<&str>`, and `Display`.

Use `variant()` to access specific forms:

```rust
let card = strings::card(&locale);
let singular = card.to_string();            // "card"
let plural = card.variant("other");         // "cards"
```

---

## Phrase Identifiers

For scenarios where you need to store a reference to a definition in serializable
data structures, RLF provides `PhraseId` -- a compact, `Copy`-able, 16-byte
identifier based on a 128-bit hash of the name. The `rlf!` macro generates
`PhraseId` constants for all definitions.

```rust
// Store in serializable data
let card_name: PhraseId = strings::phrase_ids::FIRE_ELEMENTAL;
let draw_phrase: PhraseId = strings::phrase_ids::DRAW;

// Resolve parameterless term (returns Result<Phrase, EvalError>)
let phrase = card_name.resolve(&locale).expect("term should exist");
let text = phrase.to_string();  // -> "Fire Elemental"

// Resolve phrase with parameters (returns Result<Phrase, EvalError>)
let phrase = draw_phrase.call(&locale, &[3.into()])
    .expect("phrase should exist");
let text = phrase.to_string();  // -> "Draw 3 cards."
```

See **APPENDIX_RUST_INTEGRATION.md** for complete details on `PhraseId`
generation, API, and usage patterns.

---

## Design Philosophy

**Unified interpreter, compile-time validation.** All languages (including the
source) are evaluated by the interpreter at runtime. The source language gets
full compile-time syntax and reference checking via the macro. Translations are
loaded at runtime, enabling hot-reloading and community translations.

**Immediate IDE support.** When you add a definition to `strings.rlf.rs`, it
appears in autocomplete immediately. No external tools, no build steps.

**Language-agnostic API.** Functions take a locale parameter. The same code
works for all languages -- Rust identifies what to say, RLF handles how to say it.

**Terms and phrases.** Every definition is either a term (no parameters,
selected with `:`) or a phrase (with parameters, called with `()`). This
eliminates the need for implicit resolution rules -- the definition kind is
always known.

**`$` means parameter.** Every `$` is a parameter. Every bare name is a term or
phrase. No lookup rules needed -- the syntax is self-describing.

**`:` selects, `()` calls.** The `:` operator selects variants from terms,
parameters, and phrase call results. Parentheses `()` call phrases.

**`:match` and `:from`.** The two phrase keywords make branching explicit.
`:match` branches on parameter values (numbers, tags) with a required `*`
default. `:from` inherits metadata for composition.

**Pass Phrase, not String.** When composing definitions, pass `Phrase` values
rather than pre-rendered strings. This preserves variants and tags so RLF can
select the correct grammatical form.

**Logic in Rust, text in RLF.** Complex branching stays in Rust; RLF provides
atomic text pieces. Translators don't need to understand Rust.

**Keywords and formatting are terms.** No special syntax -- define terms with
markup (`dissolve = "<k>dissolve</k>";`) and interpolate normally.

**Dynamic typing for simplicity.** Parameters accept any `Value`. Runtime errors
catch type mismatches. Translators don't need Rust types.

---

## Translation Workflow

### Adding a New Definition

1. Add the definition to `strings.rlf.rs`:
   ```rust
   rlf! {
       point = :a { one: "point", other: "points" };
       new_ability($n) = "Gain {$n} {point:$n}.";
   }
   ```

2. Use it immediately in Rust code (autocomplete works):
   ```rust
   let text = strings::new_ability(&locale, 5);
   ```

3. Later, add translations to `.rlf` files:
   ```
   // ru.rlf
   point = :fem { one: "очко", few: "очка", many: "очков" };
   new_ability($n) = "Получите {$n} {point:$n}.";
   ```

### Updating Translations

1. Edit the `.rlf` file
2. Reload in development (if hot-reload enabled) or restart
3. Changes take effect without recompilation

---

## Summary

| Primitive    | Syntax                                | Purpose                                 |
| ------------ | ------------------------------------- | --------------------------------------- |
| Term         | `name = "text";`                      | Define text with optional variants      |
| Phrase       | `name($p) = "{$p}";`                  | Accept parameters, produce text         |
| Variant      | `name = { a: "x", b: "y" };`          | Multiple forms of a term                |
| Selection    | `{term:selector}`                     | Choose a variant                        |
| Metadata tag | `name = :tag "text";`                 | Attach metadata                         |
| `:match`     | `name($p) = :match($p) { ... };`      | Branch on parameter value               |
| `:from`      | `name($p) = :from($p) "{$p}";`        | Inherit tags/variants from parameter    |
| Transform    | `{@transform:ctx term}`               | Modify text                             |

| File Type        | Extension    | Purpose                               |
| ---------------- | ------------ | ------------------------------------- |
| Source language  | `.rlf.rs`    | Compiled via `rlf!` macro             |
| Translations     | `.rlf`       | Loaded at runtime via interpreter     |

| Type             | Purpose                                | Size / Traits                       |
| ---------------- | -------------------------------------- | ----------------------------------- |
| `Phrase`         | Returned by all functions              | Heap-allocated                      |
| `PhraseId`       | Serializable reference to a definition | 16 bytes, `Copy`, `Serialize`       |
| `Value`          | Runtime parameter (number/string/phrase)| Enum                                |

| Component        | Compile-Time              | Runtime                             |
| ---------------- | ------------------------- | ----------------------------------- |
| Source language  | Full validation           | Interpreter evaluation              |
| Translations     | (optional strict check)   | Interpreter evaluation              |

Two definition types, one macro, compile-time source checking, runtime
translations.
