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

```rust
// Internal AST (conceptual)
struct PhraseDefinition {
    name: Identifier,
    parameters: Vec<Identifier>,
    body: PhraseBody,
    tags: Vec<MetadataTag>,
}

enum PhraseBody {
    Simple(TemplateString),
    Variants(Vec<(VariantKey, TemplateString)>),
}

struct TemplateString {
    segments: Vec<Segment>,
}

enum Segment {
    Literal(String),
    Interpolation {
        transforms: Vec<Transform>,
        reference: Reference,
        selectors: Vec<Selector>,
    },
}

enum Reference {
    Parameter(Identifier),
    Phrase(Identifier),
    PhraseCall { name: Identifier, args: Vec<Reference> },
}
```

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
    pub variants: HashMap<String, String>,
    /// Metadata tags.
    pub tags: Vec<String>,
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution. Panics if not found.
    pub fn variant(&self, key: &str) -> &str {
        resolve_variant(&self.variants, key)
    }
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

### Into<Value> Implementations

Common types implement `Into<Value>`: integers become `Value::Number`, strings
become `Value::String`, and `Phrase` becomes `Value::Phrase`.

---

## The PhraseId Type

`PhraseId` provides a compact, serializable reference to any phrase that can be
resolved at runtime. This is useful for storing phrase references in data
structures like game state, card definitions, or network messages—then supplying
parameters when the text is actually needed.

### Design

```rust
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhraseId(u64);
```

`PhraseId` wraps a 64-bit FNV-1a hash of the phrase name. This design provides:

| Property | Benefit |
|----------|---------|
| **Stability** | Same name → same hash, forever. Adding/removing phrases doesn't affect other IDs. |
| **Compactness** | 8 bytes, implements `Copy`, stack-allocated. |
| **Serializability** | Just a `u64`—works with JSON, bincode, protobuf, etc. |
| **Const construction** | `PhraseId::from_name()` is `const fn`, enabling compile-time constants. |

### API

```rust
impl PhraseId {
    /// Create a PhraseId from a phrase name at compile time.
    pub const fn from_name(name: &str) -> Self {
        Self(const_fnv1a_64(name))
    }

    /// Resolve a parameterless phrase to its Phrase value.
    pub fn resolve(&self, locale: &Locale) -> Result<Phrase, EvalError> {
        locale.interpreter().get_phrase_by_id(self.0, locale.language())
    }

    /// Call a phrase with positional arguments.
    pub fn call(&self, locale: &Locale, args: &[Value]) -> Result<String, EvalError> {
        locale.interpreter()
            .call_phrase_by_id(self.0, locale.language(), args)
            .map(|p| p.to_string())
    }

    /// Get the phrase name for debugging.
    /// Returns None if the phrase isn't registered.
    pub fn name(&self) -> Option<&'static str> {
        PHRASE_ID_REGISTRY.get(&self.0).copied()
    }

    /// Get the raw hash value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl PhraseId {
    /// Check if this phrase has parameters.
    pub fn has_parameters(&self, locale: &Locale) -> bool {
        locale.interpreter().phrase_parameter_count(self.0) > 0
    }

    /// Get the number of parameters this phrase expects.
    pub fn parameter_count(&self, locale: &Locale) -> usize {
        locale.interpreter().phrase_parameter_count(self.0)
    }
}
```

### Hash Function

RLF uses FNV-1a for phrase name hashing because it's simple, fast, and
implementable as a `const fn`:

```rust
const fn const_fnv1a_64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let bytes = s.as_bytes();
    let mut hash = FNV_OFFSET;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}
```

### Collision Detection

Hash collisions are theoretically possible but practically negligible with
64-bit hashes. The interpreter detects collisions when phrases are registered:

```rust
impl RlfInterpreter {
    pub fn load_phrases(&mut self, language: &str, content: &str) -> Result<usize, LoadError> {
        for phrase in parse_phrases(content)? {
            let id = PhraseId::from_name(&phrase.name);
            if let Some(existing) = self.phrase_ids.get(&id.0) {
                if existing != &phrase.name {
                    panic!(
                        "PhraseId collision: '{}' and '{}' have the same hash",
                        existing, phrase.name
                    );
                }
            }
            self.phrase_ids.insert(id.0, phrase.name.clone());
            // ... register phrase
        }
        Ok(count)
    }
}
```

