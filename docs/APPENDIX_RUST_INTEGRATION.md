# Appendix: Rust Integration

This appendix describes how the `rlf!` macro is implemented, what Rust code it
generates, and how errors are reported.

## Macro Architecture

RLF uses a single Rust procedural macro to parse `.rlf.rs` files and generate
Rust code:

- **`rlf!`**: Parses phrase definitions and generates locale-aware functions

The process has three phases:

1. **Parsing**: Extract phrase definitions from the macro input
2. **Validation**: Check names, detect undefined references
3. **Code Generation**: Emit Rust functions that check locale and dispatch

---

## Phase 1: Parsing

The macro receives tokens from inside the `rlf! { ... }` block. It parses these
into an internal representation:

The macro parses each phrase into a `PhraseDefinition` containing:

- A **name** (identifier with span information for error reporting)
- A list of **parameters** (identifiers)
- **Metadata tags** (e.g., `:a`, `:fem`)
- An optional **`:from` parameter** for metadata inheritance
- A **body**: either a `Simple` template or `Variants` (a list of
  `VariantEntry` values, each with one or more keys and a template)

Each `Template` contains a list of `Segment` values: either `Literal` text or
`Interpolation` blocks. Interpolations contain transforms, a `Reference` (either
an `Identifier` or a `Call` with arguments), and selectors. At parse time,
parameters and phrases are not distinguished -- both are represented as
`Identifier`. Resolution happens during validation and evaluation.

### Parsing Interpolations

The parser handles `{...}` blocks within template strings:

| Syntax           | Parsed As                          |
| ---------------- | ---------------------------------- |
| `{name}`         | Parameter or phrase reference      |
| `{card:n}`       | Reference with selector            |
| `{card:acc:n}`   | Reference with chained selectors   |
| `{@cap name}`    | Transform applied to reference     |
| `{@cap @a card}` | Chained transforms                 |
| `{foo(x, y)}`    | Phrase call with arguments         |

### Variant Key Parsing

Variant keys can be simple or multi-dimensional:

```rust
// Simple variants
card = { one: "card", other: "cards" };
// Parsed as: [("one", "card"), ("other", "cards")]

// Multi-dimensional variants
card = { nom.one: "карта", nom.few: "карты", acc.one: "карту", ... };
// Parsed as: [("nom.one", "карта"), ("nom.few", "карты"), ...]
```

### Multi-Key and Wildcard Parsing

Multi-key shorthand expands during parsing:

```rust
// Multi-key syntax
card = { nom, acc: "card", nom.other, acc.other: "cards" };
// Expands to: [("nom", "card"), ("acc", "card"), ("nom.other", "cards"), ...]
```

Wildcard fallbacks use partial keys:

```rust
// Wildcard syntax
card = { nom: "card", nom.other: "cards" };
// At runtime: nom.one → try "nom.one" (miss) → try "nom" (hit) → "card"
```

---

## Phase 2: Validation

After parsing, the macro performs comprehensive compile-time validation. This
catches errors early, before code generation.

### Name Resolution

Within phrase text, names in `{}` are resolved:

1. **Parameters first**: If a name matches a declared parameter, it refers to that parameter
2. **Phrases second**: Otherwise, it refers to a phrase defined in the file

**No shadowing allowed:** It is a compile error for a parameter to have the same
name as a phrase:

```rust
rlf! {
    card = "card";

    // ERROR: parameter 'card' shadows phrase 'card'
    play(card) = "Play {card}.";

    // OK: use a different parameter name
    play(c) = "Play {c}.";
}
```

**Selectors follow the same rules:** A selector like `:n` is dynamic (parameter)
if `n` is in the parameter list, otherwise it's a literal variant name.

```rust
rlf! {
    card = { one: "card", other: "cards" };

    // 'other' is literal (no parameter named 'other')
    all_cards = "All {card:other}.";

    // 'n' is dynamic (matches parameter)
    draw(n) = "Draw {card:n}.";
}
```

### Validation Checks

The validator performs these compile-time checks:

#### 1. Phrase Reference Validation

Every `{phrase_name}` must refer to a defined phrase:

```rust
rlf! {
    draw(n) = "Draw {n} {cards:n}.";  // ERROR: 'cards' not defined
}
```

#### 2. Parameter Reference Validation

Every `{param}` must be in the phrase's parameter list:

```rust
rlf! {
    draw(n) = "Draw {count} cards.";  // ERROR: 'count' not a parameter
}
```

#### 3. Literal Selector Validation

Literal selectors must match defined variants:

```rust
rlf! {
    card = { one: "card", other: "cards" };
    take = "{card:accusative}";  // ERROR: no 'accusative' variant
}
```

#### 4. Transform Existence Validation

