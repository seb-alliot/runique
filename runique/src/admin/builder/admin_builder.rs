//! One model's admin: one block per responsibility, each set through a closure.
use super::form::FormConfig;
use super::list::ListConfig;
use super::list::{ListQuery, TOTAL};
use crate::auth::permissions::rights::ColumnRights;
use crate::auth::{
    gate::{Authorized, Guarded, List, View},
    meta::ModelMeta,
};
use crate::forms::form::Forms;
use crate::utils::aliases::ADb;
use crate::utils::constante::session_key::session::{CSRF_TOKEN_KEY, FORM_NAME_KEY};
use sea_orm::{
    ColumnTrait, DbErr, FromQueryResult, IdenStatic, Iterable, PaginatorTrait, PrimaryKeyToColumn,
    QueryFilter, QueryResult, QuerySelect,
};

/// The account table's rank flags: written by the gate alone, never a form field.
const RANK_FLAGS: [&str; 3] = ["is_superuser", "is_dirigeant", "is_admin"];

pub struct ModelAdmin<E: ModelMeta> {
    pub list: ListConfig<E>,
    pub form: FormConfig<E>,
    /// The model's name in the admin, when set with `title()`.
    pub(super) title: Option<String>,
}

/// One filter of a list: its column and the values it offers.
#[derive(serde::Serialize)]
pub(crate) struct ListFilter {
    pub column: &'static str,
    pub values: Vec<String>,
}

/// A declared action the list's bar offers this account.
#[derive(serde::Serialize)]
pub struct OfferedAction {
    /// Its rank among the declared actions, what the bar sends (`a<n>`).
    pub index: usize,
    pub label: String,
}

/// One page of a list, ready for the template.
#[derive(serde::Serialize)]
pub(crate) struct ListPage {
    /// The columns shown, key out: the headers, cut tike the query.
    pub columns: Vec<&'static str>,
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    /// The readable filtered columns, with their values in the proof's scope.
    pub filters: Vec<ListFilter>,
    /// The declared actions this account may run.
    pub actions: Vec<OfferedAction>,
}

/// One row of the list query: the columns shown, plus the window total read
/// as a typed integer (SQLite gives a computed column no type, so the JSON
/// conversion alone would drop it).
struct ListRow {
    fields: serde_json::Value,
    total: i64,
}

impl FromQueryResult for ListRow {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        Ok(Self {
            fields: serde_json::Value::from_query_result(res, pre)?,
            total: res.try_get(pre, TOTAL)?,
        })
    }
}

impl<E: ModelMeta> Default for ModelAdmin<E> {
    fn default() -> Self {
        Self {
            list: ListConfig::default(),
            form: FormConfig::default(),
            title: None,
        }
    }
}

impl<E: ModelMeta> ModelAdmin<E> {
    pub fn new() -> Self {
        Self::default()
    }

    /// A list that declares no `display` shows every column but the secret
    /// ones; on the accounts, only the name, email and state — an account's
    /// other columns (rank, activation date) are never listed by default.
    pub(crate) fn with_default_display(mut self) -> Self {
        use sea_orm::Iterable;
        if self.list.has_display() {
            return self;
        }
        let shown = E::Column::iter().filter(|c| {
            let name = c.as_str();
            if E::IS_ACCOUNT_TABLE {
                ["username", "email", "is_active"].contains(&name)
            } else {
                !E::SECRET.iter().any(|s| s.as_str() == name)
            }
        });
        self.list = self.list.display(shown);
        self
    }

