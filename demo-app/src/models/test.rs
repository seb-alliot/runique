use runique::impl_objects;
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    // --- Champs implémentés (utilisables avec tes nouveaux FormFields) ---
    pub username: String,    // Match TextField
    pub email: String,       // Match EmailField
    pub website: String,     // Match URLField
    pub description: String, // Match TextAreaField / RichTextField
    pub password: String,    // Match PasswordField (stockage du hash)
    pub created_at: DateTime,
    /* --- Champs non encore implantés ou en attente de migration ---
    pub phone: String,
    pub color: String,
    pub uuid: String,
    pub postal_code: String,

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
    pub attachments: String,

    // Choix
    pub preferences: String,
    pub subscription: String,
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
