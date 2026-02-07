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

/// Whether a definition is a term (no parameters) or a phrase (with parameters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionKind {
    /// A term has no parameters and can have variant blocks.
    Term,
    /// A phrase has one or more parameters and uses a simple template body.
    Phrase,
}

/// A parsed phrase definition from a .rlf file.
#[derive(Debug, Clone, PartialEq)]
pub struct PhraseDefinition {
    /// Whether this definition is a term or a phrase.
    pub kind: DefinitionKind,
    /// Phrase name (snake_case identifier).
    pub name: String,
    /// Parameter names (empty if no parameters).
    pub parameters: Vec<String>,
    /// Metadata tags (e.g., :fem, :masc, :a).
    pub tags: Vec<Tag>,
    /// :from(param) inheritance (None if not present).
    pub from_param: Option<String>,
    /// :match parameter names (empty if no :match).
    pub match_params: Vec<String>,
    /// Phrase body (simple template, variants, or match).
    pub body: PhraseBody,
    /// Whether the definition had an explicit empty parameter list `()`.
    ///
    /// This is used for validation: `name() = ...` is an error because empty
    /// parameter lists should be terms instead.
    pub has_empty_parens: bool,
}

/// The body of a phrase definition.
#[derive(Debug, Clone, PartialEq)]
pub enum PhraseBody {
    /// Simple phrase: name = "text";
    Simple(Template),
    /// Variant phrase (terms only): name = { one: "x", other: "y" };
    Variants(Vec<VariantEntry>),
    /// Match phrase: name($n) = :match($n) { 1: "x", *other: "y" };
    Match(Vec<MatchBranch>),
}

/// A single variant entry in a variant block.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantEntry {
    /// Variant keys (multiple keys share the same template).
    pub keys: Vec<String>,
    /// Template for this variant.
    pub template: Template,
    /// Whether this entry is marked as the default with `*`.
    pub is_default: bool,
}

/// A single branch in a `:match` block.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchBranch {
    /// Match keys (multiple keys share the same template, e.g. `one, other: "text"`).
    /// For multi-parameter match, keys use dot notation (e.g. `1.masc`, `*other.*neut`).
    pub keys: Vec<MatchKey>,
    /// Template for this branch.
    pub template: Template,
}

/// A single key in a `:match` branch.
///
/// Supports named keys (`one`, `other`, `masc`), numeric keys (`0`, `1`, `2`),
/// and multi-parameter dot-notation keys (`1.masc`, `*other.fem`).
/// Each dot-separated component has an independent default marker.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchKey {
    /// The full key value (e.g. "1", "other", "1.masc", "other.fem").
    /// Numeric keys are stored as strings. Default markers (`*`) are stripped.
    pub value: String,
    /// Whether each dimension in this key is marked as default with `*`.
    /// For single-param match, this has one element.
    /// For multi-param match with dot notation, one element per dimension.
    pub default_dimensions: Vec<bool>,
}
