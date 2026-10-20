//! Every registered `ModelAdmin`, whatever its model.
use super::admin_builder::ModelAdmin;
use super::registered::RegisteredModel;
use crate::admin::refonte_admin::meta::ModelMeta;
use crate::auth::session::CurrentUser;
use crate::auth::user;

/// One model as the navigation shows it.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct NavEntry {
    pub key: &'static str,
    pub title: String,
}

pub struct AdminSite {
    models: Vec<Box<dyn RegisteredModel>>,
    /// The accounts admin: the framework's own unless the project hands its
    /// extension of `eihwaz_users` to `accounts_users()`.
    accounts: Box<dyn RegisteredModel>,
}

impl Default for AdminSite {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            accounts: Box::new(default_accounts()),
        }
    }
}

/// The framework's accounts admin, there without any declaration.
fn default_accounts() -> ModelAdmin<user::Entity> {
    ModelAdmin::new()
        .list(|l| {
            l.display([
                user::Column::Username,
                user::Column::Email,
                user::Column::IsActive,
            ])
            .sortable([user::Column::Username])
            .search([user::Column::Username, user::Column::Email])
        })
        .form(|f| f.fields([user::Column::Username, user::Column::Email]))
}

impl AdminSite {
    pub fn register<E: ModelMeta>(mut self, admin: ModelAdmin<E>) -> Self {
        self.models.push(Box::new(admin.with_default_display()));
        self
    }

    /// Replaces the framework's accounts admin with the project's own, built
    /// on its `extend!` of `eihwaz_users`. The only way to replace it:
    /// registering a model under the accounts' key is refused at boot.
    pub fn accounts_users<E: ModelMeta>(mut self, admin: ModelAdmin<E>) -> Self {
        self.accounts = Box::new(admin.with_default_display());
        self
    }

    pub(crate) fn check(&self) -> Vec<String> {
        let mut error = Vec::new();
        if let Err(e) = self.accounts.check() {
            error.push(e);
        }
        if !self.accounts.keeps_account_rules() {
            error.push(format!(
                "accounts_users: model `{}` must be the `eihwaz_users` table, with the accounts' key and protections",
                self.accounts.key()
            ));
        }
        for (i, model) in self.models.iter().enumerate() {
            if let Err(e) = model.check() {
                error.push(e)
            }
            if model.key() == self.accounts.key() {
                error.push(format!(
                    "model `{}` registered twice: the accounts admin is replaced with `accounts_users()`",
                    model.key()
                ));
            }
            if self.models.iter().take(i).any(|m| m.key() == model.key()) {
                error.push(format!("model `{}` registered twice", model.key()));
            }
        }
        error
    }

    /// Whether the project registered any model of its own, beyond the
    /// accounts the framework always serves.
    pub(crate) fn has_project_models(&self) -> bool {
        !self.models.is_empty()
    }

    /// How many rows of each model `user` may read, in one query: the
    /// gate's scope applies to every count, a model without the read right
    /// has none.
    pub(crate) async fn counts(
        &self,
        db: &crate::utils::aliases::ADb,
        user: &CurrentUser,
    ) -> Result<std::collections::HashMap<String, i64>, sea_orm::DbErr> {
        use super::registered::{COUNT_RESOURCE, COUNT_TOTAL};
        use sea_orm::ConnectionTrait;
        use sea_orm::sea_query::UnionType;
        let mut queries = std::iter::once(&self.accounts)
            .chain(self.models.iter())
            .filter_map(|m| m.count_query(db, user));
        let Some(mut union) = queries.next() else {
            return Ok(std::collections::HashMap::new());
        };
        for query in queries {
            union.union(UnionType::All, query);
        }
        let rows = db.query_all(&union).await?;
        rows.iter()
            .map(|row| {
                Ok((
                    row.try_get::<String>("", COUNT_RESOURCE)?,
                    row.try_get::<i64>("", COUNT_TOTAL)?,
                ))
            })
            .collect()
    }

