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
    draw(n) = "Draw {n} {card:n}.";
}
```

This generates functions that take a `Locale` parameter:

```rust
// Generated API - usage
let mut locale = Locale::with_language("en");
strings::hello(&locale);       // → "Hello, world!"
strings::draw(&locale, 3);     // → "Draw 3 cards."

// Switch to Russian
locale.set_language("ru");
strings::draw(&locale, 3);     // → "Возьмите 3 карты." (via interpreter)
```

**How it works:**

1. The `rlf!` macro parses the source language and generates one function per phrase
2. The macro also embeds the source phrases as data for the interpreter
3. At startup, the source phrases are registered with the interpreter
4. All evaluation (source and translations) goes through the interpreter

**Key benefit:** When you add a new phrase to `strings.rlf.rs`, it immediately
appears in IDE autocomplete for all Rust code. No build steps, no external tools—
just write the phrase and use it.

---

## Primitives

RLF has four primitives: **phrase**, **parameter**, **variant**, and
**selection**.

### Phrase

A phrase has a name and produces text.

```rust
rlf! {
    hello = "Hello, world!";
    goodbye = "Goodbye!";
}
```

### Parameter

Phrases can accept values. Parameters are declared in parentheses and
interpolated with `{}`.

```rust
rlf! {
    greet(name) = "Hello, {name}!";
    damage(amount, target) = "Deal {amount} damage to {target}.";
}
```

### Variant

A phrase can have multiple forms. Variants are declared in braces after `=`.

```rust
rlf! {
    card = {
        one: "card",
        other: "cards",
    };
}
```

Variants can be multi-dimensional using dot notation:

```rust
// In ru.rlf
card = {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};
```

**Multi-key shorthand:** Assign the same value to multiple keys with commas:

```rust
rlf! {
    card = {
        nom.one, acc.one: "card",
        nom.other, acc.other: "cards",
    };
}
```

**Wildcard fallbacks:** Omit the final dimension to create a fallback:

```rust
// In ru.rlf
card = {
    nom: "карта",        // Fallback for nom.one, nom.few, etc.
    nom.many: "карт",    // Override for nom.many specifically
    acc: "карту",
    acc.many: "карт",
};
```

Resolution order: exact match (`nom.many`) → progressively shorter fallbacks
(`nom`). If no match is found, RLF produces a **runtime error**.

**Irregular forms:** Use variants for unpredictable forms:

```rust
rlf! {
    go = { present: "go", past: "went", participle: "gone" };
    good = { base: "good", comparative: "better", superlative: "best" };
}
```

### Selection

The `:` operator selects a variant.

Literal selection uses a variant name directly:

```rust
rlf! {
    all_cards = "All {card:other}.";
}

// In ru.rlf
take_one = "Возьмите {card:acc.one}.";
```

Derived selection uses a parameter. For numbers, RLF maps to CLDR plural
categories (`zero`, `one`, `two`, `few`, `many`, `other`):

```rust
rlf! {
    draw(n) = "Draw {n} {card:n}.";
}
// n=1 → "Draw 1 card."
// n=5 → "Draw 5 cards."
```

**Escape sequences:** Use doubled characters for literals:

```rust
rlf! {
    syntax_help = "Use {{name}} for interpolation and @@ for transforms.";
    ratio = "The ratio is 1::2.";
}
// → "Use {name} for interpolation and @ for transforms."
// → "The ratio is 1:2."
```

Multi-dimensional selection chains with multiple `:` operators:

```rust
// In ru.rlf
draw(n) = "Возьмите {n} {card:acc:n}.";
// n=1 → "Возьмите 1 карту."
// n=5 → "Возьмите 5 карт."
```

**Selection on phrase parameters:**

```rust
rlf! {
    character = { one: "character", other: "characters" };
    with_cost_less_than_allies(base, counting) =
        "{base} with cost less than the number of allied {counting:other}";
}
// counting=character → "... allied characters"
```

**Dynamic selection:**

```rust
rlf! {
    character = { one: "character", other: "characters" };
    card = { one: "card", other: "cards" };
    draw_entities(n, entity) = "Draw {n} {entity:n}.";
}
// draw_entities(1, character) → "Draw 1 character."
// draw_entities(3, card) → "Draw 3 cards."
```

---

## Metadata Tags

A phrase can declare metadata tags using `:` before its content:

```rust
// In es.rlf
card = :fem "carta";
character = :masc "personaje";
```

Tags serve two purposes:

1. **Selection**: Other phrases can select variants based on tags
2. **Transforms**: Transforms can read tags to determine behavior

**Multiple tags:**

```rust
rlf! {
    // English: article hint for @a transform
    card = :a "card";
    event = :an "event";
    uniform = :a "uniform";   // phonetic exception
    hour = :an "hour";        // silent h exception
}