    /// The model's name in the navigation and on its pages. Without it: the
    /// model's `verbose_name_plural`, `verbose_name`, else its table name.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn list(mut self, l: impl FnOnce(ListConfig<E>) -> ListConfig<E>) -> Self {
        self.list = l(self.list);
        self
    }

    pub fn form(mut self, f: impl FnOnce(FormConfig<E>) -> FormConfig<E>) -> Self {
        self.form = f(self.form);
        self
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        if let Err(e) = self.list.validate() {
            errors.push(e);
        }
        if let Err(e) = self.form.validate() {
            errors.push(e);
        }
        if E::IS_ACCOUNT_TABLE {
            // A rank and the activation have their own rules (the rank rule,
            // `activated_at`): never a grouped action.
            for name in RANK_FLAGS.iter().chain(["is_active"].iter()) {
                if self.list.actions.iter().any(|a| a.col.as_str() == *name) {
                    errors.push(format!(
                        "action: `{name}` has its own rules, never a grouped action"
                    ));
                }
            }
            for flag in RANK_FLAGS {
                if self.form.has_field(flag) {
                    errors.push(format!(
                        "fields: `{flag}` is a rank, only ever set by the gate under the rank rule"
                    ));
                }
            }
        }
        for secret in E::SECRET {
            let name = secret.as_str();
            for list in self
                .list
                .lists_using(name)
                .into_iter()
                .chain(self.form.lists_using(name))
            {
                errors.push(format!(
                    "{list}: `{name}` is a secret column, never shown, searched, filtered, sorted or written by the admin"
                ));
            }
        }

        // Without `.form()`, every field of the model's form is written: the
        // action only has to stay off what `.form()` would freeze.
        for action in &self.list.actions {
            let name = action.col.as_str();
            if !self.form.is_empty() && !self.form.writable_on_edit(name) {
                errors.push(format!(
                    "action `{}`: `{name}` is not written by the form on edit (in `fields`, not `readonly` or `create_only`)",
                    action.label
                ));
            }
            if E::OWNER.is_some_and(|owner| owner.as_str() == name) {
                errors.push(format!(
                    "action `{}`: `{name}` is the owner, a transfer is its own operation",
                    action.label
                ));
            }
        }

        // Each declared column needs its field in the model's form, one that
        // can hold the column's type: checked once here, not on every write.
        let mut model_form = crate::forms::Forms::new("");
        <E::Form as crate::forms::FormAccess>::register_fields(&mut model_form);
        for col in self.form.field_columns() {
            let name = col.as_str();
            match model_form.fields.get(name) {
                None => errors.push(format!(
                    "fields: `{name}` is not a field of the model's form (`ModelMeta::Form`)"
                )),
                Some(field) if !field.accepts_column(col.def().get_column_type()) => {
                    errors.push(format!(
                        "fields: `{name}`: the form's field can't hold this column's type"
                    ));
                }
                Some(_) => {}
            }
        }

        if !self.form.is_empty() {
            for col in E::schema().columns {
                let name = col.name.as_str();
                let filled_elsewhere = col.nullable
                    || col.default.is_some()
                    || col.auto_now
                    || col.auto_now_update
                    || col.ignored
                    || (E::IS_ACCOUNT_TABLE && name == "password")
                    // The gate sets it to the account creating the row.
                    || E::OWNER.is_some_and(|owner| owner.as_str() == name);
                if filled_elsewhere || self.form.writable_on_create(name) {
                    continue;
                }
                if E::SECRET.iter().any(|s| s.as_str() == name) {
                    errors.push(format!(
                        "fields: `{name}` is a secret column needed to create a row (NOT NULL, no default): the admin can never fill it, give it a default"
                    ));
                } else {
                    errors.push(format!(
                        "fields: `{name}` is needed to create a row (NOT NULL, no default): add it to `fields` or give it a default"
                ))
                };
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("admin `{}`:\n{}", E::KEY, errors.join("\n")))
        }
    }
    /// One page of the list. Taking the gate's proof, it can't run without
    /// it: the proof's scope narrows every row and the total.
    pub(crate) async fn fetch_list(
        &self,
        db: &ADb,
        proof: &Authorized<List, E>,
        query: &ListQuery,
        site_page_size: u64,
    ) -> Result<ListPage, DbErr> {
        let backend = db.get_database_backend();
        let list = self.list.readable_only(proof.readable());
        let page_size = list.page_size(site_page_size);
        let page = query.page.max(1);
        let rows: Vec<ListRow> = list
            .select(backend, proof.conditions(), query)
            .limit(page_size)
            .offset((page - 1).saturating_mul(page_size))
            .into_model::<ListRow>()
            .all(&**db)
            .await?;

        let total = match rows.first() {
            Some(row) => u64::try_from(row.total).unwrap_or(0),
            None if page > 1 => {
                list.filtered(backend, proof.conditions(), query)
                    .count(&**db)
                    .await?
            }
            None => 0,
        };

        let rows = rows
            .into_iter()
            .map(|mut row| {
                if let Some(fields) = row.fields.as_object_mut() {
                    fields.remove(TOTAL);
                }
                row.fields
            })
            .collect();

        let keys: Vec<&str> = E::PrimaryKey::iter()
            .map(|k| k.into_column().as_str())
            .collect();

        let columns = list
            .display_columns()
            .iter()
            .map(|c| c.as_str())
            .filter(|name| !keys.contains(name))
            .collect();
        let mut filters = Vec::new();
        for col in list.filter_columns() {
            let values = ListConfig::<E>::filter_values(backend, proof.conditions(), *col)
                .into_tuple::<String>()
                .all(&**db)
                .await?;
            filters.push(ListFilter {
                column: col.as_str(),
                values,
            });
        }
        Ok(ListPage {
            columns,
            rows,
            total,
            page,
            page_size,
            filters,
            actions: Vec::new(),
        })
    }

    /// One row's page: the form's columns, read in one query filtered by the
    /// proof's scope. `None` when no row matches, whether it is missing or out
    /// of the user's reach: the caller refuses both alike.
    pub(crate) async fn fetch_detail(
        &self,
        db: &ADb,
        proof: &Authorized<View, E>,
    ) -> Result<Option<serde_json::Value>, DbErr> {
        let Some(id) = proof.id() else {
            return Ok(None);
        };
        // The row's page shows what the list shows, in its order, then what
        // else the form holds; the key comes with the form's columns.
        let mut columns: Vec<E::Column> = self.list.display_columns().to_vec();
        for col in self.form_columns() {
            if !columns.iter().any(|c| c.as_str() == col.as_str()) {
                columns.push(col);
            }
        }
        let columns = proof.readable().preserve::<E>(&columns);
        E::find_by_id(id.clone())
            .filter(proof.conditions())
            .select_only()
            .columns(columns)
            .into_json()
            .one(&**db)
            .await
    }
}

