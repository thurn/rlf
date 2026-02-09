//! Regression tests for macro/file parser parity on template escapes.

#[cfg(not(feature = "global-locale"))]
use rlf::Locale;
use rlf::rlf;

mod strings {
    use super::rlf;

    rlf! {
        escaped_quote = "He said \"hi\".";
        escaped_unicode = "\u{25CF}";
    }
}

#[test]
fn register_source_phrases_accepts_macro_escaped_quote_template() {
    #[cfg(not(feature = "global-locale"))]
    let mut locale = Locale::new();

    #[cfg(not(feature = "global-locale"))]
    strings::register_source_phrases(&mut locale);
    #[cfg(feature = "global-locale")]
    strings::register_source_phrases();

    #[cfg(not(feature = "global-locale"))]
    let phrase = strings::escaped_quote(&locale);
    #[cfg(feature = "global-locale")]
    let phrase = strings::escaped_quote();

    assert_eq!(phrase.to_string(), "He said \"hi\".");
}

#[test]
fn register_source_phrases_accepts_macro_unicode_template() {
    #[cfg(not(feature = "global-locale"))]
    let mut locale = Locale::new();

    #[cfg(not(feature = "global-locale"))]
    strings::register_source_phrases(&mut locale);
    #[cfg(feature = "global-locale")]
    strings::register_source_phrases();

    #[cfg(not(feature = "global-locale"))]
    let phrase = strings::escaped_unicode(&locale);
    #[cfg(feature = "global-locale")]
    let phrase = strings::escaped_unicode();

    assert_eq!(phrase.to_string(), "●");
}
