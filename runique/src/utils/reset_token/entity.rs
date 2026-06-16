//! SeaORM entity for the `eihwaz_reset_tokens` table.
//!
//! Stores **hashed** password-reset tokens (never the raw token). The raw token
//! lives only in the email link; the server hashes the incoming token and looks
//! up the row, so a DB read leak cannot be replayed.
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "eihwaz_reset_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// SHA-256 hex digest of the raw token (the lookup key).
    #[sea_orm(unique)]
    pub token_hash: String,

    /// FK → user table: the authoritative target of the reset.
    pub user_id: crate::utils::pk::Pk,

    /// Expiration timestamp (UTC naive).
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::auth::user::Entity",
        from = "Column::UserId",
        to = "crate::auth::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::auth::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
