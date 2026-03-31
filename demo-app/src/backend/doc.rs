use crate::entities::{cour, doc_block, doc_page, doc_section};
use runique::prelude::*;

pub async fn doc_index(lang: &str, request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    let db = request.engine.db.clone();

    let sections = search!(doc_section::Entity => Lang eq lang, asc SortOrder)
        .all(&db)
        .await
        .unwrap_or_default();

    let mut cours_difficultes: Vec<String> = search!(cour::Entity => Lang eq lang)
        .into_select()
        .select_only()
        .column(cour::Column::Difficulte)
        .distinct()
        .into_tuple::<String>()
        .all(db.as_ref())
        .await
        .unwrap_or_default();

    let order = ["debutant", "intermediaire", "avance", "specifique"];
    cours_difficultes.sort_by_key(|d| order.iter().position(|&o| o == d.as_str()).unwrap_or(99));

    let cours = search!(cour::Entity => Lang eq lang, asc SortOrder)
        .all(&db)
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
    crate::backend::inject_globals(request).await;
    let db = request.engine.db.clone();

    let section = search!(doc_section::Entity => Lang eq lang, Slug eq section_slug)
        .first(&db)
        .await
        .unwrap_or(None);

    let Some(section) = section else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let sidebar_pages =
        search!(doc_page::Entity => SectionId eq section.id, Lang eq lang, asc SortOrder)
            .all(&db)
            .await
            .unwrap_or_default();

    let full_slug = format!("{section_slug}-{page_slug}");
    let page = search!(doc_page::Entity => SectionId eq section.id, Slug eq full_slug)
        .first(&db)
        .await
        .unwrap_or(None);

    let Some(page) = page else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let blocks = search!(doc_block::Entity => PageId eq page.id, asc SortOrder)
        .all(&db)
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
