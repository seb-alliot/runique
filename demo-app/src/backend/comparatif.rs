//! The comparison page: Loco, Cot and Django in tabs, then the known limits and
//! what's coming. The Django comparison and the limits are read from the
//! documentation and what's coming from the roadmap files, so each has a single source.
use crate::backend::{inject_globals, lang::visitor_lang, split_markdown_sections};
use crate::entities::{doc_block, doc_page, doc_section};
use runique::prelude::*;

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

pub async fn comparatif_page(request: &mut Request) -> AppResult<Response> {
    inject_globals(request).await;
    let lang = visitor_lang(&request.headers);
    let loco = doc_blocks(lang, "comparatif", "loco", request).await;
    let cot = doc_blocks(lang, "comparatif", "cot", request).await;
    let django = doc_blocks(lang, "comparatif", "index", request).await;
    let limits = doc_blocks(lang, "limites", "index", request).await;
    let roadmap = split_markdown_sections(if lang == "fr" {
        include_str!("../../../ROADMAP.fr.md")
    } else {
        include_str!("../../../ROADMAP.md")
    });
    context_update!(request => {
        "title"   => if lang == "fr" { "Comparatif" } else { "Comparison" },
        "loco"    => &loco,
        "cot"     => &cot,
        "django"  => &django,
        "limits"  => &limits,
        "roadmap" => &roadmap,
    });
    request.render("comparatif/comparatif.html")
}
