# Appendix: Dreamtides Adoption Case Study

This appendix documents the real-world conversion of Dreamtides' existing Fluent-based
localization system to RLF, identifying practical challenges and design gaps discovered
during the process.

## Current Architecture Overview

Dreamtides uses localization in two distinct patterns:

### Pattern 1: Static UI Strings

UI elements (buttons, prompts, labels) are accessed via a generated `StringId` enum:

```rust
// Current usage in interface_rendering.rs
let label = builder.string(StringId::PrimaryButtonEndTurn);
let label = builder.string_with_args(StringId::PayEnergyPromptButton, fluent_args!("e" => 3));
```

The strings are defined in `strings.ftl`:

```ftl
primary-button-end-turn = End Turn
pay-energy-prompt-button = Spend {e}
```

### Pattern 2: Data-Driven Card Text

Card rules text is **dynamically generated** from ability data structures. Serializers
produce strings containing Fluent placeholders, which are resolved at display time:

```rust
// In effect_serializer.rs
StandardEffect::DrawCards { count } => {
    bindings.insert("cards".to_string(), VariableValue::Integer(*count));
    "draw {cards}.".to_string()
}

// Later, at display time (card_rendering.rs)
let serialized = ability_serializer::serialize_ability(ability);
let args = to_fluent_args(&serialized.variables);
tabula.strings.format_display_string(&serialized.text, StringContext::CardText, args)
```

This pattern enables:
- Card text stored in TOML files with placeholders: `"Draw {cards}. Discard {discards}."`
- Programmatic text assembly from ability structures
- Runtime template interpretation

---

## Converting Static UI Strings

### Direct Mapping

Most static strings map directly to RLF phrases:

**Current Fluent:**
```ftl
# Core symbols
energy-symbol = <color=#00838F>●</color>
points-symbol = <color=#F57F17>⍟</color>

# Simple labels
primary-button-end-turn = End Turn
prompt-choose-character = Choose a character
```

**RLF equivalent:**
```rust
rlf! {
    energy_symbol = "<color=#00838F>●</color>";
    points_symbol = "<color=#F57F17>⍟</color>";

    primary_button_end_turn = "End Turn";
    prompt_choose_character = "Choose a character";
}
```

### Parameterized Strings

Strings with parameters map to RLF phrase functions:

**Current Fluent:**
```ftl
e = <color=#00838F>{$e}●</color>
pay-energy-prompt-button = Spend {e}
maximum-energy = {$max} maximum {energy-symbol}
```

**RLF equivalent:**
```rust
rlf! {
    energy_symbol = "<color=#00838F>●</color>";
    e(e) = "<color=#00838F>{e}●</color>";
    pay_energy_prompt_button(e) = "Spend {e(e)}";
    maximum_energy(max) = "{max} maximum {energy_symbol}";
}
```

### Plural Handling

Fluent select expressions map to RLF variants:

**Current Fluent:**
```ftl
cards =
  {
    $cards ->
      [one] a card
      *[other] { $cards } cards
  }
```

**RLF equivalent:**
```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    cards(n) = "{@a card:n}";  // For "a card" / "2 cards"
    cards_numeral(n) = "{n} {card:n}";  // For "1 card" / "2 cards"
}
```

---

## Converting Data-Driven Card Text

### Serializer Code

The current system generates template strings programmatically:

```rust
// Current: effect_serializer.rs
StandardEffect::DrawCardsForEach { count, for_each } => {
    bindings.insert("cards", count);
    format!(
        "draw {{cards}} for each {}.",
        serialize_for_count_expression(for_each, bindings)
    )
}
```

**RLF approach:** Since serializers are Rust code, use generated phrase functions directly:

```rust
rlf! {
    draw_cards_for_each(n, target) = "draw {cards(n)} for each {target}.";
}

// In serializer
StandardEffect::DrawCardsForEach { count, for_each } => {
    strings::draw_cards_for_each(&locale, *count, serialize_target(for_each, &locale))
}
```

This gives compile-time checking and IDE support. Reserve `eval_str` for true data-driven
content (templates stored in TOML/JSON files).

---

### Multiple Instances of the Same Phrase

Current Fluent system uses numbered message definitions:

```ftl
cards1 = { $cards1 -> [one] a card *[other] { $cards1 } cards }
cards2 = { $cards2 -> [one] a card *[other] { $cards2 } cards }
```

RLF is cleaner—define the phrase once, call it with different parameters:

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    cards(n) = "{@a card:n}";
}
```

For runtime templates, `eval_str` params work like phrase parameters:

```rust
interpreter.eval_str(
    "Draw {cards(draw_count)}. Discard {cards(discard_count)}.",
    "en",
    params!{ "draw_count" => 2, "discard_count" => 1 }
)?
// → "Draw 2 cards. Discard a card."
```

This is equivalent to defining a phrase with those parameters—no special handling needed.

---

### Helper Phrases

Fluent uses `-` prefix for private messages. In RLF, all phrases are public:

```rust
rlf! {
    keyword(k) = "<color=#AA00FF>{k}</color>";
    dissolve = "{keyword(\"dissolve\")}";
    banish = "{keyword(\"banish\")}";
}
```

---

### Subtypes with Phrase-Returning Phrases

The subtype mappings demonstrate RLF's `:from` metadata inheritance feature.

**Current Fluent (verbose, duplicated):**

```ftl
# 4 separate select expressions, each with ~20 variants = 80+ definitions
a-subtype =
  { $subtype ->
      [ancient] an {-type(value: "Ancient")}
      [child] a {-type(value: "Child")}
      # ... 17 more
  }