Every `@transform` must be a known transform:

```rust
rlf! {
    card = "card";
    bad = "{@foo card}";  // ERROR: unknown transform '@foo'
}
```

The macro knows about:
- Universal transforms: `@cap`, `@upper`, `@lower`
- Source language transforms: `@a`, `@an` (English), etc.

#### 5. Transform Tag Validation (Literal Arguments)

When a transform is applied to a literal phrase reference, the macro verifies
the phrase has the required tags:

```rust
rlf! {
    card = "card";  // Missing :a or :an tag
    draw = "Draw {@a card}.";  // ERROR: '@a' requires tag ':a' or ':an'
}
```

This check only applies when the transform argument is a literal phrase. When
the argument is a parameter, the check must be deferred to runtime:

```rust
rlf! {
    card = :a "card";
    event = :an "event";

    // Compile-time check: 'card' has :a tag ✓
    draw_card = "Draw {@a card}.";

    // Runtime check: can't know what 'thing' will be
    draw(thing) = "Draw {@a thing}.";
}
```

#### 6. Tag-Based Selection Validation (Literal Arguments)

When selecting variants based on a phrase's tag, and the selector phrase is
a literal reference, the macro verifies compatibility:

```rust
rlf! {
    destroyed = { masc: "destroyed", fem: "destroyed" };
    card = :neut "card";  // Has :neut, not :masc or :fem

    // ERROR: 'card' has tag ':neut' but 'destroyed' has no 'neut' variant
    bad = "{destroyed:card}";
}
```

When the selector is a parameter, this must be a runtime check:

```rust
rlf! {
    destroyed = { masc: "destroyed", fem: "destroyed" };
    card = :fem "card";
    enemy = :masc "enemy";

    // Compile-time check: 'card' has :fem, 'destroyed' has 'fem' ✓
    card_destroyed = "{destroyed:card}";

    // Runtime check: can't know what 'target' will be
    destroy(target) = "{destroyed:target}";
}
```

#### 7. Cyclic Reference Detection

The macro detects cycles in phrase references:

```rust
rlf! {
    a = "see {b}";
    b = "see {c}";
    c = "see {a}";  // ERROR: cyclic reference: a -> b -> c -> a
}
```

The validator builds a dependency graph and performs cycle detection before
code generation.

### Validation Summary

| Check | Literal Phrase | Parameter | Status |
|-------|----------------|-----------|--------|
| Phrase exists | ✓ Compile | ✓ Compile | Error |
| Parameter exists | ✓ Compile | ✓ Compile | Error |
| Literal selector valid | ✓ Compile | N/A | Error |
| Transform exists | ✓ Compile | ✓ Compile | Error |
| Transform has required tag | ✓ Compile | Runtime | Error |
| Tag-based selection compatible | ✓ Compile | Runtime | Error |
| No cyclic references | ✓ Compile | ✓ Compile | Error |

---

## Phase 3: Code Generation

Based on the validated AST, the macro generates Rust functions and embeds the
source phrases as data. Each phrase becomes a function that delegates to the
interpreter.

### The Value Type

All parameters use a single `Value` type:

```rust
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Phrase(Phrase),
}
```

`Value` provides methods for runtime operations: `as_number()` for plural
selection, `has_tag(&str)` and `get_variant(&str)` for tag-based selection.

### The Phrase Type

Phrases without parameters return a `Phrase`:

```rust
pub struct Phrase {
    /// Default text.
    pub text: String,
    /// Variant key → variant text.
    pub variants: HashMap<VariantKey, String>,
    /// Metadata tags.
    pub tags: Vec<Tag>,
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution. Panics if not found.
    pub fn variant(&self, key: &str) -> &str;

    /// Check if this phrase has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool;

    /// Get the first tag, if any.
    pub fn first_tag(&self) -> Option<&Tag>;
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<Phrase> for String {
    fn from(phrase: Phrase) -> Self {
        phrase.text
    }
}
```

`VariantKey` and `Tag` are newtype wrappers around `String` with `Deref<Target=str>`, `From<&str>`, and `Display`.

### Into<Value> Implementations

Common types implement `Into<Value>`: integers become `Value::Number`, strings
become `Value::String`, and `Phrase` becomes `Value::Phrase`.

### The params! Macro

The `params!` macro creates a `HashMap<String, Value>` from key-value pairs,
with automatic `Into<Value>` conversion for each value:

```rust
use rlf::{params, Value};

// Empty map
let empty = params! {};

// Single entry
let p = params! { "count" => 3 };

// Multiple entries with mixed types
let p = params! {
    "count" => 5,
    "name" => "Alice",
    "score" => 9.5_f64,
};
assert_eq!(p["count"].as_number(), Some(5));
assert_eq!(p["name"].as_string(), Some("Alice"));
assert_eq!(p["score"].as_float(), Some(9.5));
```

