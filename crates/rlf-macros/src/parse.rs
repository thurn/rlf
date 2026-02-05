//! Parse implementations for converting TokenStream to macro AST.
//!
//! Implements syn::parse::Parse for all AST types defined in input.rs.

use crate::input::{
    Interpolation, MacroInput, PhraseBody, PhraseDefinition, Reference, Segment, Selector,
    SpannedIdent, Template, TransformRef, VariantEntry,
};
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, LitStr, Token};

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut phrases = Vec::new();
        while !input.is_empty() {
            phrases.push(input.parse()?);
        }
        Ok(MacroInput { phrases })
    }
}

impl Parse for PhraseDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse optional tags (: followed by ident, but not :from)
        let mut tags = Vec::new();
        while input.peek(Token![:]) && !input.peek2(syn::token::Brace) {
            // Lookahead to check if it's :from
            let colon_span = input.parse::<Token![:]>()?.span;
            let ident: Ident = input.parse()?;

            if ident == "from" {
                // This is :from(param), handle it separately
                let content;
                syn::parenthesized!(content in input);
                let param: Ident = content.parse()?;
                // We need to continue parsing tags after :from
                // Store from_param and continue
                let from_param = Some(SpannedIdent::new(&param));

                // Continue parsing remaining tags
                while input.peek(Token![:]) && !input.peek2(syn::token::Brace) {
                    input.parse::<Token![:]>()?;
                    let tag_ident: Ident = input.parse()?;
                    tags.push(SpannedIdent::new(&tag_ident));
                }

                // Parse phrase name
                let name_ident: Ident = input.parse()?;
                let name = SpannedIdent::new(&name_ident);

                // Parse optional parameters
                let parameters = if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let params: Punctuated<Ident, Token![,]> =
                        Punctuated::parse_terminated(&content)?;
                    params.iter().map(SpannedIdent::new).collect()
                } else {
                    Vec::new()
                };

                // Parse =
                input.parse::<Token![=]>()?;

                // Parse body
                let body = input.parse()?;

                // Parse ;
                input.parse::<Token![;]>()?;

                return Ok(PhraseDefinition {
                    name,
                    parameters,
                    tags,
                    from_param,
                    body,
                });
            } else {
                // Regular tag
                tags.push(SpannedIdent::from_str(ident.to_string(), colon_span));
            }
        }

        // Parse phrase name
        let name_ident: Ident = input.parse()?;
        let name = SpannedIdent::new(&name_ident);

        // Parse optional parameters
        let parameters = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let params: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;
            params.iter().map(SpannedIdent::new).collect()
        } else {
            Vec::new()
        };

        // Parse =
        input.parse::<Token![=]>()?;

        // Parse body
        let body = input.parse()?;

        // Parse ;
        input.parse::<Token![;]>()?;

        Ok(PhraseDefinition {
            name,
            parameters,
            tags,
            from_param: None,
            body,
        })
    }
}

impl Parse for PhraseBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If starts with { it's a variant block, otherwise simple template
        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);

            let mut entries = Vec::new();
            while !content.is_empty() {
                entries.push(content.parse()?);
            }

            Ok(PhraseBody::Variants(entries))
        } else {
            let template: Template = input.parse()?;
            Ok(PhraseBody::Simple(template))
        }
    }
}

impl Parse for VariantEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse keys (comma-separated identifiers, possibly with dots)
        // Format: key1, key2: "template"
        // Or: nom.one, nom.few: "template"
        let mut keys = Vec::new();

        loop {
            // Parse a key which may be dotted (e.g., nom.one)
            let first: Ident = input.parse()?;
            let mut key_str = first.to_string();
            let key_span = first.span();

            // Check for dot-separated parts
            while input.peek(Token![.]) {
                input.parse::<Token![.]>()?;
                let part: Ident = input.parse()?;
                key_str.push('.');
                key_str.push_str(&part.to_string());
            }

            keys.push(SpannedIdent::from_str(key_str, key_span));

            // Check for comma (more keys) or colon (end of keys)
            if input.peek(Token![,]) && !input.peek2(Token![:]) && !input.peek2(LitStr) {
                // More keys coming, but need to be careful about trailing comma before string
                let fork = input.fork();
                fork.parse::<Token![,]>().ok();
                if fork.peek(Ident) {
                    input.parse::<Token![,]>()?;
                    continue;
                }
            }
            break;
        }

        // Parse colon
        input.parse::<Token![:]>()?;

        // Parse template string
        let template: Template = input.parse()?;

        // Parse optional trailing comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(VariantEntry { keys, template })
    }
}

impl Parse for Template {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;
        let span = lit.span();
        let value = lit.value();

        let segments = parse_template_string(&value, span)?;

        Ok(Template { segments, span })
    }
}

