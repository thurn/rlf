// Test variant phrases with dotted keys
use rlf::{rlf, Locale};

rlf! {
    // Simple variants
    card = { one: "card", other: "cards" };

    // Dotted variant keys (compound selectors)
    noun = {
        nom.one: "thing",
        nom.other: "things",
        acc.one: "thing",
        acc.other: "things"
    };

    // Multi-key variants (shared templates)
    day = {
        one: "day",
        other, many: "days"
    };
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = card(&locale);
    let _ = noun(&locale);
    let _ = day(&locale);
}
