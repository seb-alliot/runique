use crate::app::error_build::BuildError;
use crate::config::RuniqueConfig;
use crate::context::RequestExtensions;
use crate::middleware::session::CleaningMemoryStore;
use crate::middleware::{
    MiddlewareConfig, allowed_hosts_middleware, csrf_middleware, dev_no_cache_middleware,
    error_handler_middleware, security_headers_middleware,
};
use crate::utils::aliases::{AEngine, ARuniqueConfig, ATera};
use axum::{self, Router, middleware};
use tower_http::compression::CompressionLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore};

// ═══════════════════════════════════════════════════════════════
// MiddlewareStaging — Réorganisation automatique par Slots
// ═══════════════════════════════════════════════════════════════
//
// Innovation clé de Runique :
// Le développeur configure ses middlewares dans l'ordre qu'il veut.
// Chaque middleware possède un SLOT fixe (priorité).
// Au moment du build, le staging trie par slot et applique dans
// l'ordre optimal — automatiquement.
//
// Le CSRF lit/écrit un token en session → il DÉPEND de Session.
// Avec Axum brut, si on met CSRF avant Session → bug silencieux.
// Avec Runique → ça marche quand même, le framework réordonne.
//
// ═══════════════════════════════════════════════════════════════
//
// MODÈLE AXUM :
//   .layer(A).layer(B).layer(C)
//   Exécution requête : C → B → A → Handler
//   Dernier .layer() ajouté = le plus externe = premier exécuté
//
// NOTRE STRATÉGIE :
//   Slot bas  (0)   = externe = premier exécuté sur la requête
//   Slot haut (200+) = interne = plus proche du handler
//
//   On trie DESCENDANT puis on applique .layer() dans cet ordre :
//   le slot le plus haut est appliqué EN PREMIER (.layer) = le plus INTERNE
//   le slot le plus bas est appliqué EN DERNIER (.layer) = le plus EXTERNE
//
// RÉSULTAT sur une requête entrante :
//   → Extensions(0) → ErrorHandler(10) → Custom(20+)
//   → CSP(30) → Cache(40) → Session(50) → CSRF(60)
//   → Host(70) → Handler
//
// Reproduit l'ordre prouvé de l'ancien builder :
//   ErrorHandler enveloppe TOUT → attrape toutes les erreurs
//   ErrorHandler extrait Extension(tera/config) → injectées par Extensions
//   Session exécutée AVANT CSRF → CSRF peut lire la session
//   Host = dernier rempart avant le handler
//
// ═══════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────
// Slots built-in — Ordre d'exécution garanti sur la requête
// ─────────────────────────────────────────────────────────────

const SLOT_EXTENSIONS: u16 = 0; // Injection Engine/Tera/Config (outermost)
const SLOT_COMPRESSION: u16 = 5; // Compression (externe, avant tout autre middleware)
const SLOT_ERROR_HANDLER: u16 = 10; // Attrape les erreurs de TOUTE la pile
const SLOT_SECURITY_HEADERS: u16 = 30; // CSP + security headers
const SLOT_CACHE: u16 = 40; // Headers cache
const SLOT_SESSION: u16 = 50; // Avant CSRF (CSRF en dépend)
const SLOT_SESSION_UPGRADE: u16 = 55; // Après Session (lit/écrit en session)
const SLOT_CSRF: u16 = 60; // Après Session (lit/écrit en session)
const SLOT_HOST_VALIDATION: u16 = 70; // Dernier rempart avant handler

// Les middlewares custom du dev démarrent ICI
// Placés entre ErrorHandler et CSP → enveloppés par ErrorHandler
const SLOT_CUSTOM_BASE: u16 = 20;

// ─────────────────────────────────────────────────────────────
// MiddlewareEntry — Un middleware avec son slot de priorité
// ─────────────────────────────────────────────────────────────

struct MiddlewareEntry {
    /// Slot = position dans la pile.
    /// Bas (0) = externe, premier exécuté.
    /// Haut (100+) = interne, proche du handler.
    slot: u16,

    /// Nom lisible pour le debug et les logs
    #[allow(dead_code)]
    name: &'static str,

