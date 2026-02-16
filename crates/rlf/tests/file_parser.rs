//! Integration tests for .rlf file parsing

use rlf::parser::{
    DefinitionKind, PhraseBody, Reference, Segment, TransformContext, VariantEntryBody, parse_file,
};
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
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_with_parameters() {
    let phrases = parse_file(r#"greet($name) = "Hello, {$name}!";"#).unwrap();
    assert_eq!(phrases[0].parameters, vec!["name"]);
}

#[test]
fn test_phrase_with_multiple_parameters() {
    let phrases =
        parse_file(r#"damage($amount, $target) = "Deal {$amount} to {$target}.";"#).unwrap();
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
    let phrases = parse_file(r#"subtype($s) = :from($s) "{$s}";"#).unwrap();
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
}

#[test]
fn test_phrase_with_tags_and_from() {
    let phrases = parse_file(r#"subtype($s) = :an :from($s) "<b>{$s}</b>";"#).unwrap();
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
        _ => panic!("expected variants"),
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
        _ => panic!("expected variants"),
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
        _ => panic!("expected variants"),
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
        greet($name) = "Hello, {$name}!";
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
        _ => panic!("expected variants"),
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
        draw($n) = "Draw {$n} {card:$n}.";
        subtype($s) = :from($s) "<b>{$s}</b>";
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
        _ => panic!("expected variants"),
    }
}

#[test]
fn test_template_with_interpolations() {
    let phrases = parse_file(r#"draw($n) = "Draw {$n} {card:$n}.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            // Should have: "Draw " + {$n} + " " + {card:$n} + "."
            assert!(t.segments.len() >= 3);
            // First segment is literal
            assert!(matches!(&t.segments[0], Segment::Literal(_)));
        }
        _ => panic!("expected simple body"),
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
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_call_in_template() {
    let phrases = parse_file(r#"dissolve($s) = "Dissolve {@a subtype($s)}.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            // Find the interpolation segment
            let interp = t
                .segments
                .iter()
                .find(|s| matches!(s, Segment::Interpolation { .. }));
            assert!(interp.is_some());
        }
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_escape_sequences_in_template() {
    let phrases = parse_file(r#"syntax_help = "Use {{$name}} for interpolation.";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            // The {{ should become a single { in the literal
            if let Segment::Literal(text) = &t.segments[0] {
                assert!(text.contains("{$name}"));
            } else {
                panic!("expected literal segment");
            }
        }
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_template_supports_escaped_quote() {
    let phrases = parse_file(r#"quote = "He said \"hi\".";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            assert_eq!(t.segments, vec![Segment::Literal("He said \"hi\".".into())]);
        }
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_template_supports_unicode_escape() {
    let phrases = parse_file(r#"symbol = "\u{25CF}";"#).unwrap();
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            assert_eq!(t.segments, vec![Segment::Literal("●".into())]);
        }
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_template_rejects_invalid_unicode_escape() {
    let result = parse_file(r#"symbol = "\u{}";"#);
    assert!(result.is_err());
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
        draw($n) = "Возьмите {$n} {card:$n}.";
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
        _ => panic!("expected variants"),
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
        _ => panic!("expected variants"),
    }
}

#[test]
fn test_whitespace_flexibility() {
    // Test various whitespace arrangements
    let phrases = parse_file(
        r#"
            hello="Hello!";
            greet( $name ) = "Hello, { $name }!" ;
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
                    assert_eq!(transforms[0].context, TransformContext::None);
                    assert_eq!(*reference, Reference::Identifier("card".into()));
                }
                Segment::Literal(_) => panic!("expected interpolation"),
            }
        }
        _ => panic!("expected simple body"),
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
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_auto_capitalization_with_selector() {
    let phrases = parse_file(r#"draw($n) = "{Card:$n}";"#).unwrap();
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
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_auto_capitalization_with_underscores() {
    let phrases = parse_file(r#"draw = "Draw {Fire_Elemental}.";"#).unwrap();
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
                    assert_eq!(*reference, Reference::Identifier("fire_elemental".into()));
                }
                Segment::Literal(_) => panic!("expected interpolation"),
            }
        }
        _ => panic!("expected simple body"),
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
        _ => panic!("expected simple body"),
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
        test($entity) = "{adj:$entity} {$entity:nom:one}";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 5);
}

// =============================================================================
// Dynamic transform context
// =============================================================================

#[test]
fn test_dynamic_transform_context() {
    let phrases = parse_file(r#"draw($n) = "抽{@count($n) card}";"#).unwrap();
    assert_eq!(phrases.len(), 1);
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            assert_eq!(t.segments.len(), 2);
            match &t.segments[1] {
                Segment::Interpolation {
                    transforms,
                    reference,
                    ..
                } => {
                    assert_eq!(transforms.len(), 1);
                    assert_eq!(transforms[0].name, "count");
                    assert_eq!(transforms[0].context, TransformContext::Dynamic("n".into()));
                    assert_eq!(*reference, Reference::Identifier("card".into()));
                }
                Segment::Literal(_) => panic!("expected interpolation"),
            }
        }
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_static_transform_context_unchanged() {
    let phrases = parse_file(r#"destroy = "Zerstöre {@der:acc karte}.";"#).unwrap();
    assert_eq!(phrases.len(), 1);
    match &phrases[0].body {
        PhraseBody::Simple(t) => match &t.segments[1] {
            Segment::Interpolation { transforms, .. } => {
                assert_eq!(transforms[0].name, "der");
                assert_eq!(
                    transforms[0].context,
                    TransformContext::Static("acc".into())
                );
            }
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_both_static_and_dynamic_context() {
    let phrases = parse_file(r#"test($n) = "{@transform:lit($n) ref}";"#).unwrap();
    assert_eq!(phrases.len(), 1);
    match &phrases[0].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation { transforms, .. } => {
                assert_eq!(transforms[0].name, "transform");
                assert_eq!(
                    transforms[0].context,
                    TransformContext::Both("lit".into(), "n".into())
                );
            }
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        _ => panic!("expected simple body"),
    }
}

// =============================================================================
// Literal arguments in phrase calls
// =============================================================================

#[test]
fn test_phrase_call_with_number_literal() {
    // Phrases with params use simple template bodies, not variant blocks
    let phrases = parse_file(
        r#"
        cards($n) = "{$n} cards";
        pair = "You have {cards(2)}.";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 2);
    match &phrases[1].body {
        PhraseBody::Simple(t) => {
            let interp = t
                .segments
                .iter()
                .find(|s| matches!(s, Segment::Interpolation { .. }))
                .expect("expected interpolation");
            match interp {
                Segment::Interpolation { reference, .. } => match reference {
                    Reference::PhraseCall { name, args } => {
                        assert_eq!(name, "cards");
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], Reference::NumberLiteral(2));
                    }
                    _ => panic!("expected phrase call"),
                },
                Segment::Literal(_) => panic!("expected interpolation"),
            }
        }
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_call_with_string_literal() {
    let phrases = parse_file(
        r#"
        trigger($t) = "<b>{$t}</b>";
        example = "{trigger("Attack")}";
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 2);
    match &phrases[1].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation { reference, .. } => match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "trigger");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Reference::StringLiteral("Attack".into()));
                }
                _ => panic!("expected phrase call"),
            },
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_call_with_string_literal_escape() {
    let phrases = parse_file(
        r#"
        wrap($s) = "[{$s}]";
        example = "{wrap("He said \"hi\"")}";
    "#,
    )
    .unwrap();
    match &phrases[1].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation { reference, .. } => match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "wrap");
                    assert_eq!(args[0], Reference::StringLiteral("He said \"hi\"".into()));
                }
                _ => panic!("expected phrase call"),
            },
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        _ => panic!("expected simple body"),
    }
}

#[test]
fn test_phrase_call_mixed_literal_args() {
    let phrases = parse_file(
        r#"
        fmt($a, $b, $c) = "{$a}{$b}{$c}";
        example($x) = "{fmt(42, "hello", $x)}";
    "#,
    )
    .unwrap();
    match &phrases[1].body {
        PhraseBody::Simple(t) => match &t.segments[0] {
            Segment::Interpolation { reference, .. } => match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "fmt");
                    assert_eq!(args.len(), 3);
                    assert_eq!(args[0], Reference::NumberLiteral(42));
                    assert_eq!(args[1], Reference::StringLiteral("hello".into()));
                    assert_eq!(args[2], Reference::Parameter("x".into()));
                }
                _ => panic!("expected phrase call"),
            },
            Segment::Literal(_) => panic!("expected interpolation"),
        },
        _ => panic!("expected simple body"),
    }
    drop(phrases);
}

