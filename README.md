# RLF

The Rust Localization Framework: a localization DSL embedded in Rust via macros.

## Overview

RLF generates a language-agnostic API from phrase definitions. The source language (typically English) is compiled via the `rlf!` macro into Rust functions with full IDE autocomplete. Translations are loaded at runtime via the interpreter.

```rust
rlf! {
    hello = "Hello, world!";
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}
```

This generates functions that take a `Locale` parameter:

```rust
let locale = Locale::with_language("en");
strings::hello(&locale);      // "Hello, world!"
strings::draw(&locale, 1);    // "Draw 1 card."
strings::draw(&locale, 3);    // "Draw 3 cards."
```

## Features

- **Compile-time validation** for source language phrases
- **Immediate IDE support** - new phrases appear in autocomplete instantly
- **Runtime translation loading** - no recompilation needed for new languages
- **Grammatical metadata** - tags like `:fem`, `:masc`, `:a`, `:an` enable correct article and adjective agreement
- **Transforms** - the `@` operator modifies text (e.g., `@cap` for capitalize, `@a` for indefinite articles)

## Syntax

RLF has four primitives: **phrase**, **parameter**, **variant**, and **selection**.

### Phrases and Parameters

```rust
rlf! {
    greeting = "Welcome!";
    greet(name) = "Hello, {name}!";
    damage(amount, target) = "Deal {amount} damage to {target}.";
}
```

### Variants and Selection

Variants provide multiple forms. The `:` operator selects based on a value:

```rust
rlf! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}
// draw(1) → "Draw 1 card."
// draw(5) → "Draw 5 cards."
```

### Metadata Tags

Tags provide grammatical hints for transforms and selection:

```rust
rlf! {
    card = :a "card";
    event = :an "event";
    draw_one = "Draw {@a card}.";   // "Draw a card."
    play_one = "Play {@a event}.";  // "Play an event."
}
```

### Transforms

The `@` operator applies transforms. They chain right-to-left:

```rust
rlf! {
    card = "card";
    title = "{@cap card}";          // "Card"
    heading = "{@cap @a card}";     // "A card"
}
```

## Design

See `docs/DESIGN.md` for the complete language specification.