    /// Closure type-erased qui applique le middleware sur le router
    apply: Box<dyn FnOnce(Router) -> Router + Send>,
}

// ─────────────────────────────────────────────────────────────
// Types internes
// ─────────────────────────────────────────────────────────────

/// Closure type-erased pour un session store personnalisé
/// Params: (Router, debug: bool, duration: Duration) -> Router
pub(crate) type SessionApplicator = Box<dyn FnOnce(Router, bool, Duration) -> Router + Send>;

/// Closure type-erased pour un middleware custom du développeur
pub(crate) type CustomMiddleware = Box<dyn FnOnce(Router) -> Router + Send>;

// ═══════════════════════════════════════════════════════════════
// MiddlewareStaging
// ═══════════════════════════════════════════════════════════════

pub struct MiddlewareStaging {
    /// Configuration des features middleware (CSP, Host, Cache, etc.)
    pub(crate) features: MiddlewareConfig,

    /// Durée d'inactivité avant expiration de la session
    pub(crate) session_duration: Duration,

    /// Durée session anonyme => time life session anonyme
    pub(crate) anonymous_session_duration: Duration,
    /// Applicateur de session personnalisé (None = MemoryStore par défaut)
    pub(crate) session_applicator: Option<SessionApplicator>,

    /// Watermarks mémoire du CleaningMemoryStore (low, high) en octets
    pub(crate) session_low_watermark: usize,
    pub(crate) session_high_watermark: usize,

    /// Intervalle du cleanup périodique en secondes (défaut : 60s, via RUNIQUE_SESSION_CLEANUP_SECS)
    pub(crate) session_cleanup_interval_secs: u64,

    /// Middlewares custom du développeur (ordre d'ajout préservé)
    pub(crate) custom_middlewares: Vec<CustomMiddleware>,
}

impl MiddlewareStaging {
    /// Crée un MiddlewareStaging adapté au mode (debug/production)
    pub fn new(debug: bool) -> Self {
        let features = if debug {
            MiddlewareConfig::development()
        } else {
            MiddlewareConfig::production()
        };

        Self {
            features,
            session_duration: Duration::seconds(86400),
            anonymous_session_duration: Duration::seconds(300),
            session_applicator: None,
            session_low_watermark: 128 * 1024 * 1024,
            session_high_watermark: 256 * 1024 * 1024,
            session_cleanup_interval_secs: 60,
            custom_middlewares: Vec::new(),
        }
    }

    /// Crée un MiddlewareStaging depuis la RuniqueConfig.
    ///
    /// Stratégie de résolution :
    ///   1. Les variables `RUNIQUE_ENABLE_*` du `.env` sont prioritaires
    ///   2. Si absentes, le mode debug détermine les défauts :
    ///      - debug=true  → profil `development()` (permissif)
    ///      - debug=false → profil `production()` (strict)
    ///
    /// Le dev peut ensuite surcharger via `.middleware(|m| m.with_csp(true))`.
    pub fn from_config(config: &RuniqueConfig) -> Self {
        // Profil de base selon le mode
        let defaults = if config.debug {
            MiddlewareConfig::development()
        } else {
            MiddlewareConfig::production()
        };

        // Les variables .env sont prioritaires sur le profil
        let get_env_or = |key: &str, default: bool| -> bool {
            std::env::var(key)
                .map(|v| v.parse::<bool>().unwrap_or(default))
                .unwrap_or(default)
        };

        let features = MiddlewareConfig {
            enable_csp: get_env_or("RUNIQUE_ENABLE_CSP", defaults.enable_csp),
            enable_host_validation: get_env_or(
                "RUNIQUE_ENABLE_HOST_VALIDATION",
                defaults.enable_host_validation,
            ),
            enable_debug_errors: get_env_or(
                "RUNIQUE_ENABLE_DEBUG_ERRORS",
                defaults.enable_debug_errors,
            ),
            enable_cache: get_env_or("RUNIQUE_ENABLE_CACHE", defaults.enable_cache),
        };

        let get_env_u64 = |key: &str, default: u64| -> u64 {
            std::env::var(key)
                .map(|v| v.parse::<u64>().unwrap_or(default))
                .unwrap_or(default)
        };
        let get_env_usize = |key: &str, default: usize| -> usize {
            std::env::var(key)
                .map(|v| v.parse::<usize>().unwrap_or(default))
                .unwrap_or(default)
        };

        Self {
            features,
            session_duration: Duration::seconds(86400),
            anonymous_session_duration: Duration::seconds(300),
            session_applicator: None,
            session_low_watermark: get_env_usize(
                "RUNIQUE_SESSION_LOW_WATERMARK",
                128 * 1024 * 1024,
            ),
            session_high_watermark: get_env_usize(
                "RUNIQUE_SESSION_HIGH_WATERMARK",
                256 * 1024 * 1024,
            ),
            session_cleanup_interval_secs: get_env_u64("RUNIQUE_SESSION_CLEANUP_SECS", 60),
            custom_middlewares: Vec::new(),
        }
    }