// =============================================================================
// DefinitionKind tests (Term vs Phrase)
// =============================================================================

#[test]
fn test_definition_with_params_is_phrase() {
    let phrases = parse_file(r#"greet($name) = "Hello, {$name}!";"#).unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
}

#[test]
fn test_definition_without_params_is_term() {
    let phrases = parse_file(r#"hello = "Hello, world!";"#).unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Term);
}

#[test]
fn test_term_with_variants_is_valid() {
    let phrases = parse_file(
        r#"
        card = { one: "card", other: "cards" };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Term);
    assert!(matches!(phrases[0].body, PhraseBody::Variants(_)));
}

#[test]
fn test_params_with_variant_block_is_valid() {
    let phrases = parse_file(
        r#"
        cards($n) = { one: "{$n} card", other: "{$n} cards" };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert!(matches!(phrases[0].body, PhraseBody::Variants(_)));
    assert_eq!(phrases[0].parameters, vec!["n"]);
}

#[test]
fn test_empty_parens_is_error() {
    let result = parse_file(r#"name() = "x";"#);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("empty parameter list"),
        "expected empty parameter list error, got: {err}"
    );
}

#[test]
fn test_from_without_params_is_error() {
    let result = parse_file(r#"thing = :from($s) "text";"#);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains(":from requires parameters"),
        "expected :from requires parameters error, got: {err}"
    );
}

#[test]
fn test_phrase_with_simple_body_is_valid() {
    let phrases = parse_file(r#"greet($name) = "Hello, {$name}!";"#).unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert!(matches!(phrases[0].body, PhraseBody::Simple(_)));
}

#[test]
fn test_phrase_with_from_is_valid() {
    let phrases = parse_file(r#"subtype($s) = :from($s) "<b>{$s}</b>";"#).unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
}

#[test]
fn test_phrase_with_from_and_variant_block() {
    let phrases = parse_file(
        r#"
        enemy_subtype($s) = :from($s) {
            nom: "вражеский {$s}",
            acc: "вражеского {$s}",
            *gen: "вражеского {$s}"
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 3);
            assert!(!entries[0].is_default);
            assert!(!entries[1].is_default);
            assert!(entries[2].is_default);
        }
        _ => panic!("expected variant body for :from + variant block"),
    }
}

#[test]
fn test_phrase_without_from_variant_block_accepted() {
    let phrases = parse_file(r#"draw($n) = { one: "{$n} card", other: "{$n} cards" };"#).unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert!(matches!(phrases[0].body, PhraseBody::Variants(_)));
}

#[test]
fn test_mixed_terms_and_phrases() {
    let phrases = parse_file(
        r#"
        card = :a { one: "card", other: "cards" };
        draw($n) = "Draw {$n} {card:$n}.";
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Term);
    assert_eq!(phrases[1].kind, DefinitionKind::Phrase);
}

// =============================================================================
// Default variant marker (*) tests
// =============================================================================

#[test]
fn test_default_marker_on_variant() {
    let phrases = parse_file(
        r#"
        card = { *one: "card", other: "cards" };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 2);
            assert!(entries[0].is_default);
            assert!(!entries[1].is_default);
        }
        _ => panic!("expected variants"),
    }
}

#[test]
fn test_no_default_marker() {
    let phrases = parse_file(
        r#"
        card = { one: "card", other: "cards" };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert!(!entries[0].is_default);
            assert!(!entries[1].is_default);
        }
        _ => panic!("expected variants"),
    }
}

#[test]
fn test_default_marker_on_second_variant() {
    let phrases = parse_file(
        r#"
        card = { one: "card", *other: "cards" };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert!(!entries[0].is_default);
            assert!(entries[1].is_default);
        }
        _ => panic!("expected variants"),
    }
}

#[test]
fn test_multiple_default_markers_is_error() {
    let result = parse_file(
        r#"
        card = { *one: "card", *other: "cards" };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("multiple '*'"),
        "expected multiple * error, got: {err}"
    );
}

#[test]
fn test_default_marker_on_multidimensional_key_is_error() {
    let result = parse_file(
        r#"
        card = { *nom.one: "card", other: "cards" };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("multi-dimensional"),
        "expected multi-dimensional error, got: {err}"
    );
}

#[test]
fn test_default_marker_with_tags() {
    let phrases = parse_file(
        r#"
        card = :a { *one: "card", other: "cards" };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].tags, vec![Tag::new("a")]);
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert!(entries[0].is_default);
        }
        _ => panic!("expected variants"),
    }
}