// In de.rlf (German)
karte = :fem "Karte";
charakter = :masc "Charakter";
ereignis = :neut "Ereignis";

// In zh_cn.rlf (Chinese)
pai = :zhang "牌";
jue_se = :ge "角色";
```

**Selection based on tags:**

```rust
// In es.rlf
card = :fem "carta";
character = :masc "personaje";

destroyed = {
    masc: "destruido",
    fem: "destruida",
};

destroy(thing) = "{thing} fue {destroyed:thing}.";
// thing=card      → "carta fue destruida."
// thing=character → "personaje fue destruido."
```

---

## Metadata Inheritance

The `:from(param)` modifier causes a phrase to inherit tags and variants from
a parameter. This enables **phrase-returning phrases**—functions that transform
one phrase into another while preserving grammatical metadata.

```rust
rlf! {
    ancient = :an { one: "Ancient", other: "Ancients" };
    child = :a { one: "Child", other: "Children" };

    // :from(s) inherits tags and variants from parameter s
    subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
}
```

**How `:from(s)` works:**

1. Read tags from `s` (e.g., `[:an]` from `ancient`)
2. Read variants from `s` (e.g., `{one: "Ancient", other: "Ancients"}`)
3. Evaluate template once per variant, substituting `{s}` with that variant's text
4. Return a `Phrase` with inherited tags and computed variants

**Evaluation of `subtype(ancient)`:**

| Step | Action | Result |
|------|--------|--------|
| 1 | Read `ancient.tags` | `["an"]` |
| 2 | Evaluate with `s:one` | `"<color=#2E7D32><b>Ancient</b></color>"` |
| 3 | Evaluate with `s:other` | `"<color=#2E7D32><b>Ancients</b></color>"` |
| 4 | Return Phrase | Tags: `["an"]`, variants: `{one: "...", other: "..."}` |

**Usage in templates:**

```rust
rlf! {
    dissolve_subtype(s) = "Dissolve {@a subtype(s)}.";
    dissolve_all(s) = "Dissolve all {subtype(s):other}.";
}
// dissolve_subtype(ancient) → "Dissolve an <b>Ancient</b>."
// dissolve_all(ancient) → "Dissolve all <b>Ancients</b>."
```

The `@a` transform reads the inherited `:an` tag; the `:other` selector accesses
the inherited variant. This enables composition without losing grammatical
information.

**Caller responsibility:** Per the "Pass Phrase, not String" principle, callers
must pass actual `Phrase` values, not string keys:

```rust
// Correct: pass Phrase
let ancient = strings::ancient(locale);
strings::dissolve_subtype(locale, ancient)

// For data-driven templates, resolve the name first
let key = "ancient";  // from card data
let phrase = locale.interpreter().get_phrase(locale.language(), key)?;
strings::dissolve_subtype(locale, phrase)
```

---

## Transforms

The `@` operator applies a transform. Transforms modify text and apply
right-to-left when chained:

```rust
rlf! {
    card = "card";
    draw_one = "Draw {@a card}.";        // → "Draw a card."
    title = "{@cap card}";               // → "Card"
    heading = "{@cap @a card}";          // → "A card"
}
```

**Automatic capitalization:** Referencing a phrase with an uppercase first letter
applies `@cap` automatically: `{Card}` is equivalent to `{@cap card}`.

Transforms combine with selection:

```rust
rlf! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {@cap card:n}.";
}
// n=1 → "Draw 1 Card."
// n=3 → "Draw 3 Cards."
```

**Transform context:** Some transforms need additional information. Transforms
can take an optional `:context` immediately after their name. This is separate
from phrase selection—transforms never have selectors, only context:

```
{@transform:context phrase:selector}
     │         │      │       │
     │         │      │       └─ selects variant from phrase
     │         │      └───────── phrase or parameter reference
     │         └──────────────── context for the transform
     └────────────────────────── transform name
