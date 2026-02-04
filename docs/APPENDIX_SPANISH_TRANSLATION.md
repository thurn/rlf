# Appendix: Spanish Translation Walkthrough

This appendix provides a comprehensive example of translating `cost_serializer.rs`
to Spanish using RLF. It demonstrates how to extract all localization concerns
into RLF files while keeping the serializer code language-agnostic.

## Overview

The original `cost_serializer.rs` contains ~150 lines of Rust code that produces
English text for card costs. The goal is to:

1. Extract all English text into `strings.rlf.rs` using `rlf!`
2. Create a Spanish translation in `es.rlf` (loaded at runtime)
3. Refactor `cost_serializer.rs` to be language-agnostic, delegating all
   grammatical decisions to RLF

---

## Part 1: Analysis of the Original Code

### Cost Types

The serializer handles these cost types:

```rust
Cost::AbandonCharactersCount { target, count }  // "abandon 3 allies"
Cost::DiscardCards { count, .. }                // "discard 2"
Cost::DiscardHand                               // "discard your hand"
Cost::Energy(energy)                            // "{e}"
Cost::LoseMaximumEnergy(amount)                 // "lose {maximum-energy}"
Cost::BanishCardsFromYourVoid(count)            // "{Banish} 3 from your void"
Cost::BanishCardsFromEnemyVoid(count)           // "{Banish} 3 from the opponent's void"
Cost::BanishAllCardsFromYourVoidWithMinCount(n) // "{Banish} your void with 5 or more cards"
Cost::BanishFromHand(predicate)                 // "{Banish} a card from hand"
Cost::Choice(costs)                             // cost1 or cost2
Cost::ReturnToHand { target, count }            // "return 2 allies to hand"
Cost::SpendOneOrMoreEnergy                      // "pay 1 or more {energy-symbol}"
Cost::BanishAllCardsFromYourVoid                // "{Banish} your void"
Cost::CostList(costs)                           // cost1 and cost2
```

### Problematic Patterns in the Original

The original code mixes text generation with logic:

```rust
// Problem 1: Hardcoded English text
Cost::DiscardHand => "discard your hand".to_string(),

// Problem 2: Pre-rendering predicates as strings
CollectionExpression::AnyNumberOf => {
    format!(
        "abandon any number of {}",
        predicate_serializer::serialize_predicate_plural(target, bindings)
    )
}
```

### Spanish Grammatical Requirements

Spanish requires:
- **Gender agreement**: Articles and adjectives must agree with noun gender
- **Definite articles**: el (masc), la (fem), los (masc pl), las (fem pl)
- **Indefinite articles**: un (masc), una (fem), unos (masc pl), unas (fem pl)
- **Number agreement**: Singular/plural for nouns, verbs, adjectives, articles

For this serializer:
- "carta" (card) is feminine → "una carta", "las cartas"
- "personaje" (character) is masculine → "un personaje", "los personajes"
- "aliado" (ally) is masculine → "un aliado", "los aliados"
- "mano" (hand) is feminine → "tu mano"
- "vacío" (void) is masculine → "tu vacío", "el vacío del oponente"

---

## Part 2: Key Design Principle

### Pass Phrase, Not String

The critical insight: **Rust should pass `Phrase` values to RLF phrases, not
pre-rendered strings.**

**Wrong approach:**
```rust
// Rust pre-renders predicate, losing grammatical information
let target_text = serialize_predicate_plural(target, locale);
// target_text = "allies" (String) — no gender!

format!("abandon any number of {}", target_text)
```

**Correct approach:**
```rust
// Rust passes Phrase with full grammatical information
let target = strings::ally(locale);  // Phrase with :masc tag

strings::abandon_any_number(locale, target)
// Spanish template can use gender tag for agreement
```

### Let RLF Handle All Grammatical Decisions

The serializer identifies *what* to say, not *how* to say it:

| Semantic Intent | Rust Calls | RLF Decides |
|-----------------|------------|-------------|
| "one ally" | `strings::abandon_one(locale, target)` | Article, gender |
| "3 cards" | `strings::abandon_n(locale, 3, target)` | Number agreement |
| "your hand" | `strings::your_hand(locale)` | Possessive form |

