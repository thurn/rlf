pub mod interpreter;
pub mod parser;
pub mod types;

pub use interpreter::{
    EvalContext, EvalError, LoadError, LoadWarning, Locale, PhraseRegistry, TransformRegistry,
    compute_suggestions,
};
pub use types::{Phrase, PhraseId, Tag, Value, VariantKey};

// Re-export the rlf! macro
pub use rlf_macros::rlf;

/// Creates a `HashMap<String, Value>` from key-value pairs.
///
/// Values are automatically converted via `Into<Value>`, so you can pass
/// integers, floats, strings, or `Phrase` values directly.
///
/// # Example
///
/// ```
/// use rlf::{params, Value};
///
/// let p = params! { "count" => 3, "name" => "Alice" };
/// assert_eq!(p.len(), 2);
/// assert_eq!(p["count"].as_number(), Some(3));
/// assert_eq!(p["name"].as_string(), Some("Alice"));
/// ```
#[macro_export]
macro_rules! params {
    {} => {
        ::std::collections::HashMap::<String, $crate::Value>::new()
    };
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let mut map = ::std::collections::HashMap::<String, $crate::Value>::new();
            $(
                map.insert($key.to_string(), ::std::convert::Into::<$crate::Value>::into($value));
            )+
            map
        }
    };
}
