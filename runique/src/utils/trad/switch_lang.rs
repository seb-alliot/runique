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
            _ => Lang::En,
        }
    }
}

impl From<String> for Lang {
    fn from(s: String) -> Self {
        Lang::from(s.as_str())
    }
}

#[allow(dead_code)]
impl Lang {
    /// Returns the language code (for file names)
    pub const fn code(&self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
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

        match self {
            Lang::Fr => &FR,
            Lang::En => &EN,
        }
    }

    /// Retrieves a translated message by its key (e.g., "forms.required")
    pub fn get(&self, key: &str) -> Cow<'static, str> {
        let json = self.load_json();
        let parts: Vec<&str> = key.split('.').collect();

        let mut current = json;
        for part in parts {
            match current.get(part) {
                Some(val) => current = val,
                None => return Cow::Owned(key.to_string()),
            }
        }

        match current.as_str() {
            Some(s) => Cow::Borrowed(s),
            None => Cow::Owned(key.to_string()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_simple() {
        let fr = Lang::Fr;
        assert_eq!(fr.get("forms.required"), "Ce champ est obligatoire");

        let en = Lang::En;
        assert_eq!(en.get("forms.required"), "This field is required");
    }

    #[test]
    fn test_get_nested() {
        let fr = Lang::Fr;
        assert_eq!(fr.get("error.title.not_found"), "Page non trouvée");
    }

    #[test]
    fn test_format() {
        let fr = Lang::Fr;
        assert_eq!(fr.format("forms.too_short", &[5]), "Trop court (min 5)");
    }

    #[test]
    fn test_missing_key() {
        let fr = Lang::Fr;
        assert_eq!(fr.get("missing.key"), "missing.key");
    }
}
