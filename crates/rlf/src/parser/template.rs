//! Template string parser using winnow.
//!
//! Parses RLF template strings into an AST. Handles:
//! - Literal text segments
//! - Interpolations with transforms, references, and selectors
//! - Escape sequences: {{ }} @@ ::
//! - Automatic capitalization (uppercase first letter -> @cap transform)
//! - Phrase calls with arguments

use super::ast::*;
use super::error::ParseError;
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, terminated};
use winnow::prelude::*;
use winnow::token::{any, none_of, take_while};

/// Parse a template string into an AST.
pub fn parse_template(input: &str) -> Result<Template, ParseError> {
    let mut remaining = input;
    match template(&mut remaining) {
        Ok(t) => {
            if remaining.is_empty() {
                Ok(t)
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

/// Parse a complete template into segments.
fn template(input: &mut &str) -> ModalResult<Template> {
    let segments: Vec<Segment> = repeat(0.., segment).parse_next(input)?;

    // Merge adjacent literals
    let merged = merge_literals(segments);

    Ok(Template { segments: merged })
}

/// Merge adjacent Literal segments into single segments.
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

/// Parse a single segment (escape, interpolation, or literal).
fn segment(input: &mut &str) -> ModalResult<Segment> {
    alt((escape_sequence, interpolation, literal_char)).parse_next(input)
}

/// Parse escape sequences: {{ -> {, }} -> }, @@ -> @, :: -> :
fn escape_sequence(input: &mut &str) -> ModalResult<Segment> {
    alt((
        "{{".value(Segment::Literal("{".to_string())),
        "}}".value(Segment::Literal("}".to_string())),
        "@@".value(Segment::Literal("@".to_string())),
        "::".value(Segment::Literal(":".to_string())),
    ))
    .parse_next(input)
}

/// Parse a single literal character (not { or }).
fn literal_char(input: &mut &str) -> ModalResult<Segment> {
    none_of(['{', '}'])
        .map(|c: char| Segment::Literal(c.to_string()))
        .parse_next(input)
}

/// Parse an interpolation: { transforms* reference selectors* }
fn interpolation(input: &mut &str) -> ModalResult<Segment> {
    delimited('{', interpolation_content, '}').parse_next(input)
}

/// A parsed reference with an auto-capitalization flag.
struct ParsedReference {
    reference: Reference,
    auto_cap: bool,
}

/// Parse the content inside an interpolation.
fn interpolation_content(input: &mut &str) -> ModalResult<Segment> {
    let _ = ws(input)?;
    let mut transforms: Vec<Transform> = repeat(0.., terminated(transform, ws)).parse_next(input)?;
    let parsed_ref = reference(input)?;
    let selectors: Vec<Selector> = repeat(0.., selector).parse_next(input)?;
    let _ = ws(input)?;

    // If auto-capitalization was triggered, prepend @cap transform
    if parsed_ref.auto_cap {
        transforms.insert(
            0,
            Transform {
                name: "cap".to_string(),
                context: None,
            },
        );
    }

    Ok(Segment::Interpolation {
        transforms,
        reference: parsed_ref.reference,
        selectors,
    })
}

/// Parse optional whitespace.
fn ws(input: &mut &str) -> ModalResult<()> {
    take_while(0.., |c: char| c.is_ascii_whitespace())
        .void()
        .parse_next(input)
}

/// Parse a transform: @name or @name:context
fn transform(input: &mut &str) -> ModalResult<Transform> {
    preceded('@', (identifier, opt(preceded(':', selector_identifier))))
        .map(|(name, context)| Transform {
            name: name.to_string(),
            context: context.map(|s| Selector::Identifier(s.to_string())),
        })
        .parse_next(input)
}

/// Parse a reference: identifier or identifier(args)
/// Returns the reference and a flag indicating if auto-capitalization should be applied.
fn reference(input: &mut &str) -> ModalResult<ParsedReference> {
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

    // Check for auto-capitalization: if first letter is uppercase, add @cap
    let auto_cap = first_char.is_ascii_uppercase();
    let actual_name = if auto_cap {
        // Only lowercase the first character
        let mut chars = name.chars();
        let first = chars.next().unwrap().to_ascii_lowercase();
        let mut lowered = String::with_capacity(name.len());
        lowered.push(first);
        lowered.extend(chars);
        lowered
    } else {
        name
    };

    // Check for phrase call: identifier(args)
    let args_opt: Option<Vec<Reference>> = opt(phrase_call_args).parse_next(input)?;

    let reference = match args_opt {
        Some(args) => Reference::PhraseCall {
            name: actual_name,
            args,
        },
        None => Reference::Identifier(actual_name),
    };

    Ok(ParsedReference {
        reference,
        auto_cap,
    })
}

/// Parse phrase call arguments: (arg1, arg2, ...)
fn phrase_call_args(input: &mut &str) -> ModalResult<Vec<Reference>> {
    delimited(
        ('(', ws),
        separated(0.., reference_arg, (ws, ',', ws)),
        (ws, ')'),
    )
    .parse_next(input)
}

/// Parse a reference argument (simpler than main reference - no auto-cap).
fn reference_arg(input: &mut &str) -> ModalResult<Reference> {
    identifier
        .map(|name| Reference::Identifier(name.to_string()))
        .parse_next(input)
}

/// Parse a selector: :identifier
fn selector(input: &mut &str) -> ModalResult<Selector> {
    preceded(':', selector_identifier)
        .map(|s| Selector::Identifier(s.to_string()))
        .parse_next(input)
}

/// Parse a selector identifier (alphanumeric with underscores).
fn selector_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Parse an identifier.
fn identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
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

