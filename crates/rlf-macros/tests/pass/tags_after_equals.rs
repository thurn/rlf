// Test :tag syntax after = sign (DESIGN.md canonical syntax)
use rlf::{rlf, Locale};

rlf! {
    // Single tag after =
    card = :a "card";

    // Multiple tags after =
    event = :an "event";

    // Tags on variant phrases after =
    creature = :a {
        one: "creature",
        other: "creatures"
    };

    // :from after = sign
    subtype(s) = :from(s) "<b>{s}</b>";

    // Tags and :from after = sign
    decorated(s) = :masc :from(s) "[{s}]";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let c = card(&locale);
    assert!(c.has_tag("a"));

    let e = event(&locale);
    assert!(e.has_tag("an"));

    let cr = creature(&locale);
    assert!(cr.has_tag("a"));

    let sub = subtype(&locale, card(&locale));
    assert!(sub.has_tag("a")); // inherited from card

    let _ = decorated(&locale, card(&locale));
}
