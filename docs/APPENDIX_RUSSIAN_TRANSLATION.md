# Appendix: Russian Translation Walkthrough

This appendix provides a comprehensive example of translating `predicate_serializer.rs`
to Russian using RLF. It demonstrates how to keep language-specific text in RLF files
while giving translators control over grammatical forms and word order.

## Overview

The original `predicate_serializer.rs` contains ~800 lines of Rust code that produces
English text for card predicates. The goal is to:

1. Extract all English text into `strings.rlf.rs` using `rlf!`
2. Create a Russian translation in `ru.rlf` (loaded at runtime)
3. Refactor `predicate_serializer.rs` to pass `Phrase` values instead of
   pre-rendered strings, allowing RLF to control grammatical selection

---

## Part 1: Analysis of the Original Code

### Text Categories

The serializer produces several categories of text:

**1. Pronouns/References:**
```rust
"this character"    // Predicate::This
"that character"    // Predicate::That
"them"              // Predicate::Them
"it"                // Predicate::It
```

**2. Ownership Qualifiers:**
```rust
"ally"              // Your character
"your card"         // Your card
"your event"        // Your event
"enemy"             // Enemy character
"allied {subtype}"  // Your character of type
"enemy {subtype}"   // Enemy character of type
```

**3. Card Type Bases:**
```rust
"card", "cards"
"character", "characters"
"event", "events"
```

**4. Complex Predicates:**
```rust
"a character with spark {s} or less"
"with cost less than the number of allied {counting}"
"event which could {dissolve} {target}"
```

**5. Location Qualifiers:**
```rust
"in your void"
"in the opponent's void"
```

### Russian Grammatical Requirements

Russian requires:
- **Six grammatical cases**: nominative, accusative, genitive, dative, instrumental,
  prepositional
- **Three genders**: masculine, feminine, neuter
- **Complex plural forms**: one (1, 21, 31...), few (2-4, 22-24...), many (5-20, 25-30...)
- **Animacy distinction**: affects accusative case for masculine nouns

For this serializer, we primarily need:
- Nominative (subject): "карта есть" (the card is)
- Accusative (direct object): "возьмите карту" (take the card)
- Genitive (possession/counting): "нет карт" (no cards), "5 карт" (5 cards)

---

## Part 2: Key Design Principle

### Pass Phrase, Not String

The critical insight: **Rust should pass `Phrase` values to RLF phrases, not
pre-rendered strings.** This allows RLF to select the appropriate grammatical form.

**Wrong approach** (strips variant information):
```rust
// Rust pre-renders to String, losing all variants
let counting = serialize_card_predicate_plural(count_matching, locale);
// counting = "characters" (String) — no variant table!

strings::with_cost_less_than_allied(locale, base, counting)
// RLF receives a String, cannot select {counting:gen.many}
```

**Correct approach** (preserves variants):
```rust
// Rust passes the Phrase with all variants intact
let counting = strings::character(locale);
// counting = Phrase { variants: [...], ... }

strings::with_cost_less_than_allied(locale, base, counting)
// RLF can now select {counting:gen.many} → "персонажей"
```

### Selection Handles Everything

RLF's selection mechanism (`:`) handles case and number selection on phrase
parameters. No new transforms are needed:

```
// ru.rlf
with_cost_less_than_allied(base, counting) =
    "{base:nom.one} со стоимостью меньше количества союзных {counting:gen.many}";
```

The `:gen.many` selector extracts that variant from the `counting` Phrase.

---

## Part 3: The English RLF File

English is simpler—no case, simple plurals. Each language's templates use
selectors appropriate for that language:

