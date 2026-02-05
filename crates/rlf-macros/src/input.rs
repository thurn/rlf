//! Internal AST types for the rlf! macro.
//!
//! These types mirror the parser AST but include span information for error messages.

use proc_macro2::Span;
use syn::Ident;

/// Top-level macro input containing all phrase definitions.
pub struct MacroInput {
    pub phrases: Vec<PhraseDefinition>,
}

/// A single phrase definition: `name(params) = body;`
pub struct PhraseDefinition {
    pub name: SpannedIdent,
    pub parameters: Vec<SpannedIdent>,
    pub tags: Vec<SpannedIdent>,
    pub from_param: Option<SpannedIdent>,
    pub body: PhraseBody,
}

/// Wrapper for identifiers that preserves span information.
#[derive(Clone)]
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
pub enum PhraseBody {
    Simple(Template),
    Variants(Vec<VariantEntry>),
}

/// A variant entry: `key: "template"`
pub struct VariantEntry {
    /// Variant keys (multiple keys share the same template).
    pub keys: Vec<SpannedIdent>,
    pub template: Template,
}

/// A template string with interpolations.
pub struct Template {
    pub segments: Vec<Segment>,
    pub span: Span,
}

/// A segment of a template: literal text or interpolation.
pub enum Segment {
    Literal(String),
    Interpolation(Interpolation),
}

/// An interpolation: `{@transform name:selector}`
pub struct Interpolation {
    pub transforms: Vec<TransformRef>,
    pub reference: Reference,
    pub selectors: Vec<Selector>,
    pub span: Span,
}

/// A reference to a transform with optional context.
pub struct TransformRef {
    pub name: SpannedIdent,
    pub context: Option<Selector>,
}

/// A reference to a parameter or phrase.
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
pub struct Selector {
    pub name: SpannedIdent,
}
