use std::vec;

use serde::{Deserialize, Serialize};

/// Configuration principale de l'application Rusti
///
/// Structure inspirée de settings.py de Django
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub base_dir: String,
    pub secret_key: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    pub installed_apps: Vec<String>,
    pub middleware: Vec<String>,
    pub root_urlconf: String,

    #[cfg(feature = "orm")]
    pub databases: DatabaseSettings,

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
}

/// Configuration du serveur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
}

impl ServerSettings {
    pub fn from_env() -> Self {
        use dotenvy::dotenv;
        use std::env;

        dotenv().ok();
        let ip = env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let domain_server = format!("{}:{}", ip, port);

        ServerSettings {
            ip_server: ip,
            domain_server,
            port: port.parse().unwrap_or(3000),
        }
    }
}

/// Configuration de la base de données
#[cfg(feature = "orm")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub engine: String,
    pub name: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub url: String,
}

#[cfg(feature = "orm")]
impl DatabaseSettings {
    pub fn from_env() -> Self {
        use dotenvy::dotenv;
        use std::env;

        dotenv().ok();

        let engine = env::var("DB_ENGINE").unwrap_or_else(|_| "sqlite".to_string());
        let user = env::var("DB_USER").unwrap_or_else(|_| "db_user".to_string());
        let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "db_password".to_string());
        let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let name = env::var("DB_NAME").unwrap_or_else(|_| "db_name".to_string());
        let url = if engine == "postgres" {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                user, password, host, port, name
            )
        } else {
            String::from("sqlite://local_base.sqlite?mode=rwc")
        };

        DatabaseSettings {
            engine,
            name,
            user,
            password,
            host,
            port: port.parse().unwrap_or(5432),
            url,
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
            secret_key: String::from("your-secret-key-change-in-production"),
            debug: cfg!(debug_assertions),
            allowed_hosts: vec![
                String::from("localhost"),
                String::from("127.0.0.1")
            ],
            installed_apps: vec![],
            middleware: vec![],
            root_urlconf: String::from("urls"),

            // Rusti-specific settings (pour servir des assets du framework)
            // Rusti-specific settings (pour servir des assets du framework)
            templates_rusti: String::new(),  // Pas utilisé, templates embarqués
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

            #[cfg(feature = "orm")]
            databases: DatabaseSettings::from_env(),

            staticfiles_storage: String::from("DefaultStaticFilesStorage"),
            language_code: String::from("en-us"),
            time_zone: String::from("UTC"),
            use_i18n: true,
            use_tz: true,
            auth_password_validators: vec![],
            password_hashers: vec![],
            default_auto_field: String::from("BigAutoField"),
            logging_config: String::from("default"),
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

    pub fn server(mut self, ip: impl Into<String>, port: u16) -> Self {
        let ip = ip.into();
        self.settings.server = ServerSettings {
            ip_server: ip.clone(),
            domain_server: format!("{}:{}", ip, port),
            port,
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
