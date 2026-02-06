//! Tests for interpreter foundation: registry, context, plural rules.

use rlf::Value;
use rlf::interpreter::{EvalContext, EvalError, PhraseRegistry, plural_category};
use std::collections::HashMap;

// === PhraseRegistry Tests ===

#[test]
fn registry_load_and_get() {
    let mut registry = PhraseRegistry::new();
    let content = r#"
        hello = "Hello, world!";
        card = { one: "card", other: "cards" };
    "#;
    let count = registry.load_phrases(content).unwrap();
    assert_eq!(count, 2);

    let hello = registry.get("hello").unwrap();
    assert_eq!(hello.name, "hello");

    let card = registry.get("card").unwrap();
    assert_eq!(card.name, "card");

    assert!(registry.get("missing").is_none());
}

#[test]
fn registry_get_by_id() {
    let mut registry = PhraseRegistry::new();
    registry.load_phrases(r#"hello = "Hello!";"#).unwrap();

    let id = rlf::PhraseId::from_name("hello");
    let phrase = registry.get_by_id(id.as_u128()).unwrap();
    assert_eq!(phrase.name, "hello");
}

// === EvalContext Tests ===

#[test]
fn context_params() {
    let params: HashMap<String, Value> =
        [("n".to_string(), Value::Number(5))].into_iter().collect();

    let ctx = EvalContext::new(&params);
    assert_eq!(ctx.get_param("n").unwrap().as_number(), Some(5));
    assert!(ctx.get_param("missing").is_none());
}

#[test]
fn context_cycle_detection() {
    let params = HashMap::new();
    let mut ctx = EvalContext::new(&params);

    ctx.push_call("a").unwrap();
    ctx.push_call("b").unwrap();

    // Trying to push "a" again should fail
    let err = ctx.push_call("a").unwrap_err();
    match err {
        EvalError::CyclicReference { chain } => {
            assert_eq!(chain, vec!["a", "b", "a"]);
        }
        _ => panic!("Expected CyclicReference"),
    }
}

#[test]
fn context_max_depth() {
    let params = HashMap::new();
    let mut ctx = EvalContext::with_max_depth(&params, 3);

    ctx.push_call("a").unwrap();
    ctx.push_call("b").unwrap();
    ctx.push_call("c").unwrap();

    // Fourth push should fail
    let err = ctx.push_call("d").unwrap_err();
    assert!(matches!(err, EvalError::MaxDepthExceeded));
}

#[test]
fn context_pop_call() {
    let params = HashMap::new();
    let mut ctx = EvalContext::new(&params);

    ctx.push_call("a").unwrap();
    ctx.push_call("b").unwrap();
    assert_eq!(ctx.depth(), 2);

    ctx.pop_call();
    assert_eq!(ctx.depth(), 1);

    // Now we can push "b" again (it's no longer in stack)
    ctx.push_call("b").unwrap();
    assert_eq!(ctx.depth(), 2);
}

// === Plural Category Tests ===

#[test]
fn plural_english() {
    assert_eq!(plural_category("en", 0), "other");
    assert_eq!(plural_category("en", 1), "one");
    assert_eq!(plural_category("en", 2), "other");
    assert_eq!(plural_category("en", 5), "other");
}

#[test]
fn plural_russian() {
    // Russian: 1=one, 2-4=few, 5-20=many, 21=one, 22-24=few, etc.
    assert_eq!(plural_category("ru", 1), "one");
    assert_eq!(plural_category("ru", 2), "few");
    assert_eq!(plural_category("ru", 5), "many");
    assert_eq!(plural_category("ru", 21), "one");
    assert_eq!(plural_category("ru", 22), "few");
    assert_eq!(plural_category("ru", 25), "many");
}

#[test]
fn plural_arabic() {
    // Arabic has all 6 categories
    assert_eq!(plural_category("ar", 0), "zero");
    assert_eq!(plural_category("ar", 1), "one");
    assert_eq!(plural_category("ar", 2), "two");
    assert_eq!(plural_category("ar", 3), "few");
    assert_eq!(plural_category("ar", 11), "many");
    assert_eq!(plural_category("ar", 100), "other");
}

#[test]
fn plural_japanese() {
    // Japanese has only "other" for all numbers
    assert_eq!(plural_category("ja", 0), "other");
    assert_eq!(plural_category("ja", 1), "other");
    assert_eq!(plural_category("ja", 100), "other");
}

#[test]
fn plural_cached_repeated_calls() {
    // First call populates the cache, subsequent calls use it.
    assert_eq!(plural_category("en", 1), "one");
    assert_eq!(plural_category("en", 1), "one");
    assert_eq!(plural_category("en", 2), "other");
    assert_eq!(plural_category("en", 2), "other");
    assert_eq!(plural_category("ru", 5), "many");
    assert_eq!(plural_category("ru", 5), "many");
}

#[test]
fn plural_unknown_language_falls_back_to_english() {
    // Unknown language codes should fall back to English rules
    assert_eq!(plural_category("xx", 1), "one");
    assert_eq!(plural_category("xx", 2), "other");
    assert_eq!(plural_category("zz", 0), "other");
}

#[test]
fn plural_multiple_languages_interleaved() {
    // Exercises cache with multiple languages being used alternately
    assert_eq!(plural_category("en", 1), "one");
    assert_eq!(plural_category("ru", 2), "few");
    assert_eq!(plural_category("ar", 2), "two");
    assert_eq!(plural_category("en", 5), "other");
    assert_eq!(plural_category("ru", 1), "one");
    assert_eq!(plural_category("ar", 0), "zero");
}

#[test]
fn plural_all_supported_languages() {
    // Verify every supported language produces a valid category
    let languages = [
        "ar", "bn", "de", "el", "en", "es", "fa", "fr", "he", "hi", "id", "it", "ja", "ko", "nl",
        "pl", "pt", "ro", "ru", "th", "tr", "uk", "vi", "zh",
    ];
    let valid_categories = ["zero", "one", "two", "few", "many", "other"];
    for lang in languages {
        let cat = plural_category(lang, 1);
        assert!(
            valid_categories.contains(&cat),
            "Language {lang} returned invalid category {cat:?}"
        );
    }
}