---

## Part 3: The English RLF File

```rust
// strings.rlf.rs
rlf! {
    // =========================================================================
    // Basic Types
    // =========================================================================

    card = :a { one: "card", other: "cards" };
    character = :a { one: "character", other: "characters" };
    ally = :an { one: "ally", other: "allies" };

    // =========================================================================
    // Keyword Formatting
    // =========================================================================

    banish = "<k>Banish</k>";
    energy_symbol = "<e>●</e>";

    // =========================================================================
    // Locations
    // =========================================================================

    your_void = "your void";
    opponent_void = "the opponent's void";
    your_hand = "your hand";
    hand = "hand";

    // =========================================================================
    // Abandon Costs
    // =========================================================================

    abandon_any_number(target) = "abandon any number of {target:other}";
    abandon_one(target) = "abandon {@a target}";
    abandon_n(n, target) = "abandon {n} {target:n}";

    // =========================================================================
    // Discard Costs
    // =========================================================================

    discard_n(n) = "discard {n}";
    discard_your_hand = "discard your hand";

    // =========================================================================
    // Energy Costs
    // =========================================================================

    energy_cost(n) = "{n}";
    lose_maximum_energy(n) = "lose {n}";
    pay_one_or_more_energy = "pay 1 or more {energy_symbol}";

    // =========================================================================
    // Banish Costs
    // =========================================================================

    banish_one_from_void = "{banish} another card in your void";
    banish_n_from_your_void(n) = "{banish} {n} from your void";
    banish_n_from_opponent_void(n) = "{banish} {n} from the opponent's void";
    banish_your_void = "{banish} your void";
    banish_void_with_min(n) = "{banish} your void with {n} or more cards";
    banish_from_hand(target) = "{banish} {target} from hand";

    // =========================================================================
    // Return to Hand Costs
    // =========================================================================

    return_one(target) = "return {@a target} to hand";
    return_n(n, target) = "return {n} {target:n} to hand";
    return_all_but_one(target) = "return all but one {target:one} to hand";
    return_all(target) = "return all {target:other} to hand";
    return_any_number(target) = "return any number of {target:other} to hand";
    return_up_to(n, target) = "return up to {n} {target:n} to hand";
    return_each_other(target) = "return each other {target:one} to hand";
    return_n_or_more(n, target) = "return {n} or more {target:other} to hand";

    // =========================================================================
    // Connectors
    // =========================================================================

    cost_or = " or ";
    cost_and = " and ";
}
```

---

## Part 4: The Spanish Translation File

Spanish uses the same phrase names but different templates with gender agreement:

```rust
// es.rlf
// Spanish translation for cost serializer

// =========================================================================
// Basic Types
//
// Spanish nouns have gender. Tags enable article transforms and agreement.
// =========================================================================

card = :fem {
    one: "carta",
    other: "cartas",
};

character = :masc {
    one: "personaje",
    other: "personajes",
};

ally = :masc {
    one: "aliado",
    other: "aliados",
};

// =========================================================================
// Keyword Formatting
// =========================================================================

banish = "<k>Destierra</k>";
energy_symbol = "<e>●</e>";

// =========================================================================
// Locations
//
// "vacío" is masculine, "mano" is feminine
// =========================================================================

your_void = "tu vacío";
opponent_void = "el vacío del oponente";
your_hand = "tu mano";
hand = "mano";

// =========================================================================
// Abandon Costs
//
// Spanish uses gender-agreeing quantifiers
// =========================================================================

abandon_any_number(target) = "abandona cualquier cantidad de {target:other}";
abandon_one(target) = "abandona {@un target}";
abandon_n(n, target) = "abandona {n} {target:n}";

// =========================================================================
// Discard Costs
// =========================================================================

discard_n(n) = "descarta {n}";
discard_your_hand = "descarta tu mano";

// =========================================================================
// Energy Costs
// =========================================================================

energy_cost(n) = "{n}";
lose_maximum_energy(n) = "pierde {n}";
pay_one_or_more_energy = "paga 1 o más {energy_symbol}";

// =========================================================================
// Banish Costs
// =========================================================================

banish_one_from_void = "{banish} otra carta de tu vacío";
banish_n_from_your_void(n) = "{banish} {n} de tu vacío";
banish_n_from_opponent_void(n) = "{banish} {n} del vacío del oponente";
banish_your_void = "{banish} tu vacío";
banish_void_with_min(n) = "{banish} tu vacío con {n} o más cartas";
banish_from_hand(target) = "{banish} {target} de la mano";

// =========================================================================
// Return to Hand Costs
// =========================================================================

return_one(target) = "devuelve {@un target} a la mano";
return_n(n, target) = "devuelve {n} {target:n} a la mano";
return_all_but_one(target) = "devuelve todos menos {@un target} a la mano";
return_all(target) = "devuelve {@el:other target} a la mano";
return_any_number(target) = "devuelve cualquier cantidad de {target:other} a la mano";
return_up_to(n, target) = "devuelve hasta {n} {target:n} a la mano";
return_each_other(target) = "devuelve cada otro {target:one} a la mano";
return_n_or_more(n, target) = "devuelve {n} o más {target:other} a la mano";

// =========================================================================
// Connectors
// =========================================================================

cost_or = " o ";
cost_and = " y ";
```