Accepted value types include integers (`i32`, `i64`, `u32`, `u64`, `usize`),
floats (`f32`, `f64`), string literals (`&str`), owned strings (`String`),
`Phrase` values, and `Value` directly. Keys can be any expression that
implements `ToString` (typically `&str`).

---

## The PhraseId Type

`PhraseId` provides a compact, serializable reference to any phrase that can be
resolved at runtime. This is useful for storing phrase references in data
structures like game state, card definitions, or network messages—then supplying
parameters when the text is actually needed.

### Design

```rust
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhraseId(u128);
```

`PhraseId` wraps a 128-bit FNV-1a hash of the phrase name. This design provides:

| Property | Benefit |
|----------|---------|
| **Stability** | Same name → same hash, forever. Adding/removing phrases doesn't affect other IDs. |
| **Compactness** | 16 bytes, implements `Copy`, stack-allocated. |
| **Serializability** | Just a `u128`—works with JSON, bincode, protobuf, etc. |
| **Const construction** | `PhraseId::from_name()` is `const fn`, enabling compile-time constants. |

### API

The primary `PhraseId` API uses a `Locale` reference, which handles language
selection automatically. Prefer these methods for typical usage.

```rust
impl PhraseId {
    /// Create a PhraseId from a phrase name at compile time.
    pub const fn from_name(name: &str) -> Self {
        Self(const_fnv1a_128(name))
    }

    /// Resolve a parameterless phrase to its Phrase value.
    /// Looks up the phrase in the locale's current language and evaluates it.
    pub fn resolve(&self, locale: &Locale) -> Result<Phrase, EvalError>;

    /// Call a phrase with positional arguments.
    /// Looks up the phrase, binds arguments to parameters, and evaluates it.
    pub fn call(&self, locale: &Locale, args: &[Value]) -> Result<Phrase, EvalError>;

    /// Get the phrase name for debugging.
    /// Returns None if the phrase is not registered.
    pub fn name<'a>(&self, locale: &'a Locale) -> Option<&'a str>;

    /// Check if this phrase has parameters.
    /// Returns false if the phrase is not found.
    pub fn has_parameters(&self, locale: &Locale) -> bool;

    /// Get the number of parameters this phrase expects.
    /// Returns 0 if the phrase is not found.
    pub fn parameter_count(&self, locale: &Locale) -> usize;

    /// Get the raw hash value.
    pub fn as_u128(&self) -> u128;
}
```

### Lower-Level API

The `resolve_with_registry` and `call_with_registry` methods operate on
`PhraseRegistry` directly, bypassing `Locale`. Prefer `resolve()` and `call()`
when a `Locale` is available.

```rust
impl PhraseId {
    /// Resolve using a PhraseRegistry directly.
    pub fn resolve_with_registry(
        &self,
        registry: &PhraseRegistry,
        lang: &str,
    ) -> Result<Phrase, EvalError>;

    /// Call using a PhraseRegistry directly.
    pub fn call_with_registry(
        &self,
        registry: &PhraseRegistry,
        lang: &str,
        args: &[Value],
    ) -> Result<Phrase, EvalError>;
}
```

### Global Locale Methods (requires `global-locale` feature)

When the `global-locale` feature is enabled, `PhraseId` gains methods that
operate on the global locale without requiring a `&Locale` parameter:

```rust
impl PhraseId {
    /// Resolve a parameterless phrase using the global locale.
    pub fn resolve_global(&self) -> Result<Phrase, EvalError>;

    /// Call a phrase with positional arguments using the global locale.
    pub fn call_global(&self, args: &[Value]) -> Result<Phrase, EvalError>;

    /// Get the phrase name using the global locale (returns owned String).
    pub fn name_global(&self) -> Option<String>;
}
```

### Hash Function

RLF uses 128-bit FNV-1a for phrase name hashing because it's simple, fast,
implementable as a `const fn`, and provides negligible collision probability
for any realistic number of phrases.

### Collision Detection

Hash collisions are effectively impossible with 128-bit hashes. `PhraseRegistry`
still detects collisions when phrases are inserted as a safety measure.

### Name Registry

`PhraseRegistry` maintains a reverse mapping from hash to name for id-based
lookup:

```rust
struct PhraseRegistry {
    phrases: HashMap<String, PhraseDefinition>,
    id_to_name: HashMap<u128, String>,
}
```

This mapping is populated when phrases are loaded via `load_translations_str`
or `load_translations` on `Locale`.

---

## Code Generation: PhraseId Constants

### Generated Module

For every phrase (with or without parameters), the `rlf!` macro generates a
constant in the `phrase_ids` submodule:

