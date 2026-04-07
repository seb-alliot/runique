
// ============================================================================
// CONSTANTES DE NOMS DE TABLES
// ============================================================================

pub mod table {
    pub const TABLE_EIHWAZ_GROUPES: &str = "eihwaz_groupes";
    pub const TABLE_EIHWAZ_DROITS: &str = "eihwaz_droits";
    pub const TABLE_USERS_GROUPES: &str = "users_groupes";
    pub const TABLE_EIHWAZ_USERS: &str = "eihwaz_users";
}
// ============================================================================
// CONSTANTES DE COLONNES (EIHWAZ_GROUPES)
// ============================================================================

pub mod col_name { 
pub const COL_ID: &str = "id";
pub const COL_NOM: &str = "nom";
}
// ============================================================================
// CONSTANTES DE COLONNES (EIHWAZ_DROITS)
// ============================================================================


pub const COL_GROUPE_ID: &str = "groupe_id";
pub const COL_RESOURCE_KEY: &str = "resource_key";
pub const COL_CAN_CREATE: &str = "can_create";
pub const COL_CAN_READ: &str = "can_read";
pub const COL_CAN_UPDATE: &str = "can_update";
pub const COL_CAN_DELETE: &str = "can_delete";
pub const COL_CAN_UPDATE_OWN: &str = "can_update_own";
pub const COL_CAN_DELETE_OWN: &str = "can_delete_own";

// ============================================================================
// CONSTANTES DE COLONNES (USERS_GROUPES)
// ============================================================================

pub const COL_USER_ID: &str = "user_id";

// ============================================================================
// CONSTANTES DE CLÉS ÉTRANGÈRES
// ============================================================================

pub const FK_DROITS_GROUPE_ID: &str = "fk_eihwaz_droits_groupe_id";
pub const FK_USERS_GROUPES_USER_ID: &str = "fk_users_groupes_user_id";
pub const FK_USERS_GROUPES_GROUPE_ID: &str = "fk_users_groupes_groupe_id";

pub const PK_USERS_GROUPES: &str = "pk_users_groupes";