```

Examples:

```rust
// In de.rlf
destroy_card = "Zerstöre {@der:acc karte}.";      // :acc is transform context

// In es.rlf
return_all(t) = "devuelve {@el:other t} a mano";  // :other is transform context

// In zh_cn.rlf
draw(n) = "抽{@count:n card}";                    // :n is transform context (parameter)
```

When both are present, context comes first:

```rust
get_card = "Nimm {@der:acc karte:one}.";   // :acc is context; :one selects variant
```

### Universal Transforms

| Transform | Effect                   |
| --------- | ------------------------ |
| `@cap`    | Capitalize first letter  |
| `@upper`  | All uppercase            |
| `@lower`  | All lowercase            |

### Metadata-Driven Transforms

Language-specific transforms read metadata tags:

```rust
rlf! {
    card = :a "card";
    event = :an "event";
    draw_one = "Draw {@a card}.";   // → "Draw a card."
    play_one = "Play {@a event}.";  // → "Play an event."
}
```

The `@a` transform reads the `:a` or `:an` tag. Missing tags produce runtime
errors—no phonetic guessing.

Standard transforms per language:

| Transform   | Languages              | Reads Tags                   | Effect                     |
| ----------- | ---------------------- | ---------------------------- | -------------------------- |
| `@a`        | English                | `:a`, `:an`                  | Indefinite article         |
| `@der`      | German                 | `:masc`, `:fem`, `:neut`     | Definite article + case    |
| `@el`       | Spanish                | `:masc`, `:fem`              | Definite article           |
| `@le`       | French                 | `:masc`, `:fem`, `:vowel`    | Definite article           |
| `@un`       | Romance                | `:masc`, `:fem`              | Indefinite article         |
| `@count`    | CJK                    | measure word tags            | Measure word insertion     |

**Transform aliases:** `@an` → `@a`, `@die` → `@der`, `@la` → `@el`, etc.

See **APPENDIX_STDLIB.md** for complete documentation.

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
draw(n) = "Возьмите {n} {card:n}.";
```

---

## The Locale Object

The `Locale` object manages language selection and translation data:

```rust
fn setup_localization() -> Locale {
    let mut locale = Locale::new();
    strings::register_source_phrases(locale.interpreter_mut());
    locale.load_translations("ru", "assets/localization/ru.rlf")?;
    locale.load_translations("es", "assets/localization/es.rlf")?;
    locale.set_language(&user_preferences.language);
    locale
}
```

See **APPENDIX_RUNTIME_INTERPRETER.md** for complete API documentation.

---

## Generated API

Given:

```rust
// strings.rlf.rs
rlf! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}
```

RLF generates:

