// Test phrases with multiple parameters
use rlf::{rlf, Locale};

rlf! {
    // Single parameter
    greet(name) = "Hello, {name}!";

    // Multiple parameters
    introduce(name, title) = "{title} {name}";

    // Parameter used multiple times
    echo(msg) = "{msg} {msg} {msg}";

    // Parameter with selectors
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";

    // Multiple parameters with variants
    item = { one: "item", other: "items" };
    take(count, actor) = "{actor} takes {count} {item:count}.";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = greet(&locale, "World");
    let _ = introduce(&locale, "Smith", "Dr.");
    let _ = echo(&locale, "hello");
    let _ = draw(&locale, 3);
    let _ = take(&locale, 5, "Alice");
}
