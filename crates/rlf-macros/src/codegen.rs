//! Code generation for the rlf! macro.
//!
//! Transforms validated MacroInput into Rust code that provides:
//! - Typed functions for each phrase
//! - SOURCE_PHRASES const with embedded phrase definitions
//! - register_source_phrases() function for loading
//! - phrase_ids module with PhraseId constants

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::input::{
    Interpolation, MacroInput, PhraseBody, PhraseDefinition, Reference, Segment, Template,
    TransformRef,
};

/// Main code generation entry point.
///
/// Produces TokenStream containing:
/// - Phrase functions with typed parameters
/// - SOURCE_PHRASES const
/// - register_source_phrases() function
/// - phrase_ids module
pub fn codegen(input: &MacroInput) -> TokenStream {
    let functions = generate_functions(input);
    let source_phrases = generate_source_phrases(input);
    let phrase_ids_module = generate_phrase_ids_module(input);

    quote! {
        #functions
        #source_phrases
        #phrase_ids_module
    }
}

// =============================================================================
// Phrase Function Generation
// =============================================================================

/// Generate all phrase functions.
fn generate_functions(input: &MacroInput) -> TokenStream {
    let functions: Vec<TokenStream> = input.phrases.iter().map(generate_function).collect();

    quote! {
        #(#functions)*
    }
}

