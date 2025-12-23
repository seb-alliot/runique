use std::vec;
use serde::{Deserialize, Serialize};


/// Configuration principale de l'application Rusti
///
/// Structure inspirée de settings.py de Django
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub base_dir: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    pub installed_apps: Vec<String>,
    pub middleware: Vec<String>,
    pub root_urlconf: String,

    // Rusti-specific settings
    pub static_rusti_path: String,
    pub static_rusti_url: String,
    pub media_rusti_path: String,
    pub media_rusti_url: String,
    pub templates_rusti: String,

    // Settings new-project
    pub templates_dir: Vec<String>,
    pub staticfiles_dirs: String,
    pub media_root: String,
    pub static_url: String,
    pub media_url: String,

    pub staticfiles_storage: String,

    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
    pub auth_password_validators: Vec<String>,
    pub password_hashers: Vec<String>,
    pub default_auto_field: String,
    pub logging_config: String,

    // Security settings can be added here
    pub sanitize_inputs: bool,
    pub strict_csp: bool,
    pub rate_limiting: bool,
    pub enforce_https: bool,
}

/// Configuration du serveur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}



impl ServerSettings {
    pub fn from_env() -> Self {
        use dotenvy::dotenv;
        use std::env;

        dotenv().ok();
        let ip = env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let domain_server = format!("{}:{}", ip, port);
        let secret_key = env::var("SECRET_KEY").unwrap_or_else(|_| "default_secret_key".to_string());

        ServerSettings {
            ip_server: ip,
            domain_server,
            port: port.parse().unwrap_or(3000),
            secret_key: secret_key.to_string(),
        }
    }
}


impl Settings {
    /// Crée une configuration avec valeurs par défaut
    ///
    /// # Exemple
    /// ```rust
    /// use rusti::Settings;
    ///
    /// let settings = Settings::default_values();
    /// ```
    pub fn default_values() -> Self {
        let base_dir = ".".to_string();

        Settings {
            server: ServerSettings::from_env(),
            base_dir,
            debug: cfg!(debug_assertions),
            allowed_hosts: vec![
                String::from("localhost"),
                String::from("127.0.0.1")
            ],
            installed_apps: vec![],
            middleware: vec![],
            root_urlconf: String::from("urls"),

            // Rusti-specific settings
            templates_rusti: String::new(),
            static_rusti_path: "../../rusti/static".to_string(),     // Chemin physique
            static_rusti_url: "/rusti/static".to_string(),     // URL
            media_rusti_path: "../../rusti/media".to_string(),       // Chemin physique
            media_rusti_url: "/rusti/media".to_string(),

            // Settings pour le projet utilisateur
            templates_dir: vec!["src/templates".to_string()],
            staticfiles_dirs: "src/static".to_string(),
            media_root: "src/media".to_string(),
            static_url: "/static".to_string(),
            media_url: "/media".to_string(),

            staticfiles_storage: String::from("DefaultStaticFilesStorage"),
            language_code: String::from("en-us"),
            time_zone: String::from("UTC"),
            use_i18n: true,
            use_tz: true,
            auth_password_validators: vec![],
            password_hashers: vec![],
            default_auto_field: String::from("BigAutoField"),
            logging_config: String::from("default"),

            sanitize_inputs: true,
            strict_csp: true,
            rate_limiting: true,
            enforce_https: !cfg!(debug_assertions),
        }
    }

    /// Charge la configuration depuis des variables d'environnement
    pub fn from_env() -> Self {
        Self::default_values()
    }

    /// Builder pattern pour personnaliser la configuration
    pub fn builder() -> SettingsBuilder {
        SettingsBuilder::new()
    }
}

/// Builder pour créer des Settings personnalisés
pub struct SettingsBuilder {
    settings: Settings,
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Self {
            settings: Settings::default_values(),
        }
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.settings.debug = debug;
        self
    }

    pub fn static_rusti_path(mut self, path: impl Into<String>) -> Self {
        self.settings.static_rusti_path = path.into();
        self
    }

    pub fn static_rusti_url(mut self, url: impl Into<String>) -> Self {
        self.settings.static_rusti_url = url.into();
        self
    }

    pub fn media_rusti_path(mut self, path: impl Into<String>) -> Self {
        self.settings.media_rusti_path = path.into();
        self
    }

    pub fn media_rusti_url(mut self, url: impl Into<String>) -> Self {
        self.settings.media_rusti_url = url.into();
        self
    }

    pub fn templates_rusti(mut self, dir: impl Into<String>) -> Self {
        self.settings.templates_rusti = dir.into();
        self
    }

    pub fn templates_dir(mut self, dir: impl Into<Vec<std::string::String>>) -> Self {
        self.settings.templates_dir = dir.into();
        self
    }

    pub fn staticfiles_dirs(mut self, dir: impl Into<String>) -> Self {
        self.settings.staticfiles_dirs = dir.into();
        self
    }

    pub fn media_root(mut self, dir: impl Into<String>) -> Self {
        self.settings.media_root = dir.into();
        self
    }

    pub fn static_url(mut self, url: impl Into<String>) -> Self {
        self.settings.static_url = url.into();
        self
    }

    pub fn media_url(mut self, url: impl Into<String>) -> Self {
        self.settings.media_url = url.into();
        self
    }

    pub fn sanitize_inputs(mut self, enabled: bool) -> Self {
        self.settings.sanitize_inputs = enabled;
        self
    }

    pub fn strict_csp(mut self, enabled: bool) -> Self {
        self.settings.strict_csp = enabled;
        self
    }

    pub fn rate_limiting(mut self, enabled: bool) -> Self {
        self.settings.rate_limiting = enabled;
        self
    }

    pub fn enforce_https(mut self, enabled: bool) -> Self {
        self.settings.enforce_https = enabled;
        self
    }

    pub fn server(mut self, ip: impl Into<String>, port: u16, secret_key: impl Into<String>) -> Self {
        let ip = ip.into();
        self.settings.server = ServerSettings {
            ip_server: ip.clone(),
            domain_server: format!("{}:{}", ip, port),
            port,
            secret_key: secret_key.into(),
        };
        self
    }

    pub fn build(self) -> Settings {
        self.settings
    }
}

impl Default for SettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