// =============================================================================
// :match keyword tests
// =============================================================================

#[test]
fn test_single_param_match() {
    let phrases = parse_file(
        r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases.len(), 1);
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert_eq!(phrases[0].match_params, vec!["n"]);
    match &phrases[0].body {
        PhraseBody::Match(branches) => {
            assert_eq!(branches.len(), 2);
            // First branch: 1: "a card"
            assert_eq!(branches[0].keys.len(), 1);
            assert_eq!(branches[0].keys[0].value, "1");
            assert_eq!(branches[0].keys[0].default_dimensions, vec![false]);
            // Second branch: *other: "{$n} cards"
            assert_eq!(branches[1].keys.len(), 1);
            assert_eq!(branches[1].keys[0].value, "other");
            assert_eq!(branches[1].keys[0].default_dimensions, vec![true]);
        }
        _ => panic!("expected match body"),
    }
}

#[test]
fn test_multi_param_match() {
    let phrases = parse_file(
        r#"
        n_allied($n, $entity) = :match($n, $entity) {
            1.masc: "one masc",
            1.*neut: "one neut",
            *other.masc: "other masc",
            other.*neut: "other neut",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].match_params, vec!["n", "entity"]);
    match &phrases[0].body {
        PhraseBody::Match(branches) => {
            assert_eq!(branches.len(), 4);
            // 1.masc
            assert_eq!(branches[0].keys[0].value, "1.masc");
            assert_eq!(branches[0].keys[0].default_dimensions, vec![false, false]);
            // 1.*neut
            assert_eq!(branches[1].keys[0].value, "1.neut");
            assert_eq!(branches[1].keys[0].default_dimensions, vec![false, true]);
            // *other.masc
            assert_eq!(branches[2].keys[0].value, "other.masc");
            assert_eq!(branches[2].keys[0].default_dimensions, vec![true, false]);
            // other.*neut
            assert_eq!(branches[3].keys[0].value, "other.neut");
            assert_eq!(branches[3].keys[0].default_dimensions, vec![false, true]);
        }
        _ => panic!("expected match body"),
    }
}

