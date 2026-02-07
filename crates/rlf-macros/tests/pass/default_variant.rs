// Test * default variant marker compiles successfully
use rlf::{rlf, Locale};

rlf! {
    // Default on first variant
    card = { *one: "card", other: "cards" };

    // Default on second variant
    go = { present: "go", *past: "went", participle: "gone" };

    // No default marker (backward compat)
    day = { one: "day", other: "days" };

    // Default with tags
    item = :a { *one: "item", other: "items" };
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = card(&locale);
    let _ = go(&locale);
    let _ = day(&locale);
    let _ = item(&locale);
}