```rust
// strings.rlf.rs
rlf! {
    // =========================================================================
    // Basic Card Types
    // =========================================================================

    card = :a { one: "card", other: "cards" };
    character = :a { one: "character", other: "characters" };
    event = :an { one: "event", other: "events" };

    // =========================================================================
    // Pronouns and References
    // =========================================================================

    this_character = "this character";
    that_character = "that character";
    these_characters = "these characters";
    those_characters = "those characters";
    them = "them";
    it = "it";

    // =========================================================================
    // Ownership
    // =========================================================================

    ally = :an { one: "ally", other: "allies" };
    your_card = { one: "your card", other: "your cards" };
    your_event = { one: "your event", other: "your events" };
    enemy = :an { one: "enemy", other: "enemies" };
    enemy_card = :an { one: "enemy card", other: "enemy cards" };
    enemy_event = :an { one: "enemy event", other: "enemy events" };

    // =========================================================================
    // Standalone Reference Phrases
    // =========================================================================

    an_ally = "an ally";
    an_enemy = "an enemy";
    a_character = "a character";
    a_card = "a card";
    an_event = "an event";

    // =========================================================================
    // Compositional Phrases
    // =========================================================================

    allied(entity) = "allied {entity:one}";
    allied_plural(entity) = "allied {entity:other}";
    enemy_modified(entity) = "enemy {entity:one}";
    enemy_modified_plural(entity) = "enemy {entity:other}";

    not_a(entity) = "a character that is not {@a entity}";
    ally_not(entity) = "ally that is not {@a entity}";
    non_entity_enemy(entity) = "non-{entity:one} enemy";
    characters_not_plural(entity) = "characters that are not {entity:other}";
    allies_not_plural(entity) = "allies that are not {entity:other}";

    // =========================================================================
    // Constraint Patterns
    // =========================================================================

    with_spark(base, spark, op) = "{base:one} with spark {spark}{op}";
    with_spark_plural(base, spark, op) = "{base:other} with spark {spark}{op}";
    with_cost(base, cost, op) = "{base:one} with cost {cost}{op}";
    with_cost_plural(base, cost, op) = "{base:other} with cost {cost}{op}";

    with_materialized(base) = "{base:one} with a {materialized} ability";
    with_materialized_plural(base) = "{base:other} with {materialized} abilities";
    with_activated(base) = "{base:one} with an activated ability";
    with_activated_plural(base) = "{base:other} with activated abilities";

    // =========================================================================
    // Complex Comparisons
    // =========================================================================

    with_spark_less_than_energy(base) =
        "{base:one} with spark less than the amount of {energy_symbol} paid";
    with_cost_less_than_allied(base, counting) =
        "{base:one} with cost less than the number of allied {counting:other}";
    with_cost_less_than_abandoned(base) =
        "{base:one} with cost less than the abandoned ally's cost";
    with_spark_less_than_abandoned(base) =
        "{base:one} with spark less than the abandoned ally's spark";
    with_spark_less_than_abandoned_enemy(base) =
        "{base:one} with spark less than that ally's spark";
    with_spark_less_than_abandoned_count(base) =
        "{base:one} with spark less than the number of allies abandoned this turn";
    with_cost_less_than_void(base) =
        "{base:one} with cost less than the number of cards in your void";

    // =========================================================================
    // Other Patterns
    // =========================================================================

    event_could_dissolve(target) = "an event which could {dissolve} {target}";
    your_event_could_dissolve(target) = "your event which could {dissolve} {target}";
    events_could_dissolve(target) = "events which could {dissolve} {target}";
    your_events_could_dissolve(target) = "your events which could {dissolve} {target}";

    fast_modified(base) = "a {fast} {base:one}";
    fast_modified_plural(base) = "{fast} {base:other}";

    in_your_void(things) = "{things} in your void";
    in_opponent_void(things) = "{things} in the opponent's void";

    another(entity) = "another {entity:one}";
    other_plural(entities) = "other {entities:other}";
    for_each(entity) = "each {entity:one}";

    or_less = " or less";
    or_more = " or more";

    // Keywords
    dissolve = "<k>dissolve</k>";
    materialized = "<k>materialized</k>";
    fast = "<k>fast</k>";
    energy_symbol = "<e>●</e>";
}
```

---

## Part 4: The Russian Translation File

Russian uses the same phrase names but different variant selections. The translator
has full control over case, word order, and agreement:

