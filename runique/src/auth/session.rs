//! User session, admin authentication, and authentication traits.
use crate::admin::permissions::{Groupe, Permission, pull_groupes_db};
use crate::auth::guard::{cache_permissions, evict_permissions, get_permissions};
use crate::auth::user_trait::RuniqueUser;
use crate::context::RequestExtensions;
use crate::middleware::session::session_db::RuniqueSessionStore;
use crate::utils::constante::{
    admin_key::admin_context::permission::GROUPES,
    session_key::session::{
        SESSION_ACTIVE_KEY, SESSION_USER_ID_KEY, SESSION_USER_IS_STAFF_KEY,
        SESSION_USER_IS_SUPERUSER_KEY, SESSION_USER_USERNAME_KEY,
    },
};
use crate::utils::pk::Pk;
use axum::{extract::Request, middleware::Next, response::Response};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use tower_sessions::Session;

// ═══════════════════════════════════════════════════════════════
// AdminAuth — trait + result
// ═══════════════════════════════════════════════════════════════

/// Data returned after a successful admin authentication
#[derive(Debug, Clone)]
pub struct AdminLoginResult {
    pub user_id: Pk,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
}

/// Trait to implement for plugging in admin login verification
///
/// Returns `None` if:
/// - The user does not exist
/// - The password is incorrect
/// - The account is inactive
/// - The user does not have admin rights
///
/// ## Quick implementation with `DefaultAdminAuth`:
/// ```rust,ignore
/// use runique::auth::DefaultAdminAuth;
///
/// .with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
/// ```
#[async_trait::async_trait]
pub trait AdminAuth: Send + Sync + 'static {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
        db: &DatabaseConnection,
    ) -> Option<AdminLoginResult>;
}

// ═══════════════════════════════════════════════════════════════
// UserEntity — DB trait
// ═══════════════════════════════════════════════════════════════

/// Database-side trait: how to retrieve a user by username.
///
/// ```rust,ignore
/// impl UserEntity for users::Entity {
///     type Model = users::Model;
///
///     async fn find_by_username(
///         db: &DatabaseConnection,
///         username: &str,
///     ) -> Option<Self::Model> {
///         users::Entity::find()
///             .filter(users::Column::Username.eq(username))
///             .one(db)
///             .await
///             .ok()
///             .flatten()
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait UserEntity: Send + Sync + 'static {
    /// The model returned by the query (must implement `RuniqueUser`)
    type Model: RuniqueUser;

    /// Searches for a user by id in the database
    async fn find_by_id(db: &DatabaseConnection, id: crate::utils::pk::Pk) -> Option<Self::Model>;
    /// Searches for a user by username in the database
    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model>;
    /// Searches for a user by email in the database
    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model>;

    /// Updates the password of a user identified by their email.
    ///
    /// `new_hash` is already hashed (Prisme forms automatically hash password fields).
    async fn update_password(
        db: &DatabaseConnection,
        email: &str,
        new_hash: &str,
    ) -> Result<(), sea_orm::DbErr>;
}

// ═══════════════════════════════════════════════════════════════
// DefaultAdminAuth<E>
// ═══════════════════════════════════════════════════════════════

/// Generic adapter that transforms any entity implementing `UserEntity` into `AdminAuth`.
pub struct DefaultAdminAuth<E: UserEntity>(PhantomData<E>);

