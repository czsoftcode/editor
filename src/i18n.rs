//! Internationalization using [Project Fluent](https://projectfluent.org/).
//!
//! Translations are embedded directly into the binary using `include_str!`.
//! Supported languages: `cs`, `en` (fallback), `sk`, `de`, `ru`.
//!
//! The language is detected automatically from system environment variables
//! (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`). If the system language
//! is not supported, English is used.
//!
//! # Usage
//!
//! ```rust
//! let i18n = I18n::new("cs");
//!
//! // Simple translation
//! let text = i18n.get("menu-file");
//!
//! // Translation with variables — tr! macro
//! let text = tr!(i18n, "about-version", version = "0.3.0");
//!
//! // Translation with variables — manually
//! let mut args = fluent_bundle::FluentArgs::new();
//! args.set("count", 3u32);
//! let text = i18n.get_args("panel-build-errors", &args);
//! ```

use fluent_bundle::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

// ---------------------------------------------------------------------------
// Supported languages and embedded resources
// ---------------------------------------------------------------------------

/// Language codes (BCP 47) for which translations exist.
pub const SUPPORTED_LANGS: &[&str] = &["cs", "en", "sk", "de", "ru"];

/// Default language used as fallback if the system language is not supported.
pub const FALLBACK_LANG: &str = "en";

const RESOURCES_CS: &[&str] = &[
    include_str!("../locales/cs/menu.ftl"),
    include_str!("../locales/cs/ui.ftl"),
    include_str!("../locales/cs/dialogs.ftl"),
    include_str!("../locales/cs/errors.ftl"),
];

const RESOURCES_EN: &[&str] = &[
    include_str!("../locales/en/menu.ftl"),
    include_str!("../locales/en/ui.ftl"),
    include_str!("../locales/en/dialogs.ftl"),
    include_str!("../locales/en/errors.ftl"),
];

const RESOURCES_SK: &[&str] = &[
    include_str!("../locales/sk/menu.ftl"),
    include_str!("../locales/sk/ui.ftl"),
    include_str!("../locales/sk/dialogs.ftl"),
    include_str!("../locales/sk/errors.ftl"),
];

const RESOURCES_DE: &[&str] = &[
    include_str!("../locales/de/menu.ftl"),
    include_str!("../locales/de/ui.ftl"),
    include_str!("../locales/de/dialogs.ftl"),
    include_str!("../locales/de/errors.ftl"),
];

const RESOURCES_RU: &[&str] = &[
    include_str!("../locales/ru/menu.ftl"),
    include_str!("../locales/ru/ui.ftl"),
    include_str!("../locales/ru/dialogs.ftl"),
    include_str!("../locales/ru/errors.ftl"),
];

pub(crate) fn resources_for(lang: &str) -> &'static [&'static str] {
    match lang {
        "cs" => RESOURCES_CS,
        "sk" => RESOURCES_SK,
        "de" => RESOURCES_DE,
        "ru" => RESOURCES_RU,
        _ => RESOURCES_EN,
    }
}

fn build_bundle(lang: &str) -> Bundle {
    let langid: LanguageIdentifier = lang
        .parse()
        .expect("i18n: invalid language code (must be BCP 47)");
    let mut bundle = Bundle::new_concurrent(vec![langid]);
    for source in resources_for(lang) {
        match FluentResource::try_new(source.to_string()) {
            Ok(res) => { bundle.add_resource(res).ok(); }
            Err((res, _)) => { bundle.add_resource(res).ok(); }
        }
    }
    bundle
}

// ---------------------------------------------------------------------------
// System language detection
// ---------------------------------------------------------------------------

/// Returns a human-readable name of the language in its own tongue.
/// Used in the ComboBox for language selection in Settings.
pub fn lang_display_name(lang: &str) -> &'static str {
    match lang {
        "cs" => "Čeština",
        "en" => "English",
        "sk" => "Slovenčina",
        "de" => "Deutsch",
        "ru" => "Русский",
        _ => "Unknown",
    }
}

/// Returns the language code detected from system environment variables.
/// If the system language is not in [`SUPPORTED_LANGS`], returns [`FALLBACK_LANG`].
///
/// Variables are tried in order: `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`.
/// `LANGUAGE` can be a list separated by `:` (e.g., `cs:en`) — they are tried sequentially.
pub fn detect_system_lang() -> String {
    // LANGUAGE can contain multiple languages separated by a colon
    if let Ok(val) = std::env::var("LANGUAGE") {
        for part in val.split(':') {
            if let Some(lang) = extract_lang_code(part) {
                if SUPPORTED_LANGS.contains(&lang.as_str()) {
                    return lang;
                }
            }
        }
    }

    for var in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            if let Some(lang) = extract_lang_code(&val) {
                if SUPPORTED_LANGS.contains(&lang.as_str()) {
                    return lang;
                }
            }
        }
    }

    FALLBACK_LANG.to_string()
}