```rust
// strings.rlf.rs
rlf! {
    card = { one: "card", other: "cards" };
    event = :an "event";
    hello = "Hello, world!";
    draw(n) = "Draw {n} {card:n}.";
}
```

```rust
// Generated in strings.rs
pub mod phrase_ids {
    use super::PhraseId;

    /// ID for the "card" phrase.
    pub const CARD: PhraseId = PhraseId::from_name("card");

    /// ID for the "event" phrase.
    pub const EVENT: PhraseId = PhraseId::from_name("event");

    /// ID for the "hello" phrase.
    pub const HELLO: PhraseId = PhraseId::from_name("hello");

    /// ID for the "draw" phrase. Call with 1 argument (n).
    pub const DRAW: PhraseId = PhraseId::from_name("draw");
}
```

### Resolving vs Calling

For most use cases, call phrase functions directly or use `Locale` methods:

```rust
// Via generated functions (preferred for static phrases)
let card = strings::card(&locale);
println!("{}", card);                    // "card"
println!("{}", card.variant("other"));   // "cards"

let text = strings::draw(&locale, 3);
println!("{}", text);  // "Draw 3 cards."

// Via Locale methods (for dynamic lookup by name)
let card = locale.get_phrase("card")?;
let text = locale.call_phrase("draw", &[3.into()])?;
```

`PhraseId` is useful when you need to store phrase references in serializable
data structures. Use `resolve` for parameterless phrases and `call` for phrases
with parameters:

```rust
let card = strings::phrase_ids::CARD.resolve(&locale)?;
let text = strings::phrase_ids::DRAW.call(&locale, &[3.into()])?;
```

---

## PhraseId Usage Patterns

### Storing in Data Structures

```rust
#[derive(Serialize, Deserialize)]
struct CardDefinition {
    name: PhraseId,
    flavor_text: PhraseId,
    cost: u32,
    power: u32,
}

let card = CardDefinition {
    name: strings::phrase_ids::FIRE_ELEMENTAL,
    flavor_text: strings::phrase_ids::FIRE_ELEMENTAL_FLAVOR,
    cost: 5,
    power: 4,
};

// Serialize to JSON, save to file, send over network, etc.
let json = serde_json::to_string(&card)?;

// Later, resolve to localized text
fn render_card(card: &CardDefinition, locale: &Locale) -> String {
    let name = card.name.resolve(locale)
        .map(|p| p.to_string())
        .unwrap_or_else(|_| "[missing]".to_string());
    format!("{} (Cost: {})", name, card.cost)
}
```

### Global Locale Usage (requires `global-locale` feature)

With the `global-locale` feature, `PhraseId` methods can resolve without a
`&Locale` parameter:

```rust
fn render_card(card: &CardDefinition) -> String {
    let name = card.name.resolve_global()
        .map(|p| p.to_string())
        .unwrap_or_else(|_| "[missing]".to_string());
    format!("{} (Cost: {})", name, card.cost)
}
```

### Storing Phrases with Runtime Parameters

A common pattern is storing a `PhraseId` alongside the values needed to resolve
it. This separates "which phrase" from "what values":

```rust
#[derive(Serialize, Deserialize)]
enum PromptLabel {
    /// A simple parameterless phrase
    Simple(PhraseId),
    /// A phrase that needs an energy value
    WithEnergy { phrase: PhraseId, energy: i32 },
    /// A phrase that needs a card reference
    WithCard { phrase: PhraseId, card: PhraseId },
}

impl PromptLabel {
    fn resolve(&self, locale: &Locale) -> Result<String, EvalError> {
        match self {
            PromptLabel::Simple(id) => {
                id.call(locale, &[]).map(|p| p.to_string())
            }
            PromptLabel::WithEnergy { phrase, energy } => {
                phrase.call(locale, &[(*energy).into()])
                    .map(|p| p.to_string())
            }
            PromptLabel::WithCard { phrase, card } => {
                let card_phrase = card.resolve(locale)?;
                phrase.call(locale, &[card_phrase.into()])
                    .map(|p| p.to_string())
            }
        }
    }
}

// Usage
let label = PromptLabel::WithEnergy {
    phrase: strings::phrase_ids::COSTS_ENERGY,  // "Costs {e} energy."
    energy: 3,
};
let text = label.resolve(&locale)?;  // → "Costs 3 energy."
```

### Generic Phrase Calls

For maximum flexibility, store arguments as `Vec<Value>`. Note that `Value` does
not implement `Serialize` or `Deserialize` (because it can contain `Phrase`
values with complex runtime state), so `DynamicPhrase` is not serializable:

