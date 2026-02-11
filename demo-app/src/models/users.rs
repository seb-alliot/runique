use runique::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,

    // ── Droits d'accès ──────────────────────────────────────────────────
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,

    // ── Rôles (JSON) ─────────────────────────────────────────────────────
    // Stocké en base sous forme de texte JSON : ["editor","moderator"]
    pub roles: Option<String>,

    // ── Timestamps ───────────────────────────────────────────────────────
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Model {
    /// Retourne les rôles désérialisés depuis le JSON
    ///
    /// Retourne un Vec vide si le champ est NULL ou invalide.
    pub fn get_roles(&self) -> Vec<String> {
        self.roles
            .as_deref()
            .and_then(|r| serde_json::from_str(r).ok())
            .unwrap_or_default()
    }

    /// Vérifie si le compte peut accéder à l'admin
    pub fn can_access_admin(&self) -> bool {
        self.is_active && (self.is_staff || self.is_superuser)
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
