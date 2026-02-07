// Test @@, ::, and $$ escape sequences in rlf! macro
use rlf::{rlf, Locale};

rlf! {
    syntax_help = "Use {{name}} for interpolation and @@ for transforms.";
    ratio = "The ratio is 1::2.";
    mixed = "Use @@ and :: together.";
    dollar_escape = "The cost is $$5.";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let s = syntax_help(&locale);
    assert_eq!(
        s.to_string(),
        "Use {name} for interpolation and @ for transforms."
    );

    let r = ratio(&locale);
    assert_eq!(r.to_string(), "The ratio is 1:2.");

    let m = mixed(&locale);
    assert_eq!(m.to_string(), "Use @ and : together.");

    let d = dollar_escape(&locale);
    assert_eq!(d.to_string(), "The cost is $5.");
}