```rust
// ru.rlf
// Russian translation for predicate serializer

// =========================================================================
// Basic Card Types
//
// Russian nouns decline for case and number.
// Number categories: one (1, 21), few (2-4, 22-24), many (5-20, 0)
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
    acc, gen: "персонажа",        // Animate masc: acc = gen
    acc.many, gen.many: "персонажей",
    ins.one: "персонажем",
    ins: "персонажами",
};

event = :neut :inan {
    nom, acc: "событие",          // Neuter inan: acc = nom
    nom.many, acc.many: "событий",
    gen: "события",
    gen.many: "событий",
    ins.one: "событием",
    ins: "событиями",
};

// =========================================================================
// Pronouns and References
// =========================================================================

this_character = "этот персонаж";
that_character = "тот персонаж";
these_characters = "эти персонажи";
those_characters = "те персонажи";
them = "их";
it = "его";

// =========================================================================
// Ownership: Your/Allied
// =========================================================================

ally = :masc :anim {
    nom.one: "союзник",
    nom: "союзники",
    nom.many: "союзников",
    acc, gen: "союзника",
    acc.many, gen.many: "союзников",
    ins.one: "союзником",
    ins: "союзниками",
};

your_card = {
    nom: "ваша карта",
    nom.many: "ваших карт",
    acc: "вашу карту",
    acc.many: "ваших карт",
    gen: "вашей карты",
    gen.many: "ваших карт",
    ins.one: "вашей картой",
    ins: "вашими картами",
};

your_event = {
    nom, acc: "ваше событие",
    nom.many, acc.many: "ваших событий",
    gen: "вашего события",
    gen.many: "ваших событий",
    ins.one: "вашим событием",
    ins: "вашими событиями",
};

// =========================================================================
// Ownership: Enemy
// =========================================================================

enemy = :masc :anim {
    nom.one: "враг",
    nom: "враги",
    nom.many: "врагов",
    acc, gen: "врага",
    acc.many, gen.many: "врагов",
    ins.one: "врагом",
    ins: "врагами",
};

enemy_card = {
    nom: "вражеская карта",
    nom.many: "вражеских карт",
    acc: "вражескую карту",
    acc.many: "вражеских карт",
    gen: "вражеской карты",
    gen.many: "вражеских карт",
    ins.one: "вражеской картой",
    ins: "вражескими картами",
};

enemy_event = {
    nom, acc: "вражеское событие",
    nom.many, acc.many: "вражеских событий",
    gen: "вражеского события",
    gen.many: "вражеских событий",
    ins.one: "вражеским событием",
    ins: "вражескими событиями",
};

// =========================================================================
// Standalone Reference Phrases
// =========================================================================

an_ally = "союзник";
an_enemy = "враг";
a_character = "персонаж";
a_card = "карта";
an_event = "событие";

// =========================================================================
// Compositional Phrases
//
// Allied/enemy modifiers must agree with noun gender.
// =========================================================================

allied_adj = {
    masc: "союзный",
    fem: "союзная",
    neut: "союзное",
};
allied(entity) = "{allied_adj:entity} {entity:nom.one}";
allied_plural(entity) = "союзных {entity:gen.many}";

enemy_adj = {
    masc: "вражеский",
    fem: "вражеская",
    neut: "вражеское",
};
enemy_modified(entity) = "{enemy_adj:entity} {entity:nom.one}";
enemy_modified_plural(entity) = "вражеских {entity:gen.many}";

// Negation: Russian uses instrumental case after "являться"
not_a(entity) = "персонаж, который не является {entity:ins.one}";
ally_not(entity) = "союзник, который не является {entity:ins.one}";
non_entity_enemy(entity) = "враг, не являющийся {entity:ins.one}";

characters_not_plural(entity) = "персонажи, которые не являются {entity:ins.other}";
allies_not_plural(entity) = "союзники, которые не являются {entity:ins.other}";

// =========================================================================
// Constraint Patterns
// =========================================================================

with_spark(base, spark, op) = "{base:nom.one} с искрой {spark}{op}";
with_spark_plural(base, spark, op) = "{base:nom.other} с искрой {spark}{op}";
with_cost(base, cost, op) = "{base:nom.one} со стоимостью {cost}{op}";
with_cost_plural(base, cost, op) = "{base:nom.other} со стоимостью {cost}{op}";

with_materialized(base) = "{base:nom.one} со способностью {materialized}";
with_materialized_plural(base) = "{base:nom.other} со способностями {materialized}";

with_activated(base) = "{base:nom.one} с активируемой способностью";
with_activated_plural(base) = "{base:nom.other} с активируемыми способностями";

// =========================================================================
// Complex Comparisons
// =========================================================================

with_spark_less_than_energy(base) =
    "{base:nom.one} с искрой меньше количества уплаченной {energy_symbol}";

// KEY EXAMPLE: {counting:gen.many} extracts genitive plural
with_cost_less_than_allied(base, counting) =
    "{base:nom.one} со стоимостью меньше количества союзных {counting:gen.many}";

with_cost_less_than_abandoned(base) =
    "{base:nom.one} со стоимостью меньше стоимости покинутого союзника";

with_spark_less_than_abandoned(base) =
    "{base:nom.one} с искрой меньше искры покинутого союзника";

with_spark_less_than_abandoned_enemy(base) =
    "{base:nom.one} с искрой меньше искры того союзника";

with_spark_less_than_abandoned_count(base) =
    "{base:nom.one} с искрой меньше количества союзников, покинутых в этом ходу";

with_cost_less_than_void(base) =
    "{base:nom.one} со стоимостью меньше количества карт в вашей бездне";

// =========================================================================
// Could Dissolve
// =========================================================================

event_could_dissolve(target) = "событие, способное {dissolve} {target}";
your_event_could_dissolve(target) = "ваше событие, способное {dissolve} {target}";
events_could_dissolve(target) = "события, способные {dissolve} {target}";
your_events_could_dissolve(target) = "ваши события, способные {dissolve} {target}";

// =========================================================================
// Fast Modifier
// =========================================================================

fast_modified(base) = "{fast} {base:nom.one}";
fast_modified_plural(base) = "{fast} {base:nom.other}";

// =========================================================================
// Void Location
// =========================================================================

in_your_void(things) = "{things} в вашей бездне";
in_opponent_void(things) = "{things} в бездне противника";

// =========================================================================
// Other Modifiers
// =========================================================================

another_adj = {
    masc: "другой",
    fem: "другая",
    neut: "другое",
};
another(entity) = "{another_adj:entity} {entity:nom.one}";
other_plural(entities) = "другие {entities:nom.other}";

each_adj = {
    masc: "каждый",
    fem: "каждая",
    neut: "каждое",
};
for_each(entity) = "{each_adj:entity} {entity:nom.one}";

// =========================================================================
// Operators
// =========================================================================

or_less = " или меньше";
or_more = " или больше";

// =========================================================================
// Keywords
// =========================================================================

dissolve = "<k>растворить</k>";
materialized = "<k>материализации</k>";
fast = "<k>быстрый</k>";
energy_symbol = "<e>●</e>";
```

