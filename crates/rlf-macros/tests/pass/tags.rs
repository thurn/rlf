// Test :tag syntax after = sign
use rlf::{rlf, Locale};

rlf! {
    // Single tag after =
    sword = :masc "sword";

    // Multiple tags after =
    shield = :fem :inanimate "shield";

    // Tags on variant phrases after =
    creature = :neut {
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