```rust
struct DynamicPhrase {
    id: PhraseId,
    args: Vec<Value>,
}

impl DynamicPhrase {
    fn resolve(&self, locale: &Locale) -> Option<String> {
        self.id.call(locale, &self.args)
            .map(|p| p.to_string())
            .ok()
    }
}
```

If you need serialization, store arguments as serializable types and convert
to `Value` at resolve time:

```rust
#[derive(Serialize, Deserialize)]
enum PhraseArg {
    Number(i64),
    Float(f64),
    Text(String),
    Phrase(PhraseId),
}

#[derive(Serialize, Deserialize)]
struct SerializableDynamicPhrase {
    id: PhraseId,
    args: Vec<PhraseArg>,
}

impl SerializableDynamicPhrase {
    fn resolve(&self, locale: &Locale) -> Result<String, EvalError> {
        let args: Vec<Value> = self.args.iter().map(|a| match a {
            PhraseArg::Number(n) => Value::from(*n),
            PhraseArg::Float(f) => Value::from(*f),
            PhraseArg::Text(s) => Value::from(s.as_str()),
            PhraseArg::Phrase(id) => {
                id.resolve(locale)
                    .map(Value::from)
                    .unwrap_or_else(|_| Value::from("[missing]"))
            }
        }).collect();
        self.id.call(locale, &args)
            .map(|p| p.to_string())
    }
}
```

### Comparing Phrases

Because `PhraseId` is `Copy`, `Eq`, and `Hash`, it works efficiently in
collections and comparisons:

```rust
use std::collections::HashSet;

let mut seen: HashSet<PhraseId> = HashSet::new();
seen.insert(strings::phrase_ids::CARD);
seen.insert(strings::phrase_ids::EVENT);

if seen.contains(&strings::phrase_ids::CARD) {
    // ...
}
```

### Debugging

Use `Display` formatting or `as_u128()` for logging:

```rust
fn debug_phrase(id: PhraseId) {
    println!("{id}");  // "PhraseId(0123456789abcdef0123456789abcdef)"
    println!("Hash: {:032x}", id.as_u128());
}
```

### Dynamic PhraseId Creation

While constants are preferred, you can create `PhraseId` dynamically from
strings when needed (e.g., loading from external data):

```rust
// From a string at runtime
let id = PhraseId::from_name("card");
assert_eq!(id, strings::phrase_ids::CARD);

// This works because the hash is deterministic
```

---

## Code Generation: Phrase Functions

### Example

```rust
// strings.rlf.rs
rlf! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}
```

### Generated Code

```rust
// strings.rs (generated)

/// Returns the "card" phrase.
pub fn card(locale: &Locale) -> Phrase {
    locale.get_phrase("card")
        .expect("phrase 'card' should exist")
}

/// Evaluates the "draw" phrase.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase {
    locale.call_phrase("draw", &[n.into()])
        .expect("phrase 'draw' should exist")
}

/// Source language phrases embedded as data.
const SOURCE_PHRASES: &str = r#"
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
"#;

/// Registers source language phrases with the locale. Call once at startup.
pub fn register_source_phrases(locale: &mut Locale) {
    locale.load_translations_str("en", SOURCE_PHRASES)
        .expect("source phrases should parse successfully");
}
```

### Global Locale Variant (requires `global-locale` feature)

When the `global-locale` feature is enabled, the generated code changes:

```rust
// strings.rs (generated with global-locale)

static __RLF_REGISTER: std::sync::Once = std::sync::Once::new();

/// Returns the "card" phrase.
pub fn card() -> Phrase {
    __RLF_REGISTER.call_once(|| { /* load SOURCE_PHRASES */ });
    rlf::with_locale(|locale| {
        locale.get_phrase("card")
            .expect("phrase 'card' should exist")
    })
}

/// Evaluates the "draw" phrase.
pub fn draw(n: impl Into<Value>) -> Phrase {
    __RLF_REGISTER.call_once(|| { /* load SOURCE_PHRASES */ });
    rlf::with_locale(|locale| {
        locale.call_phrase("draw", &[n.into()])
            .expect("phrase 'draw' should exist")
    })
}

/// Registers source language phrases with the global locale.
/// Called automatically on first use of any phrase function.
pub fn register_source_phrases() {
    __RLF_REGISTER.call_once(|| { /* load SOURCE_PHRASES */ });
}
```

Key differences:
- Functions take no `locale` parameter
- A `std::sync::Once` guard auto-registers source phrases on first call
- `register_source_phrases()` takes no arguments

### Key Design Points

1. **Unified evaluation**: All languages use the interpreter, including the source
2. **Embedded source data**: The macro extracts source phrases and embeds them as a string constant
3. **Startup registration**: Source phrases are loaded into the interpreter once at startup
4. **Simple generated code**: Functions are thin wrappers that delegate to the interpreter