    /// The resource keys of every model the admin serves, accounts included.
    pub(crate) fn keys(&self) -> Vec<&'static str> {
        std::iter::once(self.accounts.key())
            .chain(self.models.iter().map(|m| m.key()))
            .collect()
    }

    /// The models `user` sees in the navigation: those with at least one
    /// right, the keys of `order` first in that order, then by title.
    pub(crate) fn navigation(&self, user: &CurrentUser, order: &[String]) -> Vec<NavEntry> {
        let mut entries: Vec<NavEntry> = std::iter::once(&self.accounts)
            .chain(self.models.iter())
            .filter(|m| m.visible_to(user))
            .map(|m| NavEntry {
                key: m.key(),
                title: m.title(),
            })
            .collect();
        let rank = |key: &str| order.iter().position(|k| k == key).unwrap_or(usize::MAX);
        entries.sort_by(|a, b| {
            rank(a.key)
                .cmp(&rank(b.key))
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
        entries
    }

    /// The registered model behind a URL's resource key, if any: an unknown
    /// key never reaches a query.
    pub(crate) fn model(&self, key: &str) -> Option<&dyn RegisteredModel> {
        if self.accounts.key() == key {
            return Some(self.accounts.as_ref());
        }
        self.models
            .iter()
            .find(|m| m.key() == key)
            .map(|m| m.as_ref())
    }
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::AdminSite;
    use crate::admin::refonte_admin::builder::admin_builder::ModelAdmin;
    use crate::admin::refonte_admin::builder::list::ListQuery;
    use crate::admin::refonte_admin::builder::registered::AdminError;
    use crate::admin::refonte_admin::gate::test_support::fixtures::note::{Column, Entity};
    use crate::admin::refonte_admin::gate::test_support::fixtures::{db, pk, staff};
    use crate::auth::user;

    #[test]
    fn distinct_valid_models_pass() {
        let site = AdminSite::default()
            .register(ModelAdmin::<Entity>::new().form(|f| f.fields([Column::OwnerId])))
            .accounts_users(ModelAdmin::<user::Entity>::new());
        assert!(site.check().is_empty(), "{:?}", site.check());
    }

    #[test]
    fn the_accounts_are_served_without_any_declaration() {
        let site = AdminSite::default();
        assert!(site.check().is_empty(), "{:?}", site.check());
        assert!(site.model("users").is_some());
        assert_eq!(site.keys(), ["users"]);
        assert!(!site.has_project_models());
    }

    #[test]
    fn a_registered_model_is_a_project_model() {
        let site = AdminSite::default().register(ModelAdmin::<Entity>::new());
        assert!(site.has_project_models());
        assert_eq!(site.keys(), ["users", "notes"]);
    }

    /// The accounts are replaced on purpose, never by a key that happens to match.
    #[test]
    fn registering_the_accounts_key_is_refused() {
        let site = AdminSite::default().register(ModelAdmin::<user::Entity>::new());
        let errors = site.check();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("accounts_users()"), "{errors:?}");
    }

    #[test]
    fn the_accounts_cant_be_replaced_by_another_table() {
        let site = AdminSite::default().accounts_users(ModelAdmin::<Entity>::new());
        let errors = site.check();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("must be the `eihwaz_users` table")),
            "{errors:?}"
        );
        assert!(
            site.model("users").is_none(),
            "the notes took the accounts' place"
        );
    }

    #[test]
    fn a_model_registered_twice_is_refused() {
        let site = AdminSite::default()
            .register(ModelAdmin::<Entity>::new())
            .register(ModelAdmin::<Entity>::new());
        let errors = site.check();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("registered twice"), "{errors:?}");
    }

    #[test]
    fn each_invalid_model_reports_its_own_error() {
        let site = AdminSite::default()
            .register(
                ModelAdmin::<Entity>::new()
                    .list(|l| l.display([Column::OwnerId]).links([Column::Id])),
            )
            .accounts_users(
                ModelAdmin::<user::Entity>::new().form(|f| f.fields([user::Column::IsSuperuser])),
            );
        let errors = site.check();
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(
            errors.iter().any(|e| e.contains("admin `notes`")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("admin `users`")),
            "{errors:?}"
        );
    }
    fn with_rang(rang: crate::auth::rang::Rang) -> crate::auth::session::CurrentUser {
        let mut user = staff(pk(40), "nothing", [false; 6]);
        user.rang = rang;
        user
    }

    fn nav_keys(
        site: &AdminSite,
        user: &crate::auth::session::CurrentUser,
        order: &[String],
    ) -> Vec<&'static str> {
        site.navigation(user, order)
            .into_iter()
            .map(|e| e.key)
            .collect()
    }

    /// The navigation lists a model as soon as one right opens it, one only:
    /// creating, without reading, is enough to see it.
    #[test]
    fn the_navigation_lists_only_the_models_a_right_opens() {
        let site = AdminSite::default().register(ModelAdmin::<Entity>::new());
        let creator = staff(pk(10), "notes", [true, false, false, false, false, false]);
        assert_eq!(
            nav_keys(&site, &creator, &[]),
            ["notes"],
            "a lambda never sees the accounts"
        );
        let nobody = staff(pk(10), "other", [true, true, true, true, true, true]);
        assert!(nav_keys(&site, &nobody, &[]).is_empty());
        let su = with_rang(crate::auth::rang::Rang::Superuser);
        assert_eq!(
            nav_keys(&site, &su, &[]),
            ["notes", "users"],
            "sorted by title"
        );
    }

    #[test]
    fn a_declared_order_comes_before_the_titles() {
        let site = AdminSite::default().register(ModelAdmin::<Entity>::new());
        let su = with_rang(crate::auth::rang::Rang::Superuser);
        assert_eq!(
            nav_keys(&site, &su, &["users".to_string()]),
            ["users", "notes"]
        );
    }

    /// One query for every count, each under the gate: the accounts and the
    /// notes for a superuser, the notes alone for a lambda who may read them,
    /// nothing for one who may not.
    #[tokio::test]
    async fn the_counts_follow_the_gate() {
        let db = db().await;
        let site = AdminSite::default().register(ModelAdmin::<Entity>::new());

        let su = with_rang(crate::auth::rang::Rang::Superuser);
        let counts = site.counts(&db, &su).await.unwrap();
        assert_eq!(counts.get("notes"), Some(&3));
        assert_eq!(counts.get("users"), Some(&4));

        let reader = staff(pk(10), "notes", [false, true, false, false, false, false]);
        let counts = site.counts(&db, &reader).await.unwrap();
        assert_eq!(counts.get("notes"), Some(&3));
        assert_eq!(
            counts.get("users"),
            None,
            "a lambda never counts the accounts"
        );

        let nobody = staff(pk(10), "other", [true, true, true, true, true, true]);
        assert!(site.counts(&db, &nobody).await.unwrap().is_empty());
    }

    #[test]
    fn a_declared_title_names_the_model_in_the_navigation() {
        let site = AdminSite::default().register(ModelAdmin::<Entity>::new().title("Mes notes"));
        let su = with_rang(crate::auth::rang::Rang::Superuser);
        let titles: Vec<String> = site
            .navigation(&su, &[])
            .into_iter()
            .map(|e| e.title)
            .collect();
        assert_eq!(titles, ["Mes notes", "Users"]);
    }

    #[test]
    fn a_key_without_verbose_name_is_shown_readable() {
        use crate::admin::refonte_admin::builder::registered::readable;
        assert_eq!(readable("doc_page"), "Doc page");
        assert_eq!(readable("users"), "Users");
        assert_eq!(readable("test_all_fields"), "Test all fields");
        assert_eq!(readable(""), "");
    }

    /// No `display` declared: every column but the secret ones, and on the
    /// accounts only their name, email and state — never the activation date.
    #[test]
    fn an_undeclared_display_shows_the_safe_columns() {
        let notes = ModelAdmin::<Entity>::new().with_default_display();
        assert_eq!(notes.list.lists_using("owner_id"), ["display"]);
        assert_eq!(notes.list.lists_using("id"), ["display"]);

        let accounts = ModelAdmin::<user::Entity>::new().with_default_display();
        for shown in ["username", "email", "is_active"] {
            assert_eq!(accounts.list.lists_using(shown), ["display"], "{shown}");
        }
        for hidden in [
            "password",
            "activated_at",
            "is_superuser",
            "is_dirigeant",
            "is_admin",
        ] {
            assert!(accounts.list.lists_using(hidden).is_empty(), "{hidden}");
        }
    }

    #[test]
    fn a_declared_display_is_kept() {
        let notes = ModelAdmin::<Entity>::new()
            .list(|l| l.display([Column::OwnerId]))
            .with_default_display();
        assert!(notes.list.lists_using("id").is_empty());
    }

    fn notes_site() -> AdminSite {
        AdminSite::default().register(
            ModelAdmin::<Entity>::new()
                .list(|l| l.display([Column::Id]))
                .form(|f| f.fields([Column::OwnerId])),
        )
    }

    #[tokio::test]
    async fn a_registered_key_lists_its_rows() {
        let db = db().await;
        let reader = staff(pk(10), "notes", [false, true, false, false, false, false]);
        let site = notes_site();
        let page = site
            .model("notes")
            .unwrap()
            .list(&db, &reader, &ListQuery::default(), 25)
            .await
            .unwrap();
        assert_eq!(page.total, 3);
    }

    #[test]
    fn an_unknown_key_finds_no_model() {
        assert!(notes_site().model("eihwaz_sessions").is_none());
    }

    #[tokio::test]
    async fn the_gate_still_decides_behind_the_key() {
        let db = db().await;
        let writer = staff(pk(10), "notes", [true, false, true, true, true, true]);
        let site = notes_site();
        let result = site
            .model("notes")
            .unwrap()
            .list(&db, &writer, &ListQuery::default(), 25)
            .await;
        assert!(matches!(result, Err(AdminError::Refused(_))));
    }

    async fn detail_as(rights: [bool; 6], id: &str) -> Result<serde_json::Value, AdminError> {
        let db = db().await;
        let user = staff(pk(10), "notes", rights);
        notes_site()
            .model("notes")
            .unwrap()
            .detail(&db, &user, id)
            .await
    }

    const READ: [bool; 6] = [false, true, false, false, false, false];

    #[tokio::test]
    async fn a_row_shows_the_form_columns_and_its_key_only() {
        let row = detail_as(READ, "2").await.unwrap();
        let mut keys: Vec<&str> = row
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort();
        assert_eq!(keys, ["id", "owner_id"]);
        assert_eq!(row["id"], 2);
    }

    #[tokio::test]
    async fn a_missing_row_is_refused_like_a_forbidden_one() {
        assert!(matches!(
            detail_as(READ, "99").await,
            Err(AdminError::Refused(_))
        ));
        let no_read = [true, false, true, true, true, true];
        assert!(matches!(
            detail_as(no_read, "1").await,
            Err(AdminError::Refused(_))
        ));
    }

    #[tokio::test]
    async fn a_malformed_id_is_refused_before_any_query() {
        assert!(matches!(
            detail_as(READ, "1; DROP TABLE notes").await,
            Err(AdminError::Refused(_))
        ));
        assert!(matches!(
            detail_as(READ, "").await,
            Err(AdminError::Refused(_))
        ));
    }
}