/// Extracts a 2-3 letter language code from a locale string.
/// Examples: `"cs_CZ.UTF-8"` → `"cs"`, `"en_US"` → `"en"`, `"zh_Hant"` → `"zh"`.
fn extract_lang_code(locale: &str) -> Option<String> {
    let code = locale.split(['_', '.', '@']).next()?;
    if (2..=3).contains(&code.len()) && code.chars().all(|c| c.is_ascii_alphabetic()) {
        Some(code.to_lowercase())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// I18n — Wrapper over FluentBundle (concurrent = Send + Sync)
// ---------------------------------------------------------------------------

/// Bundle type: `concurrent` variant is `Send + Sync`, required for `Arc<Mutex<AppShared>>`.
type Bundle = fluent_bundle::concurrent::FluentBundle<FluentResource>;

/// Access to translated strings.
///
/// Instance is `Send + Sync` — can be safely shared via `Arc`.
///
/// If a key is missing in the primary language, the English fallback bundle is used.
/// Only if the key is missing in English as well, the key itself is returned.
pub struct I18n {
    bundle: Bundle,
    /// English fallback bundle; `None` if the primary language is English.
    fallback: Option<Bundle>,
    lang: String,
}

impl I18n {
    /// Creates an instance for the given language code.
    /// Unsupported codes silently revert to [`FALLBACK_LANG`].
    pub fn new(lang: &str) -> Self {
        let effective = if SUPPORTED_LANGS.contains(&lang) {
            lang
        } else {
            FALLBACK_LANG
        };

        let bundle = build_bundle(effective);
        let fallback = if effective != FALLBACK_LANG {
            Some(build_bundle(FALLBACK_LANG))
        } else {
            None
        };

        Self {
            bundle,
            fallback,
            lang: effective.to_string(),
        }
    }

    /// Detects the system language and creates a corresponding instance.
    pub fn from_system() -> Self {
        Self::new(&detect_system_lang())
    }

    /// Translates a key without arguments.
    /// If the key does not exist, returns the key itself — the error is thus visible in the UI.
    pub fn get(&self, key: &str) -> String {
        self.get_opt_args(key, None)
    }

    /// Translates a key with named arguments (variables in FTL: `{ $name }`).
    pub fn get_args<'a>(&self, key: &str, args: &'a FluentArgs<'a>) -> String {
        self.get_opt_args(key, Some(args))
    }

    /// Active language code (BCP 47), e.g., `"cs"` or `"en"`.
    pub fn lang(&self) -> &str {
        &self.lang
    }

    // --- internal ---

    fn get_opt_args<'a>(&self, key: &str, args: Option<&'a FluentArgs<'a>>) -> String {
        // 1. Try primary bundle
        if let Some(s) = Self::format_in(&self.bundle, key, args) {
            return s;
        }
        // 2. Try English fallback bundle
        if let Some(fb) = &self.fallback {
            if let Some(s) = Self::format_in(fb, key, args) {
                return s;
            }
        }
        // 3. Return the key itself — visible indication of a missing translation
        key.to_string()
    }

    /// Attempts to translate `key` in the given bundle. Returns `None` if the key or
    /// its value (pattern) does not exist.
    fn format_in<'a>(bundle: &Bundle, key: &str, args: Option<&'a FluentArgs<'a>>) -> Option<String> {
        let msg = bundle.get_message(key)?;
        let pattern = msg.value()?;
        let mut errors = Vec::new();
        Some(bundle.format_pattern(pattern, args, &mut errors).to_string())
    }
}

impl Default for I18n {
    /// Default instance: language detected from the system.
    fn default() -> Self {
        Self::from_system()
    }
}

// ---------------------------------------------------------------------------
// Macro tr! — syntactic sugar for translation
// ---------------------------------------------------------------------------

