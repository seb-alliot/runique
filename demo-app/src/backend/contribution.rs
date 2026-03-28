use crate::entities::contribution::Entity as ContributionEntity;
use crate::formulaire::{ContributionForm, contribution_type_choices};
use runique::prelude::*;

pub async fn list_contributions(
    db: &sea_orm::DatabaseConnection,
) -> Vec<crate::entities::contribution::Model> {
    ContributionEntity::find()
        .order_by_desc(crate::entities::contribution::Column::Id)
        .all(db)
        .await
        .unwrap_or_default()
}

pub async fn save_contribution(
    form: &mut ContributionForm,
    db: &sea_orm::DatabaseConnection,
    user_id: i32,
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
            .label("Type de contribution")
            .choices(contribution_type_choices()),
    );

    if request.is_get() {
        context_update!(request => { "title" => "Soumettre une contribution", "contribution_form" => &*form });
        return request.render(template);
    }
    if request.is_post() && form.is_valid().await {
        let user_id = get_user_id(&request.session).await.unwrap_or(0);
        match save_contribution(form, &request.engine.db, user_id).await {
            Ok(_) => {
                success!(request.notices => "Merci pour votre contribution !");
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                context_update!(request => { "title" => "Erreur base de données", "contribution_form" => &*form });
                return request.render(template);
            }
        }
    }
    context_update!(request => {
        "title"             => "Erreur de validation",
        "contribution_form" => &*form,
        "messages"          => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    });
    request.render(template)
}
