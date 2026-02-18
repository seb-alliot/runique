use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Display;

/// Languages supported by Runique
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Fr,
    #[default]
    En,
}

impl Lang {
    /// Returns the language code (for file names)
    const fn code(&self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
        }
    }

    /// Loads the translation JSON for this language
    fn load_json(&self) -> &'static Value {
        use once_cell::sync::Lazy;

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
                None => return Cow::Owned(key.to_string()), // Fallback: returns the key
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