---

## Part 5: The Refactored predicate_serializer.rs

The refactored serializer passes `Phrase` values instead of pre-rendered strings:

### Key Changes from Original

1. **Return `Phrase` instead of `String` for base types**
2. **Let RLF phrases do selection, not Rust functions**
3. **Remove separate `_singular` and `_plural` serialization functions**
4. **Call semantic phrases directly** — `strings::an_ally(locale)`, not
   `strings::with_article(locale, strings::ally(locale))`
5. **Pass `Phrase` to compositional phrases** — allows RLF to select forms

```rust
// predicate_serializer.rs

use ability_data::predicate::{CardPredicate, Predicate};
use crate::localization::{strings, Locale, Phrase};

/// Get the base phrase for a card type.
fn card_type_phrase(locale: &Locale, card_predicate: &CardPredicate) -> Phrase {
    match card_predicate {
        CardPredicate::Card => strings::card(locale),
        CardPredicate::Character => strings::character(locale),
        CardPredicate::Event => strings::event(locale),
        CardPredicate::CharacterType(subtype) => strings::subtype_phrase(locale, *subtype),
        _ => strings::character(locale),
    }
}

enum Ownership { Your, Enemy }

/// Get the ownership-qualified phrase.
fn ownership_phrase(locale: &Locale, ownership: Ownership, card_predicate: &CardPredicate) -> Phrase {
    match (ownership, card_predicate) {
        (Ownership::Your, CardPredicate::Character) => strings::ally(locale),
        (Ownership::Your, CardPredicate::Card) => strings::your_card(locale),
        (Ownership::Your, CardPredicate::Event) => strings::your_event(locale),
        (Ownership::Enemy, CardPredicate::Character) => strings::enemy(locale),
        (Ownership::Enemy, CardPredicate::Card) => strings::enemy_card(locale),
        (Ownership::Enemy, CardPredicate::Event) => strings::enemy_event(locale),
        _ => card_type_phrase(locale, card_predicate),
    }
}

/// Serialize a predicate to localized text.
pub fn serialize_predicate(locale: &Locale, predicate: &Predicate) -> String {
    match predicate {
        // Simple references
        Predicate::This => strings::this_character(locale).to_string(),
        Predicate::That => strings::that_character(locale).to_string(),
        Predicate::Them => strings::them(locale).to_string(),
        Predicate::It => strings::it(locale).to_string(),

        // Ownership predicates
        Predicate::Your(card_predicate) => {
            serialize_owned_predicate(locale, card_predicate, Ownership::Your)
        }
        Predicate::Enemy(card_predicate) => {
            serialize_owned_predicate(locale, card_predicate, Ownership::Enemy)
        }
        Predicate::Any(card_predicate) => {
            serialize_any_predicate(locale, card_predicate)
        }

        // Location predicates
        Predicate::YourVoid(card_predicate) => {
            let base = card_type_phrase(locale, card_predicate);
            strings::in_your_void(locale, base).to_string()
        }
        Predicate::EnemyVoid(card_predicate) => {
            let base = card_type_phrase(locale, card_predicate);
            strings::in_opponent_void(locale, base).to_string()
        }

        // Other predicates
        Predicate::Another(card_predicate) => {
            let base = card_type_phrase(locale, card_predicate);
            strings::another(locale, base).to_string()
        }
        Predicate::AnyOther(card_predicate) => {
            let base = card_type_phrase(locale, card_predicate);
            strings::other_plural(locale, base).to_string()
        }
    }
}

/// Serialize an owned predicate with constraints.
fn serialize_owned_predicate(
    locale: &Locale,
    card_predicate: &CardPredicate,
    ownership: Ownership,
) -> String {
    match card_predicate {
        // Simple types: call semantic phrase directly
        CardPredicate::Character => match ownership {
            Ownership::Your => strings::an_ally(locale).to_string(),
            Ownership::Enemy => strings::an_enemy(locale).to_string(),
        },
        CardPredicate::Card => match ownership {
            Ownership::Your => strings::your_card(locale).to_string(),
            Ownership::Enemy => strings::enemy_card(locale).to_string(),
        },
        CardPredicate::Event => match ownership {
            Ownership::Your => strings::your_event(locale).to_string(),
            Ownership::Enemy => strings::enemy_event(locale).to_string(),
        },

        // Spark constraint
        CardPredicate::CharacterWithSpark(spark, operator) => {
            let base = ownership_phrase(locale, ownership, &CardPredicate::Character);
            let op = serialize_operator(locale, operator);
            strings::with_spark(locale, base, spark.0, op)
        }

        // Cost constraint
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            let base = ownership_phrase(locale, ownership, target);
            let op = serialize_operator(locale, cost_operator);
            strings::with_cost(locale, base, cost.0, op)
        }

        // Cost compared to allied count
        // KEY: We pass Phrase for 'counting', RLF selects gen.many
        CardPredicate::CharacterWithCostComparedToControlled {
            target,
            count_matching,
            ..
        } => {
            let base = ownership_phrase(locale, ownership, target);
            let counting = card_type_phrase(locale, count_matching);
            strings::with_cost_less_than_allied(locale, base, counting)
        }

        _ => strings::a_character(locale).to_string(),
    }
}

fn serialize_operator(locale: &Locale, operator: &CostOperator) -> String {
    match operator {
        CostOperator::LessOrEqual => strings::or_less(locale).to_string(),
        CostOperator::GreaterOrEqual => strings::or_more(locale).to_string(),
        CostOperator::Equal => String::new(),
    }
}
```

