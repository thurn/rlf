// Test :from($param) metadata inheritance syntax
use rlf::{rlf, Locale, Value};

rlf! {
    // Source phrase with tags and variants
    ancient = :an {
        one: "Ancient",
        other: "Ancients"
    };

    // :from modifier inherits tags and variants from parameter
    subtype($s) = :from($s) "<b>{$s}</b>";

    // :from with explicit tags
    decorated($s) = :masc :from($s) "[{$s}]";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let ancient = ancient(&locale);
    let sub = subtype(&locale, ancient.clone());

    // Should inherit :an tag and produce variants
    assert!(sub.has_tag("an"));
    assert_eq!(sub.variant("one"), "<b>Ancient</b>");
    assert_eq!(sub.variant("other"), "<b>Ancients</b>");

    let dec = decorated(&locale, ancient);
    assert_eq!(dec.variant("one"), "[Ancient]");
}
