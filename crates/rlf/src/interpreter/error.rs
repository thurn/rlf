//! Error types for the RLF interpreter.

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
        source: std::io::Error,
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

/// An error that occurred during phrase evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Phrase not found by name.
    #[error("phrase not found: '{name}'")]
    PhraseNotFound { name: String },

    /// Phrase not found by PhraseId hash.
    #[error("phrase not found for id: {id}")]
    PhraseNotFoundById { id: u64 },

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
}