/// Generate a single phrase function.
///
/// Parameterless phrases: `pub fn name(locale: &::rlf::Locale) -> ::rlf::Phrase`
/// With parameters: `pub fn name(locale: &::rlf::Locale, p1: impl Into<::rlf::Value>, ...) -> ::rlf::Phrase`
fn generate_function(phrase: &PhraseDefinition) -> TokenStream {
    let fn_name = format_ident!("{}", phrase.name.name);
    let phrase_name = &phrase.name.name;
    let doc = format!("Returns the \"{}\" phrase.", phrase_name);

    if phrase.parameters.is_empty() {
        // Parameterless phrase
        quote! {
            #[doc = #doc]
            pub fn #fn_name(locale: &::rlf::Locale) -> ::rlf::Phrase {
                locale.get_phrase(#phrase_name)
                    .expect(concat!("phrase '", #phrase_name, "' should exist"))
            }
        }
    } else {
        // Phrase with parameters
        let param_names: Vec<_> = phrase
            .parameters
            .iter()
            .map(|p| format_ident!("{}", p.name))
            .collect();

        let param_decls: Vec<TokenStream> = param_names
            .iter()
            .map(|name| quote! { #name: impl Into<::rlf::Value> })
            .collect();

        let param_conversions: Vec<TokenStream> = param_names
            .iter()
            .map(|name| quote! { #name.into() })
            .collect();

        quote! {
            #[doc = #doc]
            pub fn #fn_name(locale: &::rlf::Locale, #(#param_decls),*) -> ::rlf::Phrase {
                locale.call_phrase(#phrase_name, &[#(#param_conversions),*])
                    .expect(concat!("phrase '", #phrase_name, "' should exist"))
            }
        }
    }
}

// =============================================================================
// SOURCE_PHRASES and register_source_phrases Generation
// =============================================================================

/// Generate SOURCE_PHRASES const and register_source_phrases function.
fn generate_source_phrases(input: &MacroInput) -> TokenStream {
    // Reconstruct phrase definitions as RLF source text
    let source = reconstruct_source(input);

    quote! {
        /// Source language phrases embedded as data.
        /// Parsed by the interpreter at runtime.
        const SOURCE_PHRASES: &str = #source;

        /// Registers source language phrases with the locale.
        /// Call once at startup before using phrase functions.
        ///
        /// # Example
        ///
        /// ```ignore
        /// let mut locale = Locale::new();
        /// register_source_phrases(&mut locale);
        /// ```
        pub fn register_source_phrases(locale: &mut ::rlf::Locale) {
            locale.load_translations_str("en", SOURCE_PHRASES)
                .expect("source phrases should parse successfully");
        }
    }
}

/// Reconstruct RLF source from MacroInput.
///
/// This recreates the phrase definitions in RLF syntax for the interpreter.
fn reconstruct_source(input: &MacroInput) -> String {
    let mut lines = Vec::new();

    for phrase in &input.phrases {
        let mut line = String::new();

        // Tags
        for tag in &phrase.tags {
            line.push(':');
            line.push_str(&tag.name);
            line.push(' ');
        }

        // Name
        line.push_str(&phrase.name.name);

        // Parameters
        if !phrase.parameters.is_empty() {
            line.push('(');
            let params: Vec<_> = phrase.parameters.iter().map(|p| p.name.as_str()).collect();
            line.push_str(&params.join(", "));
            line.push(')');
        }

        line.push_str(" = ");

        // Body
        match &phrase.body {
            PhraseBody::Simple(template) => {
                line.push('"');
                line.push_str(&reconstruct_template(template));
                line.push('"');
            }
            PhraseBody::Variants(variants) => {
                line.push_str("{ ");
                let variant_strs: Vec<String> = variants
                    .iter()
                    .map(|v| {
                        let keys: Vec<_> = v.keys.iter().map(|k| k.name.as_str()).collect();
                        format!(
                            "{}: \"{}\"",
                            keys.join(", "),
                            reconstruct_template(&v.template)
                        )
                    })
                    .collect();
                line.push_str(&variant_strs.join(", "));
                line.push_str(" }");
            }
        }

        line.push(';');
        lines.push(line);
    }

    lines.join("\n")
}

/// Reconstruct a template string from Template AST.
fn reconstruct_template(template: &Template) -> String {
    let mut result = String::new();

    for segment in &template.segments {
        match segment {
            Segment::Literal(text) => {
                // Escape special characters for RLF template strings
                // Braces need to be doubled, quotes need backslash escape
                let escaped = text
                    .replace('{', "{{")
                    .replace('}', "}}")
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                result.push_str(&escaped);
            }
            Segment::Interpolation(interp) => {
                result.push_str(&reconstruct_interpolation(interp));
            }
        }
    }

    result
}

/// Reconstruct an interpolation from Interpolation AST.
fn reconstruct_interpolation(interp: &Interpolation) -> String {
    let mut result = String::from("{");

    // Transforms
    for transform in &interp.transforms {
        result.push_str(&reconstruct_transform(transform));
    }

    // Reference
    result.push_str(&reconstruct_reference(&interp.reference));

    // Selectors
    for selector in &interp.selectors {
        result.push(':');
        result.push_str(&selector.name.name);
    }

    result.push('}');
    result
}

/// Reconstruct a transform reference.
fn reconstruct_transform(transform: &TransformRef) -> String {
    let mut result = String::from("@");
    result.push_str(&transform.name.name);

    // Handle context if present (e.g., @a:context)
    if let Some(ref ctx) = transform.context {
        result.push(':');
        result.push_str(&ctx.name.name);
    }

    result.push(' ');
    result
}

/// Reconstruct a reference (identifier or call).
fn reconstruct_reference(reference: &Reference) -> String {
    match reference {
        Reference::Identifier(ident) => ident.name.clone(),
        Reference::Call { name, args } => {
            let arg_strs: Vec<String> = args.iter().map(reconstruct_reference).collect();
            format!("{}({})", name.name, arg_strs.join(", "))
        }
    }
}

// =============================================================================
// phrase_ids Module Generation
// =============================================================================

/// Generate the phrase_ids module with PhraseId constants.
fn generate_phrase_ids_module(input: &MacroInput) -> TokenStream {
    let constants: Vec<TokenStream> = input
        .phrases
        .iter()
        .map(generate_phrase_id_constant)
        .collect();

    quote! {
        /// PhraseId constants for all defined phrases.
        ///
        /// Use these for serializable phrase references:
        /// ```ignore
        /// use crate::strings::phrase_ids;
        ///
        /// let id = phrase_ids::CARD;
        /// let phrase = id.resolve(&locale)?;
        /// ```
        pub mod phrase_ids {
            #(#constants)*
        }
    }
}

/// Generate a single PhraseId constant.
fn generate_phrase_id_constant(phrase: &PhraseDefinition) -> TokenStream {
    let phrase_name = &phrase.name.name;

    // Convert to SCREAMING_CASE
    let const_name = format_ident!("{}", to_screaming_case(phrase_name));

    // Doc comment with parameter info if applicable
    let doc = if phrase.parameters.is_empty() {
        format!("ID for the \"{}\" phrase.", phrase_name)
    } else {
        let param_names: Vec<_> = phrase.parameters.iter().map(|p| p.name.as_str()).collect();
        format!(
            "ID for the \"{}\" phrase. Call with {} argument(s) ({}).",
            phrase_name,
            phrase.parameters.len(),
            param_names.join(", ")
        )
    };

    quote! {
        #[doc = #doc]
        pub const #const_name: ::rlf::PhraseId = ::rlf::PhraseId::from_name(#phrase_name);
    }
}

/// Convert snake_case to SCREAMING_CASE.
///
/// Examples:
/// - "card" -> "CARD"
/// - "fire_elemental" -> "FIRE_ELEMENTAL"
/// - "drawCards" -> "DRAWCARDS"
fn to_screaming_case(s: &str) -> String {
    s.to_uppercase()
}
