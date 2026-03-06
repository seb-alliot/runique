#![doc = include_str!("../../../doc-tests/i18n/switch_lang.md")]

use once_cell::sync::Lazy;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Display;
use std::sync::OnceLock;

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
}

static GLOBAL_LANG: OnceLock<Lang> = OnceLock::new();

/// Initialise la langue globale de l'application (appelé une seule fois au démarrage).
/// Les appels suivants sont ignorés.
pub fn set_lang(lang: Lang) {
    GLOBAL_LANG.set(lang).ok();
}

/// Retourne la langue globale configurée (défaut : `En`).
pub fn current_lang() -> Lang {
    *GLOBAL_LANG.get().unwrap_or(&Lang::En)
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
        match s.to_lowercase().as_str() {
            "fr" | "fr-fr" | "fr-ca" | "fr-be" | "fr-ch" => Lang::Fr,
            "en" | "en-us" | "en-gb" | "en-ca" => Lang::En,
            "it" | "it-it" | "it-ch" => Lang::It,
            "es" | "es-es" | "es-mx" | "es-ar" | "es-co" | "es-cl" => Lang::Es,
            "de" | "de-de" | "de-at" | "de-ch" => Lang::De,
            "pt" | "pt-pt" | "pt-br" => Lang::Pt,
            "ja" | "ja-jp" => Lang::Ja,
            "zh" | "zh-cn" | "zh-tw" | "zh-hk" => Lang::Zh,
            _ => Lang::En, // Default to English if unrecognized
        }
    }
}

impl From<String> for Lang {
    fn from(s: String) -> Self {
        Lang::from(s.as_str())
    }
}

impl Lang {
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
        }
    }

    /// Loads the translation JSON for this language
    fn load_json(&self) -> &'static Value {
        static FR: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("fr.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'fr.json': {}", e))
        });
        static EN: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("en.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'en.json': {}", e))
        });
        static IT: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("it.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'it.json': {}", e))
        });
        static ES: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("es.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'es.json': {}", e))
        });
        static DE: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("de.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'de.json': {}", e))
        });
        static PT: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("pt.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'pt.json': {}", e))
        });
        static JA: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("ja.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'ja.json': {}", e))
        });
        static ZH: Lazy<Value> = Lazy::new(|| {
            serde_json::from_str(include_str!("zh.json"))
                .unwrap_or_else(|e| panic!("Invalid translation file 'zh.json': {}", e))
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
