//! Application configuration and settings
//!
//! This module provides Django-inspired settings management for Runique applications.
//! Settings can be loaded from environment variables or configured programmatically
//! using the builder pattern.
//!
//! # Examples
//!
//! ```no_run
//! use runique::Settings;
//!
//! // Load from environment variables
//! let settings = Settings::default_values();
//!
//! // Or use the builder pattern
//! let settings = Settings::builder()
//!     .debug(true)
//!     .allowed_hosts(vec!["example.com".to_string()])
//!     .server("127.0.0.1", 8000, "my-secret-key")
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::vec;

/// Main application configuration for Runique
///
/// Inspired by Django's settings.py, this struct contains all configuration
/// options for a Runique application including server settings, static files,
/// templates, security options, and more.
///
/// # Examples
///
/// ```no_run
/// use runique::Settings;
///
/// // Load with default values from environment
/// let settings = Settings::default_values();
///
/// // Check if debug mode is enabled
/// if settings.debug {
///     println!("Running in debug mode");
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration (IP, port, secret key)
    pub server: ServerSettings,
    /// Base directory for the application
    pub base_dir: String,
    /// Enable debug mode (more verbose logging, detailed errors)
    pub debug: bool,
    /// List of allowed host headers (security feature)
    pub allowed_hosts: Vec<String>,
    /// List of installed applications
    pub installed_apps: Vec<String>,
    /// Middleware stack
    pub middleware: Vec<String>,
    /// Root URL configuration module
    pub root_urlconf: String,

    // Runique-specific settings
    /// Path to Runique's internal static files
    pub static_runique_path: String,
    /// URL prefix for Runique's static files
    pub static_runique_url: String,
    /// Path to Runique's internal media files
    pub media_runique_path: String,
    /// URL prefix for Runique's media files
    pub media_runique_url: String,
    /// Path to Runique's internal templates
    pub templates_runique: String,

    // User project settings
    /// Directories containing user templates
    pub templates_dir: Vec<String>,
    /// Directory containing user static files
    pub staticfiles_dirs: String,
    /// Root directory for user media uploads
    pub media_root: String,
    /// URL prefix for static files
    pub static_url: String,
    /// URL prefix for media files
    pub media_url: String,

    /// Static files storage backend
    pub staticfiles_storage: String,

    /// Language code (e.g., "en-us", "fr-fr")
    pub language_code: String,
    /// Timezone (e.g., "UTC", "America/New_York")
    pub time_zone: String,
    /// Enable internationalization
    pub use_i18n: bool,
    /// Enable timezone support
    pub use_tz: bool,
    /// Password validation rules
    pub auth_password_validators: Vec<String>,
    /// Password hashing algorithms
    pub password_hashers: Vec<String>,
    /// Default auto field type for models
    pub default_auto_field: String,
    /// Logging configuration
    pub logging_config: String,

    // Security settings
    /// Enable automatic input sanitization
    pub sanitize_inputs: bool,
    /// Enable strict Content Security Policy
    pub strict_csp: bool,
    /// Enable rate limiting
    pub rate_limiting: bool,
    /// Enforce HTTPS in production
    pub enforce_https: bool,
}

/// Server configuration
///
/// Contains network and security settings for the HTTP server.
///
/// # Examples
///
/// ```no_run
/// use runique::settings::ServerSettings;
///
/// let server = ServerSettings::from_env();
/// println!("Server running on {}:{}", server.ip_server, server.port);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// IP address to bind to
    pub ip_server: String,
    /// Full domain (IP:port)
    pub domain_server: String,
    /// Port to listen on
    pub port: u16,
    /// Secret key for cryptographic signing
    pub secret_key: String,
}

impl ServerSettings {
    /// Creates server settings from environment variables
    ///
    /// Reads the following environment variables:
    /// - `IP_SERVER` - Server IP (default: "127.0.0.1")
    /// - `PORT` - Server port (default: 3000)
    /// - `SECRET_KEY` - Secret key for signing (default: "default_secret_key")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::settings::ServerSettings;
    ///
    /// // .env file:
    /// // IP_SERVER=0.0.0.0
    /// // PORT=8000
    /// // SECRET_KEY=my-secret-key
    ///
    /// let server = ServerSettings::from_env();
    /// ```
    pub fn from_env() -> Self {
        use dotenvy::dotenv;
        use std::env;

        dotenv().ok();
        let ip = env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let domain_server = format!("{}:{}", ip, port);
        let secret_key =
            env::var("SECRET_KEY").unwrap_or_else(|_| "default_secret_key".to_string());

        ServerSettings {
            ip_server: ip,
            domain_server,
            port: port.parse().unwrap_or(3000),
            secret_key: secret_key.to_string(),
        }
    }

