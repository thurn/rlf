# RLF

The Rust Localization Framework: a localization DSL embedded in Rust via macros.

## Overview

RLF generates a **language-agnostic API** from definitions written in a
localization DSL. The source language (typically English) is compiled via the
`rlf!` macro into Rust functions with full IDE autocomplete. All other
languages are loaded at runtime via the interpreter.

```rust
// strings.rlf.rs - Source language (English)
rlf! {
    card = :a { one: "card", other: "cards" };
    draw($n) = "Draw {$n} {@cap card:$n}.";
}
```

```rust
// Usage
let mut locale = Locale::with_language("en");
strings::draw(&locale, 3);     // -> "Draw 3 Cards."

locale.set_language("ru");
strings::draw(&locale, 3);     // -> "Возьмите 3 Карты."
```

**How it works:**

1. The `rlf!` macro parses the source language and generates one function per
   definition
2. The macro also embeds the source definitions as data for the interpreter
3. At startup, the source definitions are registered with the interpreter
4. All evaluation (source and translations) goes through the interpreter

---

## Whirlwind Tour

RLF has two kinds of definitions: **terms** (no parameters) and **phrases**
(with parameters). Terms represent lexical entries; phrases are templates that
produce text.

```
// Terms -- named text with optional variants and tags
card = :a { one: "card", other: "cards" };
event = :an "event";
go = { present: "go", past: "went", participle: "gone" };

// Phrases -- parameterized templates
cards($n) = :match($n) { 1: "a card", *other: "{$n} cards" };
draw($n) = "Draw {cards($n)}.";
```

**Variants** give a term multiple forms. The `:` operator selects among them:

```
{card}          // -> "card"       (default variant)
{card:other}    // -> "cards"      (select by name)
{card:$n}       // -> CLDR plural  (select by parameter)
```

**Transforms** modify text with the `@` operator. They read **tags** (metadata
like `:a`, `:fem`) from terms:

```
card = :a "card";
event = :an "event";
{@a card}       // -> "a card"     (@a reads :a tag)
{@a event}      // -> "an event"   (@a reads :an tag)
{@cap card}     // -> "Card"       (capitalize)
{Card}          // -> "Card"       (shorthand for @cap)
```

**Parameters** are always `$`-prefixed. Bare names always refer to terms or
phrases:

```
draw($n) = "Draw {$n} {card:$n}.";
//          param ^   term ^
```

**Multi-language support.** Translations are loaded at runtime using the same
syntax:

```
// ru.rlf
card = :fem { one: "карта", few: "карты", many: "карт" };
draw($n) = "Возьмите {$n} {@cap card:$n}.";
```

---

## Terms and Phrases

Every definition is either a **term** or a **phrase**. This distinction is
enforced by the compiler: using `()` on a term or `:` on a bare phrase name is
a compile error.

### Terms

A term has no parameters. It represents a lexical entry -- optionally with
variant forms and metadata tags.

```
// Simple text
hello = "Hello, world!";

// With a metadata tag
card = :a "card";

// With tags and variants
card = :a { one: "card", other: "cards" };

// Irregular forms
go = { present: "go", past: "went", participle: "gone" };
```

Terms are referenced by name in template bodies. A bare reference returns the
**default variant**: the variant marked with `*`, or the first declared variant
if no `*` is present:

```
card = { *one: "card", other: "cards" };
example = "{card}";   // -> "card" (default = *one)
```

### Phrases

A phrase has one or more `$`-prefixed parameters. Phrases are called with `()`
in template bodies:

```
// Simple template
energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";

// Calling other phrases
draw($n) = "Draw {cards($n)}.";

// Composing with transforms
dissolve($s) = "Dissolve {@a subtype($s)}.";
```

Phrase arguments can be parameters, term names, literal numbers, or literal
strings:

| Argument | Example | Value type |
|----------|---------|------------|
| `$param` | `cards($n)` | Whatever the parameter holds |
| `term_name` | `subtype(ancient)` | `Phrase` (with tags and variants) |
| `42` | `cards(2)` | Number |
| `"text"` | `trigger("Attack")` | String |

### Restrictions

