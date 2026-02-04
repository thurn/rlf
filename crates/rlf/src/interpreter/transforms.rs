//! Transform registry for string transformations.
//!
//! Transforms are functions that modify values (e.g., @cap, @upper, @lower).
//! This module provides the registry infrastructure; actual implementations
//! are added in Phase 3 (Universal Transforms and ICU4X).

use std::collections::HashMap;

use crate::interpreter::EvalError;
use crate::types::Value;

/// Transform function signature.
///
/// Takes:
/// - `value`: The value to transform
/// - `context`: Optional context for the transform (e.g., "acc" in @der:acc)
/// - `lang`: Language code for locale-specific transforms
///
/// Returns the transformed value or an error.
pub type TransformFn = fn(Value, Option<&str>, &str) -> Result<Value, EvalError>;

/// Registry for transform functions.
///
/// Transforms are registered per-language with universal transforms available to all.
/// Language-specific transforms take precedence over universal transforms.
///
/// NOTE: Transform implementations are added in Phase 3 (Universal Transforms and ICU4X).
pub struct TransformRegistry {
    /// Universal transforms available in all languages (@cap, @upper, @lower).
    universal: HashMap<String, TransformFn>,
    /// Language-specific transforms (keyed by lang -> transform_name -> fn).
    language_specific: HashMap<String, HashMap<String, TransformFn>>,
}

impl TransformRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            universal: HashMap::new(),
            language_specific: HashMap::new(),
        }
    }

    /// Get a transform by name for a language.
    ///
    /// Checks language-specific transforms first, then falls back to universal.
    pub fn get(&self, name: &str, lang: &str) -> Option<TransformFn> {
        // Language-specific takes precedence
        if let Some(lang_transforms) = self.language_specific.get(lang) {
            if let Some(f) = lang_transforms.get(name) {
                return Some(*f);
            }
        }
        // Fall back to universal
        self.universal.get(name).copied()
    }

    /// Check if a transform exists for a language.
    pub fn has_transform(&self, name: &str, lang: &str) -> bool {
        self.get(name, lang).is_some()
    }
}

impl Default for TransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}
