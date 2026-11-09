//! The comparison pages. The hub (`/comparatif`) links to one page per
//! framework. Each comparison has its own URL, title and description
//! (`/comparatif/runique-vs-django`…), what a search for "runique vs django"
//! lands on. The comparisons are read from the documentation and what's coming
//! from the roadmap files, so each has a single source.
use crate::backend::{inject_globals, lang::visitor_lang, split_markdown_sections};
use crate::entities::{doc_block, doc_page, doc_section};
use runique::prelude::*;

/// The comparisons that have a page of their own: slug in the URL, the other
/// framework's name, its page in the documentation (`comparatif-{page}`).
const RIVALS: [(&str, &str, &str); 3] = [
    ("runique-vs-django", "Django", "index"),
    ("runique-vs-loco", "Loco", "loco"),
    ("runique-vs-cot", "Cot", "cot"),
];

/// Slug and name of each comparison, for the links between the pages.
fn comparative_links() -> Vec<(&'static str, &'static str)> {
    RIVALS.iter().map(|r| (r.0, r.1)).collect()
}

async fn doc_blocks(
    lang: &str,
    section_slug: &str,
    page: &str,
    request: &Request,
) -> Vec<doc_block::Model> {
    let db = request.engine.db.clone();
    let Ok(Some(section)) = search!(doc_section::Entity => Lang eq lang, Slug eq section_slug)
        .first(&db)
        .await
    else {
        return Vec::new();
    };
    let page_slug = format!("{section_slug}-{page}");
    let Ok(Some(page)) = search!(doc_page::Entity => SectionId eq section.id, Slug eq page_slug)
        .first(&db)
        .await
    else {
        return Vec::new();
    };
    search!(doc_block::Entity => PageId eq page.id, asc SortOrder)
        .all(&db)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|block| block.block_type != doc_block::BlockType::Sommaire)
        .collect()
}

/// The comparisons' introduction: Django, Loco, Cot and Runique as
/// alternatives, one card each leading to its detailed comparison.
pub async fn comparatif_intro_page(request: &mut Request) -> AppResult<Response> {
    inject_globals(request).await;
    let lang = visitor_lang(&request.headers);
    context_update!(request => {
        "title" => if lang == "fr" {
            "Choisir un framework web : Django, Loco, Cot et Runique"
        } else {
            "Choosing a web framework: Django, Loco, Cot and Runique"
        },
    });
    request.render("comparatif/index.html")
}

/// What's coming (the roadmap), with a link to the known limits of the
/// documentation: the comparisons have their own pages, `/comparatif/runique-vs-…`.
pub async fn limites_page(request: &mut Request) -> AppResult<Response> {
    inject_globals(request).await;
    let lang = visitor_lang(&request.headers);
    // The limits are only linked: their text lives at /docs/{lang}/limites, a
    // copy here would be the same content under two URLs.
    let roadmap = split_markdown_sections(if lang == "fr" {
        include_str!("../../../ROADMAP.fr.md")
    } else {
        include_str!("../../../ROADMAP.md")
    });
    context_update!(request => {
        "title"   => if lang == "fr" { "Limites et feuille de route" } else { "Limits and roadmap" },
        "limits_url" => format!("/docs/{lang}/limites"),
        "roadmap" => &roadmap,
    });
    request.render("comparatif/limites.html")
}

/// One comparison on its own page. An unknown slug, or a comparison missing
/// from the documentation, is a 404: no empty page indexed under its name.
pub async fn comparatif_detail_page(slug: &str, request: &mut Request) -> AppResult<Response> {
    let Some(&(slug, rival, page)) = RIVALS.iter().find(|r| r.0 == slug) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    inject_globals(request).await;
    let lang = visitor_lang(&request.headers);
    let blocks = doc_blocks(lang, "comparatif", page, request).await;
    if blocks.is_empty() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let description = if lang == "fr" {
        format!("Runique et {rival} côte à côte : fonctionnalités, choix de conception et limites.")
    } else {
        format!("Runique and {rival} side by side: features, design choices and limits.")
    };
    let title = format!("Runique vs {rival}");
    let canonical = format!("/comparatif/{slug}");
    context_update!(request => {
        "title"       => &title,
        "description" => &description,
        "canonical"   => &canonical,
        "rival"       => rival,
        "rival_slug"  => slug,
        "rivals"      => &comparative_links(),
        "blocks"      => &blocks,
    });
    request.render("comparatif/detail.html")
}
