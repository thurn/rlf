//! Public AST types for RLF templates and phrase definitions.
//!
//! These types are public to enable external tooling (linters, formatters, etc.).

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
