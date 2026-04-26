//! Router construction: collects middleware entries, sorts by slot, applies to the router.
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
//!   → Extensions(0) → ErrorHandler(10) → Custom(20+)
//!   → CSP(30) → Cache(40) → Session(50) → CSRF(60)
//!   → Host(70) → Handler

use crate::context::RequestExtensions;
use crate::middleware::session::CleaningMemoryStore;
use crate::middleware::{
    allowed_hosts_middleware, csp_middleware, csrf_middleware, dev_no_cache_middleware,
    error_handler_middleware, security_headers_middleware,
};
use crate::utils::aliases::{AEngine, ARuniqueConfig, ATera};
use axum::{self, Router, middleware};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};

use super::MiddlewareStaging;

// ─── Built-in slots — Guaranteed execution order on the request ───────────────

const SLOT_EXTENSIONS: u16 = 0; // Engine/Tera/Config injection (outermost)
const SLOT_COMPRESSION: u16 = 5; // Compression (external, before any other middleware)
const SLOT_ERROR_HANDLER: u16 = 10; // Catches errors of the WHOLE stack
const SLOT_CUSTOM_BASE: u16 = 20; // Dev's custom middlewares start here
const SLOT_SECURITY_HEADERS: u16 = 30;
const SLOT_SECURITY_CSP: u16 = 31;
const SLOT_CACHE: u16 = 40;
const SLOT_SESSION: u16 = 50; // Before CSRF (CSRF depends on it)
const SLOT_SESSION_UPGRADE: u16 = 55; // After Session (reads/writes in session)
const SLOT_AUTH: u16 = 57; // After Session — loads CurrentUser from the session
const SLOT_CSRF: u16 = 60; // After Session (reads/writes in session)
const SLOT_HOST_VALIDATION: u16 = 70; // Last defense before handler

// ─── MiddlewareEntry ──────────────────────────────────────────────────────────

struct MiddlewareEntry {
    /// Low (0) = external, first executed. High (100+) = internal, close to the handler.
    slot: u16,
    #[allow(dead_code)]
    name: &'static str,
    apply: Box<dyn FnOnce(Router) -> Router + Send>,
}

// ─── apply_to_router ─────────────────────────────────────────────────────────

impl MiddlewareStaging {
    /// Builds the full middleware stack and applies it to the router.
    ///
    /// 1. Collects all entries (built-in + custom), each with a fixed slot
    /// 2. Sorts DESCENDING by slot (highest = most internal, applied first via `.layer()`)
    /// 3. Applies in order — result on request: Extensions → ErrorHandler → Custom
    ///    → CSP → Cache → Session → CSRF → Host → Handler
    pub(crate) fn apply_to_router(
        self,
        router: Router,
        config: ARuniqueConfig,
        engine: AEngine,
        tera: ATera,
    ) -> (Router, Option<Arc<CleaningMemoryStore>>) {
        let debug = config.debug;
        let mut entries: Vec<MiddlewareEntry> = Vec::new();

        // Slot 0: Extensions (Engine, Tera, Config) — outermost
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

        // Slot 5: Compression — before any other middleware
        entries.push(MiddlewareEntry {
            slot: SLOT_COMPRESSION,
            name: "Compression",
            apply: Box::new(|r| r.layer(CompressionLayer::new())),
        });

        // Slot 70: Host validation — last defense before handler
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

        // Slot 50: Session — before CSRF (CSRF depends on it)
        let memory_store: Option<Arc<CleaningMemoryStore>> = {
            let applicator = self.session_applicator;
            let anon_duration = self.anonymous_session_duration;
            let low_wm = self.session_low_watermark;
            let high_wm = self.session_high_watermark;
            let cleanup_secs = self.session_cleanup_interval_secs;
            let exclusive_login = self.exclusive_login;

            let store_arc = if applicator.is_none() {
                let mut builder = CleaningMemoryStore::default()
                    .with_watermarks(low_wm, high_wm)
                    .with_exclusive_login(exclusive_login);

                #[cfg(feature = "orm")]
                {
                    use crate::middleware::session::RuniqueSessionStore;
                    let db_store = Arc::new(RuniqueSessionStore::new(engine.db.clone()));
                    builder = builder.with_db_fallback(db_store);
                }

                let store = Arc::new(builder);
                store.spawn_cleanup(tokio::time::Duration::from_secs(cleanup_secs));
                Some(store)
            } else {
                None
            };

            let store_for_layer = store_arc.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_SESSION,
                name: "Session",
                apply: Box::new(move |r: Router| match applicator {
                    Some(apply_fn) => apply_fn(r, debug, anon_duration),
                    None => {
                        let store = store_for_layer.expect("store created above");
                        let layer = SessionManagerLayer::new((*store).clone())
                            .with_secure(!debug)
                            .with_http_only(true)
                            .with_same_site(tower_sessions::cookie::SameSite::Strict)
                            .with_expiry(Expiry::OnInactivity(anon_duration));
                        r.layer(layer)
                    }
                }),
            });

