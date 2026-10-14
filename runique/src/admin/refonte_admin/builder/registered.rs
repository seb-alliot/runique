//! The bridge between the typed `ModelAdmin<E>` and the type-erased
//! `AdminSite`: what a request can ask of a model without knowing `E`.
use super::{
    admin_builder::{ListPage, ModelAdmin},
    list::ListQuery,
};
use crate::admin::refonte_admin::gate::PkOf;
use crate::admin::refonte_admin::gate::refusal::RefusalReason;
use crate::admin::refonte_admin::{
    gate::{
        GateCtx, Refusal,
        check::{authorize_list, authorize_view},
    },
    meta::ModelMeta,
};
use crate::auth::session::CurrentUser;
use crate::utils::aliases::ADb;
use sea_orm::DbErr;

/// Why an admin request couldn't be served.
#[derive(Debug)]
pub(crate) enum AdminError {
    /// The gate said no:the same answer whatever the reason.
    Refused(Refusal),
    /// The database failes: traced by the handler, never shwon as is.
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

/// What the site nedds from a `ModelAdmin` without knowing `E` .
#[async_trait::async_trait]
pub(crate) trait RegisteredModel: Send + Sync {
    fn key(&self) -> &'static str;
    fn check(&self) -> Result<(), String>;
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

    fn check(&self) -> Result<(), String> {
        self.validate()
    }

    async fn list(
        &self,
        db: &ADb,
        user: &CurrentUser,
        query: &ListQuery,
        site_page_site: u64,
    ) -> Result<ListPage, AdminError> {
        let proof = authorize_list::<E>(&GateCtx { db, user })?;
        Ok(self.fetch_list(db, &proof, query, site_page_site).await?)
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
