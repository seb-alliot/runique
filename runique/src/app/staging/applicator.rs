//! `apply_middleware`: lists the middleware entries, checks and sorts them by slot, applies them to the router.
//!
//! ═══════════════════════════════════════════════════════════════
//! Key Runique innovation:
//! The developer configures their middlewares in the order they want.
//! Each middleware has a fixed SLOT (priority).
//! At build time, staging sorts by slot and applies in the
//! optimal order — automatically.
//!
//! CSRF reads/writes a token in the session → it DEPENDS on Session.
//! With raw Axum, if you put CSRF before Session → silent bug.
//! With Runique → it works anyway, the framework reorders.
//!
//! ═══════════════════════════════════════════════════════════════
//!
//! AXUM MODEL:
//!   .layer(A).layer(B).layer(C)
//!   Request execution: C → B → A → Handler
//!   Last added `.layer()` = outermost = first executed
//!
//! OUR STRATEGY:
//!   Low slot (0)   = external = first executed on the request
//!   High slot (200+) = internal = closer to the handler
//!
//!   We sort DESCENDING then apply `.layer()` in this order:
//!   the highest slot is applied FIRST (.layer) = the most INTERNAL
//!   the lowest slot is applied LAST (.layer) = the most EXTERNAL
//!
//! RESULT on an incoming request:
//!   → Extensions(0) → TrustedProxies(2) → CORS(8) → Lang(9) → ErrorHandler(10) → Host(15)
//!   → HttpsRedirect(17) → Custom(20+)
//!   → OpenRedirect(25) → CSP(30) → Cache(40) → Session(50) → CSRF(60) → Handler
//!
//! Host validation sits right after ErrorHandler (still covered by its panic
//! handling) but before Session/Auth/CSRF — a request with a spoofed Host
//! header is rejected before paying for session load, auth lookup and CSRF
//! checks. Moving it any lower than ErrorHandler would remove that panic net.

use crate::{
    engine::SessionStores,
    errors::build::BuildError,
    middleware::{
        allowed_hosts_middleware, anti_bot_middleware,
        config::extensions::extensions_middleware,
        csrf_middleware, dev_no_cache_middleware, error_handler_middleware,
        https_redirect_middleware, lang_middleware, open_redirect_middleware,
        security_headers_middleware,
        session::{
            CleaningMemoryStore, auth_middleware, session_db::RuniqueSessionStore,
            session_ttl_upgrade,
        },
        trusted_proxies_middleware,
    },
    utils::aliases::AEngine,
};
use axum::{self, Router, middleware};
use std::sync::Arc;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};

use super::MiddlewareStaging;

// ─── Built-in slots — Guaranteed execution order on the request ───────────────

const SLOT_EXTENSIONS: u16 = 0; // Engine injection (outermost)
const SLOT_TRUSTED_PROXIES: u16 = 10; // Real client IP extraction — before everything
const SLOT_COMPRESSION: u16 = 20; // Compression (external, before any other middleware)
const SLOT_CORS: u16 = 30; // Outside ErrorHandler — OPTIONS preflight never reaches CSRF
const SLOT_LANG: u16 = 40; // Outside ErrorHandler: an error page is in the visitor's language too
const SLOT_ERROR_HANDLER: u16 = 50; // Catches errors of the WHOLE stack
const SLOT_HOST_VALIDATION: u16 = 55; // After ErrorHandler (still caught on panic), before Session/Auth/CSRF cost
const SLOT_HTTPS_REDIRECT: u16 = 57; // After HostValidation: the redirect is built from a checked Host
const SLOT_CUSTOM_BASE: u16 = 60; // Dev's custom middlewares start here
const SLOT_OPEN_REDIRECT: u16 = 70; // After custom, before CSP — wraps response inspection
const SLOT_SECURITY_HEADERS: u16 = 80;
const SLOT_CACHE: u16 = 90;
const SLOT_SESSION: u16 = 100; // Before CSRF (CSRF depends on it)
const SLOT_SESSION_UPGRADE: u16 = 105; // After Session (reads/writes in session)
const SLOT_AUTH: u16 = 107; // After Session — reads the signed-in account from the DB
const SLOT_CSRF: u16 = 110; // After Session (reads/writes in session)
const SLOT_ANTI_BOT: u16 = 115; // After CSRF — injects honeypot field name extension

// ─── MiddlewareEntry ──────────────────────────────────────────────────────────

struct MiddlewareEntry {
    /// Low (0) = external, first executed. High (100+) = internal, close to the handler.
    slot: u16,
    name: &'static str,
    apply: Box<dyn FnOnce(Router) -> Router + Send>,
}

/// An entry whose middleware takes the engine as its state.
macro_rules! with_engine {
    ($slot:expr, $name:literal, $engine:expr, $handler:expr) => {{
        let engine = $engine.clone();
        MiddlewareEntry {
            slot: $slot,
            name: $name,
            apply: Box::new(move |r: Router| {
                r.layer(middleware::from_fn_with_state(engine, $handler))
            }),
        }
    }};
}

