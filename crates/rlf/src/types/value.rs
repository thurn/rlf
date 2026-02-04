use super::Phrase;

/// A runtime value that can be passed as a parameter to RLF phrases.
///
/// The `Value` enum provides a dynamic type system for phrase parameters,
/// allowing numbers, floats, strings, and phrases to be passed interchangeably.
///
/// # Example
///
/// ```
/// use rlf::{Value, Phrase};
///
/// // Numbers become Value::Number
/// let count: Value = 42.into();
///
/// // Strings become Value::String
/// let name: Value = "Alice".into();
///
/// // Phrases become Value::Phrase
/// let phrase: Value = Phrase::builder().text("card".to_string()).build().into();
/// ```
#[derive(Debug, Clone)]
pub enum Value {
    /// An integer number (used for plural selection).
    Number(i64),

    /// A floating-point number.
    Float(f64),

    /// A string value.
    String(String),

    /// A phrase value (carries variants and tags).
    Phrase(Phrase),
}

impl Value {
    /// Get this value as a number, if it is one.
    pub fn as_number(&self) -> Option<i64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get this value as a float, if it is one.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Number(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// Get this value as a string, if it is one.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get this value as a phrase, if it is one.
    pub fn as_phrase(&self) -> Option<&Phrase> {
        match self {
            Value::Phrase(p) => Some(p),
            _ => None,
        }
    }

    /// Check if this value (as a phrase) has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        match self {
            Value::Phrase(p) => p.has_tag(tag),
            _ => false,
        }
    }

    /// Get a variant from this value if it is a phrase.
    pub fn get_variant(&self, key: &str) -> Option<&str> {
        match self {
            Value::Phrase(p) => Some(p.variant(key)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Phrase(p) => write!(f, "{p}"),
        }
    }
}

// From implementations for common types

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as i64)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n)
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Self {
        Value::Number(n as i64)
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Number(n as i64)
    }
}

impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Value::Number(n as i64)
    }
}

impl From<f32> for Value {
    fn from(n: f32) -> Self {
        Value::Float(n as f64)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Float(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Phrase> for Value {
    fn from(p: Phrase) -> Self {
        Value::Phrase(p)
    }
}
