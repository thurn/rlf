use std::collections::HashSet;

use rlf_semantics::{TransformId, accepted_transform_names, resolve_transform};

#[test]
fn alias_resolution_matrix() {
    assert_eq!(resolve_transform("an", "en"), Some(TransformId::EnglishA));
    assert_eq!(resolve_transform("an", "pt"), None);
    assert_eq!(resolve_transform("die", "de"), Some(TransformId::GermanDer));
    assert_eq!(resolve_transform("das", "de"), Some(TransformId::GermanDer));
    assert_eq!(
        resolve_transform("eine", "de"),
        Some(TransformId::GermanEin)
    );
    assert_eq!(resolve_transform("het", "nl"), Some(TransformId::DutchDe));
    assert_eq!(resolve_transform("la", "es"), Some(TransformId::SpanishEl));
    assert_eq!(resolve_transform("una", "es"), Some(TransformId::SpanishUn));
    assert_eq!(resolve_transform("a", "pt"), Some(TransformId::PortugueseO));
    assert_eq!(
        resolve_transform("uma", "pt"),
        Some(TransformId::PortugueseUm)
    );
    assert_eq!(resolve_transform("la", "fr"), Some(TransformId::FrenchLe));
    assert_eq!(resolve_transform("une", "fr"), Some(TransformId::FrenchUn));
    assert_eq!(resolve_transform("lo", "it"), Some(TransformId::ItalianIl));
    assert_eq!(resolve_transform("uno", "it"), Some(TransformId::ItalianUn));
    assert_eq!(resolve_transform("i", "el"), Some(TransformId::GreekO));
    assert_eq!(resolve_transform("mia", "el"), Some(TransformId::GreekEnas));
    assert_eq!(resolve_transform("ki", "hi"), Some(TransformId::HindiKa));
    assert_eq!(resolve_transform("ke", "hi"), Some(TransformId::HindiKa));
}

#[test]
fn canonical_resolution_covers_all_transform_ids() {
    let cases = [
        ("en", "cap", TransformId::Cap),
        ("en", "upper", TransformId::Upper),
        ("en", "lower", TransformId::Lower),
        ("en", "a", TransformId::EnglishA),
        ("en", "the", TransformId::EnglishThe),
        ("en", "plural", TransformId::EnglishPlural),
        ("de", "der", TransformId::GermanDer),
        ("de", "ein", TransformId::GermanEin),
        ("nl", "de", TransformId::DutchDe),
        ("nl", "een", TransformId::DutchEen),
        ("es", "el", TransformId::SpanishEl),
        ("es", "un", TransformId::SpanishUn),
        ("pt", "o", TransformId::PortugueseO),
        ("pt", "um", TransformId::PortugueseUm),
        ("pt", "de", TransformId::PortugueseDe),
        ("pt", "em", TransformId::PortugueseEm),
        ("fr", "le", TransformId::FrenchLe),
        ("fr", "un", TransformId::FrenchUn),
        ("fr", "de", TransformId::FrenchDe),
        ("fr", "au", TransformId::FrenchAu),
        ("fr", "liaison", TransformId::FrenchLiaison),
        ("it", "il", TransformId::ItalianIl),
        ("it", "un", TransformId::ItalianUn),
        ("it", "di", TransformId::ItalianDi),
        ("it", "a", TransformId::ItalianA),
        ("el", "o", TransformId::GreekO),
        ("el", "enas", TransformId::GreekEnas),
        ("ro", "def", TransformId::RomanianDef),
        ("ar", "al", TransformId::ArabicAl),
        ("fa", "ezafe", TransformId::PersianEzafe),
        ("zh", "count", TransformId::ChineseCount),
        ("ja", "count", TransformId::JapaneseCount),
        ("ko", "count", TransformId::KoreanCount),
        ("vi", "count", TransformId::VietnameseCount),
        ("th", "count", TransformId::ThaiCount),
        ("bn", "count", TransformId::BengaliCount),
        ("id", "plural", TransformId::IndonesianPlural),
        ("ko", "particle", TransformId::KoreanParticle),
        ("tr", "inflect", TransformId::TurkishInflect),
        ("fi", "inflect", TransformId::FinnishInflect),
        ("hu", "inflect", TransformId::HungarianInflect),
        ("ja", "particle", TransformId::JapaneseParticle),
        ("hi", "ka", TransformId::HindiKa),
        ("hi", "ko", TransformId::HindiKo),
        ("hi", "se", TransformId::HindiSe),
        ("hi", "me", TransformId::HindiMe),
        ("hi", "par", TransformId::HindiPar),
        ("hi", "ne", TransformId::HindiNe),
    ];

    let mut covered = HashSet::new();
    for (lang, name, expected) in cases {
        let resolved = resolve_transform(name, lang);
        assert_eq!(
            resolved,
            Some(expected),
            "expected {name} in {lang} to resolve to {expected:?}"
        );
        covered.insert(expected);
    }

    assert_eq!(covered.len(), 48);
}

#[test]
fn accepted_name_discovery_includes_english_alias_and_plural() {
    let english = accepted_transform_names("en");
    assert!(english.contains(&"an"));
    assert!(english.contains(&"plural"));
    assert!(english.contains(&"the"));
}

#[test]
fn unknown_language_accepts_universal_only() {
    let universal = &["cap", "upper", "lower"];
    assert_eq!(accepted_transform_names("xx"), universal);
    assert_eq!(resolve_transform("cap", "xx"), Some(TransformId::Cap));
    assert_eq!(resolve_transform("a", "xx"), None);
}
