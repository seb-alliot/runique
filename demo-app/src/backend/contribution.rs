use crate::entities::contribution::Entity as ContributionEntity;
use crate::formulaire::{ContributionForm, contribution_type_choices};
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct ContributionItem {
    #[serde(flatten)]
    pub contribution: crate::entities::contribution::Model,
    pub username: Option<String>,
}

pub async fn list_contributions(db: &sea_orm::DatabaseConnection) -> Vec<ContributionItem> {
    search!(ContributionEntity => desc Id,)
        .also_related(runique_users::Entity)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(c, u)| ContributionItem {
            username: u.map(|u| u.username),
            contribution: c,
        })
        .collect()
}

pub async fn save_contribution(
    form: &mut ContributionForm,
    db: &sea_orm::DatabaseConnection,
    user_id: runique::utils::pk::Pk,
) -> Result<(), sea_orm::DbErr> {
    form.save(db, user_id).await.map(|_| ())
}

pub async fn handle_contribution_submit(
    request: &mut Request,
    form: &mut ContributionForm,
) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let template = "contribution/contribution_form.html";
    if !is_authenticated(&request.session).await {
        return Ok(Redirect::to("/login").into_response());
    }
    form.get_form_mut().field(
        &ChoiceField::new("contribution_type")
            .label("Contribution type")
            .choices(contribution_type_choices()),
    );

    if request.is_get() {
        context_update!(request => { "title" => "Submit a contribution", "contribution_form" => &*form });
        return request.render(template);
    }
    if request.is_post() && form.is_valid().await {
        let user_id = get_user_id(&request.session).await.unwrap_or(0);
        match save_contribution(form, &request.engine.db, user_id).await {
            Ok(_) => {
                success!(request.notices => "Thank you for your contribution!");
                return Ok(Redirect::to("/contributions").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                context_update!(request => { "title" => "Database error", "contribution_form" => &*form });
                return request.render(template);
            }
        }
    }
    context_update!(request => {
        "title"             => "Validation error",
        "contribution_form" => &*form,
        "messages"          => flash_now!(error => "Please correct the errors below"),
    });
    request.render(template)
}
