//! Public AST types for RLF templates and phrase definitions.
//!
//! These types are public to enable external tooling (linters, formatters, etc.).

use crate::types::Tag;

/// A parsed template string containing segments.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    pub segments: Vec<Segment>,
}

/// A segment within a template.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// Literal text (no interpolation).
    Literal(String),
    /// An interpolation: {transforms reference selectors}
    Interpolation {
        transforms: Vec<Transform>,
        reference: Reference,
        selectors: Vec<Selector>,
    },
}

/// A transform applied to a reference (e.g., @cap, @a, @der:acc, @count($n)).
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    /// Transform name without @ (e.g., "cap", "a", "der")
    pub name: String,
    /// Optional context for the transform.
    pub context: TransformContext,
}

/// Context for a transform: static (`:literal`), dynamic (`($param)`), or both.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformContext {
    /// No context.
    None,
    /// Static context literal (e.g., "acc" in `@der:acc`).
    Static(String),
    /// Dynamic context parameter name (e.g., "n" in `@count($n)`).
    /// The String stores the name without the `$` prefix.
    Dynamic(String),
    /// Both static and dynamic context (e.g., `@transform:lit($param)`).
    Both(String, String),
}

/// A reference to a parameter, term, or phrase.
#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    /// Reference to a term or phrase by bare name: {card}
    Identifier(String),
    /// Reference to a parameter with $ prefix: {$name}
    /// The String stores the name without the `$` prefix.
    Parameter(String),
    /// Reference to a phrase call with arguments: {phrase($arg1, $arg2)}
    PhraseCall { name: String, args: Vec<Reference> },
    /// Literal integer argument in a phrase call: {cards(2)}
    NumberLiteral(i64),
    /// Literal string argument in a phrase call: {trigger("Attack")}
    StringLiteral(String),
}

/// A selector for variant selection.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// Static selector identifier: :one, :other, :nom
    Identifier(String),
    /// Parameterized selector with $ prefix: :$n, :$entity
    /// The String stores the name without the `$` prefix.
    Parameter(String),
}

// =============================================================================
// File-level AST types (phrase definitions)
// =============================================================================

/// A parsed phrase definition from a .rlf file.
#[derive(Debug, Clone, PartialEq)]
pub struct PhraseDefinition {
    /// Phrase name (snake_case identifier).
    pub name: String,
    /// Parameter names (empty if no parameters).
    pub parameters: Vec<String>,
    /// Metadata tags (e.g., :fem, :masc, :a).
    pub tags: Vec<Tag>,
    /// :from(param) inheritance (None if not present).
    pub from_param: Option<String>,
    /// Phrase body (simple template or variants).
    pub body: PhraseBody,
}

/// The body of a phrase definition.
#[derive(Debug, Clone, PartialEq)]
pub enum PhraseBody {
    /// Simple phrase: name = "text";
    Simple(Template),
    /// Variant phrase: name = { one: "x", other: "y" };
    Variants(Vec<VariantEntry>),
}

/// A single variant entry in a variant block.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantEntry {
    /// Variant keys (multiple keys share the same template).
    pub keys: Vec<String>,
    /// Template for this variant.
    pub template: Template,
}
