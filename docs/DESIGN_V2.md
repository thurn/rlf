# RLF v2 Syntax Design

## Overview

RLF v2 introduces two syntax changes that eliminate ambiguity from the
language:

1. **$ prefix**: Parameters are always marked with `$`. Bare names always refer
   to definitions.
2. **Terms and phrases**: Definitions are divided into **terms** (no parameters,
   selected with `:`) and **phrases** (with parameters, called with `()`). It
   is an error to use `:` on a phrase name or `()` on a term name.

Together, these changes make every reference in a template body unambiguous from
syntax alone — no implicit resolution rules needed.

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
all_cards = "All {card:other}.";     // → "All cards."
title = "{@cap card}";               // → "Card"       (default = "one")
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
example = "{card}";                       // → "card" (*one is default)
```

The `*` marker can only appear on top-level variant keys (e.g., `*one`), not on
multi-dimensional keys (e.g., `*nom.one` is not valid). A bare reference to a
term with only multi-dimensional variant keys (no `:` selector) is a
compile-time error — always use an explicit selector.

Variant keys are always named identifiers. Numeric keys are not supported in
term variant blocks — use `:match` in a phrase for numeric branching.

Multi-dimensional variants use dot notation in variant block keys. In template
selection expressions, the `:` operator chains dimensions with successive `:`
separators. Dots are for declaration; colons are for selection.

```
// Declaration — dot notation in variant keys
card = :fem {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};

// Selection — colon chains in template expressions
example = "{card:acc:one}";        // → "карту" (selects acc.one)
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
// n=1 → "1 card"   (CLDR maps 1 → "one")
// n=5 → "5 cards"  (CLDR maps 5 → "other")

// Select variant based on a Phrase parameter's tags
allied_adj = { masc: "allied", fem: "allied", neut: "allied" };
modified($entity) = "{allied_adj:$entity} {$entity:one}";
```

Note: parameterized selection on a term (`{card:$n}`) maps numbers through
CLDR plural categories only. It does **not** try exact numeric keys — use
`:match` in a phrase for exact-number matching. Literal numeric selection on
terms (`{card:3}`) is not supported.

---

## Phrases

A **phrase** is a definition with one or more parameters. A definition with
an empty parameter list (`name() = ...`) is a syntax error — use a term
instead. Phrases use two keywords to control how parameters affect the output:

- **`:match($param)`** — branches on a parameter value, similar to Rust's
  `match`
- **`:from($param)`** — inherits tags and variants from a parameter

```
// Simple template
energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";

// With :match — branches on $n
cards($n) = :match($n) {
    1: "a card",
    *other: "{$n} cards",
};

// With :from — inherits metadata from $s
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
pair = "You have {cards(2)}.";       // → "You have a pair of cards."
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
// cards(1) → "a card"
// cards(3) → "3 cards"
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
// n=0 → "no cards"        (exact match)
// n=1 → "a card"          (exact match)
// n=2 → "a pair of cards" (exact match)
// n=5 → "5 cards"         (CLDR "other" → default)
```

This matches ICU MessageFormat's precedence where exact values override plural
categories. Exact numeric keys and CLDR-based resolution are exclusive to
`:match` — parameterized selection on terms (`{card:$n}`) only uses CLDR.

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
// destroyed(card)      → "destruida"  (card has tag :fem → matches "fem")
// destroyed(character) → "destruido"  (character has tag :masc → matches "masc")
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
// Russian — branch on both count ($n) and gender ($entity)
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
1. Match `$n=3`: exact `3` (miss) → CLDR `other` (hit)
2. Match `$entity=card`: tags `[:fem]` → first match `fem` (hit)
3. Select branch `other.fem` → `"{$n} союзных {$entity:gen:many}"`

Each dimension in a multi-match uses the same resolution algorithm as a
single-parameter match. Wildcard fallbacks work on intermediate dimensions just
like in variant blocks — `other` matches `other.*` if no more-specific key
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
variant — the same default rules as terms.

Usage:

```
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";
// dissolve_subtype(ancient) → "Dissolve an <b>Ancient</b>."
// dissolve_all(ancient)     → "Dissolve all <b>Ancients</b>."
```

**`:from` and variant selection.** A `:from` phrase returns a Phrase with a
full variant table. The `:` operator can select variants from a phrase call
result, just like it can from a term or parameter:

```
// English
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";
// dissolve_subtype(ancient) → "Dissolve an <b>Ancient</b>."
// dissolve_all(ancient)     → "Dissolve all <b>Ancients</b>."

