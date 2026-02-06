//! Integration tests for .rlf file parsing

use rlf::parser::{PhraseBody, Reference, Segment, parse_file};
use rlf::types::Tag;

#[test]
fn test_simple_phrase() {
    let phrases = parse_file(r#"hello = "Hello, world!";"#).unwrap();
    assert_eq!(phrases.len(), 1);
    assert_eq!(phrases[0].name, "hello");
    assert!(phrases[0].parameters.is_empty());
    assert!(phrases[0].tags.is_empty());
    assert!(phrases[0].from_param.is_none());
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            assert_eq!(t.segments.len(), 1);
        }
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_with_parameters() {
    let phrases = parse_file(r#"greet(name) = "Hello, {name}!";"#).unwrap();
    assert_eq!(phrases[0].parameters, vec!["name"]);
}

#[test]
fn test_phrase_with_multiple_parameters() {
    let phrases = parse_file(r#"damage(amount, target) = "Deal {amount} to {target}.";"#).unwrap();
    assert_eq!(phrases[0].parameters, vec!["amount", "target"]);
}

#[test]
fn test_phrase_with_tag() {
    let phrases = parse_file(r#"card = :fem "carta";"#).unwrap();
    assert_eq!(phrases[0].tags, vec![Tag::new("fem")]);
}

#[test]
fn test_phrase_with_multiple_tags() {
    let phrases = parse_file(r#"card = :a :noun "card";"#).unwrap();
    assert_eq!(phrases[0].tags.len(), 2);
}

#[test]
fn test_phrase_with_from() {
    let phrases = parse_file(r#"subtype(s) = :from(s) "{s}";"#).unwrap();
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
}

#[test]
fn test_phrase_with_tags_and_from() {
    let phrases = parse_file(r#"subtype(s) = :an :from(s) "<b>{s}</b>";"#).unwrap();
    assert_eq!(phrases[0].tags, vec![Tag::new("an")]);
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
}

#[test]
fn test_simple_variants() {
    let phrases = parse_file(
        r#"
        card = {
            one: "card",
            other: "cards",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 2);
        }
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_multidimensional_variants() {
    let phrases = parse_file(
        r#"
        card = {
            nom.one: "carta",
            nom.other: "cartas",
            acc.one: "carta",
            acc.other: "cartas",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 4);
            assert_eq!(entries[0].keys, vec!["nom.one"]);
        }
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_multikey_variants() {
    let phrases = parse_file(
        r#"
        card = {
            nom, acc: "card",
            nom.other, acc.other: "cards",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].keys, vec!["nom", "acc"]);
        }
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_trailing_comma() {
    let phrases = parse_file(
        r#"
        card = {
            one: "card",
            other: "cards",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 1);
}

#[test]
fn test_line_comments() {
    let phrases = parse_file(
        r#"
        // This is a comment
        hello = "Hello!";
        // Another comment
        bye = "Goodbye!"; // inline comment
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 2);
}

#[test]
fn test_multiple_phrases() {
    let phrases = parse_file(
        r#"
        hello = "Hello!";
        goodbye = "Goodbye!";
        greet(name) = "Hello, {name}!";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 3);
}

#[test]
fn test_empty_file() {
    let phrases = parse_file("").unwrap();
    assert!(phrases.is_empty());
}

#[test]
fn test_only_comments() {
    let phrases = parse_file(
        r#"
        // Just comments
        // Nothing else
    "#,
    )
    .unwrap();
    assert!(phrases.is_empty());
}

#[test]
fn test_variants_with_tags() {
    let phrases = parse_file(
        r#"
        card = :fem {
            one: "carta",
            other: "cartas",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].tags, vec![Tag::new("fem")]);
    match &phrases[0].body {
        PhraseBody::Variants(_) => {}
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_complex_file() {
    // A realistic .rlf file
    let phrases = parse_file(
        r#"
        // English source file
        card = :a { one: "card", other: "cards" };
        event = :an "event";
        draw(n) = "Draw {n} {card:n}.";
        subtype(s) = :from(s) "<b>{s}</b>";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 4);
}

#[test]
fn test_snake_case_names() {
    let phrases = parse_file(r#"my_phrase = "text";"#).unwrap();
    assert_eq!(phrases[0].name, "my_phrase");
}

#[test]
fn test_error_invalid_name() {
    // Names must be snake_case (start with lowercase)
    let result = parse_file(r#"MyPhrase = "text";"#);
    assert!(result.is_err());
}

#[test]
fn test_error_missing_semicolon() {
    let result = parse_file(r#"hello = "Hello!""#);
    assert!(result.is_err());
}

#[test]
fn test_wildcard_fallback_key() {
    // "nom" without a dimension is a fallback
    let phrases = parse_file(
        r#"
        card = {
            nom: "carta",
            nom.other: "cartas",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            // "nom" should be parsed as a single-segment key
            assert!(entries.iter().any(|e| e.keys == vec!["nom"]));
        }
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_template_with_interpolations() {
    let phrases = parse_file(r#"draw(n) = "Draw {n} {card:n}.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            // Should have: "Draw " + {n} + " " + {card:n} + "."
            assert!(t.segments.len() >= 3);
            // First segment is literal
            assert!(matches!(&t.segments[0], Segment::Literal(_)));
        }
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_template_with_transforms() {
    let phrases = parse_file(r#"heading = "{@cap @a card}";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation { transforms, .. } => {
                assert_eq!(transforms.len(), 2);
                assert_eq!(transforms[0].name, "cap");
                assert_eq!(transforms[1].name, "a");
            }
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_call_in_template() {
    let phrases = parse_file(r#"dissolve(s) = "Dissolve {@a subtype(s)}.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            // Find the interpolation segment
            let interp = t
                .segments
                .iter()
                .find(|s| matches!(s, Segment::Interpolation { .. }));
            assert!(interp.is_some());
        }
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_escape_sequences_in_template() {
    let phrases = parse_file(r#"syntax_help = "Use {{name}} for interpolation.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            // The {{ should become a single { in the literal
            if let Segment::Literal(text) = &t.segments[0] {
                assert!(text.contains("{name}"));
            } else {
                panic!("expected literal segment");
            }
        }
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_russian_translation_file() {
    let phrases = parse_file(
        r#"
        // Russian translation
        card = :fem {
            one: "карта",
            few: "карты",
            many: "карт",
        };
        draw(n) = "Возьмите {n} {card:n}.";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 2);
    assert_eq!(phrases[0].name, "card");
    assert_eq!(phrases[0].tags, vec![Tag::new("fem")]);
}

#[test]
fn test_german_case_variants() {
    let phrases = parse_file(
        r#"
        karte = :fem {
            nom.one: "Karte",
            nom.other: "Karten",
            acc.one: "Karte",
            acc.other: "Karten",
            dat.one: "Karte",
            dat.other: "Karten",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 6);
        }
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_error_has_line_info() {
    let result = parse_file(
        r#"
        hello = "Hello!";
        BadName = "text";
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    // Error should include line information
    assert!(err_str.contains("line") || err_str.contains(":"));
}

#[test]
fn test_numbers_in_phrase_name() {
    let phrases = parse_file(r#"draw2 = "Draw 2 cards.";"#).unwrap();
    assert_eq!(phrases[0].name, "draw2");
}

#[test]
fn test_underscore_prefix_rejected() {
    // Names must start with lowercase letter, not underscore
    let result = parse_file(r#"_private = "text";"#);
    assert!(result.is_err());
}

#[test]
fn test_variant_without_trailing_comma() {
    let phrases = parse_file(
        r#"
        card = {
            one: "card",
            other: "cards"
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 1);
}

#[test]
fn test_single_variant() {
    let phrases = parse_file(
        r#"
        word = {
            only: "only option"
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 1);
        }
        PhraseBody::Simple(_) => panic!("expected variants"),
    }
}

#[test]
fn test_whitespace_flexibility() {
    // Test various whitespace arrangements
    let phrases = parse_file(
        r#"
            hello="Hello!";
            greet( name ) = "Hello, { name }!" ;
        "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 2);
}

#[test]
fn test_auto_capitalization_adds_cap_transform() {
    let phrases = parse_file(r#"draw = "Draw {Card}.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            let interp = t
                .segments
                .iter()
                .find(|s| matches!(s, Segment::Interpolation { .. }))
                .expect("expected interpolation");
            match interp {
                Segment::Interpolation {
                    transforms,
                    reference,
                    ..
                } => {
                    assert_eq!(transforms.len(), 1);
                    assert_eq!(transforms[0].name, "cap");
                    assert!(transforms[0].context.is_none());
                    assert_eq!(*reference, Reference::Identifier("card".into()));
                }
                Segment::Literal(_) => panic!("expected interpolation"),
            }
        }
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_auto_capitalization_with_existing_transforms() {
    let phrases = parse_file(r#"draw = "{@a Card}";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation {
                transforms,
                reference,
                ..
            } => {
                assert_eq!(transforms.len(), 2);
                assert_eq!(transforms[0].name, "cap");
                assert_eq!(transforms[1].name, "a");
                assert_eq!(*reference, Reference::Identifier("card".into()));
            }
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_auto_capitalization_with_selector() {
    let phrases = parse_file(r#"draw(n) = "{Card:n}";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation {
                transforms,
                reference,
                selectors,
            } => {
                assert_eq!(transforms.len(), 1);
                assert_eq!(transforms[0].name, "cap");
                assert_eq!(*reference, Reference::Identifier("card".into()));
                assert_eq!(selectors.len(), 1);
            }
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_no_auto_capitalization_for_lowercase() {
    let phrases = parse_file(r#"draw = "Draw {card}.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            let interp = t
                .segments
                .iter()
                .find(|s| matches!(s, Segment::Interpolation { .. }))
                .expect("expected interpolation");
            match interp {
                Segment::Interpolation { transforms, .. } => {
                    assert!(transforms.is_empty());
                }
                Segment::Literal(_) => panic!("expected interpolation"),
            }
        }
        PhraseBody::Variants(_) => panic!("expected simple body"),
    }
}

#[test]
fn test_multiple_variant_blocks() {
    let phrases = parse_file(
        r#"
        a = { one: "x", two: "y" };
        b = { three: "z", four: "w" };
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 2);
}

#[test]
fn test_multiple_tagged_variant_blocks() {
    // Test multiple variant blocks with dotted keys and multi-selector templates
    let phrases = parse_file(
        r#"
        card = :fem :inan {
            nom.one: "karta",
            nom: "karty",
            gen.many: "kart"
        };
        character = :masc :anim {
            nom.one: "char",
            nom: "chars",
            gen.many: "charov"
        };
        thing = :neut :inan {
            nom: "thing",
            gen.many: "things"
        };
        adj = {
            masc: "m",
            fem: "f",
            neut: "n"
        };
        test(entity) = "{adj:entity} {entity:nom:one}";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 5);
}
