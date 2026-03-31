use crate::entities::blog::Entity as BlogEntity;
use crate::formulaire::BlogForm;
use runique::prelude::*;

pub async fn list_articles(
    db: &sea_orm::DatabaseConnection,
    search: Option<&str>,
) -> Vec<crate::entities::blog::Model> {
    if let Some(term) = search.filter(|s| !s.is_empty()) {
        search!(BlogEntity => or(Title icontains term, Summary icontains term))
            .order_by_desc(crate::entities::blog::Column::Id)
            .all(db)
            .await
            .unwrap_or_default()
    } else {
        search!(BlogEntity)
            .order_by_desc(crate::entities::blog::Column::Id)
            .all(db)
            .await
            .unwrap_or_default()
    }
}

pub async fn get_article(
    db: &sea_orm::DatabaseConnection,
    id: i32,
) -> Option<crate::entities::blog::Model> {
    search!(BlogEntity => Id eq id)
        .first(db)
        .await
        .unwrap_or(None)
}

pub async fn save_blog(
    blog: &mut BlogForm,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    blog.save(db).await.map(|_| ())
}

pub async fn handle_blog_save(request: &mut Request, blog: &mut BlogForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let template = "blog/blog.html";
    if request.is_get() {
        context_update!(request => { "title" => "Create a blog post", "blog_form" => &blog });
        return request.render(template);
    }
    if request.is_post() && blog.is_valid().await {
        match save_blog(blog, &request.engine.db).await {
            Ok(_) => {
                success!(request.notices => "Article saved!");
                return Ok(Redirect::to("/blog/liste").into_response());
            }
            Err(err) => {
                blog.get_form_mut().database_error(&err);
                context_update!(request => { "title" => "Database error", "blog_form" => &blog });
                return request.render(template);
            }
        }
    }
    context_update!(request => {
        "title"     => "Validation error",
        "blog_form" => &*blog,
        "messages"  => flash_now!(error => "Please correct the errors below"),
    });
    request.render(template)
}