// Russian — select case and number from the inherited variants
dissolve_subtype($s) = "Растворите {subtype($s):acc:one}.";
all_subtypes($s) = "все {subtype($s):nom:other}";
// dissolve_subtype(ancient) → "Растворите <b>Древнего</b>."
// all_subtypes(ancient)     → "все <b>Древние</b>"
```

This preserves `:from`'s value as a single source of truth for formatting —
define the markup once in `subtype`, then select any case/number from the
result in downstream phrases.

### Combining `:from` and `:match`

A phrase can use both `:from` and `:match` together. `:from` determines the
inherited tags and variant structure, while `:match` branches on a separate
parameter within each inherited variant's evaluation:

```
// Russian — "N allied <Warriors>"
// Inherits subtype metadata via :from, branches on count via :match
count_allied_subtype($n, $s) = :from($s) :match($n) {
    1: "союзный {subtype($s)}",
    *other: "{$n} союзных {subtype($s):gen:many}",
};
```

Evaluation of `count_allied_subtype(3, warrior)`:

| Step | Action | Result |
|------|--------|--------|
| 1 | Read `warrior.tags` and variants | Tags: `[:a]`, variants: `{one: ..., other: ...}` |
| 2 | For variant `one`: match `$n=3` | Falls to `*other` → `"3 союзных <b>Warriors</b>"` (with gen.many) |
| 3 | For variant `other`: match `$n=3` | Falls to `*other` → `"3 союзных <b>Warriors</b>"` (with gen.many) |
| 4 | Return Phrase | Tags from `warrior`, variant table with inherited structure |

**Evaluation model.** When `:from` and `:match` are combined, the template body
is evaluated once per inherited variant. Within each evaluation, `{$s}` resolves
to the text of the current inherited variant (e.g., the `one` text, then the
`other` text). References to other phrases like `{subtype($s)}` pass the full
Phrase value of `$s` — they do not see the per-variant context. Only bare
interpolation of the `:from` parameter (`{$s}`) is affected by the inherited
variant iteration.

The order of `:from` and `:match` in the declaration does not matter —
`:from($s) :match($n)` and `:match($n) :from($s)` are equivalent.

---

## The $ Prefix

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

### On terms — static keys

```
{card:other}            // → "cards"
{card:acc:one}          // → "карту" (Russian)
```

### On terms — parameterized

A `$`-prefixed parameter after `:` selects dynamically:

```
cards_numeral($n) = "{$n} {card:$n}";
// n=1 → "1 card"  (CLDR: 1 → "one")
// n=5 → "5 cards" (CLDR: 5 → "other")
```

When the parameter is a Phrase, the first matching tag is used:

```
// Russian
allied_adj = { masc: "союзный", fem: "союзная", neut: "союзное" };
allied($entity) = "{allied_adj:$entity} {$entity:nom:one}";
// entity=card (tags: [:fem])     → "союзная карта"
// entity=character (tags: [:masc]) → "союзный персонаж"
```

Static and parameterized selectors can be mixed in a selector chain.
Each selector resolves independently, and the results combine into a
multi-dimensional key:

```
// Russian — static case, parameterized number
draw($n) = "Возьмите {card:acc:$n}.";
// n=1 → "Возьмите карту."  (acc + CLDR "one" → acc.one)
// n=5 → "Возьмите карт."   (acc + CLDR "many" → acc.many)
```

Resolution for `{term:$param}`:

1. If `$param` is a **number** → map through CLDR plural rules → use as key
2. If `$param` is a **Phrase** → iterate tags, use first matching variant key
3. If `$param` is a **string** → use directly as variant key

### On parameters — static keys

Parameters that hold Phrase values can have their variants selected:

```
// English
with_cost_less_than_allied($base, $counting) =
    "{$base:one} with cost less than the number of allied {$counting:other}";

// Russian — multi-dimensional selection on parameters
with_cost_less_than_allied($base, $counting) =
    "{$base:nom:one} со стоимостью меньше количества союзных {$counting:gen:many}";