---

## Part 5: Refactored cost_serializer.rs

The refactored serializer passes `Phrase` values and delegates all grammatical
decisions to RLF:

```rust
// cost_serializer.rs

use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use crate::localization::{strings, Locale, Phrase};

/// Serialize a cost to localized text.
pub fn serialize_cost(locale: &Locale, cost: &Cost) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => {
            serialize_abandon(locale, target, count)
        }

        Cost::DiscardCards { count, .. } => {
            strings::discard_n(locale, *count)
        }

        Cost::DiscardHand => {
            strings::discard_your_hand(locale).to_string()
        }

        Cost::Energy(energy) => {
            strings::energy_cost(locale, energy.0)
        }

        Cost::LoseMaximumEnergy(amount) => {
            strings::lose_maximum_energy(locale, *amount)
        }

        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                strings::banish_one_from_void(locale).to_string()
            } else {
                strings::banish_n_from_your_void(locale, *count)
            }
        }

        Cost::BanishCardsFromEnemyVoid(count) => {
            strings::banish_n_from_opponent_void(locale, *count)
        }

        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            strings::banish_void_with_min(locale, *min_count)
        }

        Cost::BanishFromHand(predicate) => {
            let target = predicate_to_phrase(locale, predicate);
            strings::banish_from_hand(locale, target)
        }

        Cost::Choice(costs) => {
            costs
                .iter()
                .map(|c| serialize_cost(locale, c))
                .collect::<Vec<_>>()
                .join(&strings::cost_or(locale).to_string())
        }

        Cost::ReturnToHand { target, count } => {
            serialize_return_to_hand(locale, target, count)
        }

        Cost::SpendOneOrMoreEnergy => {
            strings::pay_one_or_more_energy(locale).to_string()
        }

        Cost::BanishAllCardsFromYourVoid => {
            strings::banish_your_void(locale).to_string()
        }

        Cost::CostList(costs) => {
            costs
                .iter()
                .map(|c| serialize_cost(locale, c))
                .collect::<Vec<_>>()
                .join(&strings::cost_and(locale).to_string())
        }
    }
}

/// Convert a predicate to a Phrase.
fn predicate_to_phrase(locale: &Locale, predicate: &Predicate) -> Phrase {
    match predicate {
        Predicate::Your(CardPredicate::Character) => strings::ally(locale),
        Predicate::Enemy(CardPredicate::Character) => strings::character(locale),
        Predicate::Any(CardPredicate::Card) => strings::card(locale),
        Predicate::Any(CardPredicate::Character) => strings::character(locale),
        _ => strings::card(locale),
    }
}

/// Serialize an abandon cost.
fn serialize_abandon(
    locale: &Locale,
    target: &Predicate,
    count: &CollectionExpression,
) -> String {
    let target_phrase = predicate_to_phrase(locale, target);

    match count {
        CollectionExpression::AnyNumberOf => {
            strings::abandon_any_number(locale, target_phrase)
        }
        CollectionExpression::Exactly(1) => {
            strings::abandon_one(locale, target_phrase)
        }
        CollectionExpression::Exactly(n) => {
            strings::abandon_n(locale, *n, target_phrase)
        }
        _ => strings::abandon_n(locale, 1, target_phrase),
    }
}

/// Serialize a return-to-hand cost.
fn serialize_return_to_hand(
    locale: &Locale,
    target: &Predicate,
    count: &CollectionExpression,
) -> String {
    let target_phrase = predicate_to_phrase(locale, target);

    match count {
        CollectionExpression::Exactly(1) => {
            strings::return_one(locale, target_phrase)
        }
        CollectionExpression::Exactly(n) => {
            strings::return_n(locale, *n, target_phrase)
        }
        CollectionExpression::AllButOne => {
            strings::return_all_but_one(locale, target_phrase)
        }
        CollectionExpression::All => {
            strings::return_all(locale, target_phrase)
        }
        CollectionExpression::AnyNumberOf => {
            strings::return_any_number(locale, target_phrase)
        }
        CollectionExpression::UpTo(n) => {
            strings::return_up_to(locale, *n, target_phrase)
        }
        CollectionExpression::EachOther => {
            strings::return_each_other(locale, target_phrase)
        }
        CollectionExpression::OrMore(n) => {
            strings::return_n_or_more(locale, *n, target_phrase)
        }
    }
}
```

