//! The bridge between the typed `ModelAdmin<E>` and the type-erased
//! `AdminSite`: what a request can ask of a model without knowing `E`.
use super::{
    admin_builder::{ListPage, ModelAdmin},
    list::ListQuery,
};
use crate::auth::current_user::CurrentUser;
use crate::auth::{
    gate::{
        GateCtx, PkOf, Refusal,
        check::{authorize_list, authorize_view},
        refusal::RefusalReason,
        rights::CrudAction,
    },
    meta::ModelMeta,
};
use crate::utils::aliases::ADb;
use sea_orm::{
    sea_query::SelectStatement,
    {DbErr, IdenStatic, Iterable, PrimaryKeyToColumn},
};

/// Column aliases of `count_query`.
pub(crate) const COUNT_RESOURCE: &str = "resource";
pub(crate) const COUNT_TOTAL: &str = "total";

/// Why an admin request couldn't be served.
#[derive(Debug)]
pub(crate) enum AdminError {
    /// The gate said no: the same answer whatever the reason.
    Refused(Refusal),
    /// The database failed: traced by the handler, never shown as is.
    Db(DbErr),
}

impl From<Refusal> for AdminError {
    fn from(refusal: Refusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<DbErr> for AdminError {
    fn from(error: DbErr) -> Self {
        Self::Db(error)
    }
}

/// A resource key as a title: `doc_page` → `Doc page`.
pub(crate) fn readable(key: &str) -> String {
    let spaced = key.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => spaced,
    }
}

/// What the site needs from a `ModelAdmin` without knowing `E`.
#[async_trait::async_trait]
pub(crate) trait RegisteredModel: Send + Sync {
    fn key(&self) -> &'static str;

    /// The primary key's column, as named in a list row.
    fn pk_column(&self) -> &'static str;

    fn check(&self) -> Result<(), String>;

    /// The name shown in the navigation: the builder's `title()`, else the
    /// model's plural (or singular) verbose name, else its key made readable.
    fn title(&self) -> String;

    /// Whether the navigation lists it for `user`: at least one right on it,
    /// as the gate resolves them. Listing opens nothing: each page still asks
    /// the gate.
    fn visible_to(&self, user: &CurrentUser) -> bool;

    /// Whether this model can stand in for the framework's accounts admin:
    /// the `eihwaz_users` table, with every protection of the base model.
    fn keeps_account_rules(&self) -> bool;

    /// `user`'s rights on this model, with its own key and framework flag:
    /// the one way to resolve them without knowing `E` .
    fn rights(&self, user: &CurrentUser) -> CrudAction;

    /// `SELECT '<key>' AS resource, COUNT(*) AS total` over the rows `user` may
    /// read, or `None` without the read right: the dashboard unites them into
    /// one query.
    fn count_query(&self, db: &ADb, user: &CurrentUser) -> Option<SelectStatement>;
    async fn list(
        &self,
        db: &ADb,
        user: &CurrentUser,
        query: &ListQuery,
        site_page_size: u64,
    ) -> Result<ListPage, AdminError>;
    /// One row's page, `id` as it came in the URL.
    async fn detail(
        &self,
        db: &ADb,
        user: &CurrentUser,
        id: &str,
    ) -> Result<serde_json::Value, AdminError>;
}

#[async_trait::async_trait]
impl<E: ModelMeta> RegisteredModel for ModelAdmin<E> {
    fn key(&self) -> &'static str {
        E::KEY
    }

    fn pk_column(&self) -> &'static str {
        E::PrimaryKey::iter()
            .next()
            .map_or("id", |key| key.into_column().as_str())
    }

    fn check(&self) -> Result<(), String> {
        self.validate()
    }

    fn rights(&self, user: &CurrentUser) -> CrudAction {
        CrudAction::resolve(user, E::KEY)
    }

    fn title(&self) -> String {
        if let Some(title) = &self.title {
            return title.clone();
        }
        let schema = E::schema();
        schema
            .verbose_name_plural
            .or(schema.verbose_name)
            .unwrap_or_else(|| readable(E::KEY))
    }

    fn visible_to(&self, user: &CurrentUser) -> bool {
        let rights = self.rights(user);
        rights.can_create
            || rights.can_read
            || rights.can_read_own
            || rights.can_update
            || rights.can_update_own
            || rights.can_delete
            || rights.can_delete_own
    }

    fn keeps_account_rules(&self) -> bool {
        use sea_orm::EntityName;
        E::default().table_name() == crate::auth::user::Entity.table_name()
            && E::KEY == <crate::auth::user::Entity as ModelMeta>::KEY
            && E::IS_ACCOUNT_TABLE
            && E::IS_FRAMEWORK_TABLE
            && E::SECRET.iter().any(|c| c.as_str() == "password")
    }

    fn count_query(&self, db: &ADb, user: &CurrentUser) -> Option<SelectStatement> {
        use crate::auth::gate::Guarded;
        use sea_orm::sea_query::{Asterisk, Expr, Func, SimpleExpr};
        use sea_orm::{QueryFilter, QuerySelect, QueryTrait};
        let proof = authorize_list::<E>(&GateCtx { db, user }).ok()?;
        Some(
            E::find()
                .filter(proof.conditions())
                .select_only()
                .column_as(Expr::val(E::KEY), COUNT_RESOURCE)
                .column_as(
                    SimpleExpr::from(Func::count(Expr::col(Asterisk))),
                    COUNT_TOTAL,
                )
                .into_query(),
        )
    }

    async fn list(
        &self,
        db: &ADb,
        user: &CurrentUser,
        query: &ListQuery,
        site_page_size: u64,
    ) -> Result<ListPage, AdminError> {
        let proof = authorize_list::<E>(&GateCtx { db, user })?;
        Ok(self.fetch_list(db, &proof, query, site_page_size).await?)
    }

    async fn detail(
        &self,
        db: &ADb,
        user: &CurrentUser,
        id: &str,
    ) -> Result<serde_json::Value, AdminError> {
        let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
        let id = id.parse::<PkOf<E>>().map_err(|_| not_found())?;
        let proof = authorize_view::<E>(&GateCtx { db, user }, id)?;
        self.fetch_detail(db, &proof).await?.ok_or_else(not_found)
    }
}