```

### On parameters — parameterized

A parameter's variants can also be selected using another parameter:

```
// Select from $adj_term using $entity's tags
another($entity) = "{another_adj:$entity} {$entity:nom:one}";
```

### On phrase call results

The `:` operator can select variants from a phrase call result. This is
unambiguous because `()` already identifies the phrase call:

```
{subtype($s):other}     // → select "other" from subtype result
{subtype($s):acc:one}   // → select "acc.one" from subtype result (Russian)
{subtype($s):$n}        // → parameterized select from subtype result
{cards($n):one}         // → select "one" from cards result
```

This is essential for `:from` phrases, where the returned Phrase carries a
full variant table that downstream phrases need to select from.

### Not on bare phrase names

Using `:` on a bare phrase name (without `()`) is an error:

```
{cards:other}           // ERROR: 'cards' is a phrase — use cards(...):other
```

---

## Terms and Phrases: Restrictions

| Syntax | Valid? | Why |
|--------|--------|-----|
| `{card:other}` | Yes | Static selection on term |
| `{card:$n}` | Yes | Parameterized selection on term |
| `{card:acc:$n}` | Yes | Mixed static + parameterized selection |
| `{$base:nom:one}` | Yes | Static selection on parameter value |
| `{cards($n)}` | Yes | Phrase call with parameter |
| `{cards(2)}` | Yes | Phrase call with literal number |
| `{trigger("Attack")}` | Yes | Phrase call with literal string |
| `{subtype($s):other}` | Yes | Selection on phrase call result |
| `{subtype($s):acc:one}` | Yes | Multi-dim selection on phrase call result |
| `{subtype($s):$n}` | Yes | Parameterized selection on phrase call result |
| `{cards:other}` | **Error** | `cards` is a phrase — use `cards(...):other` |
| `{card($n)}` | **Error** | `card` is a term — use `:` |
| `{card:acc($n)}` | **Error** | Cannot mix term `:` with `()` |
| `{card:3}` | **Error** | Literal numeric selection not supported on terms |
| `{cards($n, $m)}` | **Error** | Wrong number of arguments (arity mismatch) |
| `{f(g($x))}` | **Error** | Nested phrase calls not supported as arguments |
| `{f(card:one)}` | **Error** | Expressions not supported as arguments |

Error messages indicate the definition kind and suggest the correct syntax:

```
error: 'card' is a term — cannot use () call syntax
  = help: use {card:variant} or {card:$param} to select a variant

error: 'cards' is a phrase — cannot use : without ()
  = help: use {cards(...)} or {cards(...):variant}
```

### How `name(...)` is resolved

Every name is declared as either a term or a phrase. This makes resolution
straightforward:

| Expression | Term | Phrase |
|-----------|------|--------|
| `{name}` | Default variant (`*`-marked or first) | **Error**: phrase must be called with `()` |
| `{name:key}` | Select variant | **Error**: phrase cannot use `:` without `()` |
| `{name:$p}` | Parameterized select | **Error**: phrase cannot use `:` without `()` |
| `{name(...)}` | **Error**: term cannot use `()` | Call with arguments |
| `{name(...):key}` | **Error**: term cannot use `()` | Call, then select variant from result |
| `{name(...):$p}` | **Error**: term cannot use `()` | Call, then parameterized select from result |

Phrase call arguments can be:

| Argument | Meaning | Value type |
|----------|---------|------------|
| `$param` | Pass a parameter value | Whatever the parameter holds |
| `term_name` | Pass a term as a `Phrase` value | `Phrase` (with tags and variants) |
| `42` | Pass a literal integer | Number |
| `"text"` | Pass a literal string | String |

```
{energy($e)}         → call phrase 'energy', pass parameter $e
{subtype(ancient)}   → call phrase 'subtype', pass term 'ancient'
{cards(2)}           → call phrase 'cards', pass literal 2
{destroyed($thing)}  → call phrase 'destroyed', pass parameter $thing
{trigger("Attack")}  → call phrase 'trigger', pass literal string "Attack"
```

Arguments must be exactly one of the four types above — no expressions,
selectors, or nested calls. `{f(g($x))}` and `{f(card:one)}` are both errors.
To pass a phrase result or selected variant, bind it to a parameter in Rust.

Phrases accept any number of parameters. Calling a phrase with the wrong number
of arguments is an error detected at compile time (for `rlf!`) or load time
(for translation files).

**Literal strings.** A string literal argument becomes a plain `String` value.
When interpolated with `{$param}`, it renders as its text. String values have
no tags and no variants — selecting a variant from a string (e.g.,
`{$param:one}`) is a runtime error. Use string literals for display text that
needs no grammatical metadata:

```
trigger($t) = "\u{25B8} <b>{$t}::</b>";
example = "{trigger("Attack")}";     // → "▸ <b>Attack::</b>"
```

String literals inside `()` use standard escaping: `\"` for a literal quote,
`\\` for a literal backslash. Braces and other RLF special characters are
not interpreted inside string literals.

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
works for term references only. For parameters, use `{@cap $name}` explicitly.

