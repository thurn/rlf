use std::fmt::{Display, Formatter, Result as FmtResult};

use const_fnv1a_hash::fnv1a_hash_str_64;
use serde::{Deserialize, Serialize};

use crate::interpreter::{EvalError, Locale, PhraseRegistry};
use crate::types::Value;

/// A compact, serializable identifier for an RLF phrase.
///
/// `PhraseId` wraps a 64-bit FNV-1a hash of the phrase name. This provides:
/// - **Stability**: Same name always produces the same hash
/// - **Compactness**: 8 bytes, implements `Copy`, stack-allocated
/// - **Serializability**: Works with JSON, bincode, protobuf, etc.
/// - **Const construction**: `from_name()` is a `const fn`
///
/// Use `resolve()` for parameterless phrases (returns `Phrase` with variants
/// and tags) and `call()` for phrases with parameters (returns the evaluated
/// `Phrase`).
///
/// # Example
///
/// ```
/// use rlf::{Locale, PhraseId};
///
/// const HELLO: PhraseId = PhraseId::from_name("hello");
///
/// let mut locale = Locale::new();
/// locale.load_translations_str("en", r#"hello = "Hello!";"#).unwrap();
/// let phrase = HELLO.resolve(&locale).unwrap();
/// assert_eq!(phrase.to_string(), "Hello!");
/// ```
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

impl Display for PhraseId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "PhraseId({:016x})", self.0)
    }
}

// =========================================================================
// Locale-Based Resolution
// =========================================================================

impl PhraseId {
    /// Resolve a parameterless phrase to its Phrase value.
    ///
    /// Looks up the phrase in the locale's current language and evaluates it.
    /// Returns the full Phrase with text, variants, and tags.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{Locale, PhraseId};
    ///
    /// let mut locale = Locale::new();
    /// locale.load_translations_str("en", r#"
    ///     card = { one: "card", other: "cards" };
    /// "#).unwrap();
    ///
    /// let id = PhraseId::from_name("card");
    /// let phrase = id.resolve(&locale).unwrap();
    /// assert_eq!(phrase.to_string(), "card");
    /// assert_eq!(phrase.variant("other"), "cards");
    /// ```
    pub fn resolve(&self, locale: &Locale) -> Result<crate::Phrase, EvalError> {
        locale.get_phrase_by_id(self.0)
    }

    /// Call a phrase with positional arguments.
    ///
    /// Looks up the phrase in the locale's current language, binds arguments
    /// to parameters, and evaluates the phrase.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{Locale, PhraseId, Value};
    ///
    /// let mut locale = Locale::new();
    /// locale.load_translations_str("en", r#"
    ///     greet(name) = "Hello, {name}!";
    /// "#).unwrap();
    ///
    /// let id = PhraseId::from_name("greet");
    /// let phrase = id.call(&locale, &[Value::from("World")]).unwrap();
    /// assert_eq!(phrase.to_string(), "Hello, World!");
    /// ```
    pub fn call(&self, locale: &Locale, args: &[Value]) -> Result<crate::Phrase, EvalError> {
        locale.call_phrase_by_id(self.0, args)
    }

    /// Get the phrase name for debugging.
    ///
    /// Looks up the name in the locale's current language registry.
    /// Returns None if the phrase is not registered.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{Locale, PhraseId};
    ///
    /// let mut locale = Locale::new();
    /// locale.load_translations_str("en", r#"hello = "Hello!";"#).unwrap();
    ///
    /// let id = PhraseId::from_name("hello");
    /// assert_eq!(id.name(&locale), Some("hello"));
    ///
    /// let unknown = PhraseId::from_name("nonexistent");
    /// assert_eq!(unknown.name(&locale), None);
    /// ```
    pub fn name<'a>(&self, locale: &'a Locale) -> Option<&'a str> {
        locale.name_for_id(self.0)
    }

    /// Check if this phrase has parameters.
    ///
    /// Returns false if the phrase is not found.
    pub fn has_parameters(&self, locale: &Locale) -> bool {
        locale.phrase_parameter_count(self.0) > 0
    }

    /// Get the number of parameters this phrase expects.
    ///
    /// Returns 0 if the phrase is not found.
    pub fn parameter_count(&self, locale: &Locale) -> usize {
        locale.phrase_parameter_count(self.0)
    }
}

// =========================================================================
// Global Locale Resolution
// =========================================================================

#[cfg(feature = "global-locale")]
impl PhraseId {
    /// Resolve a parameterless phrase using the global locale.
    pub fn resolve_global(&self) -> Result<crate::Phrase, EvalError> {
        crate::with_locale(|locale| locale.get_phrase_by_id(self.0))
    }

    /// Call a phrase with positional arguments using the global locale.
    pub fn call_global(&self, args: &[Value]) -> Result<crate::Phrase, EvalError> {
        crate::with_locale(|locale| locale.call_phrase_by_id(self.0, args))
    }

    /// Get the phrase name using the global locale.
    ///
    /// Returns an owned `String` because the lock cannot outlive the call.
    pub fn name_global(&self) -> Option<String> {
        crate::with_locale(|locale| locale.name_for_id(self.0).map(str::to_owned))
    }
}

// =========================================================================
// Registry-Based Resolution
// =========================================================================

impl PhraseId {
    /// Resolve using a PhraseRegistry directly.
    ///
    /// This is a lower-level method that bypasses Locale. Prefer `resolve()`
    /// when a Locale is available.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{PhraseId, PhraseRegistry};
    ///
    /// let mut registry = PhraseRegistry::new();
    /// registry.load_phrases(r#"hello = "Hello!";"#).unwrap();
    ///
    /// let id = PhraseId::from_name("hello");
    /// let phrase = id.resolve_with_registry(&registry, "en").unwrap();
    /// assert_eq!(phrase.to_string(), "Hello!");
    /// ```
    pub fn resolve_with_registry(
        &self,
        registry: &PhraseRegistry,
        lang: &str,
    ) -> Result<crate::Phrase, EvalError> {
        registry.get_phrase_by_id(self.0, lang)
    }

    /// Call using a PhraseRegistry directly.
    ///
    /// This is a lower-level method that bypasses Locale. Prefer `call()`
    /// when a Locale is available.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{PhraseId, PhraseRegistry, Value};
    ///
    /// let mut registry = PhraseRegistry::new();
    /// registry.load_phrases(r#"greet(name) = "Hello, {name}!";"#).unwrap();
    ///
    /// let id = PhraseId::from_name("greet");
    /// let phrase = id.call_with_registry(&registry, "en", &[Value::from("World")]).unwrap();
    /// assert_eq!(phrase.to_string(), "Hello, World!");
    /// ```
    pub fn call_with_registry(
        &self,
        registry: &PhraseRegistry,
        lang: &str,
        args: &[Value],
    ) -> Result<crate::Phrase, EvalError> {
        registry.call_phrase_by_id(self.0, lang, args)
    }
}