```rust
// strings.rs (generated)

/// Returns the "card" phrase.
pub fn card(locale: &Locale) -> Phrase {
    locale.interpreter()
        .get_phrase(locale.language(), "card")
        .expect("phrase 'card' should exist")
}

/// Evaluates the "draw" phrase with parameter n.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase {
    locale.interpreter()
        .call_phrase(locale.language(), "draw", &[n.into()])
        .expect("phrase 'draw' should exist")
}

/// Registers source language phrases with the interpreter.
/// Call once at startup.
pub fn register_source_phrases(interpreter: &mut RlfInterpreter) {
    interpreter.load_phrases("en", SOURCE_PHRASES)
        .expect("source phrases should parse successfully");
}

const SOURCE_PHRASES: &str = r#"
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
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

**Note:** All phrase functions return `Phrase`. For phrases without `:from` or
declared variants/tags, the returned `Phrase` has empty variants and tags,
behaving like a simple string.

---

## Runtime Templates

For data-driven content (templates stored in data files), use the interpreter directly:

```rust
let template = "Draw {cards(n)} for each {target}.";
let params = hashmap!{ "n" => 2, "target" => strings::ally(&locale) };
locale.interpreter().eval_str(template, locale.language(), params)?
```

Parameters work identically to phrase parameters. See **APPENDIX_RUNTIME_INTERPRETER.md**.

---

## Runtime Values

All parameters accept a `Value` type:

```rust
strings::draw(&locale, 3);                     // number
strings::draw(&locale, "3");                   // string
strings::greet(&locale, "World");              // string
strings::destroy(&locale, strings::card(&locale));  // phrase
```

**Runtime behavior:**

| Operation              | Value Type      | Behavior                                    |
| ---------------------- | --------------- | ------------------------------------------- |
| `{x}`                  | Any             | Display the value                           |
| `{card:x}` (selection) | Number          | Select plural category                      |
| `{card:x}` (selection) | String          | Parse as number, or error                   |
| `{card:x}` (selection) | Phrase          | Look up matching tag                        |
| `{@a x}`               | Phrase with tag | Use the tag                                 |
| `{@a x}`               | Other           | **Runtime error**                           |

---

## Compile-Time Errors

RLF validates the source file at compile time:

**Unknown phrase:**

```rust
rlf! {
    draw(n) = "Draw {n} {cards:n}.";  // typo
}
```

```
error: unknown phrase 'cards'
  --> strings.rlf.rs:2:28
   |
   = help: did you mean 'card'?
```

**Unknown parameter:**

```rust
rlf! {
    draw(n) = "Draw {count} {card:n}.";
}
```

```
error: unknown parameter 'count'
  --> strings.rlf.rs:2:18
   |
   = help: declared parameters: n
