//! Integration tests for template parsing.
//!
//! These tests validate the public API of the template parser against all
//! syntax forms documented in DESIGN_V2.md.

use rlf::parser::{Reference, Segment, Selector, TransformContext, parse_template};

// =============================================================================
// Basic parsing
// =============================================================================

#[test]
fn test_pure_literal() {
    let t = parse_template("Hello, world!").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("Hello, world!".into())]);
}

#[test]
fn test_empty_string() {
    let t = parse_template("").unwrap();
    assert_eq!(t.segments, vec![]);
}

#[test]
fn test_multiline_literal() {
    let t = parse_template("Line 1\nLine 2\nLine 3").unwrap();
    assert_eq!(
        t.segments,
        vec![Segment::Literal("Line 1\nLine 2\nLine 3".into())]
    );
}

// =============================================================================
// Parameter references with $ prefix
// =============================================================================

#[test]
fn test_parameter_reference() {
    let t = parse_template("Hello, {$name}!").unwrap();
    assert_eq!(t.segments.len(), 3);
    match &t.segments[1] {
        Segment::Interpolation {
            transforms,
            reference,
            selectors,
        } => {
            assert!(transforms.is_empty());
            assert_eq!(*reference, Reference::Parameter("name".into()));
            assert!(selectors.is_empty());
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_multiple_parameters() {
    let t = parse_template("Deal {$amount} damage to {$target}.").unwrap();
    assert_eq!(t.segments.len(), 5);

    match &t.segments[1] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Parameter("amount".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    match &t.segments[3] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Parameter("target".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Term/identifier references (bare names)
// =============================================================================

#[test]
fn test_term_reference() {
    let t = parse_template("{card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            selectors,
        } => {
            assert!(transforms.is_empty());
            assert_eq!(*reference, Reference::Identifier("card".into()));
            assert!(selectors.is_empty());
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Selection (static and parameterized)
// =============================================================================

#[test]
fn test_selection_literal() {
    let t = parse_template("{card:other}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors, &[Selector::Identifier("other".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_selection_parameter() {
    let t = parse_template("{card:$n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            assert_eq!(*reference, Reference::Identifier("card".into()));
            assert_eq!(selectors, &[Selector::Parameter("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_chained_selection_mixed() {
    let t = parse_template("{card:acc:$n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], Selector::Identifier("acc".into()));
            assert_eq!(selectors[1], Selector::Parameter("n".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_all_static_selectors() {
    let t = parse_template("{card:acc:one}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], Selector::Identifier("acc".into()));
            assert_eq!(selectors[1], Selector::Identifier("one".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_triple_chained_selection() {
    let t = parse_template("{word:case:gender:other}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 3);
            assert_eq!(selectors[0], Selector::Identifier("case".into()));
            assert_eq!(selectors[1], Selector::Identifier("gender".into()));
            assert_eq!(selectors[2], Selector::Identifier("other".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_parameter_with_static_selectors() {
    let t = parse_template("{$base:nom:one}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            assert_eq!(*reference, Reference::Parameter("base".into()));
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], Selector::Identifier("nom".into()));
            assert_eq!(selectors[1], Selector::Identifier("one".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_term_with_parameterized_selector() {
    let t = parse_template("{allied_adj:$entity}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            assert_eq!(*reference, Reference::Identifier("allied_adj".into()));
            assert_eq!(selectors, &[Selector::Parameter("entity".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Transforms
// =============================================================================

#[test]
fn test_transform_cap() {
    let t = parse_template("{@cap card}").unwrap();
    match &t.segments[0] {
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

#[test]
fn test_transform_on_parameter() {
    let t = parse_template("{@cap $name}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(*reference, Reference::Parameter("name".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_upper() {
    let t = parse_template("{@upper card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "upper");
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_lower() {
    let t = parse_template("{@lower card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "lower");
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_with_context() {
    let t = parse_template("{@der:acc karte}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "der");
            assert_eq!(
                transforms[0].context,
                TransformContext::Static("acc".into())
            );
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_chained_transforms() {
    let t = parse_template("{@cap @a card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms.len(), 2);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(transforms[1].name, "a");
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_triple_chained_transforms() {
    let t = parse_template("{@cap @upper @a card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms.len(), 3);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(transforms[1].name, "upper");
            assert_eq!(transforms[2].name, "a");
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_with_context_and_selector() {
    let t = parse_template("{@der:acc karte:one}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            selectors,
            ..
        } => {
            assert_eq!(transforms[0].name, "der");
            assert_eq!(
                transforms[0].context,
                TransformContext::Static("acc".into())
            );
            assert_eq!(selectors, &[Selector::Identifier("one".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Dynamic transform context
// =============================================================================

#[test]
fn test_transform_with_dynamic_context() {
    let t = parse_template("{@count($n) card}").unwrap();
    match &t.segments[0] {
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

#[test]
fn test_transform_with_both_contexts() {
    let t = parse_template("{@transform:lit($param) ref}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "transform");
            assert_eq!(
                transforms[0].context,
                TransformContext::Both("lit".into(), "param".into())
            );
            assert_eq!(*reference, Reference::Identifier("ref".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_dynamic_context_with_selector() {
    let t = parse_template("{@count($n) card:$n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            selectors,
        } => {
            assert_eq!(transforms[0].name, "count");
            assert_eq!(transforms[0].context, TransformContext::Dynamic("n".into()));
            assert_eq!(*reference, Reference::Identifier("card".into()));
            assert_eq!(selectors, &[Selector::Parameter("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_static_context_unchanged() {
    let t = parse_template("{@der:acc card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "der");
            assert_eq!(
                transforms[0].context,
                TransformContext::Static("acc".into())
            );
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Phrase calls
// =============================================================================

#[test]
fn test_phrase_call_with_param_arg() {
    let t = parse_template("{subtype($s)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => match reference {
            Reference::PhraseCall { name, args } => {
                assert_eq!(name, "subtype");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Reference::Parameter("s".into()));
            }
            _ => panic!("expected phrase call"),
        },
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_with_term_arg() {
    let t = parse_template("{subtype(ancient)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => match reference {
            Reference::PhraseCall { name, args } => {
                assert_eq!(name, "subtype");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Reference::Identifier("ancient".into()));
            }
            _ => panic!("expected phrase call"),
        },
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_multiple_args() {
    let t = parse_template("{foo($x, $y, $z)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => match reference {
            Reference::PhraseCall { name, args } => {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 3);
                assert_eq!(args[0], Reference::Parameter("x".into()));
                assert_eq!(args[1], Reference::Parameter("y".into()));
                assert_eq!(args[2], Reference::Parameter("z".into()));
            }
            _ => panic!("expected phrase call"),
        },
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_mixed_args() {
    let t = parse_template("{foo($x, term_name)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => match reference {
            Reference::PhraseCall { name, args } => {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Reference::Parameter("x".into()));
                assert_eq!(args[1], Reference::Identifier("term_name".into()));
            }
            _ => panic!("expected phrase call"),
        },
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_with_transform() {
    let t = parse_template("{@cap subtype($s)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "subtype");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Reference::Parameter("s".into()));
                }
                _ => panic!("expected phrase call"),
            }
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_with_selector() {
    let t = parse_template("{subtype($s):other}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "subtype");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Reference::Parameter("s".into()));
                }
                _ => panic!("expected phrase call"),
            }
            assert_eq!(selectors, &[Selector::Identifier("other".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_with_param_selector() {
    let t = parse_template("{subtype($s):$n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "subtype");
                    assert_eq!(args[0], Reference::Parameter("s".into()));
                }
                _ => panic!("expected phrase call"),
            }
            assert_eq!(selectors, &[Selector::Parameter("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Escape sequences
// =============================================================================

#[test]
fn test_escape_braces() {
    let t = parse_template("Use {{name}} syntax").unwrap();
    assert_eq!(
        t.segments,
        vec![Segment::Literal("Use {name} syntax".into())]
    );
}

#[test]
fn test_escape_close_brace() {
    let t = parse_template("Value is }}").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("Value is }".into())]);
}

#[test]
fn test_dollar_literal_in_text() {
    let t = parse_template("Price is $5").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("Price is $5".into())]);
}

#[test]
fn test_at_literal_in_text() {
    let t = parse_template("user@example.com").unwrap();
    assert_eq!(
        t.segments,
        vec![Segment::Literal("user@example.com".into())]
    );
}

#[test]
fn test_colon_literal_in_text() {
    let t = parse_template("ratio 1:2").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("ratio 1:2".into())]);
}

#[test]
fn test_dollar_escape_in_interpolation() {
    let t = parse_template("{$$}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("$".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_escape_in_interpolation_context() {
    let t = parse_template("Before {{escaped}} {actual} after").unwrap();
    assert_eq!(t.segments.len(), 3);
    assert_eq!(t.segments[0], Segment::Literal("Before {escaped} ".into()));
    match &t.segments[1] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("actual".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
    assert_eq!(t.segments[2], Segment::Literal(" after".into()));
}

// =============================================================================
// Automatic capitalization (bare identifiers only)
// =============================================================================

#[test]
fn test_auto_capitalization() {
    let t = parse_template("{Card}").unwrap();
    match &t.segments[0] {
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

#[test]
fn test_auto_capitalization_preserves_rest() {
    let t = parse_template("{CardName}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(*reference, Reference::Identifier("cardName".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_auto_capitalization_with_underscores() {
    let t = parse_template("{Phrase_Name}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(*reference, Reference::Identifier("phrase_name".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_auto_capitalization_with_selectors() {
    let t = parse_template("{Card:$n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            selectors,
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(*reference, Reference::Identifier("card".into()));
            assert_eq!(selectors, &[Selector::Parameter("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_auto_capitalization_with_existing_transforms() {
    let t = parse_template("{@a Card}").unwrap();
    match &t.segments[0] {
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
    }
}

#[test]
fn test_no_auto_cap_on_parameter() {
    // $Name should NOT trigger auto-capitalization - $ always means parameter
    let result = parse_template("{$Name}");
    // $Name is parsed as Reference::Parameter("Name") with no auto-cap
    let t = result.unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert!(transforms.is_empty());
            assert_eq!(*reference, Reference::Parameter("Name".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_lowercase_no_auto_cap() {
    let t = parse_template("{card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert!(transforms.is_empty());
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Complex examples from DESIGN_V2.md
// =============================================================================

#[test]
fn test_draw_cards_template() {
    let t = parse_template("Draw {$n} {card:$n}.").unwrap();
    assert_eq!(t.segments.len(), 5);

    assert_eq!(t.segments[0], Segment::Literal("Draw ".into()));

    match &t.segments[1] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Parameter("n".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    assert_eq!(t.segments[2], Segment::Literal(" ".into()));

    match &t.segments[3] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            assert_eq!(*reference, Reference::Identifier("card".into()));
            assert_eq!(selectors, &[Selector::Parameter("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    assert_eq!(t.segments[4], Segment::Literal(".".into()));
}

#[test]
fn test_russian_template() {
    let t = parse_template("{card:acc:$n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], Selector::Identifier("acc".into()));
            assert_eq!(selectors[1], Selector::Parameter("n".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_german_definite_article() {
    let t = parse_template("Zerstöre {@der:acc karte}.").unwrap();
    assert_eq!(t.segments.len(), 3);

    assert_eq!(t.segments[0], Segment::Literal("Zerstöre ".into()));

    match &t.segments[1] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms[0].name, "der");
            assert_eq!(
                transforms[0].context,
                TransformContext::Static("acc".into())
            );
            assert_eq!(*reference, Reference::Identifier("karte".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    assert_eq!(t.segments[2], Segment::Literal(".".into()));
}

#[test]
fn test_dissolve_subtype() {
    let t = parse_template("Dissolve {@a subtype($s)}.").unwrap();
    assert_eq!(t.segments.len(), 3);

    match &t.segments[1] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms[0].name, "a");
            match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "subtype");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Reference::Parameter("s".into()));
                }
                _ => panic!("expected phrase call"),
            }
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_dissolve_all() {
    let t = parse_template("Dissolve all {subtype($s):other}.").unwrap();
    assert_eq!(t.segments.len(), 3);

    match &t.segments[1] {
        Segment::Interpolation {
            reference,
            selectors,
            ..
        } => {
            match reference {
                Reference::PhraseCall { name, args } => {
                    assert_eq!(name, "subtype");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Reference::Parameter("s".into()));
                }
                _ => panic!("expected phrase call"),
            }
            assert_eq!(selectors, &[Selector::Identifier("other".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_adjacent_interpolations() {
    let t = parse_template("{a}{b}{c}").unwrap();
    assert_eq!(t.segments.len(), 3);
    for i in 0..3 {
        match &t.segments[i] {
            Segment::Interpolation { .. } => {}
            Segment::Literal(_) => panic!("expected interpolation at index {i}"),
        }
    }
}

#[test]
fn test_whitespace_in_interpolation() {
    let t = parse_template("{ card }").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("card".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_whitespace_around_transform() {
    let t = parse_template("{ @cap   card }").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(*reference, Reference::Identifier("card".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_underscore_in_identifier() {
    let t = parse_template("{some_name}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("some_name".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_numbers_in_identifier() {
    let t = parse_template("{card2}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("card2".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_unicode_in_literal() {
    let t = parse_template("Zerstöre die Karte").unwrap();
    assert_eq!(
        t.segments,
        vec![Segment::Literal("Zerstöre die Karte".into())]
    );
}

#[test]
fn test_unicode_around_interpolation() {
    let t = parse_template("Возьмите {$n} карт").unwrap();
    assert_eq!(t.segments.len(), 3);
    assert_eq!(t.segments[0], Segment::Literal("Возьмите ".into()));
    assert_eq!(t.segments[2], Segment::Literal(" карт".into()));
}

// =============================================================================
// Error cases
// =============================================================================

#[test]
fn test_unclosed_brace() {
    let result = parse_template("{name");
    assert!(result.is_err());
}

#[test]
fn test_empty_interpolation() {
    let result = parse_template("{}");
    assert!(result.is_err());
}

#[test]
fn test_unmatched_close_brace_in_text() {
    let result = parse_template("text } more");
    assert!(result.is_err());
}
