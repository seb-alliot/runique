//! Runique's built-in user entity (table `eihwaz_users`).
use crate::auth::permissions::pull_groupes_db;
use crate::auth::permissions::rang::Rang;
pub use crate::auth::user_trait::RuniqueUser;
use crate::utils::aliases::ADb;
use crate::utils::config::TraceResult;
use crate::utils::pk::Pk;
use crate::{impl_objects, search};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, entity::prelude::*};

// ─── SeaORM Model ───────────────────────────────────────────────────────────

/// SeaORM model for Runique's built-in user table `eihwaz_users`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "eihwaz_users")]
pub struct Model {
    // `Pk` is a type alias — SeaORM's derive macro infers `auto_increment` from
    // the literal type name (i32/i64/…), so it can't see through the alias and
    // silently defaults to `false`. Explicit per-feature attribute required,
    // matching `schema()` below (i32/i64 auto-increment, Uuid never does).
    #[cfg_attr(feature = "pk-uuid", sea_orm(primary_key, auto_increment = false))]
    #[cfg_attr(not(feature = "pk-uuid"), sea_orm(primary_key, auto_increment = true))]
    pub id: Pk,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_active: bool,
    pub is_dirigeant: bool,
    pub is_admin: bool,
    pub is_superuser: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    /// When the account's owner first took it over through the emailed link.
    /// `None` while the account waits for that; never cleared afterwards. The
    /// database refuses `is_active` without it.
    pub activated_at: Option<chrono::NaiveDateTime>,
    /// Increased to close every session of the account, whatever store holds
    /// them: a session opened under an older value ends at its next request.
    #[sea_orm(default_value = 0)]
    pub session_version: i32,
}

impl_objects!(Entity);

/// SeaORM relations for `eihwaz_users`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Group memberships (`eihwaz_users_groupes`) for this user.
    #[sea_orm(has_many = "crate::auth::permissions::users_groupes::Entity")]
    UsersGroupes,
    /// Authenticated sessions (`eihwaz_sessions`) owned by this user.
    #[sea_orm(has_many = "crate::middleware::session::session_db::Entity")]
    Sessions,
}

impl Related<crate::auth::permissions::users_groupes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersGroupes.def()
    }
}

impl Related<crate::middleware::session::session_db::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// A UUID key comes from the application, never from the database: an
    /// account inserted without one gets it here, whoever inserts it (admin,
    /// CLI, a project's own registration form). Its dates too: a date the
    /// writer sets itself is kept.
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        #[cfg(feature = "pk-uuid")]
        if insert && self.id.is_not_set() {
            self.id = Set(uuid::Uuid::now_v7());
        }
        let now = chrono::Utc::now().naive_utc();
        if insert && !self.created_at.is_set() {
            self.created_at = Set(Some(now));
        }
        if !self.updated_at.is_set() {
            self.updated_at = Set(Some(now));
        }
        Ok(self)
    }
}

// ─── RuniqueUser ─────────────────────────────────────────────────────────────
impl RuniqueUser for Model {
    fn user_id(&self) -> Pk {
        self.id
    }
    fn username(&self) -> &str {
        &self.username
    }
    fn email(&self) -> &str {
        &self.email
    }
    fn is_active(&self) -> bool {
        self.is_active
    }
    fn can_sign_in(&self) -> bool {
        self.is_active && self.activated_at.is_some()
    }
    fn is_superuser(&self) -> bool {
        self.is_superuser
    }
    fn session_version(&self) -> i32 {
        self.session_version
    }
}

// ─── Account lookups ─────────────────────────────────────────────────────────
/// Access to the accounts of `eihwaz_users` — the framework's only user model,
/// extended with `extend!{ table: "eihwaz_users", ... }`.
pub struct BuiltinUserEntity;

impl BuiltinUserEntity {
    /// The account with this id.
    pub async fn find_by_id(db: &ADb, id: Pk) -> Option<Model> {
        Self::lookup_by_id(db, id)
            .await
            .trace(
                crate::utils::runique_log::get_log()
                    .db
                    .as_ref()
                    .and_then(|d| d.query),
                "find user by id",
            )
            .flatten()
    }

