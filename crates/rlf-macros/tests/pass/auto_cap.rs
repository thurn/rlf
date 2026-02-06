// Test auto-capitalization syntax: {Card} -> {@cap card}
use rlf::{rlf, Locale};

rlf! {
    card = { one: "card", other: "cards" };
    auto_cap = "Draw a {Card}.";
    auto_cap_with_selector = "Draw {Card:other}.";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = auto_cap(&locale);
    let _ = auto_cap_with_selector(&locale);
}
