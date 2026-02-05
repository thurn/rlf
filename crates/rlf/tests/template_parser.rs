//! Integration tests for template parsing.
//!
//! These tests validate the public API of the template parser against all
//! syntax forms documented in DESIGN.md.

use rlf::parser::{Reference, Segment, Selector, parse_template};

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
// Simple interpolations (LANG-02, LANG-03)
// =============================================================================

#[test]
fn test_simple_parameter() {
    let t = parse_template("Hello, {name}!").unwrap();
    assert_eq!(t.segments.len(), 3);
    match &t.segments[1] {
        Segment::Interpolation {
            transforms,
            reference,
            selectors,
        } => {
            assert!(transforms.is_empty());
            assert_eq!(*reference, Reference::Identifier("name".into()));
            assert!(selectors.is_empty());
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_multiple_parameters() {
    let t = parse_template("Deal {amount} damage to {target}.").unwrap();
    assert_eq!(t.segments.len(), 5);

    // Verify first interpolation
    match &t.segments[1] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("amount".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    // Verify second interpolation
    match &t.segments[3] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("target".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Selection (LANG-04, LANG-05, LANG-06)
// =============================================================================

#[test]
fn test_selection_literal() {
    // LANG-04: Literal selection
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
    // LANG-05: Parameter-based selection
    let t = parse_template("{card:n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            // At parse time, we don't distinguish parameters from literals
            assert_eq!(selectors, &[Selector::Identifier("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_chained_selection() {
    // LANG-06: Multi-dimensional selection
    let t = parse_template("{card:acc:n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], Selector::Identifier("acc".into()));
            assert_eq!(selectors[1], Selector::Identifier("n".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_triple_chained_selection() {
    let t = parse_template("{word:case:gender:n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 3);
            assert_eq!(selectors[0], Selector::Identifier("case".into()));
            assert_eq!(selectors[1], Selector::Identifier("gender".into()));
            assert_eq!(selectors[2], Selector::Identifier("n".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Transforms (LANG-07, LANG-08, LANG-09)
// =============================================================================

#[test]
fn test_transform_cap() {
    // LANG-07: @cap transform
    let t = parse_template("{@cap name}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert!(transforms[0].context.is_none());
            assert_eq!(*reference, Reference::Identifier("name".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_upper() {
    let t = parse_template("{@upper name}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "upper");
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_lower() {
    let t = parse_template("{@lower name}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "lower");
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_transform_with_context() {
    // LANG-08: Transform with context
    let t = parse_template("{@der:acc karte}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert_eq!(transforms[0].name, "der");
            assert_eq!(
                transforms[0].context,
                Some(Selector::Identifier("acc".into()))
            );
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_chained_transforms() {
    // LANG-09: Chained transforms
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
    // Transform context is separate from phrase selectors
    // {@der:acc karte:one} = @der with context "acc", karte with selector "one"
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
                Some(Selector::Identifier("acc".into()))
            );
            assert_eq!(selectors, &[Selector::Identifier("one".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Phrase calls (LANG-10)
// =============================================================================

#[test]
fn test_phrase_call_single_arg() {
    let t = parse_template("{subtype(s)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => match reference {
            Reference::PhraseCall { name, args } => {
                assert_eq!(name, "subtype");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], Reference::Identifier("s".into()));
            }
            Reference::Identifier(_) => panic!("expected phrase call"),
        },
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_multiple_args() {
    let t = parse_template("{foo(x, y, z)}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => match reference {
            Reference::PhraseCall { name, args } => {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 3);
                assert_eq!(args[0], Reference::Identifier("x".into()));
                assert_eq!(args[1], Reference::Identifier("y".into()));
                assert_eq!(args[2], Reference::Identifier("z".into()));
            }
            Reference::Identifier(_) => panic!("expected phrase call"),
        },
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_with_transform() {
    let t = parse_template("{@cap subtype(s)}").unwrap();
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
                }
                Reference::Identifier(_) => panic!("expected phrase call"),
            }
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_phrase_call_with_selector() {
    let t = parse_template("{subtype(s):other}").unwrap();
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
                }
                Reference::Identifier(_) => panic!("expected phrase call"),
            }
            assert_eq!(selectors, &[Selector::Identifier("other".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Escape sequences (LANG-11)
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
fn test_escape_at() {
    let t = parse_template("Use @@ for transforms").unwrap();
    assert_eq!(
        t.segments,
        vec![Segment::Literal("Use @ for transforms".into())]
    );
}

#[test]
fn test_escape_colon() {
    let t = parse_template("ratio 1::2").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("ratio 1:2".into())]);
}

#[test]
fn test_escape_close_brace() {
    let t = parse_template("Value is }}").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("Value is }".into())]);
}

#[test]
fn test_mixed_escapes() {
    let t = parse_template("{{x}} @@ ::").unwrap();
    assert_eq!(t.segments, vec![Segment::Literal("{x} @ :".into())]);
}

#[test]
fn test_escape_in_interpolation_context() {
    // Escapes work within literal text, not inside interpolations
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
// Automatic capitalization (LANG-12)
// =============================================================================

#[test]
fn test_auto_capitalization() {
    // {Card} -> @cap card
    let t = parse_template("{Card}").unwrap();
    match &t.segments[0] {
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

#[test]
fn test_auto_capitalization_preserves_rest() {
    // {CardName} -> @cap cardName
    let t = parse_template("{CardName}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            // Only the first letter is lowercased
            assert_eq!(*reference, Reference::Identifier("cardName".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_auto_capitalization_with_selectors() {
    // {Card:n} -> @cap card:n
    let t = parse_template("{Card:n}").unwrap();
    match &t.segments[0] {
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
    }
}

#[test]
fn test_auto_capitalization_with_existing_transforms() {
    // {@a Card} -> @cap @a card
    let t = parse_template("{@a Card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 2);
            assert_eq!(transforms[0].name, "cap"); // auto-cap prepended
            assert_eq!(transforms[1].name, "a"); // original transform
            assert_eq!(*reference, Reference::Identifier("card".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_lowercase_no_auto_cap() {
    // Lowercase reference should not add @cap
    let t = parse_template("{card}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { transforms, .. } => {
            assert!(transforms.is_empty());
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

// =============================================================================
// Complex examples from DESIGN.md
// =============================================================================

#[test]
fn test_draw_cards_template() {
    // "Draw {n} {card:n}."
    let t = parse_template("Draw {n} {card:n}.").unwrap();
    assert_eq!(t.segments.len(), 5);

    assert_eq!(t.segments[0], Segment::Literal("Draw ".into()));

    match &t.segments[1] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("n".into()));
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
            assert_eq!(selectors, &[Selector::Identifier("n".into())]);
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    assert_eq!(t.segments[4], Segment::Literal(".".into()));
}

#[test]
fn test_russian_template() {
    // Russian template with multi-dimensional selection
    let t = parse_template("{card:acc:n}").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { selectors, .. } => {
            assert_eq!(selectors.len(), 2);
            assert_eq!(selectors[0], Selector::Identifier("acc".into()));
            assert_eq!(selectors[1], Selector::Identifier("n".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_german_definite_article() {
    // German definite article with case context
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
                Some(Selector::Identifier("acc".into()))
            );
            assert_eq!(*reference, Reference::Identifier("karte".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }

    assert_eq!(t.segments[2], Segment::Literal(".".into()));
}

#[test]
fn test_dissolve_subtype() {
    // Phrase call with transform
    let t = parse_template("Dissolve {@a subtype(s)}.").unwrap();
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
                }
                Reference::Identifier(_) => panic!("expected phrase call"),
            }
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_dissolve_all() {
    // Phrase call with selector
    let t = parse_template("Dissolve all {subtype(s):other}.").unwrap();
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
                }
                Reference::Identifier(_) => panic!("expected phrase call"),
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
            Segment::Literal(_) => panic!("expected interpolation at index {}", i),
        }
    }
}

#[test]
fn test_whitespace_in_interpolation() {
    let t = parse_template("{ name }").unwrap();
    match &t.segments[0] {
        Segment::Interpolation { reference, .. } => {
            assert_eq!(*reference, Reference::Identifier("name".into()));
        }
        Segment::Literal(_) => panic!("expected interpolation"),
    }
}

#[test]
fn test_whitespace_around_transform() {
    let t = parse_template("{ @cap   name }").unwrap();
    match &t.segments[0] {
        Segment::Interpolation {
            transforms,
            reference,
            ..
        } => {
            assert_eq!(transforms.len(), 1);
            assert_eq!(transforms[0].name, "cap");
            assert_eq!(*reference, Reference::Identifier("name".into()));
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
    let t = parse_template("Возьмите {n} карт").unwrap();
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
    // {} should fail - no reference
    let result = parse_template("{}");
    assert!(result.is_err());
}

#[test]
fn test_unmatched_close_brace_in_text() {
    // A lone } in text should produce an error (it's not escaped)
    let result = parse_template("text } more");
    assert!(result.is_err());
}
