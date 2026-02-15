//! Tests confirming that Phrase uses VariantKey and Tag types (not raw
//! String), matching our documented API.

use std::collections::HashMap;

use rlf::{Phrase, Tag, VariantKey};

#[test]
fn phrase_variants_use_variant_key() {
    let phrase = Phrase::builder()
        .text("card".to_string())
        .variants(HashMap::from([
            (VariantKey::new("one"), "card".to_string()),
            (VariantKey::new("other"), "cards".to_string()),
        ]))
        .build();
    assert_eq!(phrase.variant("one"), "card");
    assert_eq!(phrase.variant("other"), "cards");
}

#[test]
fn phrase_tags_use_tag_type() {
    let phrase = Phrase::builder()
        .text("cat".to_string())
        .tags(vec![Tag::new("fem"), Tag::new("an")])
        .build();
    assert!(phrase.has_tag("fem"));
    assert!(phrase.has_tag("an"));
    assert_eq!(phrase.first_tag().map(Tag::as_str), Some("fem"));
}

#[test]
fn variant_key_deref_display_from() {
    let key = VariantKey::new("one");
    assert_eq!(&*key, "one");
    assert_eq!(key.to_string(), "one");

    let from_str: VariantKey = VariantKey::from("other");
    assert_eq!(from_str.as_str(), "other");
}

#[test]
fn tag_deref_display_from() {
    let tag = Tag::new("masc");
    assert_eq!(&*tag, "masc");
    assert_eq!(tag.to_string(), "masc");

    let from_str: Tag = Tag::from("fem");
    assert_eq!(from_str.as_str(), "fem");
}

#[test]
fn join_empty_returns_empty_phrase() {
    let result = Phrase::join(&[], ", ");
    assert_eq!(result.to_string(), "");
    assert!(result.variants.is_empty());
    assert!(result.tags.is_empty());
}

#[test]
fn join_single_phrase_returns_its_text() {
    let p = Phrase::builder()
        .text("hello".to_string())
        .variants(HashMap::from([(VariantKey::new("inf"), "hi".to_string())]))
        .tags(vec![Tag::new("fem")])
        .build();
    let result = Phrase::join(&[p], ", ");
    assert_eq!(result.to_string(), "hello");
    assert_eq!(result.variant("inf"), "hi");
    // Tags are not preserved by join
    assert!(result.tags.is_empty());
}

#[test]
fn join_text_with_separator() {
    let a = Phrase::builder().text("alpha".to_string()).build();
    let b = Phrase::builder().text("beta".to_string()).build();
    let c = Phrase::builder().text("gamma".to_string()).build();
    let result = Phrase::join(&[a, b, c], " and ");
    assert_eq!(result.to_string(), "alpha and beta and gamma");
}

#[test]
fn join_preserves_shared_variants() {
    let a = Phrase::builder()
        .text("рассеяйте врага".to_string())
        .variants(HashMap::from([
            (VariantKey::new("inf"), "рассеять врага".to_string()),
            (VariantKey::new("nom"), "рассеяние врага".to_string()),
        ]))
        .build();
    let b = Phrase::builder()
        .text("возьмите 2 карты".to_string())
        .variants(HashMap::from([
            (VariantKey::new("inf"), "взять 2 карты".to_string()),
            (VariantKey::new("nom"), "взятие 2 карт".to_string()),
        ]))
        .build();
    let result = Phrase::join(&[a, b], " и ");
    assert_eq!(result.to_string(), "рассеяйте врага и возьмите 2 карты");
    assert_eq!(result.variant("inf"), "рассеять врага и взять 2 карты");
    assert_eq!(result.variant("nom"), "рассеяние врага и взятие 2 карт");
}

#[test]
fn join_drops_variants_not_in_all_phrases() {
    let a = Phrase::builder()
        .text("A".to_string())
        .variants(HashMap::from([
            (VariantKey::new("inf"), "a_inf".to_string()),
            (VariantKey::new("only_a"), "only_a_val".to_string()),
        ]))
        .build();
    let b = Phrase::builder()
        .text("B".to_string())
        .variants(HashMap::from([
            (VariantKey::new("inf"), "b_inf".to_string()),
            (VariantKey::new("only_b"), "only_b_val".to_string()),
        ]))
        .build();
    let result = Phrase::join(&[a, b], ", ");
    assert_eq!(result.to_string(), "A, B");
    // Only "inf" is shared
    assert_eq!(result.variant("inf"), "a_inf, b_inf");
    assert!(!result.variants.contains_key(&VariantKey::new("only_a")));
    assert!(!result.variants.contains_key(&VariantKey::new("only_b")));
}

#[test]
fn join_no_shared_variants_returns_text_only() {
    let a = Phrase::builder()
        .text("X".to_string())
        .variants(HashMap::from([(VariantKey::new("v1"), "x1".to_string())]))
        .build();
    let b = Phrase::builder()
        .text("Y".to_string())
        .variants(HashMap::from([(VariantKey::new("v2"), "y2".to_string())]))
        .build();
    let result = Phrase::join(&[a, b], " ");
    assert_eq!(result.to_string(), "X Y");
    assert!(result.variants.is_empty());
}

#[test]
fn join_phrases_without_variants() {
    let a = Phrase::builder().text("one".to_string()).build();
    let b = Phrase::builder().text("two".to_string()).build();
    let result = Phrase::join(&[a, b], ", ");
    assert_eq!(result.to_string(), "one, two");
    assert!(result.variants.is_empty());
}
