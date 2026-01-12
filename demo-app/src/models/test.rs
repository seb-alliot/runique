// models/test_fields.rs
use runique::prelude::*;
use runique::sea_orm;
use runique::sea_orm::entity::prelude::*;
use runique::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveModelForm, Serialize, Deserialize)]
#[sea_orm(table_name = "test_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    // Texte avancé
    pub phone: String,       // PhoneField
    pub color: String,       // ColorField
    pub uuid: String,        // UUIDField
    pub postal_code: String, // PostalCodeField
    pub description: String, // TextField

    // Numérique avancé
    pub price: String,  // DecimalField
    pub rating: i64,    // RangeField (1-5)
    pub quantity: i64,  // PositiveIntegerField
    pub discount: f64,  // PercentageField
    pub amount: String, // CurrencyField

    // Temporel avancé
    pub opening_time: String, // TimeField
    pub duration: i64,        // DurationField (en secondes)

    // Fichiers
    pub image: String,       // ImageField
    pub attachments: String, // MultipleFileField (stocké en JSON)

    // Choix
    pub preferences: String,  // MultipleChoiceField (stocké en JSON)
    pub subscription: String, // RadioSelectField

    pub created_at: DateTime, // DateTime
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
