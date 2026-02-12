// Test :match keyword parsing in the macro
use rlf::{Locale, rlf};

rlf! {
    // Simple single-param match
    cards($n) = :match($n) {
        1: "a card",
        *other: "{$n} cards",
    };

    // Match with named and numeric keys
    inventory($n) = :match($n) {
        0: "no items",
        1: "one item",
        *other: "{$n} items",
    };

    // Match with multi-key shorthand
    text_number($n) = :match($n) {
        one, 1: "one",
        *other: "{$n}",
    };

    // :from + :match (from before match)
    card = :a { one: "card", other: "cards" };
    count_cards_a($n, $s) = :from($s) :match($n) {
        1: "one {$s}",
        *other: "{$n} {$s}",
    };

    // :match + :from (match before from)
    count_cards_b($n, $s) = :match($n) :from($s) {
        1: "one {$s}",
        *other: "{$n} {$s}",
    };

    // Match with tags
    items($n) = :a :match($n) {
        1: "an item",
        *other: "{$n} items",
    };
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = cards(&locale, 1);
    let _ = cards(&locale, 3);
    let _ = inventory(&locale, 0);
    let _ = text_number(&locale, 1);
    let c = card(&locale);
    let _ = count_cards_a(&locale, 1, c.clone());
    let _ = count_cards_b(&locale, 3, c);
    let _ = items(&locale, 1);
}
