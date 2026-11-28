//! `RuniqueEngine` implementation — construction, middleware attachment, store access.
use crate::{
    engine::{SecurityPolicies, SessionStores},
    utils::aliases::{ADb, ARlockmap, ARuniqueConfig, ATera, new, new_registry},
};
use std::{any::TypeId, collections::HashMap, sync::Arc};
use tera::Tera;

use crate::{config::RuniqueConfig, middleware::MiddlewareConfig};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;
/// Central machine of the framework: groups config, template engine,
/// database, URL registry, and all security policies.
#[derive(Debug)]
pub struct RuniqueEngine {
    /// General application configuration, shared with what reads it outside
    /// the engine.
    pub config: ARuniqueConfig,
    /// Shared Tera instance for read/write.
    pub tera: ATera,
    #[cfg(feature = "orm")]
    /// Database connection (feature `orm`).
    pub db: ADb,
    /// Global registry of named routes (reverse URL).
    pub url_registry: ARlockmap,
    /// Middleware toggles (cache, CSP, CSRF, etc.).
    pub features: MiddlewareConfig,
    /// CSP, allowed hosts, CSRF-exempt paths, Permissions-Policy, trusted proxies.
    pub security: SecurityPolicies,
    /// The memory store (anonymous sessions + CSRF) and the database store
    /// (authenticated sessions, table `eihwaz_sessions`).
    pub sessions: SessionStores,
    /// Extension map — custom external connections registered via `with_custom_db()`.
    /// Keyed by `TypeId`, supports multiple types simultaneously.
    pub extensions: HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>>,
}

impl RuniqueEngine {
    /// Constructs a new engine from config, Tera, and DB connection.
    #[cfg(feature = "orm")]
    pub fn new(config: RuniqueConfig, tera: Tera, db: DatabaseConnection) -> Self {
        // Single load at startup
        let features = MiddlewareConfig::from_env();

        Self {
            config: new(config),
            tera: new(tera),
            db: ADb::from_connection(db),
            url_registry: new_registry(),
            features,
            security: SecurityPolicies::default(),
            sessions: SessionStores::default(),
            extensions: HashMap::new(),
        }
    }

    /// `true` if the in-memory session store has reached its high watermark — a new
    /// session would be refused. Lets a login handler fail fast and clean (503 +
    /// `Retry-After`) instead of letting tower's commit-time `create` error surface
    /// as a generic 500. Returns `false` when Runique owns no memory store.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // At the top of a public login handler:
    /// if engine.session_store_saturated() {
    ///     return StatusCode::SERVICE_UNAVAILABLE.into_response();
    /// }
    /// ```
    #[must_use]
    pub fn session_store_saturated(&self) -> bool {
        self.sessions
            .memory
            .as_ref()
            .is_some_and(|store| store.is_saturated())
    }

    /// Closes every session of `user_id`, whatever the device or the store: its
    /// `session_version` increases, which ends them all at their next request,
    /// in a store plugged in with `with_session_store()` too; then its rows in
    /// `eihwaz_sessions` and its copies in memory go. On a database error the
    /// error is returned and the sessions may survive: try again.
    pub async fn close_user_sessions(
        &self,
        user_id: crate::schema::pk::Pk,
    ) -> Result<(), sea_orm::DbErr> {
        crate::auth::BuiltinUserEntity::increase_session_version(&self.db, user_id).await?;
        if let Some(store) = &self.sessions.database {
            store.invalidate_all(user_id).await?;
        }
        self.forget_user_sessions(user_id).await;
        Ok(())
    }

    /// Drops the memory copies of `user_id`'s sessions. Cleanup only: the
    /// database decides whether a session lives, so this runs once the rows
    /// are gone, as after a password reset's transaction.
    pub(crate) async fn forget_user_sessions(&self, user_id: crate::schema::pk::Pk) {
        if let Some(store) = &self.sessions.memory {
            store.invalidate_user_sessions(user_id).await;
        }
    }

    /// Retrieves a custom extension registered via `with_custom_db()`.
    ///
    /// Returns `Option<Arc<T>>` — `None` if this type was not registered.
    ///
    /// # Examples
    /// ```rust,ignore
    /// // In a handler:
    /// if let Some(mongo) = req.engine.extension::<mongodb::Client>() {
    ///     let col = mongo.database("mydb").collection::<Document>("items");
    /// }
    ///
    /// // Multiple types:
    /// let redis = req.engine.extension::<redis::Client>();
    /// let mongo = req.engine.extension::<mongodb::Client>();
    /// ```
    pub fn extension<T: std::any::Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.clone().downcast::<T>().ok())
    }

    /// Alias for [`extension`](Self::extension) — kept for backward compatibility.
    pub fn custom_db<T: std::any::Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extension::<T>()
    }
}
