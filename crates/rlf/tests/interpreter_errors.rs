//! Tests for error types and error message formatting.

use rlf::{EvalError, LoadError, compute_suggestions};
use std::path::PathBuf;

#[test]
fn compute_suggestions_finds_similar_keys() {
    let available = vec![
        "one".to_string(),
        "other".to_string(),
        "few".to_string(),
        "many".to_string(),
    ];

    // "on" is close to "one" (distance 1)
    let suggestions = compute_suggestions("on", &available);
    assert_eq!(suggestions, vec!["one"]);

    // "oter" is close to "other" (distance 1), also close to "one" (distance 2)
    // Both returned because max_distance=2 for keys longer than 3 chars
    let suggestions = compute_suggestions("oter", &available);
    assert!(suggestions.contains(&"other".to_string()));
    assert_eq!(suggestions[0], "other"); // closest match first

    // "xyz" has no close matches
    let suggestions = compute_suggestions("xyz", &available);
    assert!(suggestions.is_empty());
}

#[test]
fn compute_suggestions_limits_to_three() {
    let available: Vec<String> = (0..10).map(|i| format!("item{}", i)).collect();

    // "item" is close to all of them
    let suggestions = compute_suggestions("item", &available);
    assert!(suggestions.len() <= 3);
}

#[test]
fn load_error_io_displays_path() {
    let err = LoadError::Io {
        path: PathBuf::from("/path/to/file.rlf"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
    };
    let msg = err.to_string();
    assert!(msg.contains("/path/to/file.rlf"));
    assert!(msg.contains("file not found"));
}

#[test]
fn load_error_parse_displays_location() {
    let err = LoadError::Parse {
        path: PathBuf::from("translations/ru.rlf"),
        line: 42,
        column: 15,
        message: "unexpected token".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("translations/ru.rlf"));
    assert!(msg.contains("42:15"));
    assert!(msg.contains("unexpected token"));
}

#[test]
fn load_error_no_path_for_reload() {
    let err = LoadError::NoPathForReload {
        language: "ru".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("ru"));
    assert!(msg.contains("string"));
}

#[test]
fn missing_variant_includes_suggestions() {
    let err = EvalError::MissingVariant {
        phrase: "card".to_string(),
        key: "on".to_string(),
        available: vec!["one".to_string(), "other".to_string()],
        suggestions: vec!["one".to_string()],
    };
    let msg = err.to_string();
    assert!(msg.contains("did you mean: one?"));
}

#[test]
fn missing_variant_no_suggestions_when_empty() {
    let err = EvalError::MissingVariant {
        phrase: "card".to_string(),
        key: "xyz".to_string(),
        available: vec!["one".to_string(), "other".to_string()],
        suggestions: vec![],
    };
    let msg = err.to_string();
    assert!(!msg.contains("did you mean"));
}