**Naming rules.** Term and phrase names consist of lowercase ASCII letters,
digits, and underscores, and must start with a letter. A name cannot be used
for both a term and a phrase — each name refers to exactly one definition.

### Transform context

Transforms can take context — static with `:`, dynamic with `()`:

```
{@transform ref}                   No context
{@transform:literal ref}          Static context
{@transform($param) ref}          Dynamic context
{@transform:literal($param) ref}  Both (extremely rare)
```

The term/phrase restriction on `:` and `()` does **not** apply to transform
context. Transform context is a separate mechanism.

Examples:

```
// German — static context
destroy_card = "Zerstöre {@der:acc card}.";

// Chinese — dynamic context
draw($n) = "抽{@count($n) card}";

// Spanish — static context for plural article form
return_all($t) = "devuelve {@el:other $t} a la mano";

// German — static context + static selection on term
get_card = "Nimm {@der:acc card:one}.";
```

### Universal transforms

| Transform | Effect |
|-----------|--------|
| `@cap` | Capitalize first letter |
| `@upper` | All uppercase |
| `@lower` | All lowercase |

### English transforms

| Transform | Reads Tags | Effect |
|-----------|------------|--------|
| `@a` | `:a`, `:an` | Indefinite article |
| `@the` | — | Definite article |
| `@plural` | — | Select the `other` variant (English-only because other languages need case/gender-aware plural selection via `:` instead) |

### Language-specific transforms

| Transform | Languages | Reads Tags | Effect |
|-----------|-----------|------------|--------|
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

A term can declare metadata tags using `:` before its content:

```
card = :a "card";
event = :an "event";

// Spanish
carta = :fem "carta";
personaje = :masc "personaje";

// Russian — multiple tags
card = :fem :inan { ... };
character = :masc :anim { ... };
```

Tags serve three purposes:

1. **`:match` branches**: `:match($thing)` reads tags from a Phrase parameter to
   select a branch (first matching tag wins)
2. **Parameterized selection**: `{term:$param}` reads tags from `$param` to
   select a variant from the term (first matching tag wins)
3. **Transforms**: `@a` reads `:a`/`:an` to choose article form

### Tag-based selection

Both `:match` and parameterized selection (`:$param`) resolve Phrase tags the
same way: iterate tags in declaration order, use the first one that matches a
variant key.

```
// Russian
card = :fem :inan { ... };
character = :masc :anim { ... };

// Parameterized selection on a term using tags
another_adj = { masc: "другой", fem: "другая", neut: "другое" };
another($entity) = "{another_adj:$entity} {$entity:nom:one}";
// entity=card      → tags [:fem, :inan] → first match "fem" → "другая карта"
// entity=character → tags [:masc, :anim] → first match "masc" → "другой персонаж"

// :match using tags
destroyed($thing) = :match($thing) {
    masc: "destruido",
    *fem: "destruida",
};
// thing=card → tags [:fem] → matches "fem" → "destruida"
```

### `:from` inheritance

```
ancient = :an { one: "Ancient", other: "Ancients" };
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

See [The `:from` Keyword](#the-from-keyword) for details.

---

## Selection Errors

Selecting a variant that does not exist is a **runtime error**. RLF does not
silently fall back to empty strings or default variants on selection failure.

```
card = :a { one: "card", other: "cards" };