#[test]
fn test_match_named_and_numeric_keys() {
    let phrases = parse_file(
        r#"
        cards($n) = :match($n) {
            0: "no cards",
            1: "a card",
            2: "a pair of cards",
            *other: "{$n} cards",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Match(branches) => {
            assert_eq!(branches.len(), 4);
            assert_eq!(branches[0].keys[0].value, "0");
            assert_eq!(branches[1].keys[0].value, "1");
            assert_eq!(branches[2].keys[0].value, "2");
            assert_eq!(branches[3].keys[0].value, "other");
            assert!(branches[3].keys[0].default_dimensions[0]);
        }
        _ => panic!("expected match body"),
    }
}

#[test]
fn test_match_multi_key_shorthand() {
    let phrases = parse_file(
        r#"
        cards($n) = :match($n) {
            one, two: "few cards",
            *other: "{$n} cards",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Match(branches) => {
            assert_eq!(branches.len(), 2);
            assert_eq!(branches[0].keys.len(), 2);
            assert_eq!(branches[0].keys[0].value, "one");
            assert_eq!(branches[0].keys[1].value, "two");
        }
        _ => panic!("expected match body"),
    }
}

#[test]
fn test_from_then_match() {
    let phrases = parse_file(
        r#"
        count_sub($n, $s) = :from($s) :match($n) {
            1: "one {$s}",
            *other: "{$n} {$s}",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
    assert_eq!(phrases[0].match_params, vec!["n"]);
    assert!(matches!(phrases[0].body, PhraseBody::Match(_)));
}

#[test]
fn test_match_then_from() {
    let phrases = parse_file(
        r#"
        count_sub($n, $s) = :match($n) :from($s) {
            1: "one {$s}",
            *other: "{$n} {$s}",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].from_param, Some("s".to_string()));
    assert_eq!(phrases[0].match_params, vec!["n"]);
    assert!(matches!(phrases[0].body, PhraseBody::Match(_)));
}

#[test]
fn test_match_missing_default_is_error() {
    let result = parse_file(
        r#"
        cards($n) = :match($n) {
            1: "a card",
            other: "{$n} cards",
        };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("no '*' default"),
        "expected missing default error, got: {err}"
    );
}

#[test]
fn test_match_multiple_defaults_is_error() {
    let result = parse_file(
        r#"
        cards($n) = :match($n) {
            *one: "a card",
            *other: "{$n} cards",
        };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("multiple '*'"),
        "expected multiple defaults error, got: {err}"
    );
}

#[test]
fn test_match_param_not_declared_is_error() {
    let result = parse_file(
        r#"
        cards($n) = :match($unknown) {
            *other: "cards",
        };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not declared"),
        "expected undeclared param error, got: {err}"
    );
}

#[test]
fn test_phrase_variant_block_without_match_is_valid() {
    let phrases = parse_file(
        r#"
        cards($n) = { one: "{$n} card", other: "{$n} cards" };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert!(matches!(phrases[0].body, PhraseBody::Variants(_)));
}

#[test]
fn test_match_with_tags() {
    let phrases = parse_file(
        r#"
        cards($n) = :a :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
    )
    .unwrap();
    assert_eq!(phrases[0].tags, vec![Tag::new("a")]);
    assert_eq!(phrases[0].match_params, vec!["n"]);
}

#[test]
fn test_match_trailing_comma() {
    let phrases = parse_file(
        r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Match(branches) => assert_eq!(branches.len(), 2),
        _ => panic!("expected match body"),
    }
}

#[test]
fn test_match_no_trailing_comma() {
    let phrases = parse_file(
        r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards"
        };
    "#,
    )
    .unwrap();
    match &phrases[0].body {
        PhraseBody::Match(branches) => assert_eq!(branches.len(), 2),
        _ => panic!("expected match body"),
    }
}

// =============================================================================
// :match inside variant entries (:from + variant block + :match)
// =============================================================================

#[test]
fn test_from_variant_block_match_inside_entry() {
    let phrases = parse_file(
        r#"
        enemy($s) = :from($s) {
            nom: :match($s) { masc: "вражеский {$s}", *fem: "вражеская {$s}" },
            *acc: :match($s) { masc: "вражеского {$s}", *fem: "вражескую {$s}" }
        };
    "#,
    )
    .unwrap();

    assert_eq!(phrases[0].name, "enemy");
    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert_eq!(phrases[0].from_param, Some("s".to_string()));

    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 2);

            // nom entry should have a :match body
            assert_eq!(entries[0].keys, vec!["nom"]);
            assert!(!entries[0].is_default);
            match &entries[0].body {
                VariantEntryBody::Match {
                    match_params,
                    branches,
                } => {
                    assert_eq!(match_params, &["s"]);
                    assert_eq!(branches.len(), 2);
                }
                VariantEntryBody::Template(_) => panic!("expected match body in nom entry"),
            }

            // acc entry should also have a :match body
            assert_eq!(entries[1].keys, vec!["acc"]);
            assert!(entries[1].is_default);
            match &entries[1].body {
                VariantEntryBody::Match {
                    match_params,
                    branches,
                } => {
                    assert_eq!(match_params, &["s"]);
                    assert_eq!(branches.len(), 2);
                }
                VariantEntryBody::Template(_) => panic!("expected match body in acc entry"),
            }
        }
        _ => panic!("expected variants body"),
    }
}

#[test]
fn test_from_variant_block_mixed_template_and_match() {
    let phrases = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "добрый {$s}",
            *acc: :match($s) { masc: "доброго {$s}", *fem: "добрую {$s}" }
        };
    "#,
    )
    .unwrap();

    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 2);

            // nom entry is a plain template
            assert!(matches!(&entries[0].body, VariantEntryBody::Template(_)));

            // acc entry is a :match block
            assert!(matches!(&entries[1].body, VariantEntryBody::Match { .. }));
        }
        _ => panic!("expected variants body"),
    }
}

