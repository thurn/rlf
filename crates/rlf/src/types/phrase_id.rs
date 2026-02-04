use const_fnv1a_hash::fnv1a_hash_str_64;
use serde::{Deserialize, Serialize};

/// A compact, serializable identifier for an RLF phrase.
///
/// `PhraseId` wraps a 64-bit FNV-1a hash of the phrase name. This provides:
/// - **Stability**: Same name always produces the same hash
/// - **Compactness**: 8 bytes, implements `Copy`, stack-allocated
/// - **Serializability**: Works with JSON, bincode, protobuf, etc.
/// - **Const construction**: `from_name()` is a `const fn`
///
/// # Example
///
/// ```
/// use rlf::PhraseId;
///
/// // Create at compile time
/// const CARD_ID: PhraseId = PhraseId::from_name("card");
///
/// // Create at runtime
/// let draw_id = PhraseId::from_name("draw");
///
/// // Use as HashMap key
/// use std::collections::HashMap;
/// let mut phrases: HashMap<PhraseId, &str> = HashMap::new();
/// phrases.insert(CARD_ID, "card phrase");
/// ```
///
/// # Note
///
/// The `resolve()` and `call()` methods require a `Locale` which is
/// implemented in Phase 4. For now, `PhraseId` provides identification
/// and storage capabilities.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhraseId(u64);

impl PhraseId {
    /// Create a PhraseId from a phrase name at compile time.
    ///
    /// This is a `const fn`, enabling compile-time constant creation:
    ///
    /// ```
    /// use rlf::PhraseId;
    ///
    /// const FIRE_ELEMENTAL: PhraseId = PhraseId::from_name("fire_elemental");
    /// ```
    pub const fn from_name(name: &str) -> Self {
        Self(fnv1a_hash_str_64(name))
    }

    /// Get the raw hash value.
    ///
    /// Useful for debugging or when you need the underlying u64.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for PhraseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PhraseId({:016x})", self.0)
    }
}

// Note: resolve() and call() methods require Locale (Phase 4)
// Note: name() method requires registry (Phase 2)