---

## Selection Evaluation

Selection is resolved at runtime by the interpreter. Given an RLF definition
like:

```
card = { nom: "card", nom.other: "cards" };
draw(n) = "Draw {n} {card:nom:n}.";
```

**Variant resolution** tries an exact key match first. If not found, it
progressively strips the last `.segment` from the key and retries. For example,
`nom.one` -> try "nom.one" (miss) -> try "nom" (hit) -> "card". If no match is
found after all fallbacks, a `MissingVariant` error is returned listing available
variant keys and similar suggestions.

### Tag-Based Selection

When selecting based on a phrase's tag, the interpreter reads the tag and uses
it as the variant key. For example, given:

```
destroyed = { masc: "destruido", fem: "destruida" };
destroy(target) = "{target} fue {destroyed:target}.";
```

**Tag-based selection** works as follows:

1. Evaluate the `target` parameter, which produces a `Phrase` value
2. Read all metadata tags from the target `Phrase` (e.g., `:fem`)
3. Use those tags as candidate keys for variant lookup in `destroyed`
4. If no matching variant is found, return a `MissingVariant` error

When a `Phrase` has multiple tags (e.g., `:masc :anim` in Russian), all tags are
tried as candidates. This enables multi-dimensional selection for languages where
gender and animacy are independent grammatical properties.

---

## Transform Evaluation

Transforms are evaluated by the interpreter at runtime. Each transform is a
variant of the `TransformKind` enum, dispatched via its `execute()` method. For
example, given:

```
card = :a "card";
draw_one = "Draw {@a card}.";
```

**@a transform** checks the value's tags for `:an` first, then `:a`. If `:an` is
found, prepends "an "; if `:a` is found, prepends "a ". If neither tag exists,
returns a `MissingTag` error. The `@an` alias resolves to the same transform.

### Transform Aliases

**Alias resolution** maps alternative transform names to their canonical
`TransformKind` variant. For example, `@an` maps to the same `TransformKind` as
`@a`. The `TransformRegistry` handles this mapping when looking up a transform by
name and language.

---

## Metadata Inheritance Evaluation

The `:from(param)` modifier enables phrase-returning phrases. For example, given:

```
ancient = :an { one: "Ancient", other: "Ancients" };
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
```

**Metadata inheritance** works as follows when evaluating `subtype(ancient)`:

1. Look up the `:from` parameter (`s`) in the evaluation context and extract its
   `Phrase` value
2. Clone the source phrase's tags (e.g., `:an`) for inheritance
3. Evaluate the template using the source phrase's default text to produce the
   result's default text
4. For each variant in the source phrase (e.g., `one: "Ancient"`,
   `other: "Ancients"`), substitute the variant text for the parameter and
   re-evaluate the template, building the result's variant map
5. Return a new `Phrase` with the computed default text, variant map, and
   inherited tags

This enables composition patterns like `{@a subtype(s)}` where `@a` reads the
inherited tag and `:other` selectors access inherited variants.

---

## Error Handling

RLF reports errors as Rust compile-time errors with precise source locations.

### Compile-Time Error Categories

**Syntax Errors:**

```
error: expected '=' after phrase name
  --> strings.rlf.rs:3:10
   |
3  |     hello "world";
   |          ^ expected '='
```

**Undefined Phrase Reference:**

```
error: unknown phrase 'cards'
  --> strings.rlf.rs:5:28
   |
5  |     draw(n) = "Draw {n} {cards:n}.";
   |                          ^^^^^ not defined
   |
   = help: did you mean 'card'?
```

**Undefined Parameter:**

```
error: unknown parameter 'count'
  --> strings.rlf.rs:2:18
   |
2  |     draw(n) = "Draw {count} cards.";
   |                      ^^^^^ not in parameter list
   |
   = help: declared parameters: n
```

**Invalid Literal Selector:**

```
error: phrase 'card' has no variant 'accusative'
  --> strings.rlf.rs:5:22
   |
5  |     take = "{card:accusative}";
   |                  ^^^^^^^^^^^ variant not defined
   |
   = note: available variants: one, other
```

### Span Preservation

The macro preserves source spans for precise error locations. Each identifier in
the AST is wrapped in a `SpannedIdent` that pairs the name string with a
`proc_macro2::Span`, allowing the macro to point errors directly at the source
token that caused the issue.

### Helpful Suggestions

For typos, the macro suggests similar names using Levenshtein distance.

---

## Translation File Validation

Translation files (`.rlf`) are validated at load time, not compile time.

### Load-Time Checks

When loading a translation file, the interpreter validates and returns `Result`:

1. **Syntax**: Parse errors return `Err(LoadError)` with line/column information
2. **Phrase references**: Unknown phrases produce warnings (not errors)
3. **Parameter counts**: Mismatch with source language produces warnings

