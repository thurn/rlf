// Test parameterized phrases with variant blocks (no :match required)
use rlf::{rlf, Locale};

rlf! {
    // Basic phrase with variant block
    draw_cards_effect($c) = {
        *imp: "draw {$c} cards",
        inf: "to draw {$c} cards",
    };

    // Phrase variant block with nested :match inside entries
    action($c) = {
        *imp: :match($c) {
            1: "draw a card",
            *other: "draw {$c} cards",
        },
        inf: :match($c) {
            1: "to draw a card",
            *other: "to draw {$c} cards",
        },
    };
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    // Default variant (imp)
    let result = draw_cards_effect(&locale, 3);
    assert_eq!(result.to_string(), "draw 3 cards");

    // Select inf variant
    let result = draw_cards_effect(&locale, 3);
    assert_eq!(result.variant("inf").to_string(), "to draw 3 cards");

    // Nested :match within variant block
    let result = action(&locale, 1);
    assert_eq!(result.to_string(), "draw a card");
    assert_eq!(result.variant("inf").to_string(), "to draw a card");

    let result = action(&locale, 5);
    assert_eq!(result.to_string(), "draw 5 cards");
    assert_eq!(result.variant("inf").to_string(), "to draw 5 cards");
}