    /// The same lookup, a database failure kept apart from "no such account".
    /// The middlewares that close the session of a missing account read it
    /// this way: an outage must not sign everybody out.
    pub(crate) async fn lookup_by_id(db: &ADb, id: Pk) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// The account with this username.
    pub async fn find_by_username(db: &ADb, username: &str) -> Option<Model> {
        search!(Entity => Username eq username)
            .first(db)
            .await
            .trace(
                crate::utils::runique_log::get_log()
                    .db
                    .as_ref()
                    .and_then(|d| d.query),
                "find user by username",
            )
            .flatten()
    }

    /// The account with this email.
    pub async fn find_by_email(db: &ADb, email: &str) -> Option<Model> {
        search!(Entity => Email eq email)
            .first(db)
            .await
            .trace(
                crate::utils::runique_log::get_log()
                    .db
                    .as_ref()
                    .and_then(|d| d.query),
                "find user by email",
            )
            .flatten()
    }

    /// Sets the password (already hashed) of the account with this id — the
    /// path for flows where the id comes from a secret (a reset token), never
    /// from a field the client controls. Open sessions stay: signing in rehashes
    /// through here, and that must close nothing. To close them,
    /// [`close_user_sessions`](crate::engine::RuniqueEngine::close_user_sessions).
    pub async fn update_password_by_id(
        db: &ADb,
        id: Pk,
        new_hash: &str,
    ) -> Result<(), sea_orm::DbErr> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User not found".into()))?;

        let mut active: ActiveModel = user.into();
        active.password = Set(new_hash.to_string());
        active.update(db).await?;
        Ok(())
    }

    /// Activates an account still waiting for its first activation —
    /// `is_active` and `activated_at` together — and returns it, ready for
    /// `login`. An account activated before (active, or deactivated since) is
    /// left as it is (`None`): activation happens once, reactivation is the staff's.
    /// `db` is the connection, or a transaction to create and activate together.
    pub async fn activate_account<C: ConnectionTrait>(
        db: &C,
        id: Pk,
    ) -> Result<Option<Model>, sea_orm::DbErr> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User not found".into()))?;
        if user.activated_at.is_some() {
            return Ok(None);
        }
        let mut active: ActiveModel = user.into();
        active.is_active = Set(true);
        active.activated_at = Set(Some(chrono::Utc::now().naive_utc()));
        active.update(db).await.map(Some)
    }

    /// The owner sets their password through the emailed link: a pending
    /// account is activated by it, any other only gets the new password. Sessions
    /// opened before it end.
    pub async fn set_password_and_activate(
        db: &ADb,
        id: Pk,
        new_hash: &str,
    ) -> Result<(), sea_orm::DbErr> {
        Self::update_password_by_id(db, id, new_hash).await?;
        Self::activate_account(db, id).await?;
        Self::increase_session_version(db, id).await
    }

    /// Ends every session opened before this call, whatever store holds it.
    /// The update reads the current value in the database: two calls at once
    /// both count, and neither can bring back a value already passed.
    pub async fn increase_session_version<C: ConnectionTrait>(
        db: &C,
        id: Pk,
    ) -> Result<(), sea_orm::DbErr> {
        Entity::update_many()
            .col_expr(
                Column::SessionVersion,
                sea_orm::sea_query::ExprTrait::add(
                    sea_orm::sea_query::Expr::col(Column::SessionVersion),
                    1,
                ),
            )
            .filter(Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    /// Performs the password update and first-time activation inside the
    /// caller's transaction.
    pub(crate) async fn set_password_and_activate_in_transaction(
        db: &sea_orm::DatabaseTransaction,
        id: Pk,
        new_hash: &str,
    ) -> Result<(), sea_orm::DbErr> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User not found".into()))?;

        let mut password_update: ActiveModel = user.clone().into();
        password_update.password = Set(new_hash.to_string());
        password_update.update(db).await?;

        if user.activated_at.is_none() {
            let mut activation: ActiveModel = user.into();
            activation.is_active = Set(true);
            activation.activated_at = Set(Some(chrono::Utc::now().naive_utc()));
            activation.update(db).await?;
        }
        Self::increase_session_version(db, id).await
    }
}

