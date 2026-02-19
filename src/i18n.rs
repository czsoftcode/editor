//! Internacionalizace pomocí [Project Fluent](https://projectfluent.org/).
//!
//! Překlady jsou embedovány přímo do binárky pomocí `include_str!`.
//! Podporované jazyky: `cs`, `en` (fallback).
//!
//! Jazyk se detekuje automaticky ze systémových proměnných prostředí
//! (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`). Není-li systémový jazyk
//! podporován, použije se angličtina.
//!
//! # Použití
//!
//! ```rust
//! let i18n = I18n::new("cs");
//!
//! // Jednoduchý překlad
//! let text = i18n.get("menu-file");
//!
//! // Překlad s proměnnými — makro tr!
//! let text = tr!(i18n, "about-version", version = "0.3.0");
//!
//! // Překlad s proměnnými — manuálně
//! let mut args = fluent_bundle::FluentArgs::new();
//! args.set("count", 3u32);
//! let text = i18n.get_args("panel-build-errors", &args);
//! ```

use fluent_bundle::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

// ---------------------------------------------------------------------------
// Podporované jazyky a embedované zdroje
// ---------------------------------------------------------------------------

/// Jazykové kódy (BCP 47) pro které existují překlady.
pub const SUPPORTED_LANGS: &[&str] = &["cs", "en", "sk", "de", "ru"];

/// Výchozí jazyk použitý jako fallback, pokud systémový jazyk není podporován.
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
        .expect("i18n: neplatný kód jazyka (musí být BCP 47)");
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
// Detekce systémového jazyka
// ---------------------------------------------------------------------------

/// Vrátí lidsky čitelný název jazyka v jeho vlastní řeči.
/// Používá se v ComboBoxu pro výběr jazyka v Nastavení.
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

/// Vrátí kód jazyka detekovaný ze systémových proměnných prostředí.
/// Není-li systémový jazyk v [`SUPPORTED_LANGS`], vrátí [`FALLBACK_LANG`].
///
/// Proměnné jsou zkoušeny v pořadí: `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`.
/// `LANGUAGE` může být seznam oddělený `:` (např. `cs:en`) — zkouší se postupně.
pub fn detect_system_lang() -> String {
    // LANGUAGE může obsahovat více jazyků oddělených dvojtečkou
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

/// Extrahuje 2-3 písmenný kód jazyka z locale řetězce.
/// Příklady: `"cs_CZ.UTF-8"` → `"cs"`, `"en_US"` → `"en"`, `"zh_Hant"` → `"zh"`.
fn extract_lang_code(locale: &str) -> Option<String> {
    let code = locale.split(['_', '.', '@']).next()?;
    if (2..=3).contains(&code.len()) && code.chars().all(|c| c.is_ascii_alphabetic()) {
        Some(code.to_lowercase())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// I18n — obálka nad FluentBundle (concurrent = Send + Sync)
// ---------------------------------------------------------------------------

/// Typ bundlu: `concurrent` varianta je `Send + Sync`, nutné pro `Arc<Mutex<AppShared>>`.
type Bundle = fluent_bundle::concurrent::FluentBundle<FluentResource>;

/// Přístup k přeloženým řetězcům.
///
/// Instance je `Send + Sync` — lze ji bezpečně sdílet přes `Arc`.
///
/// Pokud klíč chybí v primárním jazyce, použije se anglický fallback bundle.
/// Teprve pokud klíč chybí i v angličtině, vrátí se klíč samotný.
pub struct I18n {
    bundle: Bundle,
    /// Anglický fallback bundle; `None` pokud je primárním jazykem angličtina.
    fallback: Option<Bundle>,
    lang: String,
}

impl I18n {
    /// Vytvoří instanci pro daný kód jazyka.
    /// Nepodporovaný kód se tiše vrátí na [`FALLBACK_LANG`].
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

    /// Detekuje systémový jazyk a vytvoří odpovídající instanci.
    pub fn from_system() -> Self {
        Self::new(&detect_system_lang())
    }

    /// Přeloží klíč bez argumentů.
    /// Pokud klíč neexistuje, vrátí klíč samotný — chyba je tak viditelná v UI.
    pub fn get(&self, key: &str) -> String {
        self.get_opt_args(key, None)
    }

    /// Přeloží klíč s pojmenovanými argumenty (proměnné ve FTL: `{ $name }`).
    pub fn get_args<'a>(&self, key: &str, args: &'a FluentArgs<'a>) -> String {
        self.get_opt_args(key, Some(args))
    }

    /// Kód aktivního jazyka (BCP 47), např. `"cs"` nebo `"en"`.
    pub fn lang(&self) -> &str {
        &self.lang
    }

    // --- interní ---

    fn get_opt_args<'a>(&self, key: &str, args: Option<&'a FluentArgs<'a>>) -> String {
        // 1. Zkus primární bundle
        if let Some(s) = Self::format_in(&self.bundle, key, args) {
            return s;
        }
        // 2. Zkus anglický fallback bundle
        if let Some(fb) = &self.fallback {
            if let Some(s) = Self::format_in(fb, key, args) {
                return s;
            }
        }
        // 3. Vrátit klíč samotný — viditelná indikace chybějícího překladu
        key.to_string()
    }

    /// Pokusí se přeložit `key` v daném bundlu. Vrátí `None` pokud klíč nebo
    /// jeho hodnota (pattern) neexistuje.
    fn format_in<'a>(bundle: &Bundle, key: &str, args: Option<&'a FluentArgs<'a>>) -> Option<String> {
        let msg = bundle.get_message(key)?;
        let pattern = msg.value()?;
        let mut errors = Vec::new();
        Some(bundle.format_pattern(pattern, args, &mut errors).to_string())
    }
}

