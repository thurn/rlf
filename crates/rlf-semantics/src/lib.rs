//! Shared RLF semantic tables used by both runtime and macro validation.
//!
//! This crate centralizes transform name/alias resolution to avoid drift between
//! compile-time (`rlf-macros`) and runtime (`rlf`) behavior.

/// Canonical transform identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformId {
    Cap,
    Upper,
    Lower,
    EnglishA,
    EnglishThe,
    EnglishPlural,
    GermanDer,
    GermanEin,
    DutchDe,
    DutchEen,
    SpanishEl,
    SpanishUn,
    PortugueseO,
    PortugueseUm,
    PortugueseDe,
    PortugueseEm,
    FrenchLe,
    FrenchUn,
    FrenchDe,
    FrenchAu,
    FrenchLiaison,
    ItalianIl,
    ItalianUn,
    ItalianDi,
    ItalianA,
    GreekO,
    GreekEnas,
    RomanianDef,
    ArabicAl,
    PersianEzafe,
    ChineseCount,
    JapaneseCount,
    KoreanCount,
    VietnameseCount,
    ThaiCount,
    BengaliCount,
    IndonesianPlural,
    KoreanParticle,
    TurkishInflect,
    FinnishInflect,
    HungarianInflect,
    JapaneseParticle,
    HindiKa,
    HindiKo,
    HindiSe,
    HindiMe,
    HindiPar,
    HindiNe,
}

/// Resolve a transform name for a language to a canonical transform id.
///
/// Resolution order:
/// 1. Alias canonicalization (language-aware where needed)
/// 2. Universal transforms
/// 3. Language-specific transforms
pub fn resolve_transform(name: &str, lang: &str) -> Option<TransformId> {
    let canonical = canonicalize_alias(name, lang);

    match canonical {
        "cap" => return Some(TransformId::Cap),
        "upper" => return Some(TransformId::Upper),
        "lower" => return Some(TransformId::Lower),
        _ => {}
    }

    match (lang, canonical) {
        ("en", "a") => Some(TransformId::EnglishA),
        ("en", "the") => Some(TransformId::EnglishThe),
        ("en", "plural") => Some(TransformId::EnglishPlural),
        ("de", "der") => Some(TransformId::GermanDer),
        ("de", "ein") => Some(TransformId::GermanEin),
        ("nl", "de") => Some(TransformId::DutchDe),
        ("nl", "een") => Some(TransformId::DutchEen),
        ("es", "el") => Some(TransformId::SpanishEl),
        ("es", "un") => Some(TransformId::SpanishUn),
        ("pt", "o") => Some(TransformId::PortugueseO),
        ("pt", "um") => Some(TransformId::PortugueseUm),
        ("pt", "de") => Some(TransformId::PortugueseDe),
        ("pt", "em") => Some(TransformId::PortugueseEm),
        ("fr", "le") => Some(TransformId::FrenchLe),
        ("fr", "un") => Some(TransformId::FrenchUn),
        ("fr", "de") => Some(TransformId::FrenchDe),
        ("fr", "au") => Some(TransformId::FrenchAu),
        ("fr", "liaison") => Some(TransformId::FrenchLiaison),
        ("it", "il") => Some(TransformId::ItalianIl),
        ("it", "un") => Some(TransformId::ItalianUn),
        ("it", "di") => Some(TransformId::ItalianDi),
        ("it", "a") => Some(TransformId::ItalianA),
        ("el", "o") => Some(TransformId::GreekO),
        ("el", "enas") => Some(TransformId::GreekEnas),
        ("ro", "def") => Some(TransformId::RomanianDef),
        ("ar", "al") => Some(TransformId::ArabicAl),
        ("fa", "ezafe") => Some(TransformId::PersianEzafe),
        ("zh", "count") => Some(TransformId::ChineseCount),
        ("ja", "count") => Some(TransformId::JapaneseCount),
        ("ko", "count") => Some(TransformId::KoreanCount),
        ("vi", "count") => Some(TransformId::VietnameseCount),
        ("th", "count") => Some(TransformId::ThaiCount),
        ("bn", "count") => Some(TransformId::BengaliCount),
        ("id", "plural") => Some(TransformId::IndonesianPlural),
        ("ko", "particle") => Some(TransformId::KoreanParticle),
        ("ja", "particle") => Some(TransformId::JapaneseParticle),
        ("tr", "inflect") => Some(TransformId::TurkishInflect),
        ("fi", "inflect") => Some(TransformId::FinnishInflect),
        ("hu", "inflect") => Some(TransformId::HungarianInflect),
        ("hi", "ka") => Some(TransformId::HindiKa),
        ("hi", "ko") => Some(TransformId::HindiKo),
        ("hi", "se") => Some(TransformId::HindiSe),
        ("hi", "me") => Some(TransformId::HindiMe),
        ("hi", "par") => Some(TransformId::HindiPar),
        ("hi", "ne") => Some(TransformId::HindiNe),
        _ => None,
    }
}