    // ═══════════════════════════════════════════════════
    // Configuration des features
    // ═══════════════════════════════════════════════════

    /// Active ou désactive le Content Security Policy
    pub fn with_csp(mut self, enable: bool) -> Self {
        self.features.enable_csp = enable;
        self
    }

    /// Active ou désactive la validation des hosts autorisés
    pub fn with_host_validation(mut self, enable: bool) -> Self {
        self.features.enable_host_validation = enable;
        self
    }

    /// Active ou désactive les pages d'erreur de debug
    pub fn with_debug_errors(mut self, enable: bool) -> Self {
        self.features.enable_debug_errors = enable;
        self
    }

    /// Active ou désactive le cache HTTP
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.features.enable_cache = enable;
        self
    }

    // ═══════════════════════════════════════════════════
    // Configuration de la session
    // ═══════════════════════════════════════════════════

    /// Configure la durée d'inactivité avant expiration de la session
    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.session_duration = duration;
        self
    }

    /// Configure la durée d'inactivité avant expiration de la session anonyme
    pub fn with_anonymous_session_duration(mut self, duration: Duration) -> Self {
        self.anonymous_session_duration = duration;
        self
    }

    /// Configure les watermarks mémoire du CleaningMemoryStore.
    ///
    /// - `low`  : déclenche un cleanup proactif (non-bloquant) des sessions anonymes expirées
    /// - `high` : cleanup d'urgence synchrone + refus si toujours dépassé (503)
    ///
    /// Surchargeable via `.env` : `RUNIQUE_SESSION_LOW_WATERMARK` / `RUNIQUE_SESSION_HIGH_WATERMARK` (octets)
    pub fn with_session_memory_limit(mut self, low: usize, high: usize) -> Self {
        self.session_low_watermark = low;
        self.session_high_watermark = high;
        self
    }

    /// Configure l'intervalle du cleanup périodique.
    ///
    /// Surchargeable via `.env` : `RUNIQUE_SESSION_CLEANUP_SECS`
    pub fn with_session_cleanup_interval(mut self, secs: u64) -> Self {
        self.session_cleanup_interval_secs = secs;
        self
    }

    /// Configure un store de session personnalisé (Redis, PostgreSQL, etc.)
    ///
    /// # Exemple
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.with_session_store(RedisStore::new(client))
    ///      .with_session_duration(Duration::hours(2))
    /// })
    /// ```
    pub fn with_session_store<S: SessionStore + Clone + Send + Sync + 'static>(
        mut self,
        store: S,
    ) -> Self {
        self.session_applicator = Some(Box::new(
            move |router: Router, debug: bool, duration: Duration| {
                let layer = SessionManagerLayer::new(store)
                    .with_secure(!debug)
                    .with_http_only(true)
                    .with_same_site(tower_sessions::cookie::SameSite::Strict)
                    .with_expiry(Expiry::OnInactivity(duration));
                router.layer(layer)
            },
        ));
        self
    }

    // ═══════════════════════════════════════════════════
    // Middlewares custom du développeur
    // ═══════════════════════════════════════════════════

    /// Ajoute un middleware custom.
    ///
    /// Position automatique : `len + 1` — toujours APRÈS tous les
    /// middlewares built-in, au plus proche du handler.
    ///
    /// Si plusieurs customs sont ajoutés, ils sont placés dans
    /// l'ordre d'ajout (slot 100, 101, 102...).
    ///
    /// # Exemple
    /// ```rust,ignore
    /// .middleware(|m| {
    ///     m.add_custom(|router| {
    ///         router.layer(my_auth_layer)
    ///     })
    /// })
    /// ```
    pub fn add_custom(mut self, mw: impl FnOnce(Router) -> Router + Send + 'static) -> Self {
        self.custom_middlewares.push(Box::new(mw));
        self
    }

    // ═══════════════════════════════════════════════════
    // Validation
    // ═══════════════════════════════════════════════════

    /// Valide la cohérence de la configuration des middlewares
    pub fn validate(&self) -> Result<(), BuildError> {
        // CSRF toujours activé → rien à valider
        //
        // Futures validations :
        // - host_validation activé → ALLOWED_HOSTS défini ?
        // - enable_debug_errors en production → warning
        Ok(())
    }

    /// Les middlewares sont toujours prêts
    pub fn is_ready(&self) -> bool {
        true
    }

    /// Retourne la configuration des features middleware active
    pub fn features(&self) -> &MiddlewareConfig {
        &self.features
    }

    /// Retourne la durée de session configurée
    pub fn session_duration(&self) -> Duration {
        self.session_duration
    }

    /// Retourne le nombre de middlewares custom ajoutés
    pub fn custom_count(&self) -> usize {
        self.custom_middlewares.len()
    }

    // ═══════════════════════════════════════════════════════════
    // APPLICATION — Le cœur de l'innovation
    //
    // Construit la pile complète de middlewares :
    // 1. Collecte toutes les entries (built-in + custom)
    // 2. Chaque entry a son slot fixe
    // 3. Tri DESCENDANT par slot
    // 4. Application .layer() dans cet ordre
    //
    // Résultat (exécution sur requête) :
    //   Extensions → ErrorHandler → Custom → CSP → Cache
    //   → Session → CSRF → Host → Handler
    // ═══════════════════════════════════════════════════════════

    pub(crate) fn apply_to_router(
        self,
        router: Router,
        config: ARuniqueConfig,
        engine: AEngine,
        tera: ATera,
    ) -> Router {
        let debug = config.debug;
        let mut entries: Vec<MiddlewareEntry> = Vec::new();

        // ═══════════════════════════════════════
        // BUILT-IN : chaque middleware a un slot fixe.
        // Peu importe si le dev active CSP avant Host,
        // le tri par slot garantit le bon ordre.
        // ═══════════════════════════════════════

        // Slot 0 : Extensions (Engine, Tera, Config) — le plus externe
        {
            let eng = engine.clone();
            let t = tera.clone();
            let c = config.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_EXTENSIONS,
                name: "Extensions",
                apply: Box::new(move |r| {
                    r.layer(axum::middleware::from_fn(
                        move |mut req: axum::http::Request<axum::body::Body>,
                              next: axum::middleware::Next| {
                            let extensions = RequestExtensions::new()
                                .with_tera(t.clone())
                                .with_config(c.clone())
                                .with_engine(eng.clone());
                            extensions.inject_request(&mut req);
                            async move { next.run(req).await }
                        },
                    ))
                }),
            });
        }
        // Slot 5 : Compression — avant tout autre middleware
        {
            entries.push(MiddlewareEntry {
                slot: SLOT_COMPRESSION,
                name: "Compression",
                apply: Box::new(|r| r.layer(CompressionLayer::new())),
            });
        }
        // Slot 70 : Host validation — dernier rempart avant handler
        if self.features.enable_host_validation {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_HOST_VALIDATION,
                name: "HostValidation",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(
                        eng,
                        allowed_hosts_middleware,
                    ))
                }),
            });
        }

        // Slot 50 : Session — avant CSRF (CSRF en dépend)
        {
            let applicator = self.session_applicator;
            let anon_duration = self.anonymous_session_duration;
            let low_wm = self.session_low_watermark;
            let high_wm = self.session_high_watermark;
            let cleanup_secs = self.session_cleanup_interval_secs;

            entries.push(MiddlewareEntry {
                slot: SLOT_SESSION,
                name: "Session",
                apply: Box::new(move |r: Router| match applicator {
                    Some(apply_fn) => apply_fn(r, debug, anon_duration),
                    None => {
                        let store = CleaningMemoryStore::default().with_watermarks(low_wm, high_wm);
                        store.spawn_cleanup(tokio::time::Duration::from_secs(cleanup_secs));
                        let layer = SessionManagerLayer::new(store)
                            .with_secure(!debug)
                            .with_http_only(true)
                            .with_same_site(tower_sessions::cookie::SameSite::Strict)
                            .with_expiry(Expiry::OnInactivity(anon_duration));
                        r.layer(layer)
                    }
                }),
            });
        }

        // Slot 60 : CSRF — TOUJOURS activé, après Session
        {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_CSRF,
                name: "CSRF",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(eng, csrf_middleware))
                }),
            });
        }

        // Slot 40 : Cache control
        if !self.features.enable_cache {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_CACHE,
                name: "NoCache",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(eng, dev_no_cache_middleware))
                }),
            });
        }

        // Slot 30 : CSP + Security headers
        if self.features.enable_csp {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_SECURITY_HEADERS,
                name: "SecurityHeaders",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(
                        eng,
                        security_headers_middleware,
                    ))
                }),
            });
        }

        // Slot 55 : Upgrade TTL si authentifié
        {
            entries.push(MiddlewareEntry {
                slot: SLOT_SESSION_UPGRADE,
                name: "SessionTtlUpgrade",
                apply: Box::new(move |r| {
                    r.layer(axum::middleware::from_fn(
                        move |req: axum::http::Request<axum::body::Body>,
                              next: axum::middleware::Next| {
                            async move {
                                if let Some(session) =
                                    req.extensions().get::<tower_sessions::Session>()
                                {
                                    if crate::middleware::auth::is_authenticated(session).await {
                                        session.set_expiry(Some(Expiry::OnInactivity(
                                            self.session_duration,
                                        )));
                                    }
                                }
                                next.run(req).await
                            }
                        },
                    ))
                }),
            });
        }

        // Slot 10 : Error handler — enveloppe TOUTE la pile, attrape toutes les erreurs
        // Extrait Extension(tera) et Extension(config) injectées par Extensions (slot 0)
        if self.features.enable_debug_errors {
            entries.push(MiddlewareEntry {
                slot: SLOT_ERROR_HANDLER,
                name: "ErrorHandler",
                apply: Box::new(|r| r.layer(middleware::from_fn(error_handler_middleware))),
            });
        }

        // ═══════════════════════════════════════
        // CUSTOM : Position automatique entre ErrorHandler et CSP.
        //
        // Le dev ne choisit pas de slot.
        // Ses middlewares sont enveloppés par ErrorHandler
        // mais exécutés avant les middlewares de sécurité.
        //
        // Premier custom → slot 20
        // Deuxième custom → slot 21
        // etc.
        // ═══════════════════════════════════════

        for (i, custom_mw) in self.custom_middlewares.into_iter().enumerate() {
            entries.push(MiddlewareEntry {
                slot: SLOT_CUSTOM_BASE + i as u16,
                name: "Custom",
                apply: custom_mw,
            });
        }

        // ═══════════════════════════════════════
        // TRI DESCENDANT + APPLICATION
        //
        // Slot le plus haut → premier .layer() → le plus INTERNE
        // Slot le plus bas  → dernier .layer() → le plus EXTERNE
        //
        // En Axum : dernier .layer() = premier exécuté sur la requête
        //
        // Résultat sur la requête :
        //   Extensions(0) → ErrorHandler(10) → Custom(20+)
        //   → CSP(30) → Cache(40) → Session(50) → CSRF(60)
        //   → Host(70) → Handler
        //
        // ErrorHandler enveloppe tout → attrape toutes les erreurs
        // Session avant CSRF → CSRF peut lire la session
        // ═══════════════════════════════════════

        entries.sort_by_key(|b| std::cmp::Reverse(b.slot));

        let mut router = router;
        for entry in entries {
            router = (entry.apply)(router);
        }

        router
    }
}
