//! RLF file format parser.
//!
//! Parses `.rlf` files containing phrase definitions.

use super::ast::*;
use super::error::ParseError;
use crate::types::Tag;
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, terminated};
use winnow::prelude::*;
use winnow::token::{any, none_of, take_while};

/// Parse an entire .rlf file into phrase definitions.
pub fn parse_file(input: &str) -> Result<Vec<PhraseDefinition>, ParseError> {
    let mut remaining = input;
    match file(&mut remaining) {
        Ok(phrases) => {
            // Skip any trailing whitespace/comments
            let _ = skip_ws_and_comments(&mut remaining);
            if remaining.is_empty() {
                Ok(phrases)
            } else {
                let (line, column) = calculate_position(input, remaining);
                Err(ParseError::Syntax {
                    line,
                    column,
                    message: format!(
                        "unexpected character: '{}'",
                        remaining.chars().next().unwrap_or('?')
                    ),
                })
            }
        }
        Err(e) => {
            let (line, column) = calculate_position(input, remaining);
            Err(ParseError::Syntax {
                line,
                column,
                message: format!("parse error: {}", e),
            })
        }
    }
}

/// Calculate line and column from original input and remaining input.
fn calculate_position(original: &str, remaining: &str) -> (usize, usize) {
    let consumed = original.len() - remaining.len();
    let consumed_str = &original[..consumed];
    let line = consumed_str.chars().filter(|&c| c == '\n').count() + 1;
    let last_newline = consumed_str.rfind('\n');
    let column = match last_newline {
        Some(pos) => consumed - pos,
        None => consumed + 1,
    };
    (line, column)
}

/// Parse an entire file into phrase definitions.
fn file(input: &mut &str) -> ModalResult<Vec<PhraseDefinition>> {
    let _ = skip_ws_and_comments(input)?;
    let phrases: Vec<PhraseDefinition> =
        repeat(0.., terminated(phrase_definition, skip_ws_and_comments)).parse_next(input)?;
    Ok(phrases)
}

/// Skip whitespace and line comments.
fn skip_ws_and_comments(input: &mut &str) -> ModalResult<()> {
    let _: Vec<()> = repeat(0.., alt((ws_only.void(), line_comment.void()))).parse_next(input)?;
    Ok(())
}

/// Parse whitespace (no comments).
fn ws_only<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(input)
}

/// Parse a line comment: // ... newline
fn line_comment<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    preceded("//", take_while(0.., |c| c != '\n')).parse_next(input)
}

/// Parse a phrase definition: name(params)? = tags? from? body ;
fn phrase_definition(input: &mut &str) -> ModalResult<PhraseDefinition> {
    let name = snake_case_identifier(input)?;
    let _ = skip_ws_and_comments(input)?;

    // Optional parameter list
    let parameters: Vec<String> = opt(parameter_list).parse_next(input)?.unwrap_or_default();
    let _ = skip_ws_and_comments(input)?;

    // Equals sign
    let _ = '='.parse_next(input)?;
    let _ = skip_ws_and_comments(input)?;

    // Optional tags
    let tags: Vec<Tag> = repeat(0.., terminated(tag, skip_ws_and_comments)).parse_next(input)?;

    // Optional :from(param)
    let from_param: Option<String> = opt(terminated(from_modifier, skip_ws_and_comments))
        .parse_next(input)?;

    // Body (simple template or variant block)
    let body = phrase_body(input)?;
    let _ = skip_ws_and_comments(input)?;

    // Semicolon
    let _ = ';'.parse_next(input)?;

    Ok(PhraseDefinition {
        name,
        parameters,
        tags,
        from_param,
        body,
    })
}

/// Parse a snake_case identifier (lowercase start, alphanumeric + underscore).
fn snake_case_identifier(input: &mut &str) -> ModalResult<String> {
    let ident: &str =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;

    // Validate: must start with lowercase letter
    let first = ident.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    Ok(ident.to_string())
}

/// Parse a parameter list: (param1, param2, ...)
fn parameter_list(input: &mut &str) -> ModalResult<Vec<String>> {
    delimited(
        '(',
        separated(0.., preceded(skip_ws_and_comments, parameter_name), (skip_ws_and_comments, ',', skip_ws_and_comments)),
        preceded(skip_ws_and_comments, ')'),
    )
    .parse_next(input)
}