### validate_translations()

After loading both source and target language phrases, call
`validate_translations()` to check the target language for potential issues:

```rust
pub fn validate_translations(
    &self,
    source_language: &str,
    target_language: &str,
) -> Vec<LoadWarning>;
```

Both languages must already be loaded via `load_translations` or
`load_translations_str`. Returns an empty vector if no warnings are found or
if either language is not loaded.

```rust
use rlf::{Locale, LoadWarning};

let mut locale = Locale::new();
locale.load_translations_str("en", r#"
    hello = "Hello!";
    greet(name) = "Hello, {name}!";
"#).unwrap();

locale.load_translations_str("ru", r#"
    hello = "Привет!";
    greet(first, last) = "Привет, {first} {last}!";
    extra = "Лишнее";
"#).unwrap();

let warnings = locale.validate_translations("en", "ru");
for w in &warnings {
    eprintln!("{w}");
}
// warning: phrase 'greet' in 'ru' has 2 parameter(s), but source has 1
// warning: translation 'ru' defines unknown phrase 'extra' not found in source
```

### LoadWarning Variants

The `LoadWarning` enum has four variants:

| Variant | Description | Fields |
|---------|-------------|--------|
| `UnknownPhrase` | Target defines a phrase not in the source language | `name`, `language` |
| `ParameterCountMismatch` | Target phrase has different parameter count than source | `name`, `language`, `source_count`, `translation_count` |
| `InvalidTag` | Phrase uses a metadata tag not recognized for the target language | `name`, `language`, `tag`, `valid_tags` |
| `InvalidVariantKey` | Phrase uses a variant key component not recognized for the target language | `name`, `language`, `key`, `valid_keys` |

`InvalidTag` and `InvalidVariantKey` are only checked for languages with known
validation rules (e.g., Russian, Polish). For unrecognized language codes, tag
and variant key validation is skipped.

`LoadWarning` implements `Display`, `Debug`, `Clone`, `PartialEq`, and `Eq`.

---

## Runtime Components

### ICU4X Dependencies

RLF uses ICU4X for Unicode-compliant internationalization:

```toml
[dependencies]
icu_plurals = "2"
icu_locale_core = "2"
```

### CLDR Plural Rules

**Plural category resolution** maps a language code and number to one of the
CLDR plural categories: "zero", "one", "two", "few", "many", or "other".

1. Normalize the language code to a supported ICU4X locale (unsupported codes
   fall back to English)
2. Look up or create a cached `PluralRules` instance for the language (cached
   per thread to avoid repeated construction)
3. Compute the cardinal plural category for the given number
4. Return the category as a string (e.g., "one", "other")

### Universal Transforms

Universal transforms are implemented as `TransformKind` enum variants (`Cap`,
`Upper`, `Lower`), dispatched via the `execute()` method:

- **@cap**: Uppercases the first grapheme cluster of the text, leaving the rest
  unchanged. Uses ICU4X locale-aware case mapping for correct behavior with
  accented characters and non-Latin scripts.
- **@upper**: Converts the entire text to uppercase using ICU4X locale-aware
  case mapping.
- **@lower**: Converts the entire text to lowercase using ICU4X locale-aware
  case mapping.

### Language-Specific Transforms

Each language has its own set of `TransformKind` variants. For example, English
has `EnglishA` (for `@a`/`@an`) and `EnglishThe` (for `@the`). The
`TransformRegistry` maps transform names to the appropriate variant for each
language.

**@a transform (English)** checks the value's tags for `:an` first, then `:a`.
If `:an` is found, prepends "an "; if `:a` is found, prepends "a ". If neither
tag exists, returns a `MissingTag` error. Tags are checked on the `Value`
directly, so transforms work correctly with `Phrase` values that carry metadata.

---

## Runtime Errors

The interpreter returns `Result<_, EvalError>` for all evaluation methods. Common
error conditions:

- **Phrase not found**: Phrase doesn't exist in the current language
- **Missing variant**: Selector key doesn't match any variant in the phrase
- **Missing required tag**: Transform requires a tag the phrase doesn't have
- **Argument count mismatch**: Wrong number of arguments passed to a phrase
- **Cyclic reference**: Phrase references itself directly or indirectly

Generated functions (from the `rlf!` macro) call `.expect()` on results, so these
errors cause panics in application code. This is intentional—these are programming
errors indicating the RLF definition is inconsistent with how it's being used,
and should be caught during development.

---

## Performance Considerations

### Unified Interpreter

All languages (including the source) use the interpreter:

- Interpreter lookup per phrase call
- Runtime variant resolution
- String allocation for results

This is acceptable because:

