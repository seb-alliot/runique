use crate::entities::{chapitre, cour, cour_block};
use runique::prelude::*;

pub async fn cours_index(request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let db = request.engine.db.clone();

    let cours = cour::Entity::find()
        .filter(cour::Column::Lang.eq("fr"))
        .order_by_asc(cour::Column::Ordre)
        .all(&*db)
        .await
        .unwrap_or_default();

    let niveaux = vec!["debutant", "intermediaire", "avance", "specifique"];

    context_update!(request => {
        "title"   => "Cours Rust",
        "cours"   => &cours,
        "niveaux" => &niveaux,
    });

    request.render("cours/cours_index.html")
}

pub async fn cours_detail(slug: &str, request: &mut Request) -> AppResult<Response> {
    crate::backend::inject_auth(request).await;
    let db = request.engine.db.clone();

    let Some(cour) = cour::Entity::find()
        .filter(cour::Column::Slug.eq(slug))
        .one(&*db)
        .await
        .unwrap_or(None)
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let chapitres = chapitre::Entity::find()
        .filter(chapitre::Column::CourId.eq(cour.id))
        .order_by_asc(chapitre::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    let chapitre_ids: Vec<i32> = chapitres.iter().map(|c| c.id).collect();

    let blocs = cour_block::Entity::find()
        .filter(cour_block::Column::ChapitreId.is_in(chapitre_ids))
        .order_by_asc(cour_block::Column::SortOrder)
        .all(&*db)
        .await
        .unwrap_or_default();

    context_update!(request => {
        "title"     => &cour.title,
        "cour"      => &cour,
        "chapitres" => &chapitres,
        "blocs"     => &blocs,
    });

    request.render("cours/cours_detail.html")
}