            store_arc
        };

        // Slot 60: CSRF — ALWAYS enabled, after Session
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

        // Slot 40: Cache control
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

        // Slot 30: Security headers — ALWAYS active
        {
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

        // Slot 31: CSP — only if enabled
        if self.features.enable_csp {
            let eng = engine.clone();
            entries.push(MiddlewareEntry {
                slot: SLOT_SECURITY_CSP,
                name: "CSP",
                apply: Box::new(move |r| {
                    r.layer(middleware::from_fn_with_state(eng, csp_middleware))
                }),
            });
        }

        // Slot 55: Upgrade session TTL if authenticated
        {
            let duration = self.session_duration;
            entries.push(MiddlewareEntry {
                slot: SLOT_SESSION_UPGRADE,
                name: "SessionTtlUpgrade",
                apply: Box::new(move |r| {
                    r.layer(axum::middleware::from_fn_with_state(
                        duration,
                        session_ttl_upgrade,
                    ))
                }),
            });
        }

        // Slot 57: Auth — loads CurrentUser from the session, injects into extensions
        entries.push(MiddlewareEntry {
            slot: SLOT_AUTH,
            name: "Auth",
            apply: Box::new(|r| r.layer(axum::middleware::from_fn(auth_middleware))),
        });

        // Slot 10: Error handler — wraps the WHOLE stack, catches all errors
        if self.features.enable_debug_errors {
            entries.push(MiddlewareEntry {
                slot: SLOT_ERROR_HANDLER,
                name: "ErrorHandler",
                apply: Box::new(|r| r.layer(middleware::from_fn(error_handler_middleware))),
            });
        }

        // Custom middlewares: automatically placed between ErrorHandler and CSP (slots 20+)
        for (i, custom_mw) in self.custom_middlewares.into_iter().enumerate() {
            entries.push(MiddlewareEntry {
                slot: SLOT_CUSTOM_BASE.saturating_add(i as u16),
                name: "Custom",
                apply: custom_mw,
            });
        }

        // Descending sort: highest slot → first `.layer()` → most internal.
        // In Axum, last `.layer()` = outermost = first executed on the request.
        entries.sort_by_key(|b| std::cmp::Reverse(b.slot));

        let mut router = router;
        for entry in entries {
            router = (entry.apply)(router);
        }

        (router, memory_store)
    }
}

// ─── Private middleware handlers ──────────────────────────────────────────────

/// Upgrades session TTL to the authenticated duration when the user is logged in.
async fn session_ttl_upgrade(
    axum::extract::State(duration): axum::extract::State<Duration>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if let Some(session) = req.extensions().get::<tower_sessions::Session>()
        && crate::auth::session::is_authenticated(session).await
    {
        session.set_expiry(Some(Expiry::OnInactivity(duration)));
    }
    next.run(req).await
}

/// Loads `CurrentUser` from the session and injects it into request extensions.
async fn auth_middleware(
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use crate::admin::permissions::Groupe;
    use crate::auth::session::{CurrentUser, get_user_id, get_username};
    use crate::utils::constante::{
        admin_context::permission::GROUPES,
        session_key::session::{SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY},
    };

    if let Some(session) = req.extensions().get::<tower_sessions::Session>().cloned()
        && let (Some(id), Some(username)) =
            (get_user_id(&session).await, get_username(&session).await)
    {
        let is_staff = session
            .get::<bool>(SESSION_USER_IS_STAFF_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        let is_superuser = session
            .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        let groupes = session
            .get::<Vec<Groupe>>(GROUPES)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        let current_user = CurrentUser {
            id,
            username,
            is_staff,
            is_superuser,
            groupes,
        };
        RequestExtensions::new()
            .with_current_user(current_user)
            .inject_request(&mut req);
    }
    next.run(req).await
}
