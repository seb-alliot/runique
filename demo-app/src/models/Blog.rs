use runique::impl_objects;
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "Blog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author_email: String,
    pub website: String,
    pub content: String,
    pub summary: String,
    pub created_at: DateTime,
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
