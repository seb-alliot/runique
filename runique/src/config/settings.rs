use serde::{Deserialize, Serialize};
/// Vision globale des paramètres de l'application
/// Contiendra tous les paramètres globaux de l'application
/// pour le moent , chaque module gère ses propres paramètres
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub installed_apps: Vec<String>,
    pub middleware: Vec<String>,
    pub root_urlconf: String,
    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
    pub auth_password_validators: Vec<String>,
    pub password_hashers: Vec<String>,
    pub default_auto_field: String,
    pub logging_config: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let login_redirect = std::env::var("REDIRECT_ANONYMOUS").unwrap_or("/".to_string());
        Self {
            installed_apps: vec![],
            middleware: vec![],
            root_urlconf: "project.urls".to_string(),
            language_code: "en-us".to_string(),
            time_zone: "UTC".to_string(),
            use_i18n: true,
            use_tz: true,
            auth_password_validators: vec![],
            password_hashers: vec![],
            default_auto_field: "runique.db.models.AutoField".to_string(),
            logging_config: login_redirect
        }
    }
}

impl AppSettings {
    pub fn from_env() -> Self {
        Self::default()
    }
}
