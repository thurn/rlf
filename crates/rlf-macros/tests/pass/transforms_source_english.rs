// Test source-language English transforms and aliases accepted by macro validation.
use rlf::{Locale, rlf};

rlf! {
    card = :a { one: "card", other: "cards" };
    event = :an { one: "event", other: "events" };

    the_card = "{@the card}";
    one_event = "{@an event}";
    many_cards = "{@plural card}";
}

fn main() {
    let mut locale = Locale::with_language("en");
    register_source_phrases(&mut locale);

    assert_eq!(the_card(&locale).to_string(), "the card");
    assert_eq!(one_event(&locale).to_string(), "an event");
    assert_eq!(many_cards(&locale).to_string(), "cards");
}
