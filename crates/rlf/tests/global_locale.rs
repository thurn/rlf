#![cfg(feature = "global-locale")]

//! Integration tests for the `global-locale` feature.

use rlf::PhraseId;

mod strings {
    use rlf::rlf;

    rlf! {
        card = { one: "card", other: "cards" };
        draw($n) = "Draw {$n} {card:$n}.";
        hello = "Hello!";
    }
}

// =========================================================================
// Generated phrase functions (no locale parameter)
// =========================================================================

#[test]
fn parameterless_phrase() {
    assert_eq!(strings::hello().to_string(), "Hello!");
}

#[test]
fn parameterized_phrase() {
    assert_eq!(strings::draw(3).to_string(), "Draw 3 cards.");
}

#[test]
fn variant_phrase() {
    let card = strings::card();
    assert_eq!(card.to_string(), "card");
    assert_eq!(card.variant("other"), "cards");
}

// =========================================================================
// Auto-registration
// =========================================================================

#[test]
fn register_source_phrases_is_idempotent() {
    strings::register_source_phrases();
    strings::register_source_phrases();
    assert_eq!(strings::hello().to_string(), "Hello!");
}

// =========================================================================
// Global locale API
// =========================================================================

#[test]
fn set_language_and_language_round_trip() {
    rlf::set_language("en");
    assert_eq!(rlf::language(), "en");
}

#[test]
fn with_locale_read_access() {
    rlf::with_locale(|locale| {
        assert_eq!(locale.language(), "en");
    });
}

#[test]
fn with_locale_mut_write_access() {
    rlf::with_locale_mut(|locale| {
        locale.set_language("en");
    });
    assert_eq!(rlf::language(), "en");
}

// =========================================================================
// PhraseId global methods
// =========================================================================

#[test]
fn phrase_id_resolve_global() {
    // Ensure phrases are registered
    strings::register_source_phrases();

    let id = PhraseId::from_name("hello");
    let phrase = id.resolve_global().unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

#[test]
fn phrase_id_call_global() {
    strings::register_source_phrases();

    let id = PhraseId::from_name("draw");
    let phrase = id.call_global(&[5.into()]).unwrap();
    assert_eq!(phrase.to_string(), "Draw 5 cards.");
}

#[test]
fn phrase_id_name_global() {
    strings::register_source_phrases();

    let id = PhraseId::from_name("card");
    assert_eq!(id.name_global(), Some("card".to_string()));

    let unknown = PhraseId::from_name("nonexistent");
    assert_eq!(unknown.name_global(), None);
}

// =========================================================================
// phrase_ids module still generated
// =========================================================================

#[test]
fn phrase_ids_constants_exist() {
    let _card = strings::phrase_ids::CARD;
    let _draw = strings::phrase_ids::DRAW;
    let _hello = strings::phrase_ids::HELLO;
}
