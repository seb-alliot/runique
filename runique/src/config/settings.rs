use serde::{Deserialize, Serialize};

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
        Self {
            installed_apps: vec![
                "runique_admin".to_string(),
                "runique_auth".to_string(),
                "runique_contenttypes".to_string(),
                "runique_sessions".to_string(),
                "runique_messages".to_string(),
                "runique_staticfiles".to_string(),
            ],
            middleware: vec![
                "runique.middleware.security.SecurityMiddleware".to_string(),
                "runique.contrib.sessions.middleware.SessionMiddleware".to_string(),
                "runique.middleware.common.CommonMiddleware".to_string(),
                "runique.middleware.csrf.CsrfViewMiddleware".to_string(),
                "runique.contrib.auth.middleware.AuthenticationMiddleware".to_string(),
                "runique.contrib.messages.middleware.MessageMiddleware".to_string(),
            ],
            root_urlconf: "project.urls".to_string(),
            language_code: "en-us".to_string(),
            time_zone: "UTC".to_string(),
            use_i18n: true,
            use_tz: true,
            auth_password_validators: vec![
                "runique.contrib.auth.password_validation.UserAttributeSimilarityValidator"
                    .to_string(),
                "runique.contrib.auth.password_validation.MinimumLengthValidator".to_string(),
                "runique.contrib.auth.password_validation.CommonPasswordValidator".to_string(),
                "runique.contrib.auth.password_validation.NumericPasswordValidator".to_string(),
            ],
            password_hashers: vec![
                "runique.contrib.auth.hashers.Argon2PasswordHasher".to_string(),
                "runique.contrib.auth.hashers.BCryptSHA256PasswordHasher".to_string(),
                "runique.contrib.auth.hashers.BCryptPasswordHasher".to_string(),
                "runique.contrib.auth.hashers.PBKDF2PasswordHasher".to_string(),
                "runique.contrib.auth.hashers.PBKDF2SHA1PasswordHasher".to_string(),
            ],
            default_auto_field: "runique.db.models.AutoField".to_string(),
            logging_config: "runique.utils.log.default_logging_config".to_string(),
        }
    }
}

impl AppSettings {
    pub fn from_env() -> Self {
        Self::default()
    }
}
