use runique::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_fields")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub username: String,
    pub email: String,
    pub website: String,
    pub description: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}