| Syntax | Valid? | Why |
|--------|--------|-----|
| `{card:other}` | Yes | Selection on term |
| `{cards($n)}` | Yes | Phrase call |
| `{subtype($s):other}` | Yes | Selection on phrase call result |
| `{cards:other}` | **Error** | `cards` is a phrase -- use `cards(...):other` |
| `{card($n)}` | **Error** | `card` is a term -- use `card:$n` |
| `{f(g($x))}` | **Error** | Nested phrase calls not supported |
| `{f(card:one)}` | **Error** | Expressions not supported as arguments |

---

## Variants and Selection

### Variant blocks

A variant block provides multiple forms of a term, keyed by name:

```
card = { one: "card", other: "cards" };
go = { present: "go", past: "went", participle: "gone" };
```

Mark one variant with `*` as the default. If no `*`, the first variant is the
default:

```
card = { *one: "card", other: "cards" };
```

The `*` marker can only appear on top-level variant keys (not on
multi-dimensional keys like `nom.one`). Variant keys are named identifiers --
numeric keys are not supported in variant blocks (use `:match` for numeric
branching).

### The `:` selection operator

The `:` operator selects a variant by name. It works on terms, parameters
holding Phrase values, and phrase call results:

```
{card:other}            // select from term
{$entity:one}           // select from parameter
{subtype($s):other}     // select from phrase call result
```

It does **not** work on bare phrase names (without `()`).

### Parameterized selection

A `$`-prefixed parameter after `:` selects dynamically:

```
cards_numeral($n) = "{$n} {card:$n}";
// n=1 -> "1 card"   (CLDR maps 1 -> "one")
// n=5 -> "5 cards"  (CLDR maps 5 -> "other")
```

Resolution for `{term:$param}`:

1. If `$param` is a **number** -- map through CLDR plural rules, use as key
2. If `$param` is a **Phrase** -- iterate its tags, use first matching key
3. If `$param` is a **string** -- use directly as key

Parameterized selection on terms uses CLDR mapping only. It does **not** try
exact numeric keys -- use `:match` for exact-number matching.

Static and parameterized selectors can be mixed in a chain:

```
// Russian -- static case, parameterized number
draw($n) = "Возьмите {card:acc:$n}.";
// n=1 -> "Возьмите карту."   (acc + CLDR "one" -> acc.one)
// n=5 -> "Возьмите карт."    (acc + CLDR "many" -> acc.many)
```

### Multi-dimensional variants

Multi-dimensional variants use dot notation in declaration and colon chains
in selection:

```
// Declaration -- dot notation
card = :fem {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};

// Selection -- colon chains
example = "{card:acc:one}";   // -> "карту"
```

**Multi-key shorthand** assigns the same value to multiple keys:

```
card = {
    nom.one, acc.one: "card",
    nom.other, acc.other: "cards",
};
```

**Wildcard fallbacks** omit the final dimension:

```
card = {
    nom: "карта",        // fallback for nom.*
    nom.many: "карт",    // override for nom.many
};
```

These can be combined:

```
card = {
    nom, acc: "карты",     // fallback for nom.* and acc.*
    nom.many: "карт",      // override for nom.many
};
```

**Wildcard resolution.** When selecting a multi-dimensional variant: try the
exact key first (e.g., `nom.one`), then the prefix key (e.g., `nom`), then
error.

### Selection errors

Selecting a nonexistent variant is a **runtime error** -- RLF does not silently
fall back to empty strings or default variants on selection failure:

| Scenario | Error |
|----------|-------|
| Named variant not found (`{card:dat}` when no `dat` variant) | `MissingVariant` |
| Selection on a String or Number value | `MissingVariant` |
| No matching tag and no `*` default | `MissingVariant` |
| Wrong number of arguments | `ArityMismatch` |

Static selectors (e.g., `{card:dat}`) are caught at **compile time**.
Parameterized selection (`{card:$n}`) is validated at runtime.

---

## Transforms and Tags

### Metadata tags

Terms declare metadata tags with `:` before their content:

```
card = :a "card";
event = :an "event";
carta = :fem "carta";
character = :masc :anim "персонаж";
```

Tags serve three purposes:

1. **Transforms** read tags to produce correct output (e.g., `@a` reads
   `:a`/`:an`)
2. **Parameterized selection** reads tags from a Phrase parameter to pick a
   variant
3. **`:match` branches** read tags to select a branch

### The `@` transform operator

Transforms modify text. They apply right-to-left when chained:

```
card = :a "card";
draw_one = "Draw {@a card}.";    // -> "Draw a card."
title = "{@cap card}";           // -> "Card"
heading = "{@cap @a card}";      // -> "A card"
```

**Automatic capitalization:** An uppercase first letter adds an implicit `@cap`
transform. This works for term references and phrase calls, but not for
`$`-prefixed parameters (use `{@cap $name}` instead). The implicit `@cap` is
always placed as the **outermost** (leftmost) transform, so it runs last in
right-to-left evaluation and composes correctly with explicit transforms:

```
{Card}                // -> "Card"       (equivalent to {@cap card})
{Subtype($s)}         // -> "<b>Warrior</b>"  (equivalent to {@cap subtype($s)})
{@a Card}             // -> "A card"     (equivalent to {@cap @a card})
{@a Subtype($s)}      // -> "A warrior"  (equivalent to {@cap @a subtype($s)})
```

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
{@transform ref}                  // no context
{@transform:literal ref}         // static context
{@transform($param) ref}         // dynamic context
{@transform:literal($param) ref} // both (extremely rare)
```

Examples:

```
// German -- static context for grammatical case
destroy_card = "Zerstöre {@der:acc card}.";

// Chinese -- dynamic context for classifier
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

### Language-specific transforms

| Transform | Languages | Reads Tags | Effect |
|-----------|-----------|------------|--------|
| `@a` | English | `:a`, `:an` | Indefinite article |
| `@the` | English | -- | Definite article |
| `@plural` | English | -- | Select `other` variant |
| `@der` | German | `:masc`, `:fem`, `:neut` | Definite article + case |
| `@ein` | German | `:masc`, `:fem`, `:neut` | Indefinite article + case |
| `@el` | Spanish | `:masc`, `:fem` | Definite article |
| `@le` | French | `:masc`, `:fem`, `:vowel` | Definite article |
| `@un` | Spanish, French, Italian | `:masc`, `:fem` | Indefinite article |
| `@o` | Portuguese, Greek | `:masc`, `:fem` | Definite article |
| `@count` | CJK, Vietnamese, etc. | measure word tags | Measure word / classifier |
| `@inflect` | Turkish, Finnish, Hungarian | vowel harmony tags | Agglutinative suffix |

Aliases map alternative names to the canonical form: `@an` -> `@a`,
`@die` -> `@der`, `@la` -> `@el`, etc. See **APPENDIX_STDLIB.md** for
complete per-language documentation.

---

## Phrase Keywords: `:match` and `:from`

Phrases can use two keywords to control how parameters affect the output.

### `:match`

`:match` branches on a parameter value, like Rust's `match`. One branch must
be marked with `*` as the default:

```
cards($n) = :match($n) {
    1: "a card",
    *other: "{$n} cards",
};
```

**Numeric matching** tries exact number first, then CLDR plural category,
then the `*` default:

```
cards($n) = :match($n) {
    0: "no cards",
    1: "a card",
    *other: "{$n} cards",
};
// n=0 -> "no cards"   (exact)
// n=1 -> "a card"     (exact)
// n=5 -> "5 cards"    (CLDR "other" -> default)
```

Exact numeric keys are exclusive to `:match` -- parameterized selection on
terms (`{card:$n}`) only uses CLDR categories.

**Tag-based matching** reads tags from a Phrase parameter and selects the
first matching branch:

```
// Spanish
card = :fem "carta";
character = :masc "personaje";

destroyed($thing) = :match($thing) {
    masc: "destruido",
    *fem: "destruida",
};
// destroyed(card)      -> "destruida"   (tag :fem matches)
// destroyed(character) -> "destruido"   (tag :masc matches)
```

Tag order matters when a Phrase has multiple tags -- `:match` tries them in
declaration order and returns the first match.