{card:other}            // → "cards"       (OK)
{card:acc:one}          // → ERROR         (no "acc" dimension)
{card:one}              // → "card"        (OK)
```

Specifically, these are errors:

| Scenario | Example | Error |
|----------|---------|-------|
| Named variant not found | `{card:dat}` on `{ one: ..., other: ... }` | `MissingVariant`: no variant `dat` |
| Multi-dim key not found | `{card:acc:one}` on `{ one: ..., other: ... }` | `MissingVariant`: no variant `acc.one` and no wildcard `acc` |
| Selection on a String | `{$param:one}` when `$param` is `"hello"` | `MissingVariant`: string values have no variants |
| Selection on a Number | `{$param:one}` when `$param` is `42` | `MissingVariant`: number values have no variants |
| No matching tag | `{adj:$entity}` when `$entity` has no tags matching any variant key | Falls through to `*` default if present; error if no default |
| Arity mismatch | `{cards($n, $m)}` when `cards` takes 1 parameter | `ArityMismatch`: expected 1 argument, got 2 |

For **parameterized selection** (`{term:$param}`), if no variant matches the
resolved key (after CLDR mapping or tag iteration), and the term has no `*`
default variant, this is also a `MissingVariant` error.

**Compile-time detection.** When selectors are static literals (e.g.,
`{card:dat}`), the compiler can detect missing variants at compile time and
report the error immediately. Parameterized selection (`{card:$n}`) can only be
validated at runtime.

---

## Escape Sequences

The special characters `:`, `@`, and `$` are only meaningful inside
`{}`-delimited expressions. In regular string text, they are literal and need
no escaping:

```
// No escaping needed — : and @ are literal in text
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
// → "Use {$name} for parameters."
```

---

## Generated Rust API

Given:

```rust
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
/// Returns the "card" term.
pub fn card(locale: &Locale) -> Phrase { ... }

/// Evaluates the "cards" phrase with parameter n.
pub fn cards(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }

/// Evaluates the "draw" phrase with parameter n.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }
```

All definitions return `Phrase`. Terms return `Phrase` with variants and tags
intact. Phrases return `Phrase` with resolved text. Parameter names in generated
functions drop the `$` prefix.

### PhraseId constants

The macro also generates `PhraseId` constants:

```rust
pub mod phrase_ids {
    pub const CARD: PhraseId = ...;
    pub const CARDS: PhraseId = ...;
    pub const DRAW: PhraseId = ...;
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
card = :fem :inan {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
    gen.one: "карты",
    gen.many: "карт",
    ins.one: "картой",
    ins: "картами",
};

cards($n) = :match($n) {
    1: "карту",
    few: "{$n} карты",
    *other: "{$n} карт",
};
```

---

## Complete Example: Game Localization

Based on real-world usage in the Dreamtides card game.

### English source

```rust
rlf! {
    // =====================================================
    // Terms — symbols and keywords
    // =====================================================

    energy_symbol = "<color=#00838F>\u{25CF}</color>";
    points_symbol = "<color=#F57F17>\u{234F}</color>";

    dissolve = "<color=#AA00FF>dissolve</color>";
    banish = "<color=#AA00FF>banish</color>";
    prevent = "<color=#AA00FF>prevent</color>";
    reclaim = "<color=#AA00FF>reclaim</color>";
    materialized = "<color=#AA00FF>materialized</color>";
    fast = "<b>\u{21AF}fast</b>";

    // =====================================================
    // Terms — nouns with article metadata and plural forms
    // =====================================================

    card = :a { one: "card", other: "cards" };
    character = :a { one: "character", other: "characters" };
    event = :an { one: "event", other: "events" };
    ally = :an { one: "ally", other: "allies" };

    your_card = { one: "your card", other: "your cards" };
    your_event = { one: "your event", other: "your events" };
    enemy = :an { one: "enemy", other: "enemies" };
    enemy_card = :an { one: "enemy card", other: "enemy cards" };
    enemy_event = :an { one: "enemy event", other: "enemy events" };

    // Pronoun agreement
    it_or_them = { one: "it", other: "them" };

    // Character subtypes
    ancient = :an { one: "Ancient", other: "Ancients" };
    warrior = :a { one: "Warrior", other: "Warriors" };
    explorer = :an { one: "Explorer", other: "Explorers" };

    // Standalone references
    an_ally = "an ally";
    an_enemy = "an enemy";
    a_character = "a character";
    a_card = "a card";
    an_event = "an event";

    // =====================================================
    // Phrases — simple templates
    // =====================================================

    // Energy amount with colored symbol
    energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";

    // Keyword with value
    kindle($k) = "<color=#AA00FF>kindle</color> {$k}";
    foresee($n) = "<color=#AA00FF>foresee</color> {$n}";

    // Numeral card count (always numeric, e.g., "1 card" or "5 cards")
    cards_numeral($n) = "{$n} {card:$n}";

    // Subtype display with inherited metadata
    subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";

    // Composing phrases
    dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
    dissolve_all($s) = "Dissolve all {subtype($s):other}.";

    // Compositional: ownership + entity
    allied($entity) = "allied {$entity:one}";
    allied_plural($entity) = "allied {$entity:other}";
    enemy_modified($entity) = "enemy {$entity:one}";
    enemy_modified_plural($entity) = "enemy {$entity:other}";

    // Negation patterns
    not_a($entity) = "a character that is not {@a $entity}";
    ally_not($entity) = "ally that is not {@a $entity}";
    non_entity_enemy($entity) = "non-{$entity:one} enemy";
    characters_not_plural($entity) = "characters that are not {$entity:other}";
    allies_not_plural($entity) = "allies that are not {$entity:other}";

    // Constraint patterns — take Phrase for base, select variant
    with_spark($base, $spark, $op) = "{$base:one} with spark {$spark}{$op}";
    with_spark_plural($base, $spark, $op) =
        "{$base:other} with spark {$spark}{$op}";
    with_cost($base, $cost, $op) = "{$base:one} with cost {$cost}{$op}";
    with_cost_plural($base, $cost, $op) =
        "{$base:other} with cost {$cost}{$op}";

    with_materialized($base) = "{$base:one} with a {materialized} ability";
    with_materialized_plural($base) =
        "{$base:other} with {materialized} abilities";

    // Complex comparisons — $counting is a Phrase, select :other
    with_cost_less_than_allied($base, $counting) =
        "{$base:one} with cost less than the number of allied {$counting:other}";
    with_cost_less_than_void($base) =
        "{$base:one} with cost less than the number of cards in your void";

    // Other modifiers
    another($entity) = "another {$entity:one}";
    other_plural($entities) = "other {$entities:other}";
    for_each($entity) = "each {$entity:one}";

    in_your_void($things) = "{$things} in your void";
    in_opponent_void($things) = "{$things} in the opponent's void";

    or_less = " or less";
    or_more = " or more";

    // Definition referencing another phrase
    pay_energy_button($e) = "Spend {energy($e)}";

    // =====================================================
    // Phrases — with :match
    // =====================================================

    // "a card" / "3 cards"
    cards($n) = :match($n) {
        1: "a card",
        *other: "{$n} cards",
    };

    // "top card" / "top 3 cards"
    top_n_cards($n) = :match($n) {
        1: "top card",
        *other: "top {$n} cards",
    };

    // "one" / "3" (word for 1, numeral otherwise)
    text_number($n) = :match($n) {
        1: "one",
        *other: "{$n}",
    };

    // "a copy" / "two copies"
    copies($n) = :match($n) {
        1: "a copy",
        *other: "{text_number($n)} copies",
    };

    // "an ally" / "3 allies"
    count_allies($n) = :match($n) {
        1: "an ally",
        *other: "{$n} allies",
    };

    // "an allied <Warrior>" / "3 allied <Warriors>"
    count_allied_subtype($n, $s) = :match($n) {
        1: "an allied {subtype($s)}",
        *other: "{$n} allied {subtype($s):other}",
    };

    // "it" / "them"
    it_or_them_match($n) = :match($n) {
        1: "it",
        *other: "them",
    };

    // "a random character" / "two random characters"
    n_random_characters($n) = :match($n) {
        1: "a random character",
        *other: "{text_number($n)} random characters",
    };

    // Event dissolve patterns
    event_could_dissolve($target) =
        "an event which could {dissolve} {$target}";
    events_could_dissolve($target) =
        "events which could {dissolve} {$target}";

    // "this turn" / "this turn two times"
    this_turn_times($n) = :match($n) {
        1: "this turn",
        *other: "this turn {text_number($n)} times",
    };

    // Trigger prefix formatting
    trigger($t) = "\u{25B8} <b>{$t}::</b>";

    // Help text
    help_text_dissolve = "{@cap dissolve}: Send a character to the void";
}
```

### Usage in Rust

```rust
let locale = Locale::with_language("en");

strings::energy(&locale, 3);                 // → "<color=#00838F>3●</color>"
strings::cards(&locale, 1);                  // → "a card"
strings::cards(&locale, 3);                  // → "3 cards"
strings::cards_numeral(&locale, 1);          // → "1 card"
strings::cards_numeral(&locale, 5);          // → "5 cards"
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

// Parameterized selection — pass Phrase for base, RLF selects form
let base = strings::character(&locale);
let counting = strings::character(&locale);
strings::with_cost_less_than_allied(&locale, base, counting);
// → "character with cost less than the number of allied characters"
```

### Spanish translation

```
// es.rlf
card = :fem "carta";
character = :masc "personaje";

destroyed($thing) = :match($thing) {
    masc: "destruido",
    *fem: "destruida",
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

// =========================================================================
// Basic Card Types — decline for case and number
// =========================================================================

card = :fem :inan {
    nom.one: "карта",
    nom: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc: "карты",
    acc.many: "карт",
    gen.one: "карты",
    gen: "карт",
    gen.many: "карт",
    ins.one: "картой",
    ins: "картами",
};

character = :masc :anim {
    nom.one: "персонаж",
    nom: "персонажи",
    nom.many: "персонажей",
    acc, gen: "персонажа",
    acc.many, gen.many: "персонажей",
    ins.one: "персонажем",
    ins: "персонажами",
};

event = :neut :inan {
    nom, acc: "событие",
    nom.many, acc.many: "событий",
    gen: "события",
    gen.many: "событий",
    ins.one: "событием",
    ins: "событиями",
};

// =========================================================================
// Compositional Phrases — gender agreement via parameterized selection
// =========================================================================

allied_adj = { masc: "союзный", fem: "союзная", neut: "союзное" };
allied($entity) = "{allied_adj:$entity} {$entity:nom:one}";
allied_plural($entity) = "союзных {$entity:gen:many}";

enemy_adj = { masc: "вражеский", fem: "вражеская", neut: "вражеское" };
enemy_modified($entity) = "{enemy_adj:$entity} {$entity:nom:one}";
enemy_modified_plural($entity) = "вражеских {$entity:gen:many}";

// Negation — instrumental case after "являться"
not_a($entity) = "персонаж, который не является {$entity:ins:one}";
ally_not($entity) = "союзник, который не является {$entity:ins:one}";

// =========================================================================
// Constraint Patterns — select case/number from parameter's Phrase
// =========================================================================

with_spark($base, $spark, $op) = "{$base:nom:one} с искрой {$spark}{$op}";
with_spark_plural($base, $spark, $op) = "{$base:nom} с искрой {$spark}{$op}";
with_cost($base, $cost, $op) = "{$base:nom:one} со стоимостью {$cost}{$op}";

// KEY PATTERN: {$counting:gen:many} extracts genitive plural from parameter
with_cost_less_than_allied($base, $counting) =
    "{$base:nom:one} со стоимостью меньше количества союзных {$counting:gen:many}";

// =========================================================================
// Other Modifiers — tag-based agreement
// =========================================================================

another_adj = { masc: "другой", fem: "другая", neut: "другое" };
another($entity) = "{another_adj:$entity} {$entity:nom:one}";
// entity=card ([:fem]) → "другая карта"
// entity=character ([:masc]) → "другой персонаж"

each_adj = { masc: "каждый", fem: "каждая", neut: "каждое" };
for_each($entity) = "{each_adj:$entity} {$entity:nom:one}";

// =========================================================================
// Phrases with :match
// =========================================================================

cards($n) = :match($n) {
    1: "карту",
    few: "{$n} карты",
    *other: "{$n} карт",
};

// Russian draw — explicit case+number per branch
draw($n) = :match($n) {
    one: "Возьмите {$n} {card:acc:one}.",
    few: "Возьмите {$n} {card:acc:few}.",
    *other: "Возьмите {$n} {card:acc:many}.",
};

inventory($n) = :match($n) {
    0: "У вас нет предметов.",
    1: "У вас один предмет.",
    few: "У вас {$n} предмета.",
    *other: "У вас {$n} предметов.",
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
card = :fem "Karte";
character = :masc "Charakter";

destroy_card = "Zerstöre {@der:acc card}.";
get_card = "Nimm {@der:acc card:one}.";
```

---

## Disambiguation Summary

With `$`, terms (`:`) and phrases (`()`), every reference is unambiguous from
syntax alone:

| Syntax | Meaning | How it's clear |
|--------|---------|----------------|
| `{$n}` | Parameter interpolation | `$` prefix |
| `{card}` | Term reference | No `$`, no `()` |
| `{Card}` | Term + `@cap` | No `$`, uppercase |
| `{card:other}` | Static term selection | Literal after `:` |
| `{card:acc:one}` | Multi-dim term selection | Multiple `:` literals |
| `{card:$n}` | Parameterized term selection | `$` after `:` on term |
| `{card:acc:$n}` | Mixed static + parameterized | Literal then `$` in `:` chain |
| `{$base:nom:one}` | Selection on parameter | `$` prefix + `:` literals |
| `{allied_adj:$entity}` | Tag-based term selection | `$` after `:` on term |
| `{energy($e)}` | Phrase call with param | `$` inside `()` |
| `{subtype(ancient)}` | Phrase call with term arg | Bare name inside `()` |
| `{cards(2)}` | Phrase call with literal | Number inside `()` |
| `{destroyed($thing)}` | Phrase call with param | `$` inside `()` |
| `{@a card}` | Transform on term | No `$` |
| `{@a $target}` | Transform on parameter | `$` prefix |
| `{@der:acc card}` | Transform with static ctx | Literal after `:` |
| `{@count($n) card}` | Transform with dynamic ctx | `$` inside `()` |
| `{subtype($s):other}` | Selection on phrase result | `()` identifies call, `:` selects |
| `{subtype($s):acc:one}` | Multi-dim select on phrase result | `()` call + `:` chain |
| `{subtype($s):$n}` | Parameterized select on phrase result | `()` call + `$` after `:` |
| `{@plural subtype($s)}` | Transform on phrase result | Phrase call as operand |
| `{trigger("Attack")}` | Phrase call with string literal | Quoted string inside `()` |

---

## v1 → v2 Migration

| Syntax | v1 | v2 |
|--------|----|----|
| Parameter declaration | `draw(n)` | `draw($n)` |
| Parameter interpolation | `{n}` | `{$n}` |
| Dynamic selection on term | `{card:n}` | `{card:$n}` |
| Selection on parameter | `{entity:one}` | `{$entity:one}` |
| Tag selection on term | `{adj:entity}` | `{adj:$entity}` |
| Mixed selection | `{card:acc:n}` | `{card:acc:$n}` (add `$`) |
| Dynamic transform ctx | `{@count:n card}` | `{@count($n) card}` |
| Call argument | `{energy(e)}` | `{energy($e)}` |
| Tag-based variant choice | `{destroyed:thing}` | `:match($thing)` in phrase |
| Phrase variant block | `cards(n) = { one: ..., other: ... }` | `cards($n) = :match($n) { 1: ..., *other: ... }` |
| Selection on phrase result | `{subtype(s):other}` | `{subtype($s):other}` (add `$`) |
| Static selection | `{card:other}` | `{card:other}` (unchanged) |
| Static transform ctx | `{@der:acc card}` | `{@der:acc card}` (unchanged) |
| `@plural` | Universal transform | English-only transform |
| Escape `$` | N/A | `$$` inside `{}` (new) |

---

## Design Principles

**Terms and phrases.** Every definition is either a term (no parameters,
selected with `:`) or a phrase (with parameters, called with `()`). This
eliminates the need for implicit resolution rules — the definition kind is
always known.

**$ means parameter.** Every `$` is a parameter. Every bare name is a term or
phrase. No lookup rules needed — the syntax is self-describing.

**`:` selects, `()` calls.** The `:` operator selects variants from terms,
parameters, and phrase call results. Parentheses `()` call phrases. On a bare
name, only one is valid (`:` for terms, `()` for phrases). After a phrase call,
`:` selects from the result: `{subtype($s):other}`.

**`:match` and `:from`.** The two phrase keywords make branching explicit.
`:match` branches on parameter values (numbers, tags) with a required `*`
default. `:from` inherits metadata for composition.

**Logic in Rust, text in RLF.** Complex branching stays in Rust. Phrases with
`:match` handle the common case of count-dependent and tag-dependent text.

**Pass Phrase, not String.** When composing definitions, pass `Phrase` values to
preserve variants and tags for downstream selection, matching, and transforms.
