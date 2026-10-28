//! The bridge between the typed `ModelAdmin<E>` and the type-erased
//! `AdminSite`: what a request can ask of a model without knowing `E`.
use super::{
    admin_builder::{ListPage, ModelAdmin, OfferedAction, prefill},
    bulk_form::{BULK_ACTION, BULK_IDS, BulkForm},
    confirm_form::ConfirmForm,
    list::ListQuery,
};
use crate::auth::current_user::CurrentUser;
use crate::auth::{
    gate::{
        Delete, GateCtx, PkOf, Refusal, ResetPassword, Update,
        check::{authorize_create, authorize_list, authorize_row, authorize_view},
        refusal::RefusalReason,
        rights::{CrudAction, ReadableColumns},
        save::{BulkChange, bulk_as},
    },
    meta::ModelMeta,
};
use crate::context::template::Request;
use crate::forms::field::FormAccess;
use crate::forms::validation_form::ValidationForm;
use crate::utils::aliases::ADb;
use crate::utils::constante::admin_key::admin_form;
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

/// What `user` may see of one model's history: granted with the full read
/// right only (`read_own` alone can't tell whose a past row was).
pub(crate) struct HistoryScope {
    pub key: &'static str,
    pub title: String,
    /// The columns a diff may show.
    pub readable: ReadableColumns,
    /// Whether the label kept with each entry may be shown: it is a column's
    /// value, so it follows that column's read right.
    pub label_readable: bool,
}

/// What a grouped action came to.
pub(crate) enum BulkOutcome {
    Updated(u64),
    Deleted(u64),
    /// Nothing written: the gate's one message, or a malformed request.
    Refused(String),
}

/// What a form page (create, edit) shows next.
pub(crate) enum FormOutcome {
    /// The form to render: filled from the row, or as sent with its errors.
    Show(serde_json::Value),
    /// Written: the page redirects. On the accounts' table, the email the
    /// form wrote, for the welcome link a created account gets.
    Saved(Option<String>),
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
    /// What `user` may see of this model's history, or `None`.
    fn history_scope(&self, db: &ADb, user: &CurrentUser) -> Option<HistoryScope>;
    /// One row's page, `id` as it came in the URL.
    async fn detail(
        &self,
        db: &ADb,
        user: &CurrentUser,
        id: &str,
    ) -> Result<serde_json::Value, AdminError>;
    /// The create page: on GET the empty form, on POST the form sent,
    /// written through the gate as `user`.
    async fn create(&self, req: &Request, user: &CurrentUser) -> Result<FormOutcome, AdminError>;
    /// Whether this is the accounts' table: its rows have a password to reset.
    fn is_account_table(&self) -> bool;
    /// The columns an admin's grid offers on this model: never a secret nor
    /// the key, and on the accounts never a rank nor the activation, which
    /// have their own operations.
    fn grid_columns(&self) -> Vec<&'static str>;
    /// Whether `user` may send row `id`'s owner a reset link: the gate's
    /// `ResetPassword` on this account (the rank rule included) and the
    /// confirmation's CSRF token. `Some(account)` when granted.
    async fn reset_password(
        &self,
        req: &Request,
        user: &CurrentUser,
        id: &str,
    ) -> Result<Option<crate::utils::pk::Pk>, AdminError>;
    /// A grouped action on the checked rows, sent by the list's bar: all of
    /// them or none, through the gate as `user`. At most a page of rows.
    async fn bulk(
        &self,
        req: &Request,
        user: &CurrentUser,
        site_page_size: u64,
    ) -> Result<BulkOutcome, AdminError>;
    /// Row `id`'s delete page: on GET the confirmation, on POST the delete
    /// through the gate as `user`.
    async fn delete(
        &self,
        req: &Request,
        user: &CurrentUser,
        id: &str,
    ) -> Result<FormOutcome, AdminError>;
    /// Row `id`'s edit page: on GET its form filled from the row, on POST the
    /// form sent, written through the gate as `user`.
    async fn edit(
        &self,
        req: &Request,
        user: &CurrentUser,
        id: &str,
    ) -> Result<FormOutcome, AdminError>;
}