**Multi-parameter matching** uses dot notation for branch keys. Each dimension
matches independently and must have one `*` default:

```
// Russian -- branch on both count and gender
n_allied($n, $entity) = :match($n, $entity) {
    1.masc: "союзный {$entity:nom:one}",
    1.fem: "союзная {$entity:nom:one}",
    1.*neut: "союзное {$entity:nom:one}",
    *other.masc: "{$n} союзных {$entity:gen:many}",
    other.fem: "{$n} союзных {$entity:gen:many}",
    other.*neut: "{$n} союзных {$entity:gen:many}",
};
```

**Restrictions:** A phrase with a variant block **must** use `:match`. Match
branches support both numeric keys (`0`, `1`, `2`) and named keys (`one`,
`other`, `masc`). No negative numbers or floats as match keys.

### `:from`

`:from($param)` causes a phrase to inherit tags and variants from a parameter,
enabling phrase-to-phrase composition that preserves grammatical metadata:

```
ancient = :an { one: "Ancient", other: "Ancients" };
child = :a { one: "Child", other: "Children" };

subtype($s) = :from($s) "<b>{$s}</b>";
```

`subtype(ancient)` evaluates the template once per variant of `ancient` and
returns a Phrase with the same tags and variant structure:

- Tags: `["an"]`
- Variants: `{one: "<b>Ancient</b>", other: "<b>Ancients</b>"}`

The result can be selected and transformed like any other Phrase:

```
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
dissolve_all($s) = "Dissolve all {subtype($s):other}.";
// dissolve_subtype(ancient) -> "Dissolve an <b>Ancient</b>."
// dissolve_all(ancient)     -> "Dissolve all <b>Ancients</b>."
```

`:from` and `:match` can be combined. `:from` determines the inherited
structure, `:match` branches within each variant's evaluation:

```
count_subtype($n, $s) = :from($s) :match($n) {
    1: "союзный {subtype($s)}",
    *other: "{$n} союзных {subtype($s):gen:many}",
};
```

The declaration order of `:from` and `:match` does not matter.

---

## Rust Integration

### Generated API

The `rlf!` macro generates one function per definition:

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    cards($n) = :match($n) { 1: "a card", *other: "{$n} cards" };
    draw($n) = "Draw {cards($n)}.";
}
```

Generates:

```rust
/// Returns the "card" term.
pub fn card(locale: &Locale) -> Phrase { ... }

/// Evaluates the "cards" phrase with parameter n.
pub fn cards(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }

/// Evaluates the "draw" phrase with parameter n.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase { ... }

/// Registers source language definitions with the locale. Call once at startup.
pub fn register_source_phrases(locale: &mut Locale) { ... }
```

All functions return `Phrase`. Generated functions panic on errors (indicating
programming mistakes). See **APPENDIX_RUST_INTEGRATION.md** for complete
documentation.

### The Locale object

`Locale` manages language selection and translation data:

```rust
let mut locale = Locale::new();
strings::register_source_phrases(&mut locale);
locale.load_translations("ru", "assets/localization/ru.rlf")?;
locale.set_language("ru");
```

See **APPENDIX_RUNTIME_INTERPRETER.md** for complete API documentation.

### Global Locale

The `global-locale` Cargo feature stores the locale in global state, removing
the `locale` parameter from all generated functions:

```toml
[dependencies]
rlf = { version = "0.1", features = ["global-locale"] }
```

```rust
// Without global-locale:
strings::card(&locale);