```

**Additional compile-time checks:**

- **Cyclic references**: Phrases that reference each other in a cycle are rejected
- **Parameter shadowing**: A parameter cannot have the same name as a phrase

**Translation files** are validated at load time, not compile time. Load errors
include the file path and line number.

---

## Runtime Errors

RLF uses a two-layer error model:

**Generated functions** (from `rlf!`) use `.expect()` and **panic** on errors.
These are for static phrases where errors indicate programming mistakes:

```rust
strings::draw(&locale, 3);  // Panics if "draw" phrase is missing or malformed
```

**Interpreter methods** return `Result` for data-driven content where errors
may come from external data:

```rust
locale.interpreter().eval_str(template, lang, params)?;  // Returns Result
```

Use the interpreter API directly when evaluating templates from TOML, JSON, or
other data files. This lets you handle errors gracefully rather than panicking.

**No language fallback:** If a phrase exists in English but not in Russian,
requesting the Russian version returns `PhraseNotFound`—it does not fall back
to English. Translations must be complete; missing phrases are errors.

See **APPENDIX_RUST_INTEGRATION.md** for the full error type definitions.

---

## Phrase Type

All phrase functions return a `Phrase` that carries metadata:

```rust
pub struct Phrase {
    /// Default text.
    pub text: String,
    /// Variant key → variant text.
    pub variants: HashMap<String, String>,
    /// Metadata tags.
    pub tags: Vec<String>,
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

Use `variant()` to access specific forms:

```rust
let card = strings::card(&locale);
let singular = card.to_string();            // "card"
let plural = card.variant("other");         // "cards"
```

---

## Phrase Identifiers

For scenarios where you need to store a reference to a phrase in serializable
data structures, RLF provides `PhraseId`—a compact, `Copy`-able, 8-byte
identifier based on a hash of the phrase name. The `rlf!` macro generates
`PhraseId` constants for all phrases.

```rust
// Store in serializable data
let card_name: PhraseId = strings::phrase_ids::FIRE_ELEMENTAL;
let draw_phrase: PhraseId = strings::phrase_ids::DRAW;

// Resolve parameterless phrase (returns Result<Phrase, EvalError>)
let phrase = card_name.resolve(&locale).expect("phrase should exist");
let text = phrase.to_string();  // → "Fire Elemental"

// Resolve phrase with parameters (returns Result<String, EvalError>)
let text = draw_phrase.call(&locale, &[3.into()])
    .expect("phrase should exist");  // → "Draw 3 cards."
```

See **APPENDIX_RUST_INTEGRATION.md** for complete details on `PhraseId`
generation, API, and usage patterns.

---

## Design Philosophy

**Unified interpreter, compile-time validation.** All languages (including the
source) are evaluated by the interpreter at runtime. The source language gets
full compile-time syntax and reference checking via the macro. Translations are
loaded at runtime, enabling hot-reloading and community translations.

**Immediate IDE support.** When you add a phrase to `strings.rlf.rs`, it
appears in autocomplete immediately. No external tools, no build steps.

**Language-agnostic API.** Functions take a locale parameter. The same code
works for all languages—Rust identifies what to say, RLF handles how to say it.

**Pass Phrase, not String.** When composing phrases, pass `Phrase` values rather
than pre-rendered strings. This preserves variants and tags so RLF can select
the correct grammatical form. Pre-rendering to `String` strips this metadata.

**Logic in Rust, text in RLF.** Complex branching stays in Rust; RLF provides
atomic text pieces. Translators don't need to understand Rust.

**Keywords and formatting are phrases.** No special syntax—define phrases with
markup (`dissolve = "<k>dissolve</k>";`) and interpolate normally.

**Dynamic typing for simplicity.** Parameters accept any `Value`. Runtime errors
catch type mismatches. Translators don't need Rust types.

---

## Translation Workflow

### Adding a New Phrase

1. Add the phrase to `strings.rlf.rs`:
   ```rust
   rlf! {
       new_ability(n) = "Gain {n} {point:n}.";
   }
   ```

2. Use it immediately in Rust code (autocomplete works):
   ```rust
   let text = strings::new_ability(&locale, 5);
   ```

3. Later, add translations to `.rlf` files:
   ```
   // ru.rlf
   new_ability(n) = "Получите {n} {point:n}.";
   ```

### Updating Translations

1. Edit the `.rlf` file
2. Reload in development (if hot-reload enabled) or restart
3. Changes take effect without recompilation

### Command-Line Tools

The `rlf` binary provides utilities for working with translation files:

```bash
# Validate syntax
rlf check assets/localization/ru.rlf

# Check coverage against source
rlf coverage --source strings.rlf.rs --lang ru,es,zh_cn

# Evaluate a template interactively
rlf eval --lang ru --param n=3 --template "Draw {n} {card:n}."
```

---

## Summary

| Primitive    | Syntax                         | Purpose                                 |
| ------------ | ------------------------------ | --------------------------------------- |
| Phrase       | `name = "text";`               | Define text                             |
| Parameter    | `name(p) = "{p}";`             | Accept values                           |
| Variant      | `name = { a: "x", b: "y" };`   | Multiple forms                          |
| Selection    | `{phrase:selector}`            | Choose a variant                        |
| Metadata tag | `name = :tag "text";`          | Attach metadata                         |
| Inheritance  | `name(p) = :from(p) "{p}";`    | Inherit tags/variants from parameter    |
| Transform    | `{@transform:ctx phrase}`      | Modify text                             |

| File Type        | Extension    | Purpose                               |
| ---------------- | ------------ | ------------------------------------- |
| Source language  | `.rlf.rs`    | Compiled via `rlf!` macro             |
| Translations     | `.rlf`       | Loaded at runtime via interpreter     |

| Type             | Purpose                                | Size / Traits                       |
| ---------------- | -------------------------------------- | ----------------------------------- |
| `Phrase`         | Returned by all phrase functions       | Heap-allocated                      |
| `PhraseId`       | Serializable reference to a phrase     | 8 bytes, `Copy`, `Serialize`        |
| `Value`          | Runtime parameter (number/string/phrase) | Enum                              |

| Component        | Compile-Time              | Runtime                             |
| ---------------- | ------------------------- | ----------------------------------- |
| Source language  | Full validation           | Interpreter evaluation              |
| Translations     | (optional strict check)   | Interpreter evaluation              |

Four primitives, one macro, Rust-compatible syntax, compile-time checking for
the source language, runtime loading for translations, immediate IDE support.
