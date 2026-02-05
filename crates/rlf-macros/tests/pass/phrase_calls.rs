// Test phrase call syntax with arguments
use rlf::{rlf, Locale};

rlf! {
    // Base phrases
    name = "World";
    greeting = "Hello";

    // Phrase calling another phrase
    hello = "{greeting}, {name}!";

    // Phrase with parameter calling another phrase
    greet(who) = "{greeting}, {who}!";

    // Nested phrase references
    formal = "{@cap greeting}";
    formal_hello = "{formal}, {name}.";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = name(&locale);
    let _ = greeting(&locale);
    let _ = hello(&locale);
    let _ = greet(&locale, "Alice");
    let _ = formal(&locale);
    let _ = formal_hello(&locale);
}
