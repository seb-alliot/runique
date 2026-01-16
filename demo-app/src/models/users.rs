use runique::impl_objects; // Assure-toi que l'import est correct
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub username: String,

    #[sea_orm(unique)]
    pub email: String,

    pub password: String,

    pub created_at: DateTime,
    /* --- Champs additionnels possibles ---
    pub age: i32,
    */
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    // Optionnel : Générer automatiquement la date à la création
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        }
    }
}

impl_objects!(Entity);