/// Authenticates a user by username and password against the built-in user table.
///
/// Returns `None` if the user is not found, the account is inactive, or the password is wrong.
/// A hash made with an algorithm that is no longer the configured one is rewritten on success.
pub async fn authenticate_user(db: &ADb, username: &str, password: &str) -> Option<Model> {
    let user_opt = BuiltinUserEntity::find_by_username(db, username).await;
    // Always run verify regardless of whether the user exists — prevents user enumeration
    // via timing differences (`ct_eq` equivalent at the hash layer).
    let hash = user_opt
        .as_ref()
        .map(|u| u.password.as_str())
        .unwrap_or(crate::utils::password::dummy_hash());
    let password_ok = crate::utils::password::verify(password, hash);
    if password_ok && let Some(user) = user_opt.filter(RuniqueUser::can_sign_in) {
        Some(rehash_if_outdated(db, user, password).await)
    } else {
        None
    }
}

/// Rewrites the hash of `user` with the configured algorithm when it was made
/// with another one. Only possible right after a successful sign-in: it's the
/// one moment the plain password is at hand. A failed rewrite doesn't fail the
/// sign-in — the next one tries again.
async fn rehash_if_outdated(db: &ADb, mut user: Model, password: &str) -> Model {
    if crate::utils::password::is_algorithm_current(&user.password) {
        return user;
    }
    let rehashed = match crate::utils::password::hash(password) {
        Ok(rehashed) => rehashed,
        Err(e) => {
            tracing::error!(user_id = %user.id, error = %e, "password rehash failed");
            return user;
        }
    };
    match BuiltinUserEntity::update_password_by_id(db, user.id, &rehashed).await {
        Ok(()) => user.password = rehashed,
        Err(e) => {
            tracing::error!(user_id = %user.id, error = %e, "saving the rehashed password failed")
        }
    }
    user
}

/// The account behind an admin sign-in: right password, may sign in, and a
/// rank or a staff group. The password is checked first whatever the account,
/// so an unknown or non-admin username takes as long as a real one; the group
/// lookup only follows a right password.
pub async fn authenticate_admin(db: &ADb, username: &str, password: &str) -> Option<Model> {
    let user = authenticate_user(db, username, password).await?;
    if Rang::from(&user) > Rang::Lambda {
        return Some(user);
    }
    pull_groupes_db(db, user.id)
        .await
        .iter()
        .any(|g| g.is_staff)
        .then_some(user)
}

// ─── Form Schema ────────────────────────────────────────────────────────

pub const PRIMARY_KEY: &str = "id";

/// Returns the `ModelSchema` of the `eihwaz_users` table.
/// Used by `#[form(schema = runique_users)]` — no need to declare the entity locally.
pub fn schema() -> crate::migration::schema::ModelSchema {
    #[cfg(feature = "pk-uuid")]
    let pk = crate::migration::PrimaryKeyDef::new("id")
        .uuid()
        .no_auto_increment();
    #[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
    let pk = crate::migration::PrimaryKeyDef::new("id")
        .i64()
        .auto_increment();
    #[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
    let pk = crate::migration::PrimaryKeyDef::new("id")
        .i32()
        .auto_increment();

    crate::migration::ModelSchema::new("EihwazUsers")
        .table_name("eihwaz_users")
        .primary_key(pk)
        .column(
            crate::migration::ColumnDef::new("username")
                .varchar(150)
                .required()
                .unique(),
        )
        .column(
            crate::migration::ColumnDef::new("email")
                .varchar(254)
                .required()
                .unique(),
        )
        .column(
            crate::migration::ColumnDef::new("password")
                .string()
                .required(),
        )
        .column(
            crate::migration::ColumnDef::new("is_active")
                .boolean()
                .required()
                .default(false.into()),
        )
        .column(
            crate::migration::ColumnDef::new("is_dirigeant")
                .boolean()
                .required()
                .default(false.into()),
        )
        .column(
            crate::migration::ColumnDef::new("is_admin")
                .boolean()
                .required()
                .default(false.into()),
        )
        .column(
            crate::migration::ColumnDef::new("is_superuser")
                .boolean()
                .required()
                .default(false.into()),
        )
        .column(
            crate::migration::ColumnDef::new("created_at")
                .datetime()
                .nullable(),
        )
        .column(
            crate::migration::ColumnDef::new("updated_at")
                .datetime()
                .nullable(),
        )
        // Written only by the owner's activation link: never a form field.
        .column(
            crate::migration::ColumnDef::new("activated_at")
                .datetime()
                .nullable()
                .ignore(),
        )
        // Written only by the password reset and `close_user_sessions`.
        .column(
            crate::migration::ColumnDef::new("session_version")
                .integer()
                .required()
                .default(0.into())
                .ignore(),
        )
        .build()
        .unwrap()
}
