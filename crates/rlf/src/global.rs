//! Global locale storage for the `global-locale` feature.
//!
//! Provides thread-safe access to a shared `Locale` instance, removing the need
//! to pass `&Locale` to every generated phrase function.

use std::sync::{LazyLock, RwLock};

use crate::Locale;

static GLOBAL_LOCALE: LazyLock<RwLock<Locale>> = LazyLock::new(|| RwLock::new(Locale::new()));

/// Provides read access to the global locale.
pub fn with_locale<T>(f: impl FnOnce(&Locale) -> T) -> T {
    let guard = GLOBAL_LOCALE.read().expect("global locale lock poisoned");
    f(&guard)
}

/// Provides write access to the global locale.
pub fn with_locale_mut<T>(f: impl FnOnce(&mut Locale) -> T) -> T {
    let mut guard = GLOBAL_LOCALE.write().expect("global locale lock poisoned");
    f(&mut guard)
}

/// Sets the current language for the global locale.
pub fn set_language(language: impl Into<String>) {
    with_locale_mut(|locale| locale.set_language(language));
}

/// Returns the current language of the global locale.
pub fn language() -> String {
    with_locale(|locale| locale.language().to_owned())
}
