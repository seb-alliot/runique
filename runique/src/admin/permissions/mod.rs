pub mod droit;
pub mod groupe;
pub mod groupes_droits;
pub mod users_droits;
pub mod users_groupes;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

// ─────────────────────────────────────────────────────────────────────────────
// Snapshots session — stockés en JSON dans eihwaz_sessions.data
// ─────────────────────────────────────────────────────────────────────────────

/// Droit minimal stocké dans le snapshot session.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Droit {
    pub id: i32,
    pub nom: String,
}

/// Groupe minimal stocké dans le snapshot session (inclut ses droits).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Groupe {
    pub id: i32,
    pub nom: String,
    pub droits: Vec<Droit>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Fonctions de chargement DB — appelées au login
// ─────────────────────────────────────────────────────────────────────────────

/// Charge les droits directs d'un utilisateur depuis la DB.
pub async fn pull_droits_db<C: ConnectionTrait>(db: &C, user_id: i32) -> Vec<Droit> {
    let rows = users_droits::Entity::find()
        .filter(users_droits::Column::UserId.eq(user_id))
        .find_also_related(droit::Entity)
        .all(db)
        .await
        .unwrap_or_default();

    rows.into_iter()
        .filter_map(|(_, d)| {
            d.map(|m| Droit {
                id: m.id,
                nom: m.nom,
            })
        })
        .collect()
}

/// Rafraîchit le cache mémoire des permissions pour un utilisateur donné.
/// Appelé par les signaux SeaORM après toute modification des droits/groupes.
pub async fn refresh_cache_for_user<C: ConnectionTrait>(db: &C, user_id: i32) {
    use crate::middleware::auth::permissions_cache::cache_permissions;
    let droits = pull_droits_db(db, user_id).await;
    let groupes = pull_groupes_db(db, user_id).await;
    cache_permissions(user_id, droits, groupes);
}

/// Charge les groupes d'un utilisateur avec leurs droits depuis la DB.
pub async fn pull_groupes_db<C: ConnectionTrait>(db: &C, user_id: i32) -> Vec<Groupe> {
    let groupe_rows = users_groupes::Entity::find()
        .filter(users_groupes::Column::UserId.eq(user_id))
        .find_also_related(groupe::Entity)
        .all(db)
        .await
        .unwrap_or_default();

    let mut groupes = Vec::new();

    for (_, maybe_groupe) in groupe_rows {
        let Some(g) = maybe_groupe else { continue };

        let droit_rows = groupes_droits::Entity::find()
            .filter(groupes_droits::Column::GroupeId.eq(g.id))
            .find_also_related(droit::Entity)
            .all(db)
            .await
            .unwrap_or_default();

        let droits = droit_rows
            .into_iter()
            .filter_map(|(_, d)| {
                d.map(|m| Droit {
                    id: m.id,
                    nom: m.nom,
                })
            })
            .collect();

        groupes.push(Groupe {
            id: g.id,
            nom: g.nom,
            droits,
        });
    }

    groupes
}
