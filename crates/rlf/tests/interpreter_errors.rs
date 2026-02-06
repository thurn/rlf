//! Tests for error types and error message formatting.

use rlf::{EvalError, LoadError, Locale, PhraseId, compute_suggestions};
use std::collections::HashMap;
use std::io::{self, ErrorKind};
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
        source: io::Error::new(ErrorKind::NotFound, "file not found"),
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

#[test]
fn phrase_not_found_by_id_via_resolve() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hi!";"#)
        .unwrap();

    let bogus = PhraseId::from_name("nonexistent_phrase");
    let err = bogus.resolve(&locale).unwrap_err();
    assert!(
        matches!(err, EvalError::PhraseNotFoundById { .. }),
        "expected PhraseNotFoundById, got: {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("phrase not found for id"));
}

#[test]
fn phrase_not_found_by_id_via_call() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hi!";"#)
        .unwrap();

    let bogus = PhraseId::from_name("does_not_exist");
    let err = bogus.call(&locale, &[]).unwrap_err();
    assert!(
        matches!(err, EvalError::PhraseNotFoundById { .. }),
        "expected PhraseNotFoundById, got: {err:?}"
    );
}

#[test]
fn max_depth_exceeded_via_deep_phrase_chain() {
    let mut locale = Locale::new();

    // Build a chain of 70 unique phrases: p0 -> p1 -> p2 -> ... -> p69
    // Each phrase references the next, so evaluation walks the full chain.
    // The default max depth is 64, so this triggers MaxDepthExceeded.
    let mut definitions = String::new();
    for i in 0..70 {
        if i < 69 {
            definitions.push_str(&format!("p{i} = \"{{p{next}}}\";\n", next = i + 1));
        } else {
            definitions.push_str(&format!("p{i} = \"done\";\n"));
        }
    }
    locale.load_translations_str("en", &definitions).unwrap();

    let err = locale.get_phrase("p0").unwrap_err();
    assert!(
        matches!(err, EvalError::MaxDepthExceeded),
        "expected MaxDepthExceeded, got: {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("maximum recursion depth exceeded"));
}

#[test]
fn unknown_transform_via_eval_str() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"foo = "bar";"#)
        .unwrap();

    let err = locale
        .eval_str("{@nonexistent foo}", HashMap::new())
        .unwrap_err();
    assert!(
        matches!(err, EvalError::UnknownTransform { .. }),
        "expected UnknownTransform, got: {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("@nonexistent"));
}

#[test]
fn unknown_transform_displays_name() {
    let err = EvalError::UnknownTransform {
        name: "bogus".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("unknown transform '@bogus'"));
}

#[test]
fn max_depth_exceeded_displays_message() {
    let err = EvalError::MaxDepthExceeded;
    let msg = err.to_string();
    assert_eq!(msg, "maximum recursion depth exceeded");
}

#[test]
fn phrase_not_found_by_id_displays_id() {
    let err = EvalError::PhraseNotFoundById { id: 99999 };
    let msg = err.to_string();
    assert!(msg.contains("99999"));
}