ASubtype = { ... }      # duplicated with capitalized articles
subtype = { ... }       # duplicated without articles
plural-subtype = { ... } # duplicated with plurals
```

**RLF with `:from` (concise, single source of truth):**

```rust
rlf! {
    // Each subtype defined once with article tag and plural variant
    ancient = :an { one: "Ancient", other: "Ancients" };
    child = :a { one: "Child", other: "Children" };
    detective = :a { one: "Detective", other: "Detectives" };
    enigma = :an { one: "Enigma", other: "Enigmas" };
    explorer = :an { one: "Explorer", other: "Explorers" };
    hacker = :a { one: "Hacker", other: "Hackers" };
    mage = :a { one: "Mage", other: "Mages" };
    monster = :a { one: "Monster", other: "Monsters" };
    musician = :a { one: "Musician", other: "Musicians" };
    outsider = :an { one: "Outsider", other: "Outsiders" };
    renegade = :a { one: "Renegade", other: "Renegades" };
    spirit_animal = :a { one: "Spirit Animal", other: "Spirit Animals" };
    super_ = :a { one: "Super", other: "Supers" };
    survivor = :a { one: "Survivor", other: "Survivors" };
    synth = :a { one: "Synth", other: "Synths" };
    tinkerer = :a { one: "Tinkerer", other: "Tinkerers" };
    trooper = :a { one: "Trooper", other: "Troopers" };
    visionary = :a { one: "Visionary", other: "Visionaries" };
    visitor = :a { one: "Visitor", other: "Visitors" };
    warrior = :a { one: "Warrior", other: "Warriors" };

    // Single function handles formatting + metadata inheritance
    subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
}
```

**How `:from(s)` works:**

The `:from(param)` modifier causes the phrase to:
1. Inherit tags from the parameter (`:an` from `ancient`)
2. Evaluate the template for each variant of the parameter
3. Return a `Phrase` with inherited metadata

This enables natural composition in templates:

```rust
// In card templates
"Dissolve {@a subtype(s)}."        // → "Dissolve an <b>Ancient</b>."
"Dissolve all {subtype(s):other}." // → "Dissolve all <b>Ancients</b>."
"{@cap @a subtype(s)}"             // → "An <b>Ancient</b>"
```

**Caller responsibility:**

Per the "Pass Phrase, not String" principle, callers resolve subtype keys to
Phrases before passing:

```rust
// In serializer
let s = strings::ancient(locale);  // Phrase with :an tag, one/other variants
let formatted = strings::subtype(locale, s);  // Phrase with formatting + inherited metadata
strings::dissolve_target(locale, formatted)
```

For data-driven templates (card TOML), the serializer resolves names before
calling the interpreter:

```rust
// Card data: variables = "s: ancient"
let s_key = card.variables.get("s")?;  // "ancient" (String)
let s_phrase = locale.interpreter().get_phrase(locale.language(), s_key)?;  // Phrase
let params = params!{ "s" => s_phrase };
locale.interpreter().eval_str(&card.rules_text, locale.language(), params)?
```

**Reduction:** ~80 Fluent definitions → ~21 RLF definitions (20 subtypes + 1 function)

---

## Data File Format Conversion

### TOML Card Definitions

TOML files stay the same:

```toml
[[dreamwell]]
name = "Skypath"
energy-produced = 1
rules-text = "{Foresee}."
variables = "foresee: 1"
```

RLF phrases:

```rust
rlf! {
    keyword(k) = "<color=#AA00FF>{k}</color>";
    foresee(n) = "{keyword(\"foresee\")} {n}";
}
```

RLF's **automatic capitalization** means `{Foresee(1)}` produces "Foresee 1" while
`{foresee(1)}` produces "foresee 1". Define once, get both cases.

---

## Serializer Migration

### Current Pattern

```rust
// effect_serializer.rs
pub fn serialize_standard_effect(effect: &StandardEffect, bindings: &mut VariableBindings) -> String {
    match effect {
        StandardEffect::DrawCards { count } => {
            bindings.insert("cards".to_string(), VariableValue::Integer(*count));
            "draw {cards}.".to_string()
        }
        StandardEffect::GainEnergy { gains } => {
            bindings.insert("e".to_string(), VariableValue::Integer(gains.0));
            "gain {e}.".to_string()
        }
        // ...
    }
}
```

### RLF Migration Options

**Option A: Keep templates, use interpreter**

Minimal change—serializers still produce template strings, but use RLF syntax:

```rust
pub fn serialize_standard_effect(effect: &StandardEffect, bindings: &mut VariableBindings) -> String {
    match effect {
        StandardEffect::DrawCards { count } => {
            bindings.insert("n".to_string(), Value::Number(*count));
            "draw {cards(n)}.".to_string()  // RLF template syntax
        }
        // ...
    }
}

