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