/// Translates a key using the given `I18n` instance.
///
/// # Examples
///
/// ```rust
/// // without arguments
/// let text = tr!(i18n, "menu-file");
///
/// // with named arguments
/// let text = tr!(i18n, "about-version", version = "0.3.0");
/// let text = tr!(i18n, "panel-build-errors", count = 5u64);
/// ```
#[macro_export]
macro_rules! tr {
    ($i18n:expr, $key:expr) => {
        $i18n.get($key)
    };
    ($i18n:expr, $key:expr, $($name:ident = $val:expr),+ $(,)?) => {{
        let mut args = fluent_bundle::FluentArgs::new();
        $(args.set(stringify!($name), $val);)+
        $i18n.get_args($key, &args)
    }};
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    /// Extracts message identifiers from an FTL source.
    /// Includes only lines of the form `ident = ...` (no terms `-ident`, no attributes `.attr`).
    fn ftl_keys(source: &str) -> HashSet<String> {
        let mut keys = HashSet::new();
        for line in source.lines() {
            // Skip empty lines, comments, and continuation lines (indented)
            if line.is_empty() || line.starts_with('#') || line.starts_with(' ') || line.starts_with('\t') {
                continue;
            }
            let Some(eq) = line.find('=') else { continue };
            let key = line[..eq].trim();
            // Accept only valid message identifiers (no terms starting with `-`)
            if !key.is_empty()
                && !key.starts_with('-')
                && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                keys.insert(key.to_string());
            }
        }
        keys
    }

    /// Collects all keys from the set of FTL files for a given language.
    fn all_keys_for(lang: &str) -> HashSet<String> {
        resources_for(lang)
            .iter()
            .flat_map(|src| ftl_keys(src))
            .collect()
    }

    #[test]
    fn supported_langs_load_without_panic() {
        for lang in SUPPORTED_LANGS {
            let i18n = I18n::new(lang);
            // Basic keys must exist in all languages
            assert_ne!(i18n.get("menu-file"), "menu-file", "missing key menu-file in '{lang}'");
            assert_ne!(i18n.get("panel-files"), "panel-files", "missing key panel-files in '{lang}'");
        }
    }

    #[test]
    fn fallback_for_unknown_lang() {
        let i18n = I18n::new("xx");
        assert_eq!(i18n.lang(), FALLBACK_LANG);
    }

    #[test]
    fn missing_key_returns_key_itself() {
        // Key that does not exist in primary or fallback bundle
        let i18n = I18n::new("en");
        assert_eq!(i18n.get("nonexistent-key-xyz"), "nonexistent-key-xyz");
    }

    #[test]
    fn missing_key_falls_back_to_english() {
        // Tests that I18n for another language indeed returns the EN fallback.
        let cs = I18n::new("cs");
        // "panel-files" exists in cs → returns Czech value, not the key
        assert_ne!(cs.get("panel-files"), "panel-files");
        // Fallback bundle is present for non-English languages
        assert!(cs.fallback.is_some(), "cs I18n must have a fallback bundle");
        // EN instance does not have a fallback bundle (it is the fallback)
        let en = I18n::new("en");
        assert!(en.fallback.is_none(), "en I18n should not have a fallback bundle");
    }

    #[test]
    fn all_lang_keys_match_english() {
        let en_keys = all_keys_for("en");
        assert!(!en_keys.is_empty(), "English FTL files must contain keys");

        for &lang in SUPPORTED_LANGS {
            if lang == FALLBACK_LANG {
                continue;
            }
            let lang_keys = all_keys_for(lang);

            // All English keys must also be in this language
            let mut missing: Vec<&str> = en_keys
                .iter()
                .filter(|k| !lang_keys.contains(*k))
                .map(String::as_str)
                .collect();
            missing.sort();
            assert!(
                missing.is_empty(),
                "Language '{lang}' lacks keys compared to EN:\n  {}",
                missing.join("\n  ")
            );

            // This language must not have extra keys that are not in English
            let mut extra: Vec<&str> = lang_keys
                .iter()
                .filter(|k| !en_keys.contains(*k))
                .map(String::as_str)
                .collect();
            extra.sort();
            assert!(
                extra.is_empty(),
                "Language '{lang}' has extra keys compared to EN:\n  {}",
                extra.join("\n  ")
            );
        }
    }

    #[test]
    fn extract_lang_code_variants() {
        assert_eq!(extract_lang_code("cs_CZ.UTF-8"), Some("cs".into()));
        assert_eq!(extract_lang_code("en_US.UTF-8"), Some("en".into()));
        assert_eq!(extract_lang_code("cs"), Some("cs".into()));
        assert_eq!(extract_lang_code("C"), None);
        assert_eq!(extract_lang_code(""), None);
    }

    #[test]
    fn i18n_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<I18n>();
    }
}