---

## Part 6: How Spanish Gender Agreement Works

### The @un Transform

Spanish uses the `@un` transform to add indefinite articles with gender agreement:

```
# es.rlf
card = :fem { one: "carta", other: "cartas" };
ally = :masc { one: "aliado", other: "aliados" };

abandon_one(target) = "abandona {@un target}";
```

The `@un` transform reads the `:fem` or `:masc` tag from the target phrase:

| Input | Tag | Transform Result |
|-------|-----|------------------|
| `card` | `:fem` | "una carta" |
| `ally` | `:masc` | "un aliado" |

### The @el Transform with Context

For definite articles with plural agreement:

```
return_all(target) = "devuelve {@el:other target} a la mano";
```

The `@el:other` syntax selects the plural definite article:

| Input | Tag | Transform Result |
|-------|-----|------------------|
| `card` (other) | `:fem` | "las cartas" |
| `ally` (other) | `:masc` | "los aliados" |

### Direct Number Selection

For phrases with explicit counts:

```
abandon_n(n, target) = "abandona {n} {target:n}";
```

| n | target | Result |
|---|--------|--------|
| 1 | ally | "abandona 1 aliado" |
| 3 | ally | "abandona 3 aliados" |
| 1 | card | "abandona 1 carta" |
| 5 | card | "abandona 5 cartas" |

---

## Part 7: Complete Example Traces

### Example 1: "abandon any number of allies"

**Rust code:**
```rust
let target = Predicate::Your(CardPredicate::Character);
let count = CollectionExpression::AnyNumberOf;
serialize_cost(locale, &Cost::AbandonCharactersCount { target, count })
```

**English flow:**
```
predicate_to_phrase(target) → strings::ally(locale)
    → Phrase { text: "ally", variants: [...], tags: ["an"] }

strings::abandon_any_number(locale, target_phrase)
    Template: "abandon any number of {target:other}"
    Selection: target:other → "allies"
    Result: "abandon any number of allies"
```

**Spanish flow:**
```
predicate_to_phrase(target) → strings::ally(locale)
    → Phrase { text: "aliado", variants: [...], tags: ["masc"] }

strings::abandon_any_number(locale, target_phrase)
    Template: "abandona cualquier cantidad de {target:other}"
    Selection: target:other → "aliados"
    Result: "abandona cualquier cantidad de aliados"
```

### Example 2: "return a card to hand"

**English flow:**
```
predicate_to_phrase(target) → strings::card(locale)
    → Phrase { tags: ["a"], ... }

strings::return_one(locale, target_phrase)
    Template: "return {@a target} to hand"
    Transform: @a reads :a tag → "a card"
    Result: "return a card to hand"
```