/// Parse a template string into segments.
///
/// Handles:
/// - Literal text
/// - Escaped braces: {{ and }}
/// - Interpolations: {reference}, {@transform reference}, {reference:selector}
pub(crate) fn parse_template_string(s: &str, span: Span) -> syn::Result<Vec<Segment>> {
    let mut segments = Vec::new();
    let mut current_literal = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                if chars.peek() == Some(&'{') {
                    // Escaped brace
                    chars.next();
                    current_literal.push('{');
                } else {
                    // Start of interpolation
                    if !current_literal.is_empty() {
                        segments.push(Segment::Literal(std::mem::take(&mut current_literal)));
                    }

                    // Collect until closing brace
                    let mut interp_content = String::new();
                    let mut depth = 1;
                    // Using while-let instead of for because we break out early
                    #[allow(clippy::while_let_on_iterator)]
                    while let Some(c) = chars.next() {
                        if c == '{' {
                            depth += 1;
                            interp_content.push(c);
                        } else if c == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            interp_content.push(c);
                        } else {
                            interp_content.push(c);
                        }
                    }

                    if depth != 0 {
                        return Err(syn::Error::new(span, "unclosed interpolation brace"));
                    }

                    if interp_content.is_empty() {
                        return Err(syn::Error::new(span, "empty interpolation"));
                    }

                    let interpolation = parse_interpolation(&interp_content, span)?;
                    segments.push(Segment::Interpolation(interpolation));
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    // Escaped brace
                    chars.next();
                    current_literal.push('}');
                } else {
                    return Err(syn::Error::new(span, "unexpected closing brace"));
                }
            }
            _ => {
                current_literal.push(c);
            }
        }
    }

    if !current_literal.is_empty() {
        segments.push(Segment::Literal(current_literal));
    }

    Ok(segments)
}

/// Parse the content of an interpolation: @transforms reference :selectors
fn parse_interpolation(content: &str, span: Span) -> syn::Result<Interpolation> {
    let content = content.trim();
    let mut transforms = Vec::new();
    let mut rest = content;

    // Parse transforms (start with @)
    while rest.starts_with('@') {
        rest = &rest[1..]; // Skip @

        // Find end of transform name (space, colon, or end of string)
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ':')
            .unwrap_or(rest.len());

        if end == 0 {
            return Err(syn::Error::new(span, "empty transform name after @"));
        }

        let transform_name = &rest[..end];
        rest = rest[end..].trim_start();

        // Check for transform context (e.g., @der:acc)
        let context = if rest.starts_with(':') && !rest[1..].starts_with(char::is_whitespace) {
            // This could be a transform context or the start of selectors
            // Transform context is immediately after the transform name with no space
            // Look for the next space or end to determine
            let after_colon = &rest[1..];
            let ctx_end = after_colon
                .find(|c: char| c.is_whitespace() || c == ':')
                .unwrap_or(after_colon.len());

            if ctx_end > 0 {
                // Check if this looks like a selector (comes after the reference)
                // or a context (comes immediately after transform)
                // Heuristic: if there's no reference yet and we're still parsing transforms,
                // this is a context
                let ctx_name = &after_colon[..ctx_end];
                rest = after_colon[ctx_end..].trim_start();
                Some(Selector {
                    name: SpannedIdent::from_str(ctx_name, span),
                })
            } else {
                None
            }
        } else {
            None
        };

        transforms.push(TransformRef {
            name: SpannedIdent::from_str(transform_name, span),
            context,
        });
    }

    // Parse reference (identifier or call)
    let reference = parse_reference(rest, span)?;

    // Extract selectors from the rest (they come after the reference)
    let selectors = extract_selectors(&reference.1, span)?;

    Ok(Interpolation {
        transforms,
        reference: reference.0,
        selectors,
        span,
    })
}

/// Parse a reference from a string, returning the Reference and any remaining content.
fn parse_reference(content: &str, span: Span) -> syn::Result<(Reference, String)> {
    let content = content.trim();

    if content.is_empty() {
        return Err(syn::Error::new(span, "empty reference in interpolation"));
    }

    // Find the identifier
    let ident_end = content
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(content.len());

    if ident_end == 0 {
        return Err(syn::Error::new(
            span,
            format!("invalid reference: {}", content),
        ));
    }

    let ident = &content[..ident_end];
    let rest = &content[ident_end..];

    // Check for call syntax: name(args)
    if rest.starts_with('(') {
        // Find matching closing paren
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in rest.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            return Err(syn::Error::new(span, "unclosed parenthesis in phrase call"));
        }

        let args_str = &rest[1..end]; // Content between parens
        let after_call = &rest[end + 1..];

        // Parse arguments (comma-separated references)
        let mut args = Vec::new();
        if !args_str.is_empty() {
            for arg in args_str.split(',') {
                let (arg_ref, _) = parse_reference(arg.trim(), span)?;
                args.push(arg_ref);
            }
        }

        Ok((
            Reference::Call {
                name: SpannedIdent::from_str(ident, span),
                args,
            },
            after_call.to_string(),
        ))
    } else {
        Ok((
            Reference::Identifier(SpannedIdent::from_str(ident, span)),
            rest.to_string(),
        ))
    }
}

/// Extract selectors from the remaining content after a reference.
fn extract_selectors(content: &str, span: Span) -> syn::Result<Vec<Selector>> {
    let mut selectors = Vec::new();
    let mut rest = content.trim();

    while rest.starts_with(':') {
        rest = &rest[1..]; // Skip :

        // Find end of selector name
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ':')
            .unwrap_or(rest.len());

        if end == 0 {
            return Err(syn::Error::new(span, "empty selector after :"));
        }

        let selector_name = &rest[..end];
        selectors.push(Selector {
            name: SpannedIdent::from_str(selector_name, span),
        });

        rest = rest[end..].trim_start();
    }

    if !rest.is_empty() {
        return Err(syn::Error::new(
            span,
            format!("unexpected content after selectors: {}", rest),
        ));
    }

    Ok(selectors)
}
