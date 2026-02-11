// Test :from($param) with a transform applied directly to that parameter.
use rlf::{rlf, Locale};

rlf! {
    card = :a "card";
    event = :an "event";

    predicate_with_indefinite_article($p) = :from($p) "{@a $p}";
}

fn main() {
    let mut locale = Locale::with_language("en");
    register_source_phrases(&mut locale);

    let card_phrase = card(&locale);
    let event_phrase = event(&locale);

    let card_result = predicate_with_indefinite_article(&locale, card_phrase);
    let event_result = predicate_with_indefinite_article(&locale, event_phrase);

    assert!(card_result.has_tag("a"));
    assert!(event_result.has_tag("an"));
    assert_eq!(card_result.to_string(), "a card");
    assert_eq!(event_result.to_string(), "an event");
}