### Name Registry

The interpreter maintains a reverse mapping from hash to name for debugging:

```rust
struct RlfInterpreter {
    // ... other fields
    phrase_id_names: HashMap<u64, &'static str>,
}
```

This registry is populated when source phrases are registered at startup. The
`PhraseId::name()` method queries this registry.

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

Use `resolve()` for parameterless phrases (returns a `Phrase` with variants and
tags). Use `call()` for phrases with parameters (returns the evaluated `String`):

```rust
// Parameterless: use resolve()
let card = strings::phrase_ids::CARD.resolve(&locale)?;
println!("{}", card);                    // "card"
println!("{}", card.variant("other"));   // "cards"

// With parameters: use call()
let text = strings::phrase_ids::DRAW.call(&locale, &[3.into()])?;
println!("{}", text);  // "Draw 3 cards."
```

You can also use `call()` on parameterless phrases—it just takes an empty slice:

```rust
let text = strings::phrase_ids::HELLO.call(&locale, &[])?;  // "Hello, world!"
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
                id.call(locale, &[])
            }
            PromptLabel::WithEnergy { phrase, energy } => {
                phrase.call(locale, &[(*energy).into()])
            }
            PromptLabel::WithCard { phrase, card } => {
                let card_phrase = card.resolve(locale)?;
                phrase.call(locale, &[card_phrase.into()])
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

For maximum flexibility, store arguments as `Vec<Value>`:

```rust
#[derive(Serialize, Deserialize)]
struct DynamicPhrase {
    id: PhraseId,
    args: Vec<Value>,
}