#[async_trait::async_trait]
impl<E: ModelMeta> RegisteredModel for ModelAdmin<E>
where
    E::ActiveModel: Send,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
{
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
        let mut page = self.fetch_list(db, &proof, query, site_page_size).await?;
        // The bar offers a declared action only where the gate would let it
        // write: the update right, the column ticked, never the owner through
        // `_own` alone. It decides again on each row.
        let rights = user.rights_on(E::KEY);
        let crud = self.rights(user);
        if crud.can_update || crud.can_update_own {
            page.actions = self
                .list
                .actions
                .iter()
                .enumerate()
                .filter(|(_, a)| {
                    let name = a.col.as_str();
                    rights.columns.can_update(name)
                        && (crud.can_update || E::OWNER.is_none_or(|o| o.as_str() != name))
                })
                .map(|(index, a)| OfferedAction {
                    index,
                    label: a.label.clone(),
                })
                .collect();
        }
        Ok(page)
    }

    fn history_scope(&self, db: &ADb, user: &CurrentUser) -> Option<HistoryScope> {
        if !self.rights(user).can_read {
            return None;
        }
        let proof = authorize_list::<E>(&GateCtx { db, user }).ok()?;
        let readable = proof.readable().clone();
        let label_readable = E::LABEL.is_some_and(|c| readable.allows(c.as_str()));
        Some(HistoryScope {
            key: E::KEY,
            title: self.title(),
            readable,
            label_readable,
        })
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

    async fn create(&self, req: &Request, user: &CurrentUser) -> Result<FormOutcome, AdminError> {
        let db = &req.engine.db;
        authorize_create::<E>(&GateCtx { db, user })?;

        let mut form = req.form::<E::Form>();
        let forms = form.get_form_mut();
        self.shape_for_create(forms, &user.rights_on(E::KEY).columns);
        // Named at once: the page renders it with its name, the POST sends it back.
        forms.set_name(admin_form::CREATE);
        // A create points at no row: nothing in the URL may stand for a key.
        forms.path_params.clear();

        if req.method.is_safe() {
            return Ok(FormOutcome::Show(shown(forms)));
        }
        save_through_gate::<E>(form, req, admin_form::CREATE).await
    }

    fn is_account_table(&self) -> bool {
        E::IS_ACCOUNT_TABLE
    }

    fn grid_columns(&self) -> Vec<&'static str> {
        const ACCOUNT_RULES: [&str; 5] = [
            "is_superuser",
            "is_dirigeant",
            "is_admin",
            "is_active",
            "activated_at",
        ];
        E::Column::iter()
            .map(|c| c.as_str())
            .filter(|name| !E::SECRET.iter().any(|s| s.as_str() == *name))
            .filter(|name| !E::PrimaryKey::iter().any(|k| k.into_column().as_str() == *name))
            .filter(|name| !(E::IS_ACCOUNT_TABLE && ACCOUNT_RULES.contains(name)))
            .collect()
    }

    async fn reset_password(
        &self,
        req: &Request,
        user: &CurrentUser,
        id: &str,
    ) -> Result<Option<crate::utils::pk::Pk>, AdminError> {
        let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
        if !E::IS_ACCOUNT_TABLE {
            return Err(not_found());
        }
        let key = id.parse::<PkOf<E>>().map_err(|_| not_found())?;
        authorize_row::<ResetPassword, E>(
            &GateCtx {
                db: &req.engine.db,
                user,
            },
            key,
        )
        .await?;
        let mut form = req.form::<ConfirmForm<E>>();
        form.get_form_mut().set_name(admin_form::RESET_PASSWORD);
        if ValidationForm::try_form(form, req, admin_form::RESET_PASSWORD)
            .await
            .is_err()
        {
            return Ok(None);
        }
        // The accounts' table keys its rows by `Pk`: the same text reads as one.
        id.parse::<crate::utils::pk::Pk>()
            .map(Some)
            .map_err(|_| not_found())
    }

    async fn bulk(
        &self,
        req: &Request,
        user: &CurrentUser,
        site_page_size: u64,
    ) -> Result<BulkOutcome, AdminError> {
        let refused = || BulkOutcome::Refused(Refusal::new(RefusalReason::NoRight).message());
        let mut form = req.form::<BulkForm<E>>();
        form.get_form_mut().set_name(admin_form::BULK);
        let Ok(form) = ValidationForm::try_form(form, req, admin_form::BULK).await else {
            return Ok(refused());
        };
        let value = |name: &str| {
            form.get_form()
                .fields
                .get(name)
                .map(|f| f.value().to_string())
                .unwrap_or_default()
        };
        // Every key must read as the model's: one that doesn't is refused,
        // never skipped. No more rows than a page shows.
        // The parse error is dropped here: it needn't be `Send`, and only its
        // existence matters.
        let ids: Result<Vec<PkOf<E>>, ()> = value(BULK_IDS)
            .split(',')
            .filter(|id| !id.trim().is_empty())
            .map(|id| id.trim().parse::<PkOf<E>>().map_err(|_| ()))
            .collect();
        let Ok(ids) = ids else {
            return Ok(refused());
        };
        if ids.is_empty() {
            return Ok(BulkOutcome::Refused(
                crate::utils::trad::t("admin.bulk.no_selection").into_owned(),
            ));
        }
        if ids.len() as u64 > self.list.page_size(site_page_size) {
            return Ok(refused());
        }
        let action = value(BULK_ACTION);
        let change = if action == "delete" {
            BulkChange::Delete
        } else {
            let Some(declared) = action
                .strip_prefix('a')
                .and_then(|n| n.parse::<usize>().ok())
                .and_then(|n| self.list.actions.get(n))
            else {
                return Ok(refused());
            };
            BulkChange::Set(declared.col, declared.value.clone())
        };
        match bulk_as::<E>(user, &ids, &change, &req.engine.db).await {
            Ok(count) => Ok(match change {
                BulkChange::Delete => BulkOutcome::Deleted(count),
                BulkChange::Set(..) => BulkOutcome::Updated(count),
            }),
            Err(sea_orm::DbErr::Custom(message)) => Ok(BulkOutcome::Refused(message)),
            Err(e) => Err(AdminError::Db(e)),
        }
    }

    async fn delete(
        &self,
        req: &Request,
        user: &CurrentUser,
        id: &str,
    ) -> Result<FormOutcome, AdminError> {
        let db = &req.engine.db;
        let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
        let key = id.parse::<PkOf<E>>().map_err(|_| not_found())?;
        let gate = GateCtx { db, user };
        // Without the delete right on this row, not even the confirmation.
        authorize_row::<Delete, E>(&gate, key.clone()).await?;

        let mut form = req.form::<ConfirmForm<E>>();
        let forms = form.get_form_mut();
        forms.set_name(admin_form::DELETE);
        forms
            .path_params
            .insert(self.pk_column().to_string(), id.to_string());

        if req.method.is_safe() {
            // A confirmation names a row that exists: a rank that needs no
            // row to be allowed still gets none for a missing one.
            let view = authorize_view::<E>(&gate, key)?;
            self.fetch_detail(db, &view).await?.ok_or_else(not_found)?;
            return Ok(FormOutcome::Show(shown(forms)));
        }
        match ValidationForm::try_form(form, req, admin_form::DELETE).await {
            Ok(valid) => match valid.delete(db).await {
                Ok(_) => Ok(FormOutcome::Saved(None)),
                Err(form) => Ok(FormOutcome::Show(shown(form.get_form()))),
            },
            Err(form) => Ok(FormOutcome::Show(shown(form.get_form()))),
        }
    }

    async fn edit(
        &self,
        req: &Request,
        user: &CurrentUser,
        id: &str,
    ) -> Result<FormOutcome, AdminError> {
        let db = &req.engine.db;
        let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
        let key = id.parse::<PkOf<E>>().map_err(|_| not_found())?;
        let gate = GateCtx { db, user };
        // The page leads to an update: without that right on this row, it
        // shows nothing either.
        authorize_row::<Update, E>(&gate, key.clone()).await?;
        let view = authorize_view::<E>(&gate, key)?;
        let readable = view.readable().preserve::<E>(&self.form_columns());

        let mut form = req.form::<E::Form>();
        let forms = form.get_form_mut();
        self.shape_for_edit(forms, &readable, &user.rights_on(E::KEY).columns);
        forms.set_name(admin_form::UPDATE);
        // `save_as` looks for the row under the key's own name; the route
        // calls it `{id}` whatever the model names it.
        forms
            .path_params
            .insert(self.pk_column().to_string(), id.to_string());

        if req.method.is_safe() {
            let row = self.fetch_detail(db, &view).await?.ok_or_else(not_found)?;
            prefill(forms, &row);
            return Ok(FormOutcome::Show(shown(forms)));
        }
        save_through_gate::<E>(form, req, admin_form::UPDATE).await
    }
}

/// The form as the template reads it: its HTML, errors and fields.
fn shown(form: &crate::forms::form::Forms) -> serde_json::Value {
    serde_json::to_value(form).unwrap_or_default()
}

/// Validates the form sent, then saves it: under the admin's constraint,
/// `save` writes through the gate. `Saved`, or the form back with its errors.
async fn save_through_gate<E: ModelMeta>(
    form: E::Form,
    req: &Request,
    name: &'static str,
) -> Result<FormOutcome, AdminError>
where
    E::ActiveModel: Send,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
{
    match ValidationForm::try_form(form, req, name).await {
        Ok(form) => match form.save(&req.engine.db).await {
            Ok(form) => Ok(FormOutcome::Saved(E::IS_ACCOUNT_TABLE.then(|| {
                form.get_form()
                    .fields
                    .get("email")
                    .map(|f| f.value().trim().to_string())
                    .unwrap_or_default()
            }))),
            Err(form) => Ok(FormOutcome::Show(shown(form.get_form()))),
        },
        Err(form) => Ok(FormOutcome::Show(shown(form.get_form()))),
    }
}
