use runique::impl_objects;
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "blog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub email: String,      // Email de l'auteur
    pub website: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub summary: String,    // Pour le TextArea
    #[sea_orm(column_type = "Text")]
    pub content: String,    // Pour le RichText
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        }
    }
}


impl_objects!(Entity);