1. Localized text is typically not performance-critical
2. Hot paths can cache results
3. The interpreter is optimized for common patterns
4. Simplicity outweighs micro-optimization

### Caching Opportunities

- **Parsed ASTs**: Source phrases are parsed once at startup
- **Plural rules**: CLDR rules are cached per language
- **Phrase lookup**: HashMap-based O(1) lookup by name

### Memory

- Phrases with interpolation allocate once for the result
- The `Value` type uses `String` for string values
- Typical translation files use a few hundred KB

---

## IDE Support

Because RLF uses proc-macros, rust-analyzer provides:

- **Autocomplete**: Phrase functions appear immediately
- **Go-to-definition**: Navigate to the macro invocation
- **Error highlighting**: Syntax errors and undefined references

---

## Locale Construction

The `Locale` struct provides several ways to create an instance:

### Locale::new()

Creates a locale with default settings (English, no string context):

```rust
let mut locale = Locale::new();
assert_eq!(locale.language(), "en");
```

### Locale::with_language()

Creates a locale with a specific initial language:

```rust
let mut locale = Locale::with_language("ru");
assert_eq!(locale.language(), "ru");
```

### Locale::builder()

A builder pattern (via the `bon` crate) for full configuration, including the
`string_context` option:

```rust
use rlf::Locale;

let mut locale = Locale::builder()
    .language("ru")
    .string_context("card_text".to_string())
    .build();

assert_eq!(locale.language(), "ru");
assert_eq!(locale.string_context(), Some("card_text"));
```

Builder fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `language` | `impl Into<String>` | `"en"` | Initial language code |
| `string_context` | `Option<String>` | `None` | Format variant selection context |

When `string_context` is set, variant phrases prefer the variant matching this
context as their default text. For example, with
`string_context = "card_text"`, a phrase
`{ interface: "X", card_text: "<b>X</b>" }` produces `"<b>X</b>"` as its
default text. The string context can also be changed after construction via
`locale.set_string_context(Some("card_text"))`.

---

## Global Locale API

The `global-locale` Cargo feature stores the locale in global state, removing
the `locale` parameter from generated phrase functions and enabling automatic
source phrase registration.

### Feature Flag Setup

```toml
# Cargo.toml
[dependencies]
rlf = { version = "0.1", features = ["global-locale"] }
```

### Public Functions

```rust
/// Read access to the global locale.
pub fn with_locale<T>(f: impl FnOnce(&Locale) -> T) -> T;

/// Write access to the global locale.
pub fn with_locale_mut<T>(f: impl FnOnce(&mut Locale) -> T) -> T;

/// Set the current language.
pub fn set_language(language: impl Into<String>);

/// Get the current language (returns owned String).
pub fn language() -> String;
```

### Thread Safety

The global locale uses `LazyLock<RwLock<Locale>>`:
- `with_locale` acquires a read lock (multiple concurrent readers)
- `with_locale_mut` acquires a write lock (exclusive access)
- `set_language` uses write access internally

### Initialization and Auto-Registration

The global locale is initialized with `Locale::new()` (English, empty). Source
phrases are registered automatically on first call to any generated phrase
function via a `std::sync::Once` guard.

### Migration from Explicit Locale

```rust
// Before (explicit locale):
let mut locale = Locale::new();
strings::register_source_phrases(&mut locale);
locale.load_translations_str("ru", RU_PHRASES).unwrap();
locale.set_language("ru");
let text = strings::card(&locale);

// After (global locale):
strings::register_source_phrases(); // optional, happens automatically
rlf::with_locale_mut(|locale| {
    locale.load_translations_str("ru", RU_PHRASES).unwrap();
});
rlf::set_language("ru");
let text = strings::card();
```

---

## Summary

| Component | Behavior |
|-----------|----------|
| Source language | Embedded as data, evaluated via interpreter |
| Translated languages | Loaded from files, evaluated via interpreter |
| Validation | Compile-time for source, load-time for translations |
| IDE support | Full autocomplete via proc-macro |
| PhraseId | Hash-based identifier for serializable phrase references |

### Generated Types

| Type | Purpose | Size |
|------|---------|------|
| `Phrase` | Returned by all phrase functions; carries text, variants, tags | Heap-allocated |
| `Value` | Runtime parameter type; accepts numbers, strings, phrases | Enum (24 bytes typical) |
| `PhraseId` | Serializable reference to any phrase; resolve with `call()` | 16 bytes, `Copy` |

The design prioritizes:

- **Simplicity**: One code path for all languages
- **Immediate feedback**: Add phrase, use immediately with autocomplete
- **Flexible translations**: Load/reload without recompilation
- **Simple API**: Functions take locale, return strings
- **Serializable references**: `PhraseId` enables storing phrase references in data
