// Test :* explicit default selector syntax
use rlf::{rlf, Locale};

rlf! {
    card = :a { one: "card", *other: "cards" };

    // Use :* to explicitly request the default variant
    show_default = "The default is {card:*}.";

    // :* with a parameter reference
    show_param_default($item) = "Default: {$item:*}";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    // card's default variant is "cards" (marked with *)
    let result = show_default(&locale);
    assert_eq!(result.to_string(), "The default is cards.");

    // Pass a phrase with variants and use :* to get its default
    let c = card(&locale);
    let result = show_param_default(&locale, c);
    assert_eq!(result.to_string(), "Default: cards");
}