    /// Parses ALLOWED_HOSTS from environment variable
    ///
    /// Expected format in `.env`:
    /// ```env
    /// ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.example.com
    /// ```
    ///
    /// Supports wildcards: `.example.com` will match all subdomains
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::settings::ServerSettings;
    ///
    /// let hosts = ServerSettings::parse_allowed_hosts_from_env();
    /// assert!(hosts.contains(&"localhost".to_string()));
    /// ```
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
            .unwrap_or_else(|| vec![String::from("localhost"), String::from("127.0.0.1")])
    }
}

impl Settings {
    /// Creates a configuration with default values
    ///
    /// Loads settings from environment variables when available,
    /// otherwise uses sensible defaults.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::default_values();
    /// assert_eq!(settings.language_code, "en-us");
    /// ```
    pub fn default_values() -> Self {
        let base_dir = ".".to_string();
        let runique_root = env!("CARGO_MANIFEST_DIR");
        let static_runique_path = format!("{}/static", runique_root);
        let media_runique_path = format!("{}/media", runique_root);
        let templates_runique = format!("{}/templates", runique_root);

        Settings {
            server: ServerSettings::from_env(),
            base_dir,
            debug: cfg!(debug_assertions),
            // Charge ALLOWED_HOSTS depuis .env ou utilise les valeurs par défaut
            allowed_hosts: ServerSettings::parse_allowed_hosts_from_env(),
            installed_apps: vec![],
            middleware: vec![],
            root_urlconf: String::from("urls"),

            // Runique-specific settings
            templates_runique,
            static_runique_path,
            static_runique_url: "/runique/static".to_string(),
            media_runique_path,
            media_runique_url: "/runique/media".to_string(),

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

    /// Loads configuration from environment variables
    ///
    /// This is currently an alias for `default_values()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::from_env();
    /// ```
    pub fn from_env() -> Self {
        Self::default_values()
    }

    /// Creates a settings builder for custom configuration
    ///
    /// The builder pattern allows fluent configuration of all settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .debug(false)
    ///     .server("0.0.0.0", 8000, "production-secret")
    ///     .add_allowed_host("example.com")
    ///     .build();
    /// ```
    pub fn builder() -> SettingsBuilder {
        SettingsBuilder::new()
    }

    /// Validates that ALLOWED_HOSTS is properly configured in production
    ///
    /// # Panics
    ///
    /// Panics if `debug = false` and `allowed_hosts` is empty or contains
    /// only localhost/127.0.0.1
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .debug(false)
    ///     .allowed_hosts(vec![]) // This will panic!
    ///     .build();
    /// ```
    pub fn validate_allowed_hosts(&self) {
        if !self.debug {
            if self.allowed_hosts.is_empty() {
                panic!(
                    "ALLOWED_HOSTS cannot be empty in production!\n\
                    Add your domains in the .env file:\n\
                    ALLOWED_HOSTS=example.com,www.example.com"
                );
            }

            let only_local = self
                .allowed_hosts
                .iter()
                .all(|h| h == "localhost" || h == "127.0.0.1" || h == "::1");

            if only_local {
                eprintln!(
                    "WARNING: ALLOWED_HOSTS contains only local hosts in production.\n\
                    Add your production domains in the .env file:\n\
                    ALLOWED_HOSTS=example.com,www.example.com,localhost,127.0.0.1"
                );
            }
        }
    }
}

/// Builder for creating custom Settings
///
/// Provides a fluent interface for configuring all aspects of a Runique application.
///
/// # Examples
///
/// ```no_run
/// use runique::Settings;
///
/// let settings = Settings::builder()
///     .debug(true)
///     .server("127.0.0.1", 3000, "dev-secret")
///     .allowed_hosts(vec!["localhost".to_string()])
///     .sanitize_inputs(true)
///     .strict_csp(false)
///     .build();
/// ```
pub struct SettingsBuilder {
    settings: Settings,
}

impl SettingsBuilder {
    /// Creates a new settings builder with default values
    pub fn new() -> Self {
        Self {
            settings: Settings::default_values(),
        }
    }

    /// Sets debug mode
    ///
    /// When enabled, provides more verbose logging and detailed error pages.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .debug(true)
    ///     .build();
    /// ```
    pub fn debug(mut self, debug: bool) -> Self {
        self.settings.debug = debug;
        self
    }

    /// Configures ALLOWED_HOSTS manually
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .allowed_hosts(vec!["example.com".to_string(), "www.example.com".to_string()])
    ///     .build();
    /// ```
    pub fn allowed_hosts(mut self, hosts: Vec<String>) -> Self {
        self.settings.allowed_hosts = hosts;
        self
    }

    /// Adds a host to ALLOWED_HOSTS
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .add_allowed_host("example.com")
    ///     .add_allowed_host("api.example.com")
    ///     .build();
    /// ```
    pub fn add_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.settings.allowed_hosts.push(host.into());
        self
    }

    /// Sets Runique's internal static files path
    pub fn static_runique_path(mut self, path: impl Into<String>) -> Self {
        self.settings.static_runique_path = path.into();
        self
    }

    /// Sets Runique's internal static files URL prefix
    pub fn static_runique_url(mut self, url: impl Into<String>) -> Self {
        self.settings.static_runique_url = url.into();
        self
    }

    /// Sets Runique's internal media files path
    pub fn media_runique_path(mut self, path: impl Into<String>) -> Self {
        self.settings.media_runique_path = path.into();
        self
    }

    /// Sets Runique's internal media files URL prefix
    pub fn media_runique_url(mut self, url: impl Into<String>) -> Self {
        self.settings.media_runique_url = url.into();
        self
    }

    /// Sets Runique's internal templates directory
    pub fn templates_runique(mut self, dir: impl Into<String>) -> Self {
        self.settings.templates_runique = dir.into();
        self
    }

    /// Sets user template directories
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .templates_dir(vec!["templates".to_string(), "extra_templates".to_string()])
    ///     .build();
    /// ```
    pub fn templates_dir(mut self, dir: impl Into<Vec<std::string::String>>) -> Self {
        self.settings.templates_dir = dir.into();
        self
    }

    /// Sets static files directory
    pub fn staticfiles_dirs(mut self, dir: impl Into<String>) -> Self {
        self.settings.staticfiles_dirs = dir.into();
        self
    }

    /// Sets media root directory
    pub fn media_root(mut self, dir: impl Into<String>) -> Self {
        self.settings.media_root = dir.into();
        self
    }

    /// Sets static files URL prefix
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .static_url("/assets")
    ///     .build();
    /// ```
    pub fn static_url(mut self, url: impl Into<String>) -> Self {
        self.settings.static_url = url.into();
        self
    }

    /// Sets media files URL prefix
    pub fn media_url(mut self, url: impl Into<String>) -> Self {
        self.settings.media_url = url.into();
        self
    }

    /// Enables or disables automatic input sanitization
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .sanitize_inputs(false)
    ///     .build();
    /// ```
    pub fn sanitize_inputs(mut self, enabled: bool) -> Self {
        self.settings.sanitize_inputs = enabled;
        self
    }

    /// Enables or disables strict Content Security Policy
    pub fn strict_csp(mut self, enabled: bool) -> Self {
        self.settings.strict_csp = enabled;
        self
    }

    /// Enables or disables rate limiting
    pub fn rate_limiting(mut self, enabled: bool) -> Self {
        self.settings.rate_limiting = enabled;
        self
    }

    /// Enforces HTTPS in production
    pub fn enforce_https(mut self, enabled: bool) -> Self {
        self.settings.enforce_https = enabled;
        self
    }

    /// Configures server settings (IP, port, secret key)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .server("0.0.0.0", 8000, "my-secret-key")
    ///     .build();
    /// ```
    pub fn server(
        mut self,
        ip: impl Into<String>,
        port: u16,
        secret_key: impl Into<String>,
    ) -> Self {
        let ip_val = ip.into();
        self.settings.server = ServerSettings {
            ip_server: ip_val.clone(),
            domain_server: format!("{}:{}", ip_val, port),
            port,
            secret_key: secret_key.into(),
        };
        self
    }

    /// Builds the final Settings instance
    ///
    /// Validates the configuration before returning.
    ///
    /// # Panics
    ///
    /// Panics if ALLOWED_HOSTS validation fails in production mode
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::Settings;
    ///
    /// let settings = Settings::builder()
    ///     .debug(true)
    ///     .build();
    /// ```
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
