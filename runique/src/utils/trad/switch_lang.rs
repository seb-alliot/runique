#![doc = include_str!("../../../doc-tests/i18n/switch_lang.md")]

use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Display;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU8, Ordering};

/// Languages supported by Runique
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Fr,
    #[default]
    En,
    It,
    Es,
    De,
    Pt,
    Ja,
    Zh,
    Ru,
}

static GLOBAL_LANG: AtomicU8 = AtomicU8::new(1); // 1 = Lang::En

/// Définit dynamiquement la langue globale de l'application.
pub fn set_lang(lang: Lang) {
    GLOBAL_LANG.store(lang.as_u8(), Ordering::Relaxed);
}

/// Retourne la langue globale configurée (défaut : `En`).
pub fn current_lang() -> Lang {
    Lang::from_u8(GLOBAL_LANG.load(Ordering::Relaxed))
}

/// Traduit une clé avec la langue globale.
pub fn t(key: &str) -> Cow<'static, str> {
    current_lang().get(key)
}

/// Traduit une clé avec paramètres avec la langue globale.
pub fn tf<T: Display>(key: &str, args: &[T]) -> String {
    current_lang().format(key, args)
}

impl From<&str> for Lang {
    fn from(s: &str) -> Self {
        // Normalize locale strings: "fr_FR.UTF-8" → "fr-fr", "fr" → "fr"
        let normalized = s
            .split('.')
            .next()
            .unwrap_or(s)
            .replace('_', "-")
            .to_lowercase();

        match normalized.as_str() {
            "fr" | "fr-fr" | "fr-ca" | "fr-be" | "fr-ch" => Lang::Fr,
            "en" | "en-us" | "en-gb" | "en-ca" => Lang::En,
            "it" | "it-it" | "it-ch" => Lang::It,
            "es" | "es-es" | "es-mx" | "es-ar" | "es-co" | "es-cl" => Lang::Es,
            "de" | "de-de" | "de-at" | "de-ch" => Lang::De,
            "pt" | "pt-pt" | "pt-br" => Lang::Pt,
            "ja" | "ja-jp" => Lang::Ja,
            "zh" | "zh-cn" | "zh-tw" | "zh-hk" => Lang::Zh,
            "ru" | "ru-ru" | "ru-by" | "ru-ua" => Lang::Ru,
            _ => Lang::En,
        }
    }
}

impl From<String> for Lang {
    fn from(s: String) -> Self {
        Lang::from(s.as_str())
    }
}

impl Lang {
    fn as_u8(self) -> u8 {
        match self {
            Lang::Fr => 0,
            Lang::En => 1,
            Lang::It => 2,
            Lang::Es => 3,
            Lang::De => 4,
            Lang::Pt => 5,
            Lang::Ja => 6,
            Lang::Zh => 7,
            Lang::Ru => 8,
        }
    }

    fn from_u8(v: u8) -> Self {
        match v {
            0 => Lang::Fr,
            1 => Lang::En,
            2 => Lang::It,
            3 => Lang::Es,
            4 => Lang::De,
            5 => Lang::Pt,
            6 => Lang::Ja,
            7 => Lang::Zh,
            8 => Lang::Ru,
            _ => Lang::En,
        }
    }

    /// Returns the language code (for file names)
    pub const fn code(&self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::It => "it",
            Lang::Es => "es",
            Lang::De => "de",
            Lang::Pt => "pt",
            Lang::Ja => "ja",
            Lang::Zh => "zh",
            Lang::Ru => "ru",
        }
    }

    /// Loads the translation JSON for this language
    fn load_json(&self) -> &'static Value {
        static FR: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("fr.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'fr.json': {}", e))
        });
        static EN: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("en.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'en.json': {}", e))
        });
        static IT: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("it.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'it.json': {}", e))
        });
        static ES: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("es.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'es.json': {}", e))
        });
        static DE: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("de.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'de.json': {}", e))
        });
        static PT: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("pt.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'pt.json': {}", e))
        });
        static JA: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("ja.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'ja.json': {}", e))
        });
        static ZH: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("zh.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'zh.json': {}", e))
        });
        static RU: LazyLock<Value> = LazyLock::new(|| {
            serde_json::from_str(include_str!("ru.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'ru.json': {}", e))
        });

        match self {
            Lang::Fr => &FR,
            Lang::En => &EN,
            Lang::It => &IT,
            Lang::Es => &ES,
            Lang::De => &DE,
            Lang::Pt => &PT,
            Lang::Ja => &JA,
            Lang::Zh => &ZH,
            Lang::Ru => &RU,
        }
    }

    /// Retrieves a translated message by its key (e.g., "forms.required")
    pub fn get(&self, key: &str) -> Cow<'static, str> {
        if let Some(s) = self.lookup(key) {
            return s;
        }
        if *self != Lang::En {
            if let Some(s) = Lang::En.lookup(key) {
                return s;
            }
        }
        Cow::Owned(key.to_string())
    }

    fn lookup(&self, key: &str) -> Option<Cow<'static, str>> {
        let json = self.load_json();
        let mut current = json;
        for part in key.split('.') {
            current = current.get(part)?;
        }
        current.as_str().map(Cow::Borrowed)
    }

    /// Retrieves a message with parameters (replaces {} with args)
    pub fn format<T: Display>(&self, key: &str, args: &[T]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();

        for arg in args {
            result = result.replacen("{}", &arg.to_string(), 1);
        }

        result
    }
}