/// Parse a parameter name (simple identifier).
fn parameter_name(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

/// Parse a tag: :name
fn tag(input: &mut &str) -> ModalResult<Tag> {
    preceded(
        ':',
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_'),
    )
    .verify(|s: &&str| {
        // Make sure this is not a :from modifier
        *s != "from"
    })
    .map(|s: &str| Tag::new(s))
    .parse_next(input)
}

/// Parse a :from(param) modifier.
fn from_modifier(input: &mut &str) -> ModalResult<String> {
    preceded(":from", delimited('(', parameter_name, ')'))
        .parse_next(input)
}

/// Parse a phrase body: simple template or variant block.
fn phrase_body(input: &mut &str) -> ModalResult<PhraseBody> {
    alt((
        variant_block.map(PhraseBody::Variants),
        template_string.map(PhraseBody::Simple),
    ))
    .parse_next(input)
}

/// Parse a variant block: { key: "template", ... }
fn variant_block(input: &mut &str) -> ModalResult<Vec<VariantEntry>> {
    delimited(
        ('{', skip_ws_and_comments),
        variant_entries,
        (skip_ws_and_comments, '}'),
    )
    .parse_next(input)
}

/// Parse variant entries with trailing comma support.
fn variant_entries(input: &mut &str) -> ModalResult<Vec<VariantEntry>> {
    // Parse entries separated by commas, allow trailing comma
    let entries: Vec<VariantEntry> =
        separated(0.., variant_entry, (skip_ws_and_comments, ',', skip_ws_and_comments))
            .parse_next(input)?;

    // Allow trailing comma
    let _ = opt((skip_ws_and_comments, ',')).parse_next(input)?;

    Ok(entries)
}

/// Parse a single variant entry: key1, key2: "template"
fn variant_entry(input: &mut &str) -> ModalResult<VariantEntry> {
    let keys = variant_keys(input)?;
    let _ = skip_ws_and_comments(input)?;
    let _ = ':'.parse_next(input)?;
    let _ = skip_ws_and_comments(input)?;
    let template = template_string(input)?;

    Ok(VariantEntry { keys, template })
}

/// Parse variant keys: key1, key2, key3 (before the colon).
fn variant_keys(input: &mut &str) -> ModalResult<Vec<String>> {
    separated(1.., variant_key, (skip_ws_and_comments, ',', skip_ws_and_comments)).parse_next(input)
}

/// Parse a single variant key (may include dots for multi-dimensional: nom.one).
fn variant_key(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

/// Parse a template string: "content"
fn template_string(input: &mut &str) -> ModalResult<Template> {
    delimited('"', template_content, '"').parse_next(input)
}

/// Parse the content of a template string.
fn template_content(input: &mut &str) -> ModalResult<Template> {
    let segments: Vec<Segment> = repeat(0.., template_segment).parse_next(input)?;
    let merged = merge_literals(segments);
    Ok(Template { segments: merged })
}

/// Parse a single template segment.
fn template_segment(input: &mut &str) -> ModalResult<Segment> {
    alt((escape_sequence, interpolation, template_literal_char)).parse_next(input)
}

/// Parse escape sequences in templates: {{ }} @@ ::
fn escape_sequence(input: &mut &str) -> ModalResult<Segment> {
    alt((
        "{{".value(Segment::Literal("{".to_string())),
        "}}".value(Segment::Literal("}".to_string())),
        "@@".value(Segment::Literal("@".to_string())),
        "::".value(Segment::Literal(":".to_string())),
    ))
    .parse_next(input)
}

/// Parse a literal character in a template (not { } or ").
fn template_literal_char(input: &mut &str) -> ModalResult<Segment> {
    none_of(['{', '}', '"'])
        .map(|c: char| Segment::Literal(c.to_string()))
        .parse_next(input)
}

/// Parse an interpolation: { ... }
fn interpolation(input: &mut &str) -> ModalResult<Segment> {
    delimited('{', interpolation_content, '}').parse_next(input)
}

/// Parse interpolation content.
fn interpolation_content(input: &mut &str) -> ModalResult<Segment> {
    let _ = ws(input)?;
    let transforms: Vec<Transform> = repeat(0.., terminated(transform, ws)).parse_next(input)?;
    let reference = reference(input)?;
    let selectors: Vec<Selector> = repeat(0.., selector).parse_next(input)?;
    let _ = ws(input)?;

    Ok(Segment::Interpolation {
        transforms,
        reference,
        selectors,
    })
}

/// Parse whitespace within interpolations.
fn ws(input: &mut &str) -> ModalResult<()> {
    take_while(0.., |c: char| c.is_ascii_whitespace())
        .void()
        .parse_next(input)
}

/// Parse a transform: @name or @name:context
fn transform(input: &mut &str) -> ModalResult<Transform> {
    preceded(
        '@',
        (
            simple_identifier,
            opt(preceded(':', selector_identifier)),
        ),
    )
    .map(|(name, context)| Transform {
        name: name.to_string(),
        context: context.map(|s| Selector::Identifier(s.to_string())),
    })
    .parse_next(input)
}

/// Parse a reference in an interpolation.
fn reference(input: &mut &str) -> ModalResult<Reference> {
    let first_char = any.parse_next(input)?;

    if !is_ident_start(first_char) {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let rest: &str = take_while(0.., is_ident_cont).parse_next(input)?;
    let mut name = String::with_capacity(1 + rest.len());
    name.push(first_char);
    name.push_str(rest);

    // Check for auto-capitalization
    let is_auto_cap = first_char.is_ascii_uppercase();
    let actual_name = if is_auto_cap {
        let mut lowered = name.clone();
        let first = lowered.remove(0).to_ascii_lowercase();
        lowered.insert(0, first);
        lowered
    } else {
        name
    };

    // Check for phrase call: identifier(args)
    let args_opt: Option<Vec<Reference>> = opt(phrase_call_args).parse_next(input)?;

    let base_ref = match args_opt {
        Some(args) => Reference::PhraseCall {
            name: actual_name.clone(),
            args,
        },
        None => Reference::Identifier(actual_name.clone()),
    };

    // If auto-capitalization was triggered, we need to handle it at the AST level
    // The interpreter will handle @cap transformation based on uppercase reference
    if is_auto_cap {
        // For now, just return the reference with lowercase name
        // The auto-cap handling should be done via transforms in the AST
        Ok(base_ref)
    } else {
        Ok(base_ref)
    }
}

/// Parse phrase call arguments.
fn phrase_call_args(input: &mut &str) -> ModalResult<Vec<Reference>> {
    delimited(
        ('(', ws),
        separated(0.., reference_arg, (ws, ',', ws)),
        (ws, ')'),
    )
    .parse_next(input)
}

/// Parse a reference argument.
fn reference_arg(input: &mut &str) -> ModalResult<Reference> {
    simple_identifier
        .map(|name| Reference::Identifier(name.to_string()))
        .parse_next(input)
}

/// Parse a selector: :identifier
fn selector(input: &mut &str) -> ModalResult<Selector> {
    preceded(':', selector_identifier)
        .map(|s| Selector::Identifier(s.to_string()))
        .parse_next(input)
}

/// Parse a selector identifier.
fn selector_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Parse a simple identifier.
fn simple_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Check if a character can start an identifier.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Check if a character can continue an identifier.
fn is_ident_cont(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Merge adjacent literal segments.
fn merge_literals(segments: Vec<Segment>) -> Vec<Segment> {
    let mut result = Vec::with_capacity(segments.len());

    for segment in segments {
        match segment {
            Segment::Literal(text) => {
                if let Some(Segment::Literal(prev)) = result.last_mut() {
                    prev.push_str(&text);
                } else {
                    result.push(Segment::Literal(text));
                }
            }
            other => result.push(other),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
                assert_eq!(t.segments[0], Segment::Literal("Hello, world!".to_string()));
            }
            _ => panic!("expected simple body"),
        }
    }

    #[test]
    fn test_phrase_with_parameters() {
        let phrases = parse_file(r#"greet(name) = "Hello, {name}!";"#).unwrap();
        assert_eq!(phrases[0].parameters, vec!["name"]);
    }

    #[test]
    fn test_phrase_with_multiple_parameters() {
        let phrases =
            parse_file(r#"damage(amount, target) = "Deal {amount} to {target}.";"#).unwrap();
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
        assert_eq!(phrases[0].tags[0], Tag::new("a"));
        assert_eq!(phrases[0].tags[1], Tag::new("noun"));
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
                assert_eq!(entries[0].keys, vec!["one"]);
                assert_eq!(entries[1].keys, vec!["other"]);
            }
            _ => panic!("expected variants"),
        }
    }

    #[test]
    fn test_line_comments() {
        let phrases = parse_file(
            r#"
            // This is a comment
            hello = "Hello!";
            // Another comment
            bye = "Goodbye!";
        "#,
        )
        .unwrap();
        assert_eq!(phrases.len(), 2);
        assert_eq!(phrases[0].name, "hello");
        assert_eq!(phrases[1].name, "bye");
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
                assert_eq!(entries[1].keys, vec!["nom.other"]);
                assert_eq!(entries[2].keys, vec!["acc.one"]);
                assert_eq!(entries[3].keys, vec!["acc.other"]);
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
                assert_eq!(entries[1].keys, vec!["nom.other", "acc.other"]);
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
        match &phrases[0].body {
            PhraseBody::Variants(entries) => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("expected variants"),
        }
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
                assert!(entries.iter().any(|e| e.keys == vec!["nom"]));
                assert!(entries.iter().any(|e| e.keys == vec!["nom.other"]));
            }
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
            draw(n) = "Draw {n} {card:n}.";
            subtype(s) = :from(s) "<b>{s}</b>";
        "#,
        )
        .unwrap();
        assert_eq!(phrases.len(), 4);
        assert_eq!(phrases[0].name, "card");
        assert_eq!(phrases[1].name, "event");
        assert_eq!(phrases[2].name, "draw");
        assert_eq!(phrases[3].name, "subtype");
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
    fn test_inline_comment() {
        let phrases = parse_file(
            r#"
            hello = "Hello!"; // inline comment
            bye = "Goodbye!";
        "#,
        )
        .unwrap();
        assert_eq!(phrases.len(), 2);
    }
}