// ─── apply_middleware ────────────────────────────────────────────────────────

impl MiddlewareStaging {
    /// Builds the full middleware stack and applies it to the router.
    ///
    /// Every middleware is listed below with its fixed slot, in the order a
    /// request goes through them. The stack is then checked (no two on the
    /// same slot), sorted DESCENDING by slot (highest = most internal, applied
    /// first via `.layer()`), and applied.
    pub(crate) fn apply_middleware(
        mut self,
        router: Router,
        engine: AEngine,
    ) -> Result<Router, BuildError> {
        let mut entries = vec![
            with_engine!(SLOT_EXTENSIONS, "Extensions", engine, extensions_middleware),
            // Real client IP from X-Forwarded-For, before everything that reads it
            with_engine!(
                SLOT_TRUSTED_PROXIES,
                "TrustedProxies",
                engine,
                trusted_proxies_middleware
            ),
        ];
        // Compression, before any other middleware
        if let Some(layer) = self.compression_config.layer() {
            entries.push(MiddlewareEntry {
                slot: SLOT_COMPRESSION,
                name: "Compression",
                apply: Box::new(move |r| r.layer(layer)),
            });
        }
        // Outside ErrorHandler so an OPTIONS preflight never reaches CSRF/Session
        if let Some(cors) = self.cors_config.take() {
            let layer = cors.layer();
            entries.push(MiddlewareEntry {
                slot: SLOT_CORS,
                name: "CORS",
                apply: Box::new(move |r| r.layer(layer)),
            });
        }
        // The visitor's language, for everything inside — error pages included
        if self.lang {
            entries.push(MiddlewareEntry {
                slot: SLOT_LANG,
                name: "Lang",
                apply: Box::new(|r| r.layer(middleware::from_fn(lang_middleware))),
            });
        }
        // Wraps the WHOLE stack, catches all errors
        if self.features.enable_debug_errors {
            entries.push(MiddlewareEntry {
                slot: SLOT_ERROR_HANDLER,
                name: "ErrorHandler",
                apply: Box::new(|r| r.layer(middleware::from_fn(error_handler_middleware))),
            });
        }
        // After ErrorHandler (panic still caught), before Session/Auth/CSRF so a
        // spoofed Host is rejected before that cost
        if self.features.enable_host_validation {
            entries.push(with_engine!(
                SLOT_HOST_VALIDATION,
                "HostValidation",
                engine,
                allowed_hosts_middleware
            ));
        }
        // HTTPS redirect behind a TLS-terminating proxy. Not with ACME:
        // Runique serves TLS itself there, no request carries `X-Forwarded-Proto`,
        // and its port-80 listener already redirects.
        let acme_active = cfg!(feature = "acme") && engine.config.security.acme_enabled;
        if engine.config.security.enforce_https && !acme_active {
            entries.push(with_engine!(
                SLOT_HTTPS_REDIRECT,
                "HttpsRedirect",
                engine,
                https_redirect_middleware
            ));
        }
        // The developer's middlewares, between ErrorHandler and CSP (slots 20+)
        let custom = std::mem::take(&mut self.custom_middlewares);
        for (i, apply) in custom.into_iter().enumerate() {
            entries.push(MiddlewareEntry {
                slot: SLOT_CUSTOM_BASE.saturating_add(i as u16),
                name: "Custom",
                apply,
            });
        }
        // ALWAYS active
        entries.push(with_engine!(
            SLOT_OPEN_REDIRECT,
            "OpenRedirect",
            engine,
            open_redirect_middleware
        ));
        // CSP + nonce + HSTS/X-Frame-Options/etc. — ALWAYS active.
        // `csp_middleware` (formerly slot 31, gated on `enable_csp`) used to also run here,
        // setting the same Content-Security-Policy header without a nonce — since it was
        // applied after this one, its `insert()` silently overwrote the nonce-bearing header
        // from this middleware on every response where `.with_csp()` was used (the normal,
        // documented usage). Removed 2026-09-21: this middleware already covers everything
        // csp_middleware did, correctly. See `diagramme/anomalies.md` entry E3.
        entries.push(with_engine!(
            SLOT_SECURITY_HEADERS,
            "SecurityHeaders",
            engine,
            security_headers_middleware
        ));
        if !self.features.enable_cache {
            entries.push(with_engine!(
                SLOT_CACHE,
                "NoCache",
                engine,
                dev_no_cache_middleware
            ));
        }
        entries.push(self.session_entry(&engine));
        entries.push(session_upgrade_entry(self.session_duration));
        // Reads the signed-in account from the DB, injects CurrentUser
        entries.push(MiddlewareEntry {
            slot: SLOT_AUTH,
            name: "Auth",
            apply: Box::new(|r| r.layer(axum::middleware::from_fn(auth_middleware))),
        });
        // ALWAYS enabled, after Session
        entries.push(with_engine!(SLOT_CSRF, "CSRF", engine, csrf_middleware));
        // After CSRF — injects the honeypot field name extension
        if self.anti_bot {
            entries.push(with_engine!(
                SLOT_ANTI_BOT,
                "AntiBot",
                engine,
                anti_bot_middleware
            ));
        }

        check_slots(&entries)?;
        // Descending sort: highest slot → first `.layer()` → most internal.
        // In Axum, last `.layer()` = outermost = first executed on the request.
        entries.sort_by_key(|b| std::cmp::Reverse(b.slot));
        log_stack(&entries);

        Ok(entries
            .into_iter()
            .fold(router, |router, entry| (entry.apply)(router)))
    }

