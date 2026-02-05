// Test basic macro usage compiles successfully
use rlf::{rlf, Locale};

rlf! {
    hello = "Hello, world!";
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _h = hello(&locale);
    let _c = card(&locale);
    let _d = draw(&locale, 3);

    // Test phrase_ids module
    let _id = phrase_ids::HELLO;
    let _card_id = phrase_ids::CARD;
    let _draw_id = phrase_ids::DRAW;
}
