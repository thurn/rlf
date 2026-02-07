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

/// Whether a definition is a term (no parameters) or a phrase (with parameters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    /// A term has no parameters and can have variant blocks.
    Term,
    /// A phrase has one or more parameters and uses a simple template body.
    Phrase,
}

/// A single phrase definition: `name(params) = body;`
#[derive(Debug)]
pub struct PhraseDefinition {
    /// Whether this definition is a term or a phrase.
    pub kind: DefinitionKind,
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
    /// Whether this entry is marked as the default with `*`.
    pub is_default: bool,
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
    /// Transform context: static (`:literal`), dynamic (`($param)`), or both.
    pub context: TransformContext,
}

/// Context for a transform: static (`:literal`), dynamic (`($param)`), or both.
#[derive(Debug)]
pub enum TransformContext {
    /// No context.
    None,
    /// Static context literal (e.g., "acc" in `@der:acc`).
    Static(SpannedIdent),
    /// Dynamic context parameter name (e.g., "n" in `@count($n)`).
    /// The SpannedIdent stores the name without the `$` prefix.
    Dynamic(SpannedIdent),
    /// Both static and dynamic context (e.g., `@transform:lit($param)`).
    Both(SpannedIdent, SpannedIdent),
}

/// A reference to a parameter, term, or phrase call.
#[derive(Debug)]
pub enum Reference {
    /// Reference to a term by bare name: `{card}`
    Identifier(SpannedIdent),
    /// Reference to a parameter with `$` prefix: `{$name}`
    /// The SpannedIdent stores the name without the `$` prefix.
    Parameter(SpannedIdent),
    /// Phrase call with arguments: `foo($x, term_name)`.
    Call {
        name: SpannedIdent,
        args: Vec<Reference>,
    },
    /// Literal integer argument in a phrase call: `cards(2)`
    NumberLiteral(i64, Span),
    /// Literal string argument in a phrase call: `trigger("Attack")`
    StringLiteral(String, Span),
}

/// A selector for variant selection.
#[derive(Debug)]
pub enum Selector {
    /// Static selector identifier: `:one`, `:other`, `:nom`
    Literal(SpannedIdent),
    /// Parameterized selector with `$` prefix: `:$n`, `:$entity`
    /// The SpannedIdent stores the name without the `$` prefix.
    Parameter(SpannedIdent),
}