// At display time
let result = locale.interpreter().eval_str(&template, locale.language(), bindings)?;
```

**Option B: Return phrase calls**

Serializers return structured data, not strings:

```rust
pub enum SerializedEffect {
    DrawCards { count: i64 },
    GainEnergy { amount: i64 },
    Compound(Vec<SerializedEffect>),
    // ...
}

// At display time
fn render_effect(effect: &SerializedEffect, locale: &Locale) -> String {
    match effect {
        SerializedEffect::DrawCards { count } => strings::draw_cards(locale, *count),
        SerializedEffect::GainEnergy { amount } => strings::gain_energy(locale, *amount),
        SerializedEffect::Compound(effects) => effects
            .iter()
            .map(|e| render_effect(e, locale))
            .collect::<Vec<_>>()
            .join(". "),
    }
}
```

This is cleaner but requires defining phrases for every effect pattern.

**Recommendation:** Option A for initial migration (less invasive), with Option B as
a future goal for better type safety and translator flexibility.

---

## Integration Points

### ResponseBuilder

Current code:
```rust
impl ResponseBuilder {
    pub fn string(&self, id: StringId) -> String {
        self.tabula().strings.format_pattern(id, StringContext::Interface, FluentArgs::new())
    }

    pub fn string_with_args(&self, id: StringId, args: FluentArgs) -> String {
        self.tabula().strings.format_pattern(id, StringContext::Interface, args)
    }
}
```

**RLF migration:**
```rust
impl ResponseBuilder {
    pub fn locale(&self) -> &Locale {
        &self.tabula().locale
    }
}

// Usage becomes:
let label = strings::primary_button_end_turn(builder.locale());
let label = strings::pay_energy_prompt_button(builder.locale(), 3);
```

The explicit function calls provide better IDE support and compile-time checking.

### Card Text Rendering

Current code:
```rust
let serialized = ability_serializer::serialize_ability(ability);
let args = to_fluent_args(&serialized.variables);
tabula.strings.format_display_string(&serialized.text, StringContext::CardText, args)
```

**RLF migration:**
```rust
let serialized = ability_serializer::serialize_ability(ability);
let params = to_rlf_params(&serialized.variables);
locale.interpreter().eval_str(&serialized.text, locale.language(), params)?
```

---

## Summary

All Dreamtides patterns are supported:

- **Static UI strings** → `rlf!` macro with typed functions
- **Runtime templates** → `interpreter.eval_str()` (params work like phrase parameters)
- **Dynamic phrase lookup** → `interpreter.get_phrase(lang, name)`
- **Auto-capitalization** → uppercase phrase reference (e.g., `{Card}` → `{@cap card}`)
- **Multiple phrase instances** → `{cards(n1)}... {cards(n2)}` with different param names
- **Phrase transformation** → `:from(param)` for metadata inheritance (subtypes, figments)

## Additional Observations

### Markup Handling

Both systems embed HTML-like markup in strings:
```
<color=#00838F>●</color>
<b>Materialized:</b>
```

RLF doesn't specifically address markup. This is fine—markup is just text content.
However, translators need to preserve markup structure. Consider:
- Validation tooling to check markup is preserved in translations
- Documentation for translators about markup conventions

### StringContext

The current system has `StringContext::Interface` vs `StringContext::CardText` that
affects formatting. RLF would need a similar mechanism:

```rust
rlf! {
    // Context-aware formatting
    energy_symbol = {
        interface: "●",
        card_text: "<color=#00838F>●</color>",
    };
}
```

Or pass context as a parameter to relevant phrases.

### Error Handling

Current `format_display_string` returns `Result`, and errors are often `.unwrap_or_default()`:
```rust
tabula.strings.format_display_string(&text, ...).unwrap_or_default()
```

RLF's generated functions panic on error. For data-driven content, the interpreter
returns `Result`. Migration should preserve graceful degradation for user-generated
or data-file content.

---

## Migration Strategy

### Phase 1: Parallel Systems

1. Add RLF dependency alongside Fluent
2. Define static UI strings in `strings.rlf.rs`
3. Migrate `ResponseBuilder` to use RLF for static strings
4. Keep `format_display_string` for dynamic card text (using Fluent)

### Phase 2: Interpreter Migration

1. Implement/extend RLF interpreter for runtime template evaluation
2. Convert serializer output from Fluent syntax to RLF syntax
3. Replace `format_display_string` calls with interpreter calls
4. Validate card text rendering matches previous output

### Phase 3: Full Migration

1. Convert all `strings.ftl` content to `strings.rlf.rs`
2. Remove Fluent dependency
3. Add translation files (`.rlf`) for supported languages
4. Update tooling (TV, CLI) to use RLF

### Phase 4: Optimization

1. Consider Option B (structured effect serialization) for type safety
2. Add compile-time validation for data file templates
3. Add translation coverage tooling
