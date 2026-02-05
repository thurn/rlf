//! Internal AST types for the rlf! macro.
//!
//! These types mirror the parser AST but include span information for error messages.
//!
//! Note: Fields are currently unused but will be consumed in Plan 02 (validation)
//! and Plan 03 (code generation).

#![allow(dead_code)]

use proc_macro2::Span;
use syn::Ident;

/// Top-level macro input containing all phrase definitions.
#[derive(Debug)]
pub struct MacroInput {
    pub phrases: Vec<PhraseDefinition>,
}

/// A single phrase definition: `name(params) = body;`
#[derive(Debug)]
pub struct PhraseDefinition {
    pub name: SpannedIdent,
    pub parameters: Vec<SpannedIdent>,
    pub tags: Vec<SpannedIdent>,
    pub from_param: Option<SpannedIdent>,
    pub body: PhraseBody,
}

/// Wrapper for identifiers that preserves span information.
#[derive(Clone, Debug)]
pub struct SpannedIdent {
    pub name: String,
    pub span: Span,
}

impl SpannedIdent {
    pub fn new(ident: &Ident) -> Self {
        Self {
            name: ident.to_string(),
            span: ident.span(),
        }
    }

    /// Create a SpannedIdent from a string and span.
    pub fn from_str(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

/// Phrase body: either simple template or variant map.
#[derive(Debug)]
pub enum PhraseBody {
    Simple(Template),
    Variants(Vec<VariantEntry>),
}

/// A variant entry: `key: "template"`
#[derive(Debug)]
pub struct VariantEntry {
    /// Variant keys (multiple keys share the same template).
    pub keys: Vec<SpannedIdent>,
    pub template: Template,
}

/// A template string with interpolations.
#[derive(Debug)]
pub struct Template {
    pub segments: Vec<Segment>,
    pub span: Span,
}

/// A segment of a template: literal text or interpolation.
#[derive(Debug)]
pub enum Segment {
    Literal(String),
    Interpolation(Interpolation),
}

/// An interpolation: `{@transform name:selector}`
#[derive(Debug)]
pub struct Interpolation {
    pub transforms: Vec<TransformRef>,
    pub reference: Reference,
    pub selectors: Vec<Selector>,
    pub span: Span,
}

/// A reference to a transform with optional context.
#[derive(Debug)]
pub struct TransformRef {
    pub name: SpannedIdent,
    pub context: Option<Selector>,
}

/// A reference to a parameter or phrase.
#[derive(Debug)]
pub enum Reference {
    /// Simple identifier (resolved later as parameter or phrase).
    Identifier(SpannedIdent),
    /// Phrase call with arguments: `foo(x, y)`.
    Call {
        name: SpannedIdent,
        args: Vec<Reference>,
    },
}

/// A selector: `:name` (literal) or `:n` (parameter-based).
#[derive(Debug)]
pub struct Selector {
    pub name: SpannedIdent,
}