---

## Part 6: How Selection Solves the Problem

### The Flow for "characters with cost less than the number of allied characters"

**English:**
```
strings::with_cost_less_than_allied(locale, base, counting)

base = strings::character(locale)     // Phrase with one="character", other="characters"
counting = strings::character(locale) // Same Phrase

Template: "{base:one} with cost less than the number of allied {counting:other}"
Result:   "character with cost less than the number of allied characters"
```

**Russian:**
```
strings::with_cost_less_than_allied(locale, base, counting)

base = strings::character(locale)     // Phrase with nom.one="персонаж", gen.many="персонажей"
counting = strings::character(locale) // Same Phrase

Template: "{base:nom.one} со стоимостью меньше количества союзных {counting:gen.many}"
Result:   "персонаж со стоимостью меньше количества союзных персонажей"
```

The same Rust code produces grammatically correct output in both languages because:
1. Rust passes `Phrase` values with language-appropriate variants
2. Each language's template uses selectors appropriate for that language
3. English uses simple `:one`/`:other`, Russian uses `:nom.one`/`:gen.many`

### Gender Agreement with Tag-Based Selection

For modifiers like "another" that must agree with noun gender:

```
// ru.rlf
another_adj = {
    masc: "другой",
    fem: "другая",
    neut: "другое",
};
another(entity) = "{another_adj:entity} {entity:nom.one}";
```

