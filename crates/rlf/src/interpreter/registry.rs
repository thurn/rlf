//! Phrase registry for storing and looking up phrase definitions.

use std::collections::HashMap;

use crate::interpreter::EvalError;
use crate::parser::ast::PhraseDefinition;
use crate::types::PhraseId;

/// A registry for storing and looking up phrase definitions.
///
/// The registry supports lookup by both name (string) and by PhraseId (hash).
/// This enables efficient runtime lookups while maintaining human-readable names.
#[derive(Debug, Default)]
pub struct PhraseRegistry {
    /// Phrases indexed by name.
    phrases: HashMap<String, PhraseDefinition>,
    /// Maps PhraseId hash to phrase name for id-based lookup.
    id_to_name: HashMap<u64, String>,
}

impl PhraseRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a phrase definition by name.
    pub fn get(&self, name: &str) -> Option<&PhraseDefinition> {
        self.phrases.get(name)
    }

    /// Get a phrase definition by PhraseId hash.
    pub fn get_by_id(&self, id: u64) -> Option<&PhraseDefinition> {
        self.id_to_name
            .get(&id)
            .and_then(|name| self.phrases.get(name))
    }

    /// Insert a phrase definition into the registry.
    ///
    /// Returns an error if a hash collision is detected (different name but same hash).
    pub fn insert(&mut self, def: PhraseDefinition) -> Result<(), EvalError> {
        let name = def.name.clone();
        let id = PhraseId::from_name(&name);
        let hash = id.as_u64();

        // Check for hash collision (different name but same hash)
        if let Some(existing_name) = self.id_to_name.get(&hash) {
            if existing_name != &name {
                // This is a hash collision - different names producing same hash
                // This should be extremely rare with 64-bit FNV-1a but we handle it
                return Err(EvalError::PhraseNotFound {
                    name: format!(
                        "hash collision: '{}' and '{}' produce same hash",
                        existing_name, name
                    ),
                });
            }
        }

        self.id_to_name.insert(hash, name.clone());
        self.phrases.insert(name, def);
        Ok(())
    }
}
