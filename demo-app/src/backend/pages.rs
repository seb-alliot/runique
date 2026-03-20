use crate::entities::{code_example, demo_category, demo_page, page_doc_link};
use runique::prelude::*;

pub async fn fetch_page_examples(
    slug: &str,
    db: &sea_orm::DatabaseConnection,
) -> (Vec<code_example::Model>, Vec<page_doc_link::Model>) {
    let page = demo_page::Entity::find()
        .filter(demo_page::Column::Slug.eq(slug))
        .one(db)
        .await
        .unwrap_or(None);

    let Some(page) = page else {
        return (vec![], vec![]);
    };

    let examples = code_example::Entity::find()
        .filter(code_example::Column::PageId.eq(page.id))
        .order_by_asc(code_example::Column::SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let links = page_doc_link::Entity::find()
        .filter(page_doc_link::Column::PageId.eq(page.id))
        .order_by_asc(page_doc_link::Column::SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    (examples, links)
}

pub async fn demo_code_page(slug: &str, request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;

    let db = request.engine.db.clone();

    let page = demo_page::Entity::find()
        .filter(demo_page::Column::Slug.eq(slug))
        .one(&*db)
        .await
        .unwrap_or(None);

    let Some(page) = page else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let code_examples = code_example::Entity::find()
        .filter(code_example::Column::PageId.eq(page.id))
        .order_by_asc(code_example::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let doc_links = page_doc_link::Entity::find()
        .filter(page_doc_link::Column::PageId.eq(page.id))
        .order_by_asc(page_doc_link::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let category = demo_category::Entity::find_by_id(page.category_id)
        .one(&*db)
        .await
        .unwrap_or(None);

    context_update!(request => {
        "title"         => &page.title,
        "page"          => &page,
        "code_examples" => &code_examples,
        "doc_links"     => &doc_links,
        "category"      => &category,
    });

    request.render("demo/generic.html")
}