When `entity` is `card` (tagged `:fem`), selection produces:
- `{another_adj:entity}` → looks up `entity`'s tag (`:fem`) → selects "другая"
- `{entity:nom.one}` → selects "карта"
- Result: "другая карта"

When `entity` is `character` (tagged `:masc`):
- `{another_adj:entity}` → looks up `:masc` → selects "другой"
- `{entity:nom.one}` → selects "персонаж"
- Result: "другой персонаж"

---

## Part 7: Benefits of This Approach

### For Translators

1. **Full control over grammatical forms**: Select any case/number variant
2. **Flexible word order**: Rearrange phrase templates as needed
3. **Gender agreement**: Tag-based selection handles adjective agreement
4. **No Rust knowledge required**: All work happens in `.rlf` files

### For Developers

1. **Simpler Rust code**: No separate singular/plural functions
2. **Less phrase explosion**: Compositional phrases replace combinatorial variants
3. **Type safety**: `Phrase` carries variants; selection validates at runtime
4. **Single code path**: Same Rust logic for all languages
5. **Semantic API**: Rust calls `strings::an_ally(locale)` — no linguistic decisions

### Comparison

| Aspect | Old Approach | New Approach |
|--------|--------------|--------------|
| Rust returns | `String` (pre-rendered) | `Phrase` (with variants) |
| Simple references | `with_article(ally)` | `an_ally()` |
| Case selection | Rust function choice | RLF `:case.number` selector |
| Phrase count | O(base × ownership × number) | O(base + ownership) |
| Translator control | Limited to text | Full grammatical control |
| Gender agreement | Separate phrases | Tag-based selection |
| Article decisions | Rust code | RLF phrases |

---

## Summary

Two key insights drive this design:

**1. Pass `Phrase`, not `String`.** Pre-rendering to `String` strips variant
information. By passing `Phrase` values through the system, variants are
preserved until the final RLF phrase renders them.

**2. Call semantic phrases, not presentation helpers.** Rust should call
`strings::an_ally(locale)` (semantic: "I need an ally reference"), not
`strings::with_article(locale, strings::ally(locale))` (presentational).
Linguistic decisions belong in RLF, not Rust.

The result:

1. **Translators choose** which grammatical forms to use via `:` selectors
2. **Gender agreement** works via tag-based selection on phrase parameters
3. **Word order** is fully controlled by the translator's phrase templates
4. **Rust remains language-agnostic** — it identifies semantic contexts, RLF
   handles all presentation

---

## Appendix: Phrase Transformation with `:from`

For patterns like character subtypes that need formatting while preserving
case/gender/number metadata, use `:from` metadata inheritance:

```rust
// ru.rlf
ancient = :masc :anim {
    nom.one: "Древний",
    nom.other: "Древние",
    acc.one: "Древнего",
    acc.other: "Древних",
    gen.one: "Древнего",
    gen.other: "Древних",
};

// :from(s) inherits all tags and variants
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";

// Templates can select any case/number from the inherited variants
dissolve_subtype(s) = "Растворите {subtype(s):acc.one}.";
// ancient → "Растворите <b>Древнего</b>."

all_subtypes(s) = "все {subtype(s):nom.other}";
// ancient → "все <b>Древние</b>"
```

The `:from(s)` modifier evaluates the template for every variant in `s`,
producing a `Phrase` with the same case/number structure as the source.
This enables the same "define once, use everywhere" pattern without losing
Russian's complex grammatical information.