#[test]
fn test_from_variant_block_match_with_multiple_definitions() {
    // Minimal reproduction: three variant entries with :match blocks
    let phrases = parse_file(
        r#"enemy($s) = :from($s) { a: :match($s) { *x: "1" }, b: :match($s) { *x: "2" }, *c: :match($s) { *x: "3" } };"#,
    )
    .unwrap();

    assert_eq!(phrases.len(), 1);
}

#[test]
fn test_from_variant_block_match_undeclared_param_error() {
    let result = parse_file(
        r#"
        enemy($s) = :from($s) {
            nom: :match($x) { masc: "bad {$s}", *fem: "bad {$s}" },
            *acc: "ok {$s}"
        };
    "#,
    );

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not declared"),
        "expected undeclared param error, got: {err}"
    );
}

#[test]
fn test_bodyless_from() {
    let phrases = parse_file(r#"wrapper($p) = :from($p);"#).unwrap();
    assert_eq!(phrases.len(), 1);
    assert_eq!(phrases[0].name, "wrapper");
    assert_eq!(phrases[0].from_param, Some("p".to_string()));
    match &phrases[0].body {
        PhraseBody::Simple(t) => {
            assert_eq!(t.segments.len(), 1);
            match &t.segments[0] {
                Segment::Interpolation {
                    transforms,
                    reference,
                    selectors,
                } => {
                    assert!(
                        transforms.is_empty(),
                        "body-less :from should have no transforms"
                    );
                    assert_eq!(
                        reference,
                        &Reference::Parameter("p".to_string()),
                        "body-less :from should reference the :from parameter"
                    );
                    assert!(
                        selectors.is_empty(),
                        "body-less :from should have no selectors"
                    );
                }
                Segment::Literal(_) => {
                    panic!("expected interpolation segment in body-less :from")
                }
            }
        }
        _ => panic!("expected simple body for body-less :from"),
    }
}

#[test]
fn test_bodyless_from_with_tags() {
    let phrases = parse_file(r#"wrapper($p) = :masc :from($p);"#).unwrap();
    assert_eq!(phrases[0].tags, vec![Tag::new("masc")]);
    assert_eq!(phrases[0].from_param, Some("p".to_string()));
    assert!(matches!(phrases[0].body, PhraseBody::Simple(_)));
}

#[test]
fn test_bodyless_from_without_from_is_error() {
    let result = parse_file(r#"bad($p) = ;"#);
    assert!(
        result.is_err(),
        "definition with no body and no :from should be a parse error"
    );
}

// =============================================================================
// Parameterized phrase variant blocks
// =============================================================================

#[test]
fn test_phrase_variant_block_with_match_inside_entries() {
    let phrases = parse_file(
        r#"
        action($c) = {
            *imp: :match($c) { 1: "draw a card", *other: "draw {$c} cards" },
            inf: :match($c) { 1: "to draw a card", *other: "to draw {$c} cards" },
        };
    "#,
    )
    .unwrap();

    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    match &phrases[0].body {
        PhraseBody::Variants(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].keys, vec!["imp"]);
            assert!(entries[0].is_default);
            assert!(matches!(&entries[0].body, VariantEntryBody::Match { .. }));
            assert_eq!(entries[1].keys, vec!["inf"]);
            assert!(!entries[1].is_default);
            assert!(matches!(&entries[1].body, VariantEntryBody::Match { .. }));
        }
        _ => panic!("expected variants body"),
    }
}

#[test]
fn test_phrase_variant_block_with_tags() {
    let phrases = parse_file(
        r#"
        draw($c) = :a {
            *imp: "draw {$c} cards",
            inf: "to draw {$c} cards",
        };
    "#,
    )
    .unwrap();

    assert_eq!(phrases[0].kind, DefinitionKind::Phrase);
    assert_eq!(phrases[0].tags, vec![Tag::new("a")]);
    assert!(matches!(phrases[0].body, PhraseBody::Variants(_)));
}
