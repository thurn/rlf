//! Error types for the RLF interpreter.

use std::path::PathBuf;

use thiserror::Error;

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
    #[error("missing variant '{key}' in phrase '{phrase}', available: {}", available.join(", "))]
    MissingVariant {
        phrase: String,
        key: String,
        available: Vec<String>,
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
