// Tests pour le type de colonne PK/FK des tables internes RBAC (eihwaz_*) —
// doit suivre `big-pk`/`pk-uuid`, cf. admin/table_admin/migrations_table.rs.

#[cfg(feature = "pk-uuid")]
use runique::admin::{
    create_eihwaz_history_table, create_eihwaz_reset_tokens_table, create_eihwaz_sessions_table,
};
use runique::admin::{create_eihwaz_users_groupes_table, create_eihwaz_users_table};
use sea_query::ColumnType;

fn col_type(stmt: &sea_query::TableCreateStatement, col_name: &str) -> ColumnType {
    stmt.get_columns()
        .iter()
        .find(|c| c.get_column_name() == col_name)
        .unwrap_or_else(|| panic!("colonne `{col_name}` absente"))
        .get_column_type()
        .unwrap_or_else(|| panic!("colonne `{col_name}` sans type"))
        .clone()
}

// ═══════════════════════════════════════════════════════════════
// Défaut (ni big-pk, ni pk-uuid) — i32
// ═══════════════════════════════════════════════════════════════

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
mod default_i32 {
    use super::*;

    #[test]
    fn test_eihwaz_users_pk_is_integer() {
        assert!(matches!(
            col_type(&create_eihwaz_users_table(), "id"),
            ColumnType::Integer
        ));
    }

    #[test]
    fn test_eihwaz_users_groupes_user_id_is_integer() {
        assert!(matches!(
            col_type(&create_eihwaz_users_groupes_table(), "user_id"),
            ColumnType::Integer
        ));
    }
}

// ═══════════════════════════════════════════════════════════════
// pk-uuid — la PK et toutes les FK user_id doivent être Uuid
// ═══════════════════════════════════════════════════════════════

#[cfg(feature = "pk-uuid")]
mod pk_uuid {
    use super::*;

    #[test]
    fn test_eihwaz_users_pk_is_uuid() {
        assert!(matches!(
            col_type(&create_eihwaz_users_table(), "id"),
            ColumnType::Uuid
        ));
    }

    #[test]
    fn test_eihwaz_users_groupes_user_id_is_uuid() {
        assert!(matches!(
            col_type(&create_eihwaz_users_groupes_table(), "user_id"),
            ColumnType::Uuid
        ));
    }

    #[test]
    fn test_eihwaz_sessions_user_id_is_uuid() {
        assert!(matches!(
            col_type(&create_eihwaz_sessions_table(), "user_id"),
            ColumnType::Uuid
        ));
    }

    #[test]
    fn test_eihwaz_history_user_id_is_uuid() {
        assert!(matches!(
            col_type(&create_eihwaz_history_table(), "user_id"),
            ColumnType::Uuid
        ));
    }

    #[test]
    fn test_eihwaz_reset_tokens_user_id_is_uuid() {
        assert!(matches!(
            col_type(&create_eihwaz_reset_tokens_table(), "user_id"),
            ColumnType::Uuid
        ));
    }

    // eihwaz_history garde son propre id (l'historique) en i64 fixe, volontairement
    // indépendant de `Pk` — ne doit jamais devenir Uuid même sous pk-uuid.
    #[test]
    fn test_eihwaz_history_own_id_stays_big_integer() {
        assert!(matches!(
            col_type(&create_eihwaz_history_table(), "id"),
            ColumnType::BigInteger
        ));
    }
}

// ═══════════════════════════════════════════════════════════════
// big-pk — PK et FK doivent rester BigInteger (non-régression)
// ═══════════════════════════════════════════════════════════════

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
mod big_pk {
    use super::*;

    #[test]
    fn test_eihwaz_users_pk_is_big_integer() {
        assert!(matches!(
            col_type(&create_eihwaz_users_table(), "id"),
            ColumnType::BigInteger
        ));
    }

    #[test]
    fn test_eihwaz_users_groupes_user_id_is_big_integer() {
        assert!(matches!(
            col_type(&create_eihwaz_users_groupes_table(), "user_id"),
            ColumnType::BigInteger
        ));
    }
}