impl<E: ModelMeta> ModelAdmin<E> {
    /// The columns the form pages work on: the declared fields, or without
    /// `.form()` every column but the secret ones. The key comes with them.
    pub(crate) fn form_columns(&self) -> Vec<E::Column> {
        if !self.form.is_empty() {
            return self.form.read_columns();
        }
        E::Column::iter()
            .filter(|c| !E::SECRET.iter().any(|s| s.as_str() == c.as_str()))
            .collect()
    }

    /// The model's form as the edit page shows it: its fields among the
    /// `readable` columns; locked when not ticked for update in `columns`, or
    /// declared read-only or create-only.
    pub(crate) fn shape_for_edit(
        &self,
        form: &mut Forms,
        readable: &[E::Column],
        columns: &ColumnRights,
    ) {
        self.shape(
            form,
            |name| readable.iter().any(|c| c.as_str() == name),
            |name| {
                columns.can_update(name)
                    && (self.form.is_empty() || self.form.writable_on_edit(name))
            },
        );
    }

    /// The model's form as the create page shows it: its fields; locked when
    /// not ticked for create in `columns`, or declared read-only.
    pub(crate) fn shape_for_create(&self, form: &mut Forms, columns: &ColumnRights) {
        self.shape(
            form,
            |_| true,
            |name| {
                columns.can_create(name)
                    && (self.form.is_empty() || self.form.writable_on_create(name))
            },
        );
    }

    /// The fields of the model's form `shown` lets through, locking those not
    /// `writable`. Without `.form()`, every field of the model's form; with
    /// it, only the declared ones, in their declared order, with their labels.
    /// A field left out is neither shown nor written.
    fn shape(
        &self,
        form: &mut Forms,
        shown: impl Fn(&str) -> bool,
        writable: impl Fn(&str) -> bool,
    ) {
        let kept = |name: &str| self.form.is_empty() || self.form.has_field(name);
        form.fields.retain(|name, _| {
            name == CSRF_TOKEN_KEY || name == FORM_NAME_KEY || (kept(name) && shown(name))
        });
        if !self.form.is_empty() {
            let position = |name: &str| {
                self.form
                    .field_columns()
                    .iter()
                    .position(|c| c.as_str() == name)
            };
            form.fields
                .sort_by(|a, _, b, _| position(a).cmp(&position(b)));
        }
        for (name, field) in form.fields.iter_mut() {
            if name == CSRF_TOKEN_KEY || name == FORM_NAME_KEY {
                continue;
            }
            if !writable(name) {
                field.set_readonly(true, None);
            }
            if let Some(label) = self.form.label_of(name) {
                field.set_label(label);
            }
        }
    }
}

