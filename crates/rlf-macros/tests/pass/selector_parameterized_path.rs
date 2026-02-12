// Parameterized selectors are runtime-validated; they should compile in macros.
use rlf::{Locale, rlf};

rlf! {
    card = {
        nom.one: "card",
        nom.other: "cards"
    };

    pick($n) = "{card:nom:$n}";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    assert_eq!(pick(&locale, 1).to_string(), "card");
    assert_eq!(pick(&locale, 3).to_string(), "cards");
}
