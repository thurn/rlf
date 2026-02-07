// Test literal arguments in phrase calls (number and string)
use rlf::{rlf, Locale};

rlf! {
    // Phrase with a simple template body
    cards($n) = "{$n} cards";

    // Phrase call with literal number
    pair = "You have {cards(2)}.";
    one_card = "You have {cards(1)}.";

    // Phrase that takes a string
    trigger($t) = "\u{25B8} {$t}";

    // Phrase call with literal string
    attack_trigger = "{trigger(\"Attack\")}";

    // Mixed: literal + parameter
    format_label($n) = "{$n} {cards($n)}";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = cards(&locale, 1);
    let _ = cards(&locale, 5);
    let _ = pair(&locale);
    let _ = one_card(&locale);
    let _ = trigger(&locale, "Test");
    let _ = attack_trigger(&locale);
    let _ = format_label(&locale, 3);
}