**Spanish flow:**
```
predicate_to_phrase(target) → strings::card(locale)
    → Phrase { tags: ["fem"], ... }

strings::return_one(locale, target_phrase)
    Template: "devuelve {@un target} a la mano"
    Transform: @un reads :fem tag → "una carta"
    Result: "devuelve una carta a la mano"
```

### Example 3: "discard 3 or banish 2 from your void"

**Rust code:**
```rust
let cost = Cost::Choice(vec![
    Cost::DiscardCards { count: 3, .. },
    Cost::BanishCardsFromYourVoid(2),
]);
serialize_cost(locale, &cost)
```

**English flow:**
```
serialize_cost(DiscardCards) → strings::discard_n(locale, 3) → "discard 3"
serialize_cost(Banish) → strings::banish_n_from_your_void(locale, 2) → "{banish} 2 from your void"
Join with cost_or → " or "
Result: "discard 3 or <k>Banish</k> 2 from your void"
```

**Spanish flow:**
```
serialize_cost(DiscardCards) → strings::discard_n(locale, 3) → "descarta 3"
serialize_cost(Banish) → strings::banish_n_from_your_void(locale, 2) → "{banish} 2 de tu vacío"
Join with cost_or → " o "
Result: "descarta 3 o <k>Destierra</k> 2 de tu vacío"
```

---

## Part 8: Benefits of This Approach

### What the Serializer Does NOT Need to Know

The refactored serializer is completely language-agnostic. It does NOT:

- Know that Spanish has gender
- Know that "carta" is feminine
- Know that "aliado" is masculine
- Know which article form to use
- Know how to pluralize words
- Handle any grammatical agreement

### Comparison: Old vs New

**Old approach (hardcoded English):**
```rust
CollectionExpression::Exactly(1) => {
    format!("abandon {}", serialize_predicate(target, bindings))
}
```

Problems:
- English text hardcoded in Rust
- Predicate pre-rendered as String, losing gender information
- Would need different code paths for Spanish

**New approach (RLF):**
```rust
CollectionExpression::Exactly(1) => {
    strings::abandon_one(locale, predicate_to_phrase(locale, target))
}
```

Benefits:
- No English text in Rust
- Phrase preserves gender tag
- Same Rust code works for all languages

### For Translators

1. **Full control over articles**: Use `@un`/`@el` transforms
2. **Full control over gender**: Tags enable automatic agreement
3. **Full control over word order**: Rearrange templates freely
4. **No Rust knowledge required**: All work in `.rlf` files

### For Developers

1. **Simpler serializer**: No linguistic logic
2. **Single code path**: Same Rust for all languages
3. **Type-safe**: Phrase carries metadata
4. **Testable**: Test output for any language

---

## Summary

The key insight: **keep grammatical decisions in RLF, semantic decisions in Rust.**

| Responsibility | Where |
|----------------|-------|
| "What cost type is this?" | Rust |
| "What predicate should I reference?" | Rust |
| "What article does this noun need?" | RLF |
| "What is the plural form?" | RLF |
| "How does gender agreement work?" | RLF |
| "What word order sounds natural?" | RLF |

The cost serializer becomes a simple mapping from cost types to RLF phrase calls.
All linguistic complexity—gender, articles, pluralization, word order—lives in
the RLF files where translators can control it directly.

This approach scales to any language: add a new `.rlf` file with appropriate
tags and phrase templates, and the same Rust serializer produces grammatically
correct output automatically.

---

## Appendix: Phrase Transformation with `:from`

For patterns like character subtypes where formatting needs to wrap phrases
while preserving gender, use `:from` metadata inheritance:

```rust
// es.rlf
ancient = :masc { one: "Ancestral", other: "Ancestrales" };
child = :masc { one: "Niño", other: "Niños" };
mage = :masc { one: "Mago", other: "Magos" };

// :from(s) inherits :masc/:fem tag and one/other variants
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";

// Now @un can read the inherited gender tag
dissolve_subtype(s) = "Disuelve {@un subtype(s)}.";
// ancient → "Disuelve un <b>Ancestral</b>."
// child → "Disuelve un <b>Niño</b>."
```

This enables the same "define once, use everywhere" pattern for subtypes that
would otherwise require separate definitions for each article/gender/number
combination.
