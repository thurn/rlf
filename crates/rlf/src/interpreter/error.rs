//! Error types for the RLF interpreter.

use std::fmt;
use std::io;
use std::path::PathBuf;

use strsim::levenshtein;
use thiserror::Error;

/// Compute "did you mean" suggestions for a key using Levenshtein distance.
///
/// Returns up to 3 suggestions with edit distance <= 2 (or <= 1 for short keys).
pub fn compute_suggestions(target: &str, available: &[String]) -> Vec<String> {
    let max_distance = if target.len() <= 3 { 1 } else { 2 };

    let mut scored: Vec<_> = available
        .iter()
        .filter_map(|candidate| {
            let dist = levenshtein(target, candidate);
            if dist <= max_distance && dist > 0 {
                Some((candidate.clone(), dist))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by_key(|(_, dist)| *dist);
    scored.truncate(3);
    scored.into_iter().map(|(s, _)| s).collect()
}

fn format_suggestions(suggestions: &[String]) -> String {
    if suggestions.is_empty() {
        String::new()
    } else {
        format!("; did you mean: {}?", suggestions.join(", "))
    }
}

/// Errors that occur during translation loading.
#[derive(Debug, Error)]
pub enum LoadError {
    /// File I/O error when reading translation file.
    #[error("failed to read '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Parse error with file location context.
    #[error("{path}:{line}:{column}: {message}")]
    Parse {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    /// Attempted to reload translations that were loaded from a string.
    #[error("cannot reload '{language}': was loaded from string, not file")]
    NoPathForReload { language: String },
}

/// A warning produced during translation linting.
///
/// Warnings indicate potential issues in translation files that do not prevent
/// loading but may indicate suboptimal patterns. Use `lint_definitions()` to
/// check phrase definitions for common issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadWarning {
    /// Variant block on `:from` phrase could be replaced with simple template.
    RedundantPassthroughBlock {
        /// Name of the phrase.
        name: String,
        /// Language code of the translation.
        language: String,
    },
    /// Explicit selector on `:from` parameter matches enclosing variant key.
    RedundantFromSelector {
        /// Name of the phrase.
        name: String,
        /// Language code of the translation.
        language: String,
        /// The variant key that matches the selector.
        variant_key: String,
        /// The parameter with the redundant selector.
        parameter: String,
    },
    /// Phrase without `:from` or tags uses a parameter that may carry metadata.
    LikelyMissingFrom {
        /// Name of the phrase.
        name: String,
        /// Language code of the translation.
        language: String,
        /// The parameter that may carry metadata.
        parameter: String,
    },
    /// `:from` phrase with identity template could use body-less form.
    VerboseTransparentWrapper {
        /// Name of the phrase.
        name: String,
        /// Language code of the translation.
        language: String,
    },
}

impl fmt::Display for LoadWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadWarning::RedundantPassthroughBlock { name, language } => {
                write!(
                    f,
                    "warning: phrase '{name}' in '{language}' has redundant passthrough variant block; use simple :from template instead"
                )
            }
            LoadWarning::RedundantFromSelector {
                name,
                language,
                variant_key,
                parameter,
            } => {
                write!(
                    f,
                    "warning: phrase '{name}' in '{language}' has redundant selector ':{variant_key}' on :from parameter '${parameter}'; bare '${{${parameter}}}' resolves to the same value"
                )
            }
            LoadWarning::LikelyMissingFrom {
                name,
                language,
                parameter,
            } => {
                write!(
                    f,
                    "warning: phrase '{name}' in '{language}' uses parameter '${parameter}' without :from; tags and variants may be lost"
                )
            }
            LoadWarning::VerboseTransparentWrapper { name, language } => {
                write!(
                    f,
                    "warning: phrase '{name}' in '{language}' uses ':from($p) \"{{$p}}\"'; use body-less ':from($p);' instead"
                )
            }
        }
    }
}

