// Test :tag syntax
use rlf::{rlf, Locale};

rlf! {
    // Single tag
    :masc sword = "sword";

    // Multiple tags
    :fem :inanimate shield = "shield";

    // Tags on variant phrases
    :neut creature = {
        one: "creature",
        other: "creatures"
    };
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = sword(&locale);
    let _ = shield(&locale);
    let _ = creature(&locale);
}
