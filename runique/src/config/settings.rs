use crate::utils::config::{env_or_default, AutoFieldType};
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
    pub use_tz: bool,
    pub default_auto_field: AutoFieldType,
    pub redirect_anonymous: String,
    pub logging_required: String,
    pub user_connected: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let redirect_anonymous_url: String = env_or_default("REDIRECT_ANONYMOUS", "/");
        let logging_url: String = env_or_default("LOGGING_URL", "/");
        let user_connected_url: String = env_or_default("USER_CONNECTED_URL", "/");
        let project_name: String = env_or_default("PROJECT_NAME", "myproject");
        let language_app: String = env_or_default("LANGUAGE_APP", "en-us");
        let time_zone: String = env_or_default("TIME_ZONE", "UTC");
        Self {
            installed_apps: vec![],
            middleware: vec![],
            root_urlconf: format!("{}.urls", project_name),
            language_code: language_app,
            time_zone,
            use_tz: true,
            default_auto_field: AutoFieldType::from_env(),
            redirect_anonymous: redirect_anonymous_url,
            logging_required: logging_url,
            user_connected: user_connected_url,
        }
    }
}

impl AppSettings {
    pub fn from_env() -> Self {
        Self::default()
    }
}