impl<E: UserEntity> DefaultAdminAuth<E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E: UserEntity> Default for DefaultAdminAuth<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<E: UserEntity> AdminAuth for DefaultAdminAuth<E> {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
        db: &DatabaseConnection,
    ) -> Option<AdminLoginResult> {
        // 1. Retrieve the user from the DB
        let user = E::find_by_username(db, username).await?;

        // 2. Check admin access rights + active account
        if !user.can_access_admin() {
            return None;
        }

        // 3. Check password (Argon2)
        if !crate::utils::password::verify(password, user.password_hash()) {
            return None;
        }

        // 4. Everything is fine — return the session info
        Some(AdminLoginResult {
            user_id: user.user_id(),
            username: user.username().to_string(),
            is_staff: user.is_staff(),
            is_superuser: user.is_superuser(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// CurrentUser
// ═══════════════════════════════════════════════════════════════

/// Authenticated user injected into request extensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: Pk,
    pub username: String,
    /// Access to the admin panel (read / limited operations)
    pub is_staff: bool,
    /// Full access — bypasses all admin restrictions
    pub is_superuser: bool,
    /// User groups (each group contains its permissions)
    pub groupes: Vec<Groupe>,
}

impl CurrentUser {
    /// Returns the effective CRUD permissions (Logical union of all groups).
    pub fn permissions_effectives(&self) -> Vec<Permission> {
        let mut agg: std::collections::HashMap<String, Permission> =
            std::collections::HashMap::new();

        for groupe in &self.groupes {
            for perm in &groupe.permissions {
                let entry = agg
                    .entry(perm.resource_key.clone())
                    .or_insert_with(|| Permission {
                        resource_key: perm.resource_key.clone(),
                        can_create: false,
                        can_read: false,
                        can_update: false,
                        can_delete: false,
                        can_update_own: false,
                        can_delete_own: false,
                    });
                entry.can_create |= perm.can_create;
                entry.can_read |= perm.can_read;
                entry.can_update |= perm.can_update;
                entry.can_delete |= perm.can_delete;
                entry.can_update_own |= perm.can_update_own;
                entry.can_delete_own |= perm.can_delete_own;
            }
        }

        agg.into_values().collect()
    }

    /// Checks if the user has a strict global permission (can be refined).
    #[must_use]
    pub fn can_access_resource(&self, resource_key: &str) -> bool {
        if self.is_superuser {
            return true;
        }
        self.permissions_effectives()
            .iter()
            .any(|d| d.resource_key == resource_key && d.can_read)
    }

    /// Checks if the user can access the admin panel.
    #[must_use]
    pub fn can_access_admin(&self) -> bool {
        self.is_staff || self.is_superuser
    }
}

// ═══════════════════════════════════════════════════════════════
// Session helpers
// ═══════════════════════════════════════════════════════════════

/// Checks if the user is authenticated.
pub async fn is_authenticated(session: &Session) -> bool {
    session
        .get::<i32>(SESSION_USER_ID_KEY)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// Checks if the user is authenticated and has admin access.
pub async fn is_admin_authenticated(session: &Session) -> bool {
    if !is_authenticated(session).await {
        return false;
    }

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

    is_staff || is_superuser
}

/// Retrieves the ID of the logged-in user.
pub async fn get_user_id(session: &Session) -> Option<Pk> {
    session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten()
}

/// Retrieves the username of the logged-in user.
pub async fn get_username(session: &Session) -> Option<String> {
    session
        .get::<String>(SESSION_USER_USERNAME_KEY)
        .await
        .ok()
        .flatten()
}

// ═══════════════════════════════════════════════════════════════
// Unified Login
// ═══════════════════════════════════════════════════════════════

/// Logs in a user — loads their rights and groups from the DB.
///
/// If `db_store` is provided, persists the session in DB (multi-device).
/// If `exclusive` is `true`, invalidates other sessions for the user.
///
/// ```rust,ignore
/// login(&session, &db, user.id, &user.username, user.is_staff, user.is_superuser, None, false).await?;
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn login(
    session: &Session,
    db: &DatabaseConnection,
    user_id: Pk,
    username: &str,
    is_staff: bool,
    is_superuser: bool,
    db_store: Option<&RuniqueSessionStore>,
    exclusive: bool,
) -> Result<(), tower_sessions::session::Error> {
    // If another session is already active, perform a clean logout before login
    let existing_id: Option<_> = session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten();
    if let Some(existing) = existing_id
        && existing != user_id
    {
        let _ = logout(session, db_store).await;
    }

    let groupes = pull_groupes_db(db, user_id).await;

    // Memory cache — single access point for load_user_middleware and point 6 (internal)
    cache_permissions(user_id, groupes.clone());

    session.insert(SESSION_USER_ID_KEY, user_id).await?;
    session
        .insert(SESSION_USER_USERNAME_KEY, username.to_string())
        .await?;
    session.insert(SESSION_USER_IS_STAFF_KEY, is_staff).await?;
    session
        .insert(SESSION_USER_IS_SUPERUSER_KEY, is_superuser)
        .await?;

    // DB persistence
    if let Some(store) = db_store {
        let cookie_id = session.id().map(|id| id.to_string()).unwrap_or_default();
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now()
            .naive_utc()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        let _ = store
            .create(&cookie_id, user_id, &session_id, expires_at)
            .await;

        if exclusive {
            let _ = store.invalidate_other_sessions(user_id, &cookie_id).await;
        }
    }

    Ok(())
}

/// Logs in a user starting only from their `user_id` — loads data from the DB.
///
/// Generic shortcut for any authentication flow (registration, OAuth, magic link...)
/// that already has the user identifier without needing to re-send the fields.
///
/// Returns `Ok(())` without creating a session if the account is inactive (`is_active = false`).
///
/// Uses [`BuiltinUserEntity`] for searching. For a custom model, use [`login`] directly.
pub async fn auth_login(
    session: &Session,
    db: &DatabaseConnection,
    user_id: Pk,
) -> Result<(), tower_sessions::session::Error> {
    let Some(user) = crate::auth::user::BuiltinUserEntity::find_by_id(db, user_id).await else {
        return Ok(());
    };
    if !user.is_active() {
        return Ok(());
    }
    let store = RuniqueSessionStore::new(std::sync::Arc::new(db.clone()));
    login(
        session,
        db,
        user.user_id(),
        user.username(),
        user.is_staff(),
        user.is_superuser(),
        Some(&store),
        false,
    )
    .await
}

/// Logs out a user — removes the memory session and the DB entry if provided.
pub async fn logout(
    session: &Session,
    db_store: Option<&RuniqueSessionStore>,
) -> Result<(), tower_sessions::session::Error> {
    // DB deletion before clearing the session (cookie_id still accessible)
    if let Some(store) = db_store {
        let cookie_id = session.id().map(|id| id.to_string()).unwrap_or_default();
        let _ = store.delete(&cookie_id).await;
    }

    // Clear permission cache
    if let Some(user_id) = session.get::<Pk>(SESSION_USER_ID_KEY).await.ok().flatten() {
        evict_permissions(user_id);
    }

    session.remove::<i32>(SESSION_USER_ID_KEY).await?;
    session.remove::<String>(SESSION_USER_USERNAME_KEY).await?;
    session.remove::<bool>(SESSION_USER_IS_STAFF_KEY).await?;
    session
        .remove::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
        .await?;
    session.remove::<Vec<Groupe>>(GROUPES).await?;
    session.remove::<i64>(SESSION_ACTIVE_KEY).await?;
    session.delete().await
}

/// Protects an anonymous session from cleanup.
pub async fn protect_session(
    session: &Session,
    duration_secs: i64,
) -> Result<(), tower_sessions::session::Error> {
    let protect_until = chrono::Utc::now().timestamp().saturating_add(duration_secs);
    session.insert(SESSION_ACTIVE_KEY, protect_until).await
}

/// Removes manual protection from an anonymous session.
pub async fn unprotect_session(session: &Session) -> Result<(), tower_sessions::session::Error> {
    session.remove::<i64>(SESSION_ACTIVE_KEY).await?;
    Ok(())
}

/// Checks if the logged-in user has a given permission.
///
/// Format: `"resource_key.action"` — e.g.: `"users.read"`, `"posts.create"`.
/// Available actions: `create`, `read`, `update`, `delete`, `update_own`, `delete_own`.
/// `"resource_key"` alone (without `.action`) checks any action on the resource.
/// `"any"` checks that the user belongs to at least one group.
///
/// Superusers always have access (bypass).
pub async fn has_permission(session: &Session, permission: &str) -> bool {
    // Superuser bypass
    let is_superuser = session
        .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(false);

    if is_superuser {
        return true;
    }

    let Some(user_id) = get_user_id(session).await else {
        return false;
    };

    let Some(cached) = get_permissions(user_id) else {
        return false;
    };

    if permission == "any" {
        return true; // user_id validated above → authenticated user
    }

    let (resource_key, action) = match permission.find('.') {
        Some(pos) => (&permission[..pos], &permission[pos + 1..]),
        None => (permission, "any"),
    };

    for groupe in &cached.groupes {
        for perm in &groupe.permissions {
            if perm.resource_key != resource_key {
                continue;
            }
            let has = match action {
                "create" => perm.can_create,
                "read" => perm.can_read,
                "update" => perm.can_update,
                "delete" => perm.can_delete,
                "update_own" => perm.can_update_own,
                "delete_own" => perm.can_delete_own,
                "any" => {
                    perm.can_create
                        || perm.can_read
                        || perm.can_update
                        || perm.can_delete
                        || perm.can_update_own
                        || perm.can_delete_own
                }
                _ => false,
            };
            if has {
                return true;
            }
        }
    }

    false
}

// ═══════════════════════════════════════════════════════════════
// Axum Middlewares
// ═══════════════════════════════════════════════════════════════

/// Middleware: loads user info into the request extensions.
pub async fn load_user_middleware(
    axum::extract::State(db): axum::extract::State<crate::utils::aliases::ADb>,
    session: Session,
    mut request: Request,
    next: Next,
) -> Response {
    if let (Some(user_id), Some(username)) =
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

        // Groups from cache — DB reload if cache is empty (after clear_cache)
        let groupes = match get_permissions(user_id) {
            Some(cached) => cached.groupes.clone(),
            None => {
                let groupes = pull_groupes_db(&*db, user_id).await;
                cache_permissions(user_id, groupes.clone());
                groupes
            }
        };

        let current_user = CurrentUser {
            id: user_id,
            username,
            is_staff,
            is_superuser,
            groupes,
        };

        let extensions = RequestExtensions::new().with_current_user(current_user);
        extensions.inject_request(&mut request);
    } else if session.id().is_some() {
        let _ = session.delete().await;
    }

    next.run(request).await
}
