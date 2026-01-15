use runique::impl_objects;
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, DeriveModelForm)]
#[sea_orm(table_name = "test_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    // Texte
    pub phone: String,
    pub color: String,
    pub uuid: String,
    pub postal_code: String,
    pub description: String,

    // Numérique
    pub price: f64,
    pub rating: i32,
    pub quantity: i32,
    pub discount: f64,
    pub amount: String,

    // Temporel
    pub opening_time: String,
    pub duration: i64,

    // Fichiers
    pub profile_image: String,
    pub attachments: String, // JSON string ou texte

    // Choix
    pub preferences: String, // JSON array string
    pub subscription: String,

}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
