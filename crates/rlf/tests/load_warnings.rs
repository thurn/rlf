//! Tests for PhraseRegistry utilities.

// =========================================================================
// Registry phrase_names / len / is_empty
// =========================================================================

#[test]
fn registry_len_and_is_empty() {
    use rlf::PhraseRegistry;

    let mut registry = PhraseRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);

    registry.load_phrases(r#"hello = "Hello!";"#).unwrap();
    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
}

#[test]
fn registry_phrase_names_returns_all_names() {
    use rlf::PhraseRegistry;

    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        hello = "Hello!";
        goodbye = "Goodbye!";
        greet($name) = "Hi, {$name}!";
    "#,
        )
        .unwrap();

    let mut names: Vec<&str> = registry.phrase_names().collect();
    names.sort();
    assert_eq!(names, vec!["goodbye", "greet", "hello"]);
}