/// A warning produced during phrase evaluation.
///
/// Runtime warnings indicate potential issues detected during evaluation
/// that do not prevent evaluation from completing but may indicate
/// translation bugs (e.g., silent metadata loss or missing variant selectors).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalWarning {
    /// Phrase called with a `Value::Phrase` argument but has no `:from`.
    ///
    /// Tags and variants from the argument will be lost in the result.
    PhraseArgumentWithoutFrom {
        /// Name of the phrase being called.
        phrase: String,
        /// The parameter that received a Phrase value.
        parameter: String,
    },

    /// Bare parameter reference to a Phrase with multi-dimensional variants
    /// outside `:from` context.
    ///
    /// The default variant will be used, which may not be grammatically correct.
    /// Use an explicit selector (e.g., `{$param:acc}`) or `{$param:*}` to
    /// acknowledge the default.
    MissingSelectorOnMultiDimensional {
        /// Name of the parameter with multi-dimensional variants.
        parameter: String,
        /// Available variant keys on the parameter's value.
        available_keys: Vec<String>,
    },
}

impl fmt::Display for EvalWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalWarning::PhraseArgumentWithoutFrom { phrase, parameter } => {
                write!(
                    f,
                    "warning: phrase '{phrase}' receives Phrase value for parameter '${parameter}' but has no :from; tags and variants will be lost"
                )
            }
            EvalWarning::MissingSelectorOnMultiDimensional {
                parameter,
                available_keys,
            } => {
                write!(
                    f,
                    "warning: bare reference '${{{parameter}}}' to Phrase with multi-dimensional variants; use an explicit selector or ':*' for the default; available keys: {}",
                    available_keys.join(", ")
                )
            }
        }
    }
}

/// An error that occurred during phrase evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Phrase not found by name.
    #[error("phrase not found: '{name}'")]
    PhraseNotFound { name: String },

    /// Phrase not found by PhraseId hash.
    #[error("phrase not found for id: {id}")]
    PhraseNotFoundById { id: u128 },

    /// Required variant key is missing from phrase.
    #[error("missing variant '{key}' in phrase '{phrase}', available: {}{}", available.join(", "), format_suggestions(suggestions))]
    MissingVariant {
        phrase: String,
        key: String,
        available: Vec<String>,
        suggestions: Vec<String>,
    },

    /// Transform requires a tag that the phrase doesn't have.
    #[error("transform '@{transform}' requires tag {expected:?} on phrase '{phrase}'")]
    MissingTag {
        transform: String,
        expected: Vec<String>,
        phrase: String,
    },

    /// Wrong number of arguments passed to phrase call.
    #[error("phrase '{phrase}' expects {expected} arguments, got {got}")]
    ArgumentCount {
        phrase: String,
        expected: usize,
        got: usize,
    },

    /// Cyclic reference detected during evaluation.
    #[error("cyclic reference detected: {}", chain.join(" -> "))]
    CyclicReference { chain: Vec<String> },

    /// Maximum recursion depth exceeded.
    #[error("maximum recursion depth exceeded")]
    MaxDepthExceeded,

    /// Unknown transform name.
    #[error("unknown transform '@{name}'")]
    UnknownTransform { name: String },

    /// Term cannot be called with arguments.
    #[error(
        "'{name}' is a term — cannot use () call syntax; use {{{}:variant}} or {{{}:$param}} to select a variant",
        name,
        name
    )]
    ArgumentsToTerm { name: String },

    /// Phrase cannot use `:` without `()`.
    #[error(
        "'{name}' is a phrase — cannot use : without (); use {{{}(...)}} or {{{}(...):variant}}",
        name,
        name
    )]
    SelectorOnPhrase { name: String },

    /// No branch matched in a :match block and no default was found.
    #[error("no matching branch in :match block for keys {keys:?}")]
    MissingMatchDefault { keys: Vec<Vec<String>> },

    /// Parameter not found in scope.
    #[error("unknown parameter '${name}' — not in scope")]
    UnknownParameter { name: String },
}
