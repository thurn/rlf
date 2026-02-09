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

    // :from + variant blocks for per-variant templates
    enemy_subtype($s) = :from($s) {
        one: "enemy {$s}",
        *other: "enemy {$s}"
    };

    // :from + variant blocks with additional parameters
    labeled_subtype($s, $label) = :from($s) {
        one: "{$label} {$s}",
        *other: "{$label} {$s}"
    };
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

    let dec = decorated(&locale, ancient.clone());
    assert_eq!(dec.variant("one"), "[Ancient]");

    // :from + variant blocks: per-variant templates
    let enemy = enemy_subtype(&locale, ancient.clone());
    assert!(enemy.has_tag("an"));
    assert_eq!(enemy.variant("one"), "enemy Ancient");
    assert_eq!(enemy.variant("other"), "enemy Ancients");

    // :from + variant blocks with additional parameters
    let labeled = labeled_subtype(&locale, ancient, Value::from("allied"));
    assert!(labeled.has_tag("an"));
    assert_eq!(labeled.variant("one"), "allied Ancient");
    assert_eq!(labeled.variant("other"), "allied Ancients");
}