/// Accepted transform names for the given language, including aliases.
///
/// Used for diagnostics and typo suggestions.
pub fn accepted_transform_names(lang: &str) -> &'static [&'static str] {
    match lang {
        "en" => EN_NAMES,
        "de" => DE_NAMES,
        "nl" => NL_NAMES,
        "es" => ES_NAMES,
        "pt" => PT_NAMES,
        "fr" => FR_NAMES,
        "it" => IT_NAMES,
        "el" => EL_NAMES,
        "ro" => RO_NAMES,
        "ar" => AR_NAMES,
        "fa" => FA_NAMES,
        "zh" => ZH_NAMES,
        "ja" => JA_NAMES,
        "ko" => KO_NAMES,
        "vi" => VI_NAMES,
        "th" => TH_NAMES,
        "bn" => BN_NAMES,
        "id" => ID_NAMES,
        "tr" => TR_NAMES,
        "fi" => FI_NAMES,
        "hu" => HU_NAMES,
        "hi" => HI_NAMES,
        _ => UNIVERSAL_NAMES,
    }
}

fn canonicalize_alias<'a>(name: &'a str, lang: &str) -> &'a str {
    match (name, lang) {
        ("an", _) => "a",
        ("die" | "das", _) => "der",
        ("eine", _) => "ein",
        ("het", _) => "de",
        ("la", "es") => "el",
        ("una", "es") => "un",
        ("a", "pt") => "o",
        ("uma", _) => "um",
        ("la", "fr") => "le",
        ("une", "fr") => "un",
        ("lo" | "la", "it") => "il",
        ("uno" | "una", "it") => "un",
        ("i" | "to", "el") => "o",
        ("mia" | "ena", "el") => "enas",
        ("ki" | "ke", "hi") => "ka",
        (other, _) => other,
    }
}

const UNIVERSAL_NAMES: &[&str] = &["cap", "upper", "lower"];
const EN_NAMES: &[&str] = &["cap", "upper", "lower", "a", "an", "the", "plural"];
const DE_NAMES: &[&str] = &["cap", "upper", "lower", "der", "die", "das", "ein", "eine"];
const NL_NAMES: &[&str] = &["cap", "upper", "lower", "de", "het", "een"];
const ES_NAMES: &[&str] = &["cap", "upper", "lower", "el", "la", "un", "una"];
const PT_NAMES: &[&str] = &["cap", "upper", "lower", "o", "a", "um", "uma", "de", "em"];
const FR_NAMES: &[&str] = &[
    "cap", "upper", "lower", "le", "la", "un", "une", "de", "au", "liaison",
];
const IT_NAMES: &[&str] = &[
    "cap", "upper", "lower", "il", "lo", "la", "un", "uno", "una", "di", "a",
];
const EL_NAMES: &[&str] = &[
    "cap", "upper", "lower", "o", "i", "to", "enas", "mia", "ena",
];
const RO_NAMES: &[&str] = &["cap", "upper", "lower", "def"];
const AR_NAMES: &[&str] = &["cap", "upper", "lower", "al"];
const FA_NAMES: &[&str] = &["cap", "upper", "lower", "ezafe"];
const ZH_NAMES: &[&str] = &["cap", "upper", "lower", "count"];
const JA_NAMES: &[&str] = &["cap", "upper", "lower", "count", "particle"];
const KO_NAMES: &[&str] = &["cap", "upper", "lower", "count", "particle"];
const VI_NAMES: &[&str] = &["cap", "upper", "lower", "count"];
const TH_NAMES: &[&str] = &["cap", "upper", "lower", "count"];
const BN_NAMES: &[&str] = &["cap", "upper", "lower", "count"];
const ID_NAMES: &[&str] = &["cap", "upper", "lower", "plural"];
const TR_NAMES: &[&str] = &["cap", "upper", "lower", "inflect"];
const FI_NAMES: &[&str] = &["cap", "upper", "lower", "inflect"];
const HU_NAMES: &[&str] = &["cap", "upper", "lower", "inflect"];
const HI_NAMES: &[&str] = &[
    "cap", "upper", "lower", "ka", "ki", "ke", "ko", "se", "me", "par", "ne",
];