impl Default for I18n {
    /// Výchozí instance: jazyk detekovaný ze systému.
    fn default() -> Self {
        Self::from_system()
    }
}

// ---------------------------------------------------------------------------
// Makro tr! — syntaktický cukr pro překlad
// ---------------------------------------------------------------------------

/// Přeloží klíč pomocí dané instance `I18n`.
///
/// # Příklady
///
/// ```rust
/// // bez argumentů
/// let text = tr!(i18n, "menu-file");
///
/// // s pojmenovanými argumenty
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
// Testy
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    /// Extrahuje identifikátory zpráv z FTL zdroje.
    /// Zahrnuje pouze řádky tvaru `ident = ...` (ne termy `-ident`, ne atributy `.attr`).
    fn ftl_keys(source: &str) -> HashSet<String> {
        let mut keys = HashSet::new();
        for line in source.lines() {
            // Přeskočit prázdné řádky, komentáře a continuation řádky (odsazené)
            if line.is_empty() || line.starts_with('#') || line.starts_with(' ') || line.starts_with('\t') {
                continue;
            }
            let Some(eq) = line.find('=') else { continue };
            let key = line[..eq].trim();
            // Přijmout pouze platné identifikátory zpráv (ne termy začínající `-`)
            if !key.is_empty()
                && !key.starts_with('-')
                && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                keys.insert(key.to_string());
            }
        }
        keys
    }

    /// Sbírá všechny klíče ze sady FTL souborů daného jazyka.
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
            // Základní klíče musí existovat ve všech jazycích
            assert_ne!(i18n.get("menu-file"), "menu-file", "chybí klíč menu-file v '{lang}'");
            assert_ne!(i18n.get("panel-files"), "panel-files", "chybí klíč panel-files v '{lang}'");
        }
    }

    #[test]
    fn fallback_for_unknown_lang() {
        let i18n = I18n::new("xx");
        assert_eq!(i18n.lang(), FALLBACK_LANG);
    }

    #[test]
    fn missing_key_returns_key_itself() {
        // Klíč, který neexistuje ani v primárním, ani ve fallback bundlu
        let i18n = I18n::new("en");
        assert_eq!(i18n.get("neexistujici-klic-xyz"), "neexistujici-klic-xyz");
    }

    #[test]
    fn missing_key_falls_back_to_english() {
        // Otestuje, že I18n pro jiný jazyk skutečně vrátí EN fallback.
        // Použijeme klíč z EN bundlu — pokud by cs bundle měl fallback vypnutý,
        // vrátil by klíč samotný. Tady ověřujeme, že I18n::new("cs") má fallback bundle.
        let cs = I18n::new("cs");
        // "panel-files" existuje v cs → vrátí českou hodnotu, ne klíč
        assert_ne!(cs.get("panel-files"), "panel-files");
        // Fallback bundle je přítomný pro neanglické jazyky
        assert!(cs.fallback.is_some(), "cs I18n musí mít fallback bundle");
        // EN instance nemá fallback bundle (sama je fallback)
        let en = I18n::new("en");
        assert!(en.fallback.is_none(), "en I18n nemá mít fallback bundle");
    }

    #[test]
    fn all_lang_keys_match_english() {
        let en_keys = all_keys_for("en");
        assert!(!en_keys.is_empty(), "anglické FTL soubory musí obsahovat klíče");

        for &lang in SUPPORTED_LANGS {
            if lang == FALLBACK_LANG {
                continue;
            }
            let lang_keys = all_keys_for(lang);

            // Všechny anglické klíče musí být i v tomto jazyce
            let mut missing: Vec<&str> = en_keys
                .iter()
                .filter(|k| !lang_keys.contains(*k))
                .map(String::as_str)
                .collect();
            missing.sort();
            assert!(
                missing.is_empty(),
                "Jazyk '{lang}' postrádá klíče oproti EN:\n  {}",
                missing.join("\n  ")
            );

            // Tento jazyk nesmí mít extra klíče, které nejsou v angličtině
            let mut extra: Vec<&str> = lang_keys
                .iter()
                .filter(|k| !en_keys.contains(*k))
                .map(String::as_str)
                .collect();
            extra.sort();
            assert!(
                extra.is_empty(),
                "Jazyk '{lang}' má navíc klíče oproti EN:\n  {}",
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
