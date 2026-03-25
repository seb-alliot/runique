use crate::entities::blog::Entity as BlogEntity;
use crate::formulaire::BlogForm;
use runique::prelude::*;

pub async fn list_articles(
    db: &sea_orm::DatabaseConnection,
    search: Option<&str>,
) -> Vec<crate::entities::blog::Model> {
    let mut query = BlogEntity::find().order_by_desc(crate::entities::blog::Column::Id);
    if let Some(term) = search
        && !term.is_empty()
    {
        use sea_orm::Condition;
        query = query.filter(
            Condition::any()
                .add(crate::entities::blog::Column::Title.contains(term))
                .add(crate::entities::blog::Column::Summary.contains(term)),
        );
    }
    query.all(db).await.unwrap_or_default()
}

pub async fn get_article(
    db: &sea_orm::DatabaseConnection,
    id: i32,
) -> Option<crate::entities::blog::Model> {
    BlogEntity::find_by_id(id).one(db).await.unwrap_or(None)
}

pub async fn save_blog(
    blog: &mut BlogForm,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    blog.save(db).await.map(|_| ())
}

pub async fn handle_blog_save(request: &mut Request, blog: &mut BlogForm) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let template = "blog/blog.html";
    if request.is_get() {
        context_update!(request => { "title" => "Créer un article de blog", "blog_form" => &blog });
        return request.render(template);
    }
    if request.is_post() && blog.is_valid().await {
        match save_blog(blog, &request.engine.db).await {
            Ok(_) => {
                success!(request.notices => "Article sauvegardé !");
                return Ok(Redirect::to("/blog/liste").into_response());
            }
            Err(err) => {
                blog.get_form_mut().database_error(&err);
                context_update!(request => { "title" => "Erreur base de données", "blog_form" => &blog });
                return request.render(template);
            }
        }
    }
    context_update!(request => {
        "title"     => "Erreur de validation",
        "blog_form" => &*blog,
        "messages"  => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    });
    request.render(template)
}
