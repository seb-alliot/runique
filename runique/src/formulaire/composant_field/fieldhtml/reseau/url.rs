use crate::formulaire::field::RuniqueField;
use url::Url;

pub struct URLField {
    pub require_https: bool,
}

impl URLField {
    pub fn new() -> Self {
        Self {
            require_https: false,
        }
    }

    pub fn https_only() -> Self {
        Self {
            require_https: true,
        }
    }
}

impl Default for URLField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for URLField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let trimmed = raw_value.trim();

        // Parser l'URL
        let parsed = Url::parse(trimmed).map_err(|_| "URL invalide.".to_string())?;

        // Vérifier que c'est bien http ou https
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err("L'URL doit utiliser http:// ou https://".to_string());
        }

        // Si HTTPS requis
        if self.require_https && scheme != "https" {
            return Err("L'URL doit utiliser https:// (connexion sécurisée requise).".to_string());
        }

        // Vérifier qu'il y a bien un host
        if parsed.host_str().is_none() {
            return Err("L'URL doit contenir un nom de domaine valide.".to_string());
        }

        Ok(parsed.to_string())
    }

    fn template_name(&self) -> &str {
        "url"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "url",
            "placeholder": if self.require_https {
                "https://example.com"
            } else {
                "https://example.com ou http://example.com"
            }
        })
    }
}