impl DynamicPhrase {
    fn resolve(&self, locale: &Locale) -> Option<String> {
        self.id.call(locale, &self.args)
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

Use `name()` to get the phrase name for logging:

```rust
fn debug_phrase(id: PhraseId) {
    if let Some(name) = id.name() {
        println!("Phrase: {} (hash: {:016x})", name, id.as_u64());
    } else {
        println!("Unknown phrase (hash: {:016x})", id.as_u64());
    }
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
    locale.interpreter()
        .get_phrase(locale.language(), "card")
        .expect("phrase 'card' should exist")
}

/// Evaluates the "draw" phrase.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase {
    locale.interpreter()
        .call_phrase(locale.language(), "draw", &[n.into()])
        .expect("phrase 'draw' should exist")
}

/// Source language phrases embedded as data.
const SOURCE_PHRASES: &str = r#"
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
"#;

/// Registers source language phrases with the interpreter. Call once at startup.
pub fn register_source_phrases(interpreter: &mut RlfInterpreter) {
    interpreter.load_phrases("en", SOURCE_PHRASES)
        .expect("source phrases should parse successfully");
}
```

### Key Design Points

1. **Unified evaluation**: All languages use the interpreter, including the source
2. **Embedded source data**: The macro extracts source phrases and embeds them as a string constant
3. **Startup registration**: Source phrases are loaded into the interpreter once at startup
4. **Simple generated code**: Functions are thin wrappers that delegate to the interpreter

---

## Selection Evaluation

Selection is resolved at runtime by the interpreter. The interpreter tries exact
matches first, then progressively shorter fallback keys:

```rust
// RLF:
card = { nom: "card", nom.other: "cards" };
draw(n) = "Draw {n} {card:nom:n}.";

// Interpreter logic:
fn resolve_variant(
    variants: &HashMap<String, String>,
    key: &str,
    phrase_name: &str,
) -> Result<&str, EvalError> {
    // Try exact match: "nom.one"
    if let Some(v) = variants.get(key) {
        return Ok(v);
    }
    // Try fallback: "nom"
    if let Some(dot) = key.rfind('.') {
        let fallback = &key[..dot];
        if let Some(v) = variants.get(fallback) {
            return Ok(v);
        }
    }
    Err(EvalError::MissingVariant {
        phrase: phrase_name.to_string(),
        key: key.to_string(),
        available: variants.keys().cloned().collect(),
    })
}
```

### Tag-Based Selection

When selecting based on a phrase's tag, the interpreter reads the tag and uses
it as the variant key:

```rust
// RLF:
destroyed = { masc: "destruido", fem: "destruida" };
destroy(target) = "{target} fue {destroyed:target}.";

// Interpreter logic:
// 1. Evaluate 'target' parameter → gets a Phrase value
// 2. Read first tag from target phrase (e.g., "fem")
// 3. Select 'destroyed' variant using that tag
// 4. If no matching variant, panic with descriptive error
```

---

## Transform Evaluation

Transforms are evaluated by the interpreter at runtime:

```rust
// RLF:
card = :a "card";
draw_one = "Draw {@a card}.";

// Interpreter has built-in transform functions:
fn transform_a(value: Value) -> String {
    let text = value.to_string();

    if value.has_tag("a") {
        return format!("a {}", text);
    }
    if value.has_tag("an") {
        return format!("an {}", text);
    }

    panic!("@a transform requires tag ':a' or ':an' on '{}'", text)
}
```

### Transform Aliases

Aliases are resolved by the interpreter:

```rust
// @an → @a
play_event = "Play {@an event}.";

// Interpreter maps @an to the same transform function as @a
```

---

## Metadata Inheritance Evaluation

The `:from(param)` modifier enables phrase-returning phrases. The interpreter
handles this by evaluating the template multiple times:

```rust
// RLF:
ancient = :an { one: "Ancient", other: "Ancients" };
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";

// Interpreter logic for subtype(ancient):
fn eval_phrase_with_from(
    template: &Template,
    from_param: &str,
    params: &HashMap<String, Value>,
) -> Phrase {
    let source = params.get(from_param).expect("param exists").as_phrase();

    // Inherit tags
    let tags = source.tags.clone();

    // Evaluate template for each variant
    let mut variants = HashMap::new();
    for (key, variant_text) in &source.variants {
        let mut variant_params = params.clone();
        variant_params.insert(from_param.to_string(), Value::String(variant_text.clone()));
        let result = eval_template(template, &variant_params);
        variants.insert(key.clone(), result);
    }

    // Default text uses the source's default text
    let text = {
        let mut default_params = params.clone();
        default_params.insert(from_param.to_string(), Value::String(source.text.clone()));
        eval_template(template, &default_params)
    };

    Phrase { text, variants, tags }
}
```

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

The macro preserves source spans for precise error locations:

```rust
struct Identifier {
    name: String,
    span: Span,
}
```

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

```rust
use icu_plurals::{PluralCategory, PluralRuleType, PluralRules};
use icu_locale_core::locale;

fn plural_category(lang: &str, n: i64) -> &'static str {
    let locale = match lang {
        "en" => locale!("en"),
        "ru" => locale!("ru"),
        "ar" => locale!("ar"),
        _ => locale!("en"),
    };

    let rules = PluralRules::try_new(locale.into(), PluralRuleType::Cardinal)
        .expect("locale should be supported");

    match rules.category_for(n) {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}
```

### Universal Transforms

```rust
pub fn transform_cap(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn transform_upper(s: &str) -> String {
    s.to_uppercase()
}

pub fn transform_lower(s: &str) -> String {
    s.to_lowercase()
}
```

### Language-Specific Transforms

Each language has transform implementations:

```rust
// English @a transform
fn transform_a_en(value: Value) -> String {
    let text = value.to_string();

    if value.has_tag("an") {
        format!("an {}", text)
    } else if value.has_tag("a") {
        format!("a {}", text)
    } else {
        panic!("@a transform requires tag ':a' or ':an' on '{}'", text)
    }
}
```

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
| `PhraseId` | Serializable reference to any phrase; resolve with `call()` | 8 bytes, `Copy` |

The design prioritizes:

- **Simplicity**: One code path for all languages
- **Immediate feedback**: Add phrase, use immediately with autocomplete
- **Flexible translations**: Load/reload without recompilation
- **Simple API**: Functions take locale, return strings
- **Serializable references**: `PhraseId` enables storing phrase references in data
