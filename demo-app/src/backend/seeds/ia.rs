use crate::entities::{chapitre, contrainte_ia, cour, cour_block, cour_ia};
use runique::prelude::*;
use std::fs;
use std::path::PathBuf;

fn find_ia_dir() -> Option<PathBuf> {
    let candidates = [
        "docs/ia",
        "demo-app/docs/fr/ia",
        "../docs/fr/ia",
        "../../docs/ia",
        "/app/docs/ia",
    ];
    for candidate in &candidates {
        let p = PathBuf::from(candidate);
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

async fn build_context(cour_id: i32, db: &DatabaseConnection) -> String {
    let chapitres = chapitre::Entity::find()
        .filter(chapitre::Column::CourId.eq(cour_id))
        .order_by_asc(chapitre::Column::SortOrder)
        .all(db)
        .await
        .unwrap_or_default();

    let mut context = String::new();

    for chap in chapitres {
        context.push_str(&format!("## {}\n\n", chap.title));

        let blocs = cour_block::Entity::find()
            .filter(cour_block::Column::ChapitreId.eq(chap.id))
            .order_by_asc(cour_block::Column::SortOrder)
            .all(db)
            .await
            .unwrap_or_default();

        for bloc in blocs {
            if let Some(heading) = &bloc.heading {
                context.push_str(&format!("### {}\n\n", heading));
            }
            context.push_str(&bloc.content);
            context.push_str("\n\n");
        }
    }

    context.trim().to_string()
}

pub async fn seed_ia(db: &DatabaseConnection) {
    tracing::info!("fichier ia: démarrage");

    let ia_dir = if let Some(p) = find_ia_dir() {
        p
    } else {
        tracing::warn!("fichier ia: dossier docs/ia/ introuvable, seed ignoré");
        return;
    };

    // Nettoyage (ordre FK : cour_ia → contrainte_ia)
    let stmts = ["DELETE FROM cour_ia", "DELETE FROM contrainte_ia"];
    for sql in &stmts {
        if let Err(e) = db.execute_unprepared(sql).await {
            tracing::warn!("fichier ia: erreur nettoyage ({sql}): {e}");
            return;
        }
    }

    // Lecture du system prompt français
    let contrainte_path = ia_dir.join("contrainte_fr.md");
    let contrainte_text = if let Ok(c) = fs::read_to_string(&contrainte_path) {
        c
    } else {
        tracing::warn!("fichier ia: contrainte_fr.md introuvable");
        return;
    };

    let contrainte_model = contrainte_ia::ActiveModel {
        contrainte_ia: Set(contrainte_text.trim().to_string()),
        lang: Set("fr".to_string()),
        ..Default::default()
    };

    let inserted_contrainte = match contrainte_model.insert(db).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("fichier ia: erreur insertion contrainte_ia: {e}");
            return;
        }
    };

    tracing::info!(
        "fichier ia: contrainte_ia insérée (id={})",
        inserted_contrainte.id
    );

    // Création des CourIa pour chaque cours français
    let cours = cour::Entity::find()
        .filter(cour::Column::Lang.eq("fr"))
        .order_by_asc(cour::Column::Ordre)
        .all(db)
        .await
        .unwrap_or_default();

    for (i, c) in cours.iter().enumerate() {
        let context = build_context(c.id, db).await;

        if context.is_empty() {
            tracing::warn!(
                "fichier ia: contexte vide pour '{}', entrée ignorée",
                c.slug
            );
            continue;
        }

        let cour_ia_model = cour_ia::ActiveModel {
            context: Set(context),
            contraintes: Set(String::new()),
            contrainte_id: Set(inserted_contrainte.id),
            cour_id: Set(c.id),
            sort_order: Set(i as i32 + 1),
            ..Default::default()
        };

        match cour_ia_model.insert(db).await {
            Ok(_) => tracing::info!("fichier ia: cour_ia seedé — {}", c.slug),
            Err(e) => tracing::warn!("fichier ia: erreur insertion cour_ia '{}': {e}", c.slug),
        }
    }

    tracing::info!("fichier ia: terminé");
}
