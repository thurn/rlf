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

    // Source phrase with tags for :match inside variant entries
    sword = :masc { one: "sword", *other: "swords" };
    wand = :fem { one: "wand", *other: "wands" };

    // :from + variant blocks with :match inside entries
    magical($s) = :from($s) {
        one: :match($s) { masc: "magical {$s}", *fem: "magical {$s}" },
        *other: :match($s) { masc: "magical {$s}", *fem: "magical {$s}" }
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

    // :from + variant blocks with :match inside entries
    let s = sword(&locale);
    let m = magical(&locale, s);
    assert!(m.has_tag("masc"));
    assert_eq!(m.variant("one"), "magical sword");
    assert_eq!(m.variant("other"), "magical swords");

    let w = wand(&locale);
    let m2 = magical(&locale, w);
    assert!(m2.has_tag("fem"));
    assert_eq!(m2.variant("one"), "magical wand");
    assert_eq!(m2.variant("other"), "magical wands");
}
