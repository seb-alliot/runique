//! Every regirstred `ModeelAdmin` , whatever its model.
use super::admin_builder::ModelAdmin;
use super::registered::RegisteredModel;
use crate::admin::refonte_admin::meta::ModelMeta;

#[derive(Default)]
pub struct AdminSite {
    models: Vec<Box<dyn RegisteredModel>>,
}

impl AdminSite {
    pub fn register<E: ModelMeta>(mut self, admin: ModelAdmin<E>) -> Self {
        self.models.push(Box::new(admin));
        self
    }

    pub(crate) fn check(&self) -> Vec<String> {
        let mut error = Vec::new();
        for (i, model) in self.models.iter().enumerate() {
            if let Err(e) = model.check() {
                error.push(e)
            }
            if self.models.iter().take(i).any(|m| m.key() == model.key()) {
                error.push(format!("model `{}` registered twice", model.key()));
            }
        }
        error
    }

    /// The registered model behind a URL's resource key, if any: an unknown
    /// key never reaches a query.
    pub(crate) fn model(&self, key: &str) -> Option<&dyn RegisteredModel> {
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
            .register(ModelAdmin::<user::Entity>::new());
        assert!(site.check().is_empty());
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
            .register(ModelAdmin::<Entity>::new().list(|l| l.links([Column::Id])))
            .register(
                ModelAdmin::<user::Entity>::new().form(|f| f.fields([user::Column::IsSuperuser])),
            );
        let errors = site.check();
        assert_eq!(errors.len(), 2, "{errors:?}");
        assert!(errors[0].contains("notes"), "{errors:?}");
        assert!(errors[1].contains("users"), "{errors:?}");
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
