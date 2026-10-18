//! One model's admin: one block per responsibility, each set through a closure.
use super::form::FormConfig;
use super::list::ListConfig;
use super::list::{ListQuery, TOTAL};
use crate::admin::refonte_admin::{
    gate::{Authorized, Guarded, List, View},
    meta::ModelMeta,
};
use crate::utils::aliases::ADb;
use sea_orm::{
    DbErr, FromQueryResult, IdenStatic, PaginatorTrait, QueryFilter, QueryResult, QuerySelect,
};

/// The account table's rank flags: written by the gate alone, never a form field.
const RANK_FLAGS: [&str; 3] = ["is_superuser", "is_dirigeant", "is_admin"];

pub struct ModelAdmin<E: ModelMeta> {
    pub list: ListConfig<E>,
    pub form: FormConfig<E>,
}

/// One page of a list, ready for the template.
#[derive(serde::Serialize)]
pub(crate) struct ListPage {
    pub rows: Vec<serde_json::Value>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
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
        }
    }
}

impl<E: ModelMeta> ModelAdmin<E> {
    pub fn new() -> Self {
        Self::default()
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

        for action in &self.list.actions {
            let name = action.col.as_str();
            if !self.form.is_writable(name) {
                errors.push(format!(
                    "action `{}`: `{name}` is not written by the form (in `fields`, not `readonly`)",
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
        let page_size = self.list.page_size(site_page_size);
        let page = query.page.max(1);
        let rows: Vec<ListRow> = self
            .list
            .select(backend, proof.conditions(), query)
            .limit(page_size)
            .offset((page - 1).saturating_mul(page_size))
            .into_model::<ListRow>()
            .all(&**db)
            .await?;
        let total = match rows.first() {
            Some(row) => u64::try_from(row.total).unwrap_or(0),
            None if page > 1 => {
                self.list
                    .filtered(backend, proof.conditions(), query)
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

        Ok(ListPage {
            rows,
            total,
            page,
            page_size,
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
        E::find_by_id(id.clone())
            .filter(proof.conditions())
            .select_only()
            .columns(self.form.read_columns())
            .into_json()
            .one(&**db)
            .await
    }
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::{ListPage, ModelAdmin};
    use crate::admin::refonte_admin::builder::list::ListQuery;
    use crate::admin::refonte_admin::gate::GateCtx;
    use crate::admin::refonte_admin::gate::check::authorize_list;
    use crate::admin::refonte_admin::gate::test_support::fixtures::note::{Column, Entity};
    use crate::admin::refonte_admin::gate::test_support::fixtures::{db, pk, staff};
    use crate::auth::user;
    use sea_orm::Order;

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
        let cases: [(&str, ModelAdmin<user::Entity>); 6] = [
            ("display", ModelAdmin::new().list(|l| l.display([p]))),
            ("search", ModelAdmin::new().list(|l| l.search([p]))),
            ("filter", ModelAdmin::new().list(|l| l.filter([p]))),
            ("sortable", ModelAdmin::new().list(|l| l.sortable([p]))),
            (
                "ordering",
                ModelAdmin::new().list(|l| l.ordering([(p, Order::Asc)])),
            ),
            ("fields", ModelAdmin::new().form(|f| f.fields([p]))),
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
    fn the_account_table_without_a_rank_flag_is_accepted() {
        let admin = ModelAdmin::<user::Entity>::new().form(|f| f.fields([user::Column::Username]));
        assert!(admin.validate().is_ok());
    }

    async fn page_of(admin: &ModelAdmin<Entity>, query: ListQuery) -> ListPage {
        let db = db().await;
        let reader = staff(pk(10), "notes", [false, true, false, false, false, false]);
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
}
