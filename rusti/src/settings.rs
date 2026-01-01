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

    /// Parse ALLOWED_HOSTS depuis une variable d'environnement
    ///
    /// Format attendu dans .env:
    /// ```env
    /// ALLOWED_HOSTS=localhost,127.0.0.1,exemple.com,.exemple.com
    /// ```
    ///
    /// Supporte les wildcards: `.exemple.com` matchera tous les sous-domaines
    pub fn parse_allowed_hosts_from_env() -> Vec<String> {
        use dotenvy::dotenv;
        use std::env;

        dotenv().ok();

        env::var("ALLOWED_HOSTS")
            .ok()
            .map(|hosts| {
                hosts
                    .split(',')
                    .map(|h| h.trim().to_string())
                    .filter(|h| !h.is_empty())
                    .collect()
            })
            .unwrap_or_else(|| vec![
                String::from("localhost"),
                String::from("127.0.0.1")
            ])
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
        let rusti_root = env!("CARGO_MANIFEST_DIR");
        let static_rusti_path = format!("{}/static", rusti_root);
        let media_rusti_path = format!("{}/media", rusti_root);
        let templates_rusti = format!("{}/templates", rusti_root);

        Settings {
            server: ServerSettings::from_env(),
            base_dir,
            debug: cfg!(debug_assertions),
            // Charge ALLOWED_HOSTS depuis .env ou utilise les valeurs par défaut
            allowed_hosts: ServerSettings::parse_allowed_hosts_from_env(),
            installed_apps: vec![],
            middleware: vec![],
            root_urlconf: String::from("urls"),

            // Rusti-specific settings
            templates_rusti: templates_rusti,
            static_rusti_path,
            static_rusti_url: "/rusti/static".to_string(),
            media_rusti_path,
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

    /// Valide que ALLOWED_HOSTS est correctement configuré en production
    ///
    /// # Panics
    /// Panique si debug=false et allowed_hosts est vide ou contient uniquement localhost/127.0.0.1
    pub fn validate_allowed_hosts(&self) {
        if !self.debug {
            if self.allowed_hosts.is_empty() {
                panic!(
                    "ALLOWED_HOSTS ne peut pas être vide en production!\n\
                    Ajoutez vos domaines dans le fichier .env:\n\
                    ALLOWED_HOSTS=exemple.com,www.exemple.com"
                );
            }

            let only_local = self.allowed_hosts.iter().all(|h| {
                h == "localhost" || h == "127.0.0.1" || h == "::1"
            });

            if only_local {
                eprintln!(
                    "AVERTISSEMENT: ALLOWED_HOSTS contient uniquement des hôtes locaux en production.\n\
                    Ajoutez vos domaines de production dans le fichier .env:\n\
                    ALLOWED_HOSTS=exemple.com,www.exemple.com,localhost,127.0.0.1"
                );
            }
        }
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

    /// Configure ALLOWED_HOSTS manuellement
    ///
    /// # Exemple
    /// ```rust
    /// let settings = Settings::builder()
    ///     .allowed_hosts(vec![
    ///         "exemple.com".to_string(),
    ///         "www.exemple.com".to_string(),
    ///         ".sous-domaine.exemple.com".to_string(), // Wildcard
    ///     ])
    ///     .build();
    /// ```
    pub fn allowed_hosts(mut self, hosts: Vec<String>) -> Self {
        self.settings.allowed_hosts = hosts;
        self
    }

    /// Ajoute un host à ALLOWED_HOSTS
    pub fn add_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.settings.allowed_hosts.push(host.into());
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
        // Valide la configuration avant de la retourner
        self.settings.validate_allowed_hosts();
        self.settings
    }
}

impl Default for SettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}