// With global-locale (auto-registers on first use):
strings::card();
```

API: `set_language(lang)`, `language()`, `with_locale(|locale| ...)`,
`with_locale_mut(|locale| ...)`.

### The Phrase type

```rust
pub struct Phrase {
    pub text: String,
    pub variants: HashMap<VariantKey, String>,
    pub tags: Vec<Tag>,
}
```

`Phrase` implements `Display` (returns `text`). Use `variant()` to access
specific forms:

```rust
let card = strings::card(&locale);
card.to_string();           // "card"
card.variant("other");      // "cards"
```

### PhraseId

`PhraseId` is a compact, `Copy`-able, 16-byte identifier for storing
references to definitions in serializable data:

```rust
let id: PhraseId = strings::phrase_ids::DRAW;
let phrase = id.call(&locale, &[3.into()])?;
```

See **APPENDIX_RUST_INTEGRATION.md** for complete details.

### Runtime templates

For data-driven content, evaluate templates directly via `Locale`:

```rust
let template = "Draw {cards($n)} for each {$target}.";
let params = params!{ "n" => 2, "target" => strings::ally(&locale) };
let phrase: Phrase = locale.eval_str(template, params)?;
```

`Locale` methods return `Result` (unlike generated functions which panic).

### Runtime values

All parameters accept a `Value` type:

```rust
strings::cards(&locale, 3);                          // number
strings::greet(&locale, "World");                    // string
strings::destroy(&locale, strings::card(&locale));   // phrase
```

---

## Errors

### Compile-time errors

The `rlf!` macro validates source definitions at compile time:

- Unknown term, phrase, or parameter references
- Cyclic references between definitions
- Parameter shadowing (name matches a term or phrase)
- Term/phrase misuse (`()` on a term, `:` on a bare phrase name)
- Missing `*` default in `:match` blocks
- Arity mismatch in phrase calls
- Static selection on a nonexistent variant (e.g., `{card:dat}`)

### Runtime errors

Generated functions panic on errors. `Locale` methods return `Result`.

**No language fallback.** If a definition exists in English but not in Russian,
requesting the Russian version returns `PhraseNotFound` -- it does not fall
back to English. Translations must be complete.

See **APPENDIX_RUST_INTEGRATION.md** for the full error type definitions.

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
```

Translation files use the same syntax as source files:

```
// ru.rlf
hello = "Привет, мир!";
card = :fem { one: "карта", few: "карты", many: "карт" };
cards($n) = :match($n) {
    1: "карту",
    *other: "{$n} карт",
};
```

### Translation workflow

1. Add a definition to `strings.rlf.rs` -- autocomplete works immediately
2. Use it in Rust code
3. Add translations to `.rlf` files (changes take effect without recompilation)

---

## Escape Sequences

`:`, `@`, and `$` are only special inside `{}` expressions. In regular text,
they are literal:

```
help = "Dissolve: Send a character to the void";
email = "user@example.com";
price = "The cost is $5.";
```

Only `{` and `}` need escaping in text:

| Sequence | Output | Where |
|----------|--------|-------|
| `{{` | `{` | Anywhere in text |
| `}}` | `}` | Anywhere in text |
| `::` | `:` | Inside `{}` only |
| `@@` | `@` | Inside `{}` only |
| `$$` | `$` | Inside `{}` only |

---

## Design Philosophy

**Unified interpreter, compile-time validation.** All languages are evaluated
by the interpreter at runtime. The source language gets full compile-time
checking via the macro. Translations are loaded at runtime, enabling
hot-reloading and community translations.

**Immediate IDE support.** Add a definition, get autocomplete. No external
tools, no build steps.

**Language-agnostic API.** Functions take a locale parameter. The same code
works for all languages -- Rust identifies what to say, RLF handles how to
say it.

**Pass Phrase, not String.** When composing definitions, pass `Phrase` values
rather than pre-rendered strings. This preserves variants and tags so RLF can
select the correct grammatical form.

**Logic in Rust, text in RLF.** Complex branching stays in Rust. RLF provides
atomic text pieces. Translators don't need to understand Rust.

---

## Summary

| Primitive | Syntax | Purpose |
|-----------|--------|---------|
| Term | `name = "text";` | Named text with optional variants |
| Phrase | `name($p) = "{$p}";` | Parameterized template |
| Variant | `{ a: "x", b: "y" }` | Multiple forms of a term |
| Selection | `{term:key}` | Choose a variant |
| Tag | `:tag` | Attach metadata |
| Transform | `{@transform ref}` | Modify text |
| `:match` | `:match($p) { ... }` | Branch on parameter value |
| `:from` | `:from($p)` | Inherit tags/variants from parameter |

| Component | Compile-Time | Runtime |
|-----------|-------------|---------|
| Source language | Full validation | Interpreter evaluation |
| Translations | -- | Interpreter evaluation |
