use crate::entities::{cour, doc_block, doc_page, doc_section};
use runique::prelude::*;

pub async fn doc_index(lang: &str, request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let db = request.engine.db.clone();

    let sections = doc_section::Entity::find()
        .filter(doc_section::Column::Lang.eq(lang))
        .order_by_asc(doc_section::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let mut cours_difficultes: Vec<String> = cour::Entity::find()
        .filter(cour::Column::Lang.eq(lang))
        .select_only()
        .column(cour::Column::Difficulte)
        .distinct()
        .into_tuple::<String>()
        .all(&*db)
        .await
        .unwrap_or_default();

    let order = ["debutant", "intermediaire", "avance", "specifique"];
    cours_difficultes.sort_by_key(|d| order.iter().position(|&o| o == d.as_str()).unwrap_or(99));

    let cours = cour::Entity::find()
        .filter(cour::Column::Lang.eq(lang))
        .order_by_asc(cour::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    context_update!(request => {
        "title"              => &format!("Documentation — {}", lang.to_uppercase()),
        "sections"           => &sections,
        "cours_themes"       => &cours_difficultes,
        "cours_difficultes"  => &cours_difficultes,
        "cours"              => &cours,
        "lang"               => lang,
    });

    request.render("docs/doc_index.html")
}

pub async fn doc_page(
    lang: &str,
    section_slug: &str,
    page_slug: &str,
    request: &mut Request,
) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let db = request.engine.db.clone();

    let section = doc_section::Entity::find()
        .filter(doc_section::Column::Lang.eq(lang))
        .filter(doc_section::Column::Slug.eq(section_slug))
        .one(&*db)
        .await
        .unwrap_or(None);

    let Some(section) = section else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let sidebar_pages = doc_page::Entity::find()
        .filter(doc_page::Column::SectionId.eq(section.id))
        .filter(doc_page::Column::Lang.eq(lang))
        .order_by_asc(doc_page::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let full_slug = format!("{section_slug}-{page_slug}");
    let page = doc_page::Entity::find()
        .filter(doc_page::Column::SectionId.eq(section.id))
        .filter(doc_page::Column::Slug.eq(&full_slug))
        .one(&*db)
        .await
        .unwrap_or(None);

    let Some(page) = page else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let blocks = doc_block::Entity::find()
        .filter(doc_block::Column::PageId.eq(page.id))
        .order_by_asc(doc_block::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    context_update!(request => {
        "title"         => &page.title,
        "section"       => &section,
        "page"          => &page,
        "blocks"        => &blocks,
        "sidebar_pages" => &sidebar_pages,
        "lang"          => lang,
        "section_slug"  => section_slug,
    });

    request.render("docs/doc_page.html")
}

pub async fn doc_section_index(
    lang: &str,
    section_slug: &str,
    request: &mut Request,
) -> AppResult<Response> {
    doc_page(lang, section_slug, "index", request).await
}