    /// The session stores, created once before the engine: the database
    /// store given, and the memory store that backs its authenticated
    /// sessions up in it — none when the developer plugged in their own.
    pub(crate) fn session_stores(
        &self,
        database: Option<Arc<RuniqueSessionStore>>,
    ) -> SessionStores {
        let memory = self.session_applicator.is_none().then(|| {
            #[cfg_attr(not(feature = "orm"), allow(unused_mut))]
            let mut builder = CleaningMemoryStore::default()
                .with_watermarks(self.session_low_watermark, self.session_high_watermark)
                .with_exclusive_login(self.exclusive_login);
            #[cfg(feature = "orm")]
            if let Some(database) = &database {
                builder = builder.with_db_fallback(database.clone());
            }
            let store = Arc::new(builder);
            store.spawn_cleanup(tokio::time::Duration::from_secs(
                self.session_cleanup_interval_secs,
            ));
            store
        });
        SessionStores { memory, database }
    }

    /// The session layer, before CSRF (CSRF depends on it): the developer's
    /// store, or the engine's memory store.
    fn session_entry(&mut self, engine: &AEngine) -> MiddlewareEntry {
        let dev = engine.config.dev;
        let applicator = self.session_applicator.take();
        let anon_duration = self.anonymous_session_duration;
        let store_for_layer = engine.sessions.memory.clone();
        MiddlewareEntry {
            slot: SLOT_SESSION,
            name: "Session",
            apply: Box::new(move |r: Router| match applicator {
                Some(apply_fn) => apply_fn(r, dev, anon_duration),
                None => {
                    let store = store_for_layer
                        .expect("created by `session_stores` when no store is plugged in");
                    let layer = SessionManagerLayer::new((*store).clone())
                        .with_secure(!dev)
                        .with_http_only(true)
                        .with_same_site(tower_sessions::cookie::SameSite::Strict)
                        .with_expiry(Expiry::OnInactivity(anon_duration));
                    r.layer(layer)
                }
            }),
        }
    }
}

/// Upgrades the session's lifetime once the user is signed in.
fn session_upgrade_entry(duration: Duration) -> MiddlewareEntry {
    // Single source of truth: login reads the same value (builder) as this middleware,
    // through the OnceLock set here → cookie/DB/refresh can no longer drift apart (AM3).
    crate::auth::session::set_auth_session_ttl_secs(duration.whole_seconds());
    MiddlewareEntry {
        slot: SLOT_SESSION_UPGRADE,
        name: "SessionTtlUpgrade",
        apply: Box::new(move |r| {
            r.layer(axum::middleware::from_fn_with_state(
                duration,
                session_ttl_upgrade,
            ))
        }),
    }
}

/// Refuses two middlewares on one slot instead of applying them in an
/// undefined relative order (the exact class of silent bug this slot system
/// exists to prevent — see the CSRF/Session story at the top of this file).
/// Most likely trigger: enough custom middlewares registered that
/// `SLOT_CUSTOM_BASE + i` walks into a fixed slot (e.g. 6+ custom
/// middlewares reaches 25 = SLOT_OPEN_REDIRECT).
fn check_slots(entries: &[MiddlewareEntry]) -> Result<(), BuildError> {
    let mut seen: std::collections::HashMap<u16, &'static str> = std::collections::HashMap::new();
    for entry in entries {
        if let Some(existing) = seen.insert(entry.slot, entry.name) {
            return Err(BuildError::validation(crate::translation::tf(
                runique_i18n::keys::middleware::error::SLOT_COLLISION,
                &[
                    existing.to_string(),
                    entry.name.to_string(),
                    entry.slot.to_string(),
                ],
            )));
        }
    }
    Ok(())
}

fn log_stack(entries: &[MiddlewareEntry]) {
    if let Some(level) = crate::log::log_level!(builder.middleware) {
        crate::runique_log!(level, count = entries.len(), "middleware stack");
        for entry in entries {
            crate::runique_log!(level, slot = entry.slot, name = %entry.name, "middleware slot");
        }
    }
}
