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

/// A transform applied to a reference (e.g., @cap, @a, @der:acc).
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    /// Transform name without @ (e.g., "cap", "a", "der")
    pub name: String,
    /// Optional context for the transform (e.g., "acc" in @der:acc)
    pub context: Option<Selector>,
}

/// A reference to a parameter or phrase.
#[derive(Debug, Clone, PartialEq)]
pub enum Reference {
    /// Reference to a parameter or phrase: {name}
    /// At parse time we don't distinguish parameters from phrases.
    /// Resolution happens later during interpretation.
    Identifier(String),
    /// Reference to a phrase call with arguments: {phrase(arg1, arg2)}
    PhraseCall { name: String, args: Vec<Reference> },
}

/// A selector for variant selection.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// Selector identifier: :one, :other, :nom, :n
    /// At parse time we don't distinguish literal selectors from parameter selectors.
    /// Resolution happens later during interpretation.
    Identifier(String),
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
