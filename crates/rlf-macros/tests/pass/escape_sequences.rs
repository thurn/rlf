// Test escape sequences in rlf! macro:
// - $, @, : are literal in regular text (no escaping needed)
// - {{ and }} produce literal braces
// - Inside {}: $$ -> literal $, @@ -> literal @, :: -> literal :
use rlf::{rlf, Locale};

rlf! {
    // Text escapes: {{ and }} produce literal braces
    syntax_help = "Use {{$name}} for parameters.";

    // $, @, : are literal in regular text
    price = "The cost is $5.";
    email = "user@example.com";
    ratio = "The ratio is 1:2.";
    mixed = "Price $5, email user@example.com, ratio 1:2.";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let s = syntax_help(&locale);
    assert_eq!(s.to_string(), "Use {$name} for parameters.");

    let p = price(&locale);
    assert_eq!(p.to_string(), "The cost is $5.");

    let e = email(&locale);
    assert_eq!(e.to_string(), "user@example.com");

    let r = ratio(&locale);
    assert_eq!(r.to_string(), "The ratio is 1:2.");

    let m = mixed(&locale);
    assert_eq!(
        m.to_string(),
        "Price $5, email user@example.com, ratio 1:2."
    );
}