/// Puts `row`'s values in the fields of the same name, as the page shows
/// them; a NULL leaves its field empty.
pub(crate) fn prefill(form: &mut Forms, row: &serde_json::Value) {
    let Some(row) = row.as_object() else {
        return;
    };
    for (name, field) in form.fields.iter_mut() {
        let text = match row.get(name) {
            None | Some(serde_json::Value::Null) => continue,
            Some(serde_json::Value::String(text)) => text.clone(),
            Some(other) => other.to_string(),
        };
        field.set_value(&text);
    }
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::{ListPage, ModelAdmin};
    use crate::admin::builder::list::ListQuery;
    use crate::auth::current_user::CurrentUser;
    use crate::auth::gate::GateCtx;
    use crate::auth::gate::check::{authorize_list, authorize_view};
    use crate::auth::gate::test_support::fixtures::note::{Column, Entity};
    use crate::auth::gate::test_support::fixtures::{db, pk, staff};
    use crate::auth::permissions::rang::Rang;
    use crate::auth::user;
    use sea_orm::Order;
    /// One column of each kind the create check looks at.
    mod article {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "articles")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub title: String,
            pub status: String,
            pub created_at: DateTime,
            pub updated_at: DateTime,
            pub summary: Option<String>,
            pub token: String,
            pub password: String,
            pub published: bool,
            pub owner_id: i32,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        /// A field for every column a test may put in `fields`.
        #[derive(serde::Serialize, Debug, Clone)]
        #[serde(transparent)]
        pub struct ArticleForm {
            pub form: crate::forms::Forms,
        }

        impl crate::forms::FormAccess for ArticleForm {
            fn register_fields(form: &mut crate::forms::Forms) {
                use crate::forms::fields::{BooleanField, TextField};
                form.field(&TextField::text("title"));
                form.field(&TextField::text("status"));
                form.field(&TextField::text("summary"));
                form.field(&BooleanField::new("published"));
                // Wrong on purpose: a box can't hold the text column `token`.
                form.field(&BooleanField::new("token"));
            }
            crate::impl_form_access!();
        }

        impl crate::forms::RuniqueForm for ArticleForm {}

        impl crate::forms::FormEntity for ArticleForm {
            type Entity = Entity;
        }

        impl crate::auth::meta::ModelMeta for Entity {
            const KEY: &'static str = "articles";
            type Form = ArticleForm;
            const SECRET: &'static [Column] = &[Column::Password];
            const OWNER: Option<Column> = Some(Column::OwnerId);
            fn schema() -> crate::migration::schema::ModelSchema {
                use crate::migration::ColumnDef;
                crate::migration::ModelSchema::new("Article")
                    .table_name("articles")
                    .primary_key(
                        crate::migration::PrimaryKeyDef::new("id")
                            .i32()
                            .auto_increment(),
                    )
                    .column(ColumnDef::new("title").string().required())
                    .column(
                        ColumnDef::new("status")
                            .string()
                            .required()
                            .default("draft".into()),
                    )
                    .column(
                        ColumnDef::new("created_at")
                            .datetime()
                            .required()
                            .auto_now(),
                    )
                    .column(
                        ColumnDef::new("updated_at")
                            .datetime()
                            .required()
                            .auto_now_update(),
                    )
                    .column(ColumnDef::new("summary").string().nullable())
                    .column(ColumnDef::new("token").string().required().ignore())
                    .column(ColumnDef::new("password").string().required())
                    .column(
                        ColumnDef::new("published")
                            .boolean()
                            .required()
                            .default(false.into()),
                    )
                    .column(ColumnDef::new("owner_id").integer().required())
                    .build()
                    .unwrap()
            }
        }
    }

    #[test]
    fn a_consistent_admin_is_accepted() {
        let admin = ModelAdmin::<Entity>::new()
            .list(|l| l.display([Column::Id, Column::OwnerId]).links([Column::Id]))
            .form(|f| f.fields([Column::OwnerId]));
        assert!(admin.validate().is_ok());
    }

    #[test]
    fn errors_of_every_block_are_reported_at_once() {
        let admin = ModelAdmin::<Entity>::new()
            .list(|l| l.display([Column::Id]).sortable([Column::OwnerId]))
            .form(|f| f.fields([Column::Id]).readonly([Column::OwnerId]));
        let error = admin.validate().unwrap_err();
        assert!(error.contains("notes"), "{error}");
        assert!(error.contains("sortable"), "{error}");
        assert!(error.contains("readonly"), "{error}");
    }

    #[test]
    fn every_rank_flag_is_refused_on_the_account_table() {
        for (column, flag) in [
            (user::Column::IsSuperuser, "is_superuser"),
            (user::Column::IsDirigeant, "is_dirigeant"),
            (user::Column::IsAdmin, "is_admin"),
        ] {
            let admin = ModelAdmin::<user::Entity>::new()
                .form(|f| f.fields([user::Column::Username, column]));
            let error = admin.validate().unwrap_err();
            assert!(error.contains(&format!("`{flag}` is a rank")), "{error}");
        }
    }

    /// A secret column is refused in every list that could show it, search
    /// it, filter it, sort by it or write it: even unshown, a searchable or
    /// sortable password hash gives itself away one guessed prefix at a time.
    #[test]
    fn a_secret_column_is_refused_in_every_list() {
        let p = user::Column::Password;
        let cases: [(&str, ModelAdmin<user::Entity>); 10] = [
            ("display", ModelAdmin::new().list(|l| l.display([p]))),
            ("search", ModelAdmin::new().list(|l| l.search([p]))),
            ("filter", ModelAdmin::new().list(|l| l.filter([p]))),
            ("sortable", ModelAdmin::new().list(|l| l.sortable([p]))),
            (
                "ordering",
                ModelAdmin::new().list(|l| l.ordering([(p, Order::Asc)])),
            ),
            ("fields", ModelAdmin::new().form(|f| f.fields([p]))),
            (
                "create_only",
                ModelAdmin::new().form(|f| f.fields([p]).create_only([p])),
            ),
            (
                "labels",
                ModelAdmin::new().form(|f| f.fields([p]).labels([(p, "Mot de passe")])),
            ),
            (
                "readonly",
                ModelAdmin::new().form(|f| f.fields([p]).readonly([p])),
            ),
            (
                "links",
                ModelAdmin::new().list(|l| l.display([p]).links([p])),
            ),
        ];
        for (list, admin) in cases {
            let error = admin.validate().unwrap_err();
            assert!(
                error.contains(&format!("{list}: `password` is a secret column")),
                "{list}: {error}"
            );
        }
    }

    #[test]
    fn the_account_table_without_its_password_is_accepted() {
        let admin = ModelAdmin::<user::Entity>::new()
            .list(|l| {
                l.display([user::Column::Username])
                    .search([user::Column::Email])
            })
            .form(|f| f.fields([user::Column::Username, user::Column::Email]));
        assert!(admin.validate().is_ok());
    }

    #[test]
    fn an_action_outside_fields_is_refused() {
        let admin = ModelAdmin::<user::Entity>::new()
            .list(|l| l.action(user::Column::IsActive, false, "Désactiver"))
            .form(|f| f.fields([user::Column::Username]));
        let error = admin.validate().unwrap_err();
        assert!(error.contains("action `Désactiver`"), "{error}");
        assert!(error.contains("not written by the form"), "{error}");
    }

    #[test]
    fn an_action_on_a_readonly_column_is_refused() {
        let admin = ModelAdmin::<user::Entity>::new()
            .list(|l| l.action(user::Column::IsActive, false, "Désactiver"))
            .form(|f| {
                f.fields([user::Column::Username, user::Column::IsActive])
                    .readonly([user::Column::IsActive])
            });
        let error = admin.validate().unwrap_err();
        assert!(error.contains("not written by the form"), "{error}");
    }

    #[test]
    fn a_column_without_its_field_in_the_form_is_refused() {
        let admin = ModelAdmin::<user::Entity>::new().form(|f| {
            f.fields([
                user::Column::Username,
                user::Column::Email,
                user::Column::IsActive,
            ])
        });
        let error = admin.validate().unwrap_err();
        assert!(
            error.contains("`is_active` is not a field of the model's form"),
            "{error}"
        );
        assert!(!error.contains("`username`"), "{error}");
        assert!(!error.contains("`email`"), "{error}");
    }

    #[test]
    fn a_field_that_cant_hold_its_column_is_refused() {
        let admin = ModelAdmin::<article::Entity>::new()
            .form(|f| f.fields([article::Column::Title, article::Column::Token]));
        let error = admin.validate().unwrap_err();
        assert!(
            error.contains("`token`: the form's field can't hold this column's type"),
            "{error}"
        );
        assert!(!error.contains("`title`"), "{error}");
    }

    #[test]
    fn an_action_on_a_written_column_is_accepted() {
        let admin = ModelAdmin::<article::Entity>::new()
            .list(|l| l.action(article::Column::Published, true, "Publier"))
            .form(|f| f.fields([article::Column::Title, article::Column::Published]));
        // `article` still misses its secret column: only the action matters here.
        let error = admin.validate().unwrap_err();
        assert!(!error.contains("action `Publier`"), "{error}");
    }

    #[test]
    fn an_action_on_the_owner_is_refused() {
        let admin = ModelAdmin::<Entity>::new()
            .list(|l| l.action(Column::OwnerId, pk(20), "Donner à 20"))
            .form(|f| f.fields([Column::OwnerId]));
        let error = admin.validate().unwrap_err();
        assert!(error.contains("action `Donner à 20`"), "{error}");
        assert!(error.contains("is the owner"), "{error}");
        assert!(!error.contains("not written by the form"), "{error}");
    }

    #[test]
    fn the_account_table_without_a_rank_flag_is_accepted() {
        let admin = ModelAdmin::<user::Entity>::new()
            .form(|f| f.fields([user::Column::Username, user::Column::Email]));
        assert!(admin.validate().is_ok());
    }

    async fn page_of(admin: &ModelAdmin<Entity>, query: ListQuery) -> ListPage {
        let db = db().await;
        let reader = staff(
            pk(10),
            "notes",
            [false, true, false, false, false, false, false],
        );
        let ctx = GateCtx {
            db: &db,
            user: &reader,
        };
        let proof = authorize_list::<Entity>(&ctx).unwrap();
        admin.fetch_list(&db, &proof, &query, 25).await.unwrap()
    }

    fn paged(per_page: u64) -> ModelAdmin<Entity> {
        ModelAdmin::<Entity>::new().list(|l| {
            l.display([Column::Id])
                .filter([Column::Id])
                .per_page(per_page)
        })
    }

    #[tokio::test]
    async fn every_page_carries_the_full_total() {
        let first = page_of(
            &paged(2),
            ListQuery {
                page: 1,
                ..Default::default()
            },
        )
        .await;
        let second = page_of(
            &paged(2),
            ListQuery {
                page: 2,
                ..Default::default()
            },
        )
        .await;
        assert_eq!((first.rows.len(), first.total), (2, 3));
        assert_eq!((second.rows.len(), second.total), (1, 3));
    }

    #[tokio::test]
    async fn a_page_past_the_end_is_empty_but_keeps_the_total() {
        let page = page_of(
            &paged(2),
            ListQuery {
                page: 5,
                ..Default::default()
            },
        )
        .await;
        assert!(page.rows.is_empty());
        assert_eq!(page.total, 3);
    }

    #[tokio::test]
    async fn only_the_columns_shown_and_the_key_are_read() {
        let admin = ModelAdmin::<Entity>::new().list(|l| l.display([Column::OwnerId]));
        let page = page_of(&admin, ListQuery::default()).await;
        let fields = page.rows[0].as_object().unwrap();
        let mut keys: Vec<&str> = fields.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(keys, ["id", "owner_id"]);
    }

    /// The values a filter offers for `Id`, to account `id` holding `rights`.
    async fn filter_values_for(id: crate::utils::pk::Pk, rights: [bool; 7]) -> Vec<String> {
        let db = db().await;
        let reader = staff(id, "notes", rights);
        let ctx = GateCtx {
            db: &db,
            user: &reader,
        };
        let proof = authorize_list::<Entity>(&ctx).unwrap();
        let admin =
            ModelAdmin::<Entity>::new().list(|l| l.display([Column::Id]).filter([Column::Id]));
        let page = admin
            .fetch_list(&db, &proof, &ListQuery::default(), 25)
            .await
            .unwrap();
        let mut values = page.filters.into_iter().next().unwrap().values;
        values.sort();
        values
    }

    /// A filter offers the values of the rows the account reads, never more:
    /// through `read_own` alone, another's row would show in the panel what
    /// the list hides.
    #[tokio::test]
    async fn a_filter_offers_only_the_values_in_reach() {
        let read = [false, true, false, false, false, false, false];
        let read_own = [false, false, true, false, false, false, false];
        assert_eq!(filter_values_for(pk(10), read).await, ["1", "2", "3"]);
        assert_eq!(filter_values_for(pk(10), read_own).await, ["1"]);
        assert_eq!(filter_values_for(pk(20), read_own).await, ["2"]);
    }

    #[tokio::test]
    async fn a_filter_narrows_rows_and_total_alike() {
        let query = ListQuery {
            filters: vec![("id".into(), "2".into())],
            ..Default::default()
        };
        let page = page_of(&paged(25), query).await;
        assert_eq!((page.rows.len(), page.total), (1, 1));
    }

    #[tokio::test]
    async fn the_declared_order_comes_first() {
        let admin = ModelAdmin::<Entity>::new().list(|l| {
            l.display([Column::Id])
                .ordering([(Column::Id, Order::Desc)])
        });
        let page = page_of(&admin, ListQuery::default()).await;
        assert_eq!(page.rows[0]["id"], 3);
    }

    /// An admin reading the accounts: `username` always, `email` only if `email`.
    fn account_reader(email: bool) -> CurrentUser {
        use crate::auth::permissions::ColumnRight;
        let mut admin = staff(
            pk(40),
            "users",
            [false, true, false, false, false, false, false],
        );
        admin.rang = Rang::Admin;
        admin.groupes[0].permissions[0].colonnes = [("username", true), ("email", email)]
            .into_iter()
            .map(|(name, read)| ColumnRight {
                colonne: name.into(),
                can_read: read,
                can_create: false,
                can_update: false,
            })
            .collect();
        admin.droits = admin.groupes[0].permissions.clone();
        admin
    }

    async fn accounts_page(reader: &CurrentUser, query: ListQuery) -> ListPage {
        let db = db().await;
        let ctx = GateCtx {
            db: &db,
            user: reader,
        };
        let proof = authorize_list::<user::Entity>(&ctx).unwrap();
        ModelAdmin::<user::Entity>::new()
            .list(|l| {
                l.display([user::Column::Username, user::Column::Email])
                    .search([user::Column::Email])
                    .filter([user::Column::Email])
                    .sortable([user::Column::Email])
            })
            .fetch_list(&db, &proof, &query, 25)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn an_unreadable_column_is_neither_read_nor_headed() {
        for (email, columns, keys) in [
            (false, vec!["username"], vec!["id", "username"]),
            (
                true,
                vec!["username", "email"],
                vec!["email", "id", "username"],
            ),
        ] {
            let page = accounts_page(&account_reader(email), ListQuery::default()).await;
            assert_eq!(page.columns, columns, "email readable: {email}");
            assert!(!page.rows.is_empty());
            for row in &page.rows {
                let mut read: Vec<&str> = row
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect();
                read.sort();
                assert_eq!(read, keys, "email readable: {email}");
            }
        }
    }

    /// A named list query, built afresh for each reader.
    type Probe = (&'static str, fn() -> ListQuery);

    /// Searching, filtering or sorting on an unreadable column would tell its
    /// values one guess at a time: the page must be the one without the parameter.
    #[tokio::test]
    async fn an_unreadable_column_is_neither_searched_filtered_nor_sorted() {
        let queries: [Probe; 3] = [
            ("search", || ListQuery {
                search: Some("member@".into()),
                ..Default::default()
            }),
            ("filter", || ListQuery {
                filters: vec![("email".into(), "member@example.com".into())],
                ..Default::default()
            }),
            ("sort", || ListQuery {
                sort_by: Some("email".into()),
                sort_dir: Some("desc".into()),
                ..Default::default()
            }),
        ];
        let hidden = account_reader(false);
        let shown = account_reader(true);
        let hidden_base = accounts_page(&hidden, ListQuery::default()).await;
        let shown_base = accounts_page(&shown, ListQuery::default()).await;
        for (label, query) in queries {
            let page = accounts_page(&hidden, query()).await;
            assert_eq!(page.rows, hidden_base.rows, "hidden: {label}");
            assert_eq!(page.total, hidden_base.total, "hidden: {label}");
            let page = accounts_page(&shown, query()).await;
            assert_ne!(page.rows, shown_base.rows, "shown: {label}");
            assert_eq!(page.rows[0]["username"], "member", "shown: {label}");
        }
    }

    /// A declared order on an unreadable column would still rank the rows by its
    /// values: dropped, the rows come in key order.
    #[tokio::test]
    async fn a_declared_order_on_an_unreadable_column_is_dropped() {
        let db = db().await;
        let admin = ModelAdmin::<user::Entity>::new().list(|l| {
            l.display([user::Column::Username, user::Column::Email])
                .ordering([(user::Column::Email, Order::Desc)])
        });
        for (email, first) in [(false, "crawler"), (true, "member")] {
            let reader = account_reader(email);
            let ctx = GateCtx {
                db: &db,
                user: &reader,
            };
            let proof = authorize_list::<user::Entity>(&ctx).unwrap();
            let page = admin
                .fetch_list(&db, &proof, &ListQuery::default(), 25)
                .await
                .unwrap();
            assert_eq!(page.rows[0]["username"], first, "email readable: {email}");
        }
    }

    /// The row's page adds the form's fields to the list's: those go through the
    /// same cut, or a hidden field would come back by the form.
    #[tokio::test]
    async fn the_detail_leaves_out_an_unreadable_form_field() {
        let db = db().await;
        let admin = ModelAdmin::<user::Entity>::new()
            .list(|l| l.display([user::Column::Username]))
            .form(|f| f.fields([user::Column::Username, user::Column::Email]));
        for (email, keys) in [
            (false, vec!["id", "username"]),
            (true, vec!["email", "id", "username"]),
        ] {
            let reader = account_reader(email);
            let ctx = GateCtx {
                db: &db,
                user: &reader,
            };
            let proof = authorize_view::<user::Entity>(&ctx, pk(31)).unwrap();
            let entry = admin.fetch_detail(&db, &proof).await.unwrap().unwrap();
            let mut read: Vec<&str> = entry
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            read.sort();
            assert_eq!(read, keys, "email readable: {email}");
        }
    }
    #[test]
    fn a_needed_column_outside_fields_is_refused() {
        let admin =
            ModelAdmin::<article::Entity>::new().form(|f| f.fields([article::Column::Summary]));
        let error = admin.validate().unwrap_err();
        assert!(
            error.contains("`title` is needed to create a row"),
            "{error}"
        );
    }

    #[test]
    fn a_needed_column_in_fields_is_accepted() {
        let admin =
            ModelAdmin::<article::Entity>::new().form(|f| f.fields([article::Column::Title]));
        let error = admin.validate().unwrap_err();
        assert!(!error.contains("`title`"), "{error}");
    }

    #[test]
    fn a_needed_column_in_readonly_is_refused() {
        let admin = ModelAdmin::<article::Entity>::new().form(|f| {
            f.fields([article::Column::Title])
                .readonly([article::Column::Title])
        });
        let error = admin.validate().unwrap_err();
        assert!(
            error.contains("`title` is needed to create a row"),
            "{error}"
        );
    }

    /// One mutation per `||` of `filled_elsewhere`: dropping any of them
    /// makes its column reported here.
    #[test]
    fn columns_filled_elsewhere_are_not_needed() {
        let admin =
            ModelAdmin::<article::Entity>::new().form(|f| f.fields([article::Column::Title]));
        let error = admin.validate().unwrap_err();
        for name in [
            "status",
            "created_at",
            "updated_at",
            "summary",
            "token",
            "owner_id",
        ] {
            assert!(!error.contains(&format!("`{name}`")), "{name}: {error}");
        }
    }

    #[test]
    fn a_needed_secret_column_gets_its_own_message() {
        let admin =
            ModelAdmin::<article::Entity>::new().form(|f| f.fields([article::Column::Title]));
        let error = admin.validate().unwrap_err();
        assert!(
            error.contains("`password` is a secret column needed"),
            "{error}"
        );
        assert!(!error.contains("`password` is needed to create"), "{error}");
    }

    #[test]
    fn a_model_without_fields_has_no_create_check() {
        let admin = ModelAdmin::<article::Entity>::new();
        assert!(admin.validate().is_ok());
    }

    #[test]
    fn an_action_on_a_create_only_column_is_refused() {
        let admin = ModelAdmin::<user::Entity>::new()
            .list(|l| l.action(user::Column::IsActive, false, "Désactiver"))
            .form(|f| {
                f.fields([
                    user::Column::Username,
                    user::Column::Email,
                    user::Column::IsActive,
                ])
                .create_only([user::Column::IsActive])
            });
        let error = admin.validate().unwrap_err();
        assert!(error.contains("not written by the form on edit"), "{error}");
    }

    #[test]
    fn a_needed_column_in_create_only_is_accepted() {
        let admin = ModelAdmin::<article::Entity>::new().form(|f| {
            f.fields([article::Column::Title])
                .create_only([article::Column::Title])
        });
        let error = admin.validate().unwrap_err();
        assert!(!error.contains("`title`"), "{error}");
    }
}
