use std::sync::OnceLock;
use tracing::Level;

/// Configuration des logs Runique par catégorie.
///
/// Chaque catégorie est désactivée par défaut (`None`).
/// Appeler la méthode correspondante avec un niveau active la catégorie.
///
/// # Exemple
/// ```rust,ignore
/// RuniqueApp::builder(config)
///     .with_log(|l| l
///         .csrf(Level::WARN)
///         .exclusive_login(Level::INFO)
///     )
/// ```
#[derive(Debug, Clone, Default)]
pub struct RuniqueLog {
    /// Détecte un csrf_token dans une URL GET (nettoyage silencieux).
    pub csrf: Option<Level>,
    /// Trace l'invalidation de sessions lors d'une connexion exclusive.
    pub exclusive_login: Option<Level>,
    /// Signale l'échec d'une `filter_fn` dans la vue liste admin.
    pub filter_fn: Option<Level>,
    /// Signale les erreurs d'accès au registre des rôles admin.
    pub roles: Option<Level>,
    /// Avertit si `password_init()` est appelé plusieurs fois.
    pub password_init: Option<Level>,
    /// Traces du session store : watermarks mémoire, records volumineux, erreurs cleanup.
    pub session: Option<Level>,
    /// Infos de connexion DB (connecting / connected successfully).
    pub db: Option<Level>,
}

impl RuniqueLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn csrf(mut self, level: Level) -> Self {
        self.csrf = Some(level);
        self
    }

    pub fn exclusive_login(mut self, level: Level) -> Self {
        self.exclusive_login = Some(level);
        self
    }

    pub fn filter_fn(mut self, level: Level) -> Self {
        self.filter_fn = Some(level);
        self
    }

    pub fn roles(mut self, level: Level) -> Self {
        self.roles = Some(level);
        self
    }

    pub fn password_init(mut self, level: Level) -> Self {
        self.password_init = Some(level);
        self
    }

    pub fn session(mut self, level: Level) -> Self {
        self.session = Some(level);
        self
    }

    pub fn db(mut self, level: Level) -> Self {
        self.db = Some(level);
        self
    }

    /// Active toutes les catégories au niveau `DEBUG`.
    ///
    /// Sans effet si `DEBUG` n'est pas `true` ou `1` dans l'environnement —
    /// peut être utilisé inconditionnellement dans `.with_log()`.
    ///
    /// ```rust,ignore
    /// .with_log(|l| l.dev())
    /// // ou avec surcharge
    /// .with_log(|l| l.dev().db(Level::INFO))
    /// ```
    pub fn dev(self) -> Self {
        if !crate::utils::env::is_debug() {
            return self;
        }
        self.csrf(Level::DEBUG)
            .exclusive_login(Level::DEBUG)
            .filter_fn(Level::DEBUG)
            .roles(Level::DEBUG)
            .password_init(Level::DEBUG)
            .session(Level::DEBUG)
            .db(Level::DEBUG)
    }
}

static LOG_CONFIG: OnceLock<RuniqueLog> = OnceLock::new();

/// Initialise la configuration de logs — appelé une fois pendant `build()`.
pub fn log_init(config: RuniqueLog) {
    // Double appel silencieux : la config initiale est conservée.
    LOG_CONFIG.set(config).ok();
}

/// Retourne la configuration de logs active.
/// Retourne une config vide (tout désactivé) si `log_init` n'a pas été appelé.
pub fn get_log() -> &'static RuniqueLog {
    LOG_CONFIG.get_or_init(RuniqueLog::default)
}

/// Émet un événement tracing au niveau dynamique configuré.
///
/// # Exemple
/// ```rust,ignore
/// if let Some(level) = get_log().csrf {
///     runique_log!(level, path = %path, "csrf_token détecté dans une URL GET");
/// }
/// ```
#[macro_export]
macro_rules! runique_log {
    ($level:expr, $($args:tt)*) => {
        match $level {
            ::tracing::Level::ERROR => ::tracing::error!($($args)*),
            ::tracing::Level::WARN  => ::tracing::warn!($($args)*),
            ::tracing::Level::INFO  => ::tracing::info!($($args)*),
            ::tracing::Level::DEBUG => ::tracing::debug!($($args)*),
            _                       => ::tracing::trace!($($args)*),
        }
    };
}
